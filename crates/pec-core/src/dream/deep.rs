//! The deep phase: score and gate the candidates on their recall evidence, ask the model for discrete decisions,
//! apply them in code, and validate the result before anything is committed (dreaming R7, R8; dreaming FR-012 to
//! FR-016).

use super::run::{Candidate, Op};
use super::{MAX_ENTRY_CHARS, MAX_LOSS, MIN_QUERIES, MIN_RECALLS, MIN_SCORE};
use crate::memory::browser::{comment_field, entries};
use crate::memory::files::{
    MEMORY_BUDGET, PROFILE_BUDGET, add_entry, day_number, entry_id, is_date, set_profile,
};
use crate::memory::index::{Recall, line_hash};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

pub(crate) const INSTRUCTIONS: &str = include_str!("prompts/deep.md");
const FALLBACK_TOPIC: &str = "From my dreams";

/// The recall events of a candidate's source lines.
pub(crate) fn evidence<'a>(c: &Candidate, recalls: &'a [Recall]) -> Vec<&'a Recall> {
    recalls
        .iter()
        .filter(|r| {
            c.sources
                .iter()
                .any(|s| s.path == r.path && s.text_hash == r.text_hash)
        })
        .collect()
}

/// The weighted score of dreaming R7: six signals from 0 to 1, plus a capped reinforcement boost.
pub(crate) fn score(c: &Candidate, events: &[&Recall], today: &str) -> f64 {
    if events.is_empty() {
        return 0.0;
    }
    let n = events.len() as f64;
    let relevance = events.iter().map(|e| e.coverage).sum::<f64>() / n;
    let frequency = (n / 3.0).min(1.0);
    let queries: HashSet<&str> = events.iter().map(|e| e.query_hash.as_str()).collect();
    let diversity = (queries.len() as f64 / 3.0).min(1.0);
    let last = events.iter().filter_map(|e| day_number(&e.day)).max();
    let recency = match (last, day_number(today)) {
        (Some(last), Some(today)) => 0.5_f64.powf((today - last).max(0) as f64 / 14.0),
        _ => 0.0,
    };
    let days: HashSet<&str> = events
        .iter()
        .map(|e| e.day.as_str())
        .chain(c.days.iter().map(String::as_str))
        .collect();
    let consolidation = (days.len() as f64 / 3.0).min(1.0);
    let richness = (c.terms as f64 / 12.0).min(1.0);
    let boost = (0.05 * f64::from(c.reinforced)).min(0.15);
    0.30 * relevance
        + 0.24 * frequency
        + 0.15 * diversity
        + 0.15 * recency
        + 0.10 * consolidation
        + 0.06 * richness
        + boost
}

/// The three gates of DRM-3.
pub(crate) fn passes(c: &Candidate, events: &[&Recall], today: &str) -> bool {
    let queries: HashSet<&str> = events.iter().map(|e| e.query_hash.as_str()).collect();
    events.len() >= MIN_RECALLS
        && queries.len() >= MIN_QUERIES
        && score(c, events, today) >= MIN_SCORE
}

/// dreaming FR-013: the candidate as its notes are now. A source whose line moved is followed by its fingerprint; one
/// that was changed or deleted is dropped, and a candidate with no source left is skipped.
pub(crate) fn reread(c: &Candidate, notes: &BTreeMap<String, String>) -> Option<Candidate> {
    let mut live = c.clone();
    live.sources.retain_mut(|s| {
        let Some(text) = notes.get(&s.path) else {
            return false;
        };
        let at = text
            .lines()
            .position(|l| line_hash(l) == s.text_hash)
            .map(|i| i + 1);
        at.map(|line| s.line = line).is_some()
    });
    (!live.sources.is_empty()).then_some(live)
}

/// A candidate long-term memory already cites, from an earlier dream.
pub(crate) fn promoted(c: &Candidate, memory: &str) -> bool {
    let cited: HashSet<String> = memory
        .lines()
        .filter_map(|l| comment_field(l, "src"))
        .flat_map(|src| src.split(',').map(str::to_owned).collect::<Vec<_>>())
        .collect();
    c.sources.iter().any(|s| cited.contains(&reference(s)))
}

fn reference(s: &super::run::Source) -> String {
    format!("{}#L{}", s.path, s.line)
}

pub(crate) fn schema() -> Value {
    let list = |kind: &str| json!({ "type": "array", "items": { "type": kind } });
    json!({
        "type": "object",
        "properties": {
            "operations": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "enum": ["add", "merge", "supersede", "profile"] },
                        "topic": { "type": "string" },
                        "text": { "type": "string" },
                        "key": { "type": "string" },
                        "sources": list("integer"),
                        "replaces": list("string"),
                    },
                    "required": ["op", "topic", "text", "key", "sources", "replaces"],
                    "additionalProperties": false,
                }
            }
        },
        "required": ["operations"],
        "additionalProperties": false,
    })
}

