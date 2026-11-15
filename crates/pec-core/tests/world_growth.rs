//! Growth through the conversation on a temporary workspace: once on the first message of a day, never for the
//! hello or a retry, never twice, never for missed days, and never in the way of a reply (world R7).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink};
use pec_core::Core;
use pec_core::chat::LocalClock;
use pec_core::llm::ProviderError;

fn on(day: &str) -> LocalClock {
    LocalClock {
        iso: format!("{day}T09:00:00+07:00"),
        zone: "Asia/Jakarta".into(),
    }
}

/// A set-up companion with its world, created on `day`.
fn companion(f: &Fixture, provider: &FakeProvider, day: &str) -> Core {
    let core = f.ready(provider);
    f.write("../state/state.json", r#"{"setup":{"complete":true}}"#);
    core.prepare_world(&on(day)).unwrap();
    core
}

async fn say(core: &Core, provider: &FakeProvider, day: &str) {
    core.send_message(provider, "hi", &on(day), &VecSink::default())
        .await
        .unwrap();
}

/// The world's item lines (the header excluded).
fn items(f: &Fixture) -> Vec<String> {
    f.read("world/world.jsonl")
        .lines()
        .skip(1)
        .map(str::to_owned)
        .collect()
}

fn drawn_on(f: &Fixture, day: &str) -> usize {
    items(f)
        .iter()
        .filter(|l| l.contains(&format!("\"day\":\"{day}\"")) && !l.contains("\"home."))
        .count()
}

#[tokio::test]
async fn the_first_message_of_a_day_draws_once() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-14");
    say(&core, &provider, "2026-11-14").await;
    let first = drawn_on(&f, "2026-11-14");
    assert!((2..=3).contains(&first), "{first}");
    assert_eq!(f.state()["world"]["lastDay"], "2026-11-14");

    // world SC-005: more messages that day add nothing.
    for _ in 0..120 {
        say(&core, &provider, "2026-11-14").await;
    }
    assert_eq!(drawn_on(&f, "2026-11-14"), first);

    // A reply that fails still counts as a day with a conversation.
    provider.script(vec![vec![Step::Fail(ProviderError::Unreachable, None)]]);
    say(&core, &provider, "2026-11-15").await;
    assert!(drawn_on(&f, "2026-11-15") >= 2);
}

#[tokio::test]
async fn missed_days_are_not_made_up_and_nothing_decays() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-01");
    say(&core, &provider, "2026-11-01").await;
    let before = items(&f);

    // world SC-004: sixty quiet days change nothing; the next day adds one ordinary handful.
    say(&core, &provider, "2026-12-31").await;
    let after = items(&f);
    assert_eq!(after[..before.len()], before[..]);
    let added = after.len() - before.len();
    assert!((2..=3).contains(&added), "{added}");
    assert_eq!(drawn_on(&f, "2026-12-31"), added);

    // The clock going back draws nothing.
    say(&core, &provider, "2026-12-29").await;
    assert_eq!(items(&f), after);
}

#[tokio::test]
async fn deleting_the_days_items_does_not_draw_them_again() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-14");
    say(&core, &provider, "2026-11-14").await;
    let kept: Vec<String> = f
        .read("world/world.jsonl")
        .lines()
        .filter(|l| !l.contains("\"day\":\"2026-11-14\"") || l.contains("\"home."))
        .map(|l| format!("{l}\n"))
        .collect();
    f.write("world/world.jsonl", &kept.concat());
    say(&core, &provider, "2026-11-14").await;
    assert_eq!(drawn_on(&f, "2026-11-14"), 0);
}

#[tokio::test]
async fn the_hello_and_retries_draw_nothing() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-14");
    f.write(
        "../state/state.json",
        r#"{"setup":{"complete":true},"firstHelloPending":true}"#,
    );
    core.start_first_hello(&provider, &on("2026-11-15"), &VecSink::default())
        .await
        .unwrap();
    assert_eq!(drawn_on(&f, "2026-11-15"), 0);
    core.regenerate_reply(&provider, &on("2026-11-15"), &VecSink::default())
        .await
        .unwrap();
    assert_eq!(drawn_on(&f, "2026-11-15"), 0);
}

#[tokio::test]
async fn a_world_that_cannot_be_written_never_fails_a_message() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-14");
    let world = f.workspace("world/world.jsonl");
    let saved = f.read("world/world.jsonl");
    std::fs::remove_file(&world).unwrap();
    std::fs::create_dir(&world).unwrap();

    // world FR-025: the message is sent and answered; the day isn't marked as drawn.
    let sink = VecSink::default();
    core.send_message(&provider, "hi", &on("2026-11-15"), &sink)
        .await
        .unwrap();
    assert!(matches!(
        sink.last(),
        pec_core::chat::ChatEvent::Finished {
            message: Some(_),
            ..
        }
    ));
    assert!(f.state()["world"]["lastDay"].is_null());

    // Restored, the next message that day draws.
    std::fs::remove_dir(&world).unwrap();
    f.write("world/world.jsonl", &saved);
    say(&core, &provider, "2026-11-15").await;
    assert!(drawn_on(&f, "2026-11-15") >= 2);
}
