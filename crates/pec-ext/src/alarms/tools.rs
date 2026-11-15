//! The companion's alarm tools (extensions interfaces).

use pec_core::ext::{ExtDir, ToolAnswer};
use pec_core::llm::ToolSpec;
use pec_core::memory::{MarkKind, MemoryMark};
use serde_json::{Value, json};
use time::PrimitiveDateTime;

use super::line::{Day, Line, Row, day_name, hm, unstamp};
use super::{AlarmState, Repeat, add, checked, find, list, read, write};

const STALE: &str = "That alarm has changed. List the alarms again.";

/// The tools of extensions interfaces, with strict schemas.
pub(super) fn specs() -> Vec<ToolSpec> {
    let days = json!({ "type": "array", "items": { "type": "string", "enum": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] },
        "description": "The weekdays, for weekdays (one or more) and weekly (exactly one)." });
    let repeat = json!({ "type": "string", "enum": ["once", "daily", "weekdays", "weekly"] });
    let time =
        json!({ "type": "string", "description": "HH:MM, 24-hour, the person's local time." });
    let date = json!({ "type": "string", "description": "YYYY-MM-DD, for once only. Leave it out for the next such time that hasn't passed." });
    let label = json!({ "type": "string", "description": "What to remind the person of, in their words, 1 to 200 characters." });
    let alarm = json!({ "type": "string", "description": "An alarm's key, from alarm_list." });
    let object = |properties: Value, required: &[&str]| json!({ "type": "object", "properties": properties, "required": required, "additionalProperties": false });
    vec![
        ToolSpec {
            name: "alarm_set",
            description: "Set an alarm that rings with a Windows notification at the person's local time. Only when \
                          the person asks, or agrees to your suggestion. Ask for a time if they gave none.",
            input_schema: object(
                json!({ "label": label, "time": time, "repeat": repeat, "date": date, "days": days }),
                &["label", "time", "repeat"],
            ),
        },
        ToolSpec {
            name: "alarm_list",
            description: "List the alarms still to ring, soonest first, with their keys.",
            input_schema: object(json!({}), &[]),
        },
        ToolSpec {
            name: "alarm_move",
            description: "Change an alarm's time, day, repeat or label. Give only what changes.",
            input_schema: object(
                json!({ "alarm": alarm, "label": label, "time": time, "repeat": repeat, "date": date, "days": days }),
                &["alarm"],
            ),
        },
        ToolSpec {
            name: "alarm_cancel",
            description: "Cancel an alarm.",
            input_schema: object(json!({ "alarm": alarm }), &["alarm"]),
        },
    ]
}

/// A `repeat` from a tool's input.
fn repeat_of(input: &Value, current: Option<&Repeat>) -> std::result::Result<Repeat, &'static str> {
    let days = || -> std::result::Result<Vec<Day>, &'static str> {
        input["days"]
            .as_array()
            .ok_or("days must name the weekdays")?
            .iter()
            .map(|d| {
                d.as_str()
                    .and_then(Day::parse)
                    .ok_or("days must be weekday names")
            })
            .collect()
    };
    let date = || input["date"].as_str().unwrap_or_default().to_owned();
    Ok(match (input["repeat"].as_str(), current) {
        (Some("once"), _) => Repeat::Once { date: date() },
        (Some("daily"), _) => Repeat::Daily,
        (Some("weekdays"), _) => Repeat::Weekdays { days: days()? },
        (Some("weekly"), _) => Repeat::Weekly {
            day: *days()?.first().ok_or("days must name one weekday")?,
        },
        (Some(_), _) => return Err("repeat must be once, daily, weekdays or weekly"),
        (None, Some(Repeat::Once { .. })) if input.get("date").is_some() => {
            Repeat::Once { date: date() }
        }
        (None, Some(Repeat::Once { date })) => Repeat::Once { date: date.clone() },
        (None, Some(current)) => current.clone(),
        (None, None) => return Err("repeat is required"),
    })
}

fn mark(kind: MarkKind, line: &Line) -> Option<MemoryMark> {
    Some(MemoryMark {
        kind,
        path: "alarms.md".into(),
        line: None,
        excerpt: format!("{} {}", hm(line.time), line.label),
        ok: true,
    })
}

