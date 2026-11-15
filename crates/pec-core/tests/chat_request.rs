//! The request each turn sends (Principle III). Within a context, every request is the previous one plus
//! appended messages, and a reply is replayed the same way every time (chat R7).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink, clock, tool_use};
use pec_core::Core;
use pec_core::llm::TurnRequest;
use serde_json::json;
use std::time::Duration;

async fn send(core: &Core, provider: &FakeProvider, text: &str) {
    let sink = VecSink::default();
    assert_eq!(
        core.send_message(provider, text, &clock(), &sink).await,
        Ok(None)
    );
}

/// Sends a message and stops its reply once the reply has started streaming or hanging.
async fn send_and_stop(core: &Core, provider: &FakeProvider, text: &str) {
    let (sink, now) = (VecSink::default(), clock());
    let stop = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        core.stop_reply();
    };
    let (sent, ()) = tokio::join!(core.send_message(provider, text, &now, &sink), stop);
    assert_eq!(sent, Ok(None));
}

/// chat SC-003, run by hand in release mode (chat checks). Ignored in CI: performance budgets are a
/// release-readiness check (`test-audit`). Includes saving both entries, so it is an upper bound.
#[tokio::test]
#[ignore = "release-readiness measurement"]
async fn preparing_a_request_on_10000_messages() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    drop(f.ready(&provider));
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/make-transcript.mjs"
    );
    let made = std::process::Command::new("node")
        .args([script, &f.workspace("").display().to_string(), "10000"])
        .status()
        .unwrap();
    assert!(made.success());
    let core = f.core();
    let mut times = Vec::new();
    for i in 0..100 {
        let started = std::time::Instant::now();
        send(&core, &provider, &format!("message {i}")).await;
        times.push(started.elapsed());
    }
    times.sort();
    println!("95th percentile: {:?}", times[94]);
}

fn assert_extends(previous: &TurnRequest, next: &TurnRequest) {
    assert_eq!(previous.system, next.system);
    assert_eq!(previous.model, next.model);
    assert_eq!(previous.tools, next.tools);
    assert_eq!(
        previous.messages[..],
        next.messages[..previous.messages.len()]
    );
}

#[tokio::test]
async fn each_request_extends_the_previous_one() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    for text in ["one", "two", "three"] {
        send(&core, &provider, text).await;
    }
    let requests = provider.requests();
    assert_eq!(requests.len(), 3);
    let system = &requests[0].system;
    assert!(system[0].starts_with("# How you work"));
    assert!(
        system[1].starts_with("## IDENTITY.md")
            && system[1].contains("## SOUL.md\n\n# Soul\n\nBe warm.")
    );
    assert!(system[2].contains("- Call them: Sam") && system[2].contains("## MEMORY.md"));
    assert_eq!(
        requests[0].messages[0]["content"],
        json!([
            { "type": "text", "text": "<context>Local time: Friday 13 November 2026, 21:40 (Asia/Jakarta, UTC+07:00)</context>" },
            { "type": "text", "text": "one" },
        ])
    );
    assert_eq!(
        requests[1].messages[1],
        json!({ "role": "assistant", "content": [{ "type": "text", "text": "Hello!" }] })
    );
    for pair in requests.windows(2) {
        assert_extends(&pair[0], &pair[1]);
        assert_eq!(pair[1].messages.len(), pair[0].messages.len() + 2);
    }
}

#[tokio::test]
async fn cut_off_replaced_and_fallback_replies_are_replayed_the_same_way_every_time() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let thinking = json!({ "type": "thinking", "thinking": "", "signature": "sig" });
    provider.script(vec![
        // 1: stopped after some text.
        vec![Step::Text("Part"), Step::Hang],
        // 2: stopped before any text.
        vec![Step::Hang],
        // 3: a complete reply with thinking.
        vec![Step::Blocks(vec![
            thinking.clone(),
            json!({ "type": "text", "text": "A" }),
        ])],
        // 4: its regeneration, served after a fallback.
        vec![Step::Blocks(vec![
            thinking.clone(),
            json!({ "type": "fallback", "from": {}, "to": {} }),
            json!({ "type": "text", "text": "B" }),
        ])],
        // 5: fails after some text.
        vec![
            Step::Text("Par"),
            Step::Fail(pec_core::llm::ProviderError::Unreachable, None),
        ],
    ]);
    send_and_stop(&core, &provider, "one").await;
    send_and_stop(&core, &provider, "two").await;
    send(&core, &provider, "three").await;
    let sink = VecSink::default();
    assert_eq!(
        core.regenerate_reply(&provider, &clock(), &sink).await,
        Ok(None)
    );
    send(&core, &provider, "four").await;
    assert_eq!(core.retry_reply(&provider, &clock(), &sink).await, Ok(None));

    let r = provider.requests();
    // The stopped reply is replayed as its text; the one stopped before any text is left out.
    assert_eq!(
        r[2].messages[1],
        json!({ "role": "assistant", "content": [{ "type": "text", "text": "Part" }] })
    );
    assert_eq!(r[2].messages[2]["role"], "user");
    assert_eq!(r[2].messages[3]["role"], "user");
    // Regenerate sends the request that produced the replaced reply.
    assert_eq!(r[3], r[2]);
    // The next request carries the new reply, minus the thinking before the fallback.
    assert_eq!(
        r[4].messages[4],
        json!({ "role": "assistant", "content": [
            { "type": "fallback", "from": {}, "to": {} },
            { "type": "text", "text": "B" },
        ] })
    );
    // Retrying an incomplete reply replaces it and sends the same request again.
    assert_eq!(r[5], r[4]);
    for pair in [(&r[0], &r[1]), (&r[1], &r[2]), (&r[2], &r[4])] {
        assert_extends(pair.0, pair.1);
    }
}

