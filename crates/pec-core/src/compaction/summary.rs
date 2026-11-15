//! The summary: the older turns rendered as text, the structured reply asked for, its checks, and the Markdown
//! the companion and the divider read (compaction R7, R9, compaction FR-015, FR-017).

use super::MAX_SUMMARY_CHARS;
use crate::chat::context::{Turn, reply_text};
use crate::chat::mood::strip;
use crate::chat::transcript::{Entry, Provenance, Role};
use crate::memory::files::is_date;
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) const INSTRUCTIONS: &str = include_str!("summary.md");

/// Who is who in the rendered conversation.
pub(crate) struct Names<'a> {
    pub companion: &'a str,
    pub person: &'a str,
}

/// The summary request's one user message: the previous summary, the turns, the focus note.
pub(crate) fn input(
    previous: Option<&str>,
    turns: &[Turn],
    names: &Names,
    focus: Option<&str>,
) -> String {
    let mut out = String::new();
    if let Some(previous) = previous {
        out.push_str(&format!("Previous summary:\n{}\n\n", previous.trim_end()));
    }
    out.push_str("Conversation:\n");
    for turn in turns {
        render_turn(&mut out, turn, names);
    }
    if let Some(focus) = focus {
        out.push_str(&format!("\nFocus from {}: {focus}\n", names.person));
    }
    out
}

fn render_turn(out: &mut String, turn: &Turn, names: &Names) {
    let Entry::Message {
        provenance,
        content,
        text,
        ..
    } = &turn.user.entry
    else {
        return;
    };
    let time = content
        .first()
        .and_then(|b| b["text"].as_str())
        .and_then(|t| t.strip_prefix("<context>Local time: "))
        .and_then(|t| t.split(" (").next())
        .unwrap_or_default();
    for block in content {
        if let Some(file) = block["text"]
            .as_str()
            .and_then(|t| t.strip_prefix("<notice>The user edited "))
            .and_then(|t| t.split(". From now on").next())
        {
            out.push_str(&format!("({} edited {file})\n", names.person));
        }
    }
    if *provenance == Provenance::User {
        let text = text.as_deref().unwrap_or_default();
        out.push_str(&format!("[{time}] {}: {text}\n", names.person));
    }
    for line in &turn.rest {
        match &line.entry {
            Entry::Message {
                role: Role::Assistant,
                content,
                ..
            } => {
                // Avatar FR-012: the summary never sees a mood marker to copy.
                let text = reply_text(content);
                let text = strip(&text);
                if !text.is_empty() {
                    out.push_str(&format!("{}: {text}\n", names.companion));
                }
            }
            Entry::Message { marks, .. } => {
                for mark in marks.iter().filter(|m| m.ok) {
                    out.push_str(&format!(
                        "({} saved to memory: {})\n",
                        names.companion, mark.excerpt
                    ));
                }
            }
            _ => {}
        }
    }
}

/// The structured reply (compaction interfaces).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Summary {
    thread: String,
    commitments: Vec<String>,
    relationship: String,
    facts: Vec<String>,
    notes: Vec<String>,
}

pub(crate) fn schema() -> Value {
    let list = json!({ "type": "array", "items": { "type": "string" } });
    json!({
        "type": "object",
        "properties": {
            "thread": { "type": "string" },
            "commitments": list,
            "relationship": { "type": "string" },
            "facts": list,
            "notes": list,
        },
        "required": ["thread", "commitments", "relationship", "facts", "notes"],
        "additionalProperties": false,
    })
}

/// The rendered summary from a reply's content, or `None` if it fails a check (compaction FR-017).
pub(crate) fn parse(content: &[Value]) -> Option<String> {
    let text = content
        .iter()
        .find(|b| b["type"] == "text")?
        .get("text")?
        .as_str()?;
    let summary: Summary = serde_json::from_str(text).ok()?;
    if summary.thread.trim().is_empty() || !summary.notes.iter().all(|d| is_date(d.trim())) {
        return None;
    }
    let rendered = render(&summary);
    (rendered.chars().count() <= MAX_SUMMARY_CHARS).then_some(rendered)
}

