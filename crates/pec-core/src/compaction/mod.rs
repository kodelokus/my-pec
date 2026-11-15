//! Compaction: when the conversation reaches its working size, older turns are summarized after a silent memory
//! flush, and a new context continues with the summary and the recent turns (CMP-1 to CMP-8).

mod cut;
mod flush;
mod summary;

use crate::chat::context::{self, held, opened_by, text_block, turns};
use crate::chat::transcript::{CompactionUsage, Entry, FlushNote, FlushOutcome, FlushRecord, Line};
use crate::chat::{ChatState, LocalClock, context_size, lock, now_ms, owed, request_from};
use crate::config::Settings;
use crate::credentials::ApiKey;
use crate::llm::{
    Effort, LlmProvider, ModelInfo, ProviderError, Reply, StreamFailure, TurnRequest, effort_for,
};
use crate::memory::files::{LocalNow, Origin, note_line, note_text};
use crate::onboarding::field;
use crate::{AppError, Core, Result};
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Notify;
use ts_rs::TS;

/// Held back from the working size so a turn can still be answered while a compaction is due (compaction R1).
pub(crate) const RESERVE_TOKENS: u64 = 6_000;
/// The recent turns kept word for word (compaction FR-014).
pub(crate) const KEEP_RECENT_TOKENS: u64 = 8_000;
pub(crate) const MAX_SUMMARY_CHARS: usize = 8_000;
pub(crate) const MAX_FLUSH_NOTES: usize = 20;
pub(crate) const MAX_FOCUS_CHARS: usize = 500;
/// The working size's range (compaction R11): the floor leaves room for the reserve, the kept turns, a summary and
/// the companion's files; the ceiling is the smallest window among the offered models.
pub const MIN_BUDGET: u64 = 32_000;
pub const MAX_BUDGET: u64 = 200_000;
pub const BUDGET_STEP: u64 = 1_000;

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum CompactOutcome {
    Compacted,
    NothingToCompact,
    /// A message came in; nothing was saved (compaction FR-006).
    Stopped,
    /// Nothing was saved. No error: the summary failed its checks twice (compaction FR-017).
    Failed {
        error: Option<ProviderError>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CompactResult {
    pub outcome: CompactOutcome,
    pub state: ChatState,
}

/// The working size and the compaction model, for settings (CMP-8, compaction R11).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CompactionSettings {
    #[ts(type = "number")]
    pub budget_tokens: u64,
    #[ts(type = "number")]
    pub min_budget: u64,
    #[ts(type = "number")]
    pub max_budget: u64,
    #[ts(type = "number")]
    pub budget_step: u64,
    /// None: the chat model.
    pub model: Option<String>,
    /// The chosen model is no longer offered; the chat model is used instead (compaction FR-027).
    pub model_missing: bool,
}

/// What a compaction is asked for: a focus note ("compact now"), whether only when due, and the diary entry the
/// new context carries after a dream (dreaming R10).
#[derive(Default)]
pub(crate) struct Ask<'a> {
    pub focus: Option<&'a str>,
    pub only_if_due: bool,
    pub dream: Option<&'a str>,
}

/// Clears the compacting slot however a compaction ends.
struct Compacting<'a> {
    core: &'a Core,
    stop: Arc<Notify>,
}

impl Drop for Compacting<'_> {
    fn drop(&mut self) {
        *lock(&self.core.chat.compacting) = None;
    }
}

/// What a compaction works from, taken in one look at the conversation.
struct Snapshot {
    lines: usize,
    size: u64,
    first_kept: String,
    kept_tokens: u64,
    input: String,
    /// The flush request; none when the flush is skipped (compaction R4).
    flush: Option<TurnRequest>,
    /// The latest message that opened a turn: the source the flush notes record.
    src: String,
}

impl Core {
    /// Compacts the conversation when it is due, or whenever asked with `manual` ("compact now", "switch now"),
    /// between turns (compaction interfaces).
    pub async fn compact_conversation(
        &self,
        provider: &impl LlmProvider,
        clock: &LocalClock,
        focus: Option<&str>,
        manual: bool,
    ) -> Result<CompactResult> {
        context::time_block(clock)?;
        let focus = match focus.map(str::trim).filter(|f| !f.is_empty()) {
            Some(f) if !manual || f.chars().count() > MAX_FOCUS_CHARS => {
                return Err(AppError::invalid("focus"));
            }
            focus => focus,
        };
        let outcome = match self.secrets.get() {
            None => CompactOutcome::Failed {
                error: Some(ProviderError::KeyRejected),
            },
            Some(key) => {
                let guard = self.begin_compaction()?;
                let stop = guard.stop.clone();
                let ask = Ask {
                    focus,
                    only_if_due: !manual,
                    dream: None,
                };
                self.compact(provider, &key, clock, ask, Some(&stop))
                    .await?
            }
        };
        Ok(CompactResult {
            outcome,
            state: self.chat_state()?,
        })
    }

