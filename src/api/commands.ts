// The only module that calls into the core (each feature page's Interfaces).

import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Action } from "../bindings/Action";
import type { AlarmList } from "../bindings/AlarmList";
import type { AppError } from "../bindings/AppError";
import type { AppStatus } from "../bindings/AppStatus";
import type { Autostart } from "../bindings/Autostart";
import type { AvatarManifest } from "../bindings/AvatarManifest";
import type { AvatarSource } from "../bindings/AvatarSource";
import type { AvatarState } from "../bindings/AvatarState";
import type { ChatEvent } from "../bindings/ChatEvent";
import type { ChatPage } from "../bindings/ChatPage";
import type { ChatState } from "../bindings/ChatState";
import type { CompactionSettings } from "../bindings/CompactionSettings";
import type { CompactResult } from "../bindings/CompactResult";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { DreamingSettings } from "../bindings/DreamingSettings";
import type { DreamJournal } from "../bindings/DreamJournal";
import type { DreamStatus } from "../bindings/DreamStatus";
import type { ExtensionInfo } from "../bindings/ExtensionInfo";
import type { FloatingSettings } from "../bindings/FloatingSettings";
import type { ForgetOutcome } from "../bindings/ForgetOutcome";
import type { ForgetTarget } from "../bindings/ForgetTarget";
import type { KeyCheckOutcome } from "../bindings/KeyCheckOutcome";
import type { KeyStatus } from "../bindings/KeyStatus";
import type { LocalClock } from "../bindings/LocalClock";
import type { MemoryDoc } from "../bindings/MemoryDoc";
import type { MemoryFile } from "../bindings/MemoryFile";
import type { ModelList } from "../bindings/ModelList";
import type { NoteColour } from "../bindings/NoteColour";
import type { NoteDoc } from "../bindings/NoteDoc";
import type { NoteSummary } from "../bindings/NoteSummary";
import type { Place } from "../bindings/Place";
import type { PrepareOutcome } from "../bindings/PrepareOutcome";
import type { Repeat } from "../bindings/Repeat";
import type { SaveOutcome } from "../bindings/SaveOutcome";
import type { SearchHit } from "../bindings/SearchHit";
import type { SemanticStatus } from "../bindings/SemanticStatus";
import type { SendRefusal } from "../bindings/SendRefusal";
import type { SetupCompletion } from "../bindings/SetupCompletion";
import type { SetupDraft } from "../bindings/SetupDraft";
import type { SetupOptions } from "../bindings/SetupOptions";
import type { UndoDreamOutcome } from "../bindings/UndoDreamOutcome";
import type { UndoOutcome } from "../bindings/UndoOutcome";
import type { Units } from "../bindings/Units";
import type { WeatherSettings } from "../bindings/WeatherSettings";
import type { WeatherView } from "../bindings/WeatherView";
import type { WorldView } from "../bindings/WorldView";

export const appStatus = () => invoke<AppStatus>("app_status");
export const setupOptions = () => invoke<SetupOptions>("setup_options");
export const submitKey = (key: string) => invoke<KeyCheckOutcome>("submit_key", { key });
export const saveSetupDraft = (draft: SetupDraft) => invoke<void>("save_setup_draft", { draft });
export const completeSetup = (mode: SetupCompletion) =>
  invoke<CompanionSummary>("complete_setup", { mode, clock: localClock() });
export const removeKey = () => invoke<KeyStatus>("remove_key");
export const modelList = (refresh: boolean) => invoke<ModelList>("model_list", { refresh });
export const selectModel = (id: string) => invoke<ModelList>("select_model", { id });
export const hideToTray = () => invoke<void>("hide_to_tray");

export const chatState = () => invoke<ChatState>("chat_state");
export const chatHistory = (before: string | null, limit: number) =>
  invoke<ChatPage>("chat_history", { before, limit });
export const stopReply = () => invoke<void>("stop_reply");
export const startFreshContext = () => invoke<ChatState>("start_fresh_context", { clock: localClock() });
/** A new context that carries the current turns over as text: catching up when there is nothing to compact. */
export const startContextCarrying = () =>
  invoke<ChatState>("start_context_carrying", { clock: localClock() });
export const compactConversation = (focus: string | null, manual: boolean) =>
  invoke<CompactResult>("compact_conversation", { clock: localClock(), focus, manual });
