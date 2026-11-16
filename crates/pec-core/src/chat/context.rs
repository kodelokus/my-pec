//! What the companion is given: the system prompt, the time block, notices, and the replayed messages
//! (chat R7 to R9). The same entries always produce the same request (Principle III).

use super::transcript::{Entry, Line, Provenance, ReplyState, Role};
use crate::AppError;
use crate::llm::ToolSpec;
use crate::memory::files::{LocalNow, append_line, previous_day};
use crate::memory::search::{Hit, Loaded};
use crate::memory::tools::{self, describe};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use ts_rs::TS;

const HARNESS: &str = include_str!("harness.md");
/// The rules of contexts started before memory existed; they keep them, and no tools, until they end (memory R2).
const HARNESS_CHAT_1: &str = include_str!("harness-chat-1.md");
/// The rules of contexts started before the world journey; they keep them until they end (journey R11).
const HARNESS_MEMORY_1: &str = include_str!("harness-memory-1.md");
/// The rules of contexts started before the mood marker; they keep them until they end (avatar R2).
const HARNESS_WORLD_1: &str = include_str!("harness-world-1.md");
/// The rules of contexts started before extensions; they keep them, byte for byte, until they end (extensions R2).
const HARNESS_AVATAR_1: &str = include_str!("harness-avatar-1.md");
/// The rules of contexts started before each extension brought its own SKILL.md; they keep them until they end.
const HARNESS_EXT_1: &str = include_str!("harness-ext-1.md");
/// The rules of contexts started before "How you talk"; they keep them until they end.
const HARNESS_EXT_2: &str = include_str!("harness-ext-2.md");
/// The rules of contexts started before the five newer moods; they keep them until they end.
const HARNESS_TALK_1: &str = include_str!("harness-talk-1.md");
pub(crate) const HARNESS_VERSION: &str = "mood-2";

/// The harness rules and the tools of a context, by the version its `context` entry recorded, and whether the
/// SKILL.md of each extension it recorded follows the rules (from `ext-2` on).
pub(crate) fn harness(version: &str) -> (&'static str, Vec<ToolSpec>, bool) {
    match version {
        "chat-1" => (HARNESS_CHAT_1, Vec::new(), false),
        "memory-1" => (HARNESS_MEMORY_1, tools::specs(), false),
        "world-1" => (HARNESS_WORLD_1, tools::specs(), false),
        "avatar-1" => (HARNESS_AVATAR_1, tools::specs(), false),
        "ext-1" => (HARNESS_EXT_1, tools::specs(), false),
        "ext-2" => (HARNESS_EXT_2, tools::specs(), true),
        "talk-1" => (HARNESS_TALK_1, tools::specs(), true),
        _ => (HARNESS, tools::specs(), true),
    }
}

/// The files given to the companion, with their size budgets in characters (MEM-9, chat FR-012).
pub(crate) const FILES: [(&str, Option<usize>); 4] = [
    ("IDENTITY.md", None),
    ("SOUL.md", Some(8_000)),
    ("USER.md", Some(4_000)),
    ("MEMORY.md", Some(12_000)),
];

/// The user's clock, passed by the webview with each command that sends (chat R8).
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct LocalClock {
    /// RFC 3339 local time with offset, e.g. `2026-11-13T21:40:05+07:00`.
    pub iso: String,
    /// The IANA time zone, e.g. `Asia/Jakarta`.
    pub zone: String,
}

/// Each daily note loaded when a context starts keeps its last 4,000 characters (memory R8).
const NOTE_BUDGET: usize = 4_000;
const NOTE_CUT: &str = "(Earlier notes from this day are left out.)\n";

