//! `world/world.jsonl`: a header naming the companion, then one drawn item per line, only ever appended to
//! (world data 2, world R5).

use serde::{Deserialize, Serialize};

pub(crate) const WORLD_FILE: &str = "world/world.jsonl";

/// The first line: which companion the world belongs to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Header {
    pub world: u32,
    pub companion: String,
    pub born: String,
}

/// One thing the companion drew.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Drawn {
    pub item: String,
    pub x: i32,
    pub y: i32,
    #[serde(default = "one")]
    pub size: f64,
    #[serde(default)]
    pub flip: bool,
    #[serde(default)]
    pub shade: u8,
    pub day: String,
    #[serde(default = "ambient")]
    pub why: String,
}

fn one() -> f64 {
    1.0
}

fn ambient() -> String {
    "ambient".into()
}

/// The world as the file holds it.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct World {
    pub header: Option<Header>,
    pub items: Vec<Drawn>,
    /// Lines that couldn't be read; they stay in the file (world FR-015).
    pub skipped: usize,
}

impl World {
    pub fn parse(text: &str) -> Self {
        let mut world = Self::default();
        for (i, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
            if i == 0
                && let Ok(header) = serde_json::from_str::<Header>(line)
            {
                world.header = Some(header);
                continue;
            }
            match serde_json::from_str::<Drawn>(line) {
                Ok(item) => world.items.push(item),
                Err(_) => world.skipped += 1,
            }
        }
        world
    }
}

pub(crate) fn line<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("a world line serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn daisy() -> Drawn {
        Drawn {
            item: "flower.daisy".into(),
            x: 3,
            y: -2,
            size: 0.9,
            flip: false,
            shade: 1,
            day: "2026-11-14".into(),
            why: "ambient".into(),
        }
    }

    #[test]
    fn lines_round_trip_and_defaults_fill_in() {
        assert_eq!(
            line(&daisy()),
            r#"{"item":"flower.daisy","x":3,"y":-2,"size":0.9,"flip":false,"shade":1,"day":"2026-11-14","why":"ambient"}"#
        );
        let header = Header {
            world: 1,
            companion: "K7Q2-M9TX-3HPA".into(),
            born: "2026-11-14".into(),
        };
        let text = format!(
            "{}\n{}\n{}\n",
            line(&header),
            line(&daisy()),
            r#"{"item":"pebble.round","x":0,"y":1,"day":"2026-11-15"}"#
        );
        let world = World::parse(&text);
        assert_eq!(world.header, Some(header));
        assert_eq!(world.items[0], daisy());
        let pebble = &world.items[1];
        assert_eq!(
            (pebble.size, pebble.flip, pebble.shade, pebble.why.as_str()),
            (1.0, false, 0, "ambient")
        );
    }

    #[test]
    fn unreadable_lines_are_skipped_and_counted() {
        let text = format!(
            "{}\nnot json\n{}\n{}\n",
            line(&daisy()),
            r#"{"item":"rock.small","x":1}"#,
            r#"{"x":1,"y":1,"day":"2026-11-14"}"#
        );
        let world = World::parse(&text);
        // A first line that isn't a header is an item.
        assert_eq!(world.header, None);
        assert_eq!(world.items, [daisy()]);
        assert_eq!(world.skipped, 3);
    }

    #[test]
    fn a_line_stays_under_200_bytes() {
        let longest = super::super::dictionary()
            .iter()
            .map(|e| e.id.len())
            .max()
            .unwrap();
        let extreme = Drawn {
            item: "x".repeat(longest),
            x: -99_999,
            y: -99_999,
            size: 1.2,
            flip: true,
            shade: 2,
            ..daisy()
        };
        assert!(line(&extreme).len() < 200, "{}", line(&extreme).len());
    }
}
