//! The world journey on a temporary workspace with the fake provider and the injected clock: the dream sweep names
//! the ended days' topics and draws an everyday object for each, whole, once (journey).

mod common;

use common::{FakeProvider, Fixture, Step, VecSink};
use pec_core::Core;
use pec_core::chat::LocalClock;
use pec_core::dream::DreamState;
use pec_core::llm::{ProviderError, TurnRequest};
use serde_json::{Value, json};

fn on(day: &str, time: &str) -> LocalClock {
    LocalClock {
        iso: format!("{day}T{time}:00+07:00"),
        zone: "Asia/Jakarta".into(),
    }
}

/// A set-up companion whose journey began on `day`.
fn companion(f: &Fixture, provider: &FakeProvider, day: &str) -> Core {
    let core = f.ready(provider);
    f.write("../state/state.json", r#"{"setup":{"complete":true}}"#);
    core.prepare_world(&on(day, "08:00")).unwrap();
    core
}

async fn say(core: &Core, provider: &FakeProvider, text: &str, day: &str) {
    core.send_message(provider, text, &on(day, "19:40"), &VecSink::default())
        .await
        .unwrap();
}

fn json_reply(value: Value) -> Vec<Step> {
    vec![Step::Blocks(vec![
        json!({ "type": "text", "text": value.to_string() }),
    ])]
}

fn topic(keyword: &str, line: &str, subjects: &[&str]) -> Value {
    json!({ "keyword": keyword, "line": line, "subjects": subjects })
}

/// The replies of a sweep after new messages: the themes, the diary, then the topics.
fn night(topics: Vec<Step>) -> Vec<Vec<Step>> {
    vec![
        json_reply(json!({ "themes": [] })),
        vec![
            Step::Text("I dreamt of a long road."),
            Step::End("end_turn"),
        ],
        topics,
    ]
}

fn is_topics(r: &TurnRequest) -> bool {
    r.system
        .iter()
        .any(|s| s.contains("to draw them in your world"))
}

fn topics_requests(provider: &FakeProvider) -> Vec<TurnRequest> {
    provider.requests().into_iter().filter(is_topics).collect()
}

fn read_or_empty(f: &Fixture, file: &str) -> String {
    std::fs::read_to_string(f.workspace(file)).unwrap_or_default()
}

fn lines(f: &Fixture, file: &str) -> Vec<Value> {
    read_or_empty(f, file)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn topic_objects(f: &Fixture) -> Vec<Value> {
    lines(f, "world/world.jsonl")
        .into_iter()
        .filter(|l| l["why"].as_str().is_some_and(|w| w.starts_with("topic:")))
        .collect()
}

fn db(f: &Fixture) -> rusqlite::Connection {
    rusqlite::Connection::open(f.root().join("state/pec.db")).unwrap()
}

#[tokio::test]
async fn a_day_told_about_gets_its_objects_beside_its_scenery_once() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-09");
    say(
        &core,
        &provider,
        "Just back from a 10 km run before work!",
        "2026-11-10",
    )
    .await;
    say(
        &core,
        &provider,
        "Baked a cake for my sister's birthday tomorrow.",
        "2026-11-10",
    )
    .await;
    provider.script(night(json_reply(json!({ "days": [{ "day": "2026-11-10", "topics": [
        topic("10 km run", "You ran ten kilometres before work.", &["running"]),
        topic("sister's birthday", "You baked a cake for your sister's birthday.", &["celebration", "cooking"]),
    ] }] }))));
    let status = core
        .dream_now(&provider, &on("2026-11-11", "03:00"))
        .await
        .unwrap();
    assert!(status.last_run.unwrap().completed);

    // One request, of the documented shape, with only what the person wrote.
    let requests = topics_requests(&provider);
    assert_eq!(requests.len(), 1, "journey FR-006: one more request");
    let request = &requests[0];
    assert_eq!(request.system.len(), 2);
    assert!(request.tools.is_empty());
    let schema = request.format.as_ref().unwrap();
    let subjects = &schema["properties"]["days"]["items"]["properties"]["topics"]["items"]["properties"]
        ["subjects"];
    assert!(
        subjects["items"]["enum"]
            .as_array()
            .unwrap()
            .contains(&json!("running"))
    );
    let input = request.messages[0]["content"][0]["text"].as_str().unwrap();
    assert!(input.contains("## 2026-11-10") && input.contains("19:40 Just back from a 10 km run"));
    assert!(!input.contains("Hello!"), "never what the companion said");

    // Two objects of the topics' subjects, dated the day, beside that day's scenery.
    let objects = topic_objects(&f);
    assert_eq!(objects.len(), 2);
    let scenery: Vec<Value> = lines(&f, "world/world.jsonl")
        .into_iter()
        .filter(|l| l["day"] == "2026-11-10" && l["why"] == "ambient")
        .collect();
    for (n, object) in objects.iter().enumerate() {
        assert_eq!(object["day"], "2026-11-10");
        assert_eq!(object["why"], format!("topic:2026-11-10/{n}"));
        let near = |o: &Value| {
            (o["x"].as_i64().unwrap() - object["x"].as_i64().unwrap()).abs() <= 3
                && (o["y"].as_i64().unwrap() - object["y"].as_i64().unwrap()).abs() <= 3
        };
        assert!(
            scenery
                .iter()
                .chain(&objects)
                .any(|o| o != object && near(o)),
            "{object}"
        );
    }
    let journey = lines(&f, "world/journey.jsonl");
    assert_eq!(journey.len(), 1);
    let told = &journey[0]["topics"];
    assert_eq!(told[0]["keyword"], "10 km run");
    assert_eq!(told[1]["subjects"], json!(["celebration", "cooking"]));
    assert_eq!(told[0]["item"], objects[0]["item"]);
    assert_eq!(f.state()["world"]["journeyDay"], "2026-11-10");

    // A second sweep draws nothing more; "undo last dream" leaves the world and the journey alone.
    let (world, journal) = (
        read_or_empty(&f, "world/world.jsonl"),
        read_or_empty(&f, "world/journey.jsonl"),
    );
    core.dream_now(&provider, &on("2026-11-11", "09:00"))
        .await
        .unwrap();
    assert_eq!(topics_requests(&provider).len(), 1);
    core.undo_last_dream(true).unwrap();
    assert_eq!(read_or_empty(&f, "world/world.jsonl"), world);
    assert_eq!(read_or_empty(&f, "world/journey.jsonl"), journal);
}

#[tokio::test]
async fn an_unreachable_provider_draws_nothing_and_the_day_waits() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-09");
    say(&core, &provider, "Ran 10 km today.", "2026-11-10").await;
    let world = read_or_empty(&f, "world/world.jsonl");
    provider.script(night(vec![Step::Fail(ProviderError::Unreachable, None)]));
    let status = core
        .dream_now(&provider, &on("2026-11-11", "03:00"))
        .await
        .unwrap();
    assert!(
        status.last_run.unwrap().completed,
        "the run still completes"
    );
    assert_eq!(
        read_or_empty(&f, "world/world.jsonl"),
        world,
        "journey FR-007"
    );
    assert_eq!(read_or_empty(&f, "world/journey.jsonl"), "");

    // The next sweep asks again and draws the day.
    say(&core, &provider, "Quiet day.", "2026-11-11").await;
    provider.script(night(json_reply(
        json!({ "days": [{ "day": "2026-11-10", "topics": [
        topic("10 km run", "You ran ten kilometres.", &["running"]),
    ] }] }),
    )));
    core.dream_now(&provider, &on("2026-11-12", "03:00"))
        .await
        .unwrap();
    assert_eq!(topic_objects(&f).len(), 1, "journey SC-009");
}

