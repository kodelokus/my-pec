//! What the user sees of the conversation: one message per turn, dividers and notes (chat R4).

use super::context::{committed, replaced, reply_text};
use super::mood::strip;
use super::transcript::{Entry, Line, Provenance, ReplyState, Role};
use crate::memory::MemoryMark;
use serde::Serialize;
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum MessageKind {
    User,
    Companion,
    /// A fresh context began here.
    Divider,
    /// A workspace file was updated.
    Note,
    /// Older turns were summarized here; the text is the summary (compaction FR-025).
    Compacted,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub id: String,
    pub kind: MessageKind,
    pub text: String,
    pub state: Option<ReplyState>,
    /// Unix milliseconds.
    #[ts(type = "number")]
    pub at: u64,
    /// What the companion wrote during this turn (memory FR-008).
    pub memory: Vec<MemoryMark>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChatPage {
    pub messages: Vec<ChatMessage>,
    pub has_more: bool,
}

/// What the user sees: each turn as one companion message joining its replies' text and its memory marks, with
/// no hidden requests, tool results or replaced replies, and no divider before the first context (chat R4).
pub(super) fn visible(lines: &[Line]) -> Vec<ChatMessage> {
    let gone = replaced(lines);
    let first = lines
        .iter()
        .position(|l| matches!(l.entry, Entry::Context { .. }));
    // Each compaction shows where the summarized part ends: just before its first kept message (compaction R15).
    let done = committed(lines);
    let dividers: HashMap<&str, &Line> = lines
        .iter()
        .filter_map(|l| match &l.entry {
            Entry::Compaction { first_kept, .. } if done.contains(l.id.as_str()) => {
                Some((first_kept.as_str(), l))
            }
            _ => None,
        })
        .collect();
    let mut out: Vec<ChatMessage> = Vec::new();
    // The companion message of the turn in progress.
    let mut open: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if let Some(record) = dividers.get(line.id.as_str())
            && let Entry::Compaction { summary, .. } = &record.entry
        {
            open = None;
            out.push(ChatMessage {
                id: record.id.clone(),
                kind: MessageKind::Compacted,
                text: summary.clone(),
                state: None,
                at: record.at,
                memory: Vec::new(),
            });
        }
        if Some(i) == first || gone.contains(line.id.as_str()) {
            continue;
        }
        let message = |kind, text: String, state| ChatMessage {
            id: line.id.clone(),
            kind,
            text,
            state,
            at: line.at,
            memory: Vec::new(),
        };
        match &line.entry {
            Entry::Message {
                role: Role::Assistant,
                content,
                state,
                ..
            } => {
                let text = strip(&reply_text(content)).to_owned();
                match open {
                    Some(o) => {
                        let turn = &mut out[o];
                        if !text.is_empty() {
                            if !turn.text.is_empty() {
                                turn.text.push_str("\n\n");
                            }
                            turn.text.push_str(&text);
                        }
                        turn.state = *state;
                    }
                    None => {
                        open = Some(out.len());
                        out.push(message(MessageKind::Companion, text, *state));
                    }
                }
            }
            Entry::Message {
                provenance: Provenance::Untrusted,
                marks,
                ..
            } => {
                if let Some(o) = open {
                    out[o].memory.extend(marks.iter().cloned());
                }
            }
            Entry::Message {
                provenance, text, ..
            } => {
                open = None;
                if *provenance == Provenance::User {
                    let text = text.clone().unwrap_or_default();
                    out.push(message(MessageKind::User, text, None));
                }
            }
            Entry::Notice { file, .. } => out.push(message(MessageKind::Note, file.clone(), None)),
            Entry::Context { compaction, .. } => {
                open = None;
                if compaction.is_none() {
                    out.push(message(MessageKind::Divider, String::new(), None));
                }
            }
            Entry::Replaced { .. }
            | Entry::Written { .. }
            | Entry::Forgotten { .. }
            | Entry::Compaction { .. } => {}
        }
    }
    out
}

/// The companion message of the turn that just ran, if it produced one.
pub(super) fn turn_view(lines: &[Line]) -> Option<ChatMessage> {
    visible(lines)
        .into_iter()
        .rev()
        .find(|m| m.kind != MessageKind::Note)
        .filter(|m| m.kind == MessageKind::Companion)
}
