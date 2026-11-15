//! Journey: the pick of an everyday object for a topic, by seeded weighted chance, and where it is placed
//! (journey R5, R6).

use super::dna::{Rng, Terrain, preference, seed};
use super::file::Drawn;
use super::journey::round;
use super::{Entry, Land, subjects};
use crate::memory::files::previous_day;
use std::collections::{BTreeSet, HashMap};

/// journey FR-010: an item of a related subject keeps this share of its weight.
const RELATED: f64 = 0.25;
/// journey FR-011: an item drawn within this many days, the topic's day included, keeps half its weight.
const RECENT_DAYS: usize = 7;
/// The odds the record keeps; the rest is one share.
const ODDS: usize = 5;
/// A day's objects land within this many cells of what that day drew (world's reach).
const REACH: i32 = 2;
/// How far the search for suitable land goes before any free cell will do.
const RINGS: i32 = 64;

/// For each item drawn so far: how many times, and whether within the seven days ending on `day`.
pub(crate) fn tally<'a>(drawn: &'a [Drawn], day: &str) -> HashMap<&'a str, (usize, bool)> {
    let first_recent = (1..RECENT_DAYS).fold(day.to_owned(), |d, _| previous_day(&d));
    let mut out: HashMap<&str, (usize, bool)> = HashMap::new();
    for d in drawn {
        let entry = out.entry(d.item.as_str()).or_default();
        entry.0 += 1;
        entry.1 |= d.day.as_str() >= first_recent.as_str();
    }
    out
}

/// How likely an item is, before the related share: the companion's taste, divided by one plus the times it was
/// drawn, and halved when it was drawn in the last seven days (journey FR-011). Scenery uses it too.
pub(crate) fn weight(id: &str, item: &str, tally: &HashMap<&str, (usize, bool)>) -> f64 {
    let (times, recent) = tally.get(item).copied().unwrap_or_default();
    let w = preference(id, item) / (1 + times) as f64;
    if recent { w * 0.5 } else { w }
}

/// One weighted choice and what the record keeps of it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pick {
    pub item: String,
    pub related: bool,
    pub odds: Vec<(String, f64)>,
    pub rest: f64,
}

/// The items a topic's subjects allow: theirs, then, marked related, those of their related subjects only
/// (journey FR-010). An item of an unrelated subject is never a candidate.
pub(crate) fn candidates<'a>(dictionary: &'a [Entry], topic: &[String]) -> Vec<(&'a Entry, bool)> {
    let related: Vec<&str> = subjects()
        .iter()
        .filter(|s| topic.contains(&s.id))
        .flat_map(|s| s.related.iter().map(String::as_str))
        .collect();
    let mut out: Vec<(&Entry, bool)> = Vec::new();
    for entry in dictionary {
        if entry.subjects.iter().any(|s| topic.contains(s)) {
            out.push((entry, false));
        } else if entry.subjects.iter().any(|s| related.contains(&s.as_str())) {
            out.push((entry, true));
        }
    }
    out
}

/// The `n`th topic of `day`: the seed, the day, the topic and the world so far decide it (journey FR-012).
pub(crate) fn pick(
    id: &str,
    day: &str,
    n: usize,
    topic: &[String],
    dictionary: &[Entry],
    drawn: &[Drawn],
) -> Option<(Pick, Rng)> {
    let counts = tally(drawn, day);
    let weighted: Vec<(&Entry, bool, f64)> = candidates(dictionary, topic)
        .into_iter()
        .map(|(e, related)| {
            let w = weight(id, &e.id, &counts) * if related { RELATED } else { 1.0 };
            (e, related, w)
        })
        .collect();
    let total: f64 = weighted.iter().map(|(_, _, w)| w).sum();
    if weighted.is_empty() || total <= 0.0 {
        return None;
    }
    let mut rng = Rng::new(seed(id, &format!("topic/{day}/{n}")));
    let mut target = rng.unit() * total;
    let mut chosen = weighted[weighted.len() - 1];
    for candidate in &weighted {
        if target < candidate.2 {
            chosen = *candidate;
            break;
        }
        target -= candidate.2;
    }
    let mut ranked: Vec<(&str, f64)> = weighted
        .iter()
        .map(|(e, _, w)| (e.id.as_str(), w / total))
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
    // The five likeliest, and the drawn item when it isn't one of them, so the record always says what was picked.
    let kept: Vec<(&str, f64)> = ranked
        .iter()
        .enumerate()
        .filter(|(n, (i, _))| *n < ODDS || *i == chosen.0.id)
        .map(|(_, r)| *r)
        .collect();
    let rest: f64 = 1.0 - kept.iter().map(|(_, c)| c).sum::<f64>();
    Some((
        Pick {
            item: chosen.0.id.clone(),
            related: chosen.1,
            odds: kept
                .iter()
                .map(|(i, c)| ((*i).to_owned(), round(*c)))
                .collect(),
            rest: round(rest.max(0.0)),
        },
        rng,
    ))
}

