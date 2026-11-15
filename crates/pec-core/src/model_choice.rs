use crate::config::now;
use crate::llm::{LlmProvider, ModelInfo, ProviderError};
use crate::{AppError, Core, Result};
use serde::Serialize;
use ts_rs::TS;

pub const DEFAULT_MODEL: &str = "claude-opus-5-5";

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ModelList {
    pub models: Vec<ModelInfo>,
    pub selected: String,
    /// The list did not come from the provider in this call (onboarding FR-031).
    pub stale: bool,
    /// Why a requested refresh failed (onboarding FR-033, FR-036).
    pub problem: Option<ProviderError>,
    /// The model that was replaced by the fallback in this call (onboarding FR-030).
    pub fallback_from: Option<String>,
}

/// onboarding FR-030: a model no longer offered falls back to the default, then to the first listed model.
fn resolve(chosen: &str, models: &[ModelInfo]) -> (String, Option<String>) {
    let offered = |id: &str| models.iter().any(|m| m.id == id);
    if models.is_empty() || offered(chosen) {
        return (chosen.into(), None);
    }
    let fallback = if offered(DEFAULT_MODEL) {
        DEFAULT_MODEL.into()
    } else {
        models[0].id.clone()
    };
    (fallback, Some(chosen.into()))
}

impl Core {
    pub(crate) fn save_models(&self, models: Vec<ModelInfo>) -> Result<()> {
        self.update_state(|s| {
            s.models.items = models;
            s.models.fetched_at = Some(now());
        })
    }

    pub async fn model_list(
        &self,
        provider: &impl LlmProvider,
        refresh: bool,
    ) -> Result<ModelList> {
        let (mut stale, mut problem) = (true, None);
        if let Some(key) = refresh.then(|| self.secrets.get()).flatten() {
            match provider.list_models(&key).await {
                Ok(models) => {
                    self.save_models(models)?;
                    stale = false;
                }
                Err(e) => problem = Some(e),
            }
        }
        let models = self.state()?.models.items;
        let mut settings = self.settings()?;
        let (selected, fallback_from) = resolve(&settings.chat.model, &models);
        if fallback_from.is_some() {
            settings.chat.model.clone_from(&selected);
            self.save_settings(&settings)?;
        }
        Ok(ModelList {
            models,
            selected,
            stale,
            problem,
            fallback_from,
        })
    }

    pub fn select_model(&self, id: &str) -> Result<ModelList> {
        let models = self.state()?.models.items;
        if !models.iter().any(|m| m.id == id) {
            return Err(AppError::invalid("id"));
        }
        let mut settings = self.settings()?;
        settings.chat.model = id.into();
        self.save_settings(&settings)?;
        Ok(ModelList {
            models,
            selected: id.into(),
            stale: true,
            problem: None,
            fallback_from: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(ids: &[&str]) -> Vec<ModelInfo> {
        ids.iter()
            .map(|id| ModelInfo {
                id: (*id).into(),
                display_name: id.to_uppercase(),
                supports_effort: true,
            })
            .collect()
    }

    #[test]
    fn a_model_no_longer_offered_falls_back() {
        let cases: [(&str, &[&str], &str, Option<&str>); 5] = [
            ("claude-x", &["claude-x", DEFAULT_MODEL], "claude-x", None),
            ("claude-x", &[], "claude-x", None),
            (
                "gone",
                &["claude-x", DEFAULT_MODEL],
                DEFAULT_MODEL,
                Some("gone"),
            ),
            ("gone", &["claude-x", "claude-y"], "claude-x", Some("gone")),
            (
                DEFAULT_MODEL,
                &["claude-y"],
                "claude-y",
                Some(DEFAULT_MODEL),
            ),
        ];
        for (chosen, ids, selected, from) in cases {
            let (got, fallback_from) = resolve(chosen, &list(ids));
            assert_eq!(
                (got.as_str(), fallback_from.as_deref()),
                (selected, from),
                "{chosen} in {ids:?}"
            );
        }
    }
}
