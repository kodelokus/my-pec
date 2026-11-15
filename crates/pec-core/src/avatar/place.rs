//! The floating window: whether it is on and where it goes (avatar R8).

use crate::{Core, Result};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct FloatingSettings {
    pub on: bool,
}

/// A screen's work area in physical pixels, and its scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub primary: bool,
}

/// Logical pixels between the window and the corner it is moved to.
const MARGIN: f64 = 24.0;

/// avatar FR-025: the saved place when the window fits inside one screen's work area, otherwise the bottom-right corner
/// of the primary screen's. `size` is in logical pixels.
pub fn place(saved: Option<(i32, i32)>, size: (u32, u32), screens: &[Screen]) -> (i32, i32) {
    let scaled = |v: f64, s: &Screen| (v * s.scale).round() as i32;
    let fits = |(x, y): (i32, i32), s: &Screen| {
        x >= s.x
            && y >= s.y
            && x + scaled(size.0.into(), s) <= s.x + s.width as i32
            && y + scaled(size.1.into(), s) <= s.y + s.height as i32
    };
    if let Some(at) = saved
        && screens.iter().any(|s| fits(at, s))
    {
        return at;
    }
    let Some(s) = screens.iter().find(|s| s.primary).or(screens.first()) else {
        return saved.unwrap_or_default();
    };
    (
        s.x + s.width as i32 - scaled(size.0.into(), s) - scaled(MARGIN, s),
        s.y + s.height as i32 - scaled(size.1.into(), s) - scaled(MARGIN, s),
    )
}

impl Core {
    pub fn floating_settings(&self) -> Result<FloatingSettings> {
        Ok(FloatingSettings {
            on: self.settings()?.avatar.floating.on,
        })
    }

    /// avatar FR-023: off by default; the shell opens or closes the window.
    pub fn set_floating(&self, on: bool) -> Result<FloatingSettings> {
        let mut settings = self.settings()?;
        settings.avatar.floating.on = on;
        self.save_settings(&settings)?;
        self.floating_settings()
    }

    pub fn floating_position(&self) -> Result<Option<(i32, i32)>> {
        let f = self.settings()?.avatar.floating;
        Ok(f.x.zip(f.y))
    }

    pub fn save_floating_position(&self, x: i32, y: i32) -> Result<()> {
        let mut settings = self.settings()?;
        (settings.avatar.floating.x, settings.avatar.floating.y) = (Some(x), Some(y));
        self.save_settings(&settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: (u32, u32) = (200, 260);

    fn screen(x: i32, scale: f64, primary: bool) -> Screen {
        Screen {
            x,
            y: 0,
            width: 1920,
            height: 1040,
            scale,
            primary,
        }
    }

    #[test]
    fn a_saved_place_is_kept_only_while_the_window_fits_on_a_screen() {
        let main = screen(0, 1.0, true);
        let corner = (1920 - 200 - 24, 1040 - 260 - 24);
        let cases = [
            (Some((100, 100)), vec![main], (100, 100)),
            (Some((1800, 100)), vec![main], corner),
            (Some((2000, 100)), vec![main], corner),
            (
                Some((2000, 100)),
                vec![main, screen(1920, 1.0, false)],
                (2000, 100),
            ),
            (None, vec![main], corner),
            // The secondary screen is listed first; the corner is still the primary's.
            (None, vec![screen(1920, 1.0, false), main], corner),
            // At 150%, the window is 300 × 390 and the margin 36 on that screen.
            (
                Some((3500, 100)),
                vec![main, screen(1920, 1.5, false)],
                (3500, 100),
            ),
            (
                Some((3600, 100)),
                vec![main, screen(1920, 1.5, false)],
                corner,
            ),
            (
                None,
                vec![screen(0, 1.5, true)],
                (1920 - 300 - 36, 1040 - 390 - 36),
            ),
        ];
        for (saved, screens, expected) in cases {
            assert_eq!(place(saved, SIZE, &screens), expected, "{saved:?}");
        }
    }
}
