mod commands;
mod floating;
mod notes;
pub mod secret_store;
mod tick;
mod tray;
mod words;

use pec_core::Core;
use pec_core::llm::claude::{API_URL, Claude, TIMEOUT};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

/// The argument Windows starts My Pec with at sign-in.
const AUTOSTART: &str = "--autostart";

pub(crate) struct AppCtx {
    core: Core,
    claude: Claude,
}

pub(crate) fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// The data root, the credential's service name and the provider address.
/// Only `e2e` builds can redirect them, to a scratch root and the local replay server (onboarding R12).
fn locations(data_dir: PathBuf) -> (PathBuf, &'static str, String) {
    #[cfg(feature = "e2e")]
    if let Ok(root) = std::env::var("MYPEC_E2E_ROOT") {
        let api = std::env::var("MYPEC_E2E_API_URL").unwrap_or_else(|_| API_URL.into());
        return (root.into(), "MyPec-e2e", api);
    }
    (
        data_dir.join("My Pec"),
        secret_store::SERVICE,
        API_URL.into(),
    )
}

/// In an `e2e` run, extensions' requests go to the replay server as `<api>/<host><path>` (weather R1).
#[cfg(feature = "e2e")]
struct Replayed(String, std::sync::OnceLock<pec_core::ext::Http>);

#[cfg(feature = "e2e")]
impl pec_core::ext::Fetch for Replayed {
    fn get(
        &self,
        url: &str,
        query: &[(&str, String)],
    ) -> Result<serde_json::Value, pec_core::ext::FetchError> {
        let local = format!("{}/{}", self.0, url.trim_start_matches("https://"));
        self.1.get_or_init(Default::default).get(&local, query)
    }
}

/// The name of the Windows sign-in entry. An `e2e` run uses its own, so it never reads or changes the person's.
fn autostart_name() -> &'static str {
    #[cfg(feature = "e2e")]
    if std::env::var("MYPEC_E2E_ROOT").is_ok() {
        return "My Pec e2e";
    }
    "My Pec"
}

pub fn run() {
    tauri::Builder::default()
        // onboarding FR-003: the single-instance plugin must be registered before any other.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app)
        }))
        .plugin(tauri_plugin_opener::init())
        // extensions R6: started at sign-in with `--autostart`, the window stays in the tray.
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg(AUTOSTART)
                .app_name(autostart_name())
                .build(),
        )
        .setup(|app| {
            let (root, service, api) = locations(app.path().data_dir()?);
            let secrets = secret_store::CredentialManager::new(service);
            let characters = app.path().resource_dir()?.join("characters");
            let core = Core::new(root, Box::new(secrets))
                .with_characters(characters)
                .with_extensions(pec_ext::all());
            #[cfg(feature = "e2e")]
            let core = match std::env::var("MYPEC_E2E_API_URL") {
                Ok(api) if std::env::var("MYPEC_E2E_ROOT").is_ok() => {
                    core.with_fetch(Box::new(Replayed(api, Default::default())))
                }
                _ => core,
            };
            // avatar R1: every window follows the avatar state, at most 15 times a second, always
            // the latest value.
            let (mut state, handle) = (core.subscribe_avatar(), app.handle().clone());
            std::thread::spawn(move || {
                while tauri::async_runtime::block_on(state.changed()).is_ok() {
                    let _ = handle.emit("avatar_state", *state.borrow_and_update());
                    std::thread::sleep(Duration::from_millis(1000 / 15));
                }
            });
            let floating = core.floating_settings().is_ok_and(|f| f.on);
            app.manage(AppCtx {
                core,
                claude: Claude::new(api, TIMEOUT),
            });
            if floating {
                floating::open(app.handle())?;
            }
            // extensions FR-028: at sign-in, My Pec waits in the tray; shown notes come back either way.
            if std::env::args().any(|a| a == AUTOSTART)
                && let Some(main) = app.get_webview_window("main")
            {
                main.hide()?;
            }
            notes::sync(app.handle());
            tick::start(app.handle());
            // The index catches up with the files off the main thread, so the first search finds it ready.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let _ = handle.state::<AppCtx>().core.sync_memory_index();
            });
            tray::build(app.handle())?;
            Ok(())
        })
        // onboarding FR-026: closing the window keeps the app in the tray; the first time, the webview explains.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "main"
            {
                api.prevent_close();
                if window
                    .state::<AppCtx>()
                    .core
                    .close_notice_shown()
                    .unwrap_or(true)
                {
                    let _ = window.hide();
                } else {
                    let _ = window.emit_to(window.label(), "close_notice", ());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::setup_options,
            commands::submit_key,
            commands::save_setup_draft,
            commands::complete_setup,
            commands::remove_key,
            commands::model_list,
            commands::select_model,
            commands::hide_to_tray,
            commands::chat_state,
            commands::chat_history,
            commands::send_message,
            commands::retry_reply,
            commands::regenerate_reply,
            commands::start_first_hello,
            commands::stop_reply,
            commands::start_fresh_context,
            commands::start_context_carrying,
            commands::language,
            commands::set_language,
            commands::weather_now,
            commands::weather_settings,
            commands::weather_search,
            commands::weather_choose,
            commands::weather_units,
            commands::open_link,
            commands::set_auto_recall,
            commands::set_sound,
            commands::memory_files,
            commands::memory_read,
            commands::memory_save,
            commands::memory_search,
            commands::memory_undo,
            commands::memory_forget,
            commands::open_workspace_folder,
            commands::prepare_world,
            commands::world_view,
            commands::forget_topic,
            commands::compact_conversation,
            commands::compaction_settings,
            commands::set_compaction_settings,
            commands::semantic_status,
            commands::start_semantic_download,
            commands::cancel_semantic_download,
            commands::set_semantic_search,
            commands::remove_semantic_model,
            commands::dismiss_semantic_offer,
            commands::semantic_tick,
            commands::dream_tick,
            commands::dream_now,
            commands::undo_last_dream,
            commands::dream_journal,
            commands::dreaming_settings,
            commands::set_dreaming_settings,
            commands::avatar_state,
            commands::set_listening,
            commands::avatar_manifest,
            commands::avatar_file,
            commands::choose_avatar,
            commands::set_avatar_picture,
            commands::floating_settings,
            commands::set_floating,
            commands::open_chat,
            commands::extensions,
            commands::set_extension,
            commands::autostart,
            commands::set_autostart,
            commands::notes,
            commands::new_note,
            commands::note_read,
            commands::note_save,
            commands::note_style,
            commands::show_note,
            commands::hide_note,
            commands::delete_note,
            commands::alarms,
            commands::save_alarm,
            commands::delete_alarm,
            commands::alarm_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running My Pec");
}
