//! The conversation against a scripted provider on a temporary workspace (chat interfaces).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink, clock, tool_use};
use pec_core::Core;
use pec_core::chat::{ChatEvent, MessageKind, ReplyState, RetryReason, SendRefusal};
use pec_core::llm::ProviderError;
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn a_message_is_on_disk_before_it_is_sent_and_the_reply_is_saved_as_received() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    provider.script(vec![vec![
        Step::Text("Hi "),
        Step::Text("Sam"),
        Step::End("end_turn"),
    ]]);
    let sink = VecSink::default();

    assert_eq!(
        core.send_message(&provider, "hello", &clock(), &sink).await,
        Ok(None)
    );

    let at_send = &provider.requests.lock().unwrap()[0].1;
    assert!(at_send.contains(r#""text":"hello""#), "chat FR-013");
    assert_eq!(sink.deltas(), "Hi Sam");
    let ChatEvent::Finished {
        message: Some(reply),
        ..
    } = sink.last()
    else {
        panic!("{:?}", sink.last())
    };
    assert_eq!(
        (reply.text.as_str(), reply.state),
        ("Hi Sam", Some(ReplyState::Complete))
    );
    let entries = f.transcript();
    assert_eq!(entries[0]["type"], "context");
    assert_eq!(entries[1]["text"], "hello");
    assert_eq!(entries[1]["zone"], "Asia/Jakarta");
    assert_eq!(
        entries[2]["content"],
        json!([{ "type": "text", "text": "Hi Sam" }])
    );
    let page = core.chat_history(None, 50).unwrap();
    let shown: Vec<_> = page
        .messages
        .iter()
        .map(|m| (m.kind, m.text.as_str()))
        .collect();
    assert_eq!(
        shown,
        [
            (MessageKind::User, "hello"),
            (MessageKind::Companion, "Hi Sam")
        ]
    );
}

#[tokio::test]
async fn a_refused_message_writes_nothing() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let sink = VecSink::default();
    let too_long = "a".repeat(32_001);
    for (text, refusal) in [
        (" \n\t", SendRefusal::Empty),
        (too_long.as_str(), SendRefusal::TooLong),
    ] {
        assert_eq!(
            core.send_message(&provider, text, &clock(), &sink).await,
            Ok(Some(refusal))
        );
    }

    provider.script(vec![vec![Step::Hang]]);
    let now = clock();
    let second = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        let busy = core
            .send_message(&provider, "second", &clock(), &sink)
            .await;
        core.stop_reply();
        busy
    };
    let (_, busy) = tokio::join!(core.send_message(&provider, "first", &now, &sink), second);
    assert_eq!(busy, Ok(Some(SendRefusal::Busy)));

    f.store.delete_key();
    assert_eq!(
        core.send_message(&provider, "third", &clock(), &sink).await,
        Ok(Some(SendRefusal::NoKey))
    );
    let typed: Vec<_> = f
        .transcript()
        .iter()
        .filter_map(|e| e["text"].as_str().map(String::from))
        .collect();
    assert_eq!(typed, ["first"]);
}

async fn send(core: &Core, provider: &FakeProvider, text: &str) -> VecSink {
    let sink = VecSink::default();
    assert_eq!(
        core.send_message(provider, text, &clock(), &sink).await,
        Ok(None)
    );
    sink
}

/// Runs a command that starts a reply and stops it after a short while.
async fn stopping<F: std::future::Future>(core: &Core, run: F) -> F::Output {
    let stop = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        core.stop_reply();
    };
    tokio::join!(run, stop).0
}

fn shown(core: &Core) -> Vec<(MessageKind, String)> {
    let page = core.chat_history(None, 200).unwrap();
    page.messages
        .into_iter()
        .map(|m| (m.kind, m.text))
        .collect()
}

