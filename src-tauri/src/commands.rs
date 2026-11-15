//! Thin handlers: deserialize, call the core, return its result (each feature page's Interfaces).

use crate::AppCtx;
use pec_core::AppError;
use pec_core::avatar::{AvatarManifest, AvatarSource, AvatarState, FloatingSettings};
use pec_core::chat::{
    ChatEvent, ChatPage, ChatSink, ChatState, LocalClock, SendRefusal, is_openable_link,
};
use pec_core::compaction::{CompactResult, CompactionSettings};
use pec_core::credentials::{KeyCheckOutcome, KeyStatus};
use pec_core::dream::{DreamJournal, DreamStatus, DreamingSettings, UndoDreamOutcome};
use pec_core::ext::{Autostart, ExtensionInfo};
use pec_core::memory::{
    ForgetOutcome, ForgetTarget, MemoryDoc, MemoryFile, SaveOutcome, SearchHit, SemanticStatus,
    UndoOutcome,
};
use pec_core::model_choice::ModelList;
use pec_core::onboarding::{
    self, AppStatus, CompanionSummary, SetupCompletion, SetupDraft, SetupOptions,
};
use pec_core::world::{PrepareOutcome, WorldView};
use pec_ext::alarms::{self, Action, AlarmList, Repeat};
use pec_ext::notes::{self, NoteColour, NoteDoc, NoteSummary};
use pec_ext::weather;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

type Result<T> = std::result::Result<T, AppError>;

#[tauri::command]
pub fn app_status(ctx: State<'_, AppCtx>) -> Result<AppStatus> {
    ctx.core.app_status()
}

#[tauri::command]
pub fn setup_options() -> SetupOptions {
    onboarding::setup_options()
}

#[tauri::command]
pub async fn submit_key(ctx: State<'_, AppCtx>, key: String) -> Result<KeyCheckOutcome> {
    ctx.core.submit_key(&ctx.claude, &key).await
}

#[tauri::command]
pub fn save_setup_draft(ctx: State<'_, AppCtx>, draft: SetupDraft) -> Result<()> {
    ctx.core.save_setup_draft(draft)
}

#[tauri::command]
pub fn complete_setup(
    ctx: State<'_, AppCtx>,
    mode: SetupCompletion,
    clock: LocalClock,
) -> Result<CompanionSummary> {
    ctx.core.complete_setup(mode, &clock)
}

#[tauri::command]
pub fn remove_key(ctx: State<'_, AppCtx>) -> Result<KeyStatus> {
    ctx.core.remove_key()
}

#[tauri::command]
pub async fn model_list(ctx: State<'_, AppCtx>, refresh: bool) -> Result<ModelList> {
    ctx.core.model_list(&ctx.claude, refresh).await
}

#[tauri::command]
pub fn select_model(ctx: State<'_, AppCtx>, id: String) -> Result<ModelList> {
    ctx.core.select_model(&id)
}

#[tauri::command]
pub fn hide_to_tray(ctx: State<'_, AppCtx>, window: WebviewWindow) -> Result<()> {
    ctx.core.mark_close_notice_shown()?;
    window.hide().map_err(|e| AppError::Internal {
        message: e.to_string(),
    })
}

/// Sends each reply event to the webview over the call's channel (chat R5).
pub struct ChannelSink(Channel<ChatEvent>);

impl ChatSink for ChannelSink {
    fn send(&self, event: ChatEvent) {
        let _ = self.0.send(event);
    }
}

#[tauri::command]
pub fn chat_state(ctx: State<'_, AppCtx>) -> Result<ChatState> {
    ctx.core.chat_state()
}

#[tauri::command]
pub fn memory_files(ctx: State<'_, AppCtx>) -> Result<Vec<MemoryFile>> {
    ctx.core.memory_files()
}

#[tauri::command]
pub fn memory_read(ctx: State<'_, AppCtx>, path: String) -> Result<MemoryDoc> {
    ctx.core.memory_read(&path)
}

