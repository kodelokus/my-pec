//! Compaction on a temporary workspace with the fake provider: it starts at a turn boundary, keeps the recent
//! turns, never overlaps a turn, and changes no file when it doesn't finish (compaction R17).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink, clock, tool_use};
use pec_core::chat::{ChatEvent, MessageKind};
use pec_core::compaction::CompactOutcome;
use pec_core::llm::{ProviderError, TurnRequest};
use pec_core::memory::ForgetTarget;
use pec_core::{AppError, Core};
use serde_json::{Value, json};
use std::sync::Arc;

fn usage(tokens: u64) -> Step {
    Step::Usage(json!({ "input_tokens": tokens }))
}

/// One finished turn whose reply reports `tokens` as the context's size.
async fn turn(core: &Core, provider: &FakeProvider, text: &str, tokens: u64) {
    provider.script(vec![vec![
        usage(tokens),
        Step::Text("Sure."),
        Step::End("end_turn"),
    ]]);
    core.send_message(provider, text, &clock(), &VecSink::default())
        .await
        .unwrap();
}

fn summary_json(thread: &str) -> Value {
    json!({
        "thread": thread,
        "commitments": ["Suggest three restaurants."],
        "relationship": "Warm.",
        "facts": ["Flight TP1350 on 14 March at 07:10."],
        "notes": [],
    })
}

fn summary_reply(thread: &str) -> Vec<Step> {
    vec![Step::Blocks(vec![
        json!({ "type": "text", "text": summary_json(thread).to_string() }),
    ])]
}

/// The provider calls of one compaction: a flush that notes nothing, then the summary.
fn compaction(summary: Vec<Step>) -> Vec<Vec<Step>> {
    vec![
        vec![Step::Text("Nothing new."), Step::End("end_turn")],
        summary,
    ]
}

/// Five turns; the last reaches 43,000 of the 48,000 working size, past the 42,000 threshold.
async fn long_conversation(core: &Core, provider: &FakeProvider) {
    for (text, tokens) in [
        ("one", 12_000),
        ("two", 22_000),
        ("three", 32_000),
        ("four", 39_000),
        ("five", 43_000),
    ] {
        turn(core, provider, text, tokens).await;
    }
}

async fn compact(core: &Core, provider: &FakeProvider, manual: bool) -> CompactOutcome {
    core.compact_conversation(provider, &clock(), None, manual)
        .await
        .unwrap()
        .outcome
}

fn summary_requests(provider: &FakeProvider) -> Vec<TurnRequest> {
    provider
        .requests()
        .into_iter()
        .filter(|r| r.format.is_some())
        .collect()
}

fn texts(request: &TurnRequest) -> String {
    serde_json::to_string(&request.messages).unwrap()
}