#[tokio::test]
async fn the_conversation_survives_a_restart_and_an_interrupted_reply_is_offered_again() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    send(&f.ready(&provider), &provider, "remember the blue door").await;

    let core = f.core();
    assert_eq!(shown(&core).len(), 2);
    send(&core, &provider, "what colour?").await;
    let request = provider.requests().pop().unwrap();
    assert_eq!(
        request.messages.len(),
        3,
        "chat FR-009: earlier turns are sent"
    );

    provider.script(vec![vec![Step::Hang]]);
    let (now, sink) = (clock(), VecSink::default());
    stopping(&core, core.send_message(&provider, "and now?", &now, &sink))
        .await
        .unwrap();
    assert_eq!(
        f.core().chat_state().unwrap().pending_retry,
        Some(RetryReason::Interrupted),
        "chat FR-017"
    );
}

#[tokio::test]
async fn a_half_written_line_is_skipped_and_the_file_is_never_rewritten() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    send(&f.ready(&provider), &provider, "one").await;
    let path = f.workspace("sessions/main.jsonl");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.extend_from_slice(br#"{"type":"mess"#);
    std::fs::write(&path, &bytes).unwrap();

    let core = f.core();
    assert_eq!(core.chat_state().unwrap().skipped_lines, 1, "chat FR-018");
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    send(&core, &provider, "two").await;
    assert!(
        std::fs::read(&path).unwrap().starts_with(&bytes),
        "only appended to"
    );
    assert_eq!(f.core().chat_state().unwrap().skipped_lines, 1);
    assert_eq!(shown(&f.core()).len(), 4);
}

#[tokio::test]
async fn a_fresh_context_starts_from_the_files_on_the_chosen_model() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    send(&core, &provider, "one").await;
    // Reaching the working size and a too-long conversation belong to compaction (compaction_flow.rs).
    f.write(
        "../state/settings.json",
        r#"{ "chat": { "model": "claude-sonnet-5-5" } }"#,
    );
    assert!(
        core.chat_state().unwrap().model_change_pending,
        "chat FR-032"
    );
    let state = core.start_fresh_context(&clock()).unwrap();
    assert!(!state.model_change_pending);
    send(&core, &provider, "two").await;
    let request = provider.requests().pop().unwrap();
    assert_eq!(request.model, "claude-sonnet-5-5");
    assert_eq!(request.messages.len(), 1, "chat FR-031: no earlier turns");
    assert_eq!(shown(&core)[2].0, MessageKind::Divider);
}

#[tokio::test]
async fn catching_up_with_nothing_to_compact_carries_the_turns_over_as_text() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    send(&core, &provider, "My flight is TP1350.").await;
    f.write(
        "../state/settings.json",
        r#"{ "chat": { "model": "claude-sonnet-5-5" } }"#,
    );
    let state = core.start_context_carrying(&clock()).unwrap();
    assert!(!state.model_change_pending);
    send(&core, &provider, "Which flight?").await;
    let request = provider.requests().pop().unwrap();
    assert_eq!(request.model, "claude-sonnet-5-5");
    // The earlier turn comes along, as text only like a compaction's kept turns (Principle III).
    assert_eq!(request.messages.len(), 3);
    let sent = serde_json::to_string(&request.messages).unwrap();
    assert!(sent.contains("My flight is TP1350."), "{sent}");
    assert!(!sent.contains("\"thinking\""), "{sent}");
}