/// Where an item of `land` goes on `day`: next to what that day drew, otherwise the nearest suitable free cell
/// around the world, never on anything drawn earlier (journey FR-013, journey R6).
pub(crate) fn place(id: &str, day: &str, land: Land, drawn: &[Drawn]) -> (i32, i32) {
    let terrain = Terrain::new(id);
    let occupied: BTreeSet<(i32, i32)> = drawn.iter().map(|d| (d.x, d.y)).collect();
    let fits = |&(x, y): &(i32, i32)| {
        !occupied.contains(&(x, y)) && land.suits(terrain.at(x, y), x.abs().max(y.abs()) <= 4)
    };
    let home = |&(x, y): &(i32, i32)| (x * x + y * y, y, x);
    let today: Vec<(i32, i32)> = drawn
        .iter()
        .filter(|d| d.day == day)
        .map(|d| (d.x, d.y))
        .collect();
    let near = |cell: &(i32, i32)| {
        today
            .iter()
            .map(|t| (t.0 - cell.0).pow(2) + (t.1 - cell.1).pow(2))
            .min()
    };
    let beside: BTreeSet<(i32, i32)> = today
        .iter()
        .flat_map(|&(x, y)| {
            (-REACH..=REACH).flat_map(move |dx| (-REACH..=REACH).map(move |dy| (x + dx, y + dy)))
        })
        .filter(fits)
        .collect();
    if let Some(cell) = beside.iter().min_by_key(|c| (near(c), home(c))) {
        return *cell;
    }
    let around: Vec<(i32, i32)> = if occupied.is_empty() {
        vec![(0, 0)]
    } else {
        occupied.iter().copied().collect()
    };
    let ring = |r: i32| {
        around.iter().flat_map(move |&(x, y)| {
            (-r..=r).flat_map(move |d| {
                [
                    (x + d, y - r),
                    (x + d, y + r),
                    (x - r, y + d),
                    (x + r, y + d),
                ]
            })
        })
    };
    for r in 1..=RINGS {
        if let Some(cell) = ring(r).filter(fits).min_by_key(home) {
            return cell;
        }
    }
    ring(1)
        .filter(|c| !occupied.contains(c))
        .min_by_key(home)
        .unwrap_or((0, 0))
}

#[cfg(test)]
mod tests {
    use super::super::dictionary;
    use super::*;

    const ID: &str = "K7Q2-M9TX-3HPA";

    fn drawn(item: &str, x: i32, y: i32, day: &str) -> Drawn {
        Drawn {
            item: item.into(),
            x,
            y,
            size: 1.0,
            flip: false,
            shade: 0,
            day: day.into(),
            why: "ambient".into(),
        }
    }

    fn subjects_of(item: &str) -> Vec<String> {
        dictionary()
            .iter()
            .find(|e| e.id == item)
            .unwrap()
            .subjects
            .clone()
    }

    #[test]
    fn candidates_come_from_the_topics_subjects_and_their_related_ones_only() {
        let topic = vec!["running".to_owned()];
        let related: Vec<&str> = subjects()
            .iter()
            .find(|s| s.id == "running")
            .unwrap()
            .related
            .iter()
            .map(String::as_str)
            .collect();
        for (entry, is_related) in candidates(dictionary(), &topic) {
            if is_related {
                assert!(
                    !entry.subjects.contains(&topic[0])
                        && entry.subjects.iter().any(|s| related.contains(&s.as_str())),
                    "{}",
                    entry.id
                );
            } else {
                assert!(entry.subjects.contains(&topic[0]), "{}", entry.id);
            }
        }
        // journey FR-010: never an item of an unrelated subject, and never scenery.
        let all: Vec<&str> = candidates(dictionary(), &topic)
            .iter()
            .map(|(e, _)| e.id.as_str())
            .collect();
        assert!(
            all.contains(&"shoes.running")
                && !all.contains(&"cake.birthday")
                && !all.contains(&"flower.daisy")
        );
    }

