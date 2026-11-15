//! What every extension shares, with a stand-in extension and the fake provider: the tools a context records, the
//! refusal when one is off, the mark on the reply, and the tick (extensions R1 to R4).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink, clock, tool_use};
use pec_core::Core;
use pec_core::chat::ChatEvent;
use pec_core::ext::{Due, ExtDir, Extension, Tick, ToolAnswer};
use pec_core::llm::ToolSpec;
use pec_core::memory::{MarkKind, MemoryMark};
use serde_json::{Value, json};
use time::{OffsetDateTime, PrimitiveDateTime};

/// Writes its input to its own folder; panics on `boom`; rings one alarm every tick.
struct Stub;

impl Extension for Stub {
    fn id(&self) -> &'static str {
        "stub"
    }
    fn name(&self) -> &'static str {
        "Stub"
    }
    fn about(&self) -> &'static str {
        "Keeps one line."
    }
    fn skill(&self) -> &'static str {
        "## Stub\n\nKeep a line with `stub_keep` when the person asks.\n"
    }
    fn place(&self) -> &'static str {
        "stub/"
    }
    fn tools(&self) -> Vec<ToolSpec> {
        vec![ToolSpec {
            name: "stub_keep",
            description: "Keeps a line.",
            input_schema: json!({ "type": "object" }),
        }]
    }
    fn run(&self, _tool: &str, input: &Value, dir: &ExtDir, now: PrimitiveDateTime) -> ToolAnswer {
        let text = input["text"].as_str().unwrap_or_default();
        if text == "boom" {
            return ToolAnswer::error("The stub failed.");
        }
        let path = dir.file("kept.md").unwrap();
        pec_core::ext::write_atomic(&path, format!("{now} {text}").as_bytes()).unwrap();
        let mark = MemoryMark {
            kind: MarkKind::NoteMade,
            path: "stub/kept.md".into(),
            line: None,
            excerpt: text.into(),
            ok: true,
        };
        ToolAnswer::ok(
            format!("Kept \"{text}\". Whole note: BODY-{text}"),
            Some(mark),
        )
    }
    fn tick(&self, _now: OffsetDateTime, _dir: &ExtDir) -> Tick {
        Tick {
            ring: vec![Due {
                key: "k".into(),
                label: "Stretch".into(),
                date: "2026-11-13".into(),
                time: "21:40".into(),
            }],
            missed: Vec::new(),
            changed: true,
        }
    }
}

fn with_stub(f: &Fixture, provider: &FakeProvider) -> Core {
    f.ready(provider).with_extensions(vec![Box::new(Stub)])
}

async fn send(core: &Core, provider: &FakeProvider, text: &str) -> VecSink {
    let sink = VecSink::default();
    core.send_message(provider, text, &clock(), &sink)
        .await
        .unwrap();
    sink
}

fn tool_names(request: &pec_core::llm::TurnRequest) -> Vec<&'static str> {
    request.tools.iter().map(|t| t.name).collect()
}

fn keep(text: &str) -> Vec<Vec<Step>> {
    vec![
        vec![Step::Tools(vec![tool_use(
            "t1",
            "stub_keep",
            json!({ "text": text }),
        )])],
        vec![Step::Text("Done."), Step::End("end_turn")],
    ]
}

/// The tool result the companion was given, from the last request.
fn result(provider: &FakeProvider) -> Value {
    let request = provider.requests().pop().unwrap();
    request.messages.last().unwrap()["content"][0].clone()
}

