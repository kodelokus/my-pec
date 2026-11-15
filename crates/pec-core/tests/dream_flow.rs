//! Dreaming on a temporary workspace with the fake provider and the injected clock: promotion from recall evidence,
//! the schedule, pausing, crashes, safety, the diary, undo, the morning context and settings (dreaming R14).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink, models};
use pec_core::chat::LocalClock;
use pec_core::dream::{DreamState, UndoDreamOutcome};
use pec_core::llm::Effort;
use pec_core::{AppError, Core};
use serde_json::{Value, json};

fn on(day: &str, time: &str) -> LocalClock {
    LocalClock {
        iso: format!("{day}T{time}:00+07:00"),
        zone: "Asia/Jakarta".into(),
    }
}

/// A message on `day`; its auto-recall records evidence for the notes it matches.
async fn say(core: &Core, provider: &FakeProvider, text: &str, day: &str) {
    core.send_message(provider, text, &on(day, "20:00"), &VecSink::default())
        .await
        .unwrap();
}

/// A daily note with one companion bullet per fact.
fn note(f: &Fixture, day: &str, facts: &[&str]) {
    let lines: String = facts
        .iter()
        .enumerate()
        .map(|(i, fact)| format!("- 09:{i:02} {fact} <!-- from:companion src:1-0 -->\n"))
        .collect();
    f.write(&format!("memory/{day}.md"), &format!("# {day}\n\n{lines}"));
}

fn json_reply(value: Value) -> Vec<Step> {
    vec![Step::Blocks(vec![
        json!({ "type": "text", "text": value.to_string() }),
    ])]
}

fn add(id: usize, topic: &str, text: &str) -> Value {
    json!({ "op": "add", "topic": topic, "text": text, "key": "", "sources": [id], "replaces": [] })
}

/// The calls of a sweep that commits: themes, decisions, the diary, then the roll's flush and summary.
fn dream_replies(ops: Value, diary: &'static str) -> Vec<Vec<Step>> {
    vec![
        json_reply(
            json!({ "themes": [{ "title": "Running", "summary": "Kept coming up.", "candidates": [1] }] }),
        ),
        json_reply(json!({ "operations": ops })),
        vec![Step::Text(diary), Step::End("end_turn")],
        vec![Step::Text("Nothing new."), Step::End("end_turn")],
        json_reply(
            json!({ "thread": "Talking about the week.", "commitments": [], "relationship": "Warm.", "facts": [], "notes": [] }),
        ),
    ]
}

const MARATHON: &str = "Training for the half marathon in April.";

/// One fact noted on 2 November and recalled by two different questions on two later days.
async fn recurring(f: &Fixture, provider: &FakeProvider, core: &Core, year: &str) {
    note(f, &format!("{year}-11-02"), &[MARATHON]);
    say(
        core,
        provider,
        "half marathon training",
        &format!("{year}-11-06"),
    )
    .await;
    say(core, provider, "April marathon", &format!("{year}-11-09")).await;
}

fn read_or_empty(f: &Fixture, file: &str) -> String {
    std::fs::read_to_string(f.workspace(file)).unwrap_or_default()
}

fn db(f: &Fixture) -> rusqlite::Connection {
    rusqlite::Connection::open(f.root().join("state/pec.db")).unwrap()
}

