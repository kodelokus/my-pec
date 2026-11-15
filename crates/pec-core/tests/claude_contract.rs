//! The Claude client against recorded responses (onboarding interfaces). No test reaches the real API.

mod common;

use common::KEY;
use pec_core::credentials::ApiKey;
use pec_core::llm::claude::{Claude, TIMEOUT};
use pec_core::llm::{
    Effort, LlmProvider, ProviderError, Reply, StreamFailure, ToolSpec, TurnRequest,
};
use serde_json::{Value, json};
use std::time::Duration;
use wiremock::matchers::{header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/tests/fixtures/models/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// A server that answers only requests carrying both headers (onboarding FR-011).
async fn serve(status: u16, body: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("x-api-key", KEY))
        .and(header("anthropic-version", "2026-10-21"))
        .respond_with(ResponseTemplate::new(status).set_body_raw(fixture(body), "application/json"))
        .mount(&server)
        .await;
    server
}

async fn list(base_url: &str, timeout: Duration) -> Result<Vec<String>, ProviderError> {
    let models = Claude::new(base_url, timeout)
        .list_models(&ApiKey::new(KEY))
        .await?;
    Ok(models
        .into_iter()
        .map(|m| format!("{} = {}", m.id, m.display_name))
        .collect())
}

#[tokio::test]
async fn a_list_is_read_and_unknown_fields_are_tolerated() {
    let server = serve(200, "ok.json").await;
    let ids = list(&server.uri(), TIMEOUT).await.unwrap();
    assert_eq!(ids.len(), 14);
    assert_eq!(ids[0], "claude-fable-5-2 = Claude Fable 5.2");
    assert_eq!(ids[13], "claude-haiku-4-5-20251001 = Claude Haiku 4.5");

    // chat R1: effort is sent only to models whose capabilities list it.
    let models = Claude::new(server.uri(), TIMEOUT)
        .list_models(&ApiKey::new(KEY))
        .await
        .unwrap();
    let effort = |id: &str| models.iter().find(|m| m.id == id).unwrap().supports_effort;
    assert!(effort("claude-opus-5-5"));
    assert!(!effort("claude-haiku-4-5-20251001"));
}

#[tokio::test]
async fn every_page_is_followed() {
    let server = MockServer::start().await;
    for (page, after) in [
        ("ok-paged-1.json", None),
        ("ok-paged-2.json", Some("claude-haiku-5-5")),
        ("ok-paged-3.json", Some("claude-opus-5-5")),
    ] {
        let mock = Mock::given(method("GET"))
            .and(path("/v1/models"))
            .and(header("x-api-key", KEY));
        let mock = match after {
            Some(id) => mock.and(query_param("after_id", id)),
            None => mock.and(query_param_is_missing("after_id")),
        };
        mock.respond_with(
            ResponseTemplate::new(200).set_body_raw(fixture(page), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    }
    let ids = list(&server.uri(), TIMEOUT).await.unwrap();
    assert_eq!(ids.len(), 14);
    assert_eq!(ids[13], "claude-haiku-4-5-20251001 = Claude Haiku 4.5");
}

#[tokio::test]
async fn each_failure_maps_to_its_outcome() {
    let cases = [
        (401, "401.json", ProviderError::KeyRejected),
        (403, "403.json", ProviderError::KeyRejected),
        (402, "402.json", ProviderError::NoCredit),
        (429, "429.json", ProviderError::Unreachable),
        (529, "529.json", ProviderError::Unreachable),
        (500, "500.json", ProviderError::Unreachable),
        (200, "malformed.txt", ProviderError::Unreachable),
        // The error type in the body decides before the status does.
        (400, "402.json", ProviderError::NoCredit),
        (503, "401.json", ProviderError::KeyRejected),
    ];
    for (status, body, expected) in cases {
        let server = serve(status, body).await;
        assert_eq!(
            list(&server.uri(), TIMEOUT).await,
            Err(expected),
            "{status} {body}"
        );
    }
}

#[tokio::test]
async fn no_answer_in_time_is_unreachable() {
    // Accepts the connection and never answers.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let _held = tokio::spawn(async move {
        let mut open = Vec::new();
        while let Ok((socket, _)) = listener.accept().await {
            open.push(socket);
        }
    });
    assert_eq!(
        list(&url, Duration::from_millis(300)).await,
        Err(ProviderError::Unreachable)
    );
}

#[tokio::test]
async fn a_closed_port_is_unreachable() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    assert_eq!(list(&url, TIMEOUT).await, Err(ProviderError::Unreachable));
}