#[tokio::test]
async fn a_long_absence_draws_the_seven_most_recent_days_and_never_the_rest() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-01");
    let days: Vec<String> = (1..=10).map(|d| format!("2026-11-{d:02}")).collect();
    for day in &days {
        say(&core, &provider, "Went for a walk by the river.", day).await;
    }
    // "Dream now" during the 11th: today has not ended.
    say(&core, &provider, "Worked from home.", "2026-11-11").await;
    let asked: Vec<&String> = days.iter().rev().take(7).collect();
    let answer: Vec<Value> = asked
        .iter()
        .map(|d| {
            let topics = if d.as_str() == "2026-11-05" {
                vec![]
            } else {
                vec![topic(
                    "river walk",
                    "You walked by the river.",
                    &["walking"],
                )]
            };
            json!({ "day": d, "topics": topics })
        })
        .collect();
    provider.script(night(json_reply(json!({ "days": answer }))));
    core.dream_now(&provider, &on("2026-11-11", "15:00"))
        .await
        .unwrap();
    let input = topics_requests(&provider)[0].messages[0]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_owned();
    for day in &days {
        assert_eq!(
            input.contains(&format!("## {day}")),
            asked.contains(&day),
            "{day}"
        );
    }
    assert!(!input.contains("## 2026-11-11"));
    let journey = lines(&f, "world/journey.jsonl");
    assert_eq!(journey.len(), 7);
    // A day with no topic gets an empty line.
    assert!(
        journey
            .iter()
            .any(|d| d["day"] == "2026-11-05" && d["topics"] == json!([]))
    );
    assert_eq!(topic_objects(&f).len(), 6);

    // The three older days are never asked about, nor the considered ones again.
    core.dream_now(&provider, &on("2026-11-11", "16:00"))
        .await
        .unwrap();
    assert_eq!(topics_requests(&provider).len(), 1);
    // Days before the journey began never get topics.
    let g = Fixture::new();
    let other = FakeProvider::accepting();
    let core = companion(&g, &other, "2026-11-09");
    say(&core, &other, "Went for a walk.", "2026-11-09").await;
    g.write(
        "../state/state.json",
        r#"{"setup":{"complete":true},"world":{"journeySince":"2026-11-10"}}"#,
    );
    other.script(night(json_reply(json!({ "days": [] }))));
    core.dream_now(&other, &on("2026-11-12", "03:00"))
        .await
        .unwrap();
    assert!(topics_requests(&other).is_empty(), "journey FR-005");
}