/// Today's and yesterday's daily notes, each cut to its most recent part (memory FR-014, FR-015), with `staged` lines
/// added to today's as appending them would.
pub(crate) fn read_notes(
    workspace: &Path,
    today: &str,
    staged: &[String],
) -> BTreeMap<String, String> {
    [previous_day(today), today.to_owned()]
        .into_iter()
        .filter_map(|date| {
            let mut text = std::fs::read_to_string(workspace.join(format!("memory/{date}.md")))
                .unwrap_or_default();
            if date == today {
                for line in staged {
                    text = append_line(&text, today, line).0;
                }
            }
            (!text.trim().is_empty()).then(|| (date, note_tail(&text)))
        })
        .collect()
}

/// The most recent whole lines that fit the budget, marked as cut.
fn note_tail(text: &str) -> String {
    if text.chars().count() <= NOTE_BUDGET {
        return text.to_owned();
    }
    let room = NOTE_BUDGET - NOTE_CUT.chars().count();
    let mut kept = 0;
    let start = text
        .lines()
        .rev()
        .take_while(|line| {
            kept += line.chars().count() + 1;
            kept <= room
        })
        .count();
    let lines: Vec<&str> = text.lines().collect();
    let tail = if start == 0 {
        let skip = text.chars().count() - room;
        text.chars().skip(skip).collect()
    } else {
        lines[lines.len() - start..].join("\n") + "\n"
    };
    format!("{NOTE_CUT}{tail}")
}

/// The user's local date and time from the clock of the command that caused a write.
pub(crate) fn local_now(clock: &LocalClock) -> LocalNow {
    LocalNow {
        date: clock.iso.get(..10).unwrap_or_default().into(),
        time: clock.iso.get(11..16).unwrap_or_default().into(),
    }
}

/// The daily notes snapshotted in the current context, as `memory/YYYY-MM-DD.md`, that were cut to budget.
pub(crate) fn shortened_notes(lines: &[Line]) -> Vec<String> {
    match current(lines).first().map(|l| &l.entry) {
        Some(Entry::Context {
            notes: Some(notes), ..
        }) => notes
            .iter()
            .filter(|(_, text)| text.starts_with(NOTE_CUT))
            .map(|(date, _)| format!("memory/{date}.md"))
            .collect(),
        _ => Vec::new(),
    }
}

/// The message that opens a context: the compaction summary, if any, then the daily notes (architecture 5.2's
/// epoch opener, compaction R9).
fn opener(
    summary: Option<&str>,
    dream: Option<&str>,
    notes: &BTreeMap<String, String>,
) -> Option<Value> {
    let mut parts = Vec::new();
    if let Some(summary) = summary {
        parts.push(format!(
            "<memory>A summary of your earlier conversation, written when it was compacted (not typed by the person):</memory>\n\n{}",
            summary.trim_end()
        ));
    }
    // Dreaming R10: last night's diary entry, once, as a dream and not a fact.
    if let Some(dream) = dream {
        parts.push(format!(
            "<memory>The dream you wrote in your diary last night (a dream, not a fact):</memory>\n\n{}",
            dream.trim_end()
        ));
    }
    if !notes.is_empty() {
        let mut text = "<memory>Your daily notes from yesterday and today (your own notes, not typed by the person):</memory>"
            .to_owned();
        for note in notes.values() {
            text.push_str("\n\n");
            text.push_str(note.trim_end());
        }
        parts.push(text);
    }
    (!parts.is_empty())
        .then(|| json!({ "role": "user", "content": [text_block(&parts.join("\n\n"))] }))
}

/// What the current context already holds: its notes' days and the lines of long-term memory its budget let
/// through (memory R11).
pub(crate) fn loaded(lines: &[Line]) -> Loaded {
    match current(lines).first().map(|l| &l.entry) {
        Some(Entry::Context { files, notes, .. }) => Loaded {
            days: notes.iter().flat_map(|n| n.keys().cloned()).collect(),
            memory_lines: budgeted(
                "MEMORY.md",
                files.get("MEMORY.md").map_or("", String::as_str),
            )
            .0
            .lines()
            .count(),
        },
        _ => Loaded {
            days: Vec::new(),
            memory_lines: 0,
        },
    }
}