fn render(s: &Summary) -> String {
    let list = |items: &[String]| -> String {
        items
            .iter()
            .map(|i| i.trim())
            .filter(|i| !i.is_empty())
            .map(|i| format!("- {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let notes: Vec<String> = s
        .notes
        .iter()
        .map(|d| format!("memory/{}.md", d.trim()))
        .collect();
    [
        ("Where we are", s.thread.trim().to_owned()),
        ("Open threads", list(&s.commitments)),
        ("Between us", s.relationship.trim().to_owned()),
        ("Facts", list(&s.facts)),
        ("More in the daily notes", list(&notes)),
    ]
    .into_iter()
    .filter(|(_, body)| !body.is_empty())
    .map(|(heading, body)| format!("## {heading}\n{body}"))
    .collect::<Vec<_>>()
    .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::context::turns;
    use crate::chat::transcript::Line;
    use crate::memory::{MarkKind, MemoryMark};

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
            state: None,
            stop_reason: None,
            model: None,
            usage: None,
            recalled: None,
            world: None,
            marks: Vec::new(),
        }
    }

    fn reply_json(value: Value) -> Vec<Value> {
        vec![
            json!({ "type": "thinking", "thinking": "", "signature": "s" }),
            json!({ "type": "text", "text": value.to_string() }),
        ]
    }

    fn valid() -> Value {
        json!({
            "thread": "Planning the Lisbon trip; Sam wants restaurant ideas.",
            "commitments": ["Suggest three restaurants near Alfama.", " "],
            "relationship": "Warm; Sam is a little stressed.",
            "facts": ["Flight TP1350 on 14 March at 07:10."],
            "notes": ["2026-11-13"],
        })
    }

    #[test]
    fn the_conversation_is_rendered_as_text_for_the_summary() {
        let time = json!({ "type": "text", "text": "<context>Local time: Friday 13 November 2026, 21:40 (Asia/Jakarta, UTC+07:00)</context>" });
        let mut user = message(
            Role::User,
            Provenance::User,
            vec![
                time.clone(),
                json!({ "type": "text", "text": "<memory>Possibly relevant notes…</memory>" }),
                json!({ "type": "text", "text": "<notice>The user edited SOUL.md. From now on, follow this version:</notice>\n\n…" }),
                json!({ "type": "text", "text": "Flight is TP1350" }),
            ],
        );
        if let Entry::Message { text, .. } = &mut user {
            *text = Some("Flight is TP1350".into());
        }
        let mut result = message(Role::User, Provenance::Untrusted, Vec::new());
        if let Entry::Message { marks, .. } = &mut result {
            *marks = vec![
                MemoryMark {
                    kind: MarkKind::Note,
                    path: "memory/2026-11-13.md".into(),
                    line: Some(3),
                    excerpt: "Flight TP1350".into(),
                    ok: true,
                },
                MemoryMark {
                    kind: MarkKind::Note,
                    path: "memory/2026-11-13.md".into(),
                    line: None,
                    excerpt: "not saved".into(),
                    ok: false,
                },
            ];
        }
        let lines = [
            line("1", user),
            line(
                "2",
                message(
                    Role::Assistant,
                    Provenance::Companion,
                    // Avatar FR-012: the mood marker is not part of what the summary reads.
                    vec![json!({ "type": "text", "text": "<mood>happy</mood>\nNoted!" })],
                ),
            ),
            line("3", result),
            line(
                "4",
                message(
                    Role::Assistant,
                    Provenance::Companion,
                    vec![json!({ "type": "text", "text": "Have a good trip." })],
                ),
            ),
            line(
                "5",
                message(
                    Role::User,
                    Provenance::System,
                    vec![
                        time,
                        json!({ "type": "text", "text": "<notice>Greet them</notice>" }),
                    ],
                ),
            ),
            line(
                "6",
                message(
                    Role::Assistant,
                    Provenance::Companion,
                    vec![json!({ "type": "text", "text": "Hi!" })],
                ),
            ),
        ];
        let refs: Vec<&Line> = lines.iter().collect();
        let names = Names {
            companion: "Steve",
            person: "Sam",
        };
        assert_eq!(
            input(
                Some("## Where we are\nOld."),
                &turns(&refs),
                &names,
                Some("the flight")
            ),
            "Previous summary:\n## Where we are\nOld.\n\nConversation:\n\
             (Sam edited SOUL.md)\n\
             [Friday 13 November 2026, 21:40] Sam: Flight is TP1350\n\
             Steve: Noted!\n\
             (Steve saved to memory: Flight TP1350)\n\
             Steve: Have a good trip.\n\
             Steve: Hi!\n\
             \nFocus from Sam: the flight\n"
        );
    }

    #[test]
    fn a_valid_reply_renders_the_five_parts_in_order() {
        assert_eq!(
            parse(&reply_json(valid())).unwrap(),
            "## Where we are\nPlanning the Lisbon trip; Sam wants restaurant ideas.\n\n\
             ## Open threads\n- Suggest three restaurants near Alfama.\n\n\
             ## Between us\nWarm; Sam is a little stressed.\n\n\
             ## Facts\n- Flight TP1350 on 14 March at 07:10.\n\n\
             ## More in the daily notes\n- memory/2026-11-13.md"
        );
        let mut sparse = valid();
        sparse["commitments"] = json!([]);
        sparse["notes"] = json!([]);
        sparse["relationship"] = json!("");
        assert_eq!(
            parse(&reply_json(sparse)).unwrap(),
            "## Where we are\nPlanning the Lisbon trip; Sam wants restaurant ideas.\n\n\
             ## Facts\n- Flight TP1350 on 14 March at 07:10."
        );
    }

    #[test]
    fn a_reply_that_fails_a_check_is_refused() {
        let mut empty = valid();
        empty["thread"] = json!("  ");
        let mut bad_date = valid();
        bad_date["notes"] = json!(["yesterday"]);
        let mut extra = valid();
        extra["mood"] = json!("fine");
        let mut long = valid();
        long["facts"] = json!(["x".repeat(MAX_SUMMARY_CHARS)]);
        for (case, content) in [
            ("empty thread", reply_json(empty)),
            ("bad date", reply_json(bad_date)),
            ("unknown field", reply_json(extra)),
            ("too long", reply_json(long)),
            (
                "not json",
                vec![json!({ "type": "text", "text": "Here is a summary" })],
            ),
            ("no text", Vec::new()),
        ] {
            assert_eq!(parse(&content), None, "{case}");
        }
    }
}
