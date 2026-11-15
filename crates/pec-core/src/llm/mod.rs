pub mod claude;
mod sse;

use crate::credentials::ApiKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use ts_rs::TS;

/// The core's only view of the model provider (Principle VI).
pub trait LlmProvider: Sync {
    fn list_models(
        &self,
        key: &ApiKey,
    ) -> impl Future<Output = Result<Vec<ModelInfo>, ProviderError>> + Send;

    /// Streams one reply. Text deltas go to `on_text` as they arrive; dropping the future cancels it.
    fn stream(
        &self,
        key: &ApiKey,
        request: &TurnRequest,
        on_text: &mut (dyn FnMut(&str) + Send),
    ) -> impl Future<Output = Result<Reply, StreamFailure>> + Send;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    /// Whether the model takes an effort level (chat R1). Missing in older caches.
    #[serde(default)]
    pub supports_effort: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ProviderError {
    KeyRejected,
    NoCredit,
    Unreachable,
    /// Rate-limited or overloaded; worth retrying after a wait.
    Busy,
    ContextTooLong,
    /// The reply ended with a refusal.
    Declined,
    /// Any other rejection of the request.
    Invalid,
}

/// One chat request. The system blocks and messages are built by `chat::context`.
#[derive(Debug, Clone, PartialEq)]
pub struct TurnRequest {
    pub model: String,
    pub system: Vec<String>,
    /// Messages as raw JSON, exactly as they are sent.
    pub messages: Vec<serde_json::Value>,
    /// Sent only for models that take an effort level (chat R1).
    pub effort: Option<Effort>,
    /// Fixed for a context (Principle III). Empty for contexts from before memory (memory R2).
    pub tools: Vec<ToolSpec>,
    /// A JSON schema the reply must follow (structured output); only the compaction summary sets it.
    pub format: Option<serde_json::Value>,
}

/// How hard the model thinks: chat starts low, the dream's deep phase medium (architecture section 11).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effort {
    Low,
    Medium,
}

/// The effort to send for `model`: `level` when the model takes one, otherwise none.
pub fn effort_for(cached: &[ModelInfo], model: &str, level: Effort) -> Option<Effort> {
    cached
        .iter()
        .any(|m| m.id == model && m.supports_effort)
        .then_some(level)
}

/// A tool the model may call. Provider-specific fields are added by the provider.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: serde_json::Value,
}

/// A reply that reached its end. Content blocks are kept as the raw JSON that arrived.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub content: Vec<serde_json::Value>,
    pub stop_reason: Option<String>,
    pub usage: Option<serde_json::Value>,
    pub model: String,
}

/// A reply that did not reach its end.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamFailure {
    pub error: ProviderError,
    /// The reply text received before the failure.
    pub text: String,
    /// How long the provider asked to wait before retrying.
    pub wait: Option<Duration>,
}
