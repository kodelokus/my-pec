//! The five memory tools of architecture 5.4: their specs, input validation and execution
//! (memory interfaces, memory R1).

use super::files::{LocalNow, MemoryPath, Origin};
use super::search::Hit;
use super::{MarkKind, MemoryMark};
use crate::Core;
use crate::llm::ToolSpec;
use serde_json::{Value, json};

/// One string or integer property: its name, JSON type and description.
type Property = (&'static str, &'static str, &'static str);

const TOOLS: [(&str, &str, &[Property]); 5] = [
    (
        "memory_search",
        "Search your memory (long-term memory and all daily notes) by keywords. Call it when the person refers to \
         something from the past that is not in your context, or asks what you remember about something.",
        &[(
            "query",
            "string",
            "Keywords to look for, 1 to 500 characters, in the language the memory was written in.",
        )],
    ),
    (
        "memory_get",
        "Read a memory file or part of it, usually after a search, to see more context around a hit.",
        &[
            (
                "path",
                "string",
                "MEMORY.md, USER.md or memory/YYYY-MM-DD.md, as a search result names it.",
            ),
            ("from_line", "integer", "The first line to read, from 1."),
            (
                "to_line",
                "integer",
                "The last line to read, at most 200 lines after from_line.",
            ),
        ],
    ),
    (
        "memory_note",
        "Add a short note to today's daily note. Call it on your own whenever something worth keeping comes up: \
         plans, events, feelings, things the person cares about. Don't note small talk.",
        &[(
            "text",
            "string",
            "The note, one line of 1 to 1,000 characters, written so it makes sense on its own later.",
        )],
    ),
    (
        "memory_remember",
        "Save a durable fact to long-term memory. Call it when the person explicitly asks you to remember \
         something (set requested_by_user to true), or for a lasting fact or standing decision that will matter \
         for months. Not for profile facts; use profile_update for those.",
        &[
            (
                "text",
                "string",
                "The fact, one line of 1 to 500 characters.",
            ),
            (
                "topic",
                "string",
                "A short topic heading for it, such as Health or Work, one line of 1 to 60 characters.",
            ),
            (
                "requested_by_user",
                "boolean",
                "True when the person explicitly asked you to remember this.",
            ),
        ],
    ),
    (
        "profile_update",
        "Update a fact or preference about the person in their profile, replacing the old value: what to call \
         them, their language, where they live, a stated preference. Call it as soon as the person states or \
         changes such a fact.",
        &[
            (
                "key",
                "string",
                "The fact's name as the profile writes it, such as Call them or Preferred language. 1 to 40 \
                 characters, no colon.",
            ),
            (
                "value",
                "string",
                "The new value, one line of 1 to 300 characters.",
            ),
        ],
    ),
];

/// The tools of a `memory-1` context, in a fixed order (memory R1).
pub(crate) fn specs() -> Vec<ToolSpec> {
    TOOLS
        .iter()
        .map(|(name, description, properties)| {
            let props: serde_json::Map<String, Value> = properties
                .iter()
                .map(|(p, kind, about)| {
                    (
                        (*p).to_owned(),
                        json!({ "type": kind, "description": about }),
                    )
                })
                .collect();
            let required: Vec<&str> = properties.iter().map(|(p, ..)| *p).collect();
            ToolSpec {
                name,
                description,
                input_schema: json!({
                    "type": "object",
                    "properties": props,
                    "required": required,
                    "additionalProperties": false,
                }),
            }
        })
        .collect()
}

/// A tool call's input after validation.
#[derive(Debug, PartialEq)]
pub(crate) enum ToolInput {
    Search {
        query: String,
    },
    Get {
        path: MemoryPath,
        from: usize,
        to: usize,
    },
    Note {
        text: String,
    },
    Remember {
        text: String,
        topic: String,
        by_user: bool,
    },
    Profile {
        key: String,
        value: String,
    },
}

/// What a tool call gave back: the `tool_result` content, and for writes the mark and the rewritten file.
#[derive(Debug, PartialEq)]
pub(crate) struct ToolOutcome {
    pub content: String,
    pub is_error: bool,
    pub mark: Option<MemoryMark>,
    /// The profile or long-term memory after a write: `(file, content)` for the `written` entry.
    pub written: Option<(String, String)>,
}

/// Checks a tool input against its schema and the limits the schemas leave to the core (memory interfaces).
pub(crate) fn validate(name: &str, input: &Value) -> Result<ToolInput, String> {
    let (_, _, properties) = TOOLS
        .iter()
        .find(|(n, ..)| *n == name)
        .ok_or_else(|| format!("there is no tool named {name}"))?;
    let object = input.as_object().ok_or("the input must be an object")?;
    for key in object.keys() {
        if !properties.iter().any(|(p, ..)| p == key) {
            return Err(format!("{key} is not a property of {name}"));
        }
    }
    let text = |field: &str, max: usize, one_line: bool| -> Result<String, String> {
        let value = object
            .get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{field} must be a string"))?;
        let length = value.trim().chars().count();
        if length == 0 || length > max {
            return Err(format!("{field} must be 1 to {max} characters"));
        }
        if one_line && value.contains(['\n', '\r']) {
            return Err(format!("{field} must be one line"));
        }
        Ok(value.trim().to_owned())
    };
    let number = |field: &str| -> Result<usize, String> {
        object
            .get(field)
            .and_then(Value::as_u64)
            .filter(|n| *n >= 1)
            .map(|n| n as usize)
            .ok_or_else(|| format!("{field} must be a whole number from 1"))
    };
    Ok(match name {
        "memory_search" => ToolInput::Search {
            query: text("query", 500, false)?,
        },
        "memory_get" => {
            let path = MemoryPath::parse(&text("path", 40, true)?)
                .ok()
                .filter(|p| !matches!(p, MemoryPath::Soul | MemoryPath::Diary))
                .ok_or("path must be MEMORY.md, USER.md or memory/YYYY-MM-DD.md")?;
            let (from, to) = (number("from_line")?, number("to_line")?);
            if to < from || to - from > 200 {
                return Err("to_line must be from from_line to 200 lines after it".into());
            }
            ToolInput::Get { path, from, to }
        }
        "memory_note" => ToolInput::Note {
            text: text("text", 1_000, true)?,
        },
        "memory_remember" => ToolInput::Remember {
            text: text("text", 500, true)?,
            topic: text("topic", 60, true)?,
            by_user: object
                .get("requested_by_user")
                .and_then(Value::as_bool)
                .ok_or("requested_by_user must be true or false")?,
        },
        _ => {
            let key = text("key", 40, true)?;
            if key.contains(':') {
                return Err("key must not contain a colon".into());
            }
            ToolInput::Profile {
                key,
                value: text("value", 300, true)?,
            }
        }
    })
}

fn excerpt(text: &str) -> String {
    match text.char_indices().nth(120) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text.to_owned(),
    }
}

