use crate::AppCtx;
use crate::show_main;
use crate::words::words;
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

/// The menu's items, relabelled when the language changes; "Floating <name>" kept in step with the setting
/// (avatar FR-023).
struct Items {
    open: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    floating: CheckMenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

fn floating_label(app: &AppHandle) -> String {
    let name = app
        .state::<AppCtx>()
        .core
        .app_status()
        .ok()
        .and_then(|s| s.companion)
        .map_or_else(|| pec_core::onboarding::DEFAULT_NAME.into(), |c| c.name);
    format!("{} {name}", words(app).floating)
}

/// onboarding FR-025, FR-027: Open, Settings, Floating and Quit. A left click opens the window.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let on = app
        .state::<AppCtx>()
        .core
        .floating_settings()
        .is_ok_and(|f| f.on);
    let w = words(app);
    let items = Items {
        open: MenuItem::with_id(app, "open", w.open, true, None::<&str>)?,
        settings: MenuItem::with_id(app, "settings", w.settings, true, None::<&str>)?,
        floating: CheckMenuItem::with_id(
            app,
            "floating",
            floating_label(app),
            true,
            on,
            None::<&str>,
        )?,
        quit: MenuItem::with_id(app, "quit", w.quit, true, None::<&str>)?,
    };
    let menu = Menu::with_items(
        app,
        &[&items.open, &items.settings, &items.floating, &items.quit],
    )?;
    app.manage(items);
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("My Pec")
        .menu(&menu)
        .show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.on_menu_event(|app, event| match event.id.as_ref() {
        "open" => show_main(app),
        "settings" => {
            show_main(app);
            let _ = app.emit_to("main", "open_settings", ());
        }
        "floating" => {
            let on = app
                .state::<AppCtx>()
                .core
                .floating_settings()
                .is_ok_and(|f| f.on);
            let _ = crate::floating::set(app, !on);
            // Settings, if open, shows the new value.
            let _ = app.emit_to("main", "floating_changed", ());
        }
        "quit" => {
            // A running reply is stopped and saved before the app ends.
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                app.state::<AppCtx>().core.shutdown().await;
                app.exit(0);
            });
        }
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            show_main(tray.app_handle());
        }
    })
    .build(app)?;
    Ok(())
}

pub fn floating_checked(app: &AppHandle, on: bool) {
    if let Some(items) = app.try_state::<Items>() {
        let _ = items.floating.set_checked(on);
    }
}

/// The menu in the interface language, after the person changes it.
pub fn relabel(app: &AppHandle) {
    let Some(items) = app.try_state::<Items>() else {
        return;
    };
    let w = words(app);
    let _ = items.open.set_text(w.open);
    let _ = items.settings.set_text(w.settings);
    let _ = items.floating.set_text(floating_label(app));
    let _ = items.quit.set_text(w.quit);
}
