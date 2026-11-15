//! The search index on a temporary workspace with real SQLite: it follows the files, in any language
//! (memory R9 to R12).

mod common;

use common::{FakeProvider, Fixture, VecSink, clock};

fn found(core: &pec_core::Core, query: &str) -> Vec<String> {
    core.memory_search(query)
        .unwrap()
        .into_iter()
        .map(|h| h.path)
        .collect()
}

#[test]
fn search_follows_the_files_as_they_change() {
    let f = Fixture::new();
    let core = f.core();
    assert!(
        found(&core, "anything").is_empty(),
        "works with nothing indexed or downloaded"
    );
    f.write(
        "memory/2026-10-01.md",
        "# 2026-10-01\n\n- 09:00 Bimo hides under the bed.\n",
    );
    f.write(
        "memory/notes.md",
        "- Bimo in a file that isn't a daily note.\n",
    );
    f.write("memory/2026-10-02.md", "");
    std::fs::write(
        f.workspace("memory/2026-10-02.md"),
        [0xff, 0xfe, b'B', b'i'],
    )
    .unwrap();
    assert_eq!(found(&core, "Bimo"), ["memory/2026-10-01.md"]);

    // memory FR-023: an outside edit is found by the next search, and removed text is not.
    f.write(
        "memory/2026-10-01.md",
        "# 2026-10-01\n\n- 09:00 Rina is visiting.\n",
    );
    assert!(found(&core, "Bimo").is_empty());
    assert_eq!(found(&core, "rina"), ["memory/2026-10-01.md"]);
    std::fs::remove_file(f.workspace("memory/2026-10-01.md")).unwrap();
    assert!(found(&core, "Rina").is_empty());
}

#[test]
fn a_large_edit_is_indexed_over_several_searches() {
    let f = Fixture::new();
    let core = f.core();
    let filler = "- 09:00 An ordinary day with ordinary things.\n".repeat(6_500);
    f.write("memory/2026-10-03.md", &filler);
    f.write("memory/2026-10-02.md", &filler);
    f.write(
        "memory/2026-10-01.md",
        &format!("{filler}- 10:00 The zebra escaped.\n"),
    );
    // Each search indexes at most 512 KB, newest notes first (memory R12).
    assert!(found(&core, "zebra").is_empty());
    assert!(found(&core, "zebra").is_empty());
    assert_eq!(found(&core, "zebra"), ["memory/2026-10-01.md"]);
}

#[test]
fn notes_are_found_in_any_language() {
    let f = Fixture::new();
    let core = f.core();
    f.write(
        "memory/2026-11-01.md",
        "- 09:00 Rapat dengan klien besok pagi.\n",
    );
    f.write("memory/2026-11-02.md", "- 10:00 明日は天気予報で雨です。\n");
    f.write(
        "memory/2026-11-03.md",
        "- 11:00 Bought new running shoes.\n",
    );
    f.write(
        "memory/2026-11-04.md",
        "- 12:00 Ngopi di kafé dekat kantor.\n",
    );
    for (query, path) in [
        ("rapat", "memory/2026-11-01.md"),
        ("天気", "memory/2026-11-02.md"),
        ("雨", "memory/2026-11-02.md"),
        ("shoes", "memory/2026-11-03.md"),
        ("kafe", "memory/2026-11-04.md"),
    ] {
        assert_eq!(found(&core, query), [path], "{query}");
    }
}

