//! The memory browser's commands on a temporary workspace: list, read, save, undo and forget
//! (memory interfaces, memory R13 to R15).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink, clock, tool_use};
use pec_core::memory::{FileKind, ForgetTarget, SaveOutcome, UndoOutcome};
use pec_core::{AppError, Core};
use serde_json::json;

fn invalid(field: &str) -> AppError {
    AppError::InvalidInput {
        field: field.into(),
    }
}

/// One turn in which the companion updates the profile.
async fn companion_sets(core: &Core, provider: &FakeProvider, key: &str, value: &str) {
    provider.script(vec![
        vec![Step::Tools(vec![tool_use(
            "p",
            "profile_update",
            json!({ "key": key, "value": value }),
        )])],
        vec![Step::Text("Done."), Step::End("end_turn")],
    ]);
    core.send_message(provider, "hi", &clock(), &VecSink::default())
        .await
        .unwrap();
}

#[test]
fn files_are_listed_read_and_saved_only_over_what_was_read() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write(
        "MEMORY.md",
        "# Memory\n\n## Health\n- Allergic to peanuts. <!-- id:m_8f3a at:2026-11-14 from:user src:1-0 -->\n",
    );
    f.write("memory/2026-11-01.md", "- 09:00 One.\n");
    f.write("memory/2026-11-03.md", "- 09:00 Three.\n");
    f.write("memory/notes.md", "Not a daily note.\n");
    std::fs::write(f.workspace("memory/2026-11-02.md"), [0xff, 0xfe]).unwrap();

    let files = core.memory_files().unwrap();
    let listed: Vec<_> = files
        .iter()
        .map(|f| (f.path.as_str(), f.kind, f.budget, f.readable))
        .collect();
    assert_eq!(
        listed,
        [
            ("SOUL.md", FileKind::Soul, Some(8_000), true),
            ("USER.md", FileKind::Profile, Some(4_000), true),
            ("MEMORY.md", FileKind::Memory, Some(12_000), true),
            ("DREAMS.md", FileKind::Diary, None, true),
            ("memory/2026-11-03.md", FileKind::Daily, None, true),
            ("memory/2026-11-02.md", FileKind::Daily, None, false),
            ("memory/2026-11-01.md", FileKind::Daily, None, true),
        ]
    );
    assert_eq!(
        core.memory_read("memory/2026-11-02.md"),
        Err(invalid("unreadable"))
    );
    for path in ["IDENTITY.md", "memory/notes.md", "memory/2026-12-01.md"] {
        assert_eq!(core.memory_read(path), Err(invalid("path")), "{path}");
    }

    let doc = core.memory_read("MEMORY.md").unwrap();
    assert_eq!(doc.entries.len(), 1);
    assert_eq!(doc.entries[0].text, "Allergic to peanuts.");
    assert_eq!(doc.entries[0].id.as_deref(), Some("m_8f3a"));
    assert!(!doc.can_undo);

    // memory FR-025: the user's text, byte for byte.
    let typed = "# Memory\n\n## Health\n- Allergic to peanuts and shellfish.\n";
    let SaveOutcome::Saved { hash } = core.memory_save("MEMORY.md", typed, &doc.hash).unwrap()
    else {
        panic!("saved")
    };
    assert_eq!(f.read("MEMORY.md"), typed);
    // memory FR-029: a save from a stale copy writes nothing and hands back the current text.
    assert_eq!(
        core.memory_save("MEMORY.md", "older copy", &doc.hash)
            .unwrap(),
        SaveOutcome::Conflict {
            text: typed.into(),
            hash: hash.clone()
        }
    );
    assert_eq!(f.read("MEMORY.md"), typed);
}

#[tokio::test]
async fn undo_steps_back_through_the_companions_changes_byte_for_byte() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let original = f.read("USER.md");
    companion_sets(&core, &provider, "Call them", "Sammy").await;
    let first = f.read("USER.md");
    companion_sets(&core, &provider, "Lives in", "Bandung").await;
    assert!(core.memory_read("USER.md").unwrap().can_undo);

    // memory FR-009, FR-028, SC-008.
    assert_eq!(core.memory_undo("USER.md", false), Ok(UndoOutcome::Undone));
    assert_eq!(f.read("USER.md"), first);
    assert_eq!(core.memory_undo("USER.md", false), Ok(UndoOutcome::Undone));
    assert_eq!(f.read("USER.md"), original);
    assert_eq!(core.memory_undo("USER.md", false), Ok(UndoOutcome::Nothing));

    // A user edit since the companion's change is not undone without asking.
    companion_sets(&core, &provider, "Call them", "Sam I am").await;
    f.write("USER.md", "# About the user\n\n- Edited by hand.\n");
    assert_eq!(core.memory_undo("USER.md", false), Ok(UndoOutcome::Edited));
    assert_eq!(f.read("USER.md"), "# About the user\n\n- Edited by hand.\n");
    assert_eq!(core.memory_undo("USER.md", true), Ok(UndoOutcome::Undone));
    assert_eq!(f.read("USER.md"), original);
    assert_eq!(core.memory_undo("SOUL.md", false), Err(invalid("path")));
}