#[tokio::test]
async fn two_weeks_of_recalls_promote_what_keeps_coming_up() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let recurring = [
        (
            "Training for the half marathon in April.",
            "half marathon training",
            "April marathon",
        ),
        (
            "Sister Rina lives in Surabaya.",
            "Rina Surabaya",
            "sister Rina",
        ),
        ("Allergic to cashew nuts.", "cashew nuts", "allergic cashew"),
        (
            "Learning to play the cello.",
            "cello learning",
            "play cello",
        ),
        ("The dog is named Biscuit.", "dog Biscuit", "Biscuit named"),
        (
            "Works as a dentist in Malang.",
            "dentist Malang",
            "works dentist",
        ),
        (
            "Planning a trip to Lisbon in June.",
            "Lisbon trip",
            "June Lisbon",
        ),
        (
            "Favourite tea is jasmine oolong.",
            "jasmine oolong",
            "oolong tea",
        ),
        (
            "Building a wooden bookshelf for the study.",
            "wooden bookshelf",
            "bookshelf study",
        ),
        (
            "Grandmother turns ninety in March.",
            "grandmother ninety",
            "March grandmother",
        ),
    ];
    let once = [
        "Saw a heron by the river.",
        "Lost a glove at the station.",
        "Tried a mango smoothie.",
        "Repainted the fence green.",
        "Watched a documentary about volcanoes.",
        "Found an old harmonica.",
        "Kite got stuck in a tree.",
        "Bought paperclips in bulk.",
        "Neighbour plays the trumpet.",
        "Fixed a squeaky hinge.",
    ];
    note(&f, "2026-11-02", &recurring.map(|r| r.0));
    note(&f, "2026-11-12", &once);
    for (_, first, second) in recurring {
        say(&core, &provider, first, "2026-11-06").await;
        say(&core, &provider, second, "2026-11-09").await;
    }
    let notes_before = (
        read_or_empty(&f, "memory/2026-11-02.md"),
        read_or_empty(&f, "memory/2026-11-12.md"),
    );
    // The model would keep everything it is shown; only what passed the gates reaches the files.
    let ops: Vec<Value> = (1..=20)
        .map(|id| {
            add(
                id,
                "About Sam",
                recurring.get(id - 1).map_or_else(|| once[id - 11], |r| r.0),
            )
        })
        .collect();
    provider.script(dream_replies(Value::Array(ops), "A week of plans."));
    let status = core
        .dream_now(&provider, &on("2026-11-14", "09:00"))
        .await
        .unwrap();
    assert_eq!(status.state, DreamState::Idle);

    let memory = read_or_empty(&f, "MEMORY.md");
    for (i, (fact, ..)) in recurring.iter().enumerate() {
        let line = memory
            .lines()
            .find(|l| l.contains(fact))
            .unwrap_or_else(|| panic!("{fact} promoted"));
        // dreaming SC-003: a source that names the note line holding the fact.
        assert!(
            line.contains(&format!("src:memory/2026-11-02.md#L{}", i + 3)),
            "{line}"
        );
    }
    for fact in once {
        assert!(!memory.contains(fact), "{fact} is a one-off");
    }
    // dreaming FR-019: the notes and the soul are untouched; the deep request saw only the gated candidates.
    assert_eq!(
        (
            read_or_empty(&f, "memory/2026-11-02.md"),
            read_or_empty(&f, "memory/2026-11-12.md")
        ),
        notes_before
    );
    assert_eq!(read_or_empty(&f, "SOUL.md"), "# Soul\n\nBe warm.");
    let deep = provider
        .requests()
        .iter()
        .map(|r| serde_json::to_string(&r.messages).unwrap())
        .find(|t| t.contains("Current long-term memory:"))
        .unwrap();
    assert!(deep.contains("Rina") && !deep.contains("heron"));
    let journal = core.dream_journal().unwrap();
    let run = &journal.runs[0];
    assert!(run.completed && !run.fell_back);
    assert_eq!(run.added, 10);
    assert!(journal.can_undo);
    let preimages: i64 = db(&f)
        .query_row(
            "SELECT count(*) FROM preimages WHERE message LIKE 'dream:%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(preimages, 1, "MEMORY.md changed; USER.md didn't");

    // Nothing new since: a second sweep changes nothing.
    core.dream_now(&provider, &on("2026-11-14", "10:00"))
        .await
        .unwrap();
    assert_eq!(read_or_empty(&f, "MEMORY.md"), memory);
}

#[tokio::test]
async fn a_night_runs_once_when_the_person_has_been_idle() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    // Not yet 03:00: the latest night is the 13th, which counts as the first night.
    provider.script(dream_replies(
        json!([add(1, "Running", MARATHON)]),
        "Ran in my sleep.",
    ));
    let status = core
        .dream_tick(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    assert!(
        status
            .last_run
            .as_ref()
            .is_some_and(|r| r.completed && r.night == "2030-11-14")
    );
    assert!(read_or_empty(&f, "MEMORY.md").contains(MARATHON));
    let requests = provider.requests().len();
    // The same night never runs twice, the clock set back runs nothing.
    for clock in [on("2030-11-14", "23:00"), on("2030-11-13", "09:00")] {
        assert_eq!(
            core.dream_tick(&provider, &clock).await.unwrap().state,
            DreamState::Idle
        );
    }
    assert_eq!(provider.requests().len(), requests);
    // The next night runs at its time.
    let status = core
        .dream_tick(&provider, &on("2030-11-15", "03:05"))
        .await
        .unwrap();
    assert_eq!(status.last_run.unwrap().night, "2030-11-15");
}