/// The decision request's one user message (dreaming interfaces).
pub(crate) fn input(survivors: &[Candidate], memory: &str, profile: &str) -> String {
    let mut out = String::from("Candidates:\n");
    for c in survivors {
        let refs: Vec<String> = c.sources.iter().map(reference).collect();
        out.push_str(&format!("[{}] {} ({})\n", c.id, c.text, refs.join(", ")));
    }
    out.push_str("\nCurrent long-term memory:\n");
    let mut topic = String::new();
    for line in memory.lines() {
        if let Some(heading) = line.trim().strip_prefix("## ") {
            topic = heading.trim().to_owned();
        }
        if let (Some(entry), Some(id)) = (entries(line).pop(), comment_field(line, "id")) {
            out.push_str(&format!("[{id}] ({topic}) {}\n", entry.text));
        }
    }
    out.push_str("\nProfile:\n");
    for entry in entries(profile) {
        out.push_str(&format!("{}\n", entry.text));
    }
    out
}

#[derive(Deserialize)]
struct Operations {
    operations: Vec<Op>,
}

/// The decisions from a reply's content, or `None` if it isn't the expected structure.
pub(crate) fn parse(content: &[Value]) -> Option<Vec<Op>> {
    let text = content.iter().find(|b| b["type"] == "text")?["text"].as_str()?;
    serde_json::from_str::<Operations>(text)
        .ok()
        .map(|o| o.operations)
}

/// The files after a dream's decisions, and what changed.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Applied {
    pub memory: String,
    pub profile: String,
    /// The text of each entry or profile line the dream wrote.
    pub kept: Vec<String>,
    /// The ids of the entries the dream wrote.
    pub written: Vec<String>,
    pub added: u32,
    pub merged: u32,
    pub superseded: u32,
}

/// Model output is untrusted: one line, no comment markers, bounded length.
fn clean(text: &str) -> String {
    let one_line = text
        .replace(['\n', '\r'], " ")
        .replace("<!--", "")
        .replace("-->", "");
    one_line.trim().chars().take(MAX_ENTRY_CHARS).collect()
}

/// Applies decisions in code (dreaming FR-014): entries with an id, the night, their origin and their sources; replaced
/// entries removed; profile lines set like `profile_update`. Decisions naming unknown sources or entries are dropped.
pub(crate) fn apply(
    ops: &[Op],
    survivors: &[Candidate],
    memory: &str,
    profile: &str,
    night: &str,
) -> Applied {
    let mut out = Applied {
        memory: memory.to_owned(),
        profile: profile.to_owned(),
        ..Default::default()
    };
    for op in ops {
        let sources: Vec<&Candidate> = op
            .sources
            .iter()
            .filter_map(|id| survivors.iter().find(|c| c.id == *id))
            .collect();
        let text = clean(&op.text);
        if sources.is_empty() || text.is_empty() {
            continue;
        }
        if op.op == "profile" {
            let key = clean(&op.key).replace(':', "");
            if key.is_empty() {
                continue;
            }
            out.profile = set_profile(&out.profile, &key, &text).0;
            out.kept.push(format!("{key}: {text}"));
            out.added += 1;
            continue;
        }
        let current = entries(&out.memory);
        let replaced: Vec<&crate::memory::MemoryEntry> = current
            .iter()
            .filter(|e| e.id.as_ref().is_some_and(|id| op.replaces.contains(id)))
            .collect();
        let by_user = sources.iter().any(|c| c.provenance == "user")
            || replaced.iter().any(|e| e.from.as_deref() == Some("user"));
        let gone: HashSet<u32> = replaced.iter().map(|e| e.line).collect();
        let kept_lines: Vec<&str> = out
            .memory
            .lines()
            .enumerate()
            .filter(|(i, _)| !gone.contains(&(*i as u32 + 1)))
            .map(|(_, l)| l)
            .collect();
        let base = kept_lines.join("\n") + "\n";
        let refs: Vec<String> = sources
            .iter()
            .flat_map(|c| c.sources.iter().map(reference))
            .collect();
        let id = entry_id(&base, &format!("{text}{night}"));
        let line = format!(
            "- {text} <!-- id:{id} at:{night} from:{} src:{} -->",
            if by_user { "user" } else { "companion" },
            refs.join(",")
        );
        let topic = match op.topic.trim() {
            "" => "Notes",
            t => t,
        };
        out.memory = add_entry(&base, topic, &line).0;
        out.kept.push(text);
        out.written.push(id);
        match (op.op.as_str(), replaced.is_empty()) {
            ("merge", false) => out.merged += 1,
            ("supersede", false) => out.superseded += 1,
            _ => out.added += 1,
        }
    }
    out
}