#[tokio::test]
async fn a_conversation_past_its_working_size_compacts_and_goes_on() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.set_auto_recall(false).unwrap();
    turn(&core, &provider, "one", 12_000).await;
    assert!(!core.chat_state().unwrap().compaction_due);
    // Not due yet: nothing happens and nothing is written.
    let before = f.read("sessions/main.jsonl");
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::NothingToCompact
    );
    assert_eq!(f.read("sessions/main.jsonl"), before);

    for (text, tokens) in [
        ("two", 22_000),
        ("three", 32_000),
        ("four", 39_000),
        ("five", 43_000),
    ] {
        turn(&core, &provider, text, tokens).await;
    }
    assert!(
        core.chat_state().unwrap().compaction_due,
        "compaction FR-002"
    );
    // The profile changed while the context ran; the new context starts from it (compaction FR-019).
    f.write("USER.md", "# About the user\n\n- Call them: Sammy\n");
    let before = f.read("sessions/main.jsonl");
    provider.script(compaction(summary_reply("Planning the Lisbon trip.")));
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Compacted
    );

    // compaction SC-004: only appended to, two entries in one go.
    let after = f.read("sessions/main.jsonl");
    assert!(after.starts_with(&before));
    let entries = f.transcript();
    let (record, opened) = (&entries[entries.len() - 2], &entries[entries.len() - 1]);
    assert_eq!(record["type"], "compaction");
    assert_eq!(opened["type"], "context");
    assert_eq!(opened["compaction"], record["id"]);
    let five = entries.iter().find(|e| e["text"] == "five").unwrap();
    assert_eq!(
        record["firstKept"], five["id"],
        "compaction FR-014: the turns that fit 8,000 tokens"
    );
    assert_eq!(
        (
            record["tokensBefore"].as_u64(),
            record["keptTokens"].as_u64()
        ),
        (Some(43_000), Some(4_000))
    );
    assert_eq!(record["model"], "claude-opus-5-5");
    assert!(
        record["usage"]["summary"].is_null(),
        "the fake reply reported no usage"
    );
    assert!(
        record["summary"]
            .as_str()
            .unwrap()
            .starts_with("## Where we are\nPlanning the Lisbon trip.")
    );
    assert!(
        opened["files"]["USER.md"]
            .as_str()
            .unwrap()
            .contains("Sammy")
    );

    // The summary request saw the older turns as text, not the kept one.
    let request = summary_requests(&provider).pop().unwrap();
    assert!(request.tools.is_empty() && request.system.len() == 1);
    let input = texts(&request);
    assert!(input.contains("Sammy: one") && input.contains("Steve: Sure."));
    assert!(!input.contains("Sammy: five"));

    let state = core.chat_state().unwrap();
    assert!(!state.compaction_due, "no reply yet in the new context");

    // The next request: the summary opener, the kept turn as text, then the new message, with no notice.
    turn(&core, &provider, "six", 15_000).await;
    let request = provider.requests().pop().unwrap();
    assert!(
        request.messages[0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Planning the Lisbon trip.")
    );
    assert_eq!(
        request.messages[1]["content"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["text"],
        "five"
    );
    assert_eq!(
        request.messages[2],
        json!({ "role": "assistant", "content": [{ "type": "text", "text": "Sure." }] })
    );
    assert!(!texts(&request).contains("<notice>"));
    assert!(request.system[2].contains("Sammy"));
    assert!(!core.chat_state().unwrap().compaction_due);

    // compaction FR-025: a divider with the summary just before the first kept message, and every message still there.
    let shown = core.chat_history(None, 200).unwrap().messages;
    let divider = shown
        .iter()
        .position(|m| m.kind == MessageKind::Compacted)
        .unwrap();
    assert!(shown[divider].text.contains("Planning the Lisbon trip."));
    assert_eq!(shown[divider + 1].text, "five");
    assert_eq!(
        shown.iter().filter(|m| m.kind == MessageKind::User).count(),
        6
    );
    assert!(!shown.iter().any(|m| m.kind == MessageKind::Divider));
}

#[tokio::test]
async fn thirty_days_of_conversation_never_outgrow_the_working_size() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.set_auto_recall(false).unwrap();
    // compaction SC-008: each turn adds 3,000 tokens to a context that starts at 15,000 after each compaction.
    let (mut size, mut compactions) = (12_000, 0);
    for day in 0..90 {
        turn(&core, &provider, &format!("turn {day}"), size).await;
        assert!(size <= 48_000 + 3_000, "{size}");
        if core.chat_state().unwrap().compaction_due {
            provider.script(compaction(summary_reply("Still chatting.")));
            assert_eq!(
                compact(&core, &provider, false).await,
                CompactOutcome::Compacted
            );
            compactions += 1;
            size = 15_000;
        } else {
            size += 3_000;
        }
    }
    assert!(compactions >= 8, "{compactions}");
}

#[tokio::test]
async fn a_message_sent_during_a_compaction_stops_it_and_saves_nothing() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    long_conversation(&core, &provider).await;
    let before = f.read("sessions/main.jsonl");
    // compaction SC-006: the compaction's first call never ends by itself; the message doesn't wait for it.
    provider.script(vec![vec![Step::Hang]]);
    provider.script(vec![vec![Step::Text("Right away."), Step::End("end_turn")]]);
    let (sink, now) = (VecSink::default(), clock());
    let (compacted, sent) = tokio::join!(
        compact(&core, &provider, false),
        core.send_message(&provider, "still there?", &now, &sink)
    );
    assert_eq!(compacted, CompactOutcome::Stopped);
    assert_eq!(sent, Ok(None));
    assert_eq!(sink.deltas(), "Right away.");
    let after = f.read("sessions/main.jsonl");
    assert!(after.starts_with(&before));
    assert!(!after.contains("\"compaction\""), "compaction FR-007");
    // Answered in the old context.
    assert!(texts(&provider.requests().pop().unwrap()).contains("\"five\""));
}

