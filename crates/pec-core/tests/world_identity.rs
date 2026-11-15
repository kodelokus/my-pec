//! The companion's id, birth date and world on a temporary workspace: setup, older companions, and the files the
//! user owns (world R2, R5, R11).

mod common;

use common::{FakeProvider, Fixture, KEY, VecSink, clock};
use pec_core::AppError;
use pec_core::chat::LocalClock;
use pec_core::onboarding::SetupCompletion;
use serde_json::Value;

fn on(day: &str) -> LocalClock {
    LocalClock {
        iso: format!("{day}T09:00:00+07:00"),
        zone: "Asia/Jakarta".into(),
    }
}

fn lines(f: &Fixture) -> Vec<Value> {
    f.read("world/world.jsonl")
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// A companion as the memory feature left it: set up, no id, no world.
fn older(f: &Fixture, provider: &FakeProvider) -> pec_core::Core {
    let core = f.ready(provider);
    f.write("../state/state.json", r#"{"setup":{"complete":true}}"#);
    core
}

#[test]
fn setup_gives_a_new_companion_its_id_birth_date_and_home() {
    let f = Fixture::new();
    f.store_key(KEY);
    let core = f.core();
    core.complete_setup(SetupCompletion::New, &on("2026-11-14"))
        .unwrap();

    let identity = f.read("IDENTITY.md");
    let id = identity
        .lines()
        .find_map(|l| l.strip_prefix("- Id: "))
        .unwrap()
        .to_owned();
    assert!(identity.ends_with("- Born: 2026-11-14\n"));
    let world = lines(&f);
    assert_eq!(world.len(), 2);
    assert_eq!(world[0]["companion"], id.as_str());
    assert_eq!(world[0]["born"], "2026-11-14");
    assert!(world[1]["item"].as_str().unwrap().starts_with("home."));
    assert_eq!(
        (world[1]["x"].as_i64(), world[1]["y"].as_i64()),
        (Some(0), Some(0))
    );
    assert_eq!(world[1]["day"], "2026-11-14");

    let view = core.world_view().unwrap();
    assert_eq!(view.born.as_deref(), Some("2026-11-14"));
    assert_eq!(view.items.len(), 1);
    assert!(view.items[0].known);
}

#[tokio::test]
async fn an_older_companion_gets_its_id_without_a_notice() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = older(&f, &provider);
    let hand_edited = "# Identity\n\n- Name: Steve\n- Avatar: pip\nMy own note, no newline";
    f.write("IDENTITY.md", hand_edited);
    core.send_message(&provider, "hi", &clock(), &VecSink::default())
        .await
        .unwrap();

    // A day later, the update arrives and the main view opens.
    let outcome = core.prepare_world(&on("2026-11-14")).unwrap();
    assert!(
        outcome.recreated.is_empty(),
        "a first world is not a recreated one"
    );

    // world FR-038: two lines at the end, every other byte kept; the birth date is the first message's day.
    let identity = f.read("IDENTITY.md");
    let added = identity.strip_prefix(hand_edited).unwrap();
    assert!(added.starts_with("\n- Id: "));
    assert!(added.ends_with("\n- Born: 2026-11-13\n"));
    assert_eq!(lines(&f)[1]["day"], "2026-11-14");

    // world FR-039: no notice at the next message.
    core.send_message(&provider, "still here", &clock(), &VecSink::default())
        .await
        .unwrap();
    let request = provider.requests().pop().unwrap();
    let sent = request.messages.last().unwrap()["content"].to_string();
    assert!(!sent.contains("<notice>"), "{sent}");

    // Idempotent.
    let before = (f.read("IDENTITY.md"), f.read("world/world.jsonl"));
    core.prepare_world(&on("2026-11-15")).unwrap();
    assert_eq!((f.read("IDENTITY.md"), f.read("world/world.jsonl")), before);
}

#[test]
fn an_existing_id_is_kept_and_a_missing_one_comes_back_from_the_world() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = older(&f, &provider);
    let with_id = "# Identity\n\n- Name: Steve\n- Id: k7q2-m9tx-3hpa\n";
    f.write("IDENTITY.md", with_id);
    core.prepare_world(&on("2026-11-14")).unwrap();
    assert_eq!(f.read("IDENTITY.md"), with_id, "a readable id adds nothing");
    assert_eq!(lines(&f)[0]["companion"], "K7Q2-M9TX-3HPA");

    // world FR-038: the world's header restores a lost id.
    f.write("IDENTITY.md", "# Identity\n\n- Name: Steve\n");
    core.prepare_world(&on("2026-11-15")).unwrap();
    assert!(
        f.read("IDENTITY.md")
            .ends_with("- Id: K7Q2-M9TX-3HPA\n- Born: 2026-11-15\n")
    );
}