#[tokio::test]
async fn a_message_while_naming_topics_pauses_with_nothing_written() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-09");
    say(&core, &provider, "Ran 10 km today.", "2026-11-10").await;
    let (world, before) = (read_or_empty(&f, "world/world.jsonl"), f.state());
    provider.script(vec![
        json_reply(json!({ "themes": [] })),
        vec![Step::Text("A dream."), Step::End("end_turn")],
        vec![Step::Hang],
        vec![Step::Text("Right away."), Step::End("end_turn")],
    ]);
    let clock = on("2026-11-11", "07:00");
    let (dreamt, sent) = tokio::join!(core.dream_now(&provider, &clock), async {
        // Let the sweep reach the topics call before the message arrives.
        while !provider.requests().iter().any(is_topics) {
            tokio::task::yield_now().await;
        }
        core.send_message(&provider, "morning!", &clock, &VecSink::default())
            .await
    });
    assert_eq!(dreamt.unwrap().state, DreamState::Paused);
    assert_eq!(sent, Ok(None));
    assert!(
        read_or_empty(&f, "world/world.jsonl").starts_with(&world),
        "journey SC-006"
    );
    assert_eq!(topic_objects(&f).len(), 0);
    assert_eq!(read_or_empty(&f, "world/journey.jsonl"), "");
    assert_eq!(
        f.state()["world"]["journeyDay"],
        before["world"]["journeyDay"]
    );
    provider.script(vec![json_reply(
        json!({ "days": [{ "day": "2026-11-10", "topics": [
        topic("10 km run", "You ran ten kilometres.", &["running"]),
    ] }] }),
    )]);
    core.dream_now(&provider, &clock).await.unwrap();
    assert_eq!(topic_objects(&f).len(), 1);
}

#[tokio::test]
async fn writing_cut_short_is_completed_once_and_nothing_is_removed() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-09");
    say(
        &core,
        &provider,
        "Ran 10 km and baked a cake.",
        "2026-11-10",
    )
    .await;
    provider.script(night(json_reply(
        json!({ "days": [{ "day": "2026-11-10", "topics": [
        topic("10 km run", "You ran ten kilometres.", &["running"]),
        topic("cake", "You baked a cake.", &["cooking"]),
    ] }] }),
    )));
    core.dream_now(&provider, &on("2026-11-11", "03:00"))
        .await
        .unwrap();
    let (world, journal) = (
        read_or_empty(&f, "world/world.jsonl"),
        read_or_empty(&f, "world/journey.jsonl"),
    );

    // As if the app had stopped after the first world line: the second and the journey line are missing, and a
    // scenery line was appended after them.
    let mut kept: Vec<&str> = world.lines().collect();
    let second = kept.pop().unwrap().to_owned();
    let scenery = r#"{"item":"grass.tuft","x":40,"y":40,"size":1.0,"flip":false,"shade":0,"day":"2026-11-11","why":"ambient"}"#;
    f.write(
        "world/world.jsonl",
        &format!("{}\n{scenery}\n", kept.join("\n")),
    );
    f.write("world/journey.jsonl", "");
    db(&f)
        .execute(
            "UPDATE dream_runs SET phase = 'drawing', ended_at = NULL",
            [],
        )
        .unwrap();
    f.core()
        .dream_now(&provider, &on("2026-11-11", "09:00"))
        .await
        .unwrap();

    let now = read_or_empty(&f, "world/world.jsonl");
    assert!(
        now.starts_with(&format!("{}\n{scenery}\n", kept.join("\n"))),
        "nothing removed"
    );
    assert_eq!(now.matches(&second).count(), 1, "the missing line once");
    assert_eq!(topic_objects(&f).len(), 2);
    assert_eq!(read_or_empty(&f, "world/journey.jsonl"), journal);
    assert_eq!(topics_requests(&provider).len(), 1, "no second request");
}