fn turn(model: &str, effort: bool) -> TurnRequest {
    TurnRequest {
        model: model.into(),
        system: vec!["rules".into(), "soul".into()],
        messages: vec![json!({ "role": "user", "content": [{ "type": "text", "text": "Hi" }] })],
        effort: effort.then_some(Effort::Low),
        tools: Vec::new(),
        format: None,
    }
}

/// Replays one message fixture and returns the outcome with the text passed on as it arrived.
async fn stream(
    status: u16,
    body: &str,
    retry_after: Option<&str>,
    request: &TurnRequest,
) -> (MockServer, Result<Reply, StreamFailure>, String) {
    let server = MockServer::start().await;
    let mut response =
        ResponseTemplate::new(status).set_body_raw(message_fixture(body), "text/event-stream");
    if let Some(seconds) = retry_after {
        response = response.insert_header("retry-after", seconds);
    }
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", KEY))
        .and(header("anthropic-version", "2026-10-21"))
        .respond_with(response)
        .mount(&server)
        .await;
    let mut deltas = String::new();
    let result = Claude::new(server.uri(), TIMEOUT)
        .stream(&ApiKey::new(KEY), request, &mut |d: &str| {
            deltas.push_str(d)
        })
        .await;
    (server, result, deltas)
}

fn message_fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/tests/fixtures/messages/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[tokio::test]
async fn a_streamed_reply_is_assembled_exactly_as_received() {
    let (_, reply, deltas) = stream(200, "reply.sse", None, &turn("claude-opus-5-5", true)).await;
    let reply = reply.unwrap();
    assert_eq!(
        reply.content,
        [json!({ "type": "text", "text": "Hello, Sam!" })]
    );
    assert_eq!(reply.stop_reason.as_deref(), Some("end_turn"));
    assert_eq!(reply.usage.unwrap()["output_tokens"], 6);
    assert_eq!(deltas, "Hello, Sam!");

    let (_, reply, _) = stream(200, "thinking.sse", None, &turn("claude-opus-5-5", true)).await;
    assert_eq!(
        reply.unwrap().content,
        [
            json!({ "type": "thinking", "thinking": "", "signature": "EqQBCgIYAhIM9a2bQ==" }),
            json!({ "type": "text", "text": "It is Thursday." }),
        ]
    );
}

#[tokio::test]
async fn the_request_follows_the_contract() {
    let (server, _, _) = stream(200, "reply.sse", None, &turn("claude-opus-5-5", true)).await;
    let sent = &server.received_requests().await.unwrap()[0];
    let body: Value = serde_json::from_slice(&sent.body).unwrap();
    assert_eq!(body["stream"], true);
    assert_eq!(body["max_tokens"], 16000);
    assert_eq!(body["cache_control"], json!({ "type": "ephemeral" }));
    assert_eq!(
        body["system"][0],
        json!({ "type": "text", "text": "rules" })
    );
    assert_eq!(
        body["system"][1],
        json!({ "type": "text", "text": "soul", "cache_control": { "type": "ephemeral" } })
    );
    assert_eq!(body["output_config"], json!({ "effort": "low" }));
    assert_eq!(body["fallbacks"], "default");
    assert_eq!(
        sent.headers.get("anthropic-beta").unwrap(),
        "server-side-fallback-2026-07-01"
    );
    for absent in ["thinking", "temperature", "top_p", "top_k", "tools"] {
        assert!(body.get(absent).is_none(), "{absent}");
    }

    let (server, _, _) = stream(200, "reply.sse", None, &turn("claude-haiku-4-5", false)).await;
    let sent = &server.received_requests().await.unwrap()[0];
    let body: Value = serde_json::from_slice(&sent.body).unwrap();
    assert!(body.get("output_config").is_none());
    assert!(body.get("fallbacks").is_none());
    assert!(sent.headers.get("anthropic-beta").is_none());
}

/// The compaction interfaces: the summary asks for a JSON schema next to the effort.
#[tokio::test]
async fn a_structured_output_request_sends_its_schema() {
    let schema = json!({ "type": "object", "properties": { "thread": { "type": "string" } }, "required": ["thread"], "additionalProperties": false });
    let mut request = turn("claude-opus-5-5", true);
    request.format = Some(schema.clone());
    let (server, _, _) = stream(200, "reply.sse", None, &request).await;
    let body: Value =
        serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
    assert_eq!(
        body["output_config"],
        json!({ "effort": "low", "format": { "type": "json_schema", "schema": schema } })
    );
    assert!(body.get("tools").is_none());

    let mut request = turn("claude-haiku-4-5", false);
    request.format = Some(schema.clone());
    let (server, _, _) = stream(200, "reply.sse", None, &request).await;
    let body: Value =
        serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
    assert_eq!(
        body["output_config"],
        json!({ "format": { "type": "json_schema", "schema": schema } })
    );
}