#[tokio::test]
async fn a_context_records_its_extensions_and_keeps_its_tools_when_they_are_toggled() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = with_stub(&f, &provider);
    provider.script(keep("tea"));
    let sink = send(&core, &provider, "Keep tea.").await;

    let context = f
        .transcript()
        .into_iter()
        .find(|l| l["type"] == "context")
        .unwrap();
    assert_eq!(context["harness"], "talk-1");
    assert_eq!(context["extensions"], json!(["stub"]));
    // The recorded extension's SKILL.md follows the harness.
    let system = &provider.requests()[0].system[0];
    assert!(system.ends_with(Stub.skill()), "{system}");
    // extensions R2: memory tools first, then the recorded extension's.
    let tools = tool_names(&provider.requests()[0]);
    assert_eq!(tools.last(), Some(&"stub_keep"));
    assert_eq!(tools.len(), 6);
    // extensions FR-005: the change is on the reply; the result is an untrusted message (Principle IV).
    assert_eq!(f.read("stub/kept.md"), "2026-11-13 21:40:00.0 tea");
    assert!(sink.events().iter().any(
        |e| matches!(e, ChatEvent::MemoryWritten { mark } if mark.kind == MarkKind::NoteMade)
    ));
    let results = f
        .transcript()
        .into_iter()
        .find(|l| l["content"][0]["type"] == "tool_result")
        .unwrap();
    assert_eq!(results["provenance"], "untrusted");

    // extensions FR-007: turned off, the tool list stays and the call is refused with nothing changed.
    core.set_extension("stub", false).unwrap();
    provider.script(keep("coffee"));
    send(&core, &provider, "Keep coffee.").await;
    assert_eq!(result(&provider)["is_error"], true);
    assert_eq!(
        result(&provider)["content"],
        "Stub is turned off. The person can turn it on in Settings, under Extensions."
    );
    assert_eq!(f.read("stub/kept.md"), "2026-11-13 21:40:00.0 tea");
    let requests = provider.requests();
    assert_eq!(
        tool_names(&requests[0]),
        tool_names(requests.last().unwrap())
    );

    // A context started while it is off has only the memory tools, and is told the extension is off and where
    // to turn it on.
    core.start_fresh_context(&clock()).unwrap();
    send(&core, &provider, "Hello again.").await;
    let request = provider.requests().pop().unwrap();
    assert_eq!(tool_names(&request).len(), 5);
    assert!(
        request.system[0].ends_with(
            "No extension is turned on.\n\n## Stub (turned off)\n\nKeeps one line. It is turned off, so you have \
             no tools for it. If the person asks for it, tell them they can turn it on in Settings, under \
             Extensions.\n"
        ),
        "{}",
        request.system[0]
    );
    assert!(!core.chat_state().unwrap().update_pending);
    // Turned on mid-context, the tools still don't change until the next context, and the chat offers one.
    core.set_extension("stub", true).unwrap();
    send(&core, &provider, "And now?").await;
    assert_eq!(tool_names(&provider.requests().pop().unwrap()).len(), 5);
    assert!(core.chat_state().unwrap().update_pending);
    core.start_fresh_context(&clock()).unwrap();
    assert!(!core.chat_state().unwrap().update_pending);
}

#[tokio::test]
async fn a_failing_extension_answers_with_an_error_and_the_reply_completes() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = with_stub(&f, &provider);
    provider.script(keep("boom"));
    let sink = send(&core, &provider, "Break it.").await;
    // extensions FR-010.
    assert_eq!(result(&provider)["is_error"], true);
    assert_eq!(sink.deltas(), "Done.");
}