#[tokio::test]
async fn a_message_pauses_a_sweep_and_the_next_tick_resumes_it() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    // The REM call never ends by itself; the message doesn't wait for it (dreaming SC-008).
    provider.script(vec![
        vec![Step::Hang],
        vec![Step::Text("Right away."), Step::End("end_turn")],
    ]);
    let (sink, clock) = (VecSink::default(), on("2030-11-14", "09:00"));
    let (dreamt, sent) = tokio::join!(core.dream_now(&provider, &clock), async {
        tokio::task::yield_now().await;
        core.send_message(&provider, "still up?", &clock, &sink)
            .await
    });
    assert_eq!(dreamt.unwrap().state, DreamState::Paused);
    assert_eq!(sent, Ok(None));
    assert_eq!(sink.deltas(), "Right away.");
    let paused: String = db(&f)
        .query_row("SELECT phase FROM dream_runs ORDER BY id DESC", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(paused, "rem");
    // Idle again (idleness itself is the schedule's unit test): it resumes from REM without gathering again.
    provider.script(dream_replies(
        json!([add(1, "Running", MARATHON)]),
        "Resumed.",
    ));
    let status = core
        .dream_tick(&provider, &on("2031-01-01", "09:00"))
        .await
        .unwrap();
    assert!(status.last_run.unwrap().completed);
    assert!(read_or_empty(&f, "MEMORY.md").contains(MARATHON));
}

#[tokio::test]
async fn an_interrupted_sweep_changes_nothing_and_resumes_once() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    let memory = read_or_empty(&f, "MEMORY.md");
    // The app ends during the deep call: the sweep's future is dropped.
    provider.script(vec![json_reply(json!({ "themes": [] })), vec![Step::Hang]]);
    let clock = on("2030-11-14", "09:00");
    let cut = tokio::time::timeout(
        std::time::Duration::from_millis(300),
        core.dream_now(&provider, &clock),
    )
    .await;
    assert!(cut.is_err());
    drop(core);
    assert_eq!(read_or_empty(&f, "MEMORY.md"), memory, "dreaming SC-004");
    let core = f.core();
    provider.script(vec![
        json_reply(json!({ "operations": [add(1, "Running", MARATHON)] })),
        vec![Step::Text("Again."), Step::End("end_turn")],
        vec![Step::Text("Nothing new."), Step::End("end_turn")],
        json_reply(json!({ "thread": "t", "commitments": [], "relationship": "", "facts": [], "notes": [] })),
    ]);
    core.dream_now(&provider, &clock).await.unwrap();
    assert_eq!(read_or_empty(&f, "MEMORY.md").matches(MARATHON).count(), 1);
    let runs: i64 = db(&f)
        .query_row("SELECT count(*) FROM dream_runs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(runs, 1, "resumed, not started again");

    // A run left mid-commit is rolled back to its preimages, then completed once.
    db(&f)
        .execute("UPDATE dream_runs SET phase = 'committing'", [])
        .unwrap();
    provider.script(vec![
        json_reply(json!({ "operations": [add(1, "Running", MARATHON)] })),
        vec![Step::Text("Third time."), Step::End("end_turn")],
        vec![Step::Text("Nothing new."), Step::End("end_turn")],
        json_reply(json!({ "thread": "t", "commitments": [], "relationship": "", "facts": [], "notes": [] })),
    ]);
    f.core().dream_now(&provider, &clock).await.unwrap();
    assert_eq!(read_or_empty(&f, "MEMORY.md").matches(MARATHON).count(), 1);
}

#[tokio::test]
async fn a_rejected_key_fails_the_run_and_it_waits_an_hour() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    provider.script(vec![vec![Step::Fail(
        pec_core::llm::ProviderError::KeyRejected,
        None,
    )]]);
    let status = core
        .dream_tick(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    assert!(!status.last_run.unwrap().completed);
    assert_eq!(read_or_empty(&f, "MEMORY.md"), "# Memory\n");
    let requests = provider.requests().len();
    // Within the hour (by the core's own record), the night is not retried.
    core.dream_tick(&provider, &on("2030-11-14", "09:10"))
        .await
        .unwrap();
    assert_eq!(provider.requests().len(), requests);
}