    pub fn compaction_settings(&self) -> Result<CompactionSettings> {
        let settings = self.settings()?;
        let cached = self.state()?.models.items;
        let model_missing = settings
            .compaction
            .model
            .as_ref()
            .is_some_and(|m| !cached.iter().any(|c| &c.id == m));
        Ok(CompactionSettings {
            budget_tokens: settings.context.budget(),
            min_budget: MIN_BUDGET,
            max_budget: MAX_BUDGET,
            budget_step: BUDGET_STEP,
            model: settings.compaction.model,
            model_missing,
        })
    }

    /// compaction FR-003, FR-027: the working size in its range and steps, and a model from the offered list or none.
    pub fn set_compaction_settings(
        &self,
        budget_tokens: u64,
        model: Option<String>,
    ) -> Result<CompactionSettings> {
        if !(MIN_BUDGET..=MAX_BUDGET).contains(&budget_tokens)
            || !budget_tokens.is_multiple_of(BUDGET_STEP)
        {
            return Err(AppError::invalid("budgetTokens"));
        }
        let cached = self.state()?.models.items;
        if model
            .as_ref()
            .is_some_and(|m| !cached.iter().any(|c| &c.id == m))
        {
            return Err(AppError::invalid("model"));
        }
        let mut settings = self.settings()?;
        settings.context.budget_tokens = budget_tokens;
        settings.compaction.model = model;
        self.save_settings(&settings)?;
        self.compaction_settings()
    }