#[tokio::test]
async fn the_view_reads_the_map_as_a_diary() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-09");
    say(
        &core,
        &provider,
        "Ran 10 km and baked a cake.",
        "2026-11-10",
    )
    .await;
    say(&core, &provider, "Just a quiet day.", "2026-11-11").await;
    f.write(
        "memory/2026-11-10.md",
        "# 2026-11-10\n\n- 19:42 Sister's birthday tomorrow.\n",
    );
    provider.script(night(json_reply(json!({ "days": [
        { "day": "2026-11-11", "topics": [] },
        { "day": "2026-11-10", "topics": [
            topic("10 km run", "You ran ten kilometres.", &["running"]),
            topic("sister's birthday", "You baked a cake.", &["celebration"]),
        ] },
    ] }))));
    core.dream_now(&provider, &on("2026-11-12", "03:00"))
        .await
        .unwrap();
    let view = core.world_view().unwrap();

    // journey FR-024: a topic object has its day's keywords, its own first; scenery of that day has them too.
    let second = view
        .items
        .iter()
        .position(|i| i.topic.as_ref().is_some_and(|t| t.n == 1))
        .unwrap();
    assert_eq!(
        view.items[second].keywords,
        ["sister's birthday", "10 km run"]
    );
    let scenery = view
        .items
        .iter()
        .find(|i| i.day == "2026-11-10" && i.topic.is_none())
        .unwrap();
    assert_eq!(scenery.keywords, ["10 km run", "sister's birthday"]);
    assert!(
        view.items
            .iter()
            .filter(|i| i.day == "2026-11-11")
            .all(|i| i.keywords.is_empty())
    );

    // journey FR-026 to FR-029: newest first, the stories beside their objects, the scenery, the notes link.
    let days: Vec<&str> = view.journey.iter().map(|d| d.day.as_str()).collect();
    assert_eq!(days, ["2026-11-11", "2026-11-10"]);
    let tenth = &view.journey[1];
    assert!(tenth.has_notes && !view.journey[0].has_notes);
    assert_eq!(tenth.topics[1].item, Some(second as u32));
    let story = tenth.topics[1].story.as_ref().unwrap();
    assert_eq!(
        (story.keyword.as_str(), story.line.as_str()),
        ("sister's birthday", "You baked a cake.")
    );
    assert!(!story.odds.is_empty());
    assert!(!tenth.scenery.is_empty());
    assert!(
        tenth
            .scenery
            .iter()
            .all(|i| view.items[*i as usize].topic.is_none())
    );
    assert_eq!(view.makeup.len(), 2);
    assert!(!view.journey_hash.is_empty());

    // A journey line whose object is gone has no index; an unreadable journey leaves the picture.
    let world = read_or_empty(&f, "world/world.jsonl");
    let without: Vec<&str> = world
        .lines()
        .filter(|l| !l.contains("topic:2026-11-10/0"))
        .collect();
    f.write("world/world.jsonl", &(without.join("\n") + "\n"));
    assert_eq!(core.world_view().unwrap().journey[1].topics[0].item, None);
    f.write("world/journey.jsonl", "not json\n");
    let view = core.world_view().unwrap();
    assert!(view.journey.is_empty() && view.journey_skipped == 1 && !view.items.is_empty());
}

