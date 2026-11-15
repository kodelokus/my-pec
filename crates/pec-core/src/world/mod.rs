//! The companion's world: its id and birth date, its DNA, the item dictionary, the append-only world file and
//! daily growth, with no model call.

mod describe;
mod dna;
mod file;
mod grow;
mod id;
pub use id::random_chars;
pub(crate) mod journey;
mod pick;

use crate::chat::context;
use crate::chat::transcript::{self, Line, WorldSnapshot};
use crate::chat::{LocalClock, local_today, lock};
use crate::memory::browser::ForgetOutcome;
use crate::memory::files::fingerprint;
use crate::{AppError, Core, Result};
use file::{Drawn, Header, WORLD_FILE, World, line};
use id::{identity, new_id, parse_id};
use journey::{Day, JOURNEY_FILE, Journey, NamedDay, Told, Topic};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use ts_rs::TS;

/// The kind of place an item belongs to (world data 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Land {
    Meadow,
    Water,
    Woods,
    Rock,
    Any,
    NearHome,
    /// Any land but water: where everyday objects stand (journey R4).
    Ground,
}

impl Land {
    /// Whether an item of this kind may stand on a cell of `land`, `near_home` cells from the home.
    pub(crate) fn suits(self, land: Land, near_home: bool) -> bool {
        match self {
            Land::Any => true,
            Land::Ground => land != Land::Water,
            Land::NearHome => near_home && land != Land::Water,
            kind => kind == land,
        }
    }
}

/// One item of the bundled dictionary.
#[derive(Debug, Deserialize)]
pub(crate) struct Entry {
    pub id: String,
    pub land: Land,
    pub stage: u8,
    /// A home is drawn only when a world is created, never by growth.
    #[serde(default)]
    pub home: bool,
    /// Journey: an everyday object's subjects. Scenery has none, and only everyday objects have them.
    #[serde(default)]
    pub subjects: Vec<String>,
}

/// One subject of the fixed list that topics and everyday objects are filed under (journey FR-018).
#[derive(Debug, Deserialize)]
pub(crate) struct Subject {
    pub id: String,
    /// What it covers, in English, for the model.
    pub about: String,
    pub related: Vec<String>,
}

/// The subjects that ship with this version, read once from `subjects.json`.
pub(crate) fn subjects() -> &'static [Subject] {
    static SUBJECTS: OnceLock<Vec<Subject>> = OnceLock::new();
    SUBJECTS.get_or_init(|| {
        serde_json::from_str(include_str!("subjects.json")).expect("subjects.json is valid")
    })
}

/// The items that ship with this version, read once from `items.json` (world R8).
pub(crate) fn dictionary() -> &'static [Entry] {
    static ITEMS: OnceLock<Vec<Entry>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        serde_json::from_str(include_str!("items.json")).expect("items.json is valid")
    })
}

/// The companion's colours (world R4).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct WorldPalette {
    pub ground: String,
    pub water: String,
    pub leaf: String,
    pub bark: String,
    pub stone: String,
    pub accent: String,
}

/// One drawn item, as the world view shows it.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DrawnItem {
    pub item: String,
    /// False for an id this version's dictionary doesn't have; shown as a placeholder (world FR-015).
    pub known: bool,
    pub x: i32,
    pub y: i32,
    pub size: f64,
    pub flip: bool,
    pub shade: u8,
    pub day: String,
    /// Journey: the topic it was drawn for.
    pub topic: Option<TopicRef>,
    /// Its day's keywords, its own first; empty for a day without topics (journey FR-024).
    pub keywords: Vec<String>,
}

/// The `n`th topic of `day` in the journey.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct TopicRef {
    pub day: String,
    pub n: u32,
}