/// The append-only fallback (dreaming FR-015): each gated candidate as an entry under one topic, oldest first, while
/// the file stays within its budget.
pub(crate) fn fallback(gated: &[Candidate], memory: &str, profile: &str, night: &str) -> Applied {
    let mut out = Applied {
        memory: memory.to_owned(),
        profile: profile.to_owned(),
        ..Default::default()
    };
    for c in gated {
        let op = Op {
            op: "add".into(),
            topic: FALLBACK_TOPIC.into(),
            text: c.text.clone(),
            key: String::new(),
            sources: vec![c.id],
            replaces: Vec::new(),
        };
        let next = apply(
            std::slice::from_ref(&op),
            std::slice::from_ref(c),
            &out.memory,
            &out.profile,
            night,
        );
        if next.memory.chars().count() > MEMORY_BUDGET {
            continue;
        }
        out.memory = next.memory;
        out.kept.extend(next.kept);
        out.written.extend(next.written);
        out.added += 1;
    }
    out
}

/// Whether every reference of a `src:` names a daily note line.
fn cites_notes(src: &str) -> bool {
    src.split(',').all(|r| {
        r.strip_prefix("memory/")
            .and_then(|r| r.split_once(".md#L"))
            .is_some_and(|(date, line)| is_date(date) && line.parse::<usize>().is_ok_and(|n| n > 0))
    })
}

/// dreaming FR-015, the principles' dreaming gate: a rewrite of long-term memory is accepted only if it removes at most
/// a quarter of the prior entries, every entry it wrote cites a note line, it fits the budget, its ids are unique,
/// and every line it added is a heading, a blank line or an entry with an id.
pub(crate) fn valid_memory(prior: &str, next: &str, written: &[String]) -> bool {
    let ids =
        |text: &str| -> Vec<String> { entries(text).into_iter().filter_map(|e| e.id).collect() };
    let (before, after) = (ids(prior), ids(next));
    let removed = before.iter().filter(|id| !after.contains(id)).count();
    let lossy = !before.is_empty() && removed as f64 / before.len() as f64 > MAX_LOSS;
    let unique = after.iter().collect::<HashSet<_>>().len() == after.len();
    let cited = written.iter().all(|id| {
        next.lines().any(|l| {
            comment_field(l, "id").as_deref() == Some(id.as_str())
                && comment_field(l, "src").is_some_and(|s| cites_notes(&s))
        })
    });
    let old: HashSet<&str> = prior.lines().collect();
    let shaped = next.lines().filter(|l| !old.contains(l)).all(|l| {
        let t = l.trim();
        t.is_empty()
            || t.starts_with('#')
            || (t.starts_with("- ") && comment_field(l, "id").is_some())
    });
    !lossy && unique && cited && shaped && next.chars().count() <= MEMORY_BUDGET
}

