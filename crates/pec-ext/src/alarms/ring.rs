//! What a tick rings and what it reports missed (extensions R4, extensions data 2 and 5).

use pec_core::ext::Tick;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};

use super::line::{Row, Status, stamp, unstamp};

/// extensions R4: due longer ago than this, outside the tick that should have rung it, is missed.
const LATE: Duration = Duration::minutes(2);

/// Bookkeeping in `state/alarms.json`, keyed by the line after its box (extensions data 2).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct State {
    pub(super) alarms: BTreeMap<String, Rang>,
    pub(super) last_tick: Option<LastTick>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct Rang {
    /// The time it last rang, `YYYY-MM-DD HH:MM`.
    pub(super) rang: Option<String>,
    pub(super) snoozed_until: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct LastTick {
    /// Unix seconds, to tell sleep from a clock change.
    unix: i64,
    /// Local time, `YYYY-MM-DD HH:MM`, with seconds dropped.
    #[serde(with = "local")]
    local: PrimitiveDateTime,
}

mod local {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::PrimitiveDateTime;

    pub fn serialize<S: Serializer>(at: &PrimitiveDateTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{} {:02}", super::stamp(*at), at.second()))
    }

    pub fn deserialize<'d, D: Deserializer<'d>>(d: D) -> Result<PrimitiveDateTime, D::Error> {
        let text = String::deserialize(d)?;
        let (minute, second) = text
            .rsplit_once(' ')
            .ok_or_else(|| serde::de::Error::custom("time"))?;
        super::unstamp(minute)
            .and_then(|at| at.replace_second(second.parse().ok()?).ok())
            .ok_or_else(|| serde::de::Error::custom("time"))
    }
}

/// What is due at `now`: rung in time, or missed while the app or the computer slept (extensions data 5).
/// `first`: the first tick since the app started, when a one-time alarm left ringing is reported missed.
pub(super) fn tick(
    rows: &mut [Row],
    state: &mut State,
    now: OffsetDateTime,
    first: bool,
) -> (Tick, bool) {
    let here = PrimitiveDateTime::new(now.date(), now.time())
        .replace_nanosecond(0)
        .unwrap_or(PrimitiveDateTime::new(now.date(), now.time()));
    let last = state.last_tick;
    let mut tick = Tick::default();
    let mut rewrite = false;
    for row in rows.iter_mut() {
        let Row::Alarm(line) = row else { continue };
        if line.state != Status::Waiting {
            continue;
        }
        let rang = state.alarms.entry(line.body()).or_default();
        if let Some(until) = rang.snoozed_until.as_deref().and_then(unstamp) {
            if until <= here {
                tick.ring.push(line.due(until));
                rang.snoozed_until = None;
            }
            continue;
        }
        let since = rang.rang.as_deref().and_then(unstamp);
        let due = match line.once() {
            // Rung and not answered yet; after a restart the notification is gone, so it was missed.
            Some(at) if since.is_some() => {
                if first && here - at > LATE {
                    line.state = Status::Missed;
                    rewrite = true;
                    tick.missed.push(line.due(at));
                }
                continue;
            }
            Some(at) => at,
            // Repeating alarms count from the last tick; with no bookkeeping, from now.
            None => match last.map(|l| l.local).max(since) {
                Some(from) => match line.next_after(from) {
                    Some(at) => at,
                    None => continue,
                },
                None => continue,
            },
        };
        if due > here {
            continue;
        }
        // In this tick's window while the app ran, or only just late: it rings. A clock change moves the local
        // time but not the elapsed time, so an alarm in a skipped hour still rings.
        let in_window = last.is_some_and(|l| {
            due > l.local && now.unix_timestamp() - l.unix <= LATE.whole_seconds()
        });
        if in_window || here - due <= LATE {
            tick.ring.push(line.due(due));
            rang.rang = Some(stamp(due));
        } else {
            tick.missed.push(line.due(due));
            match line.once() {
                Some(_) => {
                    line.state = Status::Missed;
                    rewrite = true;
                }
                None => {
                    let mut latest = due;
                    while let Some(next) = line.next_after(latest).filter(|n| *n <= here) {
                        latest = next;
                    }
                    rang.rang = Some(stamp(latest));
                }
            }
        }
    }
    let bodies: Vec<String> = rows
        .iter()
        .filter_map(|r| match r {
            Row::Alarm(l) => Some(l.body()),
            Row::Text(_) => None,
        })
        .collect();
    state.alarms.retain(|body, _| bodies.contains(body));
    state.last_tick = Some(LastTick {
        unix: now.unix_timestamp(),
        local: here,
    });
    (tick, rewrite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alarms::line::rows;
    use crate::alarms::tests::at;
    use time::macros::datetime;

    /// Ticks every 15 seconds from `from` to `to`, collecting what rang and what was missed.
    fn run_ticks(
        rows: &mut [Row],
        state: &mut State,
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> (Vec<String>, Vec<String>) {
        let (mut rang, mut missed) = (Vec::new(), Vec::new());
        let mut now = from;
        let mut first = state.last_tick.is_none();
        while now <= to {
            let (tick, _) = tick(rows, state, now, first);
            first = false;
            rang.extend(
                tick.ring
                    .iter()
                    .map(|d| format!("{} {} {}", d.date, d.time, d.label)),
            );
            missed.extend(
                tick.missed
                    .iter()
                    .map(|d| format!("{} {} {}", d.date, d.time, d.label)),
            );
            now += Duration::seconds(15);
        }
        (rang, missed)
    }

    #[test]
    fn a_tick_rings_what_fell_due_and_repeating_alarms_move_on() {
        let mut rows = rows(
            "- [ ] 2026-11-14 19:00 Call Mum\n- [ ] every day 19:00 Tea\n- [ ] 2026-11-14 19:00 Stretch\n",
        );
        let mut state = State::default();
        let (rang, missed) = run_ticks(
            &mut rows,
            &mut state,
            at("2026-11-14 18:59"),
            at("2026-11-15 19:01"),
        );
        // Several due in the same minute ring together, in one tick.
        assert_eq!(
            rang,
            [
                "2026-11-14 19:00 Call Mum",
                "2026-11-14 19:00 Tea",
                "2026-11-14 19:00 Stretch",
                "2026-11-15 19:00 Tea"
            ]
        );
        assert!(missed.is_empty());
    }

    #[test]
    fn missed_alarms_are_reported_once_and_never_rung_late() {
        let text = "- [ ] 2026-11-14 21:00 Water the plants\n- [ ] every day 21:00 Tea\n- [ ] 2026-11-14 23:30 Later\n";
        let mut rows = rows(text);
        let mut state = State::default();
        run_ticks(
            &mut rows,
            &mut state,
            at("2026-11-14 20:00"),
            at("2026-11-14 20:01"),
        );
        // The computer slept from 20:01 to 23:00, and to the day after for the repeating one.
        let (rang, missed) = run_ticks(
            &mut rows,
            &mut state,
            at("2026-11-15 22:00"),
            at("2026-11-15 22:01"),
        );
        assert!(rang.is_empty(), "{rang:?}");
        assert_eq!(
            missed,
            [
                "2026-11-14 21:00 Water the plants",
                "2026-11-14 21:00 Tea",
                "2026-11-14 23:30 Later"
            ]
        );
        assert!(matches!(&rows[0], Row::Alarm(l) if l.state == Status::Missed));
        // Reported once; the repeating one rings again at its next time.
        let (rang, missed) = run_ticks(
            &mut rows,
            &mut state,
            at("2026-11-15 22:01"),
            at("2026-11-16 21:00"),
        );
        assert!(missed.is_empty());
        assert_eq!(rang, ["2026-11-16 21:00 Tea"]);
    }

    #[test]
    fn a_restart_while_ringing_reports_the_alarm_missed() {
        let mut rows = rows("- [ ] 2026-11-14 21:00 Call Mum\n");
        let mut state = State::default();
        run_ticks(
            &mut rows,
            &mut state,
            at("2026-11-14 20:59"),
            at("2026-11-14 21:00"),
        );
        let (_, missed) = run_ticks(
            &mut rows,
            &mut state,
            at("2026-11-14 21:10"),
            at("2026-11-14 21:10"),
        );
        assert!(missed.is_empty(), "same run: still ringing");
        let (tick, _) = tick(&mut rows, &mut state, at("2026-11-14 21:20"), true);
        assert_eq!(tick.missed.len(), 1);
    }

    #[test]
    fn lost_bookkeeping_counts_repeating_alarms_from_now() {
        let mut rows = rows("- [ ] every day 21:00 Tea\n");
        let mut state = State::default();
        let (rang, missed) = run_ticks(
            &mut rows,
            &mut state,
            at("2026-11-14 22:00"),
            at("2026-11-14 22:01"),
        );
        assert!(rang.is_empty() && missed.is_empty());
    }

    #[test]
    fn a_clock_change_or_a_new_time_zone_rings_at_local_time() {
        // Clocks go forward from 02:00 to 03:00: 02:30 never shows, so it rings at the first tick after.
        let mut rows = rows("- [ ] every day 02:30 Night pill\n");
        let mut state = State::default();
        let before = datetime!(2026-03-29 01:59:50)
            .assume_offset(time::UtcOffset::from_hms(1, 0, 0).unwrap());
        tick(&mut rows, &mut state, before, true);
        let after = datetime!(2026-03-29 03:00:05)
            .assume_offset(time::UtcOffset::from_hms(2, 0, 0).unwrap());
        let (t, _) = tick(&mut rows, &mut state, after, false);
        assert_eq!(t.ring.len(), 1);
        // Travelling west, the computer's zone changes from Tokyo to Jakarta at 08:00: the clock shows 06:00, and
        // the alarm rings when it shows 09:00 there, not at 09:00 Tokyo time.
        let mut rows = crate::alarms::line::rows("- [ ] every day 09:00 Call home\n");
        let mut state = State::default();
        let tokyo = time::UtcOffset::from_hms(9, 0, 0).unwrap();
        let jakarta = time::UtcOffset::from_hms(7, 0, 0).unwrap();
        let start = datetime!(2026-11-14 08:00).assume_offset(tokyo);
        tick(&mut rows, &mut state, start, true);
        let mut rang = Vec::new();
        for i in 1..=(4 * 60 * 4) {
            let now = (start + Duration::seconds(15 * i)).to_offset(jakarta);
            let (t, _) = tick(&mut rows, &mut state, now, false);
            if !t.ring.is_empty() {
                rang.push(now.time());
            }
        }
        assert_eq!(rang, [time::macros::time!(09:00)]);
    }
}
