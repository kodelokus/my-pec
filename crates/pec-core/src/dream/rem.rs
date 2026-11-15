//! The REM phase: themes across days, which reinforce the candidates they group and colour the diary. A theme can
//! only point at candidates, never add a fact (dreaming R6, dreaming FR-011).

use super::MAX_THEMES;
use super::run::{Candidate, Said, Theme};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) const INSTRUCTIONS: &str = include_str!("prompts/rem.md");

pub(crate) fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "themes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "summary": { "type": "string" },
                        "candidates": { "type": "array", "items": { "type": "integer" } },
                    },
                    "required": ["title", "summary", "candidates"],
                    "additionalProperties": false,
                }
            }
        },
        "required": ["themes"],
        "additionalProperties": false,
    })
}

pub(crate) fn input(candidates: &[Candidate], said: &[Said]) -> String {
    let mut out = String::from("Candidate notes:\n");
    for c in candidates {
        let day = c.days.last().map_or("", String::as_str);
        out.push_str(&format!("[{}] ({day}) {}\n", c.id, c.text));
    }
    out.push_str("\nThe person's messages:\n");
    for s in said {
        out.push_str(&format!("[{}] {}\n", s.at, s.text));
    }
    out
}

#[derive(Deserialize)]
struct Themes {
    themes: Vec<Theme>,
}

/// At most `MAX_THEMES` themes, each keeping only ids of real candidates; `None` if the reply isn't the structure.
pub(crate) fn parse(content: &[Value], candidates: &[Candidate]) -> Option<Vec<Theme>> {
    let text = content.iter().find(|b| b["type"] == "text")?["text"].as_str()?;
    let mut themes = serde_json::from_str::<Themes>(text).ok()?.themes;
    themes.truncate(MAX_THEMES);
    for theme in &mut themes {
        theme
            .candidates
            .retain(|id| candidates.iter().any(|c| c.id == *id));
    }
    Some(themes)
}

/// Each candidate in a theme gets one reinforcement.
pub(crate) fn reinforce(candidates: &mut [Candidate], themes: &[Theme]) {
    for c in candidates {
        if themes.iter().any(|t| t.candidates.contains(&c.id)) {
            c.reinforced += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: usize, text: &str) -> Candidate {
        Candidate {
            id,
            text: text.into(),
            provenance: "companion".into(),
            sources: Vec::new(),
            days: vec!["2026-11-14".into()],
            reinforced: 0,
            terms: 3,
        }
    }

    #[test]
    fn themes_point_only_at_candidates_and_reinforce_them() {
        let mut candidates = vec![candidate(1, "Ran 10 km"), candidate(2, "New job")];
        let said = [Said {
            at: "Friday 13 November 2026, 21:40".into(),
            text: "I ran again".into(),
        }];
        assert_eq!(
            input(&candidates, &said),
            "Candidate notes:\n[1] (2026-11-14) Ran 10 km\n[2] (2026-11-14) New job\n\n\
             The person's messages:\n[Friday 13 November 2026, 21:40] I ran again\n"
        );
        let reply: Vec<Value> = vec![
            json!({ "type": "text", "text": json!({ "themes": (0..7).map(|i| json!({
            "title": format!("T{i}"), "summary": "s", "candidates": [1, 99],
        })).collect::<Vec<_>>() }).to_string() }),
        ];
        let themes = parse(&reply, &candidates).unwrap();
        assert_eq!(themes.len(), MAX_THEMES);
        assert_eq!(themes[0].candidates, [1]);
        reinforce(&mut candidates, &themes);
        assert_eq!((candidates[0].reinforced, candidates[1].reinforced), (1, 0));
        assert_eq!(
            parse(
                &[json!({ "type": "text", "text": "not json" })],
                &candidates
            ),
            None
        );
    }
}
