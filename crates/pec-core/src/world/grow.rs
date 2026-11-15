//! Daily growth: a pure function of the companion's id, the local date and the world so far (world FR-019 to FR-024,
//! world R6). No model call, no catch-up, no decay.

use super::Entry;
use super::dna::{Rng, Terrain, seed};
use super::file::{Drawn, World};
use super::pick::{tally, weight};
use std::collections::BTreeSet;

/// A stage's items become available once the world holds this many items (world FR-023).
const STAGES: [usize; 4] = [0, 10, 70, 260];
/// New items land within this many cells of an item already there (world FR-022).
const REACH: i32 = 2;
/// `near-home` items stay within this many cells of the home.
const NEAR_HOME: i32 = 4;

/// The stage a world of `count` items has reached, 1 to 4.
pub(crate) fn stage(count: usize) -> u8 {
    STAGES.iter().filter(|start| count >= **start).count() as u8
}

/// The newest day the world grew. The home's own line doesn't count: it marks when the world began, not a day of
/// growth, so the first message on setup day still draws.
pub(crate) fn last_grown<'a>(world: &'a World, dictionary: &[Entry]) -> Option<&'a str> {
    world
        .items
        .iter()
        .filter(|d| !dictionary.iter().any(|e| e.home && e.id == d.item))
        .map(|d| d.day.as_str())
        .max()
}

/// The items drawn for `today`: none unless today is later than every day the world grew and the last drawn day
/// kept in state (world FR-019, FR-020); otherwise 2 or 3, each next to what is there (world FR-022), following the
/// DNA, the land and the stage (world FR-023). The same id, date and world always give the same items (world FR-024).
pub(crate) fn grow(
    id: &str,
    today: &str,
    world: &World,
    dictionary: &[Entry],
    last_day: Option<&str>,
) -> Vec<Drawn> {
    let newest = last_grown(world, dictionary).max(last_day);
    if newest.is_some_and(|day| day >= today) {
        return Vec::new();
    }
    let mut rng = Rng::new(seed(id, &format!("grow/{today}")));
    // journey FR-015: 2 or 3, so a day with three topic objects holds at most six items.
    let count = 2 + rng.below(2);
    let terrain = Terrain::new(id);
    // Each item's weight is worked out once; looking it up keeps a day's growth cheap in a large world. Repeats and
    // recent draws weigh less, as for topic objects (journey FR-011).
    let counts = tally(&world.items, today);
    let weights: Vec<f64> = dictionary
        .iter()
        .map(|e| weight(id, &e.id, &counts))
        .collect();
    let mut occupied: BTreeSet<(i32, i32)> = world.items.iter().map(|d| (d.x, d.y)).collect();
    // The free cells within reach, found once and kept up to date as items are placed.
    let mut free = frontier(&occupied);
    let mut placed: Vec<Drawn> = Vec::new();
    for _ in 0..count {
        let reached = stage(world.items.len() + placed.len());
        let mut candidates: Vec<(i32, i32)> = free.iter().copied().collect();
        candidates.sort_by_key(|&(x, y)| (x * x + y * y, y, x));
        if candidates.is_empty() {
            break;
        }
        // Weighted towards the home: the square of a uniform number favours the front of the list.
        let start = (rng.unit().powi(2) * candidates.len() as f64) as usize;
        for k in 0..candidates.len() {
            let (x, y) = candidates[(start + k) % candidates.len()];
            let land = terrain.at(x, y);
            let near_home = x.abs().max(y.abs()) <= NEAR_HOME;
            let eligible: Vec<(&Entry, f64)> = dictionary
                .iter()
                .zip(weights.iter().copied())
                .filter(|(e, _)| !e.home && e.subjects.is_empty() && e.stage <= reached)
                .filter(|(e, _)| e.land.suits(land, near_home))
                .collect();
            let Some(entry) = pick(&mut rng, &eligible) else {
                continue;
            };
            placed.push(Drawn {
                item: entry.id.clone(),
                x,
                y,
                size: f64::from(8 + rng.below(5) as u8) / 10.0,
                flip: rng.below(2) == 1,
                shade: rng.below(3) as u8,
                day: today.to_owned(),
                why: "ambient".into(),
            });
            if occupied.is_empty() {
                free.clear();
            }
            occupied.insert((x, y));
            free.remove(&(x, y));
            free.extend(around(x, y).filter(|cell| !occupied.contains(cell)));
            break;
        }
    }
    placed
}

