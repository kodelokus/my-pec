//! Where to cut: the most recent whole turns that fit the kept size stay word for word, measured with the
//! provider's own counts (compaction R6, compaction FR-014, FR-018).

use crate::chat::context::is_turn_start;
use crate::chat::reply::context_tokens;
use crate::chat::transcript::{Entry, Line, Role};
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub(crate) struct Cut {
    /// Index of the first kept message.
    pub first_kept: usize,
    pub kept_tokens: u64,
}

/// `messages` are what the context holds, oldest first; those from `measured_from` on came after its `context`
/// entry, so the replies among them report sizes of this context. `size` is the context's current size.
/// `None` when nothing lies before the cut, or the context holds nothing new since it began.
pub(crate) fn cut(
    messages: &[&Line],
    measured_from: usize,
    size: u64,
    gone: &HashSet<&str>,
    keep: u64,
) -> Option<Cut> {
    if measured_from >= messages.len() {
        return None;
    }
    let starts: Vec<usize> = (0..messages.len())
        .filter(|&i| is_turn_start(messages[i]))
        .collect();
    let latest = *starts.last()?;
    // The size of the context just before message `i`: the last reply of this context before it.
    let before = |i: usize| -> Option<u64> {
        messages[measured_from.min(i)..i]
            .iter()
            .rev()
            .find_map(|l| match &l.entry {
                Entry::Message {
                    role: Role::Assistant,
                    usage: Some(usage),
                    ..
                } if !gone.contains(l.id.as_str()) => Some(context_tokens(usage)),
                _ => None,
            })
    };
    let first_kept = starts
        .iter()
        .copied()
        .find(|&i| before(i).is_some_and(|b| size.saturating_sub(b) <= keep))
        .unwrap_or(latest);
    if !messages[..first_kept]
        .iter()
        .any(|l| !gone.contains(l.id.as_str()))
    {
        return None;
    }
    Some(Cut {
        first_kept,
        kept_tokens: before(first_kept).map_or(size, |b| size.saturating_sub(b)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::transcript::Provenance;
    use serde_json::json;

    fn message(id: &str, role: Role, provenance: Provenance, size: Option<u64>) -> Line {
        Line {
            id: id.into(),
            at: 0,
            entry: Entry::Message {
                role,
                provenance,
                content: Vec::new(),
                text: None,
                zone: None,
                state: None,
                stop_reason: None,
                model: None,
                usage: size.map(|s| json!({ "input_tokens": s })),
                recalled: None,
                world: None,
                marks: Vec::new(),
            },
        }
    }

    fn user(id: &str) -> Line {
        message(id, Role::User, Provenance::User, None)
    }

    fn reply(id: &str, size: u64) -> Line {
        message(id, Role::Assistant, Provenance::Companion, Some(size))
    }

    fn tool(id: &str) -> Line {
        message(id, Role::User, Provenance::Untrusted, None)
    }

    /// Four turns of 10,000 tokens each after a 2,000-token start; the last reply reports 42,000.
    fn four_turns() -> Vec<Line> {
        vec![
            user("u1"),
            reply("r1", 12_000),
            user("u2"),
            reply("r2", 22_000),
            user("u3"),
            reply("r3", 32_000),
            user("u4"),
            reply("r4", 42_000),
        ]
    }

    fn run(lines: &[Line], from: usize, size: u64, gone: &[&str], keep: u64) -> Option<Cut> {
        let refs: Vec<&Line> = lines.iter().collect();
        cut(&refs, from, size, &gone.iter().copied().collect(), keep)
    }

    #[test]
    fn the_cut_keeps_the_most_recent_turns_that_fit() {
        let lines = four_turns();
        for (keep, first_kept, kept_tokens) in [
            // u4's tail is 42,000 − 32,000; u3's is 20,000, too big for 10,000.
            (10_000, 6, 10_000),
            (20_000, 4, 20_000),
            (25_000, 4, 20_000),
            // Smaller than the latest exchange: the latest user message is kept anyway.
            (8_000, 6, 10_000),
        ] {
            assert_eq!(
                run(&lines, 0, 42_000, &[], keep),
                Some(Cut {
                    first_kept,
                    kept_tokens
                }),
                "{keep}"
            );
        }
    }

    #[test]
    fn a_tool_round_is_never_split() {
        let lines = [
            user("u1"),
            reply("r1", 12_000),
            user("u2"),
            reply("call", 20_000),
            tool("result"),
            reply("r2", 24_000),
        ];
        // The tail from the tool result (4,000) would fit, but a tool result is never a cut.
        assert_eq!(run(&lines, 0, 24_000, &[], 5_000).unwrap().first_kept, 2);
    }

    #[test]
    fn replaced_replies_are_neither_measured_nor_counted_as_older() {
        let mut lines = four_turns();
        lines.insert(6, reply("old", 41_000));
        // Replaced: u4's size before is still r3's 32,000.
        let cut = run(&lines, 0, 42_000, &["old"], 10_000).unwrap();
        assert_eq!((cut.first_kept, cut.kept_tokens), (7, 10_000));

        // Only replaced messages before the latest user message: nothing to compact.
        let lines = vec![reply("gone", 5_000), user("u1"), reply("r1", 9_000)];
        assert_eq!(run(&lines, 0, 9_000, &["gone"], 1_000), None);
    }

    #[test]
    fn only_replies_of_this_context_measure_a_position() {
        let lines = four_turns();
        // Messages before index 4 are kept turns from the last compaction: their sizes don't count.
        let cut = run(&lines, 4, 42_000, &[], 30_000).unwrap();
        assert_eq!((cut.first_kept, cut.kept_tokens), (6, 10_000));
    }

    #[test]
    fn nothing_before_the_cut_is_nothing_to_compact() {
        assert_eq!(
            run(&[user("u1"), reply("r1", 9_000)], 0, 9_000, &[], 1_000),
            None
        );
        assert_eq!(run(&[], 0, 0, &[], 1_000), None);
        // Right after a compaction: only kept turns, nothing new yet.
        assert_eq!(run(&four_turns(), 8, 0, &[], 1_000), None);
        // The latest user message with no reply yet, after one finished turn, can be cut before.
        let lines = vec![user("u1"), reply("r1", 9_000), user("u2")];
        assert_eq!(
            run(&lines, 0, 9_000, &[], 1_000),
            Some(Cut {
                first_kept: 2,
                kept_tokens: 0
            })
        );
    }
}
