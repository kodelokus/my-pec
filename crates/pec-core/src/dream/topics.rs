//! Journey: the world phase's due days, its request and the validation of its answer (journey R2, R3,
//! journey interfaces).

use crate::chat::context::date_of_time_block;
use crate::chat::transcript::{Entry, Line, Provenance, Role};
use crate::memory::browser::comment_field;
use crate::memory::search::without_comment;
use crate::world::journey::{Named, NamedDay};
use crate::world::subjects;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(crate) const INSTRUCTIONS: &str = include_str!("../world/prompts/topics.md");
/// journey FR-005: one sweep names topics for at most this many days.
pub(crate) const MAX_DAYS: usize = 7;
const MAX_TOPICS: usize = 3;
const MESSAGE_CHARS: usize = 400;
const DAY_CHARS: usize = 3_000;
const NOTE_CHARS: usize = 1_000;
const KEYWORD_CHARS: usize = 30;
const KEYWORD_WORDS: usize = 3;
const LINE_CHARS: usize = 120;

/// A message the person typed, with the local day, weekday and time its time block names.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Typed {
    pub day: String,
    pub weekday: String,
    pub time: String,
    pub text: String,
}

/// Everything the person typed, in order: their own words only, never forgotten messages (journey FR-003, FR-004).
pub(crate) fn typed(lines: &[Line], forgotten: &HashSet<&str>) -> Vec<Typed> {
    lines
        .iter()
        .filter(|l| !forgotten.contains(l.id.as_str()))
        .filter_map(|l| match &l.entry {
            Entry::Message {
                role: Role::User,
                provenance: Provenance::User,
                content,
                text: Some(text),
                ..
            } => {
                let block = content.first()?["text"].as_str()?;
                let day = date_of_time_block(block)?;
                let rest = block.strip_prefix("<context>Local time: ")?;
                let weekday = rest.split_whitespace().next()?.to_owned();
                let time = rest.split(", ").nth(1)?.get(..5)?.to_owned();
                Some(Typed {
                    day,
                    weekday,
                    time,
                    text: text.clone(),
                })
            }
            _ => None,
        })
        .collect()
}

/// The days to name topics for, newest first: ended, on or after `since`, later than `considered`, with something
/// the person typed; at most `MAX_DAYS` (journey R2). Without `since` the journey hasn't begun (journey FR-005).
pub(crate) fn due_days(
    typed: &[Typed],
    today: &str,
    since: Option<&str>,
    considered: Option<&str>,
) -> Vec<String> {
    let Some(since) = since else {
        return Vec::new();
    };
    let days: BTreeSet<&str> = typed
        .iter()
        .map(|t| t.day.as_str())
        .filter(|d| *d < today && *d >= since)
        .filter(|d| considered.is_none_or(|c| *d > c))
        .collect();
    days.into_iter()
        .rev()
        .take(MAX_DAYS)
        .map(Into::into)
        .collect()
}

fn cut(text: &str, chars: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= chars {
        return text.to_owned();
    }
    let kept: String = text.chars().take(chars - 1).collect();
    format!("{}…", kept.trim_end())
}

/// A day's messages within `DAY_CHARS`: all of them when they fit, otherwise the first and then every s-th, with
/// the smallest step s that fits, so the sample spreads across the day.
fn sample(messages: &[String]) -> Vec<&String> {
    let size = |picked: &[&String]| picked.iter().map(|m| m.chars().count() + 9).sum::<usize>();
    for step in 1..=messages.len().max(1) {
        let picked: Vec<&String> = messages.iter().step_by(step).collect();
        if size(&picked) <= DAY_CHARS {
            return picked;
        }
    }
    messages.iter().take(1).collect()
}