/// The cells within reach of one cell.
fn around(x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> {
    (-REACH..=REACH).flat_map(move |dx| (-REACH..=REACH).map(move |dy| (x + dx, y + dy)))
}

/// Free cells within reach of a drawn item; around the centre for an empty world.
fn frontier(occupied: &BTreeSet<(i32, i32)>) -> BTreeSet<(i32, i32)> {
    if occupied.is_empty() {
        return around(0, 0).collect();
    }
    occupied
        .iter()
        .flat_map(|&(x, y)| around(x, y))
        .filter(|cell| !occupied.contains(cell))
        .collect()
}

/// An item chosen with the companion's preferences as weights.
fn pick<'a>(rng: &mut Rng, eligible: &[(&'a Entry, f64)]) -> Option<&'a Entry> {
    let total: f64 = eligible.iter().map(|(_, w)| w).sum();
    if eligible.is_empty() || total <= 0.0 {
        return None;
    }
    let mut target = rng.unit() * total;
    for (entry, weight) in eligible {
        if target < *weight {
            return Some(entry);
        }
        target -= weight;
    }
    eligible.last().map(|(e, _)| *e)
}

#[cfg(test)]
mod tests {
    use super::super::file::line;
    use super::super::{Land, dictionary};
    use super::*;

    const ID: &str = "K7Q2-M9TX-3HPA";

    fn home(day: &str) -> Drawn {
        Drawn {
            item: "home.cottage".into(),
            x: 0,
            y: 0,
            size: 1.0,
            flip: false,
            shade: 0,
            day: day.into(),
            why: "ambient".into(),
        }
    }

    fn world_with(items: Vec<Drawn>) -> World {
        World {
            header: None,
            items,
            skipped: 0,
        }
    }

    #[test]
    fn a_day_draws_a_small_handful_next_to_what_is_there() {
        let world = world_with(vec![home("2026-11-14")]);
        let drawn = grow(ID, "2026-11-14", &world, dictionary(), None);
        // The home's own line doesn't stop the first day from growing.
        assert!((2..=3).contains(&drawn.len()), "{}", drawn.len());
        let mut cells = BTreeSet::from([(0, 0)]);
        for item in &drawn {
            assert!(
                !dictionary().iter().any(|e| e.home && e.id == item.item),
                "never a home"
            );
            // Journey: scenery is never an everyday object, which only a day's topic draws.
            assert!(
                dictionary()
                    .iter()
                    .any(|e| e.id == item.item && e.subjects.is_empty())
            );
            assert!(
                cells
                    .iter()
                    .any(|(x, y)| (x - item.x).abs() <= REACH && (y - item.y).abs() <= REACH)
            );
            assert!(cells.insert((item.x, item.y)), "one item per cell");
            assert_eq!(item.day, "2026-11-14");
        }
        // world FR-024: the same inputs, the same items.
        assert_eq!(grow(ID, "2026-11-14", &world, dictionary(), None), drawn);
    }

    #[test]
    fn nothing_is_drawn_twice_or_for_an_earlier_day() {
        let mut items = vec![home("2026-11-01")];
        items.extend(grow(
            ID,
            "2026-11-10",
            &world_with(items.clone()),
            dictionary(),
            None,
        ));
        let world = world_with(items);
        assert!(grow(ID, "2026-11-10", &world, dictionary(), None).is_empty());
        assert!(grow(ID, "2026-11-09", &world, dictionary(), None).is_empty());
        // The last drawn day kept in state counts even when the file no longer shows it.
        let bare = world_with(vec![home("2026-11-01")]);
        assert!(grow(ID, "2026-11-10", &bare, dictionary(), Some("2026-11-10")).is_empty());
        assert!(!grow(ID, "2026-11-11", &bare, dictionary(), Some("2026-11-10")).is_empty());
    }

