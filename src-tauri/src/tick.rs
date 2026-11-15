//! The extension tick: every 15 seconds from startup to quit, whatever the windows do, the core says what is due
//! and the shell rings it as a Windows notification (extensions R4, R5; extensions interfaces).

use crate::words::words;
use crate::{AppCtx, notes, show_main};
use pec_core::ext::Due;
use pec_ext::alarms::{self, Action};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_winrt_notification::{Scenario, Toast, ToastDismissalReason};
use time::{OffsetDateTime, PrimitiveDateTime};

const EVERY: Duration = Duration::from_secs(15);
/// The most labels a notification lists.
const LISTED: usize = 5;

/// Windows didn't show the last notification, or has My Pec's notifications turned off (extensions edge cases).
static HELD: AtomicBool = AtomicBool::new(false);

/// The alarm list says so before the next alarm, not only after one failed to show.
pub fn held() -> bool {
    HELD.load(Ordering::SeqCst) || turned_off()
}

pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
}

pub fn local_now() -> PrimitiveDateTime {
    let now = now();
    PrimitiveDateTime::new(now.date(), now.time())
}

pub fn start(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        loop {
            let tick = app.state::<AppCtx>().core.ext_tick(now());
            if !tick.ring.is_empty() {
                ring(&app, tick.ring.clone());
            }
            if !tick.missed.is_empty() {
                missed(&app, &tick.missed);
            }
            let mut changed = tick.changed;
            if !(tick.ring.is_empty() && tick.missed.is_empty())
                && !changed.iter().any(|c| c == "alarms")
            {
                changed.push("alarms".into());
            }
            if changed.iter().any(|c| c == "notes") {
                notes::sync(&app);
            }
            if !changed.is_empty() {
                let _ = app.emit("ext_changed", changed);
            }
            std::thread::sleep(EVERY);
        }
    });
}

/// Installed builds have the bundle's AppUserModelID; a build run from `target/` borrows PowerShell's (extensions R5).
fn app_id() -> &'static str {
    let from_target = std::env::current_exe()
        .is_ok_and(|exe| exe.components().any(|c| c.as_os_str() == "target"));
    if from_target {
        Toast::POWERSHELL_APP_ID
    } else {
        "com.kodelokus.mypec"
    }
}

/// `Call Mum at 19:00`, up to five, then `and N more`.
fn listed(app: &AppHandle, due: &[Due], with_date: bool) -> String {
    let w = words(app);
    let mut lines: Vec<String> = due
        .iter()
        .take(LISTED)
        .map(|d| match with_date {
            true => format!("{} ({} {})", d.label, d.date, d.time),
            false => format!("{} {} {}", d.label, w.at, d.time),
        })
        .collect();
    if due.len() > LISTED {
        lines.push(format!("{} {} {}", w.and, due.len() - LISTED, w.more));
    }
    lines.join("\n")
}

fn companion(app: &AppHandle) -> String {
    let core = &app.state::<AppCtx>().core;
    core.app_status()
        .ok()
        .and_then(|s| s.companion)
        .map_or_else(|| pec_core::onboarding::DEFAULT_NAME.into(), |c| c.name)
}

/// Snooze or Done on every alarm of a notification, as the list's buttons do (extensions FR-021).
pub fn act(app: &AppHandle, keys: &[String], action: Action) {
    let core = &app.state::<AppCtx>().core;
    if let Ok(dir) = core.ext_dir("alarms") {
        for key in keys {
            let _ = alarms::act(&dir, key, action, local_now());
        }
    }
    let _ = app.emit("ext_changed", ["alarms"]);
}

/// extensions FR-022: a click on a notification opens the alarm list.
fn open_alarms(app: &AppHandle) {
    show_main(app);
    let _ = app.emit("open_alarms", ());
}

/// One notification for everything due in this tick, with Snooze and Done (extensions FR-020, FR-021).
fn ring(app: &AppHandle, due: Vec<Due>) {
    let (title, text) = match due.as_slice() {
        [one] => (
            one.label.clone(),
            format!("{} · {}", companion(app), one.time),
        ),
        _ => (
            format!("{} {}", due.len(), words(app).reminders),
            listed(app, &due, false),
        ),
    };
    let keys: Vec<String> = due.into_iter().map(|d| d.key).collect();
    let (on_click, on_close, then) = (app.clone(), app.clone(), keys.clone());
    let shown = Toast::new(app_id())
        .title(&title)
        .text1(&text)
        .scenario(Scenario::Alarm)
        .add_button(words(app).snooze, "snooze")
        .add_button(words(app).done, "done")
        .on_activated(move |action| {
            match action.as_deref() {
                Some("snooze") => act(&on_click, &keys, Action::Snooze),
                Some("done") => act(&on_click, &keys, Action::Done),
                _ => open_alarms(&on_click),
            }
            Ok(())
        })
        // The person dismissing it is Done; Windows hiding it or timing it out changes nothing.
        .on_dismissed(move |reason| {
            if matches!(reason, Some(ToastDismissalReason::UserCanceled)) {
                act(&on_close, &then, Action::Done);
            }
            Ok(())
        })
        .show();
    HELD.store(shown.is_err() || turned_off(), Ordering::SeqCst);
}

/// Whether Windows says My Pec's notifications are turned off: by the person, for every app, or by a policy.
/// Focus (do not disturb) can't be read, so it isn't counted.
fn turned_off() -> bool {
    use windows::UI::Notifications::{NotificationSetting, ToastNotificationManager};
    ToastNotificationManager::CreateToastNotifierWithId(&windows::core::HSTRING::from(app_id()))
        .and_then(|n| n.Setting())
        .is_ok_and(|s| s != NotificationSetting::Enabled)
}

/// extensions FR-023: one notification listing what was missed; never rung late.
fn missed(app: &AppHandle, due: &[Due]) {
    let handle = app.clone();
    let shown = Toast::new(app_id())
        .title(words(app).missed)
        .text1(&listed(app, due, true))
        .on_activated(move |_| {
            open_alarms(&handle);
            Ok(())
        })
        .show();
    HELD.store(shown.is_err() || turned_off(), Ordering::SeqCst);
}
