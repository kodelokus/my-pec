//! Sticky notes on the desktop: one small frameless window per shown note, outside the taskbar (extensions R7). Built
//! like the floating character's window.

use crate::AppCtx;
use crate::floating::screens;
use pec_core::AppError;
use pec_core::avatar::place;
use pec_ext::notes::{self, Place};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};

/// Logical pixels (extensions data 6).
const SIZE: (u32, u32) = (260, 260);
const MIN: (u32, u32) = (160, 120);
/// The place is saved this long after the last move or resize.
const SETTLE: Duration = Duration::from_millis(500);

/// `note-<file without .md>`: a note is shown in at most one window.
pub fn label(file: &str) -> String {
    format!("note-{}", file.trim_end_matches(".md"))
}

/// The note a window shows, by its label; `None` for every other window.
pub fn file_of(window: &WebviewWindow) -> Option<String> {
    window
        .label()
        .strip_prefix("note-")
        .map(|stem| format!("{stem}.md"))
}

/// A note window may act only on its own note; the main window on any.
pub fn own(window: &WebviewWindow, file: &str) -> Result<(), AppError> {
    match file_of(window) {
        Some(mine) if mine != file => Err(AppError::invalid("file")),
        _ => Ok(()),
    }
}

/// Opens a note's window, or brings it to the front if it is open.
pub fn open(app: &AppHandle, file: &str, on_top: bool) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(&label(file)) {
        return window.set_focus();
    }
    let core = &app.state::<AppCtx>().core;
    let Ok(dir) = core.ext_dir("notes") else {
        return Ok(());
    };
    let saved = notes::place(&dir, file);
    let script = format!(
        "window.noteFile = {};",
        serde_json::to_string(file).unwrap_or_default()
    );
    let window = WebviewWindowBuilder::new(app, label(file), WebviewUrl::App("index.html".into()))
        .initialization_script(&script)
        .title(crate::words::words(app).note)
        .inner_size(SIZE.0.into(), SIZE.1.into())
        .min_inner_size(MIN.0.into(), MIN.1.into())
        .decorations(false)
        .always_on_top(on_top)
        .skip_taskbar(true)
        .resizable(true)
        .visible(false)
        .build()?;
    if let Some(p) = saved {
        window.set_size(PhysicalSize::new(p.width, p.height))?;
    }
    // extensions FR-013: a note off every screen comes back onto the main one.
    let (x, y) = place(saved.map(|p| (p.x, p.y)), SIZE, &screens(&window)?);
    window.set_position(PhysicalPosition::new(x, y))?;
    window.show()?;
    let changes = Arc::new(AtomicU64::new(0));
    let (handle, file) = (app.clone(), file.to_owned());
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
            let this = changes.fetch_add(1, Ordering::SeqCst) + 1;
            let (changes, handle, file) = (changes.clone(), handle.clone(), file.clone());
            std::thread::spawn(move || {
                std::thread::sleep(SETTLE);
                if changes.load(Ordering::SeqCst) != this {
                    return;
                }
                let Some(window) = handle.get_webview_window(&label(&file)) else {
                    return;
                };
                if let (Ok(at), Ok(size), Ok(dir)) = (
                    window.outer_position(),
                    window.inner_size(),
                    handle.state::<AppCtx>().core.ext_dir("notes"),
                ) {
                    let place = Place {
                        x: at.x,
                        y: at.y,
                        width: size.width,
                        height: size.height,
                    };
                    let _ = notes::set_place(&dir, &file, place);
                }
            });
        }
    });
    Ok(())
}

pub fn close(app: &AppHandle, file: &str) {
    if let Some(window) = app.get_webview_window(&label(file)) {
        let _ = window.destroy();
    }
}

/// Puts the desktop in step with the notes: shown ones open, the rest closed, all closed while notes are off
/// (extensions FR-004, FR-013).
pub fn sync(app: &AppHandle) {
    let core = &app.state::<AppCtx>().core;
    let shown = match core.ext_dir("notes") {
        Ok(dir) if core.extension_on("notes") => notes::shown(&dir),
        _ => Vec::new(),
    };
    for window in app.webview_windows().into_values() {
        if let Some(file) = file_of(&window)
            && !shown.iter().any(|(f, _)| *f == file)
        {
            let _ = window.destroy();
        }
    }
    for (file, on_top) in shown {
        if app.get_webview_window(&label(&file)).is_none() {
            let _ = open(app, &file, on_top);
        }
    }
}