/// A companion whose 10 November was drawn with a run and a cake.
async fn drawn_tenth(f: &Fixture, provider: &FakeProvider) -> Core {
    let core = companion(f, provider, "2026-11-09");
    say(&core, provider, "Ran 10 km and baked a cake.", "2026-11-10").await;
    provider.script(night(json_reply(
        json!({ "days": [{ "day": "2026-11-10", "topics": [
        topic("10 km run", "You ran ten kilometres.", &["running"]),
        topic("sister's birthday", "You baked a cake.", &["celebration"]),
    ] }] }),
    )));
    core.dream_now(provider, &on("2026-11-11", "03:00"))
        .await
        .unwrap();
    core
}

fn last_user_text(request: &TurnRequest) -> String {
    request.messages.last().unwrap()["content"].to_string()
}

#[tokio::test]
async fn a_new_context_knows_the_world_and_a_running_one_is_told_once() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = companion(&f, &provider, "2026-11-09");
    say(
        &core,
        &provider,
        "Ran 10 km and baked a cake.",
        "2026-11-10",
    )
    .await;
    // The context began before anything was drawn for the 10th.
    let first = provider.requests().last().unwrap().clone();
    assert!(first.system[1].contains("## Your world"));
    assert!(!first.system[1].contains("10 km run"));
    provider.script(night(json_reply(
        json!({ "days": [{ "day": "2026-11-10", "topics": [
        topic("10 km run", "You ran ten kilometres.", &["running"]),
    ] }] }),
    )));
    // The sweep's roll does nothing here: nothing was committed to memory.
    core.dream_now(&provider, &on("2026-11-11", "03:00"))
        .await
        .unwrap();

    // journey FR-038: the next message is told, once, and records how far.
    say(&core, &provider, "Good morning!", "2026-11-11").await;
    let told = provider.requests().last().unwrap().clone();
    let text = last_user_text(&told);
    assert!(
        text.contains("While you dreamt, you drew in your world") && text.contains("10 km run"),
        "{text}"
    );
    let entry = f
        .transcript()
        .into_iter()
        .rev()
        .find(|l| l["role"] == "user")
        .unwrap();
    assert_eq!(entry["world"], "2026-11-10");
    say(&core, &provider, "And again.", "2026-11-11").await;
    assert!(!last_user_text(provider.requests().last().unwrap()).contains("While you dreamt"));
    // journey SC-012: within the context each request is the previous one plus appended messages.
    let chats: Vec<TurnRequest> = provider
        .requests()
        .into_iter()
        .filter(|r| {
            !is_topics(r)
                && r.format.is_none()
                && r.system.len() == 3
                && r.system[1].contains("## Your world")
        })
        .collect();
    assert!(chats.len() >= 3);
    for pair in chats.windows(2) {
        assert_eq!(pair[0].system, pair[1].system);
        assert_eq!(
            pair[1].messages[..pair[0].messages.len()],
            pair[0].messages[..]
        );
    }

    // journey FR-036: a fresh context carries the description with the new object and its story.
    core.start_fresh_context(&on("2026-11-11", "10:00"))
        .unwrap();
    let context = f
        .transcript()
        .into_iter()
        .rev()
        .find(|l| l["type"] == "context")
        .unwrap();
    assert_eq!(context["harness"], "mood-2");
    assert_eq!(context["world"]["through"], "2026-11-10");
    say(&core, &provider, "What did you draw?", "2026-11-11").await;
    let fresh = provider.requests().last().unwrap().clone();
    assert!(fresh.system[1].contains("## Your world") && fresh.system[1].contains("\"10 km run\""));
    assert!(
        fresh.system[0].contains("# Your world"),
        "journey FR-039: the rules say the world is its own"
    );
    assert!(
        !last_user_text(&fresh).contains("While you dreamt"),
        "already in the description"
    );
}

#[tokio::test]
async fn a_context_from_before_the_journey_carries_on_unchanged() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    // No world yet: the context begins without a description.
    let core = f.ready(&provider);
    f.write("../state/state.json", r#"{"setup":{"complete":true}}"#);
    say(&core, &provider, "Hello there.", "2026-11-10").await;
    let before = provider.requests().last().unwrap().clone();
    assert!(!before.system[1].contains("## Your world"));
    core.prepare_world(&on("2026-11-10", "08:00")).unwrap();
    say(&core, &provider, "Ran 10 km today.", "2026-11-10").await;
    provider.script(night(json_reply(
        json!({ "days": [{ "day": "2026-11-10", "topics": [
        topic("10 km run", "You ran ten kilometres.", &["running"]),
    ] }] }),
    )));
    core.dream_now(&provider, &on("2026-11-11", "03:00"))
        .await
        .unwrap();
    assert_eq!(topic_objects(&f).len(), 1);
    say(&core, &provider, "Morning.", "2026-11-11").await;
    let after = provider.requests().last().unwrap().clone();
    assert_eq!(after.system, before.system, "journey FR-037");
    assert!(!last_user_text(&after).contains("While you dreamt"));
    assert!(!core.chat_state().unwrap().context_outdated);
}