/// One turn in which the companion remembers something on request.
async fn companion_remembers(core: &Core, provider: &FakeProvider, text: &str) {
    provider.script(vec![
        vec![Step::Tools(vec![tool_use(
            "r",
            "memory_remember",
            json!({ "text": text, "topic": "Facts", "requested_by_user": true }),
        )])],
        vec![Step::Text("Noted."), Step::End("end_turn")],
    ]);
    core.send_message(
        provider,
        &format!("Remember: {text}"),
        &clock(),
        &VecSink::default(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn forgetting_removes_a_memory_everywhere_it_could_come_back_from() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.set_auto_recall(false).unwrap();
    companion_remembers(&core, &provider, "Allergic to peanuts.").await;
    companion_remembers(&core, &provider, "Likes green tea.").await;
    let source = f
        .transcript()
        .into_iter()
        .find(|e| e["text"] == "Remember: Allergic to peanuts.")
        .unwrap()["id"]
        .clone();
    let doc = core.memory_read("MEMORY.md").unwrap();
    let entry = doc
        .entries
        .iter()
        .find(|e| e.text == "Allergic to peanuts.")
        .unwrap();
    assert!(!core.memory_search("peanuts").unwrap().is_empty());

    let outcome = core
        .memory_forget(
            "MEMORY.md",
            ForgetTarget::Entry {
                id: entry.id.clone().unwrap(),
            },
            &doc.hash,
        )
        .unwrap();

    // memory FR-027, SC-007: gone from the file and from search; the conversation still holds it for now.
    assert!(outcome.in_current_context);
    assert!(!f.read("MEMORY.md").contains("peanuts"));
    assert!(f.read("MEMORY.md").contains("Likes green tea."));
    assert!(core.memory_search("peanuts").unwrap().is_empty());
    // Principle IV: no undo brings it back.
    while core.memory_undo("MEMORY.md", true) == Ok(UndoOutcome::Undone) {
        assert!(!f.read("MEMORY.md").contains("peanuts"));
    }
    // The record names the source message and holds no text.
    let record = f.transcript().pop().unwrap();
    assert_eq!(record["type"], "forgotten");
    assert_eq!(record["messages"], json!([source]));
    assert!(!record.to_string().contains("peanuts"));
    // Not loaded into a future context.
    core.start_fresh_context(&clock()).unwrap();
    core.send_message(&provider, "hi", &clock(), &VecSink::default())
        .await
        .unwrap();
    let request = provider.requests().pop().unwrap();
    assert!(!format!("{:?}", request.system).contains("peanuts"));
}

#[test]
fn a_line_a_note_or_a_whole_day_can_be_forgotten_and_nothing_else() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let profile = "# About the user\n\n- Call them: Sam\n- Secret: something private\n- Preferred language: English\n";
    f.write("USER.md", profile);
    f.write(
        "memory/2026-11-01.md",
        "# 2026-11-01\n\n- 09:00 Keep this.\n- 10:00 Forget this.\n",
    );
    f.write(
        "memory/2026-11-02.md",
        "# 2026-11-02\n\n- 09:00 A whole day to forget.\n",
    );
    let hash = |path: &str| core.memory_read(path).unwrap().hash;

    let user = hash("USER.md");
    assert_eq!(
        core.memory_forget("USER.md", ForgetTarget::Line { line: 4 }, "stale"),
        Err(invalid("changed"))
    );
    let outcome = core
        .memory_forget("USER.md", ForgetTarget::Line { line: 4 }, &user)
        .unwrap();
    assert!(!outcome.in_current_context);
    assert_eq!(
        f.read("USER.md"),
        "# About the user\n\n- Call them: Sam\n- Preferred language: English\n"
    );

    core.memory_forget(
        "memory/2026-11-01.md",
        ForgetTarget::Line { line: 4 },
        &hash("memory/2026-11-01.md"),
    )
    .unwrap();
    assert_eq!(
        f.read("memory/2026-11-01.md"),
        "# 2026-11-01\n\n- 09:00 Keep this.\n"
    );
    assert!(core.memory_search("Zanzibar").unwrap().is_empty());

    core.memory_forget(
        "memory/2026-11-02.md",
        ForgetTarget::File,
        &hash("memory/2026-11-02.md"),
    )
    .unwrap();
    assert!(!f.workspace("memory/2026-11-02.md").exists());
    assert!(
        core.memory_files()
            .unwrap()
            .iter()
            .all(|f| f.path != "memory/2026-11-02.md")
    );
    assert!(core.memory_search("whole day").unwrap().is_empty());

    let memory = hash("MEMORY.md");
    for (path, target) in [
        ("MEMORY.md", ForgetTarget::File),
        ("USER.md", ForgetTarget::Entry { id: "m_1".into() }),
        ("SOUL.md", ForgetTarget::Line { line: 1 }),
        ("MEMORY.md", ForgetTarget::Line { line: 99 }),
    ] {
        let base = if path == "MEMORY.md" {
            memory.clone()
        } else {
            hash(path)
        };
        assert_eq!(
            core.memory_forget(path, target, &base),
            Err(invalid("target")),
            "{path}"
        );
    }
}