#[tokio::test]
async fn untrusted_forgotten_and_recalled_only_material_never_promotes() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    f.write(
        "memory/2030-11-02.md",
        "# 2030-11-02\n\n\
         - 09:00 Password hint is bluebird. <!-- from:untrusted src:1-0 -->\n\
         - 09:01 Secret plan for zeppelin racing. <!-- from:companion src:FORGOTTEN -->\n\
         - 09:02 Training for the half marathon in April. <!-- from:companion src:1-0 -->\n",
    );
    for (text, day) in [
        ("password hint bluebird", "2030-11-06"),
        ("bluebird password", "2030-11-09"),
        ("zeppelin racing plan", "2030-11-06"),
        ("secret zeppelin", "2030-11-09"),
        ("half marathon training", "2030-11-06"),
        ("April marathon", "2030-11-09"),
    ] {
        say(&core, &provider, text, day).await;
    }
    // A forgotten source message: as memory records it.
    f.write(
        "sessions/main.jsonl",
        &format!(
            "{}{}\n",
            read_or_empty(&f, "sessions/main.jsonl"),
            r#"{"id":"9999999999998-0","at":1,"type":"forgotten","messages":["FORGOTTEN"]}"#
        ),
    );
    let core = f.core();
    let ops: Vec<Value> = (1..=3)
        .map(|id| add(id, "Notes", &format!("Fact {id}")))
        .collect();
    provider.script(dream_replies(Value::Array(ops), "Careful dream."));
    core.dream_now(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    // dreaming SC-002: the person's own words may inform themes, but no such note is a candidate or reaches the decisions.
    let requests = provider.requests();
    let text = |r: &pec_core::llm::TurnRequest| serde_json::to_string(&r.messages).unwrap();
    let rem = requests
        .iter()
        .map(text)
        .find(|t| t.contains("Candidate notes:"))
        .unwrap();
    let candidates = rem.split("The person's messages:").next().unwrap();
    let deep = requests
        .iter()
        .map(text)
        .find(|t| t.contains("Current long-term memory:"))
        .unwrap();
    for forbidden in ["bluebird", "zeppelin"] {
        assert!(
            !candidates.contains(forbidden) && !deep.contains(forbidden),
            "{forbidden}"
        );
    }
    let memory = read_or_empty(&f, "MEMORY.md");
    assert_eq!(memory.matches("<!-- id:").count(), 1, "{memory}");
    assert!(!read_or_empty(&f, "DREAMS.md").is_empty());
    // The diary is never gathered: a second dream with only the diary new promotes nothing.
    core.dream_now(&provider, &on("2030-11-15", "09:00"))
        .await
        .unwrap();
    assert_eq!(read_or_empty(&f, "MEMORY.md"), memory);
}

#[tokio::test]
async fn a_lossy_rewrite_or_a_failed_request_falls_back_to_appending() {
    for (case, deep) in [
        (
            "lossy",
            json_reply(json!({ "operations": [{
                "op": "supersede", "topic": "Health", "text": MARATHON, "key": "",
                "sources": [1], "replaces": ["m_a", "m_b"],
            }] })),
        ),
        (
            "failed",
            vec![Step::Fail(pec_core::llm::ProviderError::Unreachable, None)],
        ),
    ] {
        let f = Fixture::new();
        let provider = FakeProvider::accepting();
        let core = f.ready(&provider);
        let prior = "# Memory\n\n## Health\n- A <!-- id:m_a at:d from:user src:1-0 -->\n- B <!-- id:m_b at:d from:user src:1-0 -->\n\
                     - C <!-- id:m_c at:d from:user src:1-0 -->\n";
        f.write("MEMORY.md", prior);
        recurring(&f, &provider, &core, "2030").await;
        let mut replies = dream_replies(json!([]), "Fell back.");
        replies[1] = deep;
        provider.script(replies);
        core.dream_now(&provider, &on("2030-11-14", "09:00"))
            .await
            .unwrap();
        let memory = read_or_empty(&f, "MEMORY.md");
        assert!(memory.starts_with(prior.trim_end()), "{case}: nothing lost");
        assert!(
            memory.contains("## From my dreams\n- Training for the half marathon in April."),
            "{case}"
        );
        assert!(core.dream_journal().unwrap().runs[0].fell_back, "{case}");
    }
}