    /// Takes the compacting slot: refused while a reply or another compaction runs.
    fn begin_compaction(&self) -> Result<Compacting<'_>> {
        let active = lock(&self.chat.active);
        let mut slot = lock(&self.chat.compacting);
        if active.is_some() || slot.is_some() || lock(&self.chat.dreaming).is_some() {
            return Err(AppError::invalid("busy"));
        }
        let stop = Arc::new(Notify::new());
        *slot = Some(stop.clone());
        Ok(Compacting { core: self, stop })
    }

    /// The compaction itself (compaction R3 to R9). `stop` is absent inline, where the reply slot is already held
    /// and nothing can come in (compaction R10).
    pub(crate) async fn compact(
        &self,
        provider: &impl LlmProvider,
        key: &ApiKey,
        clock: &LocalClock,
        ask: Ask<'_>,
        stop: Option<&Notify>,
    ) -> Result<CompactOutcome> {
        let Ask {
            focus,
            only_if_due,
            dream,
        } = ask;
        let settings = self.settings()?;
        let cached = self.state()?.models.items;
        let Some(snapshot) = self.snapshot(&settings, &cached, focus, only_if_due)? else {
            return Ok(CompactOutcome::NothingToCompact);
        };
        // The compaction model when it is still offered, otherwise the chat model (compaction FR-027).
        let chosen = settings
            .compaction
            .model
            .clone()
            .filter(|m| cached.iter().any(|c| &c.id == m));
        let effort = |model: &str| effort_for(&cached, model, Effort::Low);

        // The flush: its notes are held until the commit (compaction R4, compaction FR-012, FR-013).
        let now = context::local_now(clock);
        let (mut flush_usage, mut notes, mut outcome) = (None, Vec::new(), FlushOutcome::Skipped);
        if let Some(mut request) = snapshot.flush.clone() {
            if let Some(model) = &chosen {
                request.effort = effort(model);
                request.model.clone_from(model);
            }
            outcome = match call(provider, key, &request, stop).await {
                None => return Ok(CompactOutcome::Stopped),
                Some(Ok(reply)) if reply.stop_reason.as_deref() != Some("refusal") => {
                    flush_usage = reply.usage;
                    let today = std::fs::read_to_string(
                        self.workspace_dir().join(format!("memory/{}.md", now.date)),
                    )
                    .unwrap_or_default();
                    notes = flush::accepted(&reply.content, &today);
                    if notes.is_empty() {
                        FlushOutcome::Nothing
                    } else {
                        FlushOutcome::Saved
                    }
                }
                Some(_) => FlushOutcome::Failed,
            };
        }

        let model = chosen.unwrap_or_else(|| settings.chat.model.clone());
        let request = TurnRequest {
            effort: effort(&model),
            system: vec![summary::INSTRUCTIONS.to_owned()],
            messages: vec![json!({ "role": "user", "content": [text_block(&snapshot.input)] })],
            tools: Vec::new(),
            format: Some(summary::schema()),
            model,
        };
        // A reply that fails its checks is asked for once more (compaction FR-017).
        let mut summary_usage = None;
        let mut written = None;
        for _ in 0..2 {
            let reply = match call(provider, key, &request, stop).await {
                None => return Ok(CompactOutcome::Stopped),
                Some(Err(failure)) => {
                    return Ok(CompactOutcome::Failed {
                        error: Some(failure.error),
                    });
                }
                Some(Ok(reply)) => reply,
            };
            summary_usage = reply.usage.clone();
            let finished = !matches!(reply.stop_reason.as_deref(), Some("max_tokens" | "refusal"));
            if let Some(text) = summary::parse(&reply.content).filter(|_| finished) {
                written = Some(text);
                break;
            }
        }
        let Some(summary) = written else {
            return Ok(CompactOutcome::Failed { error: None });
        };

        let staged: Vec<String> = notes
            .iter()
            .map(|text| note_line(&now.time, text, Origin::Companion, &snapshot.src))
            .collect();
        let record = Entry::Compaction {
            summary,
            first_kept: snapshot.first_kept.clone(),
            tokens_before: snapshot.size,
            kept_tokens: snapshot.kept_tokens,
            model: request.model,
            usage: CompactionUsage {
                flush: flush_usage,
                summary: summary_usage,
            },
            flush: FlushRecord {
                outcome,
                notes: notes
                    .iter()
                    .map(|text| FlushNote {
                        time: now.time.clone(),
                        text: text.clone(),
                    })
                    .collect(),
            },
            focus: focus.map(Into::into),
        };
        if !self.commit(clock, &snapshot, record, &staged, stop.is_some(), dream)? {
            return Ok(CompactOutcome::Stopped);
        }
        self.save_flush_notes(&now, &notes, &snapshot.src)?;
        Ok(CompactOutcome::Compacted)
    }

    /// One look at the conversation: whether a compaction is due, the cut, the summary's input and the flush
    /// request (compaction R1, R4, R6, R7, R13).
    fn snapshot(
        &self,
        settings: &Settings,
        cached: &[ModelInfo],
        focus: Option<&str>,
        only_if_due: bool,
    ) -> Result<Option<Snapshot>> {
        let workspace = self.workspace_dir();
        let identity = std::fs::read_to_string(workspace.join("IDENTITY.md")).unwrap_or_default();
        let profile = std::fs::read_to_string(workspace.join("USER.md")).unwrap_or_default();
        let names = summary::Names {
            companion: field(&identity, "Name").unwrap_or("the companion"),
            person: field(&profile, "Call them").unwrap_or("the person"),
        };
        self.transcript(|t| {
            let size = context_size(&t.lines);
            let due =
                !owed(&t.lines) && size > 0 && size >= settings.context.budget() - RESERVE_TOKENS;
            if only_if_due && !due {
                return Ok(None);
            }
            let gone = context::replaced(&t.lines);
            let (messages, from) = held(&t.lines);
            let Some(cut) = cut::cut(&messages, from, size, &gone, KEEP_RECENT_TOKENS) else {
                return Ok(None);
            };
            let forgotten = context::forgotten(&t.lines);
            let older: Vec<&Line> = messages[..cut.first_kept]
                .iter()
                .copied()
                .filter(|l| !gone.contains(l.id.as_str()))
                .collect();
            let older: Vec<_> = turns(&older)
                .into_iter()
                .filter(|turn| !forgotten.contains(turn.user.id.as_str()))
                .collect();
            let previous = opened_by(&t.lines).and_then(|o| o.summary);
            // No flush without memory tools, or while the companion still sees something forgotten (compaction R13).
            let sees_forgotten = messages.iter().any(|l| forgotten.contains(l.id.as_str()))
                || context::current(&t.lines)
                    .iter()
                    .any(|l| matches!(l.entry, Entry::Forgotten { .. }));
            let request = request_from(self, &t.lines, cached)?;
            let flush =
                (!request.tools.is_empty() && !sees_forgotten).then(|| flush::request(request));
            let src = messages
                .iter()
                .rev()
                .find(|l| context::is_turn_start(l))
                .map_or_else(String::new, |l| l.id.clone());
            Ok(Some(Snapshot {
                lines: t.lines.len(),
                size,
                first_kept: messages[cut.first_kept].id.clone(),
                kept_tokens: cut.kept_tokens,
                input: summary::input(previous, &older, &names, focus),
                flush,
                src,
            }))
        })
    }

    /// The commit: the record and the new context in one write, only if nothing came in meanwhile, with the
    /// pending marker set first when there are notes to save after it (compaction R3, R5).
    fn commit(
        &self,
        clock: &LocalClock,
        snapshot: &Snapshot,
        record: Entry,
        staged: &[String],
        yielding: bool,
        dream: Option<&str>,
    ) -> Result<bool> {
        self.transcript(|t| {
            let busy = yielding && lock(&self.chat.active).is_some();
            if busy || t.lines.len() != snapshot.lines {
                return Ok(false);
            }
            let at = now_ms();
            let id = t.next_id(at);
            let opened = Line {
                id: t.next_id(at),
                at,
                entry: self.context_entry(clock, Some(id.clone()), staged, dream)?,
            };
            if !staged.is_empty() {
                self.update_state(|s| s.compaction.pending = Some(id.clone()))?;
            }
            let record = Line {
                id,
                at,
                entry: record,
            };
            t.append_all(vec![record, opened])?;
            Ok(true)
        })
    }

    /// Appends a committed compaction's notes to the day's note, skipping any already there, and clears the
    /// pending marker (compaction R5).
    fn save_flush_notes(&self, now: &LocalNow, notes: &[String], src: &str) -> Result<()> {
        if notes.is_empty() {
            return Ok(());
        }
        let path = self.workspace_dir().join(format!("memory/{}.md", now.date));
        let today = std::fs::read_to_string(path).unwrap_or_default();
        let present: HashSet<&str> = today.lines().filter_map(note_text).collect();
        for text in notes.iter().filter(|t| !present.contains(t.as_str())) {
            self.add_note(now, text, Origin::Companion, src)?;
        }
        self.update_state(|s| s.compaction.pending = None)
    }

    /// Finishes the notes of a compaction the app stopped between its commit and its notes, once, when the
    /// conversation file is first opened; a compaction that isn't fully on disk only clears the marker (compaction R5).
    pub(crate) fn finish_pending(&self, lines: &[Line]) -> Result<()> {
        let Some(id) = self.state()?.compaction.pending else {
            return Ok(());
        };
        let at = lines.iter().position(|l| l.id == id);
        let done = context::committed(lines).contains(id.as_str());
        if let (Some(at), true) = (at, done)
            && let Entry::Compaction { flush, .. } = &lines[at].entry
        {
            let date = lines[at..].iter().find_map(|l| match &l.entry {
                Entry::Context { notes: Some(n), .. } => n.keys().max().cloned(),
                _ => None,
            });
            let src = lines[..at]
                .iter()
                .rev()
                .find(|l| context::is_turn_start(l))
                .map_or_else(String::new, |l| l.id.clone());
            if let (Some(date), Some(first)) = (date, flush.notes.first()) {
                let now = LocalNow {
                    date,
                    time: first.time.clone(),
                };
                let texts: Vec<String> = flush.notes.iter().map(|n| n.text.clone()).collect();
                return self.save_flush_notes(&now, &texts, &src);
            }
        }
        self.update_state(|s| s.compaction.pending = None)
    }
}

/// One model call, given up as soon as `stop` fires.
pub(crate) async fn call(
    provider: &impl LlmProvider,
    key: &ApiKey,
    request: &TurnRequest,
    stop: Option<&Notify>,
) -> Option<std::result::Result<Reply, StreamFailure>> {
    let mut ignore = |_: &str| {};
    match stop {
        Some(stop) => tokio::select! {
            result = provider.stream(key, request, &mut ignore) => Some(result),
            () = stop.notified() => None,
        },
        None => Some(provider.stream(key, request, &mut ignore).await),
    }
}