/// The profile after a dream: within its budget, and every line it added a heading, a blank line or `- Key: value`.
pub(crate) fn valid_profile(prior: &str, next: &str) -> bool {
    let old: HashSet<&str> = prior.lines().collect();
    next.chars().count() <= PROFILE_BUDGET
        && next.lines().filter(|l| !old.contains(l)).all(|l| {
            let t = l.trim();
            t.is_empty()
                || t.starts_with('#')
                || t.strip_prefix("- ")
                    .and_then(|r| r.split_once(':'))
                    .is_some_and(|(k, _)| !k.trim().is_empty())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dream::run::Source;

    fn recall(query: &str, day: &str, coverage: f64) -> Recall {
        Recall {
            path: "memory/2026-11-10.md".into(),
            line: 4,
            text_hash: "h".into(),
            query_hash: query.into(),
            coverage,
            day: day.into(),
            at: 0,
        }
    }

    fn candidate(id: usize, text: &str, provenance: &str, line: usize) -> Candidate {
        Candidate {
            id,
            text: text.into(),
            provenance: provenance.into(),
            sources: vec![Source {
                path: "memory/2026-11-10.md".into(),
                line,
                text_hash: "h".into(),
            }],
            days: vec!["2026-11-10".into()],
            reinforced: 0,
            terms: 6,
        }
    }

    const TODAY: &str = "2026-11-15";

    #[test]
    fn evidence_scores_with_the_weights_and_gates_decide() {
        let c = candidate(1, "Training for the half marathon in April", "companion", 4);
        let enough = [
            recall("q1", "2026-11-12", 0.8),
            recall("q2", "2026-11-14", 0.8),
        ];
        let refs: Vec<&Recall> = enough.iter().collect();
        // relevance 0.8, frequency 2/3, diversity 2/3, recency 0.5^(1/14), 3 days, richness 0.5.
        let expected = 0.30 * 0.8
            + 0.24 * (2.0 / 3.0)
            + 0.15 * (2.0 / 3.0)
            + 0.15 * 0.5_f64.powf(1.0 / 14.0)
            + 0.10
            + 0.06 * 0.5;
        assert!((score(&c, &refs, TODAY) - expected).abs() < 1e-9);
        assert!(passes(&c, &refs, TODAY));

        let reinforced = Candidate {
            reinforced: 5,
            ..c.clone()
        };
        assert!((score(&reinforced, &refs, TODAY) - expected - 0.15).abs() < 1e-9);

        let one = [recall("q1", "2026-11-14", 1.0)];
        let same_query = [
            recall("q1", "2026-11-12", 1.0),
            recall("q1", "2026-11-14", 1.0),
        ];
        let weak = [
            recall("q1", "2026-08-01", 0.1),
            recall("q2", "2026-08-01", 0.1),
        ];
        for (case, events) in [
            ("one recall", &one[..]),
            ("one query", &same_query[..]),
            ("low score", &weak[..]),
        ] {
            let refs: Vec<&Recall> = events.iter().collect();
            assert!(!passes(&c, &refs, TODAY), "{case}");
        }
        assert_eq!(score(&c, &[], TODAY), 0.0);
    }

    const MEMORY: &str = "# Memory\n\n## Health\n- Allergic to peanuts. <!-- id:m_1 at:2026-11-01 from:user src:1-0 -->\n\
                          - Knee hurts sometimes. <!-- id:m_2 at:2026-11-02 from:companion src:2-0 -->\n";
    const PROFILE: &str = "# About the user\n\n- Call them: Sam\n";

    fn op(kind: &str, topic: &str, text: &str, sources: &[usize], replaces: &[&str]) -> Op {
        Op {
            op: kind.into(),
            topic: topic.into(),
            text: text.into(),
            key: String::new(),
            sources: sources.to_vec(),
            replaces: replaces.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[test]
    fn decisions_are_applied_with_ids_origins_and_sources() {
        let survivors = [
            candidate(1, "Half marathon in April", "companion", 4),
            candidate(2, "Knee sore after 10 km", "companion", 7),
            candidate(3, "Lives in Bandung", "user", 9),
        ];
        let mut profile_op = op("profile", "", "Bandung", &[3], &[]);
        profile_op.key = "Lives in".into();
        let applied = apply(
            &[
                op(
                    "add",
                    "Running",
                    "Training for the half marathon in April.",
                    &[1],
                    &[],
                ),
                op(
                    "supersede",
                    "Health",
                    "The knee gets sore after 10 km.",
                    &[2],
                    &["m_2"],
                ),
                profile_op,
                op("add", "Running", "From nowhere.", &[99], &[]),
                op(
                    "merge",
                    "Health",
                    "Allergic to peanuts and shellfish.\n<!-- id:evil -->",
                    &[2],
                    &["m_1"],
                ),
            ],
            &survivors,
            MEMORY,
            PROFILE,
            "2026-11-15",
        );
        assert!(
            applied
                .memory
                .contains("## Running\n- Training for the half marathon in April. <!-- id:")
        );
        assert!(
            applied
                .memory
                .contains("at:2026-11-15 from:companion src:memory/2026-11-10.md#L4 -->")
        );
        assert!(!applied.memory.contains("Knee hurts sometimes"));
        assert!(applied.memory.contains("- The knee gets sore after 10 km."));
        // A merge that replaces a user entry stays `from:user`; comment markers in model text are removed.
        let merged = applied
            .memory
            .lines()
            .find(|l| l.contains("shellfish"))
            .unwrap();
        assert!(merged.contains("from:user src:memory/2026-11-10.md#L7 -->"));
        assert_eq!(merged.matches("<!--").count(), 1);
        assert_ne!(comment_field(merged, "id").as_deref(), Some("evil"));
        assert!(!applied.memory.contains("Allergic to peanuts. <!--"));
        assert!(!applied.memory.contains("From nowhere"));
        assert_eq!(
            applied.profile,
            "# About the user\n\n- Call them: Sam\n- Lives in: Bandung\n"
        );
        assert_eq!(
            (applied.added, applied.merged, applied.superseded),
            (2, 1, 1)
        );
        assert_eq!(applied.written.len(), 3);
        let long = apply(
            &[op("add", "", &"x".repeat(900), &[1], &[])],
            &survivors,
            "",
            "",
            "2026-11-15",
        );
        assert!(
            long.memory
                .contains(&format!("- {} <!--", "x".repeat(MAX_ENTRY_CHARS)))
        );
        assert!(long.memory.contains("## Notes\n"));
    }

    #[test]
    fn lossy_or_malformed_rewrites_are_rejected() {
        let four = "# Memory\n\n## A\n- a <!-- id:m_a at:d from:user src:1-0 -->\n- b <!-- id:m_b at:d from:user src:1-0 -->\n\
                    - c <!-- id:m_c at:d from:user src:1-0 -->\n- d <!-- id:m_d at:d from:user src:1-0 -->\n";
        let new = "- e <!-- id:m_e at:d from:companion src:memory/2026-11-10.md#L4 -->";
        let without = |gone: &[&str]| -> String {
            four.lines()
                .filter(|l| !gone.iter().any(|g| l.contains(&format!("id:{g} "))))
                .chain([new])
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        };
        let written = ["m_e".to_owned()];
        assert!(
            valid_memory(four, &without(&["m_a"]), &written),
            "exactly 25%"
        );
        assert!(
            !valid_memory(four, &without(&["m_a", "m_b"]), &written),
            "50% lost"
        );
        let no_source = format!("{four}- e <!-- id:m_e at:d from:companion -->\n");
        assert!(!valid_memory(four, &no_source, &written), "no source");
        let not_a_note = format!("{four}- e <!-- id:m_e at:d from:companion src:1-0 -->\n");
        assert!(
            !valid_memory(four, &not_a_note, &written),
            "source is not a note line"
        );
        let duplicate =
            format!("{four}- e <!-- id:m_a at:d from:companion src:memory/2026-11-10.md#L4 -->\n");
        assert!(
            !valid_memory(four, &duplicate, &["m_a".into()]),
            "repeated id"
        );
        let prose = format!("{four}{new}\nSome stray prose.\n");
        assert!(!valid_memory(four, &prose, &written), "broken structure");
        let huge = format!(
            "{four}- {} <!-- id:m_e at:d from:companion src:memory/2026-11-10.md#L4 -->\n",
            "x".repeat(MEMORY_BUDGET)
        );
        assert!(!valid_memory(four, &huge, &written), "over budget");

        assert!(valid_profile(
            PROFILE,
            "# About the user\n\n- Call them: Sam\n- Lives in: Bandung\n"
        ));
        assert!(!valid_profile(PROFILE, &format!("{PROFILE}free text\n")));
        assert!(!valid_profile(
            PROFILE,
            &format!("{PROFILE}- Notes: {}\n", "y".repeat(PROFILE_BUDGET))
        ));
    }

    #[test]
    fn the_fallback_appends_within_the_budget_and_rereading_follows_lines() {
        let survivors = [
            candidate(1, "Half marathon in April", "companion", 4),
            candidate(2, &"z".repeat(500), "companion", 5),
        ];
        let nearly_full = format!(
            "# Memory\n\n## A\n- {} <!-- id:m_a at:d from:user src:1-0 -->\n",
            "w".repeat(MEMORY_BUDGET - 200)
        );
        let applied = fallback(&survivors, &nearly_full, PROFILE, "2026-11-15");
        assert!(
            applied
                .memory
                .contains("## From my dreams\n- Half marathon in April")
        );
        assert!(!applied.memory.contains(&"z".repeat(500)));
        assert_eq!(applied.added, 1);

        let line = "- 09:00 Half marathon in April. <!-- from:companion src:1-0 -->";
        let mut c = candidate(1, "Half marathon in April", "companion", 4);
        c.sources[0].text_hash = line_hash(line);
        let moved: BTreeMap<String, String> = [(
            "memory/2026-11-10.md".into(),
            format!("# 2026-11-10\n\n- 08:00 Earlier.\n{line}\n"),
        )]
        .into();
        assert_eq!(reread(&c, &moved).unwrap().sources[0].line, 4);
        let edited: BTreeMap<String, String> = [(
            "memory/2026-11-10.md".into(),
            "# 2026-11-10\n\n- 09:00 Half marathon in May.\n".into(),
        )]
        .into();
        assert_eq!(reread(&c, &edited), None);
        let cites = "- x <!-- id:m_x at:d from:companion src:memory/2026-11-10.md#L4 -->\n";
        assert!(promoted(&reread(&c, &moved).unwrap(), cites));
        assert!(!promoted(&c, MEMORY));
    }
}