#[tokio::test]
async fn a_line_appended_meanwhile_stops_it() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = Arc::new(f.ready(&provider));
    long_conversation(&core, &provider).await;
    let other = core.clone();
    let mut steps = vec![Step::Do(Arc::new(move || {
        other.start_fresh_context(&clock()).unwrap();
    }))];
    steps.extend(summary_reply("Too late."));
    provider.script(compaction(steps));
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Stopped
    );
    assert!(!f.read("sessions/main.jsonl").contains("Too late."));
}

#[tokio::test]
async fn a_compaction_that_fails_saves_nothing() {
    let invalid = vec![Step::Blocks(vec![
        json!({ "type": "text", "text": "{\"thread\":\"\"}" }),
    ])];
    let refused = vec![Step::Text("{}"), Step::End("refusal")];
    for (case, script, expected) in [
        (
            "checks",
            vec![invalid.clone(), invalid],
            CompactOutcome::Failed { error: None },
        ),
        (
            "refusal",
            vec![refused.clone(), refused],
            CompactOutcome::Failed { error: None },
        ),
        (
            "provider",
            vec![vec![Step::Fail(ProviderError::Unreachable, None)]],
            CompactOutcome::Failed {
                error: Some(ProviderError::Unreachable),
            },
        ),
    ] {
        let f = Fixture::new();
        let provider = FakeProvider::accepting();
        let core = f.ready(&provider);
        long_conversation(&core, &provider).await;
        let before = f.read("sessions/main.jsonl");
        for steps in script {
            provider.script(compaction(steps));
        }
        assert_eq!(compact(&core, &provider, false).await, expected, "{case}");
        assert_eq!(
            f.read("sessions/main.jsonl"),
            before,
            "{case}: compaction FR-007"
        );
        // compaction FR-023: still due, so the next turn boundary tries again.
        assert!(core.chat_state().unwrap().compaction_due, "{case}");
    }
}

#[tokio::test]
async fn a_compaction_waits_for_a_turn_boundary() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    long_conversation(&core, &provider).await;
    let busy = Err(AppError::InvalidInput {
        field: "busy".into(),
    });
    // compaction FR-005: not while a reply is being written.
    provider.script(vec![vec![Step::Hang]]);
    let (sink, now) = (VecSink::default(), clock());
    let (sent, compacted) = tokio::join!(core.send_message(&provider, "six", &now, &sink), async {
        tokio::task::yield_now().await;
        let result = core
            .compact_conversation(&provider, &clock(), None, true)
            .await;
        core.stop_reply();
        result
    });
    assert_eq!(sent, Ok(None));
    assert_eq!(compacted.map(|r| r.outcome), busy);

    // Not while a reply is owed: a cut-off reply asks to be retried first.
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    long_conversation(&core, &provider).await;
    provider.script(vec![vec![
        usage(44_000),
        Step::Text("cut"),
        Step::End("max_tokens"),
    ]]);
    core.send_message(&provider, "six", &clock(), &VecSink::default())
        .await
        .unwrap();
    assert!(!core.chat_state().unwrap().compaction_due);
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::NothingToCompact
    );
}