#[tauri::command]
pub fn memory_save(
    ctx: State<'_, AppCtx>,
    path: String,
    text: String,
    base_hash: String,
) -> Result<SaveOutcome> {
    ctx.core.memory_save(&path, &text, &base_hash)
}

#[tauri::command]
pub fn memory_search(ctx: State<'_, AppCtx>, query: String) -> Result<Vec<SearchHit>> {
    ctx.core.memory_search(&query)
}

#[tauri::command]
pub fn memory_forget(
    ctx: State<'_, AppCtx>,
    path: String,
    target: ForgetTarget,
    base_hash: String,
) -> Result<ForgetOutcome> {
    ctx.core.memory_forget(&path, target, &base_hash)
}

/// journey FR-041: a story leaves the journey; its drawing stays.
#[tauri::command]
pub fn forget_topic(
    ctx: State<'_, AppCtx>,
    day: String,
    n: u32,
    base_hash: String,
) -> Result<ForgetOutcome> {
    ctx.core.forget_topic(&day, n, &base_hash)
}

#[tauri::command]
pub fn memory_undo(ctx: State<'_, AppCtx>, path: String, force: bool) -> Result<UndoOutcome> {
    ctx.core.memory_undo(&path, force)
}

/// memory FR-025: the workspace folder in the system file manager, opened from Rust (no opener permission in the webview).
#[tauri::command]
pub fn open_workspace_folder(app: AppHandle, ctx: State<'_, AppCtx>) -> Result<()> {
    app.opener()
        .open_path(
            ctx.core.workspace_folder().display().to_string(),
            None::<&str>,
        )
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })
}

#[tauri::command]
pub fn prepare_world(ctx: State<'_, AppCtx>, clock: LocalClock) -> Result<PrepareOutcome> {
    ctx.core.prepare_world(&clock)
}

#[tauri::command]
pub fn world_view(ctx: State<'_, AppCtx>) -> Result<WorldView> {
    ctx.core.world_view()
}

#[tauri::command]
pub fn set_auto_recall(ctx: State<'_, AppCtx>, enabled: bool) -> Result<ChatState> {
    ctx.core.set_auto_recall(enabled)
}

#[tauri::command]
pub fn set_sound(ctx: State<'_, AppCtx>, enabled: bool) -> Result<ChatState> {
    ctx.core.set_sound(enabled)
}

#[tauri::command]
pub fn chat_history(
    ctx: State<'_, AppCtx>,
    before: Option<String>,
    limit: u32,
) -> Result<ChatPage> {
    ctx.core.chat_history(before.as_deref(), limit)
}

#[tauri::command]
pub async fn send_message(
    ctx: State<'_, AppCtx>,
    text: String,
    clock: LocalClock,
    on_event: Channel<ChatEvent>,
) -> Result<Option<SendRefusal>> {
    let sink = ChannelSink(on_event);
    ctx.core
        .send_message(&ctx.claude, &text, &clock, &sink)
        .await
}

#[tauri::command]
pub async fn retry_reply(
    ctx: State<'_, AppCtx>,
    clock: LocalClock,
    on_event: Channel<ChatEvent>,
) -> Result<Option<SendRefusal>> {
    ctx.core
        .retry_reply(&ctx.claude, &clock, &ChannelSink(on_event))
        .await
}

#[tauri::command]
pub async fn regenerate_reply(
    ctx: State<'_, AppCtx>,
    clock: LocalClock,
    on_event: Channel<ChatEvent>,
) -> Result<Option<SendRefusal>> {
    ctx.core
        .regenerate_reply(&ctx.claude, &clock, &ChannelSink(on_event))
        .await
}