/// A day of the journey, as the diary list shows it (journey FR-026).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JourneyDay {
    pub day: String,
    pub topics: Vec<JourneyTopic>,
    /// Indexes into the items: the scenery drawn that day.
    pub scenery: Vec<u32>,
    pub has_notes: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct JourneyTopic {
    pub n: u32,
    /// The item id drawn for it.
    pub drawn: String,
    /// Its index into the items; none when the world file no longer has it.
    pub item: Option<u32>,
    /// None once forgotten (journey FR-041).
    pub story: Option<Story>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Story {
    pub keyword: String,
    pub line: String,
    pub subjects: Vec<String>,
    pub related: bool,
    pub odds: Vec<Odds>,
    pub rest: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Odds {
    pub item: String,
    pub chance: f64,
}

/// A subject's share of the topic objects, 0 to 1 (journey FR-031).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SubjectShare {
    pub subject: String,
    pub share: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorldView {
    pub born: Option<String>,
    pub palette: WorldPalette,
    pub items: Vec<DrawnItem>,
    /// Newest first.
    pub journey: Vec<JourneyDay>,
    /// Largest first.
    pub makeup: Vec<SubjectShare>,
    pub skipped_lines: u32,
    pub journey_skipped: u32,
    /// The base for `forget_topic`.
    pub journey_hash: String,
}

/// Workspace files `prepare_world` had to recreate (world data 7).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct PrepareOutcome {
    pub recreated: Vec<String>,
}

impl Core {
    /// Gives the companion an id, a birth date and a world if it lacks any of them (world FR-001 to FR-003, FR-018,
    /// world FR-038, FR-039, world R2, R11). Idempotent. Returns the files it had to recreate.
    pub(crate) fn ensure_world(&self, today: &str) -> Result<Vec<String>> {
        let workspace = self.workspace_dir();
        let world = std::fs::read_to_string(workspace.join(WORLD_FILE))
            .ok()
            .map(|text| World::parse(&text));
        let restored = world
            .as_ref()
            .and_then(|w| w.header.as_ref())
            .and_then(|h| parse_id(&h.companion));
        // Read before taking the writer's lock: the conversation has a lock of its own.
        let first_day = self.first_conversation_day()?;
        let mut added = None;
        let (companion, born, had_id) = self.edit_workspace("IDENTITY.md", |current| {
            let (id, born) = identity(current);
            if let Some(id) = id {
                return Ok((current.to_owned(), (id, born, true)));
            }
            let id = restored.clone().unwrap_or_else(new_id);
            let mut next = current.to_owned();
            if !next.is_empty() && !next.ends_with('\n') {
                next.push('\n');
            }
            next.push_str(&format!("- Id: {id}\n"));
            let born = born.unwrap_or_else(|| {
                let day = first_day.clone().unwrap_or_else(|| today.to_owned());
                next.push_str(&format!("- Born: {day}\n"));
                day
            });
            added = Some(next.clone());
            Ok((next, (id, Some(born), false)))
        })?;
        if let Some(content) = added {
            self.record_written("IDENTITY.md", content)?;
        }
        let mut recreated = Vec::new();
        if world.is_none() {
            let header = Header {
                world: 1,
                companion: companion.clone(),
                born: born.unwrap_or_else(|| today.to_owned()),
            };
            let homes: Vec<&Entry> = dictionary().iter().filter(|e| e.home).collect();
            let home = homes
                [dna::Rng::new(dna::seed(&companion, "home")).below(homes.len() as u64) as usize];
            let item = Drawn {
                item: home.id.clone(),
                x: 0,
                y: 0,
                size: 1.0,
                flip: false,
                shade: 0,
                day: today.to_owned(),
                why: "ambient".into(),
            };
            self.append_workspace(WORLD_FILE, &[line(&header), line(&item)])?;
            // A companion that already had an id had a world: say it was recreated (world R11).
            if had_id {
                recreated.push(WORLD_FILE.to_owned());
            }
        }
        Ok(recreated)
    }

    /// world FR-019 to FR-025: the day's items, once, after a message the user sent. Most messages stop at the
    /// `lastDay` comparison and never read the world file (world R7).
    pub(crate) fn grow_today(&self, today: &str) -> Result<()> {
        let last_day = self.state()?.world.last_day;
        if last_day.as_deref().is_some_and(|day| day >= today) {
            return Ok(());
        }
        // No id or no world file yet: `prepare_world` creates them when the main view opens, before any message.
        let workspace = self.workspace_dir();
        let identity_text =
            std::fs::read_to_string(workspace.join("IDENTITY.md")).unwrap_or_default();
        let (Some(companion), Ok(text)) = (
            identity(&identity_text).0,
            std::fs::read_to_string(workspace.join(WORLD_FILE)),
        ) else {
            return Ok(());
        };
        let world = World::parse(&text);
        let drawn = grow::grow(&companion, today, &world, dictionary(), last_day.as_deref());
        if !drawn.is_empty() {
            let lines: Vec<String> = drawn.iter().map(line).collect();
            self.append_workspace(WORLD_FILE, &lines)?;
        }
        self.update_state(|s| s.world.last_day = Some(today.to_owned()))
    }

    /// journey R9: the description a new context is given; none before the world exists.
    pub(crate) fn world_snapshot(&self) -> Option<WorldSnapshot> {
        let workspace = self.workspace_dir();
        let (Some(id), born) =
            identity(&std::fs::read_to_string(workspace.join("IDENTITY.md")).unwrap_or_default())
        else {
            return None;
        };
        let world = World::parse(&std::fs::read_to_string(workspace.join(WORLD_FILE)).ok()?);
        let journey = Journey::parse(
            &std::fs::read_to_string(workspace.join(JOURNEY_FILE)).unwrap_or_default(),
        );
        let born = born
            .or_else(|| world.header.as_ref().map(|h| h.born.clone()))
            .unwrap_or_default();
        Some(WorldSnapshot {
            text: describe::describe(&id, &born, &world, &journey),
            through: journey.newest().map(Into::into),
        })
    }

    /// journey R10: the notice for what was drawn since the current context was last told, and the
    /// day it tells through. Only a context with a world snapshot is told; most messages stop at the state check.
    pub(crate) fn world_notice(&self, lines: &[Line]) -> Result<Option<(String, String)>> {
        let current = context::current(lines);
        let Some(transcript::Entry::Context {
            world: Some(snapshot),
            ..
        }) = current.first().map(|l| &l.entry)
        else {
            return Ok(None);
        };
        let told = current
            .iter()
            .filter_map(|l| match &l.entry {
                transcript::Entry::Message { world, .. } => world.clone(),
                _ => None,
            })
            .chain(snapshot.through.clone())
            .max();
        let Some(newest) = self.state()?.world.journey_day else {
            return Ok(None);
        };
        if told.as_deref().is_some_and(|t| newest.as_str() <= t) {
            return Ok(None);
        }
        let text =
            std::fs::read_to_string(self.workspace_dir().join(JOURNEY_FILE)).unwrap_or_default();
        Ok(describe::notice(&Journey::parse(&text), told.as_deref()))
    }

    /// journey FR-041, FR-043: topic `n` of `day` loses its keyword, line and subjects; its object stays.
    pub fn forget_topic(&self, day: &str, n: u32, base_hash: &str) -> Result<ForgetOutcome> {
        if lock(&self.chat.dreaming).is_some() {
            return Err(AppError::invalid("busy"));
        }
        let told = self.edit_workspace(JOURNEY_FILE, |current| {
            if fingerprint(current) != base_hash {
                return Err(AppError::invalid("changed"));
            }
            let told = Journey::parse(current)
                .day(day)
                .and_then(|d| d.topics.get(n as usize))
                .and_then(Topic::told)
                .cloned()
                .ok_or_else(|| AppError::invalid("topic"))?;
            let next = journey::forget(current, day, n as usize)
                .ok_or_else(|| AppError::invalid("topic"))?;
            Ok((next, told))
        })?;
        Ok(ForgetOutcome {
            in_current_context: self.seen_in_context(&[told.keyword, told.line])?,
        })
    }

    /// journey FR-042: every story of these days is forgotten. Returns the forgotten keywords and lines.
    pub(crate) fn forget_days(&self, days: &[String]) -> Result<Vec<String>> {
        if !self.workspace_dir().join(JOURNEY_FILE).exists() {
            return Ok(Vec::new());
        }
        self.edit_workspace(JOURNEY_FILE, |current| {
            let mut text = current.to_owned();
            let mut gone = Vec::new();
            for day in days {
                let count = Journey::parse(&text).day(day).map_or(0, |d| d.topics.len());
                for n in 0..count {
                    if let Some(told) = Journey::parse(&text)
                        .day(day)
                        .and_then(|d| d.topics[n].told().cloned())
                    {
                        text = journey::forget(&text, day, n).unwrap_or(text);
                        gone.extend([told.keyword, told.line]);
                    }
                }
            }
            Ok((text, gone))
        })
    }

    /// journey R2: the newest day the journey has considered.
    pub(crate) fn journey_considered(&self) -> Result<Option<String>> {
        if let Some(day) = self.state()?.world.journey_day {
            return Ok(Some(day));
        }
        let text =
            std::fs::read_to_string(self.workspace_dir().join(JOURNEY_FILE)).unwrap_or_default();
        Ok(Journey::parse(&text).newest().map(Into::into))
    }

    /// journey R5 to R7: the world and journey lines the named days give, the oldest day first, each
    /// pick counting the ones before it. `None` without an id.
    pub(crate) fn plan_journey(
        &self,
        named: &[NamedDay],
    ) -> Result<Option<(Vec<String>, Vec<String>)>> {
        let workspace = self.workspace_dir();
        let identity_text =
            std::fs::read_to_string(workspace.join("IDENTITY.md")).unwrap_or_default();
        let Some(companion) = identity(&identity_text).0 else {
            return Ok(None);
        };
        let mut drawn =
            World::parse(&std::fs::read_to_string(workspace.join(WORLD_FILE)).unwrap_or_default())
                .items;
        let (mut world_lines, mut journey_lines) = (Vec::new(), Vec::new());
        let mut days: Vec<&NamedDay> = named.iter().collect();
        days.sort_by(|a, b| a.day.cmp(&b.day));
        for named in days {
            let mut topics = Vec::new();
            for topic in &named.topics {
                let n = topics.len();
                let Some((chosen, mut rng)) = pick::pick(
                    &companion,
                    &named.day,
                    n,
                    &topic.subjects,
                    dictionary(),
                    &drawn,
                ) else {
                    continue;
                };
                let land = dictionary()
                    .iter()
                    .find(|e| e.id == chosen.item)
                    .map_or(Land::Ground, |e| e.land);
                let (x, y) = pick::place(&companion, &named.day, land, &drawn);
                let item = Drawn {
                    item: chosen.item.clone(),
                    x,
                    y,
                    size: f64::from(8 + rng.below(5) as u8) / 10.0,
                    flip: rng.below(2) == 1,
                    shade: rng.below(3) as u8,
                    day: named.day.clone(),
                    why: format!("topic:{}/{n}", named.day),
                };
                world_lines.push(line(&item));
                drawn.push(item);
                topics.push(Topic::Told(Told {
                    keyword: topic.keyword.clone(),
                    line: topic.line.clone(),
                    subjects: topic.subjects.clone(),
                    item: chosen.item,
                    related: chosen.related,
                    odds: chosen.odds,
                    rest: chosen.rest,
                }));
            }
            journey_lines.push(journey::line(&Day {
                day: named.day.clone(),
                topics,
            }));
        }
        Ok(Some((world_lines, journey_lines)))
    }

    /// journey R8: appends each planned line its file doesn't hold yet, the world's first, so a
    /// rerun completes an interrupted night and never removes or repeats a line (Principle I).
    pub(crate) fn draw_planned(&self, world: &[String], journey: &[String]) -> Result<()> {
        for (file, planned) in [(WORLD_FILE, world), (JOURNEY_FILE, journey)] {
            let text = std::fs::read_to_string(self.workspace_dir().join(file)).unwrap_or_default();
            let held: std::collections::HashSet<&str> = text.lines().collect();
            let missing: Vec<String> = planned
                .iter()
                .filter(|l| !held.contains(l.as_str()))
                .cloned()
                .collect();
            if !missing.is_empty() {
                self.append_workspace(file, &missing)?;
            }
        }
        Ok(())
    }

    /// Called when the main view opens: an older companion gets its id and world with no prompt.
    pub fn prepare_world(&self, clock: &LocalClock) -> Result<PrepareOutcome> {
        if !self.state()?.setup.complete {
            return Err(AppError::invalid("setup"));
        }
        let today = local_today(clock)?;
        let recreated = self.ensure_world(&today)?;
        // journey FR-005: days before the journey began never get topics.
        if self.state()?.world.journey_since.is_none() {
            self.update_state(|s| s.world.journey_since = Some(today))?;
        }
        Ok(PrepareOutcome { recreated })
    }

    /// world FR-026 to FR-031: the world as the file holds it, never worked out again (world FR-014). The land is not
    /// sent (world FR-007).
    pub fn world_view(&self) -> Result<WorldView> {
        let workspace = self.workspace_dir();
        let identity_text =
            std::fs::read_to_string(workspace.join("IDENTITY.md")).unwrap_or_default();
        let (Some(companion_id), born) = identity(&identity_text) else {
            return Err(AppError::invalid("setup"));
        };
        let world =
            World::parse(&std::fs::read_to_string(workspace.join(WORLD_FILE)).unwrap_or_default());
        // journey FR-035: a journey that can't be read is an empty one; the picture still shows.
        let journey_text =
            std::fs::read_to_string(workspace.join(JOURNEY_FILE)).unwrap_or_default();
        let journey = Journey::parse(&journey_text);
        let [ground, water, leaf, bark, stone, accent] = dna::palette(&companion_id);
        let topic_of = |d: &Drawn| {
            let (day, n) = d.why.strip_prefix("topic:")?.split_once('/')?;
            Some(TopicRef {
                day: day.into(),
                n: n.parse().ok()?,
            })
        };
        let keywords = |day: &str, own: Option<u32>| -> Vec<String> {
            let told: Vec<(u32, &str)> = journey
                .day(day)
                .map(|d| {
                    d.topics
                        .iter()
                        .enumerate()
                        .filter_map(|(n, t)| Some((n as u32, t.told()?.keyword.as_str())))
                        .collect()
                })
                .unwrap_or_default();
            let first = told.iter().filter(|(n, _)| Some(*n) == own);
            first
                .chain(told.iter().filter(|(n, _)| Some(*n) != own))
                .map(|(_, k)| (*k).to_owned())
                .collect()
        };
        let items: Vec<DrawnItem> = world
            .items
            .into_iter()
            .map(|d| {
                let topic = topic_of(&d);
                DrawnItem {
                    known: dictionary().iter().any(|e| e.id == d.item),
                    keywords: keywords(
                        &d.day,
                        topic.as_ref().filter(|t| t.day == d.day).map(|t| t.n),
                    ),
                    item: d.item,
                    x: d.x,
                    y: d.y,
                    size: d.size,
                    flip: d.flip,
                    shade: d.shade,
                    day: d.day,
                    topic,
                }
            })
            .collect();
        let mut days: Vec<&Day> = journey.days.iter().collect();
        days.sort_by(|a, b| b.day.cmp(&a.day));
        let journey_days = days
            .into_iter()
            .map(|day| JourneyDay {
                topics: day
                    .topics
                    .iter()
                    .enumerate()
                    .map(|(n, t)| JourneyTopic {
                        n: n as u32,
                        drawn: t.item().to_owned(),
                        item: items
                            .iter()
                            .position(|i| {
                                i.topic
                                    .as_ref()
                                    .is_some_and(|r| r.day == day.day && r.n == n as u32)
                            })
                            .map(|i| i as u32),
                        story: t.told().map(|told| Story {
                            keyword: told.keyword.clone(),
                            line: told.line.clone(),
                            subjects: told.subjects.clone(),
                            related: told.related,
                            odds: told
                                .odds
                                .iter()
                                .map(|(item, chance)| Odds {
                                    item: item.clone(),
                                    chance: *chance,
                                })
                                .collect(),
                            rest: told.rest,
                        }),
                    })
                    .collect(),
                scenery: items
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| {
                        i.day == day.day && i.topic.is_none() && !i.item.starts_with("home.")
                    })
                    .map(|(n, _)| n as u32)
                    .collect(),
                has_notes: workspace.join(format!("memory/{}.md", day.day)).exists(),
                day: day.day.clone(),
            })
            .collect();
        Ok(WorldView {
            makeup: journey::makeup(&journey)
                .into_iter()
                .map(|(subject, share)| SubjectShare { subject, share })
                .collect(),
            journey: journey_days,
            journey_skipped: journey.skipped as u32,
            journey_hash: fingerprint(&journey_text),
            palette: WorldPalette {
                ground,
                water,
                leaf,
                bark,
                stone,
                accent,
            },
            items,
            skipped_lines: world.skipped as u32,
            born,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dictionary_is_valid_and_keeps_every_shipped_id() {
        let items = dictionary();
        let mut ids: Vec<&str> = items.iter().map(|e| e.id.as_str()).collect();
        for id in &ids {
            let well_formed = id
                .split_once('.')
                .is_some_and(|(kind, name)| !kind.is_empty() && !name.is_empty())
                && id.bytes().all(|b| b.is_ascii_lowercase() || b == b'.');
            assert!(well_formed, "{id}");
        }
        assert!(items.iter().all(|e| (1..=4).contains(&e.stage)));
        assert!(items.iter().any(|e| e.home));
        for stage in 1..=4 {
            assert!(
                items.iter().any(|e| e.stage == stage && !e.home),
                "stage {stage}"
            );
        }
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "ids are unique");
        // world FR-010: an id that has shipped is never removed.
        let shipped = include_str!("../../tests/fixtures/world/shipped-ids.txt");
        for id in shipped.lines().filter(|l| !l.is_empty()) {
            assert!(ids.binary_search(&id).is_ok(), "{id} was removed");
        }
    }

    #[test]
    fn every_subject_has_everyday_objects_and_none_is_removed() {
        let known: Vec<&str> = subjects().iter().map(|s| s.id.as_str()).collect();
        let well_formed = |id: &str| {
            !id.is_empty()
                && id
                    .split('-')
                    .all(|w| !w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase()))
        };
        for subject in subjects() {
            assert!(well_formed(&subject.id), "{}", subject.id);
            assert!(!subject.about.is_empty());
            for related in &subject.related {
                assert!(
                    known.contains(&related.as_str()) && *related != subject.id,
                    "{} related {related}",
                    subject.id
                );
            }
            // journey FR-018: at least three everyday objects per subject.
            let items = dictionary()
                .iter()
                .filter(|e| e.subjects.contains(&subject.id))
                .count();
            assert!(items >= 3, "{} has {items}", subject.id);
        }
        assert_eq!(known.iter().filter(|id| **id == "other").count(), 1);
        for entry in dictionary().iter().filter(|e| !e.subjects.is_empty()) {
            assert!(
                entry.subjects.iter().all(|s| known.contains(&s.as_str())),
                "{}",
                entry.id
            );
            // journey FR-014: stages are for scenery; everyday objects are there from the first day.
            assert!(
                matches!(entry.land, Land::Ground | Land::Water) && entry.stage == 1 && !entry.home,
                "{}",
                entry.id
            );
        }
        let shipped = include_str!("../../tests/fixtures/world/shipped-subjects.txt");
        for id in shipped.lines().filter(|l| !l.is_empty()) {
            assert!(known.contains(&id), "{id} was removed");
        }
    }
}
