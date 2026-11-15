use super::sse::{Assembly, Events, Step};
use super::{Effort, LlmProvider, ModelInfo, ProviderError, Reply, StreamFailure, TurnRequest};
use crate::credentials::ApiKey;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub const API_URL: &str = "https://api.anthropic.com";
pub const TIMEOUT: Duration = Duration::from_secs(10);
const API_VERSION: &str = "2026-10-21";
const MAX_TOKENS: u32 = 16_000;
/// The API sends `ping` events while the model thinks, so this much silence means the connection is gone.
const SILENCE: Duration = Duration::from_secs(90);
/// Models that accept the server-side refusal fallback (chat R1).
const FALLBACK_MODELS: [&str; 5] = [
    "claude-opus-5-5",
    "claude-opus-5",
    "claude-sonnet-5-5",
    "claude-fable-5",
    "claude-fable-5-1",
];
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

/// The HTTPS client for the Claude API. See onboarding interfaces and chat interfaces.
pub struct Claude {
    http: reqwest::Client,
    base_url: String,
    timeout: Duration,
}

impl Claude {
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            http: reqwest::Client::builder()
                .connect_timeout(timeout)
                .build()
                .expect("the HTTP client builds"),
            base_url: base_url.into(),
            timeout,
        }
    }
}

#[derive(Deserialize)]
struct Page {
    data: Vec<WireModel>,
    #[serde(default)]
    has_more: bool,
    last_id: Option<String>,
}

#[derive(Deserialize)]
struct WireModel {
    id: String,
    display_name: String,
    #[serde(default)]
    capabilities: Value,
}

#[derive(Deserialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Deserialize)]
struct ErrorDetail {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    message: String,
}

impl LlmProvider for Claude {
    async fn list_models(&self, key: &ApiKey) -> Result<Vec<ModelInfo>, ProviderError> {
        // onboarding FR-011: the limit covers every page together, not each page.
        let deadline = Instant::now() + self.timeout;
        let mut models = Vec::new();
        let mut after: Option<String> = None;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let mut request = self
                .http
                .get(format!("{}/v1/models", self.base_url))
                .header("x-api-key", key.expose())
                .header("anthropic-version", API_VERSION)
                .query(&[("limit", "1000")])
                .timeout(remaining);
            if let Some(id) = &after {
                request = request.query(&[("after_id", id)]);
            }
            let response = request
                .send()
                .await
                .map_err(|_| ProviderError::Unreachable)?;
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(|_| ProviderError::Unreachable)?;
            if !status.is_success() {
                return Err(for_model_list(classify(status.as_u16(), &body)));
            }
            let page: Page =
                serde_json::from_slice(&body).map_err(|_| ProviderError::Unreachable)?;
            models.extend(page.data.into_iter().map(|m| ModelInfo {
                supports_effort: m.capabilities["effort"]["supported"].as_bool() == Some(true),
                id: m.id,
                display_name: m.display_name,
            }));
            match (page.has_more, page.last_id) {
                (true, Some(last)) => after = Some(last),
                _ => return Ok(models),
            }
        }
    }

    async fn stream(
        &self,
        key: &ApiKey,
        request: &TurnRequest,
        on_text: &mut (dyn FnMut(&str) + Send),
    ) -> Result<Reply, StreamFailure> {
        let failed = |error, text: &str| StreamFailure {
            error,
            text: text.into(),
            wait: None,
        };
        let fallback = FALLBACK_MODELS.contains(&request.model.as_str());
        let mut http = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", key.expose())
            .header("anthropic-version", API_VERSION)
            .json(&body(request, fallback));
        if fallback {
            http = http.header("anthropic-beta", FALLBACK_BETA);
        }
        let mut response = http
            .send()
            .await
            .map_err(|_| failed(ProviderError::Unreachable, ""))?;
        let status = response.status();
        if !status.is_success() {
            let wait = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok()?.trim().parse::<u64>().ok())
                .map(Duration::from_secs);
            let body = tokio::time::timeout(self.timeout, response.bytes())
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or_default();
            return Err(StreamFailure {
                error: classify(status.as_u16(), &body),
                text: String::new(),
                wait,
            });
        }
        let (mut events, mut assembly, mut text) =
            (Events::default(), Assembly::default(), String::new());
        loop {
            let chunk = match tokio::time::timeout(SILENCE, response.chunk()).await {
                Ok(Ok(Some(chunk))) => chunk,
                _ => return Err(failed(ProviderError::Unreachable, &text)),
            };
            for event in events.push(&chunk) {
                match assembly.apply(&event) {
                    Step::Text(delta) => {
                        text.push_str(&delta);
                        on_text(&delta);
                    }
                    Step::Error(kind) => return Err(failed(classify_kind(&kind, ""), &text)),
                    Step::Done => {
                        return Ok(Reply {
                            content: assembly
                                .blocks
                                .into_iter()
                                .filter(|b| !b.is_null())
                                .collect(),
                            stop_reason: assembly.stop_reason,
                            usage: assembly.usage,
                            model: assembly.model,
                        });
                    }
                    Step::None => {}
                }
            }
        }
    }
}