#[tauri::command]
pub async fn start_first_hello(
    ctx: State<'_, AppCtx>,
    clock: LocalClock,
    on_event: Channel<ChatEvent>,
) -> Result<Option<SendRefusal>> {
    let sink = ChannelSink(on_event);
    ctx.core.start_first_hello(&ctx.claude, &clock, &sink).await
}

#[tauri::command]
pub fn stop_reply(ctx: State<'_, AppCtx>) {
    ctx.core.stop_reply();
}

#[tauri::command]
pub fn start_fresh_context(ctx: State<'_, AppCtx>, clock: LocalClock) -> Result<ChatState> {
    ctx.core.start_fresh_context(&clock)
}

/// NFR-I18N-1: the interface language, read before the first screen is drawn.
#[tauri::command]
pub fn language(ctx: State<'_, AppCtx>) -> String {
    ctx.core.language()
}

/// The webview reloads its text; the tray is relabelled here.
#[tauri::command]
pub fn set_language(app: AppHandle, language: String) -> Result<String> {
    let chosen = app.state::<AppCtx>().core.set_language(&language)?;
    crate::tray::relabel(&app);
    let _ = app.emit("language_changed", ());
    Ok(chosen)
}

#[tauri::command]
pub fn start_context_carrying(ctx: State<'_, AppCtx>, clock: LocalClock) -> Result<ChatState> {
    ctx.core.start_context_carrying(&clock)
}

#[tauri::command]
pub fn open_link(app: AppHandle, url: String) -> Result<()> {
    if !is_openable_link(&url) {
        return Err(AppError::InvalidInput {
            field: "url".into(),
        });
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })
}

#[tauri::command]
pub async fn compact_conversation(
    ctx: State<'_, AppCtx>,
    clock: LocalClock,
    focus: Option<String>,
    manual: bool,
) -> Result<CompactResult> {
    ctx.core
        .compact_conversation(&ctx.claude, &clock, focus.as_deref(), manual)
        .await
}

#[tauri::command]
pub fn compaction_settings(ctx: State<'_, AppCtx>) -> Result<CompactionSettings> {
    ctx.core.compaction_settings()
}

#[tauri::command]
pub fn set_compaction_settings(
    ctx: State<'_, AppCtx>,
    budget_tokens: u64,
    model: Option<String>,
) -> Result<CompactionSettings> {
    ctx.core.set_compaction_settings(budget_tokens, model)
}

/// Search by meaning (search interfaces).
#[tauri::command]
pub fn semantic_status(ctx: State<'_, AppCtx>) -> Result<SemanticStatus> {
    ctx.core.semantic_status()
}

#[tauri::command]
pub async fn start_semantic_download(ctx: State<'_, AppCtx>) -> Result<SemanticStatus> {
    ctx.core.start_semantic_download().await
}

#[tauri::command]
pub fn cancel_semantic_download(ctx: State<'_, AppCtx>) -> Result<SemanticStatus> {
    ctx.core.cancel_semantic_download()
}

#[tauri::command]
pub fn set_semantic_search(ctx: State<'_, AppCtx>, enabled: bool) -> Result<SemanticStatus> {
    ctx.core.set_semantic_search(enabled)
}

#[tauri::command]
pub fn remove_semantic_model(ctx: State<'_, AppCtx>) -> Result<SemanticStatus> {
    ctx.core.remove_semantic_model()
}

#[tauri::command]
pub fn dismiss_semantic_offer(ctx: State<'_, AppCtx>) -> Result<SemanticStatus> {
    ctx.core.dismiss_semantic_offer()
}

#[tauri::command]
pub async fn semantic_tick(ctx: State<'_, AppCtx>) -> Result<SemanticStatus> {
    ctx.core.semantic_tick().await
}

#[tauri::command]
pub async fn dream_tick(ctx: State<'_, AppCtx>, clock: LocalClock) -> Result<DreamStatus> {
    ctx.core.dream_tick(&ctx.claude, &clock).await
}