#[tokio::test]
async fn the_first_hello_is_sent_once_and_waits_for_a_working_connection() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write("../state/state.json", r#"{ "firstHelloPending": true }"#);
    provider.script(vec![
        vec![Step::Fail(ProviderError::Unreachable, None)],
        vec![Step::Text("Hi Sam, I am Steve."), Step::End("end_turn")],
    ]);
    let sink = VecSink::default();
    assert_eq!(
        core.start_first_hello(&provider, &clock(), &sink).await,
        Ok(None)
    );
    let state = core.chat_state().unwrap();
    assert!(state.first_hello_pending, "a failed greeting stays pending");
    assert!(state.pending_retry.is_some());

    assert_eq!(core.retry_reply(&provider, &clock(), &sink).await, Ok(None));
    assert!(
        !core.chat_state().unwrap().first_hello_pending,
        "chat FR-021"
    );
    assert_eq!(
        shown(&core),
        [(MessageKind::Companion, "Hi Sam, I am Steve.".into())]
    );
    let hello = provider.requests()[0].messages[0]["content"][1]["text"].clone();
    assert!(
        hello
            .as_str()
            .unwrap()
            .contains("They want to be called Sam")
    );
    let hidden = f
        .transcript()
        .iter()
        .filter(|e| e["provenance"] == "system")
        .count();
    assert_eq!(hidden, 1);
    assert!(
        core.start_first_hello(&provider, &clock(), &sink)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn speaking_first_cancels_the_greeting() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write("../state/state.json", r#"{ "firstHelloPending": true }"#);
    send(&core, &provider, "hi").await;
    assert!(!core.chat_state().unwrap().first_hello_pending);
    let sink = VecSink::default();
    assert!(
        core.start_first_hello(&provider, &clock(), &sink)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn stop_ends_a_reply_at_once_and_regenerate_replaces_the_last_reply() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let now = clock();
    provider.script(vec![
        vec![Step::Text("Part"), Step::Hang],
        vec![Step::Hang],
        vec![Step::Fail(
            ProviderError::Busy,
            Some(Duration::from_secs(30)),
        )],
    ]);

    let sink = VecSink::default();
    let started = std::time::Instant::now();
    stopping(&core, core.send_message(&provider, "one", &now, &sink))
        .await
        .unwrap();
    assert!(started.elapsed() < Duration::from_secs(1), "chat SC-006");
    let ChatEvent::Finished {
        message: Some(stopped),
        ..
    } = sink.last()
    else {
        panic!("{:?}", sink.last())
    };
    assert_eq!(
        (stopped.text.as_str(), stopped.state),
        ("Part", Some(ReplyState::Stopped))
    );

    let sink = VecSink::default();
    stopping(&core, core.send_message(&provider, "two", &now, &sink))
        .await
        .unwrap();
    assert!(matches!(
        sink.last(),
        ChatEvent::Finished { message: None, .. }
    ));
    assert!(core.chat_state().unwrap().pending_retry.is_some());

    // Stop also ends the wait before an automatic retry.
    let sink = VecSink::default();
    let started = std::time::Instant::now();
    stopping(&core, core.retry_reply(&provider, &clock(), &sink))
        .await
        .unwrap();
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(sink.events().contains(&ChatEvent::Waiting {
        seconds: 30,
        text: String::new()
    }));

    send(&core, &provider, "three").await;
    provider.script(vec![vec![Step::Text("Better"), Step::End("end_turn")]]);
    let sink = VecSink::default();
    assert_eq!(
        core.regenerate_reply(&provider, &clock(), &sink).await,
        Ok(None)
    );
    let messages = shown(&core);
    assert_eq!(
        messages.last().unwrap(),
        &(MessageKind::Companion, "Better".into()),
        "chat FR-023"
    );
    assert_eq!(
        messages[messages.len() - 2],
        (MessageKind::User, "three".into())
    );
    assert_eq!(
        f.transcript()
            .iter()
            .filter(|e| e["type"] == "replaced")
            .count(),
        1
    );
}

#[tokio::test(start_paused = true)]
async fn a_busy_provider_is_retried_for_a_minute_and_other_failures_keep_the_message() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let busy = || vec![Step::Fail(ProviderError::Busy, None)];
    provider.script(vec![
        vec![Step::Text("Once"), Step::Fail(ProviderError::Busy, None)],
        busy(),
        busy(),
        busy(),
        busy(),
        vec![Step::Text("Done"), Step::End("end_turn")],
    ]);
    let sink = send(&core, &provider, "story").await;
    let waits: Vec<u32> = sink
        .events()
        .iter()
        .filter_map(|e| match e {
            ChatEvent::Waiting { seconds, .. } => Some(*seconds),
            _ => None,
        })
        .collect();
    assert_eq!(waits, [1, 2, 4, 8, 16], "chat FR-025");
    assert_eq!(
        shown(&core).last().unwrap().1,
        "Done",
        "the partial was replaced"
    );

    let wait = |s| Some(Duration::from_secs(s));
    provider.script(vec![
        vec![Step::Fail(ProviderError::Busy, wait(50))],
        vec![Step::Fail(ProviderError::Busy, wait(20))],
    ]);
    let sink = send(&core, &provider, "again").await;
    assert!(matches!(
        sink.last(),
        ChatEvent::Failed {
            error: ProviderError::Busy,
            ..
        }
    ));

    for error in [
        ProviderError::Unreachable,
        ProviderError::KeyRejected,
        ProviderError::NoCredit,
        ProviderError::Invalid,
    ] {
        provider.script(vec![vec![Step::Fail(error, None)]]);
        let sink = send(&core, &provider, "kept").await;
        assert!(
            matches!(sink.last(), ChatEvent::Failed { error: e, partial: None, .. } if e == error)
        );
        assert_eq!(
            core.chat_state().unwrap().pending_retry,
            Some(RetryReason::Failed { error }),
            "chat FR-024"
        );
        assert_eq!(shown(&core).last().unwrap().1, "kept");
    }

    provider.script(vec![vec![
        Step::Text("I would rather not"),
        Step::End("refusal"),
    ]]);
    let sink = send(&core, &provider, "declined").await;
    assert!(matches!(
        sink.last(),
        ChatEvent::Failed {
            error: ProviderError::Declined,
            ..
        }
    ));
    assert_eq!(
        shown(&core).last().unwrap().1,
        "declined",
        "a refusal's partial is discarded"
    );

    provider.script(vec![
        vec![
            Step::Text("Half"),
            Step::Fail(ProviderError::Unreachable, None),
        ],
        vec![Step::Text("Cut"), Step::End("max_tokens")],
    ]);
    let sink = send(&core, &provider, "partial").await;
    let ChatEvent::Failed {
        partial: Some(partial),
        ..
    } = sink.last()
    else {
        panic!("{:?}", sink.last())
    };
    assert_eq!(partial.state, Some(ReplyState::Incomplete), "chat FR-027");
    assert_eq!(core.retry_reply(&provider, &clock(), &sink).await, Ok(None));
    let last = core.chat_history(None, 2).unwrap().messages;
    assert_eq!(
        (last[0].text.as_str(), last[1].text.as_str()),
        ("partial", "Cut")
    );
    assert_eq!(last[1].state, Some(ReplyState::Incomplete));
}

#[tokio::test]
async fn an_edited_file_reaches_the_next_message_as_a_notice() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    send(&core, &provider, "one").await;
    f.write("SOUL.md", "# Soul\n\nSpeak like a pirate.");
    let soul = std::fs::read(f.workspace("SOUL.md")).unwrap();
    send(&core, &provider, "two").await;
    send(&core, &provider, "three").await;

    let r = provider.requests();
    assert_eq!(r[1].system, r[0].system, "Principle III");
    let notice = r[1].messages.last().unwrap()["content"][1]["text"].clone();
    let notice = notice.as_str().unwrap();
    assert!(notice.starts_with("<notice>The user edited SOUL.md."));
    assert!(notice.ends_with("Speak like a pirate."));
    assert_eq!(r[1].messages.last().unwrap()["content"][2]["text"], "two");
    let third = r[2].messages.last().unwrap()["content"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(third, 2, "no second notice without a second edit");
    assert_eq!(
        std::fs::read(f.workspace("SOUL.md")).unwrap(),
        soul,
        "chat FR-011"
    );
    assert_eq!(shown(&core)[2], (MessageKind::Note, "SOUL.md".into()));

    f.write("MEMORY.md", &"m".repeat(12_001));
    send(&core, &provider, "four").await;
    assert_eq!(
        core.chat_state().unwrap().shortened,
        ["MEMORY.md"],
        "chat FR-012"
    );
    assert_eq!(f.read("MEMORY.md").len(), 12_001);
}

fn note(text: &str) -> serde_json::Value {
    tool_use("n1", "memory_note", json!({ "text": text }))
}

#[tokio::test]
async fn a_turn_ends_after_eight_tool_calls() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let search = || {
        vec![Step::Tools(vec![tool_use(
            "s",
            "memory_search",
            json!({ "query": "tea" }),
        )])]
    };
    let mut script: Vec<_> = (0..8).map(|_| search()).collect();
    script.push(vec![Step::Text("Done."), Step::End("end_turn")]);
    provider.script(script);
    send(&core, &provider, "hi").await;

    let requests = provider.requests();
    assert_eq!(requests.len(), 9);
    let result = |i: usize| requests[i].messages.last().unwrap()["content"][0]["content"].clone();
    assert!(!result(7).as_str().unwrap().contains("too many times"));
    assert!(
        result(8).as_str().unwrap().contains("too many times"),
        "{}",
        result(8)
    );
}

#[tokio::test]
async fn stop_during_a_tool_turn_ends_it_and_keeps_what_was_written() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    provider.script(vec![
        vec![Step::Tools(vec![note("Tea.")])],
        vec![Step::Hang],
    ]);
    let sink = VecSink::default();
    stopping(&core, core.send_message(&provider, "hi", &clock(), &sink))
        .await
        .unwrap();

    let ChatEvent::Finished {
        message: Some(turn),
        state,
    } = sink.last()
    else {
        panic!("{:?}", sink.last())
    };
    assert_eq!(turn.memory.len(), 1);
    assert!(!state.replying);
    assert_eq!(state.pending_retry, Some(RetryReason::Interrupted));
    assert!(f.read("memory/2026-11-13.md").contains("Tea."));
}