#[tokio::test]
async fn forgotten_turns_never_reach_the_summary_or_the_kept_turns() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.set_auto_recall(false).unwrap();
    let remember = |text: &'static str, tokens: u64| {
        vec![
            vec![Step::Tools(vec![tool_use(
                "r",
                "memory_remember",
                json!({ "text": text, "topic": "Facts", "requested_by_user": true }),
            )])],
            vec![usage(tokens), Step::Text("Noted."), Step::End("end_turn")],
        ]
    };
    provider.script(remember("Allergic to peanuts.", 12_000));
    core.send_message(
        &provider,
        "Remember: Allergic to peanuts.",
        &clock(),
        &VecSink::default(),
    )
    .await
    .unwrap();
    for (text, tokens) in [("two", 22_000), ("three", 32_000), ("four", 39_000)] {
        turn(&core, &provider, text, tokens).await;
    }
    provider.script(remember("Likes green tea.", 43_000));
    core.send_message(
        &provider,
        "Remember: Likes green tea.",
        &clock(),
        &VecSink::default(),
    )
    .await
    .unwrap();
    let forget = |text: &str| {
        let doc = core.memory_read("MEMORY.md").unwrap();
        let entry = doc.entries.iter().find(|e| e.text == text).unwrap();
        core.memory_forget(
            "MEMORY.md",
            ForgetTarget::Entry {
                id: entry.id.clone().unwrap(),
            },
            &doc.hash,
        )
        .unwrap()
    };
    forget("Allergic to peanuts.");

    // No flush while the companion still sees something forgotten: the summary is the only call (compaction R13).
    provider.script(vec![summary_reply("Talking about tea.")]);
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Compacted
    );
    let record = f
        .transcript()
        .into_iter()
        .rfind(|e| e["type"] == "compaction")
        .unwrap();
    assert_eq!(record["flush"]["outcome"], "skipped");
    let input = texts(&summary_requests(&provider).pop().unwrap());
    assert!(!input.contains("peanuts"), "compaction FR-016");
    assert!(input.contains("Sam: two"));
    turn(&core, &provider, "next", 15_000).await;
    let request = provider.requests().pop().unwrap();
    assert!(!texts(&request).contains("peanuts"));
    assert!(texts(&request).contains("green tea"));

    // Forgetting what only a kept turn holds still says the companion sees it (compaction R13).
    assert!(forget("Likes green tea.").in_current_context);
}

fn note_call(text: &str) -> Value {
    tool_use("n", "memory_note", json!({ "text": text }))
}

fn flush_saving(texts: &[&str]) -> Vec<Step> {
    vec![Step::Tools(texts.iter().map(|t| note_call(t)).collect())]
}

const NOTE: &str = "memory/2026-11-13.md";

fn last_compaction(f: &Fixture) -> Value {
    f.transcript()
        .into_iter()
        .rfind(|e| e["type"] == "compaction")
        .unwrap()
}

/// The pending marker; none when `state.json` was never written.
fn pending(f: &Fixture) -> Value {
    std::fs::read(f.root().join("state/state.json")).map_or(Value::Null, |b| {
        serde_json::from_slice::<Value>(&b).unwrap()["compaction"]["pending"].clone()
    })
}

fn set_pending(f: &Fixture, id: Option<&str>) {
    let mut state = f.state();
    state["compaction"] = json!({ "pending": id });
    f.write("../state/state.json", &state.to_string());
}

#[tokio::test]
async fn the_flush_saves_new_notes_once_with_their_source() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.set_auto_recall(false).unwrap();
    long_conversation(&core, &provider).await;
    let chat = provider.requests().pop().unwrap();
    provider.script(vec![
        flush_saving(&["Rina visits next Tuesday.", "Rina visits next Tuesday."]),
        summary_reply("Planning."),
    ]);
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Compacted
    );

    // The flush request is the next chat request plus the notice, so the cache is read (compaction R4).
    let flush = provider.requests().into_iter().rev().nth(1).unwrap();
    assert_eq!(
        (&flush.system, &flush.tools, &flush.model, flush.effort),
        (&chat.system, &chat.tools, &chat.model, chat.effort)
    );
    assert!(flush.format.is_none());
    let mut expected = chat.messages.clone();
    expected.push(json!({ "role": "assistant", "content": [{ "type": "text", "text": "Sure." }] }));
    assert_eq!(flush.messages[..expected.len()], expected[..]);
    assert_eq!(flush.messages.len(), expected.len() + 1);
    assert!(texts(&flush).contains("about to leave your view"));

    // compaction FR-010, FR-011: once, with its origin and source; recorded with the compaction.
    let entries = f.transcript();
    let five = entries.iter().find(|e| e["text"] == "five").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let line = format!("- 21:40 Rina visits next Tuesday. <!-- from:companion src:{five} -->");
    assert_eq!(f.read(NOTE).matches(&line).count(), 1);
    assert_eq!(
        last_compaction(&f)["flush"],
        json!({ "outcome": "saved", "notes": [{ "time": "21:40", "text": "Rina visits next Tuesday." }] })
    );
    assert!(
        entries.last().unwrap()["notes"]["2026-11-13"]
            .as_str()
            .unwrap()
            .contains(&line)
    );
    assert!(pending(&f).is_null());
    // compaction FR-009: nothing of it in the conversation.
    let shown = serde_json::to_string(&core.chat_history(None, 200).unwrap().messages).unwrap();
    assert!(!shown.contains("Rina"));

    // A second compaction offered the same fact writes nothing new.
    for (text, tokens) in [("six", 25_000), ("seven", 35_000), ("eight", 43_000)] {
        turn(&core, &provider, text, tokens).await;
    }
    provider.script(vec![
        flush_saving(&["Rina visits next Tuesday."]),
        summary_reply("Later."),
    ]);
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Compacted
    );
    assert_eq!(f.read(NOTE).matches("Rina visits next Tuesday.").count(), 1);
    assert_eq!(last_compaction(&f)["flush"]["outcome"], "none");
}