fn invalid(field: &str) -> pec_core::AppError {
    pec_core::AppError::InvalidInput {
        field: field.into(),
    }
}

#[tokio::test]
async fn a_story_can_be_forgotten_and_its_drawing_stays() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = drawn_tenth(&f, &provider).await;
    let world = read_or_empty(&f, "world/world.jsonl");
    let hash = core.world_view().unwrap().journey_hash;
    assert_eq!(
        core.forget_topic("2026-11-10", 0, "stale"),
        Err(invalid("changed"))
    );
    assert_eq!(
        core.forget_topic("2026-11-10", 5, &hash),
        Err(invalid("topic"))
    );
    let outcome = core.forget_topic("2026-11-10", 0, &hash).unwrap();
    assert!(
        !outcome.in_current_context,
        "the context began before the night"
    );
    let journal = read_or_empty(&f, "world/journey.jsonl");
    assert!(!journal.contains("10 km run") && !journal.contains("You ran ten kilometres"));
    assert_eq!(
        read_or_empty(&f, "world/world.jsonl"),
        world,
        "the drawing stays"
    );
    let view = core.world_view().unwrap();
    assert!(view.journey[0].topics[0].story.is_none() && view.journey[0].topics[0].item.is_some());
    assert!(
        view.items
            .iter()
            .all(|i| !i.keywords.contains(&"10 km run".to_owned()))
    );
    assert_eq!(
        core.forget_topic("2026-11-10", 0, &view.journey_hash),
        Err(invalid("topic")),
        "already forgotten"
    );

    // journey SC-013: a fresh context's description has no trace of it; one still holding a story says so (journey FR-043).
    core.start_fresh_context(&on("2026-11-11", "10:00"))
        .unwrap();
    let context = f
        .transcript()
        .into_iter()
        .rev()
        .find(|l| l["type"] == "context")
        .unwrap();
    let text = context["world"]["text"].as_str().unwrap().to_owned();
    assert!(
        !text.contains("10 km run") && text.contains("sister's birthday"),
        "{text}"
    );
    let hash = core.world_view().unwrap().journey_hash;
    assert!(
        core.forget_topic("2026-11-10", 1, &hash)
            .unwrap()
            .in_current_context
    );
}

#[tokio::test]
async fn forgetting_a_memory_forgets_its_days_stories_and_the_switch_stops_naming() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = drawn_tenth(&f, &provider).await;
    f.write(
        "memory/2026-11-10.md",
        "# 2026-11-10\n\n- 19:42 Sister's birthday tomorrow. <!-- from:user src:2-0 -->\n",
    );
    let doc = core.memory_read("memory/2026-11-10.md").unwrap();
    let line = pec_core::memory::ForgetTarget::Line { line: 3 };
    core.memory_forget("memory/2026-11-10.md", line, &doc.hash)
        .unwrap();
    let view = core.world_view().unwrap();
    assert!(
        view.journey[0].topics.iter().all(|t| t.story.is_none()),
        "journey FR-042"
    );
    assert_eq!(topic_objects(&f).len(), 2);

    // journey FR-008: with topic drawing off a sweep names nothing; scenery still grows.
    let settings = core.dreaming_settings().unwrap();
    let clock = on("2026-11-11", "12:00");
    core.set_dreaming_settings(settings.enabled, &settings.time, None, false, &clock)
        .unwrap();
    assert!(!core.dreaming_settings().unwrap().world);
    say(&core, &provider, "Long walk today.", "2026-11-12").await;
    let before = topics_requests(&provider).len();
    core.dream_now(&provider, &on("2026-11-13", "03:00"))
        .await
        .unwrap();
    assert_eq!(topics_requests(&provider).len(), before);
    assert!(
        lines(&f, "world/world.jsonl")
            .iter()
            .any(|l| l["day"] == "2026-11-12")
    );
}
