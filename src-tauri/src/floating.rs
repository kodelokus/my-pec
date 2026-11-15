//! The floating character: a small frameless window above the others (avatar R8).

use crate::{AppCtx, tray};
use pec_core::AppError;
use pec_core::avatar::{FloatingSettings, Screen, place};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const LABEL: &str = "floating";
const SIZE: (u32, u32) = (200, 260);
/// The position is saved this long after the last move.
const SETTLE: Duration = Duration::from_millis(500);

fn failed(e: tauri::Error) -> AppError {
    AppError::Internal {
        message: e.to_string(),
    }
}

/// avatar FR-023: saves the setting, opens or closes the window and keeps the tray item in step.
pub fn set(app: &AppHandle, on: bool) -> Result<FloatingSettings, AppError> {
    let settings = app.state::<AppCtx>().core.set_floating(on)?;
    if on {
        open(app).map_err(failed)?;
    } else if let Some(window) = app.get_webview_window(LABEL) {
        window.destroy().map_err(failed)?;
    }
    tray::floating_checked(app, on);
    Ok(settings)
}

/// avatar FR-025: placed by the core's rule against the current screens; its place is saved once it stops moving.
pub fn open(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(LABEL).is_some() {
        return Ok(());
    }
    let core = &app.state::<AppCtx>().core;
    // The window may not ask for the companion's status, so the page is told it floats, and the name for its label,
    // before it loads.
    let name = core
        .app_status()
        .ok()
        .and_then(|s| s.companion)
        .map_or_else(String::new, |c| c.name);
    let script = format!(
        "window.floatingName = {};",
        serde_json::to_string(&name).unwrap_or_default()
    );
    let url = WebviewUrl::App("index.html".into());
    let window = WebviewWindowBuilder::new(app, LABEL, url)
        .initialization_script(&script)
        .title("My Pec")
        .inner_size(SIZE.0.into(), SIZE.1.into())
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .visible(false)
        .build()?;
    let (x, y) = place(
        core.floating_position().ok().flatten(),
        SIZE,
        &screens(&window)?,
    );
    window.set_position(PhysicalPosition::new(x, y))?;
    window.show()?;
    let moves = Arc::new(AtomicU64::new(0));
    let handle = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Moved(at) = event {
            let this = moves.fetch_add(1, Ordering::SeqCst) + 1;
            let (moves, handle, at) = (moves.clone(), handle.clone(), *at);
            std::thread::spawn(move || {
                std::thread::sleep(SETTLE);
                if moves.load(Ordering::SeqCst) == this {
                    let core = &handle.state::<AppCtx>().core;
                    let _ = core.save_floating_position(at.x, at.y);
                }
            });
        }
    });
    Ok(())
}

/// The screens' work areas, for the core's `place` rule (shared with note windows).
pub(crate) fn screens(window: &tauri::WebviewWindow) -> tauri::Result<Vec<Screen>> {
    let primary = window.primary_monitor()?.map(|m| *m.position());
    Ok(window
        .available_monitors()?
        .iter()
        .map(|m| {
            let area = m.work_area();
            Screen {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
                scale: m.scale_factor(),
                primary: primary == Some(*m.position()),
            }
        })
        .collect())
}
