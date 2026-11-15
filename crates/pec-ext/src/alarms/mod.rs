//! Alarms: one line each in `alarms.md`, rung by the shell's tick (extensions R4, R5, R8; extensions data 1, 2 and 5;
//! extensions interfaces).

mod line;
mod ring;
mod tools;

pub use line::Day;

use line::{Line, Row, Status, clock, date, hm, rows, stamp, unstamp, ymd};
use pec_core::ext::{ExtDir, Extension, Tick, ToolAnswer, write_atomic};
use pec_core::llm::ToolSpec;
use pec_core::{AppError, Result};
use ring::State;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime};
use ts_rs::TS;

/// extensions FR-021.
const SNOOZE: Duration = Duration::minutes(10);
/// Done and missed one-time alarms older than this are removed when the app next writes the file.
const KEEP_DAYS: i64 = 30;
const HEADER: &str = "# Alarms\n";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Repeat {
    Once { date: String },
    Daily,
    Weekdays { days: Vec<Day> },
    Weekly { day: Day },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AlarmState {
    Waiting,
    Snoozed,
    Done,
    Missed,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Alarm {
    pub key: String,
    pub label: String,
    /// `HH:MM`.
    pub time: String,
    pub repeat: Repeat,
    pub state: AlarmState,
    /// When it rings next, `YYYY-MM-DD HH:MM`.
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlarmList {
    pub alarms: Vec<Alarm>,
    /// Lines of `alarms.md` that couldn't be read; they are left as they are (extensions FR-025).
    pub unreadable: u32,
    /// Windows didn't show the last notification; set by the shell.
    pub notifications_held: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Action {
    Snooze,
    Done,
}

fn read(dir: &ExtDir) -> Vec<Row> {
    rows(&std::fs::read_to_string(dir.place()).unwrap_or_default())
}

/// Writes the file back, without done and missed one-time alarms older than 30 days.
fn write(dir: &ExtDir, rows: &[Row], today: Date) -> Result<()> {
    let old = today - Duration::days(KEEP_DAYS);
    let mut text: String = rows
        .iter()
        .filter_map(|r| match r {
            Row::Alarm(l)
                if l.state != Status::Waiting && l.once().is_some_and(|at| at.date() < old) =>
            {
                None
            }
            Row::Alarm(l) => Some(l.render()),
            Row::Text(t) => Some(t.clone()),
        })
        .collect::<Vec<_>>()
        .join("\n");
    text.push('\n');
    write_atomic(dir.place(), text.as_bytes())
}

fn unreadable(rows: &[Row]) -> u32 {
    rows.iter()
        .filter(|r| matches!(r, Row::Text(t) if !t.trim().is_empty() && !t.starts_with('#')))
        .count() as u32
}

fn find<'a>(rows: &'a mut [Row], key: &str) -> Option<&'a mut Line> {
    rows.iter_mut().find_map(|r| match r {
        Row::Alarm(l) if l.key() == key => Some(l),
        _ => None,
    })
}

/// The list as the alarm list and `alarm_list` show it: waiting ones soonest first, then done and missed.
pub fn list(dir: &ExtDir, now: PrimitiveDateTime) -> AlarmList {
    let rows = read(dir);
    let state: State = dir.read_state();
    let mut alarms: Vec<Alarm> = rows
        .iter()
        .filter_map(|r| match r {
            Row::Alarm(l) => Some(l),
            Row::Text(_) => None,
        })
        .map(|l| {
            let rang = state.alarms.get(&l.body()).cloned().unwrap_or_default();
            let snoozed = rang.snoozed_until.as_deref().and_then(unstamp);
            let (state, next) = match (l.state, snoozed) {
                (Status::Done, _) => (AlarmState::Done, None),
                (Status::Missed, _) => (AlarmState::Missed, None),
                (Status::Waiting, Some(until)) => (AlarmState::Snoozed, Some(until)),
                (Status::Waiting, None) => {
                    let from = rang
                        .rang
                        .as_deref()
                        .and_then(unstamp)
                        .map_or(now, |r| r.max(now));
                    (AlarmState::Waiting, l.once().or_else(|| l.next_after(from)))
                }
            };
            Alarm {
                key: l.key(),
                label: l.label.clone(),
                time: hm(l.time),
                repeat: l.repeat.clone(),
                state,
                next: next.map(stamp),
            }
        })
        .collect();
    alarms.sort_by_key(|a| {
        (
            matches!(a.state, AlarmState::Done | AlarmState::Missed),
            a.next.clone(),
        )
    });
    AlarmList {
        alarms,
        unreadable: unreadable(&rows),
        notifications_held: false,
    }
}

/// Checks an alarm the person or the companion gave; a `once` alarm without a date is the next such time that
/// hasn't passed (extensions FR-024).
fn checked(
    label: &str,
    time: &str,
    repeat: Repeat,
    now: PrimitiveDateTime,
) -> std::result::Result<Line, &'static str> {
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 200 || label.contains(['\n', '\r']) {
        return Err("label must be one line of 1 to 200 characters");
    }
    let time = clock(time).ok_or("time must be HH:MM, 24-hour")?;
    let repeat = match repeat {
        Repeat::Once { date: d } if d.is_empty() => {
            let today = PrimitiveDateTime::new(now.date(), time);
            let day = if today > now {
                now.date()
            } else {
                now.date() + Duration::days(1)
            };
            Repeat::Once { date: ymd(day) }
        }
        Repeat::Once { date: d } => {
            let day = date(&d).ok_or("date must be YYYY-MM-DD")?;
            if PrimitiveDateTime::new(day, time) <= now {
                return Err("past");
            }
            Repeat::Once { date: ymd(day) }
        }
        Repeat::Weekdays { mut days } => {
            days.sort_by_key(|d| *d as u8);
            days.dedup();
            if days.is_empty() {
                return Err("days must name at least one weekday");
            }
            Repeat::Weekdays { days }
        }
        other => other,
    };
    Ok(Line {
        state: Status::Waiting,
        repeat,
        time,
        label: label.into(),
    })
}

fn invalid(reason: &str) -> AppError {
    AppError::invalid(match reason {
        "past" => "date",
        r if r.starts_with("label") => "label",
        r if r.starts_with("time") => "time",
        r if r.starts_with("days") => "days",
        _ => "date",
    })
}

/// Adds an alarm (no key) or changes one (extensions FR-017).
pub fn save(
    dir: &ExtDir,
    key: Option<&str>,
    label: &str,
    time: &str,
    repeat: Repeat,
    now: PrimitiveDateTime,
) -> Result<AlarmList> {
    let line = checked(label, time, repeat, now).map_err(invalid)?;
    {
        let _guard = dir.lock();
        let mut rows = read(dir);
        match key {
            Some(key) => *find(&mut rows, key).ok_or_else(|| AppError::invalid("key"))? = line,
            None => add(&mut rows, line),
        }
        write(dir, &rows, now.date())?;
    }
    Ok(list(dir, now))
}

fn add(rows: &mut Vec<Row>, line: Line) {
    if rows.is_empty() {
        rows.extend([
            Row::Text(HEADER.trim_end().into()),
            Row::Text(String::new()),
        ]);
    }
    let end = rows
        .iter()
        .rposition(|r| matches!(r, Row::Alarm(_)))
        .map_or(rows.len(), |i| i + 1);
    rows.insert(end, Row::Alarm(line));
}

pub fn delete(dir: &ExtDir, key: &str, now: PrimitiveDateTime) -> Result<AlarmList> {
    {
        let _guard = dir.lock();
        let mut rows = read(dir);
        let before = rows.len();
        rows.retain(|r| !matches!(r, Row::Alarm(l) if l.key() == key));
        if rows.len() == before {
            return Err(AppError::invalid("key"));
        }
        write(dir, &rows, now.date())?;
    }
    Ok(list(dir, now))
}

/// Snooze rings again in 10 minutes; Done finishes a one-time alarm, and a repeating one waits for its next time
/// (extensions FR-021).
pub fn act(dir: &ExtDir, key: &str, action: Action, now: PrimitiveDateTime) -> Result<AlarmList> {
    {
        let _guard = dir.lock();
        let mut rows = read(dir);
        let line = find(&mut rows, key).ok_or_else(|| AppError::invalid("key"))?;
        let mut state: State = dir.read_state();
        let rang = state.alarms.entry(line.body()).or_default();
        rang.snoozed_until = (action == Action::Snooze).then(|| stamp(now + SNOOZE));
        if action == Action::Done && line.once().is_some() {
            line.state = Status::Done;
            write(dir, &rows, now.date())?;
        }
        dir.write_state(&state)?;
    }
    Ok(list(dir, now))
}

/// The alarms extension. Remembers the file's last size and time to tell when it changed (extensions R9).
#[derive(Default)]
pub struct Alarms {
    seen: Mutex<Option<(std::time::SystemTime, u64)>>,
    ticked: AtomicBool,
}

/// The file's modification time and size, if it exists.
fn signature(path: &std::path::Path) -> Option<(std::time::SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

impl Extension for Alarms {
    fn id(&self) -> &'static str {
        "alarms"
    }

    fn name(&self) -> &'static str {
        "Alarms"
    }

    fn about(&self) -> &'static str {
        "Reminders that ring with a Windows notification, once or repeating. The companion can set, list, move and \
         cancel them when you ask."
    }

    fn skill(&self) -> &'static str {
        include_str!("SKILL.md")
    }

    fn place(&self) -> &'static str {
        "alarms.md"
    }

    fn tools(&self) -> Vec<ToolSpec> {
        tools::specs()
    }

    fn run(&self, tool: &str, input: &Value, dir: &ExtDir, now: PrimitiveDateTime) -> ToolAnswer {
        tools::run(tool, input, dir, now)
    }

    fn tick(&self, now: OffsetDateTime, dir: &ExtDir) -> Tick {
        let first = !self.ticked.swap(true, Ordering::SeqCst);
        let mut result = {
            let _guard = dir.lock();
            let mut rows = read(dir);
            let mut state: State = dir.read_state();
            let (tick, rewrite) = ring::tick(&mut rows, &mut state, now, first);
            if rewrite {
                let _ = write(dir, &rows, now.date());
            }
            let _ = dir.write_state(&state);
            tick
        };
        let signature = signature(dir.place());
        let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
        result.changed = *seen != signature;
        *seen = signature;
        result
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use time::macros::datetime;

    pub(crate) fn at(text: &str) -> OffsetDateTime {
        let local = unstamp(text).unwrap();
        local.assume_offset(time::UtcOffset::from_hms(7, 0, 0).unwrap())
    }

    #[test]
    fn snooze_rings_again_after_ten_minutes_and_done_finishes_once_alarms() {
        let (_root, core) = crate::testing::core();
        let dir = core.ext_dir("alarms").unwrap();
        let now = datetime!(2026-11-14 18:00);
        let list = save(
            &dir,
            None,
            "Call Mum",
            "19:00",
            Repeat::Once {
                date: String::new(),
            },
            now,
        )
        .unwrap();
        let key = list.alarms[0].key.clone();
        let ext = Alarms::default();
        let rings = |t: &str| ext.tick(at(t), &dir).ring.len();
        rings("2026-11-14 18:59");
        assert_eq!(rings("2026-11-14 19:00"), 1);
        let list = act(&dir, &key, Action::Snooze, datetime!(2026-11-14 19:01)).unwrap();
        assert_eq!(
            (list.alarms[0].state, list.alarms[0].next.as_deref()),
            (AlarmState::Snoozed, Some("2026-11-14 19:11"))
        );
        assert_eq!(rings("2026-11-14 19:10"), 0);
        assert_eq!(rings("2026-11-14 19:11"), 1);
        let list = act(&dir, &key, Action::Done, datetime!(2026-11-14 19:12)).unwrap();
        assert_eq!(list.alarms[0].state, AlarmState::Done);
        assert!(
            std::fs::read_to_string(dir.place())
                .unwrap()
                .contains("- [x] 2026-11-14 19:00 Call Mum")
        );
        // A repeating alarm stays waiting and moves to its next time.
        let list = save(&dir, None, "Tea", "07:30", Repeat::Daily, now).unwrap();
        let tea = list
            .alarms
            .iter()
            .find(|a| a.label == "Tea")
            .unwrap()
            .key
            .clone();
        let list = act(&dir, &tea, Action::Done, now).unwrap();
        let tea = list.alarms.iter().find(|a| a.label == "Tea").unwrap();
        assert_eq!(
            (tea.state, tea.next.as_deref()),
            (AlarmState::Waiting, Some("2026-11-15 07:30"))
        );
    }

    #[test]
    fn writing_keeps_hand_lines_and_drops_old_finished_alarms() {
        let text = "# Alarms\n\nnot an alarm\n- [x] 2026-10-01 08:00 Old dentist\n- [!] 2026-11-13 21:00 Recent\n";
        let (_root, core) = crate::testing::core();
        let dir = core.ext_dir("alarms").unwrap();
        std::fs::create_dir_all(dir.place().parent().unwrap()).unwrap();
        std::fs::write(dir.place(), text).unwrap();
        let list = save(
            &dir,
            None,
            "Tea",
            "07:30",
            Repeat::Daily,
            datetime!(2026-11-14 12:00),
        )
        .unwrap();
        assert_eq!(list.unreadable, 1);
        assert_eq!(
            std::fs::read_to_string(dir.place()).unwrap(),
            "# Alarms\n\nnot an alarm\n- [!] 2026-11-13 21:00 Recent\n- [ ] every day 07:30 Tea\n"
        );
    }
}