    #[test]
    fn a_pick_records_its_odds_and_is_the_same_every_time() {
        let world = vec![drawn("home.cottage", 0, 0, "2026-11-01")];
        let topic = vec!["cooking".to_owned()];
        let (pick, _) = pick(ID, "2026-11-10", 0, &topic, dictionary(), &world).unwrap();
        assert!(
            (5..=6).contains(&pick.odds.len())
                && pick.odds[..5].windows(2).all(|w| w[0].1 >= w[1].1)
        );
        // The drawn item is always in the record, likely or not.
        for n in 0..40 {
            let (other, _) =
                super::pick(ID, "2026-11-10", n, &topic, dictionary(), &world).unwrap();
            assert!(other.odds.iter().any(|(i, _)| *i == other.item), "{n}");
        }
        let sum: f64 = pick.odds.iter().map(|(_, c)| c).sum::<f64>() + pick.rest;
        assert!((sum - 1.0).abs() < 0.01, "{sum}");
        assert!(subjects_of(&pick.item).contains(&topic[0]) || pick.related);
        // journey FR-012: the same seed, day, topic and world, the same pick; another position, its own roll.
        assert_eq!(
            super::pick(ID, "2026-11-10", 0, &topic, dictionary(), &world)
                .unwrap()
                .0,
            pick
        );
        let others: Vec<String> = (0..20)
            .map(|n| {
                super::pick(ID, "2026-11-10", n, &topic, dictionary(), &world)
                    .unwrap()
                    .0
                    .item
            })
            .collect();
        assert!(others.iter().any(|i| *i != pick.item));
    }

    #[test]
    fn an_object_goes_beside_its_days_items_on_suitable_free_land() {
        let terrain = Terrain::new(ID);
        // That day's scenery on a stretch of dry land away from the home.
        let (sx, sy) = (6..60)
            .map(|x| (x, 6))
            .find(|&(x, y)| {
                (-1..=2).all(|dx| (-1..=1).all(|dy| terrain.at(x + dx, y + dy) != Land::Water))
            })
            .unwrap();
        let world = vec![
            drawn("home.cottage", 0, 0, "2026-11-01"),
            drawn("flower.daisy", 1, 0, "2026-11-01"),
            drawn("grass.tuft", sx, sy, "2026-11-10"),
            drawn("clover.patch", sx + 1, sy, "2026-11-10"),
        ];
        let (x, y) = place(ID, "2026-11-10", Land::Ground, &world);
        assert!(
            world.iter().all(|d| (d.x, d.y) != (x, y)),
            "never on an earlier item"
        );
        assert!(terrain.at(x, y) != Land::Water);
        assert!(
            [(sx, sy), (sx + 1, sy)]
                .iter()
                .any(|(a, b)| (a - x).abs() <= REACH && (b - y).abs() <= REACH),
            "({x}, {y})"
        );
        // A day with nothing drawn yet: the nearest suitable free cell around the world.
        let (x, y) = place(ID, "2026-11-20", Land::Ground, &world);
        assert!(world.iter().all(|d| (d.x, d.y) != (x, y)) && terrain.at(x, y) != Land::Water);
        assert!(
            world
                .iter()
                .any(|d| (d.x - x).abs().max((d.y - y).abs()) == 1),
            "({x}, {y})"
        );
        // Water things land on water, however far.
        let (x, y) = place(ID, "2026-11-20", Land::Water, &world);
        assert_eq!(terrain.at(x, y), Land::Water);
    }