#[tokio::test]
async fn an_older_context_keeps_its_request_and_offers_to_catch_up() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = with_stub(&f, &provider);
    send(&core, &provider, "one").await;
    let context = f
        .transcript()
        .into_iter()
        .find(|l| l["type"] == "context")
        .unwrap();

    // Principle III: each older context has its old rules byte for byte; only `ext-2` is followed by SKILL.md.
    let ext_2 = format!(
        "{}\n{}",
        include_str!("../src/chat/harness-ext-2.md"),
        Stub.skill()
    );
    for (version, system, recorded, tools) in [
        (
            "avatar-1",
            include_str!("../src/chat/harness-avatar-1.md"),
            false,
            5,
        ),
        (
            "ext-1",
            include_str!("../src/chat/harness-ext-1.md"),
            true,
            6,
        ),
        ("ext-2", ext_2.as_str(), true, 6),
    ] {
        let mut old = context.clone();
        old["harness"] = version.into();
        old.as_object_mut().unwrap().remove("skills");
        if !recorded {
            old.as_object_mut().unwrap().remove("extensions");
        }
        f.write("sessions/main.jsonl", &format!("{old}\n"));
        let core = f.core().with_extensions(vec![Box::new(Stub)]);
        send(&core, &provider, "two").await;
        let request = provider.requests().pop().unwrap();
        assert_eq!(request.system[0], system, "{version}");
        assert_eq!(tool_names(&request).len(), tools, "{version}");
        // The chat offers a fresh context, which has today's rules and extensions.
        assert!(core.chat_state().unwrap().update_pending, "{version}");
    }

    // A context keeps the extension text it recorded, even after an app update changes a SKILL.md.
    let mut recorded = context.clone();
    recorded["skills"] = "## Stub\n\nAn older SKILL.md.\n".into();
    f.write("sessions/main.jsonl", &format!("{recorded}\n"));
    let core = f.core().with_extensions(vec![Box::new(Stub)]);
    send(&core, &provider, "three").await;
    let request = provider.requests().pop().unwrap();
    assert!(
        request.system[0].ends_with("\n## Stub\n\nAn older SKILL.md.\n"),
        "{}",
        request.system[0]
    );
    assert!(!core.chat_state().unwrap().update_pending);
}

#[test]
fn the_tick_asks_only_the_extensions_that_are_on_and_leaves_files_alone() {
    let f = Fixture::new();
    let core = Core::new(f.root(), Box::new(common::MemoryStore::default()))
        .with_extensions(vec![Box::new(Stub)]);
    f.write("stub/kept.md", "mine");
    let now = OffsetDateTime::UNIX_EPOCH;
    let tick = core.ext_tick(now);
    assert_eq!(tick.ring.len(), 1);
    assert_eq!(tick.changed, ["stub"]);
    // extensions SC-006: off, it shows nothing and its files are untouched.
    core.set_extension("stub", false).unwrap();
    assert_eq!(core.ext_tick(now), pec_core::ext::ExtTick::default());
    assert_eq!(f.read("stub/kept.md"), "mine");
    let info = core.extensions().unwrap();
    assert!(!info[0].on && info[0].folder.ends_with("stub"));
    assert!(core.set_extension("nope", true).is_err());
}

#[tokio::test]
async fn extension_files_are_not_memory() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = with_stub(&f, &provider);
    f.write("notes/list-k7q2mx.md", "Zanzibar spices");
    f.write("alarms.md", "- [ ] every day 07:30 Zanzibar call\n");
    // extensions FR-009: not found by memory search.
    assert_eq!(core.memory_search("Zanzibar"), Ok(Vec::new()));
}

#[tokio::test]
async fn a_compaction_records_the_extensions_on_then_and_never_summarizes_their_text() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = with_stub(&f, &provider);
    provider.script(keep("tea"));
    send(&core, &provider, "Keep tea.").await;
    for (text, tokens) in [
        ("two", 22_000),
        ("three", 32_000),
        ("four", 39_000),
        ("five", 43_000),
    ] {
        provider.script(vec![vec![
            Step::Usage(json!({ "input_tokens": tokens })),
            Step::Text("Sure."),
            Step::End("end_turn"),
        ]]);
        send(&core, &provider, text).await;
    }
    core.set_extension("stub", false).unwrap();
    let summary = json!({ "thread": "Chatting.", "commitments": [], "relationship": "Warm.", "facts": [], "notes": [] });
    provider.script(vec![
        vec![Step::Text("Nothing new."), Step::End("end_turn")],
        vec![Step::Blocks(vec![
            json!({ "type": "text", "text": summary.to_string() }),
        ])],
    ]);
    let result = core
        .compact_conversation(&provider, &clock(), None, false)
        .await
        .unwrap();
    assert_eq!(
        result.outcome,
        pec_core::compaction::CompactOutcome::Compacted
    );
    // extensions FR-007: the context the compaction opened records the extensions on at that moment: none.
    let opened = f
        .transcript()
        .into_iter()
        .rfind(|l| l["type"] == "context")
        .unwrap();
    assert!(opened.get("extensions").is_none(), "{opened}");
    // extensions FR-009: what the extension returned never reaches the summary.
    let asked = provider
        .requests()
        .into_iter()
        .find(|r| r.format.is_some())
        .unwrap();
    assert!(
        !serde_json::to_string(&asked.messages)
            .unwrap()
            .contains("BODY-tea")
    );
}

