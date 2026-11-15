//! The memory tools end to end: the companion's calls, the files they write, and what the user sees
//! (memory interfaces, memory R3 to R7).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink, clock, tool_use};
use pec_core::chat::ChatEvent;
use pec_core::memory::MarkKind;
use serde_json::{Value, json};

fn marks(sink: &VecSink) -> Vec<(MarkKind, bool)> {
    sink.events()
        .into_iter()
        .filter_map(|e| match e {
            ChatEvent::MemoryWritten { mark } => Some((mark.kind, mark.ok)),
            _ => None,
        })
        .collect()
}

/// The blocks of the last message in a request.
fn last_blocks(request: &pec_core::llm::TurnRequest) -> Vec<Value> {
    request.messages.last().unwrap()["content"]
        .as_array()
        .unwrap()
        .clone()
}

#[tokio::test]
async fn the_write_tools_save_memory_and_show_it_on_the_reply() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    provider.script(vec![
        vec![Step::Tools(vec![
            tool_use(
                "t1",
                "memory_remember",
                json!({ "text": "Allergic to peanuts.", "topic": "Health", "requested_by_user": true }),
            ),
            tool_use(
                "t2",
                "profile_update",
                json!({ "key": "call them", "value": "Sammy" }),
            ),
            tool_use(
                "t3",
                "memory_note",
                json!({ "text": "Rina visits next week." }),
            ),
        ])],
        vec![Step::Text("Got it."), Step::End("end_turn")],
    ]);
    let sink = VecSink::default();
    let said = "Remember that I'm allergic to peanuts. And call me Sammy.";
    assert_eq!(
        core.send_message(&provider, said, &clock(), &sink).await,
        Ok(None)
    );

    // memory FR-005, FR-002: in long-term memory with its date, origin and source, before the turn ends.
    let memory = f.read("MEMORY.md");
    assert!(
        memory.contains("## Health\n- Allergic to peanuts. <!-- id:m_"),
        "{memory}"
    );
    assert!(memory.contains("at:2026-11-13 from:user src:"), "{memory}");
    // memory FR-006: the old value is gone.
    let profile = f.read("USER.md");
    assert!(profile.contains("- Call them: Sammy\n") && !profile.contains("Sam\n"));
    // memory FR-004, FR-003: today's note by the user's clock.
    assert!(
        f.read("memory/2026-11-13.md")
            .contains("- 21:40 Rina visits next week. <!-- from:companion src:")
    );
    let requests = provider.requests.lock().unwrap().clone();
    assert!(
        requests[1].1.contains(r#""type":"written""#),
        "saved before the next call"
    );

    // memory FR-008: each write is shown, live and on the saved reply.
    assert_eq!(
        marks(&sink),
        [
            (MarkKind::Remembered, true),
            (MarkKind::Profile, true),
            (MarkKind::Note, true)
        ]
    );
    let ChatEvent::Finished {
        message: Some(reply),
        ..
    } = sink.last()
    else {
        panic!("{:?}", sink.last())
    };
    assert_eq!(reply.text, "Got it.");
    assert_eq!(reply.memory.len(), 3);
    assert_eq!(core.chat_history(None, 10).unwrap().messages[1], reply);

    // memory R3: one untrusted message answers every call, in order.
    let results: Vec<_> = last_blocks(&requests[1].0)
        .iter()
        .map(|b| b["tool_use_id"].clone())
        .collect();
    assert_eq!(results, [json!("t1"), json!("t2"), json!("t3")]);
    let entries = f.transcript();
    let tool_message = entries
        .iter()
        .find(|e| e["provenance"] == "untrusted")
        .unwrap();
    assert_eq!(tool_message["marks"].as_array().unwrap().len(), 3);
    let written: Vec<_> = entries
        .iter()
        .filter(|e| e["type"] == "written")
        .map(|e| e["file"].as_str().unwrap())
        .collect();
    assert_eq!(written, ["MEMORY.md", "USER.md"]);

    // memory FR-009: each rewrite kept the file it replaced.
    let db = rusqlite::Connection::open(f.root().join("state/pec.db")).unwrap();
    let kept: Vec<String> = db
        .prepare("SELECT path FROM preimages ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(kept, ["MEMORY.md", "USER.md"]);
}

#[tokio::test]
async fn a_failed_or_full_write_is_reported_and_the_turn_goes_on() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    // Long-term memory can't be read or written; the profile is full.
    std::fs::remove_file(f.workspace("MEMORY.md")).unwrap();
    std::fs::create_dir(f.workspace("MEMORY.md")).unwrap();
    f.write("USER.md", &format!("- Notes: {}\n", "y".repeat(3_990)));
    provider.script(vec![
        vec![Step::Tools(vec![
            tool_use(
                "t1",
                "memory_remember",
                json!({ "text": "Likes tea.", "topic": "Food", "requested_by_user": false }),
            ),
            tool_use(
                "t2",
                "profile_update",
                json!({ "key": "Lives in", "value": "Bandung" }),
            ),
            tool_use("t3", "memory_note", json!({ "text": "two\nlines" })),
        ])],
        vec![Step::Text("Done."), Step::End("end_turn")],
    ]);
    let sink = VecSink::default();
    assert_eq!(
        core.send_message(&provider, "hi", &clock(), &sink).await,
        Ok(None)
    );

    // memory FR-012, FR-011: a failure and a fallback are both shown; an invalid input writes nothing and shows nothing.
    assert_eq!(
        marks(&sink),
        [(MarkKind::Remembered, false), (MarkKind::KeptInNote, true)]
    );
    let errors: Vec<_> = last_blocks(&provider.requests()[1])
        .iter()
        .map(|b| b["is_error"].as_bool().unwrap_or(false))
        .collect();
    assert_eq!(errors, [true, false, true]);
    assert!(f.read("memory/2026-11-13.md").contains("Lives in: Bandung"));
    assert!(
        matches!(sink.last(), ChatEvent::Finished { message: Some(m), .. } if m.text == "Done.")
    );
}