#[tokio::test]
async fn a_tool_interrupted_by_a_crash_is_reported_not_run_again() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    drop(f.ready(&provider));
    // The app ended right after saving a reply that asked for a tool, before the tool ran.
    let core = f.core();
    send(&core, &provider, "hi").await;
    let asked = json!({
        "id": "9999999999999-0", "at": 9_999_999_999_999u64, "type": "message", "role": "assistant",
        "provenance": "companion", "content": [note("Never written.")], "state": "complete",
        "stop_reason": "tool_use"
    });
    let file = f.workspace("sessions/main.jsonl");
    let mut text = std::fs::read_to_string(&file).unwrap();
    text.push_str(&format!("{asked}\n"));
    std::fs::write(&file, text).unwrap();

    let core = f.core();
    assert_eq!(
        core.chat_state().unwrap().pending_retry,
        Some(RetryReason::Interrupted)
    );
    provider.script(vec![vec![Step::Text("Sorry."), Step::End("end_turn")]]);
    let sink = VecSink::default();
    assert_eq!(core.retry_reply(&provider, &clock(), &sink).await, Ok(None));

    let request = provider.requests().pop().unwrap();
    let answer = &request.messages.last().unwrap()["content"][0];
    assert_eq!(answer["tool_use_id"], "n1");
    assert_eq!(answer["is_error"], true);
    assert!(answer["content"].as_str().unwrap().contains("interrupted"));
    assert!(!f.workspace("memory/2026-11-13.md").exists());
}