#[tokio::test]
async fn a_flush_that_fails_still_compacts_and_an_unfinished_one_saves_nothing() {
    for (case, flush, outcome) in [
        (
            "failed",
            vec![Step::Fail(ProviderError::Unreachable, None)],
            "failed",
        ),
        (
            "refused",
            vec![Step::Text("no"), Step::End("refusal")],
            "failed",
        ),
        (
            "nothing",
            vec![Step::Text("All noted."), Step::End("end_turn")],
            "none",
        ),
    ] {
        let f = Fixture::new();
        let provider = FakeProvider::accepting();
        let core = f.ready(&provider);
        long_conversation(&core, &provider).await;
        provider.script(vec![flush, summary_reply("Planning.")]);
        assert_eq!(
            compact(&core, &provider, false).await,
            CompactOutcome::Compacted,
            "{case}"
        );
        assert_eq!(last_compaction(&f)["flush"]["outcome"], outcome, "{case}");
    }

    // compaction FR-012: notes from a flush whose summary then fails are not saved.
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    long_conversation(&core, &provider).await;
    let invalid = vec![Step::Blocks(vec![json!({ "type": "text", "text": "{}" })])];
    provider.script(vec![
        flush_saving(&["Rina visits."]),
        invalid.clone(),
        invalid,
    ]);
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Failed { error: None }
    );
    assert!(!f.workspace(NOTE).exists());
    assert!(pending(&f).is_null());
}

#[tokio::test]
async fn a_crash_between_the_commit_and_the_notes_is_finished_once() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    long_conversation(&core, &provider).await;
    provider.script(vec![
        flush_saving(&["Rina visits."]),
        summary_reply("Planning."),
    ]);
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Compacted
    );
    let id = last_compaction(&f)["id"].as_str().unwrap().to_owned();
    let with_note = f.read(NOTE);
    let without: String = with_note
        .lines()
        .filter(|l| !l.contains("Rina"))
        .map(|l| format!("{l}\n"))
        .collect();

    // The app stopped after the commit: the next launch adds the note once and clears the marker.
    f.write(NOTE, &without);
    set_pending(&f, Some(&id));
    f.core().chat_state().unwrap();
    assert_eq!(f.read(NOTE), with_note);
    assert!(pending(&f).is_null());

    // Principle I: a note deleted by hand after a clean compaction is never put back.
    f.write(NOTE, &without);
    f.core().chat_state().unwrap();
    assert_eq!(f.read(NOTE), without);

    // A compaction that is not fully on disk adds nothing; the marker is cleared.
    let stray = r#"{"id":"9999999999999-0","at":1,"type":"compaction","summary":"s","firstKept":"x","tokensBefore":1,"keptTokens":0,"model":"m","usage":{},"flush":{"outcome":"saved","notes":[{"time":"22:00","text":"Never."}]}}"#;
    f.write(
        "sessions/main.jsonl",
        &format!("{}{stray}\n", f.read("sessions/main.jsonl")),
    );
    set_pending(&f, Some("9999999999999-0"));
    f.core().chat_state().unwrap();
    assert!(!f.read(NOTE).contains("Never."));
    assert!(pending(&f).is_null());
}