#[test]
fn the_world_waits_for_setup() {
    let f = Fixture::new();
    let core = f.core();
    assert_eq!(
        core.prepare_world(&on("2026-11-14")),
        Err(AppError::InvalidInput {
            field: "setup".into()
        })
    );
}

#[tokio::test]
async fn the_users_edits_to_the_world_are_kept() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = older(&f, &provider);
    core.prepare_world(&on("2026-11-14")).unwrap();
    core.send_message(&provider, "hi", &on("2026-11-14"), &VecSink::default())
        .await
        .unwrap();
    // The user removes the home and the first drawn item, and adds an unknown item and a broken line.
    let original = f.read("world/world.jsonl");
    let mut lines: Vec<&str> = original.lines().collect();
    let removed = lines.remove(2);
    lines.remove(1);
    let unknown = r#"{"item":"thing.unknown","x":9,"y":9,"day":"2026-11-14"}"#;
    let edited = format!("{}\n{unknown}\nnot json\n", lines.join("\n"));
    f.write("world/world.jsonl", &edited);

    let view = core.world_view().unwrap();
    assert_eq!(view.skipped_lines, 1);
    assert!(
        view.items
            .iter()
            .any(|i| i.item == "thing.unknown" && !i.known)
    );

    core.send_message(&provider, "hi", &on("2026-11-15"), &VecSink::default())
        .await
        .unwrap();
    let after = f.read("world/world.jsonl");
    // world FR-013, FR-015: the edited file is kept byte for byte; growth only adds after it.
    assert!(after.starts_with(&edited), "{after}");
    assert!(!after.contains(removed));
    assert!(
        !after.contains("\"home."),
        "a removed home is not added back"
    );
    assert!(
        after.len() > edited.len(),
        "growth continues from what is there"
    );
}

#[test]
fn a_deleted_world_is_recreated_and_a_changed_id_is_honoured() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = older(&f, &provider);
    core.prepare_world(&on("2026-11-14")).unwrap();
    let header = f
        .read("world/world.jsonl")
        .lines()
        .next()
        .unwrap()
        .to_owned();

    // A changed id is used; the world's record of the old one is left as it was.
    let identity = f.read("IDENTITY.md");
    let changed = identity
        .lines()
        .map(|l| {
            if l.starts_with("- Id: ") {
                "- Id: Z9W8-V7U6-T5S4"
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let before = core.world_view().unwrap().palette;
    f.write("IDENTITY.md", &changed);
    core.prepare_world(&on("2026-11-15")).unwrap();
    // The identity file's id seeds the view (journey FR-033: no longer shown, still used).
    assert_ne!(core.world_view().unwrap().palette, before);
    assert_eq!(f.read("world/world.jsonl").lines().next().unwrap(), header);

    // world R11: a deleted world file comes back with its home and is reported.
    std::fs::remove_file(f.workspace("world/world.jsonl")).unwrap();
    let outcome = core.prepare_world(&on("2026-11-16")).unwrap();
    assert_eq!(outcome.recreated, ["world/world.jsonl"]);
    let world = lines(&f);
    assert_eq!(world.len(), 2);
    assert_eq!(world[0]["companion"], "Z9W8-V7U6-T5S4");
    assert_eq!(world[1]["day"], "2026-11-16");
}