#[tokio::test]
async fn each_stream_outcome_is_reported() {
    let request = turn("claude-opus-5-5", true);
    let (_, refusal, _) = stream(200, "refusal.sse", None, &request).await;
    assert_eq!(refusal.unwrap().stop_reason.as_deref(), Some("refusal"));

    let (_, fallback, _) = stream(200, "fallback.sse", None, &request).await;
    let fallback = fallback.unwrap();
    assert_eq!(fallback.model, "claude-opus-4-8");
    assert_eq!(fallback.content[0]["type"], "fallback");

    let failures = [
        (
            529,
            "overloaded.json",
            Some("7"),
            ProviderError::Busy,
            "",
            Some(7),
        ),
        (
            400,
            "too-long.json",
            None,
            ProviderError::ContextTooLong,
            "",
            None,
        ),
        (
            200,
            "stream-error.sse",
            None,
            ProviderError::Busy,
            "Once upon",
            None,
        ),
        (
            401,
            "../models/401.json",
            None,
            ProviderError::KeyRejected,
            "",
            None,
        ),
        (
            402,
            "../models/402.json",
            None,
            ProviderError::NoCredit,
            "",
            None,
        ),
    ];
    for (status, file, retry_after, error, text, wait) in failures {
        let (_, result, _) = stream(status, file, retry_after, &request).await;
        assert_eq!(
            result,
            Err(StreamFailure {
                error,
                text: text.into(),
                wait: wait.map(Duration::from_secs),
            }),
            "{status} {file}"
        );
    }
}

fn tool(name: &'static str) -> ToolSpec {
    ToolSpec {
        name,
        description: "A tool.",
        input_schema: json!({
            "type": "object",
            "properties": { "text": { "type": "string", "description": "Text." } },
            "required": ["text"],
            "additionalProperties": false
        }),
    }
}

#[tokio::test]
async fn tools_are_sent_strict_and_tool_calls_are_assembled() {
    let mut request = turn("claude-opus-5-5", true);
    request.tools = vec![tool("memory_search"), tool("memory_note")];
    let (server, reply, deltas) = stream(200, "tool-use.sse", None, &request).await;
    let body: Value =
        serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
    let names: Vec<_> = body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["memory_search", "memory_note"]);
    for sent in body["tools"].as_array().unwrap() {
        assert_eq!(sent["strict"], true);
        assert_eq!(sent["eager_input_streaming"], true);
        assert_eq!(sent["input_schema"], tool("x").input_schema);
    }
    assert!(body.get("tool_choice").is_none());

    let reply = reply.unwrap();
    assert_eq!(reply.stop_reason.as_deref(), Some("tool_use"));
    assert_eq!(deltas, "I'll remember that.");
    assert_eq!(
        reply.content[1],
        json!({
            "type": "tool_use", "id": "toolu_01", "name": "memory_remember",
            "input": { "text": "Allergic to peanuts.", "topic": "Health", "requested_by_user": true }
        })
    );

    let (_, reply, _) = stream(200, "tool-use-parallel.sse", None, &request).await;
    let calls: Vec<_> = reply
        .unwrap()
        .content
        .iter()
        .map(|b| (b["id"].clone(), b["input"].clone()))
        .collect();
    assert_eq!(
        calls,
        [
            (
                json!("toolu_02"),
                json!({ "text": "Rina is visiting next week." })
            ),
            (
                json!("toolu_03"),
                json!({ "key": "Call them", "value": "Sam" })
            ),
        ]
    );
}

/// The dreaming interfaces: the deep phase asks for medium effort, alone or with a schema.
#[tokio::test]
async fn an_effort_level_is_sent_as_asked() {
    let schema = json!({ "type": "object" });
    for (effort, format, expected) in [
        (Some(Effort::Medium), None, json!({ "effort": "medium" })),
        (
            Some(Effort::Medium),
            Some(schema.clone()),
            json!({ "effort": "medium", "format": { "type": "json_schema", "schema": schema } }),
        ),
        (None, None, Value::Null),
    ] {
        let mut request = turn("claude-opus-5-5", false);
        request.effort = effort;
        request.format = format;
        let (server, _, _) = stream(200, "reply.sse", None, &request).await;
        let body: Value =
            serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
        assert_eq!(body["output_config"], expected, "{effort:?}");
    }
}

#[tokio::test]
async fn the_topics_fixture_streams_into_its_answer() {
    let (_, reply, _) = stream(200, "topics.sse", None, &turn("claude-opus-5-5", true)).await;
    let reply = reply.unwrap();
    let text = reply.content[0]["text"].as_str().unwrap();
    let answer: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(answer["days"][0]["topics"][0]["keyword"], "10 km run");
}