    #[test]
    fn repeats_and_recent_draws_weigh_less_for_objects_and_scenery() {
        let world = vec![
            drawn("laptop.open", 1, 0, "2026-10-01"),
            drawn("laptop.open", 2, 0, "2026-10-02"),
            drawn("laptop.open", 3, 0, "2026-10-03"),
            drawn("briefcase.leather", 4, 0, "2026-11-08"),
        ];
        let counts = tally(&world, "2026-11-10");
        let plain = |item: &str| preference(ID, item);
        // journey FR-011: divided by one plus the times drawn; halved when drawn within the seven days ending on the
        // day.
        assert!((weight(ID, "laptop.open", &counts) - plain("laptop.open") / 4.0).abs() < 1e-12);
        assert!(
            (weight(ID, "briefcase.leather", &counts) - plain("briefcase.leather") / 4.0).abs()
                < 1e-12
        );
        assert!((weight(ID, "lamp.desk", &counts) - plain("lamp.desk")).abs() < 1e-12);
        // Scenery grows with the same weights.
        assert!(
            (weight(
                ID,
                "flower.daisy",
                &tally(&[drawn("flower.daisy", 0, 1, "2026-11-10")], "2026-11-10")
            ) - plain("flower.daisy") / 4.0)
                .abs()
                < 1e-12
        );
    }

    /// Thirty days of one subject, one topic a day, as the night draws them.
    fn thirty_days_of_work() -> (Vec<Drawn>, Vec<(Pick, Vec<Drawn>)>) {
        let mut world = vec![drawn("home.cottage", 0, 0, "2026-10-01")];
        let mut picks = Vec::new();
        for d in 1..=30 {
            let day = format!("2026-10-{d:02}");
            let (chosen, _) =
                pick(ID, &day, 0, &["work".to_owned()], dictionary(), &world).unwrap();
            picks.push((chosen.clone(), world.clone()));
            let (x, y) = place(ID, &day, Land::Ground, &world);
            world.push(drawn(&chosen.item, x, y, &day));
        }
        (world, picks)
    }

    #[test]
    fn a_steady_subject_still_gives_a_varied_map_that_can_be_checked() {
        let (world, picks) = thirty_days_of_work();
        let topic = vec!["work".to_owned()];
        let allowed: Vec<&str> = candidates(dictionary(), &topic)
            .iter()
            .map(|(e, _)| e.id.as_str())
            .collect();
        let objects: Vec<&str> = world[1..].iter().map(|d| d.item.as_str()).collect();
        let mut kinds: Vec<&str> = objects.clone();
        kinds.sort_unstable();
        kinds.dedup();
        let most = kinds
            .iter()
            .map(|k| objects.iter().filter(|o| *o == k).count())
            .max()
            .unwrap();
        // journey SC-003: no item over 40%, at least five different ones, none from an unrelated subject.
        assert!(
            most * 10 <= objects.len() * 4,
            "{most} of {}",
            objects.len()
        );
        assert!(kinds.len() >= 5, "{kinds:?}");
        assert!(objects.iter().all(|o| allowed.contains(o)));
        // journey SC-004: the same inputs, the same picks, every time.
        for _ in 0..100 {
            assert_eq!(thirty_days_of_work().0, world);
        }
        // journey SC-005: every recorded chance matches a recomputation from the world as it was.
        for (chosen, before) in &picks {
            let day = &chosen_day(before);
            let counts = tally(before, day);
            let weights: Vec<(&str, f64)> = candidates(dictionary(), &topic)
                .iter()
                .map(|(e, related)| {
                    (
                        e.id.as_str(),
                        weight(ID, &e.id, &counts) * if *related { RELATED } else { 1.0 },
                    )
                })
                .collect();
            let total: f64 = weights.iter().map(|(_, w)| w).sum();
            for (item, chance) in &chosen.odds {
                let expected = weights.iter().find(|(i, _)| i == item).unwrap().1 / total;
                assert!(
                    (expected - chance).abs() <= 0.0005,
                    "{item}: {expected} vs {chance}"
                );
            }
        }
    }

    /// The day a pick was made for: the one after the newest item of the world it was made in.
    fn chosen_day(before: &[Drawn]) -> String {
        let newest = before.iter().map(|d| d.day.as_str()).max().unwrap();
        let n: u32 = newest[8..].parse().unwrap();
        format!("2026-10-{:02}", if before.len() == 1 { 1 } else { n + 1 })
    }
}