#[tauri::command]
pub async fn dream_now(ctx: State<'_, AppCtx>, clock: LocalClock) -> Result<DreamStatus> {
    ctx.core.dream_now(&ctx.claude, &clock).await
}

#[tauri::command]
pub fn undo_last_dream(ctx: State<'_, AppCtx>, force: bool) -> Result<UndoDreamOutcome> {
    ctx.core.undo_last_dream(force)
}

#[tauri::command]
pub fn dream_journal(ctx: State<'_, AppCtx>) -> Result<DreamJournal> {
    ctx.core.dream_journal()
}

#[tauri::command]
pub fn dreaming_settings(ctx: State<'_, AppCtx>) -> Result<DreamingSettings> {
    ctx.core.dreaming_settings()
}

#[tauri::command]
pub fn set_dreaming_settings(
    ctx: State<'_, AppCtx>,
    enabled: bool,
    time: String,
    model: Option<String>,
    world: bool,
    clock: LocalClock,
) -> Result<DreamingSettings> {
    ctx.core
        .set_dreaming_settings(enabled, &time, model, world, &clock)
}

/// The avatar state for a window that just opened (avatar interfaces).
#[tauri::command]
pub fn avatar_state(ctx: State<'_, AppCtx>) -> AvatarState {
    ctx.core.avatar_state()
}

#[tauri::command]
pub fn set_listening(ctx: State<'_, AppCtx>, listening: bool) {
    ctx.core.set_listening(listening);
}

#[tauri::command]
pub fn avatar_manifest(ctx: State<'_, AppCtx>) -> Result<AvatarManifest> {
    ctx.core.avatar_manifest()
}

/// The bytes go back as a raw response, not a JSON array.
#[tauri::command]
pub fn avatar_file(
    ctx: State<'_, AppCtx>,
    source: AvatarSource,
    path: String,
) -> Result<tauri::ipc::Response> {
    ctx.core
        .avatar_file(&source, &path)
        .map(tauri::ipc::Response::new)
}

#[tauri::command]
pub fn choose_avatar(
    ctx: State<'_, AppCtx>,
    character: String,
    still: bool,
) -> Result<AvatarManifest> {
    ctx.core.choose_avatar(&character, still)
}

/// The picture's bytes arrive as the raw request body.
#[tauri::command]
pub fn set_avatar_picture(
    ctx: State<'_, AppCtx>,
    request: tauri::ipc::Request<'_>,
) -> Result<AvatarManifest> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err(AppError::InvalidInput {
            field: "pictureType".into(),
        });
    };
    ctx.core.set_avatar_picture(bytes)
}

#[tauri::command]
pub fn floating_settings(ctx: State<'_, AppCtx>) -> Result<FloatingSettings> {
    ctx.core.floating_settings()
}

/// Async: on Windows, a window built inside a synchronous command never loads its page.
#[tauri::command]
pub async fn set_floating(app: AppHandle, on: bool) -> Result<FloatingSettings> {
    crate::floating::set(&app, on)
}

#[tauri::command]
pub fn open_chat(app: AppHandle) {
    crate::show_main(&app);
}

// Extensions (extensions interfaces). Their rules are in pec-ext; the core confines each one to its own
// place.

fn changed(app: &AppHandle, id: &str) {
    let _ = app.emit("ext_changed", [id]);
}

#[tauri::command]
pub fn extensions(ctx: State<'_, AppCtx>) -> Result<Vec<ExtensionInfo>> {
    ctx.core.extensions()
}

/// Async: turning notes on opens their windows.
#[tauri::command]
pub async fn set_extension(app: AppHandle, id: String, on: bool) -> Result<Vec<ExtensionInfo>> {
    let list = app.state::<AppCtx>().core.set_extension(&id, on)?;
    crate::notes::sync(&app);
    changed(&app, &id);
    Ok(list)
}

#[tauri::command]
pub fn autostart(app: AppHandle) -> Result<Autostart> {
    let on = app.autolaunch().is_enabled().unwrap_or(false);
    app.state::<AppCtx>().core.autostart(on)
}

