//! Journey: what the companion is told about its world: the description saved in each new context and the
//! notice after a night that drew something (journey R9, R10, journey interfaces).

use super::Land;
use super::dna::Terrain;
use super::file::{Drawn, World};
use super::journey::{Journey, Topic, makeup};

/// journey FR-036: the description's budget, and the notice's.
const BUDGET: usize = 1_200;
const RECENT: usize = 5;
const MAKEUP: usize = 4;
const PER_QUARTER: usize = 2;

/// An item's English name: its id read backwards (`shoes.running` → "running shoes"); a home by its name alone.
pub(crate) fn english(item: &str) -> String {
    match item.split_once('.') {
        Some(("home", name)) => name.to_owned(),
        Some((kind, name)) => format!("{name} {kind}"),
        None => item.to_owned(),
    }
}

fn land_name(land: Land) -> &'static str {
    match land {
        Land::Water => "water",
        Land::Woods => "woods",
        Land::Rock => "rocks",
        _ => "meadow",
    }
}

/// North, east, south or west of the home; the home's own cell has none. y grows southward.
fn quarter(x: i32, y: i32) -> Option<usize> {
    if (x, y) == (0, 0) {
        None
    } else if y.abs() >= x.abs() {
        Some(if y < 0 { 0 } else { 2 })
    } else {
        Some(if x > 0 { 1 } else { 3 })
    }
}

fn is_topic(d: &Drawn) -> bool {
    d.why.starts_with("topic:")
}

/// The told topics, newest day first and the day's last topic first, with the day and the drawn item.
fn recent(journey: &Journey) -> Vec<(&str, &super::journey::Told)> {
    let mut days: Vec<_> = journey.days.iter().collect();
    days.sort_by(|a, b| b.day.cmp(&a.day));
    days.iter()
        .flat_map(|d| {
            d.topics
                .iter()
                .rev()
                .filter_map(Topic::told)
                .map(|t| (d.day.as_str(), t))
        })
        .collect()
}

fn told_line(day: &str, told: &super::journey::Told) -> String {
    format!(
        "{} for \"{}\" ({day}): {}",
        english(&told.item),
        told.keyword,
        told.line
    )
}

