//! `world/journey.jsonl`: one line per day the companion considered, with each topic's keyword, line, subjects and
//! the odds its object was picked with. Appended to, and rewritten only to forget a story (journey R7,
//! journey data 1).

use serde::{Deserialize, Serialize};

pub(crate) const JOURNEY_FILE: &str = "world/journey.jsonl";

/// One topic of a day, as validated from the model's answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Named {
    pub keyword: String,
    pub line: String,
    pub subjects: Vec<String>,
}

/// A day the model was asked about, with the topics that passed validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct NamedDay {
    pub day: String,
    pub topics: Vec<Named>,
}

/// A day the companion considered; no topics means there was nothing to draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Day {
    pub day: String,
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Topic {
    Told(Told),
    /// journey FR-041: the story is gone; the object stays.
    Forgotten {
        item: String,
        forgotten: bool,
    },
}

/// A topic with its story and the pick that drew its object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Told {
    pub keyword: String,
    pub line: String,
    pub subjects: Vec<String>,
    pub item: String,
    pub related: bool,
    /// The five likeliest candidates and their chances, likeliest first.
    pub odds: Vec<(String, f64)>,
    /// The combined chance of the other candidates.
    pub rest: f64,
}

impl Topic {
    pub fn item(&self) -> &str {
        match self {
            Topic::Told(told) => &told.item,
            Topic::Forgotten { item, .. } => item,
        }
    }

    pub fn told(&self) -> Option<&Told> {
        match self {
            Topic::Told(told) => Some(told),
            Topic::Forgotten { .. } => None,
        }
    }
}

/// The journey as the file holds it.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Journey {
    /// In file order; the first line for a day counts, later ones are ignored.
    pub days: Vec<Day>,
    /// Lines that couldn't be read; they stay in the file (journey FR-021).
    pub skipped: usize,
}

impl Journey {
    pub fn parse(text: &str) -> Self {
        let mut journey = Self::default();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            match serde_json::from_str::<Day>(line) {
                Ok(day) if journey.day(&day.day).is_none() => journey.days.push(day),
                Ok(_) => {}
                Err(_) => journey.skipped += 1,
            }
        }
        journey
    }

    pub fn day(&self, day: &str) -> Option<&Day> {
        self.days.iter().find(|d| d.day == day)
    }

    /// The newest day considered.
    pub fn newest(&self) -> Option<&str> {
        self.days.iter().map(|d| d.day.as_str()).max()
    }
}

pub(crate) fn line(day: &Day) -> String {
    serde_json::to_string(day).expect("a journey line serializes")
}

/// journey FR-031: each topic with its story counts once, split evenly over its subjects; largest share first.
pub(crate) fn makeup(journey: &Journey) -> Vec<(String, f64)> {
    let mut counts: Vec<(String, f64)> = Vec::new();
    let told: Vec<&Told> = journey
        .days
        .iter()
        .flat_map(|d| &d.topics)
        .filter_map(Topic::told)
        .collect();
    for topic in &told {
        for subject in &topic.subjects {
            let part = 1.0 / topic.subjects.len() as f64;
            match counts.iter_mut().find(|(s, _)| s == subject) {
                Some((_, n)) => *n += part,
                None => counts.push((subject.clone(), part)),
            }
        }
    }
    for (_, n) in &mut counts {
        *n /= told.len() as f64;
    }
    counts.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    counts
}

/// journey FR-041: the text with topic `n` of `day` forgotten, its place and object kept; every other line as it was.
/// `None` when there is no such topic with a story.
pub(crate) fn forget(text: &str, day: &str, n: usize) -> Option<String> {
    let mut done = false;
    let mut out: Vec<String> = Vec::new();
    for raw in text.lines() {
        match serde_json::from_str::<Day>(raw) {
            Ok(mut parsed) if !done && parsed.day == day => {
                done = true;
                let item = parsed.topics.get(n)?.told()?.item.clone();
                parsed.topics[n] = Topic::Forgotten {
                    item,
                    forgotten: true,
                };
                out.push(line(&parsed));
            }
            _ => out.push(raw.to_owned()),
        }
    }
    let mut next = out.join("\n");
    if text.ends_with('\n') {
        next.push('\n');
    }
    done.then_some(next)
}

