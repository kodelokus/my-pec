//! The companion's DNA, worked out from its id whenever needed and never stored (world FR-005, world R4).

use super::Land;
use crate::memory::files::fingerprint;

/// SplitMix64: a tiny, fully specified generator, so a world grows the same way in every app version.
pub(crate) struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }

    /// A number in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// A seed for one purpose, from memory's stable hash of the id.
pub(crate) fn seed(id: &str, purpose: &str) -> u64 {
    u64::from_str_radix(&fingerprint(&format!("{id}/{purpose}")), 16).unwrap_or(0)
}

/// Ground, water, leaf, bark, stone and accent, as `#RRGGBB`.
const PALETTES: [[&str; 6]; 8] = [
    [
        "#dfe8c8", "#8cc3d8", "#6aa84f", "#8b5e3c", "#a7a39a", "#e8a33d",
    ],
    [
        "#efe3c6", "#7fb2c9", "#88a64a", "#7a5232", "#b3aa9b", "#d9534f",
    ],
    [
        "#d6e6d2", "#6fa8b8", "#4f9a6b", "#6d4c3a", "#9aa3a0", "#f2c14e",
    ],
    [
        "#e9dcc9", "#9ab8d6", "#9bb35a", "#946846", "#b8b0a2", "#c46fb0",
    ],
    [
        "#dde3d0", "#5f9fb5", "#5c8f4a", "#5e4535", "#8e938c", "#ef8354",
    ],
    [
        "#f1e6d0", "#88bfc4", "#7fb069", "#a0714f", "#c2b9a7", "#5b8bd9",
    ],
    [
        "#d9e2c3", "#7aa6b8", "#6b9e5a", "#7d5a44", "#a29d93", "#e4c34a",
    ],
    [
        "#e4ecd9", "#94c1cf", "#71a85e", "#866047", "#aeb2a8", "#d06a7a",
    ],
];

/// The palette's colours, a little lighter or darker per companion.
pub(crate) fn palette(id: &str) -> [String; 6] {
    let mut rng = Rng::new(seed(id, "palette"));
    let chosen = PALETTES[rng.below(PALETTES.len() as u64) as usize];
    let factor = 0.92 + rng.unit() * 0.16;
    chosen.map(|hex| {
        let channel = |i: usize| {
            let value = u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            (f64::from(value) * factor).round().clamp(0.0, 255.0) as u8
        };
        format!("#{:02x}{:02x}{:02x}", channel(1), channel(3), channel(5))
    })
}

/// The index of the palette an id uses, for the test that checks every palette appears.
#[cfg(test)]
pub(crate) fn palette_index(id: &str) -> usize {
    Rng::new(seed(id, "palette")).below(PALETTES.len() as u64) as usize
}

/// A value in `[0, 1)` at a lattice point.
fn lattice(seed: u64, x: i64, y: i64) -> f64 {
    let mixed = seed
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    Rng::new(mixed).unit()
}

/// Smooth value noise with lattice points every `scale` cells.
fn octave(seed: u64, x: i32, y: i32, scale: i32) -> f64 {
    let (fx, fy) = (
        f64::from(x) / f64::from(scale),
        f64::from(y) / f64::from(scale),
    );
    let (x0, y0) = (fx.floor() as i64, fy.floor() as i64);
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let (tx, ty) = (smooth(fx - x0 as f64), smooth(fy - y0 as f64));
    let top = lattice(seed, x0, y0) * (1.0 - tx) + lattice(seed, x0 + 1, y0) * tx;
    let bottom = lattice(seed, x0, y0 + 1) * (1.0 - tx) + lattice(seed, x0 + 1, y0 + 1) * tx;
    top * (1.0 - ty) + bottom * ty
}

/// The lay of the land, seeded by the id. The home and its neighbours are always meadow.
pub(crate) struct Terrain {
    seed: u64,
    water: f64,
    meadow: f64,
    woods: f64,
}

impl Terrain {
    pub fn new(id: &str) -> Self {
        let mut rng = Rng::new(seed(id, "land-thresholds"));
        let water = 0.30 + rng.unit() * 0.10;
        let meadow = water + 0.22 + rng.unit() * 0.10;
        let woods = meadow + 0.12 + rng.unit() * 0.08;
        Self {
            seed: seed(id, "land"),
            water,
            meadow,
            woods,
        }
    }

    pub fn at(&self, x: i32, y: i32) -> Land {
        if x.abs() <= 1 && y.abs() <= 1 {
            return Land::Meadow;
        }
        let value = 0.65 * octave(self.seed, x, y, 7) + 0.35 * octave(self.seed ^ 1, x, y, 3);
        if value < self.water {
            Land::Water
        } else if value < self.meadow {
            Land::Meadow
        } else if value < self.woods {
            Land::Woods
        } else {
            Land::Rock
        }
    }
}

/// How much this companion favours an item: 0.5 to 2.0.
pub(crate) fn preference(id: &str, item: &str) -> f64 {
    0.5 + 1.5 * Rng::new(seed(id, &format!("prefers/{item}"))).unit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_generator_is_splitmix64() {
        let mut rng = Rng::new(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
    }

    #[test]
    fn two_ids_get_different_colours_and_land() {
        let (a, b) = ("K7Q2-M9TX-3HPA", "Z9W8-V7U6-T5S4");
        assert_ne!(palette(a), palette(b));
        let land = |id: &str| -> Vec<Land> {
            let terrain = Terrain::new(id);
            (-20..20)
                .flat_map(|x| (-20..20).map(move |y| (x, y)))
                .map(|(x, y)| terrain.at(x, y))
                .collect()
        };
        assert_ne!(land(a), land(b));
        // The same id always gives the same land.
        assert_eq!(land(a), land(a));
        assert_eq!(palette(a), palette(a));
    }

    #[test]
    fn the_home_area_is_meadow_and_preferences_stay_in_range() {
        let terrain = Terrain::new("K7Q2-M9TX-3HPA");
        for (x, y) in [(0, 0), (1, 1), (-1, 0), (0, -1), (1, -1)] {
            assert_eq!(terrain.at(x, y), Land::Meadow);
        }
        for item in ["flower.daisy", "tree.oak", "boat.rowing"] {
            let w = preference("K7Q2-M9TX-3HPA", item);
            assert!((0.5..=2.0).contains(&w), "{item} {w}");
        }
    }
}