/// The world description (journey interfaces), within `BUDGET` characters.
pub(crate) fn describe(id: &str, born: &str, world: &World, journey: &Journey) -> String {
    let home = world
        .items
        .iter()
        .find(|d| d.item.starts_with("home."))
        .map_or_else(|| "home".to_owned(), |d| english(&d.item));
    let drawn: Vec<&Drawn> = world
        .items
        .iter()
        .filter(|d| !d.item.starts_with("home."))
        .collect();
    let head = "## Your world\n\n";
    if drawn.is_empty() {
        return format!("{head}Your world is just your {home} so far.");
    }
    let terrain = Terrain::new(id);
    let names = ["North", "East", "South", "West"];
    // Each quarter's commonest land and its newest topic objects.
    let quarters: Vec<(usize, &'static str, Vec<String>)> = (0..4)
        .filter_map(|q| {
            let here: Vec<&&Drawn> = drawn
                .iter()
                .filter(|d| quarter(d.x, d.y) == Some(q))
                .collect();
            if here.is_empty() {
                return None;
            }
            let mut counts = [0usize; 4];
            for d in &here {
                counts[match terrain.at(d.x, d.y) {
                    Land::Water => 0,
                    Land::Woods => 1,
                    Land::Rock => 2,
                    _ => 3,
                }] += 1;
            }
            let commonest = [Land::Water, Land::Woods, Land::Rock, Land::Meadow]
                [(0..4).max_by_key(|i| (counts[*i], *i)).unwrap_or(3)];
            let mut objects: Vec<&&Drawn> = here.into_iter().filter(|d| is_topic(d)).collect();
            objects.sort_by(|a, b| b.day.cmp(&a.day));
            let objects = objects
                .iter()
                .take(PER_QUARTER)
                .map(|d| english(&d.item))
                .collect();
            Some((q, land_name(commonest), objects))
        })
        .collect();
    let recent = recent(journey);
    let shares: Vec<String> = makeup(journey)
        .into_iter()
        .take(MAKEUP)
        .map(|(s, n)| format!("{} {}%", s.replace('-', " "), (n * 100.0).round()))
        .collect();
    let build = |recent_count: usize, with_objects: bool| {
        let mut text = format!(
            "{head}You have drawn {} things since {born}, around your {home}.\n",
            drawn.len()
        );
        let places: Vec<String> = quarters
            .iter()
            .map(|(q, land, objects)| {
                if with_objects && !objects.is_empty() {
                    format!(
                        "{}: mostly {land}, with {}.",
                        names[*q],
                        objects.join(" and ")
                    )
                } else {
                    format!("{}: mostly {land}.", names[*q])
                }
            })
            .collect();
        text.push_str(&places.join(" "));
        if recent_count > 0 {
            text.push_str("\nMost recently you drew:");
            for (day, told) in recent.iter().take(recent_count) {
                text.push_str(&format!("\n- {}", told_line(day, told)));
            }
        }
        if !shares.is_empty() {
            text.push_str(&format!(
                "\nYour world is mostly made of: {}.",
                shares.join(", ")
            ));
        }
        text
    };
    let mut count = recent.len().min(RECENT);
    loop {
        let text = build(count, true);
        if text.chars().count() <= BUDGET {
            return text;
        }
        if count == 0 {
            let text = build(0, false);
            return text.chars().take(BUDGET).collect();
        }
        count -= 1;
    }
}

/// The notice for journey days after `told` (journey interfaces): the text and the newest day
/// it covers, or none when there is nothing to tell.
pub(crate) fn notice(journey: &Journey, told: Option<&str>) -> Option<(String, String)> {
    let mut days: Vec<_> = journey
        .days
        .iter()
        .filter(|d| told.is_none_or(|t| d.day.as_str() > t))
        .collect();
    days.sort_by(|a, b| a.day.cmp(&b.day));
    let through = days.last()?.day.clone();
    let mut parts: Vec<String> = days
        .iter()
        .flat_map(|d| {
            d.topics
                .iter()
                .filter_map(Topic::told)
                .map(|t| told_line(&d.day, t))
        })
        .collect();
    let wrap = |parts: &[String]| {
        format!(
            "<notice>While you dreamt, you drew in your world: {}</notice>",
            parts.join(" ")
        )
    };
    while !parts.is_empty() && wrap(&parts).chars().count() > BUDGET {
        parts.remove(0);
    }
    (!parts.is_empty()).then(|| (wrap(&parts), through))
}

#[cfg(test)]
mod tests {
    use super::super::journey::{Day, Told};
    use super::*;

    const ID: &str = "K7Q2-M9TX-3HPA";

    fn drawn(item: &str, x: i32, y: i32, day: &str, why: &str) -> Drawn {
        Drawn {
            item: item.into(),
            x,
            y,
            size: 1.0,
            flip: false,
            shade: 0,
            day: day.into(),
            why: why.into(),
        }
    }

    fn told(item: &str, keyword: &str, line: &str, subjects: &[&str]) -> Topic {
        Topic::Told(Told {
            keyword: keyword.into(),
            line: line.into(),
            subjects: subjects.iter().map(|s| (*s).to_owned()).collect(),
            item: item.into(),
            related: false,
            odds: Vec::new(),
            rest: 0.0,
        })
    }

    fn world(items: Vec<Drawn>) -> World {
        World {
            header: None,
            items,
            skipped: 0,
        }
    }

    #[test]
    fn the_description_tells_the_count_the_directions_the_recent_objects_and_the_makeup() {
        let w = world(vec![
            drawn("home.cottage", 0, 0, "2026-03-03", "ambient"),
            drawn("flower.daisy", 0, -2, "2026-11-10", "ambient"),
            drawn("shoes.running", 0, -3, "2026-11-10", "topic:2026-11-10/0"),
            drawn("cake.birthday", 3, 0, "2026-11-10", "topic:2026-11-10/1"),
        ]);
        let j = Journey {
            days: vec![Day {
                day: "2026-11-10".into(),
                topics: vec![
                    told(
                        "shoes.running",
                        "10 km run",
                        "You ran ten kilometres.",
                        &["running"],
                    ),
                    told(
                        "cake.birthday",
                        "sister's birthday",
                        "You baked a cake.",
                        &["celebration", "eating-out"],
                    ),
                ],
            }],
            skipped: 0,
        };
        let text = describe(ID, "2026-03-03", &w, &j);
        assert!(
            text.starts_with(
                "## Your world\n\nYou have drawn 3 things since 2026-03-03, around your cottage."
            ),
            "{text}"
        );
        assert!(
            text.contains("North: mostly") && text.contains("with running shoes."),
            "{text}"
        );
        assert!(
            text.contains("East: mostly") && text.contains("with birthday cake."),
            "{text}"
        );
        assert!(!text.contains("South:") && !text.contains("West:"));
        assert!(text.contains("- birthday cake for \"sister's birthday\" (2026-11-10): You baked a cake.\n- running shoes"));
        assert!(
            text.contains(
                "Your world is mostly made of: running 50%, celebration 25%, eating out 25%."
            ),
            "{text}"
        );
        assert_eq!(
            describe(
                ID,
                "2026-03-03",
                &world(vec![w.items[0].clone()]),
                &Journey::default()
            ),
            "## Your world\n\nYour world is just your cottage so far."
        );
    }

    #[test]
    fn the_description_stays_within_its_budget_for_any_world() {
        for size in [10usize, 5_000] {
            let items: Vec<Drawn> = (0..size)
                .map(|i| {
                    drawn(
                        "shoes.running",
                        (i % 101) as i32 - 50,
                        (i / 101) as i32 - 25,
                        "2026-11-10",
                        &format!("topic:2026-{:02}-01/0", i % 12 + 1),
                    )
                })
                .collect();
            let days: Vec<Day> = (0..size.min(400))
                .map(|i| Day {
                    day: format!("2025-{:04}", i),
                    topics: vec![told(
                        "shoes.running",
                        &"k".repeat(30),
                        &"l".repeat(120),
                        &["running"],
                    )],
                })
                .collect();
            let text = describe(
                ID,
                "2025-01-01",
                &world(items),
                &Journey { days, skipped: 0 },
            );
            assert!(
                text.chars().count() <= 1_200,
                "{size}: {}",
                text.chars().count()
            );
            assert!(
                text.contains("Most recently"),
                "{size}: keeps some recent objects"
            );
        }
    }

    #[test]
    fn the_notice_tells_newer_days_oldest_first_without_forgotten_stories() {
        let j = Journey {
            days: vec![
                Day {
                    day: "2026-11-09".into(),
                    topics: vec![told("cup.coffee", "old", "Old.", &["coffee"])],
                },
                Day {
                    day: "2026-11-10".into(),
                    topics: vec![
                        told("shoes.running", "10 km run", "You ran.", &["running"]),
                        Topic::Forgotten {
                            item: "cake.birthday".into(),
                            forgotten: true,
                        },
                    ],
                },
                Day {
                    day: "2026-11-11".into(),
                    topics: vec![told(
                        "laptop.open",
                        "deadline",
                        "You worked late.",
                        &["work"],
                    )],
                },
            ],
            skipped: 0,
        };
        let (text, through) = notice(&j, Some("2026-11-09")).unwrap();
        assert_eq!(through, "2026-11-11");
        assert_eq!(
            text,
            "<notice>While you dreamt, you drew in your world: running shoes for \"10 km run\" (2026-11-10): You ran. open laptop for \"deadline\" (2026-11-11): You worked late.</notice>"
        );
        assert_eq!(notice(&j, Some("2026-11-11")), None);
        let only_forgotten = Journey {
            days: vec![Day {
                day: "2026-11-12".into(),
                topics: vec![Topic::Forgotten {
                    item: "x.y".into(),
                    forgotten: true,
                }],
            }],
            skipped: 0,
        };
        assert_eq!(notice(&only_forgotten, Some("2026-11-11")), None);
        // Within its budget, keeping the newest.
        let many = Journey {
            days: (10..30)
                .map(|d| Day {
                    day: format!("2026-11-{d}"),
                    topics: vec![told(
                        "cup.coffee",
                        &"k".repeat(30),
                        &"l".repeat(120),
                        &["coffee"],
                    )],
                })
                .collect(),
            skipped: 0,
        };
        let (text, _) = notice(&many, None).unwrap();
        assert!(
            text.chars().count() <= 1_200
                && text.contains("2026-11-29")
                && !text.contains("2026-11-10")
        );
    }
}