/// The auto-recall block, or nothing when no hit qualified.
pub(crate) fn recall_block(hits: &[Hit]) -> Option<Value> {
    if hits.is_empty() {
        return None;
    }
    let mut text = "<memory>Possibly relevant notes from your memory (found automatically, not typed by the person):</memory>\n"
        .to_owned();
    for hit in hits {
        text.push('\n');
        text.push_str(&describe(hit));
    }
    Some(text_block(&text))
}

/// The `tool_result` blocks of one tool-result message, in the order of the calls (memory R3).
pub(crate) fn tool_results<'a>(
    results: impl Iterator<Item = (&'a str, &'a str, bool)>,
) -> Vec<Value> {
    results
        .map(|(id, content, is_error)| {
            let mut block = json!({ "type": "tool_result", "tool_use_id": id, "content": content });
            if is_error {
                block["is_error"] = json!(true);
            }
            block
        })
        .collect()
}

/// The `tool_use` blocks a reply asks to have answered.
pub(crate) fn tool_calls(content: &[Value]) -> Vec<(String, String, Value)> {
    content
        .iter()
        .filter(|b| b["type"] == "tool_use")
        .map(|b| {
            (
                b["id"].as_str().unwrap_or_default().to_owned(),
                b["name"].as_str().unwrap_or_default().to_owned(),
                b["input"].clone(),
            )
        })
        .collect()
}

pub(crate) fn read_files(workspace: &Path) -> BTreeMap<String, String> {
    FILES
        .iter()
        .map(|(name, _)| {
            let text = std::fs::read_to_string(workspace.join(name)).unwrap_or_default();
            ((*name).to_owned(), text)
        })
        .collect()
}

/// The budgeted part of a file, and whether it was cut.
fn budgeted(name: &str, text: &str) -> (String, bool) {
    let limit = FILES.iter().find(|(n, _)| *n == name).and_then(|(_, l)| *l);
    match limit {
        Some(limit) if text.chars().count() > limit => (text.chars().take(limit).collect(), true),
        _ => (text.to_owned(), false),
    }
}

fn section(files: &BTreeMap<String, String>, name: &str) -> String {
    let text = files.get(name).map_or("", String::as_str);
    format!("## {name}\n\n{}", budgeted(name, text).0)
}

/// The three system blocks: harness, then identity, soul and the world, then profile and memory.
pub(crate) fn system(
    harness: &str,
    files: &BTreeMap<String, String>,
    world: Option<&str>,
) -> Vec<String> {
    let mut identity = format!(
        "{}\n\n{}",
        section(files, "IDENTITY.md"),
        section(files, "SOUL.md")
    );
    // Journey R9: the world description the context was given, after the soul.
    if let Some(world) = world {
        identity.push_str("\n\n");
        identity.push_str(world);
    }
    vec![
        harness.to_owned(),
        identity,
        format!(
            "{}\n\n{}",
            section(files, "USER.md"),
            section(files, "MEMORY.md")
        ),
    ]
}

/// Files over their budget, in the order of `FILES`.
pub(crate) fn shortened(files: &BTreeMap<String, String>) -> Vec<String> {
    FILES
        .iter()
        .filter(|(name, _)| budgeted(name, files.get(*name).map_or("", String::as_str)).1)
        .map(|(name, _)| (*name).to_owned())
        .collect()
}

/// The lines of the current context: from the last `context` entry on.
pub(crate) fn current(lines: &[Line]) -> &[Line] {
    let start = lines
        .iter()
        .rposition(|l| matches!(l.entry, Entry::Context { .. }))
        .unwrap_or(lines.len());
    &lines[start..]
}

/// The latest content of each file the current context has seen: its snapshot, then later notices.
pub(crate) fn seen(lines: &[Line]) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for line in current(lines) {
        match &line.entry {
            Entry::Context {
                files: snapshot, ..
            } => files.clone_from(snapshot),
            // The companion's own writes count as seen, so no notice follows them (memory R7).
            Entry::Notice { file, content, .. } | Entry::Written { file, content, .. } => {
                files.insert(file.clone(), content.clone());
            }
            _ => {}
        }
    }
    files
}