fn too_long() -> Vec<Step> {
    vec![Step::Fail(ProviderError::ContextTooLong, None)]
}

#[tokio::test]
async fn too_long_on_a_first_call_compacts_and_retries_once() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    for (text, tokens) in [("one", 12_000), ("two", 22_000)] {
        turn(&core, &provider, text, tokens).await;
    }
    // compaction FR-022: compacted inline with the new message kept, then answered, with no error shown.
    provider.script(vec![too_long()]);
    provider.script(compaction(summary_reply("Two short turns.")));
    provider.script(vec![vec![Step::Text("Got it."), Step::End("end_turn")]]);
    let sink = VecSink::default();
    core.send_message(&provider, "a long paste", &clock(), &sink)
        .await
        .unwrap();
    let events = sink.events();
    assert!(events.contains(&ChatEvent::Compacting));
    assert!(matches!(events.last(), Some(ChatEvent::Finished { .. })));
    assert_eq!(sink.deltas(), "Got it.");
    let entries = f.transcript();
    let paste = entries
        .iter()
        .find(|e| e["text"] == "a long paste")
        .unwrap();
    assert_eq!(last_compaction(&f)["firstKept"], paste["id"]);
    let retried = provider.requests().pop().unwrap();
    assert!(texts(&retried).contains("Two short turns."));
    assert_eq!(retried.messages.last().unwrap()["role"], "user");
    assert!(!core.chat_state().unwrap().context_full);
}

#[tokio::test]
async fn too_long_again_or_mid_turn_fails_plainly_and_keeps_the_message() {
    // The retry fails too.
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    turn(&core, &provider, "one", 12_000).await;
    provider.script(vec![too_long()]);
    provider.script(compaction(summary_reply("One turn.")));
    provider.script(vec![too_long()]);
    let sink = VecSink::default();
    core.send_message(&provider, "huge", &clock(), &sink)
        .await
        .unwrap();
    assert!(matches!(
        sink.last(),
        ChatEvent::Failed {
            error: ProviderError::ContextTooLong,
            ..
        }
    ));
    let state = core.chat_state().unwrap();
    assert!(state.context_full && state.pending_retry.is_some());
    let shown = core.chat_history(None, 200).unwrap().messages;
    assert_eq!(shown.last().unwrap().text, "huge");
    core.start_fresh_context(&clock()).unwrap();
    assert!(!core.chat_state().unwrap().context_full);

    // Nothing to compact: the first message of a context is already too long.
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    provider.script(vec![too_long()]);
    let sink = VecSink::default();
    core.send_message(&provider, "huge", &clock(), &sink)
        .await
        .unwrap();
    assert!(core.chat_state().unwrap().context_full);
    assert!(!f.read("sessions/main.jsonl").contains("\"compaction\""));

    // Mid-turn, after a tool round: fails without compacting.
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    turn(&core, &provider, "one", 12_000).await;
    provider.script(vec![flush_saving(&["A note."]), too_long()]);
    let sink = VecSink::default();
    core.send_message(&provider, "note it", &clock(), &sink)
        .await
        .unwrap();
    assert!(!sink.events().contains(&ChatEvent::Compacting));
    assert!(core.chat_state().unwrap().context_full);
}

#[tokio::test]
async fn compact_now_takes_a_focus_note_and_says_when_there_is_nothing_to_compact() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    for (text, tokens) in [("one", 12_000), ("two", 14_000), ("three", 16_000)] {
        turn(&core, &provider, text, tokens).await;
    }
    let invalid = |field: &str| {
        Err(AppError::InvalidInput {
            field: field.into(),
        })
    };
    let long = "x".repeat(501);
    for (focus, manual) in [(Some(long.as_str()), true), (Some("the formula"), false)] {
        assert_eq!(
            core.compact_conversation(&provider, &clock(), focus, manual)
                .await
                .map(|r| r.outcome),
            invalid("focus")
        );
    }

    // compaction FR-024: below the working size, on request, with the focus note in the summary's input and the record.
    provider.script(compaction(summary_reply("The formula.")));
    let result = core
        .compact_conversation(
            &provider,
            &clock(),
            Some("  the formula we settled on "),
            true,
        )
        .await
        .unwrap();
    assert_eq!(result.outcome, CompactOutcome::Compacted);
    assert!(
        texts(&summary_requests(&provider).pop().unwrap())
            .contains("Focus from Sam: the formula we settled on")
    );
    assert_eq!(last_compaction(&f)["focus"], "the formula we settled on");

    // Right after, nothing is older than the kept turns.
    let before = f.read("sessions/main.jsonl");
    assert_eq!(
        compact(&core, &provider, true).await,
        CompactOutcome::NothingToCompact
    );
    assert_eq!(f.read("sessions/main.jsonl"), before);
}