#[tokio::test]
async fn a_tool_turn_is_one_message_and_regenerate_replaces_all_of_it() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    provider.script(vec![
        vec![
            Step::Text("Let me note that."),
            Step::Tools(vec![
                json!({ "type": "text", "text": "Let me note that." }),
                note("Tea."),
            ]),
        ],
        vec![Step::Text("Done."), Step::End("end_turn")],
        vec![Step::Text("Better."), Step::End("end_turn")],
    ]);
    let sink = send(&core, &provider, "hi").await;
    assert_eq!(sink.deltas(), "Let me note that.\n\nDone.");
    assert_eq!(
        shown(&core),
        [
            (MessageKind::User, "hi".into()),
            (MessageKind::Companion, "Let me note that.\n\nDone.".into())
        ]
    );

    let sink = VecSink::default();
    assert_eq!(
        core.regenerate_reply(&provider, &clock(), &sink).await,
        Ok(None)
    );
    assert_eq!(
        shown(&core),
        [
            (MessageKind::User, "hi".into()),
            (MessageKind::Companion, "Better.".into())
        ]
    );
    // The new reply answers the same request as the replaced turn's first call.
    let requests = provider.requests();
    assert_eq!(requests[2], requests[0]);
}

#[tokio::test]
async fn a_context_from_before_memory_keeps_its_rules_until_a_fresh_one_starts() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    drop(f.ready(&provider));
    f.write(
        "sessions/main.jsonl",
        "{\"id\":\"1-0\",\"at\":1,\"type\":\"context\",\"model\":\"claude-opus-5-5\",\"files\":{\"SOUL.md\":\"s\"},\"harness\":\"chat-1\"}\n",
    );
    let core = f.core();
    assert!(core.chat_state().unwrap().context_outdated);
    send(&core, &provider, "one").await;
    let old = provider.requests().pop().unwrap();
    assert!(old.tools.is_empty());
    assert!(!old.system[0].contains("# Your memory"));

    core.start_fresh_context(&clock()).unwrap();
    assert!(!core.chat_state().unwrap().context_outdated);
    send(&core, &provider, "two").await;
    let new = provider.requests().pop().unwrap();
    assert_eq!(new.tools.len(), 5);
    assert!(new.system[0].contains("# Your memory"));
}