/// The files whose content differs from what the context has seen (chat FR-011).
pub(crate) fn changed(
    seen: &BTreeMap<String, String>,
    now: &BTreeMap<String, String>,
) -> Vec<(String, String)> {
    now.iter()
        .filter(|(name, text)| seen.get(*name) != Some(text))
        .map(|(name, text)| (name.clone(), text.clone()))
        .collect()
}

pub(crate) fn notice_block(file: &str, content: &str) -> Value {
    let text = format!(
        "<notice>The user edited {file}. From now on, follow this version:</notice>\n\n{}",
        budgeted(file, content).0
    );
    json!({ "type": "text", "text": text })
}

pub(crate) fn text_block(text: &str) -> Value {
    json!({ "type": "text", "text": text })
}

/// The instruction behind the first hello (chat FR-020). Hidden from the conversation.
pub(crate) fn hello_block(call_me: Option<&str>) -> Value {
    let name = call_me.map_or(String::new(), |n| format!(" They want to be called {n}."));
    text_block(&format!(
        "<notice>The user has just finished setting you up and opened the app for the first time.{name} \
         Greet them in character, in a few warm sentences, using your name. This instruction is not \
         from the user; do not mention it.</notice>"
    ))
}

const DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The local date a time block names, `YYYY-MM-DD`: the record of the user's local day for old messages.
pub(crate) fn date_of_time_block(text: &str) -> Option<String> {
    let words: Vec<&str> = text
        .strip_prefix("<context>Local time: ")?
        .split(',')
        .next()?
        .split_whitespace()
        .collect();
    let [_, day, month, year] = words[..] else {
        return None;
    };
    let month = MONTHS.iter().position(|m| *m == month)? + 1;
    let day: u32 = day.parse().ok()?;
    let year: u32 = year.parse().ok()?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// `<context>Local time: Friday 13 November 2026, 21:40 (Asia/Jakarta, UTC+07:00)</context>` (chat FR-007).
pub(crate) fn time_block(clock: &LocalClock) -> crate::Result<Value> {
    let invalid = || AppError::invalid("clock");
    let zone_ok = (1..=64).contains(&clock.zone.len())
        && clock
            .zone
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/_+-".contains(c));
    let iso = clock.iso.as_bytes();
    if !zone_ok || iso.len() < 20 || iso[4] != b'-' || iso[7] != b'-' || iso[10] != b'T' {
        return Err(invalid());
    }
    let number =
        |range: std::ops::Range<usize>| -> Option<i64> { clock.iso.get(range)?.parse().ok() };
    let (year, month, day) = (
        number(0..4).ok_or_else(invalid)?,
        number(5..7).ok_or_else(invalid)?,
        number(8..10).ok_or_else(invalid)?,
    );
    let (hour, minute) = (
        number(11..13).ok_or_else(invalid)?,
        number(14..16).ok_or_else(invalid)?,
    );
    let tail = clock.iso[19..].trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    let offset = match tail {
        "Z" => "+00:00",
        t if t.len() == 6
            && (t.starts_with('+') || t.starts_with('-'))
            && t.as_bytes()[3] == b':' =>
        {
            t
        }
        _ => return Err(invalid()),
    };
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return Err(invalid());
    }
    let weekday = DAYS[days_from_civil(year, month, day).rem_euclid(7) as usize];
    Ok(text_block(&format!(
        "<context>Local time: {weekday} {day} {} {year}, {hour:02}:{minute:02} ({}, UTC{offset})</context>",
        MONTHS[(month - 1) as usize],
        clock.zone
    )))
}

/// Days since 1970-01-01 shifted so that 0 is a Monday (Howard Hinnant's days-from-civil).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    // 1970-01-01 was a Thursday, index 3.
    era * 146_097 + doe - 719_468 + 3
}