#[tokio::test]
async fn settings_choose_the_working_size_and_the_model_that_compacts() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.model_list(&provider, true).await.unwrap();
    let invalid = |field: &str| {
        Err(AppError::InvalidInput {
            field: field.into(),
        })
    };
    for budget in [31_000, 201_000, 40_500] {
        assert_eq!(
            core.set_compaction_settings(budget, None),
            invalid("budgetTokens"),
            "{budget}"
        );
    }
    assert_eq!(
        core.set_compaction_settings(48_000, Some("nope".into())),
        invalid("model")
    );
    let defaults = core.compaction_settings().unwrap();
    assert_eq!(
        (
            defaults.budget_tokens,
            defaults.min_budget,
            defaults.max_budget,
            defaults.model
        ),
        (48_000, 32_000, 200_000, None)
    );

    // compaction FR-003: a 32,000 working size makes a 26,000-token context due.
    let saved = core
        .set_compaction_settings(32_000, Some("claude-sonnet-5-5".into()))
        .unwrap();
    assert_eq!(saved.budget_tokens, 32_000);
    for (text, tokens) in [("one", 12_000), ("two", 20_000), ("three", 26_000)] {
        turn(&core, &provider, text, tokens).await;
    }
    assert!(core.chat_state().unwrap().compaction_due);

    // compaction FR-027: the flush and the summary use the compaction model; chat keeps its own.
    provider.script(compaction(summary_reply("Short.")));
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Compacted
    );
    let requests = provider.requests();
    let n = requests.len();
    assert_eq!(requests[n - 2].model, "claude-sonnet-5-5");
    assert_eq!(requests[n - 1].model, "claude-sonnet-5-5");
    assert_eq!(last_compaction(&f)["model"], "claude-sonnet-5-5");
    turn(&core, &provider, "four", 15_000).await;
    assert_eq!(provider.requests().pop().unwrap().model, "claude-opus-5-5");

    // A compaction model no longer offered: the chat model is used, and settings say so.
    provider.set(Ok(common::models(&["claude-opus-5-5"])));
    core.model_list(&provider, true).await.unwrap();
    assert!(core.compaction_settings().unwrap().model_missing);
    for (text, tokens) in [("five", 22_000), ("six", 27_000)] {
        turn(&core, &provider, text, tokens).await;
    }
    provider.script(compaction(summary_reply("Shorter.")));
    assert_eq!(
        compact(&core, &provider, false).await,
        CompactOutcome::Compacted
    );
    assert_eq!(last_compaction(&f)["model"], "claude-opus-5-5");
}

#[tokio::test]
async fn switching_the_chat_model_compacts_onto_it() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.model_list(&provider, true).await.unwrap();
    long_conversation(&core, &provider).await;
    core.select_model("claude-sonnet-5-5").unwrap();
    assert!(core.chat_state().unwrap().model_change_pending);

    // compaction FR-028: "switch now" compacts; the new context and its replies use the new model.
    provider.script(compaction(summary_reply("Switching.")));
    assert_eq!(
        compact(&core, &provider, true).await,
        CompactOutcome::Compacted
    );
    assert_eq!(f.transcript().last().unwrap()["model"], "claude-sonnet-5-5");
    assert!(!core.chat_state().unwrap().model_change_pending);
    turn(&core, &provider, "six", 15_000).await;
    let request = provider.requests().pop().unwrap();
    assert_eq!(request.model, "claude-sonnet-5-5");
    assert!(texts(&request).contains("Switching."));
}