/// The request's user message: for each day, newest first, what the person typed and the day's note lines the
/// person or the companion wrote (journey interfaces).
pub(crate) fn input(
    days: &[String],
    typed: &[Typed],
    notes: &BTreeMap<String, String>,
    forgotten: &HashSet<&str>,
) -> String {
    let mut out =
        String::from("Days to look back on, newest first. Only what the person wrote is shown.\n");
    for day in days {
        let of_day: Vec<&Typed> = typed.iter().filter(|t| &t.day == day).collect();
        let weekday = of_day.first().map_or("", |t| t.weekday.as_str());
        out.push_str(&format!("\n## {day} ({weekday})\nWhat they wrote:\n"));
        let messages: Vec<String> = of_day
            .iter()
            .map(|t| {
                format!(
                    "{} {}",
                    t.time,
                    cut(&t.text, MESSAGE_CHARS).replace('\n', " ")
                )
            })
            .collect();
        for message in sample(&messages) {
            out.push_str(&format!("- {message}\n"));
        }
        let mut room = NOTE_CHARS;
        let mut lines = Vec::new();
        for line in notes.get(day).map_or("", String::as_str).lines() {
            let provenance = comment_field(line, "from").unwrap_or_else(|| "user".into());
            let forgotten_src =
                comment_field(line, "src").is_some_and(|s| forgotten.contains(s.as_str()));
            let text = without_comment(line).trim();
            if !text.starts_with("- ")
                || forgotten_src
                || (provenance != "user" && provenance != "companion")
            {
                continue;
            }
            let Some(left) = room.checked_sub(text.chars().count() + 1) else {
                break;
            };
            room = left;
            lines.push(text.to_owned());
        }
        if !lines.is_empty() {
            out.push_str("Your notes from that day:\n");
            for line in lines {
                out.push_str(&line);
                out.push('\n');
            }
        }
    }
    out
}

/// The answer's structure, with the subject list as an enum.
pub(crate) fn schema() -> Value {
    let ids: Vec<&str> = subjects().iter().map(|s| s.id.as_str()).collect();
    let about: Vec<String> = subjects()
        .iter()
        .map(|s| format!("{}: {}", s.id, s.about))
        .collect();
    json!({
        "type": "object",
        "properties": {
            "days": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "day": { "type": "string", "description": "The date as given, YYYY-MM-DD." },
                        "topics": {
                            "type": "array",
                            "maxItems": MAX_TOPICS,
                            "items": {
                                "type": "object",
                                "properties": {
                                    "keyword": { "type": "string", "description": "At most three words, at most 30 characters, in the person's language." },
                                    "line": { "type": "string", "description": "One sentence of at most 120 characters, in your voice, to the person." },
                                    "subjects": {
                                        "type": "array",
                                        "minItems": 1,
                                        "maxItems": 2,
                                        "items": { "type": "string", "enum": ids },
                                        "description": about.join("; "),
                                    },
                                },
                                "required": ["keyword", "line", "subjects"],
                                "additionalProperties": false,
                            }
                        },
                    },
                    "required": ["day", "topics"],
                    "additionalProperties": false,
                }
            }
        },
        "required": ["days"],
        "additionalProperties": false,
    })
}

#[derive(Deserialize)]
struct Answer {
    days: Vec<AnsweredDay>,
}

#[derive(Deserialize)]
struct AnsweredDay {
    day: String,
    #[serde(default)]
    topics: Vec<Value>,
}

/// Plain text: trimmed, without control characters or markup brackets.
fn clean(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() && *c != '<' && *c != '>')
        .collect::<String>()
        .trim()
        .to_owned()
}

/// A topic that keeps the rules of journey FR-002, or none.
fn valid(topic: &Value) -> Option<Named> {
    let raw_line = topic["line"].as_str()?;
    if raw_line.contains(['\n', '\r']) {
        return None;
    }
    let keyword = clean(topic["keyword"].as_str()?);
    let line = clean(raw_line);
    let words = keyword.split_whitespace().count();
    let keyword_ok =
        (1..=KEYWORD_WORDS).contains(&words) && keyword.chars().count() <= KEYWORD_CHARS;
    let line_ok = !line.is_empty() && line.chars().count() <= LINE_CHARS;
    let mut chosen: Vec<String> = Vec::new();
    for subject in topic["subjects"].as_array()? {
        let id = subject.as_str()?;
        if !subjects().iter().any(|s| s.id == id) || chosen.iter().any(|c| c == id) {
            return None;
        }
        chosen.push(id.to_owned());
    }
    (keyword_ok && line_ok && (1..=2).contains(&chosen.len())).then_some(Named {
        keyword,
        line,
        subjects: chosen,
    })
}