/// A chance as the record keeps it: three decimals.
pub(crate) fn round(chance: f64) -> f64 {
    (chance * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_day() -> Day {
        Day {
            day: "2026-11-10".into(),
            topics: vec![Topic::Told(Told {
                keyword: "10 km run".into(),
                line: "You ran ten kilometres before work.".into(),
                subjects: vec!["running".into()],
                item: "shoes.running".into(),
                related: false,
                odds: vec![
                    ("shoes.running".into(), 0.312),
                    ("bottle.water".into(), 0.204),
                ],
                rest: 0.484,
            })],
        }
    }

    #[test]
    fn lines_round_trip_in_the_documented_shape() {
        assert_eq!(
            line(&run_day()),
            r#"{"day":"2026-11-10","topics":[{"keyword":"10 km run","line":"You ran ten kilometres before work.","subjects":["running"],"item":"shoes.running","related":false,"odds":[["shoes.running",0.312],["bottle.water",0.204]],"rest":0.484}]}"#
        );
        let forgotten =
            r#"{"day":"2026-11-12","topics":[{"item":"cake.birthday","forgotten":true}]}"#;
        let text = format!("{}\n{forgotten}\n", line(&run_day()));
        let journey = Journey::parse(&text);
        assert_eq!(journey.days[0], run_day());
        assert_eq!(journey.days[1].topics[0].item(), "cake.birthday");
        assert_eq!(journey.days[1].topics[0].told(), None);
        assert_eq!(line(&journey.days[1]), forgotten);
        assert_eq!(journey.newest(), Some("2026-11-12"));
    }

    #[test]
    fn unreadable_lines_are_skipped_and_a_days_first_line_counts() {
        let empty = r#"{"day":"2026-11-10","topics":[]}"#;
        let text = format!(
            "not json\n{}\n{empty}\n{{\"topics\":[]}}\n",
            line(&run_day())
        );
        let journey = Journey::parse(&text);
        assert_eq!(journey.days, [run_day()]);
        assert_eq!(journey.skipped, 2);
    }

    fn told(keyword: &str, subjects: &[&str]) -> Topic {
        let Topic::Told(mut t) = run_day().topics.remove(0) else {
            unreachable!()
        };
        t.keyword = keyword.into();
        t.subjects = subjects.iter().map(|s| (*s).to_owned()).collect();
        Topic::Told(t)
    }

    #[test]
    fn the_makeup_splits_each_story_over_its_subjects() {
        let journey = Journey {
            days: vec![
                Day {
                    day: "2026-11-10".into(),
                    topics: vec![
                        told("run", &["running"]),
                        told("cake", &["cooking", "celebration"]),
                    ],
                },
                Day {
                    day: "2026-11-11".into(),
                    topics: vec![
                        told("run", &["running"]),
                        Topic::Forgotten {
                            item: "x.y".into(),
                            forgotten: true,
                        },
                    ],
                },
            ],
            skipped: 0,
        };
        let shares = makeup(&journey);
        let pairs: Vec<(&str, f64)> = shares
            .iter()
            .map(|(s, n)| (s.as_str(), (n * 1000.0).round() / 1000.0))
            .collect();
        assert_eq!(
            pairs,
            [
                ("running", 0.667),
                ("celebration", 0.167),
                ("cooking", 0.167)
            ]
        );
        assert!(makeup(&Journey::default()).is_empty());
    }

    #[test]
    fn forgetting_rewrites_one_topic_and_keeps_everything_else() {
        let mut two = run_day();
        two.topics.push(told("cake", &["cooking"]));
        let other = r#"{"day":"2026-11-09","topics":[]}"#;
        let text = format!("{other}\nbroken line\n{}\n", line(&two));
        let next = forget(&text, "2026-11-10", 1).unwrap();
        let lines: Vec<&str> = next.lines().collect();
        assert_eq!(lines[..2], [other, "broken line"]);
        let day = Journey::parse(&next).days[1].clone();
        assert_eq!(
            day.topics[0],
            run_day().topics[0],
            "the other topic keeps its place"
        );
        assert_eq!(
            day.topics[1],
            Topic::Forgotten {
                item: "shoes.running".into(),
                forgotten: true
            }
        );
        assert!(next.ends_with('\n'));
        assert_eq!(forget(&next, "2026-11-10", 1), None, "already forgotten");
        assert_eq!(forget(&text, "2026-11-10", 5), None);
        assert_eq!(forget(&text, "2026-11-30", 0), None);
    }

    /// journey SC-015, run by hand in release mode (journey checks).
    #[test]
    #[ignore = "release-readiness measurement"]
    fn ten_years_of_full_days_stay_under_five_megabytes() {
        let mut topic = run_day().topics.remove(0);
        if let Topic::Told(told) = &mut topic {
            told.keyword = "x".repeat(30);
            told.line = "y".repeat(120);
            told.subjects = vec!["eating-out".into(), "celebration".into()];
            told.odds = (0..5)
                .map(|_| ("calculator.pocket".into(), 0.123))
                .collect();
        }
        let day = Day {
            day: "2026-11-10".into(),
            topics: vec![topic.clone(), topic.clone(), topic],
        };
        let bytes = (line(&day).len() + 1) * 3_653;
        assert!(bytes < 5_000_000, "{bytes} bytes after ten years");
    }
}