pub(crate) fn failed(content: impl Into<String>) -> ToolOutcome {
    ToolOutcome {
        is_error: true,
        ..answer(content)
    }
}

fn answer(content: impl Into<String>) -> ToolOutcome {
    ToolOutcome {
        content: content.into(),
        is_error: false,
        mark: None,
        written: None,
    }
}

/// `- 2026-10-23 (memory/2026-10-23.md, line 4): text`, one line per hit (memory interfaces).
pub(crate) fn describe(hit: &Hit) -> String {
    let place = hit.day.as_deref().unwrap_or("long-term memory");
    let lines = if hit.first == hit.last {
        format!("line {}", hit.first)
    } else {
        format!("lines {}–{}", hit.first, hit.last)
    };
    format!(
        "- {place} ({}, {lines}): {}",
        hit.path,
        hit.text.replace('\n', " / ")
    )
}

impl Core {
    /// memory FR-017: up to 8 hits, or an error the companion can explain when the index is unavailable (memory
    /// FR-024).
    fn search_tool(&self, query: &str, now: &LocalNow) -> ToolOutcome {
        match self.search_memory(query, Some(&now.date), 8, true) {
            Err(_) => failed("Memory search is unavailable right now."),
            Ok((hits, _)) if hits.is_empty() => answer("No matches."),
            Ok((hits, _)) => {
                // dreaming R1: what the companion found is recall evidence too.
                let _ = self.record_recalls(query, &hits, &now.date, "search");
                answer(hits.iter().map(describe).collect::<Vec<_>>().join("\n"))
            }
        }
    }

    /// memory FR-017: numbered lines of a memory file.
    fn read_tool(&self, path: &MemoryPath, from: usize, to: usize) -> ToolOutcome {
        let Ok(text) = std::fs::read_to_string(self.workspace_dir().join(path.rel())) else {
            return failed(format!("{} doesn't exist or can't be read.", path.rel()));
        };
        let lines: Vec<String> = text
            .lines()
            .enumerate()
            .skip(from - 1)
            .take(to + 1 - from)
            .map(|(i, line)| format!("{}: {line}", i + 1))
            .collect();
        if lines.is_empty() {
            return failed(format!(
                "{} has only {} lines.",
                path.rel(),
                text.lines().count()
            ));
        }
        answer(lines.join("\n"))
    }
}