/// The request body of chat interfaces.
fn body(request: &TurnRequest, fallback: bool) -> Value {
    let last = request.system.len().saturating_sub(1);
    let system: Vec<Value> = request
        .system
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let mut block = json!({ "type": "text", "text": text });
            if i == last {
                block["cache_control"] = json!({ "type": "ephemeral" });
            }
            block
        })
        .collect();
    let mut body = json!({
        "model": request.model,
        "max_tokens": MAX_TOKENS,
        "stream": true,
        "cache_control": { "type": "ephemeral" },
        "system": system,
        "messages": request.messages,
    });
    if !request.tools.is_empty() {
        // Strict keeps arguments schema-valid under the default `auto` choice; inputs are validated in the core too.
        body["tools"] = request
            .tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                    "strict": true,
                    "eager_input_streaming": true,
                })
            })
            .collect();
    }
    if let Some(effort) = request.effort {
        body["output_config"]["effort"] = json!(match effort {
            Effort::Low => "low",
            Effort::Medium => "medium",
        });
    }
    if let Some(schema) = &request.format {
        body["output_config"]["format"] = json!({ "type": "json_schema", "schema": schema });
    }
    if fallback {
        body["fallbacks"] = json!("default");
    }
    body
}

/// The error type in the body decides first, the status second.
fn classify(status: u16, body: &[u8]) -> ProviderError {
    match serde_json::from_slice::<ErrorBody>(body) {
        Ok(b) if !b.error.kind.is_empty() => match classify_kind(&b.error.kind, &b.error.message) {
            ProviderError::Invalid => by_status(status),
            known => known,
        },
        _ => by_status(status),
    }
}

fn classify_kind(kind: &str, message: &str) -> ProviderError {
    match kind {
        "authentication_error" | "permission_error" => ProviderError::KeyRejected,
        "billing_error" => ProviderError::NoCredit,
        "rate_limit_error" | "overloaded_error" | "api_error" => ProviderError::Busy,
        "invalid_request_error" if message.contains("prompt is too long") => {
            ProviderError::ContextTooLong
        }
        _ => ProviderError::Invalid,
    }
}

fn by_status(status: u16) -> ProviderError {
    match status {
        401 | 403 => ProviderError::KeyRejected,
        402 => ProviderError::NoCredit,
        429 | 500..=599 => ProviderError::Busy,
        _ => ProviderError::Invalid,
    }
}

/// The onboarding feature's model list knows three outcomes; every other failure reads as unreachable.
fn for_model_list(error: ProviderError) -> ProviderError {
    match error {
        ProviderError::KeyRejected | ProviderError::NoCredit => error,
        _ => ProviderError::Unreachable,
    }
}