/// Every asked day with the topics that pass validation, in the asked order; `None` if the reply isn't the
/// structure.
pub(crate) fn parse(content: &[Value], asked: &[String]) -> Option<Vec<NamedDay>> {
    let text = content.iter().find(|b| b["type"] == "text")?["text"].as_str()?;
    let answer: Answer = serde_json::from_str(text).ok()?;
    Some(
        asked
            .iter()
            .map(|day| NamedDay {
                day: day.clone(),
                topics: answer
                    .days
                    .iter()
                    .find(|d| &d.day == day)
                    .map(|d| d.topics.iter().filter_map(valid).take(MAX_TOPICS).collect())
                    .unwrap_or_default(),
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A transcript line as the app writes it: a message with its time block.
    fn message(id: &str, role: &str, provenance: &str, day: u32, time: &str, text: &str) -> Line {
        let block = format!(
            "<context>Local time: Tuesday {day} November 2026, {time} (Asia/Jakarta, UTC+07:00)</context>"
        );
        serde_json::from_value(json!({
            "id": id, "at": 1, "type": "message", "role": role, "provenance": provenance,
            "content": [{ "type": "text", "text": block }, { "type": "text", "text": text }], "text": text,
        }))
        .unwrap()
    }

    fn said(day: u32, text: &str) -> Line {
        message(
            &format!("{day}-{}", text.len()),
            "user",
            "user",
            day,
            "07:12",
            text,
        )
    }

    #[test]
    fn due_days_are_ended_days_the_person_talked_about_newest_first() {
        let mut lines: Vec<Line> = (1..=12).map(|d| said(d, "hello")).collect();
        lines.push(message(
            "x",
            "user",
            "system",
            13,
            "08:00",
            "hidden request",
        ));
        lines.push(message("y", "assistant", "user", 13, "08:00", "a reply"));
        let typed = typed(&lines, &HashSet::new());
        let due = |since, considered| due_days(&typed, "2026-11-12", since, considered);
        // Today (the 12th) never; the seven most recent ended days; nothing for a day without the person's words.
        let seven: Vec<String> = (5..=11).rev().map(|d| format!("2026-11-{d:02}")).collect();
        assert_eq!(due(Some("2026-11-01"), None), seven);
        assert_eq!(
            due(Some("2026-11-09"), None),
            ["2026-11-11", "2026-11-10", "2026-11-09"]
        );
        // A considered day, and every day before it, is never due again, even if its line was deleted by hand.
        assert_eq!(due(Some("2026-11-01"), Some("2026-11-10")), ["2026-11-11"]);
        // journey FR-005: before the journey began, nothing is due.
        assert!(due(None, None).is_empty());
        let forgotten = HashSet::from(["11-5"]);
        assert!(
            super::typed(&lines, &forgotten)
                .iter()
                .all(|t| t.day != "2026-11-11")
        );
    }

    #[test]
    fn the_input_holds_only_what_the_person_wrote_within_its_budget() {
        let mut lines = vec![
            said(10, "Just back from a 10 km run before work!"),
            message(
                "c",
                "assistant",
                "user",
                10,
                "07:13",
                "You should try yoga too.",
            ),
            message("t", "user", "untrusted", 10, "07:14", "tool result text"),
        ];
        // A 500-message day.
        for i in 0..500 {
            lines.push(message(
                &format!("m{i}"),
                "user",
                "user",
                9,
                "12:00",
                &format!("{i} {}", "word ".repeat(100)),
            ));
        }
        let typed = typed(&lines, &HashSet::new());
        let notes = BTreeMap::from([(
            "2026-11-10".to_owned(),
            "# 2026-11-10\n\n- 19:42 Sister's birthday is on 11 November. <!-- from:companion src:2-0 -->\n- 19:50 Tool said hi <!-- from:tool src:3-0 -->\n- 20:00 Forgotten thing <!-- from:user src:9-9 -->\n".to_owned(),
        )]);
        let days = vec!["2026-11-10".to_owned(), "2026-11-09".to_owned()];
        let input = input(&days, &typed, &notes, &HashSet::from(["9-9"]));
        assert!(input.contains("## 2026-11-10 (Tuesday)\nWhat they wrote:\n- 07:12 Just back from a 10 km run before work!\n"));
        assert!(
            input.contains(
                "Your notes from that day:\n- 19:42 Sister's birthday is on 11 November.\n"
            )
        );
        for absent in [
            "yoga",
            "tool result",
            "Tool said",
            "Forgotten thing",
            "<!--",
        ] {
            assert!(!input.contains(absent), "{absent}");
        }
        let long_day = input.split("## 2026-11-09").nth(1).unwrap();
        assert!(
            long_day.chars().count() <= 3_000 + 100,
            "{}",
            long_day.chars().count()
        );
        // The first message is kept, each cut to 400 characters, and the sample spreads across the day.
        assert!(long_day.contains("- 12:00 0 word"));
        assert!(
            long_day
                .lines()
                .filter(|l| l.starts_with("- "))
                .all(|l| l.chars().count() <= 400 + 8)
        );
        assert!(long_day.contains("…"));
        assert!(
            long_day
                .lines()
                .any(|l| l.starts_with("- 12:00 4") && l.len() > 10)
        );
    }

    fn reply(days: Value) -> Vec<Value> {
        vec![json!({ "type": "text", "text": json!({ "days": days }).to_string() })]
    }

    fn topic(keyword: &str, line: &str, subjects: &[&str]) -> Value {
        json!({ "keyword": keyword, "line": line, "subjects": subjects })
    }

    #[test]
    fn answers_keep_only_topics_that_follow_the_rules() {
        let good = topic(
            "10 km run",
            "You ran ten kilometres before work.",
            &["running"],
        );
        let cases = [
            (topic("one two three four", "A line.", &["running"]), false),
            (topic(&"k".repeat(31), "A line.", &["running"]), false),
            (topic("run", "", &["running"]), false),
            (topic("run", &"l".repeat(121), &["running"]), false),
            (topic("run", "Two\nlines.", &["running"]), false),
            (topic("run", "A line.", &["jogging"]), false),
            (topic("run", "A line.", &["running", "running"]), false),
            (
                topic("run", "A line.", &["running", "sport", "walking"]),
                false,
            ),
            (topic("run", "A line.", &[]), false),
            (
                topic(" <b>run</b> ", "A\u{7} line.", &["running", "sport"]),
                true,
            ),
        ];
        for (case, kept) in cases {
            let parsed = parse(
                &reply(json!([{ "day": "2026-11-10", "topics": [case.clone()] }])),
                &["2026-11-10".into()],
            )
            .unwrap();
            assert_eq!(parsed[0].topics.len(), usize::from(kept), "{case}");
        }
        let cleaned = parse(
            &reply(json!([{ "day": "2026-11-10", "topics": [cases_clean()] }])),
            &["2026-11-10".into()],
        )
        .unwrap();
        assert_eq!(cleaned[0].topics[0].keyword, "brun/b");
        assert_eq!(cleaned[0].topics[0].line, "A line.");
        // A day not asked about is dropped, a fourth topic too, and an asked day left out has no topics.
        let asked = vec!["2026-11-10".to_owned(), "2026-11-09".to_owned()];
        let four = json!([{ "day": "2026-11-10", "topics": [good, good, good, good] }, { "day": "2026-10-01", "topics": [good] }]);
        let parsed = parse(&reply(four), &asked).unwrap();
        assert_eq!(
            parsed
                .iter()
                .map(|d| (d.day.as_str(), d.topics.len()))
                .collect::<Vec<_>>(),
            [("2026-11-10", 3), ("2026-11-09", 0)]
        );
        assert_eq!(
            parse(&[json!({ "type": "text", "text": "not json" })], &asked),
            None
        );
    }

    fn cases_clean() -> Value {
        topic(" <b>run</b> ", "A\u{7} line.", &["running"])
    }
}