#[tokio::test]
async fn edits_during_a_sweep_are_honored() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = std::sync::Arc::new(f.ready(&provider));
    recurring(&f, &provider, &core, "2030").await;
    // The person edits MEMORY.md while the model decides: the edit stays and the decision is applied to it.
    let path = f.workspace("MEMORY.md");
    let mut replies = dream_replies(json!([add(1, "Running", MARATHON)]), "Edited.");
    replies[1].insert(
        0,
        Step::Do(std::sync::Arc::new(move || {
            std::fs::write(&path, "# Memory\n\n## Mine\n- My own line.\n").unwrap();
        })),
    );
    provider.script(replies);
    core.dream_now(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    let memory = read_or_empty(&f, "MEMORY.md");
    assert!(
        memory.contains("- My own line.") && memory.contains(MARATHON),
        "{memory}"
    );

    // A note line changed after it was gathered is skipped.
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    let path = f.workspace("memory/2030-11-02.md");
    let mut replies = dream_replies(json!([add(1, "Running", MARATHON)]), "Skipped.");
    replies[0].insert(
        0,
        Step::Do(std::sync::Arc::new(move || {
            std::fs::write(
                &path,
                "# 2030-11-02\n\n- 09:00 Training for the full marathon in May.\n",
            )
            .unwrap();
        })),
    );
    provider.script(replies);
    core.dream_now(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    assert!(!read_or_empty(&f, "MEMORY.md").contains("marathon"));
}

#[tokio::test]
async fn the_diary_and_journal_tell_what_happened_and_undo_takes_it_back() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    let (memory, profile) = (read_or_empty(&f, "MEMORY.md"), read_or_empty(&f, "USER.md"));
    let mut ops = vec![add(1, "Running", MARATHON)];
    ops.push(json!({ "op": "profile", "topic": "", "text": "running", "key": "Hobby", "sources": [1], "replaces": [] }));
    provider.script(dream_replies(Value::Array(ops), "I dreamt of a long road."));
    core.dream_now(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    // The diary request speaks with the soul; the entry lists exactly what was kept.
    let diary_request = provider
        .requests()
        .into_iter()
        .find(|r| r.system.len() == 2)
        .unwrap();
    assert!(diary_request.system[0].contains("## SOUL.md"));
    assert_eq!(
        read_or_empty(&f, "DREAMS.md"),
        format!(
            "# Dream diary\n\n## 2030-11-14\nI dreamt of a long road.\n\nKept:\n- {MARATHON}\n- Hobby: running\n"
        )
    );
    let journal = core.dream_journal().unwrap();
    assert!(journal.diary.contains("long road"));
    assert_eq!(journal.runs[0].added, 2);
    assert!(journal.runs[0].tokens == 0 && journal.can_undo);

    // dreaming FR-018, the principles' dreaming gate: undo restores both files byte for byte.
    assert_eq!(
        core.undo_last_dream(false).unwrap(),
        UndoDreamOutcome::Undone
    );
    assert_eq!(
        (read_or_empty(&f, "MEMORY.md"), read_or_empty(&f, "USER.md")),
        (memory.clone(), profile)
    );
    assert_eq!(
        core.undo_last_dream(false).unwrap(),
        UndoDreamOutcome::NothingToUndo
    );
    assert!(!core.dream_journal().unwrap().can_undo);
}

#[tokio::test]
async fn undo_asks_before_discarding_a_later_edit() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    let memory = read_or_empty(&f, "MEMORY.md");
    provider.script(dream_replies(
        json!([add(1, "Running", MARATHON)]),
        "Dream.",
    ));
    core.dream_now(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    f.write(
        "MEMORY.md",
        &format!("{}- Edited later.\n", read_or_empty(&f, "MEMORY.md")),
    );
    assert_eq!(
        core.undo_last_dream(false).unwrap(),
        UndoDreamOutcome::Changed {
            files: vec!["MEMORY.md".into()]
        }
    );
    assert!(read_or_empty(&f, "MEMORY.md").contains("Edited later."));
    assert_eq!(
        core.undo_last_dream(true).unwrap(),
        UndoDreamOutcome::Undone
    );
    assert_eq!(read_or_empty(&f, "MEMORY.md"), memory);
}

#[tokio::test]
async fn the_next_morning_starts_from_the_new_memory_with_the_dream() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    recurring(&f, &provider, &core, "2030").await;
    provider.script(dream_replies(
        json!([add(1, "Running", MARATHON)]),
        "I dreamt of a long road.",
    ));
    core.dream_now(&provider, &on("2030-11-14", "09:00"))
        .await
        .unwrap();
    say(&core, &provider, "good morning", "2030-11-14").await;
    let request = provider.requests().pop().unwrap();
    assert!(
        request.system[2].contains(MARATHON),
        "dreaming FR-023: the new memory"
    );
    let opener = request.messages[0]["content"][0]["text"].as_str().unwrap();
    assert!(
        opener.contains("Talking about the week."),
        "the conversation's thread"
    );
    assert!(
        opener.contains("(a dream, not a fact):</memory>") && opener.contains("long road"),
        "dreaming FR-024"
    );

    // A sweep that changes nothing doesn't open a new context.
    let contexts = read_or_empty(&f, "sessions/main.jsonl")
        .matches("\"type\":\"context\"")
        .count();
    core.dream_now(&provider, &on("2030-11-15", "09:00"))
        .await
        .unwrap();
    assert_eq!(
        read_or_empty(&f, "sessions/main.jsonl")
            .matches("\"type\":\"context\"")
            .count(),
        contexts
    );
}

