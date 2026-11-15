//! When a night is due: the local date of the most recent dream time, later than the last completed night, with the
//! person idle (dreaming R3, dreaming FR-001 to FR-004).

use super::IDLE_MS;
use crate::memory::files::previous_day;

/// The night the most recent dream time at or before the local `date` and `time` belongs to.
pub(crate) fn latest_night(date: &str, time: &str, dream_time: &str) -> String {
    if time >= dream_time {
        date.into()
    } else {
        previous_day(date)
    }
}

/// A valid `HH:MM` dream time.
pub(crate) fn is_time(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 5
        && b[2] == b':'
        && text[..2].parse::<u8>().is_ok_and(|h| h < 24)
        && text[3..].parse::<u8>().is_ok_and(|m| m < 60)
        && b[..2].iter().chain(&b[3..]).all(u8::is_ascii_digit)
}

/// Unix milliseconds of a validated RFC 3339 clock (`2026-11-15T09:00:05.123+07:00`): "now" for idleness, so the
/// injected clock decides it (dreaming FR-029).
pub(crate) fn unix_ms(iso: &str) -> Option<u64> {
    let n = |r: std::ops::Range<usize>| -> Option<i64> { iso.get(r)?.parse().ok() };
    let (y, mo, d, h, mi, s) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let tail = iso
        .get(19..)?
        .trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    let offset = match tail {
        "Z" => 0,
        t if t.len() == 6 => {
            let sign = if t.starts_with('-') { -1 } else { 1 };
            sign * (t.get(1..3)?.parse::<i64>().ok()? * 60 + t.get(4..6)?.parse::<i64>().ok()?)
        }
        _ => return None,
    };
    // Howard Hinnant's days-from-civil.
    let y = if mo <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((mo + 9) % 12) + 2) / 5 + d - 1;
    let days = era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468;
    let seconds = days * 86_400 + h * 3_600 + mi * 60 + s - offset * 60;
    u64::try_from(seconds).ok().map(|s| s * 1000)
}

/// What decides whether a night is due, gathered by the caller.
pub(crate) struct Moment<'a> {
    pub night: &'a str,
    pub last_night: Option<&'a str>,
    /// Unix milliseconds now, and of the last conversation entry or the core's start, whichever is later.
    pub now_ms: u64,
    pub active_ms: u64,
    /// A reply or a compaction runs, or a reply is owed.
    pub busy: bool,
}

/// Whether the night is due (dreaming R3).
pub(crate) fn due(m: &Moment) -> bool {
    m.last_night.is_none_or(|last| m.night > last)
        && !m.busy
        && m.now_ms.saturating_sub(m.active_ms) >= IDLE_MS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_latest_night_is_the_date_of_the_last_dream_time() {
        for (date, time, dream, night) in [
            ("2026-11-15", "02:59", "03:00", "2026-11-14"),
            ("2026-11-15", "03:00", "03:00", "2026-11-15"),
            ("2026-11-15", "23:10", "03:00", "2026-11-15"),
            ("2026-11-15", "13:59", "14:00", "2026-11-14"),
            ("2026-03-01", "01:00", "03:00", "2026-02-28"),
        ] {
            assert_eq!(latest_night(date, time, dream), night, "{date} {time}");
        }
    }

    #[test]
    fn a_night_is_due_once_when_the_person_is_idle() {
        let minute = 60_000;
        let at = |night, last, idle_minutes: u64, busy| Moment {
            night,
            last_night: last,
            now_ms: 100 * minute,
            active_ms: (100 - idle_minutes) * minute,
            busy,
        };
        for (moment, expected, case) in [
            (
                at("2026-11-15", Some("2026-11-14"), 5, false),
                true,
                "missed night, idle",
            ),
            (at("2026-11-15", None, 5, false), true, "first night"),
            (
                at("2026-11-15", Some("2026-11-14"), 4, false),
                false,
                "not idle long enough",
            ),
            (
                at("2026-11-15", Some("2026-11-14"), 60, true),
                false,
                "a reply runs",
            ),
            (
                at("2026-11-15", Some("2026-11-15"), 60, false),
                false,
                "already dreamt",
            ),
            // The clock set back, or a flight west: an earlier night never runs again.
            (
                at("2026-11-14", Some("2026-11-15"), 60, false),
                false,
                "clock back",
            ),
        ] {
            assert_eq!(due(&moment), expected, "{case}");
        }
    }

    #[test]
    fn a_clock_is_read_as_unix_time() {
        assert_eq!(unix_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            unix_ms("2026-11-15T09:00:05.123+07:00"),
            Some(1_794_708_005_000)
        );
        assert_eq!(unix_ms("2026-11-15T02:00:05Z"), Some(1_794_708_005_000));
        assert_eq!(
            unix_ms("2026-11-14T21:00:05-05:00"),
            Some(1_794_708_005_000)
        );
        assert_eq!(unix_ms("nonsense"), None);
    }

    #[test]
    fn dream_times_are_validated() {
        for ok in ["00:00", "03:00", "23:59"] {
            assert!(is_time(ok), "{ok}");
        }
        for bad in ["24:00", "3:00", "03:60", "03-00", "ab:cd", "+3:00", ""] {
            assert!(!is_time(bad), "{bad}");
        }
    }
}
