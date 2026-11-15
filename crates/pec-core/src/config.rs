//! `settings.json` and `state.json` (onboarding data 2). A missing file means defaults.

use crate::llm::ModelInfo;
use crate::model_choice::DEFAULT_MODEL;
use crate::onboarding::SetupDraft;
use crate::workspace::write_atomic;
use crate::{AppError, Core, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub chat: ChatSettings,
    pub context: ContextSettings,
    pub compaction: CompactionConfig,
    pub dreaming: DreamingConfig,
    pub recall: RecallSettings,
    pub search: SearchConfig,
    pub avatar: AvatarConfig,
    /// Extensions: each extension on or off, by id; unknown ids are kept.
    pub extensions: std::collections::BTreeMap<String, crate::ext::ExtensionSetting>,
    /// The interface language; none means English.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// The interface languages the app has text for: English and Indonesian.
pub const LANGUAGES: [&str; 2] = ["en", "id"];

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AvatarConfig {
    pub floating: FloatingConfig,
}

/// Avatar FR-023, FR-025: off by default; the last place in physical pixels, none before it was placed.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct FloatingConfig {
    pub on: bool,
    pub x: Option<i32>,
    pub y: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SearchConfig {
    pub semantic: SemanticConfig,
}

/// Search R10: no download until the user accepts; once accepted, on.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SemanticConfig {
    pub accepted: bool,
    pub enabled: bool,
    pub offer_dismissed: bool,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            accepted: false,
            enabled: true,
            offer_dismissed: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct DreamingConfig {
    /// Dreaming FR-026: on by default, at 03:00 local time, with the chat model.
    pub enabled: bool,
    /// `HH:MM`, 24-hour.
    pub time: String,
    pub model: Option<String>,
    /// Journey FR-008: name the day's topics and draw them in the world.
    pub world: bool,
}

impl Default for DreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            time: "03:00".into(),
            model: None,
            world: true,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct CompactionConfig {
    /// The model that writes flushes and summaries; none means the chat model (compaction FR-027).
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct RecallSettings {
    /// memory FR-019, FR-020: relevant memory is attached to each message unless the user turns it off.
    pub auto: bool,
}

impl Default for RecallSettings {
    fn default() -> Self {
        Self { auto: true }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ContextSettings {
    /// The working size; read through `budget()` (compaction R11).
    pub budget_tokens: u64,
}

impl ContextSettings {
    /// The working size, a stored value out of range read as the nearest end of it.
    pub fn budget(&self) -> u64 {
        self.budget_tokens
            .clamp(crate::compaction::MIN_BUDGET, crate::compaction::MAX_BUDGET)
    }
}

impl Default for ContextSettings {
    fn default() -> Self {
        Self {
            budget_tokens: 48_000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ChatSettings {
    pub model: String,
    /// A soft chime when a reply is done; on by default.
    pub sound: bool,
}

impl Default for ChatSettings {
    fn default() -> Self {
        Self {
            model: DEFAULT_MODEL.into(),
            sound: true,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AppState {
    pub setup: SetupProgress,
    pub first_hello_pending: bool,
    pub close_notice_shown: bool,
    pub models: ModelCache,
    pub world: WorldState,
    pub compaction: CompactionState,
    pub dreaming: DreamingState,
    /// Extensions FR-027: the person was asked once about launch at login.
    pub autostart_offered: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct DreamingState {
    /// The night of the last completed sweep (dreaming R3).
    pub last_night: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct CompactionState {
    /// A committed compaction whose flush notes may not be in the daily note yet (compaction R5).
    pub pending: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct WorldState {
    /// The last local day items were drawn (world R7). Bookkeeping; the world file is the record.
    pub last_day: Option<String>,
    /// Journey R2: days before this never get topics.
    pub journey_since: Option<String>,
    /// The newest day the journey has considered; only later days are due.
    pub journey_day: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SetupProgress {
    pub complete: bool,
    pub draft: Option<SetupDraft>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ModelCache {
    pub items: Vec<ModelInfo>,
    /// Unix seconds.
    pub fetched_at: Option<u64>,
}

fn load<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| AppError::Internal {
            message: format!("{}: {e}", path.display()),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(AppError::Internal {
            message: format!("{}: {e}", path.display()),
        }),
    }
}

fn save<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value).expect("state serializes");
    write_atomic(path, &bytes)
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl Core {
    pub(crate) fn settings(&self) -> Result<Settings> {
        load(&self.state_dir().join("settings.json"))
    }

    pub(crate) fn save_settings(&self, settings: &Settings) -> Result<()> {
        save(&self.state_dir().join("settings.json"), settings)
    }

    /// The interface language the person chose, as a language code; English until they choose (NFR-I18N-1).
    pub fn language(&self) -> String {
        self.settings()
            .ok()
            .and_then(|s| s.language)
            .unwrap_or_else(|| "en".into())
    }

    pub fn set_language(&self, language: &str) -> Result<String> {
        if !LANGUAGES.contains(&language) {
            return Err(AppError::invalid("language"));
        }
        let mut settings = self.settings()?;
        settings.language = Some(language.into());
        self.save_settings(&settings)?;
        Ok(self.language())
    }

    pub(crate) fn state(&self) -> Result<AppState> {
        load(&self.state_dir().join("state.json"))
    }

    pub(crate) fn update_state<T>(&self, change: impl FnOnce(&mut AppState) -> T) -> Result<T> {
        let _guard = self.state_lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut state = self.state()?;
        let out = change(&mut state);
        save(&self.state_dir().join("state.json"), &state)?;
        Ok(out)
    }

    pub fn close_notice_shown(&self) -> Result<bool> {
        Ok(self.state()?.close_notice_shown)
    }

    /// onboarding FR-026: the "still running in the tray" notice is shown once.
    pub fn mark_close_notice_shown(&self) -> Result<()> {
        self.update_state(|s| s.close_notice_shown = true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compaction_settings_and_state_default_and_round_trip() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.compaction.model, None);
        assert_eq!(settings.context.budget(), 48_000);
        for (stored, read) in [(1_000, 32_000), (48_000, 48_000), (900_000, 200_000)] {
            let s: Settings = serde_json::from_value(
                serde_json::json!({ "context": { "budgetTokens": stored } }),
            )
            .unwrap();
            assert_eq!(s.context.budget(), read, "{stored}");
        }
        let state: AppState = serde_json::from_str(r#"{"compaction":{"pending":"9-0"}}"#).unwrap();
        assert_eq!(state.compaction.pending.as_deref(), Some("9-0"));
        assert_eq!(
            serde_json::to_value(&state).unwrap()["compaction"],
            serde_json::json!({ "pending": "9-0" })
        );
        assert_eq!(AppState::default().compaction.pending, None);
    }

    #[test]
    fn dreaming_settings_and_state_default_and_round_trip() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(settings.dreaming.enabled);
        assert_eq!(settings.dreaming.time, "03:00");
        assert_eq!(settings.dreaming.model, None);
        let state: AppState =
            serde_json::from_str(r#"{"dreaming":{"lastNight":"2026-11-14"}}"#).unwrap();
        assert_eq!(state.dreaming.last_night.as_deref(), Some("2026-11-14"));
        assert_eq!(
            serde_json::to_value(&state).unwrap()["dreaming"],
            serde_json::json!({ "lastNight": "2026-11-14" })
        );
    }

    #[test]
    fn semantic_search_settings_default_to_no_download() {
        // Search R10: nothing downloads until the user accepts; accepting is enough to turn it on.
        let settings: Settings = serde_json::from_str("{}").unwrap();
        let semantic = &settings.search.semantic;
        assert!(!semantic.accepted && semantic.enabled && !semantic.offer_dismissed);
        let text =
            r#"{"search":{"semantic":{"accepted":true,"enabled":false,"offerDismissed":true}}}"#;
        let settings: Settings = serde_json::from_str(text).unwrap();
        assert_eq!(
            serde_json::to_value(&settings).unwrap()["search"],
            serde_json::json!({ "semantic": { "accepted": true, "enabled": false, "offerDismissed": true } })
        );
    }

    #[test]
    fn floating_settings_default_off_and_round_trip() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        let floating = &settings.avatar.floating;
        assert!(!floating.on && floating.x.is_none() && floating.y.is_none());
        let text = r#"{"floating":{"on":true,"x":-1200,"y":40}}"#;
        let settings: Settings = serde_json::from_str(&format!(r#"{{"avatar":{text}}}"#)).unwrap();
        assert_eq!(
            serde_json::to_value(&settings).unwrap()["avatar"],
            serde_json::from_str::<serde_json::Value>(text).unwrap()
        );
    }

    #[test]
    fn world_journey_settings_and_state_default_and_round_trip() {
        let settings: Settings = serde_json::from_str(r#"{"dreaming":{"enabled":false}}"#).unwrap();
        assert!(settings.dreaming.world, "journey FR-008: on by default");
        assert_eq!(AppState::default().world.journey_since, None);
        let text =
            r#"{"lastDay":"2026-11-14","journeySince":"2026-11-10","journeyDay":"2026-11-13"}"#;
        let state: AppState = serde_json::from_str(&format!(r#"{{"world":{text}}}"#)).unwrap();
        assert_eq!(state.world.journey_since.as_deref(), Some("2026-11-10"));
        assert_eq!(state.world.journey_day.as_deref(), Some("2026-11-13"));
        assert_eq!(
            serde_json::to_value(&state).unwrap()["world"],
            serde_json::from_str::<serde_json::Value>(text).unwrap()
        );
    }
}