/// The ids of messages a `replaced` entry took out of the conversation: from its target, the first reply of the
/// turn, up to the entry itself (chat data 2).
pub(crate) fn replaced(lines: &[Line]) -> HashSet<&str> {
    let mut gone = HashSet::new();
    for (end, line) in lines.iter().enumerate() {
        if let Entry::Replaced { target } = &line.entry
            && let Some(start) = lines[..end].iter().rposition(|l| l.id == *target)
        {
            gone.extend(
                lines[start..end]
                    .iter()
                    .filter(|l| matches!(l.entry, Entry::Message { .. }))
                    .map(|l| l.id.as_str()),
            );
        }
    }
    gone
}

/// The messages of the current context, by the replay rules of chat R7: the opener, the turns a
/// compaction kept as text only (compaction R8), then everything after the `context` entry.
pub(crate) fn messages(lines: &[Line]) -> Vec<Value> {
    let gone = replaced(lines);
    let forgotten = forgotten(lines);
    let current = current(lines);
    let opened = opened_by(lines);
    let (notes, dream) = match current.first().map(|l| &l.entry) {
        Some(Entry::Context { notes, dream, .. }) => {
            (notes.clone().unwrap_or_default(), dream.as_deref())
        }
        _ => (BTreeMap::new(), None),
    };
    let kept: Vec<&Line> = opened.as_ref().map_or_else(Vec::new, |o| {
        lines[o.first_kept..o.at]
            .iter()
            .filter(|l| matches!(l.entry, Entry::Message { .. }) && !gone.contains(l.id.as_str()))
            .collect()
    });
    let kept = turns(&kept)
        .into_iter()
        .filter(|t| !forgotten.contains(t.user.id.as_str()))
        .flat_map(|t| kept_turn(&t));
    opener(opened.as_ref().and_then(|o| o.summary), dream, &notes)
        .into_iter()
        .chain(kept)
        .chain(
            current
                .iter()
                .filter(|l| !gone.contains(l.id.as_str()))
                .filter_map(replay),
        )
        .collect()
}

/// The turns the current context took over: those its compaction kept, with the summary, or those it carried
/// from the context before.
pub(crate) struct Opened<'a> {
    pub summary: Option<&'a str>,
    /// Index of the first kept message, and of the line the kept ones end before.
    pub first_kept: usize,
    pub at: usize,
}

pub(crate) fn opened_by(lines: &[Line]) -> Option<Opened<'_>> {
    let start = lines
        .iter()
        .rposition(|l| matches!(l.entry, Entry::Context { .. }))?;
    let id = match &lines[start].entry {
        Entry::Context {
            carried: Some(first),
            ..
        } => {
            return Some(Opened {
                summary: None,
                first_kept: lines[..start].iter().position(|l| &l.id == first)?,
                at: start,
            });
        }
        Entry::Context {
            compaction: Some(id),
            ..
        } => id,
        _ => return None,
    };
    let at = lines.iter().rposition(|l| &l.id == id)?;
    let Entry::Compaction {
        summary,
        first_kept,
        ..
    } = &lines[at].entry
    else {
        return None;
    };
    let first_kept = lines[..at].iter().position(|l| &l.id == first_kept)?;
    Some(Opened {
        summary: Some(summary),
        first_kept,
        at,
    })
}

/// Every message the current context holds, oldest first: the kept turns' lines, then the lines after its
/// `context` entry; and the index where the latter begin (compaction R6).
pub(crate) fn held(lines: &[Line]) -> (Vec<&Line>, usize) {
    let is_message = |l: &&Line| matches!(l.entry, Entry::Message { .. });
    let mut held: Vec<&Line> = opened_by(lines).map_or_else(Vec::new, |o| {
        lines[o.first_kept..o.at]
            .iter()
            .filter(is_message)
            .collect()
    });
    let from = held.len();
    held.extend(current(lines).iter().filter(is_message));
    (held, from)
}

