use crate::llm::{LlmProvider, ProviderError};
use crate::{AppError, Core, Result};
use serde::Serialize;
use std::fmt;
use ts_rs::TS;

/// The user's provider key. It never prints itself and is never serialized (Principle II).
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn last_four(&self) -> String {
        let skip = self.0.chars().count().saturating_sub(4);
        self.0.chars().skip(skip).collect()
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(****)")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("****")
    }
}

/// All the webview ever learns about the key (onboarding FR-014).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "status")]
#[ts(export)]
pub enum KeyStatus {
    Absent,
    Present {
        #[serde(rename = "lastFour")]
        last_four: String,
    },
}

#[derive(Debug)]
pub struct StoreUnavailable;

/// The operating system's credential store, holding one key.
pub trait SecretStore: Send + Sync {
    fn get(&self) -> Option<ApiKey>;
    fn set(&self, key: &ApiKey) -> std::result::Result<(), StoreUnavailable>;
    fn delete(&self) -> std::result::Result<(), StoreUnavailable>;
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "outcome")]
#[ts(export)]
pub enum KeyCheckOutcome {
    Accepted {
        #[serde(rename = "lastFour")]
        last_four: String,
    },
    Incomplete,
    Rejected,
    NoCredit,
    Unreachable,
    StorageUnavailable,
}

const MIN_KEY_CHARS: usize = 20;

/// onboarding FR-010: surrounding whitespace is ignored; an empty, short or broken value is not a key.
fn parse_key(raw: &str) -> Option<ApiKey> {
    let value = raw.trim();
    let complete = value.chars().count() >= MIN_KEY_CHARS && !value.contains(char::is_whitespace);
    complete.then(|| ApiKey::new(value))
}

impl Core {
    pub fn key_status(&self) -> KeyStatus {
        match self.secrets.get() {
            Some(key) => KeyStatus::Present {
                last_four: key.last_four(),
            },
            None => KeyStatus::Absent,
        }
    }

    /// onboarding FR-013, FR-028: the key is stored only once the provider accepts it, replacing any earlier key.
    pub async fn submit_key(
        &self,
        provider: &impl LlmProvider,
        raw: &str,
    ) -> Result<KeyCheckOutcome> {
        let Some(key) = parse_key(raw) else {
            return Ok(KeyCheckOutcome::Incomplete);
        };
        let models = match provider.list_models(&key).await {
            Ok(models) => models,
            Err(ProviderError::KeyRejected) => return Ok(KeyCheckOutcome::Rejected),
            Err(ProviderError::NoCredit) => return Ok(KeyCheckOutcome::NoCredit),
            Err(_) => return Ok(KeyCheckOutcome::Unreachable),
        };
        if self.secrets.set(&key).is_err() {
            return Ok(KeyCheckOutcome::StorageUnavailable);
        }
        self.save_models(models)?;
        Ok(KeyCheckOutcome::Accepted {
            last_four: key.last_four(),
        })
    }

    pub fn remove_key(&self) -> Result<KeyStatus> {
        self.secrets.delete().map_err(|_| AppError::Internal {
            message: "the credential store refused".into(),
        })?;
        Ok(KeyStatus::Absent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "sk-ant-api03-abcdefghijklmnopWXYZ";

    #[test]
    fn printing_a_key_never_shows_it() {
        let key = ApiKey::new(KEY);
        for printed in [format!("{key}"), format!("{key:?}"), format!("{key:#?}")] {
            assert!(!printed.contains("abcdefghijklmnop"), "{printed}");
            assert!(!printed.contains("WXYZ"), "{printed}");
        }
    }

    #[test]
    fn obviously_incomplete_values_are_not_keys() {
        for raw in [
            "",
            "   ",
            "abc",
            "sk-ant-short",
            "sk-ant-api03-abcd efghijklmnop",
            "sk-ant-api03\nabcdefghijklmnop",
        ] {
            assert!(parse_key(raw).is_none(), "{raw:?}");
        }
    }

    #[test]
    fn surrounding_whitespace_is_ignored() {
        let key = parse_key(&format!("  \r\n{KEY}\t\n")).expect("a key");
        assert_eq!(key.expose(), KEY);
        assert_eq!(key.last_four(), "WXYZ");
    }
}