#[tokio::test]
async fn relevant_memory_is_attached_to_the_message_before_it_is_sent() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write(
        "memory/2026-10-23.md",
        "# 2026-10-23\n\n- 20:00 Bimo hides under the bed when it thunders.\n",
    );
    let storm = "There's a big storm and Bimo is scared of the thunder";
    let last_content =
        |p: &FakeProvider| p.requests().pop().unwrap().messages.last().unwrap()["content"].clone();

    send(&core, &provider, storm).await;
    let (request, file) = provider.requests.lock().unwrap()[0].clone();
    let content = &request.messages.last().unwrap()["content"];
    let recall = content[1]["text"].as_str().unwrap();
    assert!(recall.starts_with("<memory>Possibly relevant notes from your memory"));
    assert!(recall.contains("2026-10-23 (memory/2026-10-23.md, line 3): - 20:00 Bimo hides"));
    assert_eq!(content[2]["text"], storm);
    // memory FR-021: marked as recalled, and on disk before the request went out.
    assert!(file.contains(r#""recalled":1"#));

    send(&core, &provider, "What a lovely morning").await;
    assert_eq!(
        last_content(&provider).as_array().unwrap().len(),
        2,
        "nothing relevant"
    );

    // memory FR-020: switched off, nothing is attached, and the companion can still search.
    assert!(!core.set_auto_recall(false).unwrap().auto_recall);
    provider.script(vec![
        vec![Step::Tools(vec![tool_use(
            "s",
            "memory_search",
            json!({ "query": "Bimo" }),
        )])],
        vec![Step::Text("He hides."), Step::End("end_turn")],
    ]);
    send(&core, &provider, storm).await;
    let requests = provider.requests();
    let sent = &requests[requests.len() - 2].messages.last().unwrap()["content"];
    assert_eq!(sent.as_array().unwrap().len(), 2);
    let searched = &requests[requests.len() - 1].messages.last().unwrap()["content"][0]["content"];
    assert!(searched.as_str().unwrap().contains("Bimo hides"));
}

#[tokio::test]
async fn chat_works_when_the_index_cannot_be_used() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    // A folder where the index should be: it can be neither opened nor deleted.
    std::fs::create_dir_all(f.root().join("state/pec.db")).unwrap();
    f.write(
        "memory/2026-10-23.md",
        "- 20:00 Bimo hides when it thunders.\n",
    );
    provider.script(vec![
        vec![Step::Tools(vec![tool_use(
            "s",
            "memory_search",
            json!({ "query": "Bimo" }),
        )])],
        vec![
            Step::Text("I can't look that up right now."),
            Step::End("end_turn"),
        ],
    ]);

    let sink = send(&core, &provider, "Bimo and the thunder").await;

    // chat FR-024: no recall, an explained tool error, and a finished reply.
    assert!(matches!(
        sink.last(),
        ChatEvent::Finished {
            message: Some(_),
            ..
        }
    ));
    let requests = provider.requests();
    let sent = &requests[0].messages.last().unwrap()["content"];
    assert_eq!(sent.as_array().unwrap().len(), 2);
    let result = &requests[1].messages.last().unwrap()["content"][0];
    assert_eq!(result["is_error"], true);
    assert_eq!(result["content"], "Memory search is unavailable right now.");
}
