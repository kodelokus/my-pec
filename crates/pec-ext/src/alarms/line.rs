//! The line grammar of `alarms.md` and each line's next time (extensions data 1, 5 and 7).

use pec_core::ext::Due;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{Date, Duration, PrimitiveDateTime, Time};
use ts_rs::TS;

use super::Repeat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

const DAYS: [(Day, &str, &str); 7] = [
    (Day::Mon, "Mon", "Monday"),
    (Day::Tue, "Tue", "Tuesday"),
    (Day::Wed, "Wed", "Wednesday"),
    (Day::Thu, "Thu", "Thursday"),
    (Day::Fri, "Fri", "Friday"),
    (Day::Sat, "Sat", "Saturday"),
    (Day::Sun, "Sun", "Sunday"),
];

impl Day {
    pub(super) fn short(self) -> &'static str {
        DAYS[self as usize].1
    }

    pub(super) fn long(self) -> &'static str {
        DAYS[self as usize].2
    }

    pub(super) fn parse(text: &str) -> Option<Self> {
        DAYS.iter()
            .find(|(_, short, long)| {
                text.eq_ignore_ascii_case(short) || text.eq_ignore_ascii_case(long)
            })
            .map(|(day, ..)| *day)
    }

    pub(super) fn of(date: Date) -> Self {
        DAYS[date.weekday().number_days_from_monday() as usize].0
    }
}

/// The box: `[ ]` waiting, `[x]` done, `[!]` missed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Status {
    Waiting,
    Done,
    Missed,
}

/// One parsed line of `alarms.md`.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Line {
    pub(super) state: Status,
    pub(super) repeat: Repeat,
    pub(super) time: Time,
    pub(super) label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Row {
    Alarm(Line),
    /// A heading, an empty line or one that couldn't be read, kept as it is.
    Text(String),
}

pub(super) fn date(text: &str) -> Option<Date> {
    let number = |s: Option<&str>| s?.parse::<u8>().ok();
    let year = text.get(..4)?.parse().ok()?;
    let ok = text.len() == 10 && &text[4..5] == "-" && &text[7..8] == "-";
    let month = number(text.get(5..7))?.try_into().ok()?;
    Date::from_calendar_date(year, month, number(text.get(8..10))?)
        .ok()
        .filter(|_| ok)
}

pub(super) fn clock(text: &str) -> Option<Time> {
    let (h, m) = text.split_once(':')?;
    if h.len() != 2 || m.len() != 2 {
        return None;
    }
    Time::from_hms(h.parse().ok()?, m.parse().ok()?, 0).ok()
}

pub(super) fn ymd(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

pub(super) fn hm(t: Time) -> String {
    format!("{:02}:{:02}", t.hour(), t.minute())
}

pub(super) fn stamp(at: PrimitiveDateTime) -> String {
    format!("{} {}", ymd(at.date()), hm(at.time()))
}

pub(super) fn unstamp(text: &str) -> Option<PrimitiveDateTime> {
    let (d, t) = text.split_once(' ')?;
    Some(PrimitiveDateTime::new(date(d)?, clock(t)?))
}

/// `Saturday 15 November`.
pub(super) fn day_name(d: Date) -> String {
    format!("{} {} {}", Day::of(d).long(), d.day(), d.month())
}

/// `Monday, Wednesday and Friday`.
fn joined(days: &[Day]) -> String {
    let names: Vec<&str> = days.iter().map(|d| d.long()).collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => names.concat(),
    }
}

impl Line {
    pub(super) fn parse(text: &str) -> Option<Self> {
        let rest = text.trim_end().strip_prefix("- [")?;
        let state = match rest.get(..1)? {
            " " => Status::Waiting,
            "x" | "X" => Status::Done,
            "!" => Status::Missed,
            _ => return None,
        };
        let rest = rest.get(1..)?.strip_prefix("] ")?;
        let (repeat, rest) = if let Some(rest) = rest.strip_prefix("every day ") {
            (Repeat::Daily, rest)
        } else if let Some(rest) = rest.strip_prefix("every ") {
            let (day, rest) = rest.split_once(' ')?;
            (
                Repeat::Weekly {
                    day: Day::parse(day)?,
                },
                rest,
            )
        } else if let Some(d) = rest.get(..10).and_then(date) {
            (Repeat::Once { date: ymd(d) }, rest.get(11..)?)
        } else {
            let mut days = Vec::new();
            let mut rest = rest;
            while let Some((word, after)) = rest.split_once(' ')
                && let Some(day) = Day::parse(word)
            {
                if !days.contains(&day) {
                    days.push(day);
                }
                rest = after;
            }
            if days.is_empty() {
                return None;
            }
            days.sort_by_key(|d| *d as u8);
            (Repeat::Weekdays { days }, rest)
        };
        let (time, label) = rest.split_once(' ')?;
        let label = label.trim();
        if label.is_empty() || label.chars().count() > 200 {
            return None;
        }
        // Repeating alarms are always waiting (extensions data 1).
        let state = if matches!(repeat, Repeat::Once { .. }) {
            state
        } else {
            Status::Waiting
        };
        Some(Self {
            state,
            repeat,
            time: clock(time)?,
            label: label.into(),
        })
    }

    /// The line after its box: what bookkeeping and keys are tied to.
    pub(super) fn body(&self) -> String {
        let when = match &self.repeat {
            Repeat::Once { date } => date.clone(),
            Repeat::Daily => "every day".into(),
            Repeat::Weekdays { days } => {
                days.iter().map(|d| d.short()).collect::<Vec<_>>().join(" ")
            }
            Repeat::Weekly { day } => format!("every {}", day.short()),
        };
        format!("{when} {} {}", hm(self.time), self.label)
    }