/// extensions FR-026, FR-027: only through this command, so the answer to the offer is recorded.
#[tauri::command]
pub fn set_autostart(app: AppHandle, on: bool) -> Result<Autostart> {
    let launcher = app.autolaunch();
    let done = if on {
        launcher.enable()
    } else {
        launcher.disable()
    };
    done.map_err(|e| AppError::Internal {
        message: e.to_string(),
    })?;
    let on = launcher.is_enabled().unwrap_or(on);
    app.state::<AppCtx>().core.mark_autostart_offered(on)
}

#[tauri::command]
pub fn notes(ctx: State<'_, AppCtx>) -> Result<Vec<NoteSummary>> {
    Ok(notes::list(&ctx.core.ext_dir("notes")?))
}

/// Async: it opens the new note's window. A note window passes its text when its file changed underneath it
/// (extensions R9).
#[tauri::command]
pub async fn new_note(app: AppHandle, text: Option<String>) -> Result<NoteSummary> {
    let ctx = app.state::<AppCtx>();
    let note = notes::create(
        &ctx.core.ext_dir("notes")?,
        text.as_deref().unwrap_or_default(),
    )?;
    let _ = crate::notes::open(&app, &note.file, false);
    changed(&app, "notes");
    Ok(note)
}

#[tauri::command]
pub fn note_read(ctx: State<'_, AppCtx>, window: WebviewWindow, file: String) -> Result<NoteDoc> {
    crate::notes::own(&window, &file)?;
    notes::read(&ctx.core.ext_dir("notes")?, &file)
}

#[tauri::command]
pub fn note_save(
    app: AppHandle,
    window: WebviewWindow,
    file: String,
    text: String,
    base_hash: String,
) -> Result<SaveOutcome> {
    crate::notes::own(&window, &file)?;
    let ctx = app.state::<AppCtx>();
    let dir = ctx.core.ext_dir("notes")?;
    let saved = notes::save(&dir, &file, &text, &base_hash)?;
    changed(&app, "notes");
    Ok(saved)
}

#[tauri::command]
pub fn note_style(
    app: AppHandle,
    window: WebviewWindow,
    file: String,
    colour: Option<NoteColour>,
    on_top: Option<bool>,
) -> Result<NoteDoc> {
    crate::notes::own(&window, &file)?;
    let doc = notes::style(
        &app.state::<AppCtx>().core.ext_dir("notes")?,
        &file,
        colour,
        on_top,
    )?;
    if let Some(note) = app.get_webview_window(&crate::notes::label(&file)) {
        let _ = note.set_always_on_top(doc.on_top);
    }
    changed(&app, "notes");
    Ok(doc)
}

/// Async: showing a note builds its window.
#[tauri::command]
pub async fn show_note(app: AppHandle, file: String, shown: bool) -> Result<()> {
    let ctx = app.state::<AppCtx>();
    let dir = ctx.core.ext_dir("notes")?;
    notes::set_shown(&dir, &file, shown)?;
    if shown {
        let on_top = notes::read(&dir, &file)?.on_top;
        let _ = crate::notes::open(&app, &file, on_top);
    } else {
        crate::notes::close(&app, &file);
    }
    changed(&app, "notes");
    Ok(())
}

/// A note window hides itself (extensions FR-011).
#[tauri::command]
pub fn hide_note(app: AppHandle, window: WebviewWindow) -> Result<()> {
    let file = crate::notes::file_of(&window).ok_or_else(|| AppError::invalid("window"))?;
    notes::set_shown(&app.state::<AppCtx>().core.ext_dir("notes")?, &file, false)?;
    crate::notes::close(&app, &file);
    changed(&app, "notes");
    Ok(())
}