    #[test]
    fn an_empty_world_grows_from_the_centre() {
        let drawn = grow(
            ID,
            "2026-11-14",
            &world_with(Vec::new()),
            dictionary(),
            None,
        );
        assert!(drawn.iter().any(|d| (d.x, d.y) == (0, 0)));
    }

    /// Grows a world from its home over a fixed run of days with gaps, as the app would.
    fn grown(id: &str, days: usize) -> World {
        let mut world = world_with(vec![home("2026-01-01")]);
        for day in (0..days).filter(|d| d % 5 != 3) {
            // Valid, increasing dates over any number of years: 12 months of 28 days.
            let date = format!(
                "{}-{:02}-{:02}",
                2026 + day / 336,
                1 + day % 336 / 28,
                1 + day % 28
            );
            let drawn = grow(id, &date, &world, dictionary(), None);
            world.items.extend(drawn);
        }
        world
    }

    #[test]
    fn growth_is_the_same_every_time() {
        // world SC-002: compare with the committed file, byte for byte.
        let text: String = grown(ID, 40).items.iter().map(|d| line(d) + "\n").collect();
        let golden = include_str!("../../tests/fixtures/world/golden.jsonl");
        assert_eq!(text, golden.replace("\r\n", "\n"));
    }

    #[test]
    fn stages_open_with_size_and_items_grow_where_they_belong() {
        assert_eq!(
            [
                stage(0),
                stage(9),
                stage(10),
                stage(69),
                stage(70),
                stage(260)
            ],
            [1, 1, 2, 2, 3, 4]
        );
        let world = grown(ID, 160);
        let terrain = Terrain::new(ID);
        let mut seen = BTreeSet::new();
        for (before, drawn) in world.items.iter().enumerate().skip(1) {
            let entry = dictionary().iter().find(|e| e.id == drawn.item).unwrap();
            // world FR-023: never from a stage the world hadn't reached when it was drawn.
            assert!(entry.stage <= stage(before), "{} at {before}", drawn.item);
            seen.insert(entry.stage);
            let land = terrain.at(drawn.x, drawn.y);
            match entry.land {
                Land::Any => {}
                Land::NearHome => {
                    assert!(drawn.x.abs().max(drawn.y.abs()) <= NEAR_HOME && land != Land::Water)
                }
                kind => assert_eq!(kind, land, "{} on {land:?}", drawn.item),
            }
        }
        assert_eq!(
            seen,
            BTreeSet::from([1, 2, 3, 4]),
            "every stage appears in a long-lived world"
        );
        // world FR-013: growing only ever adds after what is there.
        let shorter = grown(ID, 60);
        assert_eq!(world.items[..shorter.items.len()], shorter.items[..]);
    }

    /// world SC-001 and SC-009, run by hand in release mode (world checks).
    #[test]
    #[ignore = "release-readiness measurement"]
    fn a_thousand_companions() {
        use super::super::dna::palette_index;
        let started = std::time::Instant::now();
        let ids: Vec<String> = (0..1_000).map(|_| super::super::id::new_id()).collect();
        let mut worlds = BTreeSet::new();
        let (mut palettes, mut lands) = (BTreeSet::new(), BTreeSet::new());
        for id in &ids {
            let world = grown(id, 30);
            let terrain = Terrain::new(id);
            for d in &world.items {
                lands.insert(format!("{:?}", terrain.at(d.x, d.y)));
            }
            palettes.insert(palette_index(id));
            worlds.insert(world.items.iter().map(line).collect::<Vec<_>>().join(
                "
",
            ));
        }
        assert_eq!(worlds.len(), 1_000, "no two worlds are the same");
        assert_eq!(palettes.len(), 8, "every palette is used");
        assert_eq!(lands.len(), 4, "every kind of land is used");
        let decade: usize = grown(ID, 3_650)
            .items
            .iter()
            .map(|d| line(d).len() + 1)
            .sum();
        assert!(decade < 5_000_000, "{decade} bytes after ten years");
        println!("ten years: {decade} bytes; took {:?}", started.elapsed());
    }
}
