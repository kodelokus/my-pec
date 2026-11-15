//! The shell's own few words (tray menu, alarm notifications, note window title) in the interface language the
//! person chose; everything else is in the webview's strings (NFR-I18N-1).

use crate::AppCtx;
use tauri::{AppHandle, Manager};

pub struct Words {
    pub open: &'static str,
    pub settings: &'static str,
    /// Before the companion's name: "Floating Pec".
    pub floating: &'static str,
    pub quit: &'static str,
    pub snooze: &'static str,
    pub done: &'static str,
    pub missed: &'static str,
    /// After the number of reminders due together: "3 reminders".
    pub reminders: &'static str,
    /// Between a reminder and its time: "Stretch at 07:30".
    pub at: &'static str,
    /// Before the number not listed: "and 2 more".
    pub and: &'static str,
    pub more: &'static str,
    pub note: &'static str,
}

const EN: Words = Words {
    open: "Open My Pec",
    settings: "Settings",
    floating: "Floating",
    quit: "Quit",
    snooze: "Snooze",
    done: "Done",
    missed: "Missed reminders",
    reminders: "reminders",
    at: "at",
    and: "and",
    more: "more",
    note: "Note",
};

const ID: Words = Words {
    open: "Buka My Pec",
    settings: "Pengaturan",
    floating: "Mengambang:",
    quit: "Keluar",
    snooze: "Tunda",
    done: "Selesai",
    missed: "Pengingat terlewat",
    reminders: "pengingat",
    at: "jam",
    and: "dan",
    more: "lagi",
    note: "Catatan",
};

pub fn words(app: &AppHandle) -> &'static Words {
    match app.state::<AppCtx>().core.language().as_str() {
        "id" => &ID,
        _ => &EN,
    }
}