/// The ids of messages the user forgot memories from (memory R14).
pub(crate) fn forgotten(lines: &[Line]) -> HashSet<&str> {
    lines
        .iter()
        .filter_map(|l| match &l.entry {
            Entry::Forgotten { messages } => Some(messages),
            _ => None,
        })
        .flatten()
        .map(String::as_str)
        .collect()
}

/// The `compaction` entries a `context` entry names: only those count (compaction R5).
pub(crate) fn committed(lines: &[Line]) -> HashSet<&str> {
    lines
        .iter()
        .filter_map(|l| match &l.entry {
            Entry::Context {
                compaction: Some(id),
                ..
            } => Some(id.as_str()),
            _ => None,
        })
        .collect()
}

/// A message that opens a turn: one the person typed, or the hidden hello request.
pub(crate) fn is_turn_start(line: &Line) -> bool {
    matches!(
        line.entry,
        Entry::Message {
            role: Role::User,
            provenance: Provenance::User | Provenance::System,
            ..
        }
    )
}

/// A turn: the message that opened it, its replies and its tool results, in order.
pub(crate) struct Turn<'a> {
    pub user: &'a Line,
    pub rest: Vec<&'a Line>,
}

/// Groups messages into turns; messages before the first turn's start are left out.
pub(crate) fn turns<'a>(messages: &[&'a Line]) -> Vec<Turn<'a>> {
    let mut out: Vec<Turn<'a>> = Vec::new();
    for line in messages {
        if is_turn_start(line) {
            out.push(Turn {
                user: line,
                rest: Vec::new(),
            });
        } else if let Some(turn) = out.last_mut() {
            turn.rest.push(line);
        }
    }
    out
}

/// The text of a turn's replies, in order.
pub(crate) fn turn_text(turn: &Turn) -> Vec<String> {
    turn.rest
        .iter()
        .filter_map(|l| match &l.entry {
            Entry::Message {
                role: Role::Assistant,
                content,
                ..
            } => Some(reply_text(content)),
            _ => None,
        })
        .filter(|t| !t.is_empty())
        .collect()
}

/// A kept turn as text only: the user's message as saved, then one reply with the turn's text, without
/// thinking, tool calls or tool results (compaction R8).
fn kept_turn(turn: &Turn) -> Vec<Value> {
    let Entry::Message { content, .. } = &turn.user.entry else {
        return Vec::new();
    };
    let mut out = vec![json!({ "role": "user", "content": content })];
    let text: Vec<Value> = turn_text(turn).iter().map(|t| text_block(t)).collect();
    if !text.is_empty() {
        out.push(json!({ "role": "assistant", "content": text }));
    }
    out
}

fn replay(line: &Line) -> Option<Value> {
    match &line.entry {
        Entry::Message {
            role: Role::User,
            content,
            ..
        } => Some(json!({ "role": "user", "content": content })),
        Entry::Message {
            role: Role::Assistant,
            content,
            state,
            ..
        } => {
            let blocks = replayed(content, state.unwrap_or(ReplyState::Complete));
            (!blocks.is_empty()).then(|| json!({ "role": "assistant", "content": blocks }))
        }
        _ => None,
    }
}

/// A complete reply goes back as received, minus what the API forbids echoing; a cut-off one as its text.
fn replayed(content: &[Value], state: ReplyState) -> Vec<Value> {
    if state != ReplyState::Complete {
        return content
            .iter()
            .filter(|b| b["type"] == "text" && b["text"].as_str().is_some_and(|t| !t.is_empty()))
            .cloned()
            .collect();
    }
    let boundary = content.iter().rposition(|b| b["type"] == "fallback");
    content
        .iter()
        .enumerate()
        .filter(|(_, b)| b.get("_unmerged").is_none())
        .filter(|(i, b)| {
            let before_fallback = boundary.is_some_and(|f| *i < f);
            let internal = matches!(
                b["type"].as_str(),
                Some("thinking" | "redacted_thinking" | "tool_use")
            );
            !(before_fallback && internal)
        })
        .map(|(_, b)| b.clone())
        .collect()
}