#[tokio::test]
async fn notes_tool_turns_and_notices_still_only_append() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write(
        "memory/2026-11-12.md",
        "# 2026-11-12\n\n- 09:00 Yesterday's note.\n",
    );
    let today: String = (0..200)
        .map(|i| format!("- 10:{:02} Note number {i} about the day.\n", i % 60))
        .collect();
    f.write("memory/2026-11-13.md", &format!("# 2026-11-13\n\n{today}"));
    // The user edits their profile while the companion writes its second reply.
    let profile = f.workspace("USER.md");
    let edit = move || {
        std::fs::write(
            &profile,
            "# About the user\n\n- Call them: Sam\n- Lives in: Bandung\n",
        )
        .unwrap()
    };
    provider.script(vec![
        vec![Step::Text("Hi."), Step::End("end_turn")],
        vec![
            Step::Do(std::sync::Arc::new(edit)),
            Step::Tools(vec![tool_use(
                "t1",
                "memory_note",
                json!({ "text": "Tea." }),
            )]),
        ],
        vec![Step::Text("Noted."), Step::End("end_turn")],
        vec![Step::Text("Ok."), Step::End("end_turn")],
    ]);
    for text in ["one", "two", "three"] {
        send(&core, &provider, text).await;
    }

    let r = provider.requests();
    assert_eq!(r.len(), 4);
    // memory FR-014, FR-015: the context opens with the last two days' notes, today's cut to its most recent part.
    let opener = r[0].messages[0]["content"][0]["text"].as_str().unwrap();
    assert!(opener.starts_with("<memory>Your daily notes from yesterday and today"));
    assert!(opener.contains("Yesterday's note."));
    assert!(opener.contains("(Earlier notes from this day are left out.)"));
    assert!(opener.contains("Note number 199 ") && !opener.contains("Note number 0 "));
    assert_eq!(
        core.chat_state().unwrap().shortened,
        ["memory/2026-11-13.md"]
    );
    // memory FR-007: exactly the five memory tools, the same in every request; none writes the soul.
    let names: Vec<_> = r[0].tools.iter().map(|t| t.name).collect();
    assert_eq!(
        names,
        [
            "memory_search",
            "memory_get",
            "memory_note",
            "memory_remember",
            "profile_update"
        ]
    );
    // Including the tool-result message that also carries the notice for the user's edit.
    for pair in r.windows(2) {
        assert_extends(&pair[0], &pair[1]);
    }
}

#[tokio::test]
async fn a_recall_block_keeps_requests_append_only() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write(
        "memory/2026-10-23.md",
        "- 20:00 Bimo hides when it thunders.\n",
    );
    for text in [
        "Bimo and the thunder again",
        "Thunder tonight, Bimo hid",
        "Goodnight",
    ] {
        send(&core, &provider, text).await;
    }
    let r = provider.requests();
    assert!(
        r[0].messages[0]["content"][1]["text"]
            .as_str()
            .unwrap()
            .starts_with("<memory>")
    );
    for pair in r.windows(2) {
        assert_extends(&pair[0], &pair[1]);
    }
}

/// compaction SC-007: within each context every request extends the previous one, across two compactions,
/// and a context a compaction opened starts with no thinking or tool blocks from before it (chat R8).
#[tokio::test]
async fn requests_stay_append_only_across_compactions() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.set_auto_recall(false).unwrap();
    let thinking = json!({ "type": "thinking", "thinking": "", "signature": "sig" });
    let summary = json!({ "thread": "Chatting.", "commitments": [], "relationship": "Warm.", "facts": [], "notes": [] });
    for round in 0..2 {
        for (i, size) in [12_000, 20_000, 28_000, 36_000, 44_000]
            .into_iter()
            .enumerate()
        {
            provider.script(vec![
                vec![Step::Tools(vec![
                    thinking.clone(),
                    tool_use(
                        "n",
                        "memory_note",
                        json!({ "text": format!("note {round} {i}") }),
                    ),
                ])],
                vec![
                    Step::Usage(json!({ "input_tokens": size })),
                    Step::Blocks(vec![
                        thinking.clone(),
                        json!({ "type": "text", "text": "Sure." }),
                    ]),
                ],
            ]);
            send(&core, &provider, &format!("message {round} {i}")).await;
        }
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
    }
    send(&core, &provider, "after").await;
    send(&core, &provider, "again").await;

    // Chat requests only, grouped by the context they were sent in.
    let mut contexts: Vec<Vec<TurnRequest>> = Vec::new();
    let mut seen = 0;
    for (request, file) in provider.requests.lock().unwrap().iter() {
        let flush = serde_json::to_string(&request.messages)
            .unwrap()
            .contains("about to leave your view");
        if request.format.is_some() || flush {
            continue;
        }
        let context = file.matches("\"type\":\"context\"").count();
        if context != seen || contexts.is_empty() {
            contexts.push(Vec::new());
            seen = context;
        }
        contexts.last_mut().unwrap().push(request.clone());
    }
    assert_eq!(contexts.len(), 3);
    for requests in &contexts {
        for pair in requests.windows(2) {
            assert_extends(&pair[0], &pair[1]);
        }
    }
    for requests in &contexts[1..] {
        let first = serde_json::to_string(&requests[0].messages).unwrap();
        for block in ["\"thinking\"", "\"tool_use\"", "\"tool_result\""] {
            assert!(!first.contains(block), "{block}");
        }
    }
}