fn refused(reason: &str) -> ToolAnswer {
    ToolAnswer::error(if reason == "past" {
        "That time has already passed. Ask which day they mean.".into()
    } else {
        format!("The input was not valid: {reason}.")
    })
}

pub(super) fn run(tool: &str, input: &Value, dir: &ExtDir, now: PrimitiveDateTime) -> ToolAnswer {
    let text = |field: &str| input[field].as_str().unwrap_or_default();
    match tool {
        "alarm_list" => {
            let waiting: Vec<String> = list(dir, now)
                .alarms
                .into_iter()
                .filter(|a| matches!(a.state, AlarmState::Waiting | AlarmState::Snoozed))
                .map(|a| {
                    let next = a
                        .next
                        .as_deref()
                        .and_then(unstamp)
                        .map(|n| format!("{} at {}", day_name(n.date()), hm(n.time())));
                    format!("- {}: {} — {}", a.key, next.unwrap_or_default(), a.label)
                })
                .collect();
            if waiting.is_empty() {
                ToolAnswer::ok("No alarms set.", None)
            } else {
                ToolAnswer::ok(waiting.join("\n"), None)
            }
        }
        "alarm_set" => {
            let line = match repeat_of(input, None)
                .and_then(|r| checked(text("label"), text("time"), r, now))
            {
                Ok(line) => line,
                Err(reason) => return refused(reason),
            };
            let saved = {
                let _guard = dir.lock();
                let mut rows = read(dir);
                add(&mut rows, line.clone());
                write(dir, &rows, now.date())
            };
            match saved {
                Ok(()) => {
                    let when = if line.once().is_some() {
                        format!("for {}", line.phrase())
                    } else {
                        line.phrase()
                    };
                    ToolAnswer::ok(
                        format!("Set \"{}\" {when}.", line.label),
                        mark(MarkKind::AlarmSet, &line),
                    )
                }
                Err(_) => ToolAnswer::error("The alarms file can't be written right now."),
            }
        }
        "alarm_move" | "alarm_cancel" => {
            let _guard = dir.lock();
            let mut rows = read(dir);
            let Some(current) = find(&mut rows, text("alarm")).cloned() else {
                return ToolAnswer::error(STALE);
            };
            let (answer, line) = if tool == "alarm_cancel" {
                rows.retain(|r| !matches!(r, Row::Alarm(l) if l.key() == current.key()));
                (format!("Cancelled \"{}\".", current.label), current)
            } else {
                let label = input["label"].as_str().unwrap_or(&current.label).to_owned();
                let time = input["time"]
                    .as_str()
                    .map_or_else(|| hm(current.time), Into::into);
                let line = match repeat_of(input, Some(&current.repeat))
                    .and_then(|r| checked(&label, &time, r, now))
                {
                    Ok(line) => line,
                    Err(reason) => return refused(reason),
                };
                *find(&mut rows, &current.key()).expect("found above") = line.clone();
                (
                    format!("Moved \"{}\" to {}.", line.label, line.phrase()),
                    line,
                )
            };
            let kind = if tool == "alarm_cancel" {
                MarkKind::AlarmCancelled
            } else {
                MarkKind::AlarmChanged
            };
            match write(dir, &rows, now.date()) {
                Ok(()) => ToolAnswer::ok(answer, mark(kind, &line)),
                Err(_) => ToolAnswer::error("The alarms file can't be written right now."),
            }
        }
        _ => ToolAnswer::error(format!("There is no tool named {tool}.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alarms::Alarms;
    use pec_core::ext::Extension;
    use time::macros::datetime;

    #[test]
    fn the_companion_sets_lists_moves_and_cancels_alarms() {
        let (_root, core) = crate::testing::core();
        let dir = core.ext_dir("alarms").unwrap();
        let ext = Alarms::default();
        let now = datetime!(2026-11-14 18:00); // a Saturday
        let call = |input: Value| ext.run(input["tool"].as_str().unwrap(), &input, &dir, now);
        let said = |input: Value| {
            let answer = call(input);
            (
                answer.content,
                answer.is_error,
                answer.mark.map(|m| (m.kind, m.excerpt)),
            )
        };
        assert_eq!(said(json!({ "tool": "alarm_list" })).0, "No alarms set.");
        // extensions FR-024: no date is the next such time that hasn't passed, and the result names the day.
        assert_eq!(
            said(
                json!({ "tool": "alarm_set", "label": "Call Mum", "time": "19:00", "repeat": "once" })
            ),
            (
                "Set \"Call Mum\" for Saturday 14 November at 19:00.".into(),
                false,
                Some((MarkKind::AlarmSet, "19:00 Call Mum".into()))
            )
        );
        assert_eq!(
            said(json!({ "tool": "alarm_set", "label": "Dentist", "time": "08:00", "repeat": "once" })).0,
            "Set \"Dentist\" for Sunday 15 November at 08:00."
        );
        for (input, answer) in [
            (
                json!({ "label": "Tea", "time": "07:30", "repeat": "daily" }),
                "Set \"Tea\" every day at 07:30.",
            ),
            (
                json!({ "label": "Run", "time": "06:45", "repeat": "weekdays", "days": ["Fri", "Mon", "Wed"] }),
                "Set \"Run\" every Monday, Wednesday and Friday at 06:45.",
            ),
            (
                json!({ "label": "Laundry", "time": "10:00", "repeat": "weekly", "days": ["Sat"] }),
                "Set \"Laundry\" every Saturday at 10:00.",
            ),
        ] {
            let mut input = input;
            input["tool"] = "alarm_set".into();
            assert_eq!(said(input).0, answer);
        }
        for (input, answer) in [
            (
                json!({ "label": "Late", "time": "17:00", "repeat": "once", "date": "2026-11-14" }),
                "That time has already passed. Ask which day they mean.",
            ),
            (
                json!({ "label": "Odd", "time": "7pm", "repeat": "daily" }),
                "The input was not valid: time must be HH:MM, 24-hour.",
            ),
            (
                json!({ "label": "", "time": "07:00", "repeat": "daily" }),
                "The input was not valid: label must be one line of 1 to 200 characters.",
            ),
        ] {
            let mut input = input;
            input["tool"] = "alarm_set".into();
            assert_eq!(said(input), (answer.into(), true, None));
        }

        // Soonest first, with keys.
        let listed = said(json!({ "tool": "alarm_list" })).0;
        let lines: Vec<&str> = listed.lines().collect();
        assert_eq!(lines.len(), 5);
        assert!(
            lines[0].ends_with(": Saturday 14 November at 19:00 — Call Mum"),
            "{listed}"
        );
        assert!(
            lines[1].ends_with(": Sunday 15 November at 07:30 — Tea"),
            "{listed}"
        );
        assert!(
            lines[4].ends_with(": Saturday 21 November at 10:00 — Laundry"),
            "{listed}"
        );
        let key =
            |label: &str| lines.iter().find(|l| l.ends_with(label)).unwrap()[2..10].to_owned();

        assert_eq!(
            said(json!({ "tool": "alarm_move", "alarm": key("Tea"), "time": "08:00" })),
            (
                "Moved \"Tea\" to every day at 08:00.".into(),
                false,
                Some((MarkKind::AlarmChanged, "08:00 Tea".into()))
            )
        );
        // U1: the old key no longer matches a line.
        assert_eq!(
            said(json!({ "tool": "alarm_cancel", "alarm": key("Tea") })),
            (STALE.into(), true, None)
        );
        assert_eq!(
            said(json!({ "tool": "alarm_cancel", "alarm": key("Laundry") })),
            (
                "Cancelled \"Laundry\".".into(),
                false,
                Some((MarkKind::AlarmCancelled, "10:00 Laundry".into()))
            )
        );
        let file = std::fs::read_to_string(dir.place()).unwrap();
        assert!(
            file.starts_with("# Alarms\n\n- [ ] 2026-11-14 19:00 Call Mum\n"),
            "{file}"
        );
        assert!(
            file.contains("- [ ] every day 08:00 Tea\n") && !file.contains("Laundry"),
            "{file}"
        );
    }
}