export const compactionSettings = () => invoke<CompactionSettings>("compaction_settings");
export const dreamTick = () => invoke<DreamStatus>("dream_tick", { clock: localClock() });
export const dreamNow = () => invoke<DreamStatus>("dream_now", { clock: localClock() });
export const undoLastDream = (force: boolean) => invoke<UndoDreamOutcome>("undo_last_dream", { force });
export const dreamJournal = () => invoke<DreamJournal>("dream_journal");
export const dreamingSettings = () => invoke<DreamingSettings>("dreaming_settings");
export const setDreamingSettings = (enabled: boolean, time: string, model: string | null, world: boolean) =>
  invoke<DreamingSettings>("set_dreaming_settings", { enabled, time, model, world, clock: localClock() });
export const setCompactionSettings = (budgetTokens: number, model: string | null) =>
  invoke<CompactionSettings>("set_compaction_settings", { budgetTokens, model });
export const openLink = (url: string) => invoke<void>("open_link", { url });
export const setAutoRecall = (enabled: boolean) => invoke<ChatState>("set_auto_recall", { enabled });
export const setSound = (enabled: boolean) => invoke<ChatState>("set_sound", { enabled });

export const memoryFiles = () => invoke<MemoryFile[]>("memory_files");
export const memoryRead = (path: string) => invoke<MemoryDoc>("memory_read", { path });
export const memorySave = (path: string, text: string, baseHash: string) =>
  invoke<SaveOutcome>("memory_save", { path, text, baseHash });
export const memorySearch = (query: string) => invoke<SearchHit[]>("memory_search", { query });
export const memoryUndo = (path: string, force: boolean) =>
  invoke<UndoOutcome>("memory_undo", { path, force });
export const openWorkspaceFolder = () => invoke<void>("open_workspace_folder");

// Search by meaning.
export const semanticStatus = () => invoke<SemanticStatus>("semantic_status");
export const startSemanticDownload = () => invoke<SemanticStatus>("start_semantic_download");
export const cancelSemanticDownload = () => invoke<SemanticStatus>("cancel_semantic_download");
export const setSemanticSearch = (enabled: boolean) =>
  invoke<SemanticStatus>("set_semantic_search", { enabled });
export const removeSemanticModel = () => invoke<SemanticStatus>("remove_semantic_model");
export const dismissSemanticOffer = () => invoke<SemanticStatus>("dismiss_semantic_offer");
export const semanticTick = () => invoke<SemanticStatus>("semantic_tick");

export const prepareWorld = () => invoke<PrepareOutcome>("prepare_world", { clock: localClock() });
export const worldView = () => invoke<WorldView>("world_view");
export const forgetTopic = (day: string, n: number, baseHash: string) =>
  invoke<ForgetOutcome>("forget_topic", { day, n, baseHash });
export const memoryForget = (path: string, target: ForgetTarget, baseHash: string) =>
  invoke<ForgetOutcome>("memory_forget", { path, target, baseHash });

// The avatar.
export const avatarState = () => invoke<AvatarState>("avatar_state");
export const setListening = (listening: boolean) => invoke<void>("set_listening", { listening });
export const avatarManifest = () => invoke<AvatarManifest>("avatar_manifest");
export const avatarFile = (source: AvatarSource, path: string) =>
  invoke<ArrayBuffer>("avatar_file", { source, path });
export const chooseAvatar = (character: string, still: boolean) =>
  invoke<AvatarManifest>("choose_avatar", { character, still });
/** The picture's bytes go as the raw request body (onboarding R7). */
export const setAvatarPicture = (bytes: ArrayBuffer) => invoke<AvatarManifest>("set_avatar_picture", bytes);
export const floatingSettings = () => invoke<FloatingSettings>("floating_settings");
export const setFloating = (on: boolean) => invoke<FloatingSettings>("set_floating", { on });
export const openChat = () => invoke<void>("open_chat");
export const onAvatarState = (handler: (state: AvatarState) => void) =>
  listen<AvatarState>("avatar_state", (event) => handler(event.payload));