#[tauri::command]
pub fn delete_note(app: AppHandle, file: String) -> Result<()> {
    crate::notes::close(&app, &file);
    notes::delete(&app.state::<AppCtx>().core.ext_dir("notes")?, &file)?;
    changed(&app, "notes");
    Ok(())
}

fn alarm_list(list: AlarmList) -> AlarmList {
    AlarmList {
        notifications_held: crate::tick::held(),
        ..list
    }
}

#[tauri::command]
pub fn alarms(ctx: State<'_, AppCtx>) -> Result<AlarmList> {
    let dir = ctx.core.ext_dir("alarms")?;
    Ok(alarm_list(alarms::list(&dir, crate::tick::local_now())))
}

#[tauri::command]
pub fn save_alarm(
    app: AppHandle,
    key: Option<String>,
    label: String,
    time: String,
    repeat: Repeat,
) -> Result<AlarmList> {
    let ctx = app.state::<AppCtx>();
    let dir = ctx.core.ext_dir("alarms")?;
    let list = alarms::save(
        &dir,
        key.as_deref(),
        &label,
        &time,
        repeat,
        crate::tick::local_now(),
    )?;
    changed(&app, "alarms");
    Ok(alarm_list(list))
}

#[tauri::command]
pub fn delete_alarm(app: AppHandle, key: String) -> Result<AlarmList> {
    let ctx = app.state::<AppCtx>();
    let dir = ctx.core.ext_dir("alarms")?;
    let list = alarms::delete(&dir, &key, crate::tick::local_now())?;
    changed(&app, "alarms");
    Ok(alarm_list(list))
}

#[tauri::command]
pub fn alarm_action(app: AppHandle, key: String, action: Action) -> Result<AlarmList> {
    let ctx = app.state::<AppCtx>();
    let dir = ctx.core.ext_dir("alarms")?;
    let list = alarms::act(&dir, &key, action, crate::tick::local_now())?;
    changed(&app, "alarms");
    Ok(alarm_list(list))
}

// Weather: requests run on a blocking thread, never on the main thread (weather interfaces).

fn weather_dir(app: &AppHandle) -> Result<pec_core::ext::ExtDir<'_>> {
    app.state::<AppCtx>().inner().core.ext_dir("weather")
}

/// Runs a weather call that may reach the network on a blocking thread.
async fn blocking<T: Send + 'static>(
    app: AppHandle,
    call: impl FnOnce(&AppHandle) -> Result<T> + Send + 'static,
) -> Result<T> {
    tauri::async_runtime::spawn_blocking(move || call(&app))
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?
}

#[tauri::command]
pub fn weather_now(app: AppHandle) -> Result<Option<weather::WeatherView>> {
    let dir = weather_dir(&app)?;
    let on = app.state::<AppCtx>().core.extension_on("weather");
    Ok(on
        .then(|| weather::current(&dir, crate::tick::now()))
        .flatten())
}

#[tauri::command]
pub fn weather_settings(app: AppHandle) -> Result<weather::WeatherSettings> {
    Ok(weather::settings(&weather_dir(&app)?, crate::tick::now()))
}

#[tauri::command]
pub async fn weather_search(app: AppHandle, name: String) -> Result<Option<Vec<weather::Place>>> {
    blocking(app, move |app| weather::search(&weather_dir(app)?, &name)).await
}

#[tauri::command]
pub async fn weather_choose(
    app: AppHandle,
    place: weather::Place,
) -> Result<weather::WeatherSettings> {
    let shown = blocking(app.clone(), move |app| {
        weather::choose(&weather_dir(app)?, &place, crate::tick::now())
    })
    .await?;
    changed(&app, "weather");
    Ok(shown)
}

#[tauri::command]
pub fn weather_units(app: AppHandle, units: weather::Units) -> Result<weather::WeatherSettings> {
    let shown = weather::set_units(&weather_dir(&app)?, units, crate::tick::now())?;
    changed(&app, "weather");
    Ok(shown)
}