    pub(super) fn render(&self) -> String {
        let mark = match self.state {
            Status::Waiting => ' ',
            Status::Done => 'x',
            Status::Missed => '!',
        };
        format!("- [{mark}] {}", self.body())
    }

    /// extensions data 7: the first 8 hex characters of the SHA-256 of the line after its box.
    pub(super) fn key(&self) -> String {
        let digest = Sha256::digest(self.body().as_bytes());
        digest[..4].iter().map(|b| format!("{b:02x}")).collect()
    }

    pub(super) fn once(&self) -> Option<PrimitiveDateTime> {
        match &self.repeat {
            Repeat::Once { date: d } => Some(PrimitiveDateTime::new(date(d)?, self.time)),
            _ => None,
        }
    }

    pub(super) fn rings_on(&self, d: Date) -> bool {
        match &self.repeat {
            Repeat::Once { date: once } => date(once) == Some(d),
            Repeat::Daily => true,
            Repeat::Weekdays { days } => days.contains(&Day::of(d)),
            Repeat::Weekly { day } => Day::of(d) == *day,
        }
    }

    /// The first time after `after` that matches its rule (extensions data 5).
    pub(super) fn next_after(&self, after: PrimitiveDateTime) -> Option<PrimitiveDateTime> {
        if let Some(at) = self.once() {
            return (at > after).then_some(at);
        }
        (0..=7)
            .map(|i| PrimitiveDateTime::new(after.date() + Duration::days(i), self.time))
            .find(|at| *at > after && self.rings_on(at.date()))
    }

    /// `Saturday 15 November at 19:00`, or `every day at 07:30`.
    pub(super) fn phrase(&self) -> String {
        let when = match &self.repeat {
            Repeat::Once { date: d } => {
                return format!(
                    "{} at {}",
                    date(d).map(day_name).unwrap_or_default(),
                    hm(self.time)
                );
            }
            Repeat::Daily => "every day".into(),
            Repeat::Weekdays { days } => format!("every {}", joined(days)),
            Repeat::Weekly { day } => format!("every {}", day.long()),
        };
        format!("{when} at {}", hm(self.time))
    }

    pub(super) fn due(&self, at: PrimitiveDateTime) -> Due {
        Due {
            key: self.key(),
            label: self.label.clone(),
            date: ymd(at.date()),
            time: hm(self.time),
        }
    }
}

pub(super) fn rows(text: &str) -> Vec<Row> {
    text.lines()
        .map(|l| Line::parse(l).map_or_else(|| Row::Text(l.into()), Row::Alarm))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alarms::unreadable;
    use time::macros::datetime;

    fn line(text: &str) -> Line {
        Line::parse(text).unwrap_or_else(|| panic!("{text}"))
    }

    #[test]
    fn every_kind_of_line_reads_and_writes_back_the_same() {
        for text in [
            "- [ ] 2026-11-15 19:00 Call Mum",
            "- [ ] every day 07:30 Take the vitamins",
            "- [ ] Mon Wed Fri 06:45 Run",
            "- [ ] every Sat 10:00 Laundry",
            "- [x] 2026-11-14 08:00 Dentist",
            "- [!] 2026-11-13 21:00 Water the plants",
        ] {
            assert_eq!(line(text).render(), text);
        }
        assert_eq!(
            line("- [ ] every Saturday 10:00 Laundry").render(),
            "- [ ] every Sat 10:00 Laundry"
        );
        assert_eq!(
            line("- [ ] Fri Mon Fri 06:45 Run").render(),
            "- [ ] Mon Fri 06:45 Run"
        );
        // Repeating alarms are always waiting.
        assert_eq!(line("- [x] every day 07:30 Tea").state, Status::Waiting);
        for bad in [
            "- [ ] 2026-02-30 19:00 No such day",
            "- [ ] 2026-11-15 25:00 No such hour",
            "- [ ] 2026-11-15 7:00 Short hour",
            "- [ ] 2026-11-15 19:00",
            "- [?] 2026-11-15 19:00 Odd box",
            "- [ ] sometimes 19:00 Vague",
            "Call Mum at 7",
            &format!("- [ ] every day 07:30 {}", "a".repeat(201)),
        ] {
            assert_eq!(Line::parse(bad), None, "{bad}");
        }
        let text = "# Alarms\n\nA note to myself\n- [ ] every day 07:30 Tea\n";
        assert_eq!(unreadable(&rows(text)), 1);
    }

    #[test]
    fn the_next_time_follows_each_rule() {
        let after = datetime!(2026-11-14 19:00); // a Saturday
        for (text, next) in [
            (
                "- [ ] 2026-11-15 07:30 Once",
                Some(datetime!(2026-11-15 07:30)),
            ),
            ("- [ ] 2026-11-14 18:00 Once, passed", None),
            (
                "- [ ] every day 07:30 Daily",
                Some(datetime!(2026-11-15 07:30)),
            ),
            (
                "- [ ] every day 19:30 Daily, later today",
                Some(datetime!(2026-11-14 19:30)),
            ),
            (
                "- [ ] Mon Wed Fri 06:45 Weekdays",
                Some(datetime!(2026-11-16 06:45)),
            ),
            (
                "- [ ] every Sat 19:00 Weekly, now",
                Some(datetime!(2026-11-21 19:00)),
            ),
        ] {
            assert_eq!(line(text).next_after(after), next, "{text}");
        }
    }
}