#[tokio::test]
async fn settings_choose_whether_when_and_with_which_model() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    core.model_list(&provider, true).await.unwrap();
    let clock = on("2030-11-14", "09:00");
    let invalid = |field: &str| {
        Err(AppError::InvalidInput {
            field: field.into(),
        })
    };
    assert_eq!(
        core.set_dreaming_settings(true, "25:00", None, true, &clock),
        invalid("time")
    );
    assert_eq!(
        core.set_dreaming_settings(true, "03:00", Some("nope".into()), true, &clock),
        invalid("model")
    );
    let defaults = core.dreaming_settings().unwrap();
    assert!(defaults.enabled && defaults.time == "03:00" && defaults.model.is_none());

    // Off: no tick runs a night, but dream now does, with the chosen model.
    core.set_dreaming_settings(
        false,
        "14:00",
        Some("claude-sonnet-5-5".into()),
        true,
        &clock,
    )
    .unwrap();
    recurring(&f, &provider, &core, "2030").await;
    let requests = provider.requests().len();
    core.dream_tick(&provider, &on("2030-11-14", "15:00"))
        .await
        .unwrap();
    assert_eq!(provider.requests().len(), requests);
    provider.script(dream_replies(
        json!([add(1, "Running", MARATHON)]),
        "Chosen.",
    ));
    core.dream_now(&provider, &on("2030-11-14", "15:00"))
        .await
        .unwrap();
    let dream_requests: Vec<_> = provider.requests()[requests..3 + requests].to_vec();
    assert!(
        dream_requests
            .iter()
            .all(|r| r.model == "claude-sonnet-5-5")
    );
    // dreaming R13: themes and the diary at low effort, the decisions at medium.
    let efforts: Vec<_> = dream_requests.iter().map(|r| r.effort).collect();
    assert_eq!(
        efforts,
        [Some(Effort::Low), Some(Effort::Medium), Some(Effort::Low)]
    );

    // On again at 14:00: the night it was off is not caught up; the next 14:00 is.
    core.set_dreaming_settings(true, "14:00", None, true, &on("2030-11-16", "15:00"))
        .unwrap();
    assert_eq!(
        core.dream_tick(&provider, &on("2030-11-16", "16:00"))
            .await
            .unwrap()
            .last_run
            .unwrap()
            .night,
        "2030-11-14"
    );
    let next = core
        .dream_tick(&provider, &on("2030-11-17", "14:05"))
        .await
        .unwrap();
    assert_eq!(next.last_run.unwrap().night, "2030-11-17");

    // A withdrawn model falls back to the chat model, and settings say so.
    core.set_dreaming_settings(
        true,
        "14:00",
        Some("claude-sonnet-5-5".into()),
        true,
        &clock,
    )
    .unwrap();
    provider.set(Ok(models(&["claude-opus-5-5"])));
    core.model_list(&provider, true).await.unwrap();
    assert!(core.dreaming_settings().unwrap().model_missing);
}