#[tokio::test]
async fn a_user_edit_is_kept_and_announced_and_the_companions_own_write_is_not() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let sink = VecSink::default();
    core.send_message(&provider, "one", &clock(), &sink)
        .await
        .unwrap();
    // The user edits long-term memory while the companion is writing its reply.
    let file = f.workspace("MEMORY.md");
    let edit = move || std::fs::write(&file, "# Memory\n\n## Mine\n- My own line.\n").unwrap();
    provider.script(vec![
        vec![
            Step::Do(std::sync::Arc::new(edit)),
            Step::Tools(vec![tool_use(
                "t1",
                "memory_remember",
                json!({ "text": "Likes tea.", "topic": "Food", "requested_by_user": true }),
            )]),
        ],
        vec![Step::Text("Noted."), Step::End("end_turn")],
    ]);
    core.send_message(&provider, "two", &clock(), &sink)
        .await
        .unwrap();
    core.send_message(&provider, "three", &clock(), &sink)
        .await
        .unwrap();

    // memory FR-010: the companion's write went on top of the user's edit.
    let memory = f.read("MEMORY.md");
    assert!(memory.starts_with("# Memory\n\n## Mine\n- My own line.\n\n## Food\n- Likes tea."));
    let requests = provider.requests();
    // memory R7: the user's edit reaches the companion after the tool results...
    let blocks = last_blocks(&requests[2]);
    assert_eq!(blocks[0]["type"], "tool_result");
    assert!(
        blocks[1]["text"]
            .as_str()
            .unwrap()
            .starts_with("<notice>The user edited MEMORY.md.")
    );
    // ...and the companion's own write brings no notice afterwards (memory FR-016).
    let next = serde_json::to_string(&last_blocks(&requests[3])).unwrap();
    assert!(!next.contains("<notice>"), "{next}");
}