/// memory SC-004 and SC-009, run by hand in release mode (memory checks). Ignored in CI: performance budgets
/// and the language set are release-readiness checks (`test-audit`).
#[tokio::test]
#[ignore = "release-readiness measurement"]
async fn search_over_10000_chunks() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    let words = [
        "tea", "rain", "Bimo", "work", "Rina", "shoes", "garden", "movie", "train", "soup",
    ];
    for day in 0..1_000 {
        let date = format!("2024-{:02}-{:02}", day % 12 + 1, day % 28 + 1);
        let notes: String = (0..10)
            .map(|i| {
                format!(
                    "- 09:{i:02} Note {day}-{i} about {} and the day.\n\n",
                    words[(day + i) % 10]
                )
            })
            .collect();
        let path = f.workspace(&format!("memory/{date}.md"));
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        f.write(&format!("memory/{date}.md"), &format!("{old}{notes}"));
    }
    core.sync_memory_index().unwrap();
    let percentile = |mut times: Vec<std::time::Duration>| {
        times.sort();
        times[times.len() * 95 / 100]
    };
    let searches: Vec<_> = (0..100)
        .map(|i| {
            let started = std::time::Instant::now();
            core.memory_search(&format!("{} day", words[i % 10]))
                .unwrap();
            started.elapsed()
        })
        .collect();
    println!("search, 95th percentile: {:?}", percentile(searches));
    let mut sends = Vec::new();
    for i in 0..50 {
        let started = std::time::Instant::now();
        core.send_message(
            &provider,
            &format!("about {}", words[i % 10]),
            &clock(),
            &VecSink::default(),
        )
        .await
        .unwrap();
        sends.push(started.elapsed());
    }
    println!("send with recall, 95th percentile: {:?}", percentile(sends));

    let set: Vec<(String, String)> =
        serde_json::from_str(include_str!("fixtures/languages/queries.json")).unwrap();
    let g = Fixture::new();
    let core = g.core();
    for (i, (note, _)) in set.iter().enumerate() {
        g.write(
            &format!("memory/2025-01-{:02}.md", i + 1),
            &format!("- 09:00 {note}\n"),
        );
    }
    let hits = set
        .iter()
        .enumerate()
        .filter(|(i, (_, query))| {
            found(&core, query).contains(&format!("memory/2025-01-{:02}.md", i + 1))
        })
        .count();
    println!("language set: {hits} of {} found", set.len());
}

#[test]
fn a_missing_or_damaged_index_is_rebuilt_from_the_files() {
    let f = Fixture::new();
    f.write("MEMORY.md", "# Memory\n\n## Pets\n- Bimo is a beagle.\n");
    f.write(
        "memory/2026-10-01.md",
        "- 09:00 Bimo hides from thunder.\n- 10:00 Rina called.\n",
    );
    let queries = ["Bimo", "thunder", "Rina", "beagle"];
    let results = |core: &pec_core::Core| -> Vec<Vec<String>> {
        queries.iter().map(|q| found(core, q)).collect()
    };
    let before = results(&f.core());
    let db = f.root().join("state/pec.db");
    let clear = || {
        for suffix in ["-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", db.display()));
        }
    };

    // memory FR-022, SC-006: deleting the index loses nothing.
    std::fs::remove_file(&db).unwrap();
    clear();
    let core = f.core();
    core.sync_memory_index().unwrap();
    assert_eq!(results(&core), before);
    drop(core);

    // A damaged index is detected, deleted and rebuilt.
    clear();
    std::fs::write(
        &db,
        "this is not a database, only some garbage bytes".repeat(20),
    )
    .unwrap();
    let core = f.core();
    core.sync_memory_index().unwrap();
    assert_eq!(results(&core), before);
}

#[test]
fn an_index_from_before_the_rename_is_moved_with_its_history() {
    let f = Fixture::new();
    f.write("MEMORY.md", "# Memory\n\n## Pets\n- Bimo is a beagle.\n");
    let core = f.core();
    core.sync_memory_index().unwrap();
    drop(core);
    let state = f.root().join("state");
    // A row only the database holds, like the undo history: a rebuild from the files would lose it.
    rusqlite::Connection::open(state.join("pec.db"))
        .unwrap()
        .execute_batch("CREATE TABLE kept (x); INSERT INTO kept VALUES (1);")
        .unwrap();
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::rename(
            state.join(format!("pec.db{suffix}")),
            state.join(format!("steve.db{suffix}")),
        );
    }

    let core = f.core();
    assert_eq!(found(&core, "beagle").len(), 1);
    drop(core);
    assert!(!state.join("steve.db").exists());
    let kept: i64 = rusqlite::Connection::open(state.join("pec.db"))
        .unwrap()
        .query_row("SELECT x FROM kept", [], |r| r.get(0))
        .unwrap();
    assert_eq!(kept, 1);
}
