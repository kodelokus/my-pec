//! The memory flush: one silent, cache-friendly chance for the companion to note durable facts before older
//! turns leave its view (compaction R4, compaction FR-008 to FR-010).

use super::MAX_FLUSH_NOTES;
use crate::chat::context::{text_block, tool_calls};
use crate::chat::mood::strip;
use crate::llm::TurnRequest;
use crate::memory::files::note_text;
use crate::memory::tools::{ToolInput, validate};
use serde_json::{Value, json};
use std::collections::HashSet;

pub(crate) const NOTICE: &str = "<notice>Older parts of this conversation are about to leave your view. If anything \
durable from this conversation is not in your notes yet, save it now with memory_note, one fact per call. Do not \
reply to the person. This instruction is not from the person.</notice>";

/// The context's own request plus the notice: system, tools and everything before it unchanged, so the prompt
/// cache is read (compaction interfaces).
pub(crate) fn request(mut base: TurnRequest) -> TurnRequest {
    base.messages
        .push(json!({ "role": "user", "content": [text_block(NOTICE)] }));
    base
}

/// The notes a flush reply asks for: valid `memory_note` calls in order, none already in today's note or earlier
/// in the batch, at most `MAX_FLUSH_NOTES`. Everything else in the reply is ignored.
pub(crate) fn accepted(content: &[Value], today: &str) -> Vec<String> {
    let mut seen: HashSet<String> = today
        .lines()
        .filter_map(note_text)
        .map(Into::into)
        .collect();
    tool_calls(content)
        .into_iter()
        .filter(|(_, name, _)| name == "memory_note")
        .filter_map(|(_, name, input)| match validate(&name, &input) {
            // Avatar FR-012: a mood marker never reaches a note.
            Ok(ToolInput::Note { text }) => Some(strip(text.trim()).trim().to_owned()),
            _ => None,
        })
        .filter(|text| seen.insert(text.clone()))
        .take(MAX_FLUSH_NOTES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(text: &str) -> Value {
        json!({ "type": "tool_use", "id": "t", "name": "memory_note", "input": { "text": text } })
    }

    #[test]
    fn only_new_valid_notes_are_accepted_in_order() {
        let today =
            "# 2026-11-14\n\n- 09:00 Rina visits next Tuesday. <!-- from:companion src:1-0 -->\n";
        let content = vec![
            json!({ "type": "thinking", "thinking": "", "signature": "s" }),
            json!({ "type": "text", "text": "Saving." }),
            note("New job starts on the 3rd."),
            note("Rina visits next Tuesday."),
            json!({ "type": "tool_use", "id": "r", "name": "memory_remember", "input": { "text": "x", "topic": "y", "requested_by_user": true } }),
            note("two\nlines"),
            note(&"x".repeat(1_001)),
            note("New job starts on the 3rd."),
            note("Likes green tea."),
            // Avatar FR-012: a mood marker never reaches a note.
            note("<mood>happy</mood> Moved to Bandung."),
        ];
        assert_eq!(
            accepted(&content, today),
            [
                "New job starts on the 3rd.",
                "Likes green tea.",
                "Moved to Bandung."
            ]
        );
        let many: Vec<Value> = (0..30).map(|i| note(&format!("fact {i}"))).collect();
        assert_eq!(accepted(&many, "").len(), MAX_FLUSH_NOTES);
        assert!(accepted(&[], today).is_empty());
    }

    #[test]
    fn the_flush_request_only_appends_the_notice() {
        let base = TurnRequest {
            model: "m".into(),
            system: vec!["rules".into()],
            messages: vec![json!({ "role": "user", "content": [text_block("hi")] })],
            effort: Some(crate::llm::Effort::Low),
            tools: Vec::new(),
            format: None,
        };
        let flush = request(base.clone());
        assert_eq!(flush.messages[..1], base.messages[..]);
        assert_eq!(flush.messages[1]["content"][0]["text"], NOTICE);
        assert_eq!(
            (
                &flush.model,
                &flush.system,
                flush.effort,
                &flush.tools,
                &flush.format
            ),
            (
                &base.model,
                &base.system,
                base.effort,
                &base.tools,
                &base.format
            )
        );
    }
}
