//! Dreaming: a nightly sweep that consolidates the daily notes into long-term memory in three phases, light, REM
//! and deep, then writes a diary entry and opens a context with the new memory (dreaming, DRM-1 to DRM-8).

mod commit;
mod deep;
mod diary;
mod light;
mod rem;
mod run;
mod schedule;
mod topics;

use crate::chat::context::{self, text_block};
use crate::chat::transcript::Entry;
use crate::chat::{LocalClock, local_today, lock, now_ms};
use crate::compaction::{Ask, CompactOutcome, call};
use crate::config::now;
use crate::credentials::ApiKey;
use crate::llm::{Effort, LlmProvider, ProviderError, Reply, TurnRequest, effort_for};
use crate::memory::files::{daily_dates, previous_day};
use crate::{AppError, Core, Result};
use run::{Candidate, Drawing, Phase, Run};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use tokio::sync::Notify;
use ts_rs::TS;

/// How long the person must be idle before a due night runs (dreaming R3).
pub(crate) const IDLE_MS: u64 = 5 * 60 * 1000;
/// The first sweep looks back this many days of notes.
const FIRST_LOOKBACK_DAYS: usize = 7;
/// Two lines are one candidate when their terms overlap this much (dreaming R5).
pub(crate) const DEDUP_OVERLAP: f64 = 0.8;
pub(crate) const MAX_THEMES: usize = 5;
/// The gates (dreaming R7).
pub(crate) const MIN_SCORE: f64 = 0.6;
pub(crate) const MIN_RECALLS: usize = 2;
pub(crate) const MIN_QUERIES: usize = 2;
/// A rewrite may remove at most this share of the prior entries (architecture 15).
pub(crate) const MAX_LOSS: f64 = 0.25;
pub(crate) const MAX_ENTRY_CHARS: usize = 600;
/// A failed run is retried no sooner than this, in seconds (dreaming R4).
const RETRY_AFTER_S: i64 = 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub enum DreamState {
    Idle,
    Running,
    /// A message came in; the sweep resumes when the person is idle again.
    Paused,
}

/// One sweep, for the dream journal (dreaming FR-022).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DreamRun {
    #[ts(type = "number")]
    pub id: i64,
    pub night: String,
    /// Unix seconds.
    #[ts(type = "number")]
    pub started_at: i64,
    #[ts(type = "number | null")]
    pub ended_at: Option<i64>,
    pub completed: bool,
    pub added: u32,
    pub merged: u32,
    pub superseded: u32,
    pub fell_back: bool,
    #[ts(type = "number")]
    pub tokens: u64,
    pub undone: bool,
}