impl Core {
    /// Runs one tool call. Never fails: a problem becomes an `is_error` result the companion can read (memory FR-012).
    /// `src` is the id of the user message that started the turn.
    pub(crate) fn run_tool(
        &self,
        name: &str,
        input: &Value,
        now: &LocalNow,
        src: &str,
    ) -> ToolOutcome {
        let input = match validate(name, input) {
            Ok(input) => input,
            Err(reason) => return failed(format!("The input was not valid: {reason}.")),
        };
        let (kind, path, fact, landed) = match &input {
            ToolInput::Search { query } => return self.search_tool(query, now),
            ToolInput::Get { path, from, to } => return self.read_tool(path, *from, *to),
            ToolInput::Note { text } => (
                MarkKind::Note,
                MemoryPath::Daily(now.date.clone()).rel(),
                text.clone(),
                self.add_note(now, text, Origin::Companion, src),
            ),
            ToolInput::Remember {
                text,
                topic,
                by_user,
            } => {
                let origin = if *by_user {
                    Origin::User
                } else {
                    Origin::Companion
                };
                (
                    MarkKind::Remembered,
                    "MEMORY.md".into(),
                    text.clone(),
                    self.remember(now, text, topic, origin, src),
                )
            }
            ToolInput::Profile { key, value } => (
                MarkKind::Profile,
                "USER.md".into(),
                format!("{key}: {value}"),
                self.update_profile(now, key, value, src),
            ),
        };
        let Ok(landed) = landed else {
            let mut outcome = failed("This could not be saved: the file could not be written.");
            outcome.mark = Some(MemoryMark {
                kind,
                path,
                line: None,
                excerpt: excerpt(&fact),
                ok: false,
            });
            return outcome;
        };
        let content = match (&input, landed.kept_in_note) {
            (ToolInput::Note { .. }, _) => {
                format!("Noted in {}, line {}.", landed.path, landed.line)
            }
            (ToolInput::Remember { topic, .. }, false) => {
                format!("Remembered in MEMORY.md under {topic}.")
            }
            (ToolInput::Remember { .. }, true) => format!(
                "Long-term memory is full, so this was kept in {}, line {}. Suggest the person tidies their \
                 long-term memory in the memory browser.",
                landed.path, landed.line
            ),
            (ToolInput::Profile { key, value }, false) => {
                format!("Updated USER.md: {key} is now {value}.")
            }
            _ => format!(
                "The profile is full, so this was kept in {}, line {}. Suggest the person tidies their profile \
                 in the memory browser.",
                landed.path, landed.line
            ),
        };
        ToolOutcome {
            content,
            is_error: false,
            mark: Some(MemoryMark {
                kind: if landed.kept_in_note {
                    MarkKind::KeptInNote
                } else {
                    kind
                },
                path: landed.path.clone(),
                line: Some(landed.line as u32),
                excerpt: excerpt(&fact),
                ok: true,
            }),
            written: landed.content.map(|c| (landed.path, c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_schema_is_strict_and_lists_every_property() {
        let specs = specs();
        assert_eq!(specs.len(), 5);
        for spec in specs {
            let schema = &spec.input_schema;
            assert_eq!(schema["additionalProperties"], false, "{}", spec.name);
            let props = schema["properties"].as_object().unwrap();
            assert_eq!(schema["required"].as_array().unwrap().len(), props.len());
            for p in props.values() {
                assert_eq!(p.as_object().unwrap().len(), 2, "only type and description");
            }
        }
    }

    #[test]
    fn invalid_inputs_are_refused_with_a_reason() {
        let long = |n: usize| "x".repeat(n);
        let ok = [
            ("memory_search", json!({ "query": long(500) })),
            (
                "memory_get",
                json!({ "path": "memory/2026-11-14.md", "from_line": 1, "to_line": 201 }),
            ),
            ("memory_note", json!({ "text": long(1_000) })),
            (
                "memory_remember",
                json!({ "text": long(500), "topic": long(60), "requested_by_user": false }),
            ),
            (
                "profile_update",
                json!({ "key": long(40), "value": long(300) }),
            ),
        ];
        for (name, input) in ok {
            assert!(validate(name, &input).is_ok(), "{name} {input}");
        }
        let refused = [
            ("memory_search", json!({ "query": long(501) })),
            ("memory_search", json!({ "query": "  " })),
            ("memory_search", json!({ "query": "a", "extra": 1 })),
            ("memory_search", json!({})),
            ("memory_search", json!("a")),
            (
                "memory_get",
                json!({ "path": "SOUL.md", "from_line": 1, "to_line": 2 }),
            ),
            (
                "memory_get",
                json!({ "path": "IDENTITY.md", "from_line": 1, "to_line": 2 }),
            ),
            (
                "memory_get",
                json!({ "path": "MEMORY.md", "from_line": 5, "to_line": 4 }),
            ),
            (
                "memory_get",
                json!({ "path": "MEMORY.md", "from_line": 1, "to_line": 202 }),
            ),
            (
                "memory_get",
                json!({ "path": "MEMORY.md", "from_line": 0, "to_line": 2 }),
            ),
            ("memory_note", json!({ "text": long(1_001) })),
            ("memory_note", json!({ "text": "two\nlines" })),
            (
                "memory_remember",
                json!({ "text": "a", "topic": long(61), "requested_by_user": true }),
            ),
            ("memory_remember", json!({ "text": "a", "topic": "T" })),
            (
                "memory_remember",
                json!({ "text": "a", "topic": "T", "requested_by_user": "yes" }),
            ),
            ("profile_update", json!({ "key": "a:b", "value": "c" })),
            ("profile_update", json!({ "key": long(41), "value": "c" })),
            ("profile_update", json!({ "key": "k", "value": "a\rb" })),
            ("unknown_tool", json!({})),
        ];
        for (name, input) in refused {
            assert!(validate(name, &input).is_err(), "{name} {input}");
        }
    }
}