/// The text of a reply, as shown in the conversation.
pub(crate) fn reply_text(content: &[Value]) -> String {
    content
        .iter()
        .filter(|b| b["type"] == "text")
        .filter_map(|b| b["text"].as_str())
        .collect()
}

/// Whether a line is a message the user never sees (the first-hello request).
pub(crate) fn hidden(line: &Line) -> bool {
    matches!(
        line.entry,
        Entry::Message {
            provenance: Provenance::System,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(iso: &str, zone: &str) -> LocalClock {
        LocalClock {
            iso: iso.into(),
            zone: zone.into(),
        }
    }

    #[test]
    fn the_time_block_names_the_local_day_time_and_zone() {
        let block = time_block(&clock("2026-11-13T21:40:05.123+07:00", "Asia/Jakarta")).unwrap();
        assert_eq!(
            block["text"],
            "<context>Local time: Friday 13 November 2026, 21:40 (Asia/Jakarta, UTC+07:00)</context>"
        );
        let utc = time_block(&clock("2024-02-29T00:05:00Z", "UTC")).unwrap();
        assert_eq!(
            utc["text"],
            "<context>Local time: Thursday 29 February 2024, 00:05 (UTC, UTC+00:00)</context>"
        );
    }

    #[test]
    fn a_malformed_clock_is_invalid_input() {
        for (iso, zone) in [
            ("2026-11-13 21:40:05+07:00", "Asia/Jakarta"),
            ("2026-13-13T21:40:05+07:00", "Asia/Jakarta"),
            ("2026-11-13T21:40:05", "Asia/Jakarta"),
            ("2026-11-13T21:40:05+07:00", ""),
            ("2026-11-13T21:40:05+07:00", "Asia/Jakarta<script>"),
        ] {
            assert_eq!(
                time_block(&clock(iso, zone)),
                Err(AppError::invalid("clock")),
                "{iso} {zone}"
            );
        }
    }
}

/// Compaction R8, R9: how a context opened by a compaction is replayed.
#[cfg(test)]
mod compacted_tests {
    use super::*;
    use crate::chat::transcript::{CompactionUsage, FlushOutcome, FlushRecord};

    fn line(id: &str, entry: Entry) -> Line {
        Line {
            id: id.into(),
            at: 0,
            entry,
        }
    }

    fn message(role: Role, provenance: Provenance, content: Vec<Value>) -> Entry {
        Entry::Message {
            role,
            provenance,
            content,
            text: None,
            zone: None,
            state: Some(ReplyState::Complete),
            stop_reason: None,
            model: None,
            usage: None,
            recalled: None,
            world: None,
            marks: Vec::new(),
        }
    }

    fn user(text: &str) -> Entry {
        message(Role::User, Provenance::User, vec![text_block(text)])
    }

    fn reply(blocks: Vec<Value>) -> Entry {
        message(Role::Assistant, Provenance::Companion, blocks)
    }

    fn context(compaction: Option<&str>) -> Entry {
        Entry::Context {
            model: "m".into(),
            files: BTreeMap::new(),
            harness: HARNESS_VERSION.into(),
            notes: Some(BTreeMap::from([(
                "2026-11-14".into(),
                "# 2026-11-14\n\n- 09:00 Tea.\n".into(),
            )])),
            compaction: compaction.map(Into::into),
            dream: None,
            world: None,
            extensions: Vec::new(),
            skills: None,
            carried: None,
        }
    }

    fn compaction(summary: &str, first_kept: &str) -> Entry {
        Entry::Compaction {
            summary: summary.into(),
            first_kept: first_kept.into(),
            tokens_before: 0,
            kept_tokens: 0,
            model: "m".into(),
            usage: CompactionUsage::default(),
            flush: FlushRecord {
                outcome: FlushOutcome::Nothing,
                notes: Vec::new(),
            },
            focus: None,
        }
    }

    fn thinking() -> Value {
        json!({ "type": "thinking", "thinking": "", "signature": "s" })
    }

    /// Turns u1 (forgotten), u2 (with a tool round), u3 (replaced reply) and u4, compacted from u2 on.
    fn conversation() -> Vec<Line> {
        vec![
            line("c1", context(None)),
            line("u1", user("my secret")),
            line("r1", reply(vec![text_block("Kept safe.")])),
            line("u2", user("note this")),
            line(
                "r2a",
                reply(vec![
                    thinking(),
                    text_block("On it."),
                    json!({ "type": "tool_use", "id": "t", "name": "memory_note", "input": {} }),
                ]),
            ),
            line(
                "t2",
                message(
                    Role::User,
                    Provenance::Untrusted,
                    vec![json!({ "type": "tool_result", "tool_use_id": "t", "content": "ok" })],
                ),
            ),
            line("r2b", reply(vec![thinking(), text_block("Noted.")])),
            line("u3", user("again")),
            line("r3", reply(vec![text_block("first try")])),
            line(
                "x3",
                Entry::Replaced {
                    target: "r3".into(),
                },
            ),
            line("u4", user("hello?")),
            line("r4", reply(vec![thinking()])),
            line(
                "f",
                Entry::Forgotten {
                    messages: vec!["u1".into()],
                },
            ),
            line("k", compaction("## Where we are\nTea.", "u1")),
            line("c2", context(Some("k"))),
            line("u5", user("new")),
            line("r5", reply(vec![thinking(), text_block("Fresh.")])),
        ]
    }

    #[test]
    fn kept_turns_are_replayed_as_text_after_the_summary() {
        let lines = conversation();
        let replayed = messages(&lines);
        let opener = replayed[0]["content"][0]["text"].as_str().unwrap();
        assert!(opener.starts_with("<memory>A summary of your earlier conversation"));
        assert!(opener.contains("## Where we are\nTea.\n\n<memory>Your daily notes"));
        assert!(opener.ends_with("- 09:00 Tea."));
        assert_eq!(
            replayed[1..],
            [
                json!({ "role": "user", "content": [text_block("note this")] }),
                json!({ "role": "assistant", "content": [text_block("On it."), text_block("Noted.")] }),
                json!({ "role": "user", "content": [text_block("again")] }),
                json!({ "role": "user", "content": [text_block("hello?")] }),
                json!({ "role": "user", "content": [text_block("new")] }),
                json!({ "role": "assistant", "content": [thinking(), text_block("Fresh.")] }),
            ]
        );
        // The same lines always give the same request.
        assert_eq!(messages(&lines), replayed);
    }

    #[test]
    fn a_second_compaction_can_keep_turns_from_before_the_first() {
        let mut lines = conversation();
        lines.push(line("k2", compaction("## Where we are\nLater.", "u4")));
        lines.push(line("c3", context(Some("k2"))));
        let replayed = messages(&lines);
        assert!(
            replayed[0]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Later.")
        );
        assert_eq!(
            replayed[1..],
            [
                json!({ "role": "user", "content": [text_block("hello?")] }),
                json!({ "role": "user", "content": [text_block("new")] }),
                json!({ "role": "assistant", "content": [text_block("Fresh.")] }),
            ]
        );
        let (held, from) = held(&lines);
        assert_eq!(
            held.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
            ["u4", "r4", "u5", "r5"]
        );
        assert_eq!(from, 4);
    }

    #[test]
    fn a_compaction_without_its_context_entry_does_not_count() {
        let mut lines = conversation();
        lines.push(line("k2", compaction("never committed", "u5")));
        assert!(!committed(&lines).contains("k2"));
        assert!(committed(&lines).contains("k"));
        assert!(
            !messages(&lines)[0]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("never committed")
        );
    }
}