impl From<&Run> for DreamRun {
    fn from(run: &Run) -> Self {
        let tokens = run
            .usage
            .values()
            .map(crate::chat::reply::context_tokens)
            .sum();
        Self {
            id: run.id,
            night: run.night.clone(),
            started_at: run.started_at,
            ended_at: run.ended_at,
            completed: run.phase == Phase::Done,
            added: run.added,
            merged: run.merged,
            superseded: run.superseded,
            fell_back: run.fell_back,
            tokens,
            undone: run.undone,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DreamStatus {
    pub state: DreamState,
    pub last_run: Option<DreamRun>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DreamJournal {
    /// `DREAMS.md` as it is, or empty.
    pub diary: String,
    /// The newest first.
    pub runs: Vec<DreamRun>,
    pub can_undo: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum UndoDreamOutcome {
    Undone,
    NothingToUndo,
    /// These files changed since the dream; undo again with `force` to discard the changes.
    Changed {
        files: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DreamingSettings {
    pub enabled: bool,
    /// `HH:MM`, local time.
    pub time: String,
    /// None: the chat model.
    pub model: Option<String>,
    /// The chosen model is no longer offered; the chat model is used instead.
    pub model_missing: bool,
    /// journey FR-008: the day's topics are named and drawn in the world.
    pub world: bool,
}

/// Clears the dreaming slot however a sweep ends.
struct Dreaming<'a> {
    core: &'a Core,
    stop: Arc<Notify>,
}

impl Drop for Dreaming<'_> {
    fn drop(&mut self) {
        *lock(&self.core.chat.dreaming) = None;
        self.core.avatar.dreaming(false);
    }
}

/// What a phase leaves the sweep to do next.
enum Step {
    Next,
    /// A message came in.
    Paused,
    /// The provider refused the key or has no credit: nothing changed, try again later.
    Failed,
    /// The roll could not open a context yet; the next due tick tries again.
    Later,
}

/// The outcome of one model call in a phase.
enum Answer {
    Reply(Reply),
    /// Unavailable, refused or unusable: the phase falls back.
    Unavailable,
    Stopped,
    Fatal,
}

impl Core {
    /// dreaming FR-001 to FR-005: called by the main view once a minute with the person's clock. Runs or resumes a due
    /// night, and returns when it is done or paused.
    pub async fn dream_tick(
        &self,
        provider: &impl LlmProvider,
        clock: &LocalClock,
    ) -> Result<DreamStatus> {
        context::time_block(clock)?;
        if lock(&self.chat.dreaming).is_some() {
            return self.dream_status(DreamState::Running);
        }
        let Some(key) = self.secrets.get() else {
            return self.dream_status(DreamState::Idle);
        };
        let settings = self.settings()?;
        let local = context::local_now(clock);
        let night = schedule::latest_night(&local.date, &local.time, &settings.dreaming.time);
        let busy = lock(&self.chat.active).is_some() || lock(&self.chat.compacting).is_some();
        let active_ms = self
            .transcript(|t| Ok(t.lines.last().map_or(0, |l| l.at)))?
            .max(self.started);
        let now_ms = schedule::unix_ms(&clock.iso).unwrap_or_else(now_ms);
        let unfinished = self.unfinished_run()?;
        let failed_recently = self.runs(1)?.first().is_some_and(|r| {
            r.phase == Phase::Failed
                && r.night == night
                && r.ended_at
                    .is_some_and(|at| now() as i64 - at < RETRY_AFTER_S)
        });
        let last_night = self.state()?.dreaming.last_night;
        let moment = schedule::Moment {
            night: &night,
            last_night: last_night.as_deref(),
            now_ms,
            active_ms,
            busy,
        };
        let due = settings.dreaming.enabled && !failed_recently && schedule::due(&moment);
        let resume = unfinished.is_some()
            && schedule::due(&schedule::Moment {
                last_night: None,
                ..moment
            });
        if !due && !resume {
            return self.dream_status(DreamState::Idle);
        }
        let night = unfinished.as_ref().map_or(night, |r| r.night.clone());
        self.sweep(provider, &key, clock, &night, unfinished).await
    }

    /// dreaming FR-027: a sweep now over everything since the last completed one, whether dreaming is on or off.
    pub async fn dream_now(
        &self,
        provider: &impl LlmProvider,
        clock: &LocalClock,
    ) -> Result<DreamStatus> {
        context::time_block(clock)?;
        if lock(&self.chat.active).is_some() || lock(&self.chat.compacting).is_some() {
            return Err(AppError::invalid("busy"));
        }
        let key = self.secrets.get().ok_or_else(|| AppError::invalid("key"))?;
        let local = context::local_now(clock);
        let night =
            schedule::latest_night(&local.date, &local.time, &self.settings()?.dreaming.time);
        let unfinished = self.unfinished_run()?;
        let night = unfinished.as_ref().map_or(night, |r| r.night.clone());
        self.sweep(provider, &key, clock, &night, unfinished).await
    }

    fn dream_status(&self, state: DreamState) -> Result<DreamStatus> {
        Ok(DreamStatus {
            state,
            last_run: self.runs(1)?.first().map(DreamRun::from),
        })
    }

    pub fn dream_journal(&self) -> Result<DreamJournal> {
        let diary =
            std::fs::read_to_string(self.workspace_dir().join("DREAMS.md")).unwrap_or_default();
        let runs = self.runs(30)?;
        let can_undo = self.runs(1_000)?.iter().any(|r| r.committed && !r.undone);
        Ok(DreamJournal {
            diary,
            runs: runs.iter().map(DreamRun::from).collect(),
            can_undo,
        })
    }

    pub fn dreaming_settings(&self) -> Result<DreamingSettings> {
        let settings = self.settings()?;
        let cached = self.state()?.models.items;
        let model_missing = settings
            .dreaming
            .model
            .as_ref()
            .is_some_and(|m| !cached.iter().any(|c| &c.id == m));
        Ok(DreamingSettings {
            enabled: settings.dreaming.enabled,
            time: settings.dreaming.time,
            model: settings.dreaming.model,
            model_missing,
            world: settings.dreaming.world,
        })
    }

    /// dreaming FR-026. Turning dreaming on starts from the latest night, so nights it was off are not caught up
    /// (dreaming R3).
    pub fn set_dreaming_settings(
        &self,
        enabled: bool,
        time: &str,
        model: Option<String>,
        world: bool,
        clock: &LocalClock,
    ) -> Result<DreamingSettings> {
        context::time_block(clock)?;
        if !schedule::is_time(time) {
            return Err(AppError::invalid("time"));
        }
        let cached = self.state()?.models.items;
        if model
            .as_ref()
            .is_some_and(|m| !cached.iter().any(|c| &c.id == m))
        {
            return Err(AppError::invalid("model"));
        }
        let mut settings = self.settings()?;
        let turned_on = enabled && !settings.dreaming.enabled;
        settings.dreaming.enabled = enabled;
        settings.dreaming.time = time.into();
        settings.dreaming.model = model;
        settings.dreaming.world = world;
        self.save_settings(&settings)?;
        if turned_on {
            let local = context::local_now(clock);
            let night = schedule::latest_night(&local.date, &local.time, time);
            self.update_state(|s| s.dreaming.last_night = Some(night))?;
        }
        self.dreaming_settings()
    }

    /// Takes the dreaming slot: refused while a reply, a compaction or another sweep runs.
    fn begin_dreaming(&self) -> Result<Dreaming<'_>> {
        let active = lock(&self.chat.active);
        let compacting = lock(&self.chat.compacting);
        let mut slot = lock(&self.chat.dreaming);
        if active.is_some() || compacting.is_some() || slot.is_some() {
            return Err(AppError::invalid("busy"));
        }
        let stop = Arc::new(Notify::new());
        *slot = Some(stop.clone());
        // avatar FR-004: asleep while the sweep runs; the guard wakes it however the sweep ends.
        self.avatar.dreaming(true);
        Ok(Dreaming { core: self, stop })
    }

    /// The sweep: each phase reads the checkpoint, does its work and saves the next phase (dreaming R4).
    async fn sweep(
        &self,
        provider: &impl LlmProvider,
        key: &ApiKey,
        clock: &LocalClock,
        night: &str,
        resume: Option<Run>,
    ) -> Result<DreamStatus> {
        let guard = self.begin_dreaming()?;
        let stop = guard.stop.clone();
        let mut run = match resume {
            Some(run) => run,
            None => {
                let since = self
                    .last_completed_run()?
                    .and_then(|r| r.ended_at)
                    .unwrap_or(now() as i64 - FIRST_LOOKBACK_DAYS as i64 * 86_400);
                self.start_run(night, since)?
            }
        };
        if run.phase == Phase::Committing {
            self.roll_back(&mut run)?;
        }
        loop {
            let step = match run.phase {
                Phase::Light => self.light_phase(&mut run).map(|()| Step::Next)?,
                Phase::Rem => self.rem_phase(&mut run, provider, key, &stop).await?,
                Phase::Deep => self.deep_phase(&mut run, provider, key, &stop).await?,
                Phase::Committing => {
                    self.roll_back(&mut run)?;
                    Step::Next
                }
                Phase::Diary => self.diary_phase(&mut run, provider, key, &stop).await?,
                Phase::World => {
                    self.world_phase(&mut run, provider, key, clock, &stop)
                        .await?
                }
                Phase::Drawing => self.drawing_phase(&mut run)?,
                Phase::Roll => {
                    self.roll_phase(&mut run, provider, key, clock, &stop)
                        .await?
                }
                Phase::Done | Phase::Failed => break,
            };
            match step {
                Step::Next => {}
                Step::Paused => {
                    drop(guard);
                    return self.dream_status(DreamState::Paused);
                }
                Step::Later => break,
                Step::Failed => {
                    run.phase = Phase::Failed;
                    run.ended_at = Some(now() as i64);
                    self.save_run(&run)?;
                    break;
                }
            }
        }
        if run.phase == Phase::Done {
            run.ended_at.get_or_insert(now() as i64);
            self.save_run(&run)?;
            let night = run.night.clone();
            self.update_state(|s| {
                if s.dreaming
                    .last_night
                    .as_deref()
                    .is_none_or(|last| night.as_str() > last)
                {
                    s.dreaming.last_night = Some(night);
                }
            })?;
        }
        drop(guard);
        self.dream_status(DreamState::Idle)
    }

    /// The dreaming model when it is still offered, otherwise the chat model (dreaming FR-026).
    fn dream_request(
        &self,
        system: Vec<String>,
        user: &str,
        effort: Effort,
    ) -> Result<TurnRequest> {
        let settings = self.settings()?;
        let cached = self.state()?.models.items;
        let model = settings
            .dreaming
            .model
            .filter(|m| cached.iter().any(|c| &c.id == m))
            .unwrap_or(settings.chat.model);
        Ok(TurnRequest {
            effort: effort_for(&cached, &model, effort),
            system,
            messages: vec![serde_json::json!({ "role": "user", "content": [text_block(user)] })],
            tools: Vec::new(),
            format: None,
            model,
        })
    }

    async fn ask(
        &self,
        provider: &impl LlmProvider,
        key: &ApiKey,
        request: &TurnRequest,
        stop: &Notify,
        run: &mut Run,
        name: &str,
    ) -> Answer {
        match call(provider, key, request, Some(stop)).await {
            None => Answer::Stopped,
            Some(Err(failure)) => match failure.error {
                ProviderError::KeyRejected | ProviderError::NoCredit => Answer::Fatal,
                _ => Answer::Unavailable,
            },
            Some(Ok(reply)) => {
                if let Some(usage) = &reply.usage {
                    run.usage.insert(name.into(), usage.clone());
                }
                if reply.stop_reason.as_deref() == Some("refusal") {
                    Answer::Unavailable
                } else {
                    Answer::Reply(reply)
                }
            }
        }
    }

    /// Light: note lines since the last sweep and older ones recalled since, and the person's messages (dreaming R5).
    fn light_phase(&self, run: &mut Run) -> Result<()> {
        let from_day = match self.state()?.dreaming.last_night {
            Some(night) => night,
            None => (0..FIRST_LOOKBACK_DAYS).fold(run.night.clone(), |d, _| previous_day(&d)),
        };
        let recalled: HashSet<(String, String)> = self
            .recalls()?
            .into_iter()
            .filter(|r| r.at >= run.since)
            .map(|r| (r.path, r.text_hash))
            .collect();
        let workspace = self.workspace_dir();
        let notes: BTreeMap<String, String> = daily_dates(&workspace)
            .into_iter()
            .filter(|d| {
                d.as_str() >= from_day.as_str()
                    || recalled.iter().any(|(p, _)| p == &format!("memory/{d}.md"))
            })
            .filter_map(|d| {
                let text =
                    std::fs::read_to_string(workspace.join(format!("memory/{d}.md"))).ok()?;
                Some((d, text))
            })
            .collect();
        let since_ms = run.since.max(0) as u64 * 1000;
        let (forgotten, said) = self.transcript(|t| {
            let forgotten: HashSet<String> = context::forgotten(&t.lines)
                .into_iter()
                .map(Into::into)
                .collect();
            let refs: HashSet<&str> = forgotten.iter().map(String::as_str).collect();
            let said = light::said(&t.lines, since_ms, &refs);
            Ok((forgotten, said))
        })?;
        let refs: HashSet<&str> = forgotten.iter().map(String::as_str).collect();
        run.checkpoint.candidates = light::gather(&notes, &from_day, &recalled, &refs);
        run.checkpoint.messages = said;
        run.phase = Phase::Rem;
        self.save_run(run)
    }

    /// REM: themes across days, reinforcing the candidates they group (dreaming R6).
    async fn rem_phase(
        &self,
        run: &mut Run,
        provider: &impl LlmProvider,
        key: &ApiKey,
        stop: &Notify,
    ) -> Result<Step> {
        let cp = &run.checkpoint;
        if !cp.candidates.is_empty() || !cp.messages.is_empty() {
            let mut request = self.dream_request(
                vec![rem::INSTRUCTIONS.into()],
                &rem::input(&cp.candidates, &cp.messages),
                Effort::Low,
            )?;
            request.format = Some(rem::schema());
            match self.ask(provider, key, &request, stop, run, "rem").await {
                Answer::Stopped => return Ok(Step::Paused),
                Answer::Fatal => return Ok(Step::Failed),
                Answer::Unavailable => {}
                Answer::Reply(reply) => {
                    if let Some(themes) = rem::parse(&reply.content, &run.checkpoint.candidates) {
                        rem::reinforce(&mut run.checkpoint.candidates, &themes);
                        run.checkpoint.themes = themes;
                    }
                }
            }
        }
        run.phase = Phase::Deep;
        self.save_run(run)?;
        Ok(Step::Next)
    }

    /// Deep: score, gate, re-read, decide, validate and commit (dreaming R7, R8).
    async fn deep_phase(
        &self,
        run: &mut Run,
        provider: &impl LlmProvider,
        key: &ApiKey,
        stop: &Notify,
    ) -> Result<Step> {
        let workspace = self.workspace_dir();
        let read = |rel: &str| std::fs::read_to_string(workspace.join(rel)).unwrap_or_default();
        let (memory, profile) = (read("MEMORY.md"), read("USER.md"));
        let recalls = self.recalls()?;
        let paths: HashSet<&str> = run
            .checkpoint
            .candidates
            .iter()
            .flat_map(|c| c.sources.iter().map(|s| s.path.as_str()))
            .collect();
        let notes: BTreeMap<String, String> =
            paths.iter().map(|p| ((*p).to_owned(), read(p))).collect();
        let gated: Vec<Candidate> = run
            .checkpoint
            .candidates
            .iter()
            .filter(|c| deep::passes(c, &deep::evidence(c, &recalls), &run.night))
            .filter_map(|c| deep::reread(c, &notes))
            .filter(|c| !deep::promoted(c, &memory))
            .collect();
        if gated.is_empty() {
            run.phase = Phase::Diary;
            self.save_run(run)?;
            return Ok(Step::Next);
        }
        let mut request = self.dream_request(
            vec![deep::INSTRUCTIONS.into()],
            &deep::input(&gated, &memory, &profile),
            Effort::Medium,
        )?;
        request.format = Some(deep::schema());
        let ops = match self.ask(provider, key, &request, stop, run, "deep").await {
            Answer::Stopped => return Ok(Step::Paused),
            Answer::Fatal => return Ok(Step::Failed),
            Answer::Unavailable => None,
            Answer::Reply(reply) => deep::parse(&reply.content),
        };
        let night = run.night.clone();
        // dreaming FR-015: decisions only if the result validates, otherwise the append-only fallback.
        let build = |memory: &str, profile: &str| -> (deep::Applied, bool) {
            if let Some(ops) = &ops {
                let applied = deep::apply(ops, &gated, memory, profile, &night);
                if deep::valid_memory(memory, &applied.memory, &applied.written)
                    && deep::valid_profile(profile, &applied.profile)
                {
                    return (applied, false);
                }
            }
            (deep::fallback(&gated, memory, profile, &night), true)
        };
        let (applied, fell_back) = build(&memory, &profile);
        run.checkpoint.decisions = ops.clone().unwrap_or_default();
        run.checkpoint.gated = gated.iter().map(|c| c.id).collect();
        run.checkpoint.kept = applied.kept.clone();
        (run.added, run.merged, run.superseded) =
            (applied.added, applied.merged, applied.superseded);
        run.fell_back = fell_back;
        let prior = BTreeMap::from([("MEMORY.md", memory.clone()), ("USER.md", profile.clone())]);
        let next = BTreeMap::from([("MEMORY.md", applied.memory), ("USER.md", applied.profile)]);
        let rebuild = |rel: &str, current: &str| -> Option<String> {
            if rel == "MEMORY.md" {
                Some(build(current, &profile).0.memory)
            } else {
                Some(build(&memory, current).0.profile)
            }
        };
        self.commit_dream(run, &prior, &next, &rebuild)?;
        Ok(Step::Next)
    }

    /// The diary entry, in the companion's voice when the model answers, plain otherwise (dreaming R11).
    async fn diary_phase(
        &self,
        run: &mut Run,
        provider: &impl LlmProvider,
        key: &ApiKey,
        stop: &Notify,
    ) -> Result<Step> {
        let cp = &run.checkpoint;
        if cp.candidates.is_empty() && cp.messages.is_empty() {
            // Journey: earlier days may still wait for their topics.
            run.phase = Phase::World;
            self.save_run(run)?;
            return Ok(Step::Next);
        }
        let files = context::read_files(&self.workspace_dir());
        let soul = context::system("", &files, None).swap_remove(1);
        let request = self.dream_request(
            vec![soul, diary::INSTRUCTIONS.into()],
            &diary::input(&cp.themes, cp.candidates.len(), &cp.kept),
            Effort::Low,
        )?;
        let narrative = match self.ask(provider, key, &request, stop, run, "diary").await {
            Answer::Stopped => return Ok(Step::Paused),
            Answer::Reply(reply) => Some(context::reply_text(&reply.content)),
            Answer::Unavailable | Answer::Fatal => None,
        };
        let cp = &run.checkpoint;
        let entry = diary::entry(
            &run.night,
            narrative.as_deref(),
            cp.candidates.len(),
            &cp.kept,
        );
        self.edit_workspace("DREAMS.md", |current| {
            Ok((diary::append(current, &entry), ()))
        })?;
        run.checkpoint.diary = Some(entry);
        run.phase = Phase::World;
        self.save_run(run)?;
        Ok(Step::Next)
    }

    /// Journey: the ended days' topics, one request for all of them (dreaming R1 to R3). Nothing is written
    /// here; a failure leaves the days due for the next sweep (dreaming FR-007).
    async fn world_phase(
        &self,
        run: &mut Run,
        provider: &impl LlmProvider,
        key: &ApiKey,
        clock: &LocalClock,
        stop: &Notify,
    ) -> Result<Step> {
        let after = if run.committed {
            Phase::Roll
        } else {
            Phase::Done
        };
        let today = local_today(clock)?;
        let since = self.state()?.world.journey_since;
        let considered = self.journey_considered()?;
        let (typed, forgotten) = self.transcript(|t| {
            let forgotten: HashSet<String> = context::forgotten(&t.lines)
                .into_iter()
                .map(Into::into)
                .collect();
            let refs: HashSet<&str> = forgotten.iter().map(String::as_str).collect();
            Ok((topics::typed(&t.lines, &refs), forgotten))
        })?;
        let days = topics::due_days(&typed, &today, since.as_deref(), considered.as_deref());
        if !self.settings()?.dreaming.world || days.is_empty() {
            run.phase = after;
            self.save_run(run)?;
            return Ok(Step::Next);
        }
        let workspace = self.workspace_dir();
        let notes: BTreeMap<String, String> = days
            .iter()
            .filter_map(|d| {
                Some((
                    d.clone(),
                    std::fs::read_to_string(workspace.join(format!("memory/{d}.md"))).ok()?,
                ))
            })
            .collect();
        let refs: HashSet<&str> = forgotten.iter().map(String::as_str).collect();
        let soul = context::system("", &context::read_files(&workspace), None).swap_remove(1);
        let mut request = self.dream_request(
            vec![soul, topics::INSTRUCTIONS.into()],
            &topics::input(&days, &typed, &notes, &refs),
            Effort::Low,
        )?;
        request.format = Some(topics::schema());
        let named = match self.ask(provider, key, &request, stop, run, "world").await {
            Answer::Stopped => return Ok(Step::Paused),
            Answer::Reply(reply) => topics::parse(&reply.content, &days),
            Answer::Unavailable | Answer::Fatal => None,
        };
        let planned = match named {
            Some(named) => self.plan_journey(&named)?.map(|lines| (named, lines)),
            None => None,
        };
        let Some((named, (world, journey))) = planned else {
            run.phase = after;
            self.save_run(run)?;
            return Ok(Step::Next);
        };
        run.checkpoint.topics = named;
        run.checkpoint.drawing = Some(Drawing { world, journey });
        run.phase = Phase::Drawing;
        self.save_run(run)?;
        Ok(Step::Next)
    }

    /// journey R8: appends the planned lines; run again after a stop, it adds only what is missing.
    fn drawing_phase(&self, run: &mut Run) -> Result<Step> {
        if let Some(drawing) = &run.checkpoint.drawing {
            self.draw_planned(&drawing.world, &drawing.journey)?;
        }
        if let Some(newest) = run.checkpoint.topics.iter().map(|d| d.day.clone()).max() {
            self.update_state(|s| {
                if s.world
                    .journey_day
                    .as_deref()
                    .is_none_or(|d| newest.as_str() > d)
                {
                    s.world.journey_day = Some(newest);
                }
            })?;
        }
        run.phase = if run.committed {
            Phase::Roll
        } else {
            Phase::Done
        };
        self.save_run(run)?;
        Ok(Step::Next)
    }

    /// dreaming FR-023, FR-024: a context with the new memory and the dream. A compaction keeps the conversation's
    /// thread; with nothing new since the context began, it is reopened instead (dreaming R10).
    async fn roll_phase(
        &self,
        run: &mut Run,
        provider: &impl LlmProvider,
        key: &ApiKey,
        clock: &LocalClock,
        stop: &Notify,
    ) -> Result<Step> {
        let dream = run.checkpoint.diary.clone();
        let ask = Ask {
            focus: None,
            only_if_due: false,
            dream: dream.as_deref(),
        };
        match self.compact(provider, key, clock, ask, Some(stop)).await? {
            CompactOutcome::Stopped => return Ok(Step::Paused),
            CompactOutcome::Failed { .. } => return Ok(Step::Later),
            CompactOutcome::Compacted => {}
            CompactOutcome::NothingToCompact => self.transcript(|t| {
                let current = context::current(&t.lines);
                let fresh = !current
                    .iter()
                    .any(|l| matches!(l.entry, Entry::Message { .. }));
                // With turns that can't be compacted yet, the next message carries the changed files as a notice.
                if fresh && !current.is_empty() {
                    let compaction = match &current[0].entry {
                        Entry::Context { compaction, .. } => compaction.clone(),
                        _ => None,
                    };
                    let entry = self.context_entry(clock, compaction, &[], dream.as_deref())?;
                    self.append(t, entry)?;
                }
                Ok(())
            })?,
        }
        run.phase = Phase::Done;
        self.save_run(run)?;
        Ok(Step::Next)
    }
}