/// Starts off, like an extension that goes online, and adds what its file says to each new message.
struct Sky;

impl Extension for Sky {
    fn id(&self) -> &'static str {
        "sky"
    }
    fn name(&self) -> &'static str {
        "Sky"
    }
    fn about(&self) -> &'static str {
        "Says the sky."
    }
    fn skill(&self) -> &'static str {
        "## Sky\n\nThe context block may say the sky.\n"
    }
    fn place(&self) -> &'static str {
        "sky.md"
    }
    fn tools(&self) -> Vec<ToolSpec> {
        Vec::new()
    }
    fn run(&self, _: &str, _: &Value, _: &ExtDir, _: PrimitiveDateTime) -> ToolAnswer {
        ToolAnswer::error("No tools.")
    }
    fn tick(&self, _: OffsetDateTime, _: &ExtDir) -> Tick {
        Tick::default()
    }
    fn default_on(&self) -> bool {
        false
    }
    fn context_line(&self, dir: &ExtDir, _: OffsetDateTime) -> Option<String> {
        let sky = std::fs::read_to_string(dir.place()).ok()?;
        Some(format!("Sky: {}.", sky.trim()))
    }
}

/// The time block of each message the person sent, in the last request.
fn time_blocks(provider: &FakeProvider) -> Vec<String> {
    provider
        .requests()
        .pop()
        .unwrap()
        .messages
        .iter()
        .filter(|m| m["role"] == "user")
        .filter_map(|m| m["content"][0]["text"].as_str())
        .filter(|t| t.starts_with("<context>"))
        .map(Into::into)
        .collect()
}

// weather FR-001, FR-018, FR-019, extensions R5.
#[tokio::test]
async fn an_extension_starts_off_and_its_line_reaches_only_new_messages_of_contexts_that_recorded_it()
 {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider).with_extensions(vec![Box::new(Sky)]);
    f.write("sky.md", "rain");
    assert!(!core.extension_on("sky"));
    send(&core, &provider, "one").await;
    // Turned on mid-context, the context didn't record it: still no line.
    core.set_extension("sky", true).unwrap();
    send(&core, &provider, "two").await;
    assert!(time_blocks(&provider).iter().all(|t| !t.contains("Sky:")));

    core.start_fresh_context(&clock()).unwrap();
    send(&core, &provider, "three").await;
    let before = provider.requests().pop().unwrap();
    f.write("sky.md", "sun");
    send(&core, &provider, "four").await;
    let blocks = time_blocks(&provider);
    assert!(
        blocks[0].ends_with(")\nSky: rain.</context>"),
        "{}",
        blocks[0]
    );
    assert!(
        blocks[1].ends_with(")\nSky: sun.</context>"),
        "{}",
        blocks[1]
    );
    // Principle III: the earlier message is sent exactly as before.
    let after = provider.requests().pop().unwrap();
    assert_eq!(before.system, after.system);
    assert_eq!(before.messages[..], after.messages[..before.messages.len()]);

    core.set_extension("sky", false).unwrap();
    send(&core, &provider, "five").await;
    assert!(!time_blocks(&provider).last().unwrap().contains("Sky:"));
}