/** A reply streams its events to `onEvent`; the promise settles when the reply ends or is refused. */
type OnEvent = (event: ChatEvent) => void;
const channel = (onEvent: OnEvent) => Object.assign(new Channel<ChatEvent>(), { onmessage: onEvent });
export const sendMessage = (text: string, onEvent: OnEvent) =>
  invoke<SendRefusal | null>("send_message", { text, clock: localClock(), onEvent: channel(onEvent) });
export const retryReply = (onEvent: OnEvent) =>
  invoke<SendRefusal | null>("retry_reply", { clock: localClock(), onEvent: channel(onEvent) });
export const regenerateReply = (onEvent: OnEvent) =>
  invoke<SendRefusal | null>("regenerate_reply", { clock: localClock(), onEvent: channel(onEvent) });
export const startFirstHello = (onEvent: OnEvent) =>
  invoke<SendRefusal | null>("start_first_hello", { clock: localClock(), onEvent: channel(onEvent) });

/** The user's local time with its offset, and their time zone (onboarding R8). */
export function localClock(now = new Date()): LocalClock {
  const pad = (n: number) => String(Math.abs(n)).padStart(2, "0");
  const offset = -now.getTimezoneOffset();
  const date = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
  const time = `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`;
  const zone = `${offset < 0 ? "-" : "+"}${pad(Math.trunc(offset / 60))}:${pad(offset % 60)}`;
  return { iso: `${date}T${time}${zone}`, zone: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC" };
}

export const onEvent = (
  name: "close_notice" | "open_settings" | "floating_changed" | "open_alarms",
  handler: () => void,
) => listen(name, () => handler());

/** A rejected command carries an `AppError`; anything else is reported as internal. */
export function asAppError(error: unknown): AppError {
  return typeof error === "object" && error !== null && "kind" in error
    ? (error as AppError)
    : { kind: "Internal", message: String(error) };
}

// Extensions (extensions interfaces).
export const extensions = () => invoke<ExtensionInfo[]>("extensions");
export const setExtension = (id: string, on: boolean) => invoke<ExtensionInfo[]>("set_extension", { id, on });
export const autostart = () => invoke<Autostart>("autostart");
export const setAutostart = (on: boolean) => invoke<Autostart>("set_autostart", { on });
export const notes = () => invoke<NoteSummary[]>("notes");
export const newNote = (text: string | null = null) => invoke<NoteSummary>("new_note", { text });
export const noteRead = (file: string) => invoke<NoteDoc>("note_read", { file });
export const noteSave = (file: string, text: string, baseHash: string) =>
  invoke<SaveOutcome>("note_save", { file, text, baseHash });
export const noteStyle = (file: string, colour: NoteColour | null, onTop: boolean | null) =>
  invoke<NoteDoc>("note_style", { file, colour, onTop });
export const showNote = (file: string, shown: boolean) => invoke<void>("show_note", { file, shown });
export const hideNote = () => invoke<void>("hide_note");
export const deleteNote = (file: string) => invoke<void>("delete_note", { file });
export const alarms = () => invoke<AlarmList>("alarms");
export const saveAlarm = (key: string | null, label: string, time: string, repeat: Repeat) =>
  invoke<AlarmList>("save_alarm", { key, label, time, repeat });
export const deleteAlarm = (key: string) => invoke<AlarmList>("delete_alarm", { key });
export const alarmAction = (key: string, action: Action) =>
  invoke<AlarmList>("alarm_action", { key, action });
/** The tick or a change saw an extension's files change (onboarding R9). */
/** NFR-I18N-1: the interface language; every window reloads its text when it changes. */
export const language = () => invoke<string>("language");
export const setLanguage = (language: string) => invoke<string>("set_language", { language });
export const onLanguageChanged = (handler: () => void) => listen("language_changed", handler);

export const onExtChanged = (handler: (ids: string[]) => void) =>
  listen<string[]>("ext_changed", (event) => handler(event.payload));

/** The weather to draw, or null; never makes a request. */
export const weatherNow = () => invoke<WeatherView | null>("weather_now");
export const weatherSettings = () => invoke<WeatherSettings>("weather_settings");
/** Up to five places, or null when the service couldn't be reached. Sends only the name. */
export const weatherSearch = (name: string) => invoke<Place[] | null>("weather_search", { name });
export const weatherChoose = (place: Place) => invoke<WeatherSettings>("weather_choose", { place });
export const weatherUnits = (units: Units) => invoke<WeatherSettings>("weather_units", { units });