#[tokio::test]
async fn search_and_read_answer_with_places_and_lines() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write(
        "memory/2026-10-23.md",
        "# 2026-10-23\n\n- 20:00 Bimo hides under the bed when it thunders.\n- 20:05 Second line.\n",
    );
    provider.script(vec![
        vec![Step::Tools(vec![
            tool_use("s1", "memory_search", json!({ "query": "Bimo thunder" })),
            tool_use("s2", "memory_search", json!({ "query": "zebra" })),
            tool_use(
                "g1",
                "memory_get",
                json!({ "path": "memory/2026-10-23.md", "from_line": 3, "to_line": 4 }),
            ),
        ])],
        vec![Step::Text("ok"), Step::End("end_turn")],
    ]);
    core.send_message(&provider, "hi", &clock(), &VecSink::default())
        .await
        .unwrap();

    let results: Vec<_> = last_blocks(&provider.requests()[1])
        .iter()
        .map(|b| b["content"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        results,
        [
            "- 2026-10-23 (memory/2026-10-23.md, line 3): - 20:00 Bimo hides under the bed when it thunders.",
            "No matches.",
            "3: - 20:00 Bimo hides under the bed when it thunders.\n4: - 20:05 Second line.",
        ]
    );
}

/// dreaming R1: recall evidence rows, as dreaming reads them.
fn recalls(f: &Fixture) -> Vec<(String, i64, String, f64, String, String)> {
    let db = rusqlite::Connection::open(f.root().join("state/pec.db")).unwrap();
    db.prepare(
        "SELECT path, line, query_hash, coverage, day, source FROM recall_events ORDER BY id",
    )
    .unwrap()
    .query_map([], |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            r.get(4)?,
            r.get(5)?,
        ))
    })
    .unwrap()
    .collect::<rusqlite::Result<_>>()
    .unwrap()
}

#[tokio::test]
async fn recalled_daily_notes_are_kept_as_evidence_without_the_query() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write(
        "memory/2026-11-01.md",
        "# 2026-11-01\n\n- 09:00 Training for the half marathon in April. <!-- from:companion src:1-0 -->\n",
    );
    // Today's note is in the context already; it still counts as relevant to the message.
    f.write(
        "memory/2026-11-13.md",
        "# 2026-11-13\n\n- 09:00 Half marathon shoes arrived. <!-- from:companion src:2-0 -->\n",
    );
    f.write(
        "MEMORY.md",
        "# Memory\n\n## Running\n- Half marathon training plan. <!-- id:m_1 at:2026-11-01 from:user src:1-0 -->\n",
    );
    core.send_message(
        &provider,
        "half marathon training",
        &clock(),
        &VecSink::default(),
    )
    .await
    .unwrap();
    let rows = recalls(&f);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows.iter().any(|r| r.0 == "memory/2026-11-13.md"));
    let (path, line, query, coverage, day, source) =
        rows.iter().find(|r| r.0 == "memory/2026-11-01.md").unwrap();
    assert_eq!(
        (path.as_str(), *line, day.as_str(), source.as_str()),
        ("memory/2026-11-01.md", 3, "2026-11-13", "auto")
    );
    assert!((coverage - 1.0).abs() < 1e-9);
    assert!(
        !query.contains("marathon"),
        "a fingerprint, never the query"
    );

    // The companion's own search records its daily-note hits too, never MEMORY.md.
    provider.script(vec![
        vec![Step::Tools(vec![tool_use(
            "s",
            "memory_search",
            json!({ "query": "marathon shoes" }),
        )])],
        vec![Step::Text("Found them."), Step::End("end_turn")],
    ]);
    core.send_message(&provider, "what did I buy?", &clock(), &VecSink::default())
        .await
        .unwrap();
    let searched: Vec<_> = recalls(&f)
        .into_iter()
        .filter(|r| r.5 == "search")
        .collect();
    assert!(!searched.is_empty());
    assert!(searched.iter().all(|r| r.0.starts_with("memory/")));
}
