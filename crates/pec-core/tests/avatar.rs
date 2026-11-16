//! The avatar on a temporary workspace, a temporary characters folder with stand-in model files (the core never
//! parses them) and the fake provider: the manifest and its files, the mood marker, and the state over a turn and a
//! dream (avatar R11).

mod common;

use common::{FakeProvider, Fixture, KEY, Step, VecSink, clock, tool_use};
use pec_core::avatar::{Activity, AvatarLook, AvatarProblem, AvatarSource, AvatarState, Mood};
use pec_core::chat::ChatEvent;
use pec_core::llm::ProviderError;
use pec_core::onboarding::SetupCompletion;
use pec_core::{AppError, Core};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A characters folder with Pip's real template and stand-in model files.
fn characters(f: &Fixture) -> PathBuf {
    let dir = f.root().join("characters");
    let pip = dir.join("pip");
    std::fs::create_dir_all(pip.join("live2d/textures")).unwrap();
    let template = include_str!("../../../characters/pip/avatar.json");
    std::fs::write(pip.join("avatar.json"), template).unwrap();
    for (file, bytes) in [
        ("pip.model3.json", "{}"),
        ("pip.moc3", "moc"),
        ("textures/t.png", "png"),
    ] {
        std::fs::write(pip.join("live2d").join(file), bytes).unwrap();
    }
    dir
}

/// A companion right after setup, with the bundled model or without any.
fn set_up(f: &Fixture, with_model: bool) -> Core {
    f.store_key(KEY);
    let core = match with_model {
        true => f.core().with_characters(characters(f)),
        false => f.core(),
    };
    core.complete_setup(SetupCompletion::New, &clock()).unwrap();
    core
}

fn invalid(field: &str) -> AppError {
    AppError::InvalidInput {
        field: field.into(),
    }
}

fn manifest(f: &Fixture) -> Value {
    serde_json::from_str(&f.read("avatar/manifest.json")).unwrap()
}

fn is_live2d(look: &AvatarLook) -> bool {
    matches!(look, AvatarLook::Live2d { character, model, .. } if character == "pip" && model == "pip/pip.model3.json")
}

#[test]
fn setup_copies_the_character_into_the_workspace_and_writes_version_2() {
    let f = Fixture::new();
    let core = f.core().with_characters(characters(&f));
    assert!(
        is_live2d(&core.avatar_manifest().unwrap().look),
        "before setup"
    );
    f.store_key(KEY);
    core.complete_setup(SetupCompletion::New, &clock()).unwrap();
    assert_eq!(manifest(&f)["version"], 2);
    assert_eq!(manifest(&f)["type"], "live2d");
    assert_eq!(f.read("avatar/pip/pip.moc3"), "moc");
    let file = core.avatar_file(&AvatarSource::Workspace, "pip/textures/t.png");
    assert_eq!(file.unwrap(), b"png");
    assert_eq!(
        core.avatar_file(&AvatarSource::Workspace, "pip/gone.png"),
        Err(invalid("missing"))
    );

    let f = Fixture::new();
    set_up(&f, false);
    assert_eq!(
        manifest(&f),
        json!({ "version": 2, "type": "static", "character": "pip" })
    );
}

#[test]
fn files_are_read_only_inside_the_avatar_folders() {
    let f = Fixture::new();
    let core = set_up(&f, true);
    let pip = AvatarSource::Bundled {
        character: "pip".into(),
    };
    assert_eq!(core.avatar_file(&pip, "pip/pip.moc3").unwrap(), b"moc");
    let refused = [
        (AvatarSource::Workspace, "../state/settings.json"),
        (AvatarSource::Workspace, "pip\\..\\..\\state\\state.json"),
        (AvatarSource::Workspace, "C:/Windows/win.ini"),
        (AvatarSource::Workspace, "/manifest.json"),
        (AvatarSource::Workspace, "pip/pip.moc3.txt"),
        (AvatarSource::Workspace, ""),
        (pip.clone(), "other/pip.moc3"),
        (pip.clone(), "pip/../avatar.json"),
        (
            AvatarSource::Bundled {
                character: "ozzy".into(),
            },
            "ozzy/ozzy.moc3",
        ),
    ];
    for (source, path) in refused {
        assert_eq!(
            core.avatar_file(&source, path),
            Err(invalid("path")),
            "{source:?} {path}"
        );
    }
}

#[test]
fn the_onboarding_manifest_is_upgraded_once_and_edits_are_never_rewritten() {
    let f = Fixture::new();
    let core = set_up(&f, true);
    f.write(
        "avatar/manifest.json",
        "{\n  \"type\": \"static\",\n  \"character\": \"pip\"\n}\n",
    );
    let before = core.avatar_state().look;
    assert!(is_live2d(&core.avatar_manifest().unwrap().look));
    assert_eq!(manifest(&f)["version"], 2);
    assert!(core.avatar_state().look > before, "windows read it again");

    let edited = r#"{"type":"static","character":"pip","note":"mine"}"#;
    f.write("avatar/manifest.json", edited);
    let look = core.avatar_state().look;
    let read = core.avatar_manifest().unwrap();
    assert!(matches!(read.look, AvatarLook::Static { .. }) && read.problem.is_none());
    for text in [edited, "{oops"] {
        f.write("avatar/manifest.json", text);
        core.avatar_manifest().unwrap();
        assert_eq!(f.read("avatar/manifest.json"), text, "never rewritten");
    }
    assert_eq!(
        core.avatar_manifest().unwrap().problem,
        Some(AvatarProblem::Unreadable)
    );
    assert_eq!(core.avatar_state().look, look, "a hand edit bumps nothing");
}

#[test]
fn choosing_a_look_repairs_and_falls_back_as_the_files_require() {
    let f = Fixture::new();
    let core = set_up(&f, true);
    let still = core.choose_avatar("pip", true).unwrap();
    assert_eq!(
        still.look,
        AvatarLook::Static {
            character: "pip".into()
        }
    );
    assert!(f.read("IDENTITY.md").contains("- Avatar: pip\n"));
    assert_eq!(core.choose_avatar("ozzy", false), Err(invalid("character")));
    assert!(is_live2d(&core.choose_avatar("pip", false).unwrap().look));

    // avatar FR-032: missing files come back from the bundle; a character the app no longer has gives way to the default.
    std::fs::remove_dir_all(f.workspace("avatar/pip")).unwrap();
    assert_eq!(core.avatar_manifest().unwrap().problem, None);
    assert_eq!(f.read("avatar/pip/pip.moc3"), "moc");
    f.write(
        "avatar/manifest.json",
        r#"{"version":2,"type":"live2d","character":"ozzy","model":"ozzy/ozzy.model3.json"}"#,
    );
    let gone = core.avatar_manifest().unwrap();
    assert!(is_live2d(&gone.look));
    assert_eq!(gone.problem, Some(AvatarProblem::CharacterGone));
    assert_eq!(manifest(&f)["character"], "pip");
    assert_eq!(core.choose_avatar("pip", false).unwrap().problem, None);
}

#[test]
fn an_own_picture_is_checked_by_its_bytes_and_keeps_the_character() {
    let f = Fixture::new();
    let core = set_up(&f, true);
    let png = b"\x89PNG\r\n\x1a\n rest".to_vec();
    let jpeg = b"\xff\xd8\xff\xe0 rest".to_vec();
    let picture = core.set_avatar_picture(&png).unwrap();
    assert_eq!(
        picture.look,
        AvatarLook::Picture {
            character: "pip".into(),
            image: "picture.png".into()
        }
    );
    core.set_avatar_picture(&jpeg).unwrap();
    assert!(f.workspace("avatar/picture.jpg").is_file());
    assert!(
        !f.workspace("avatar/picture.png").exists(),
        "the older one goes"
    );
    let manifest_before = f.read("avatar/manifest.json");
    for (bytes, field) in [
        (b"just text, named .png".to_vec(), "pictureType"),
        (
            [png.clone(), vec![0; 5 * 1024 * 1024]].concat(),
            "pictureSize",
        ),
    ] {
        assert_eq!(core.set_avatar_picture(&bytes), Err(invalid(field)));
    }
    assert_eq!(f.read("avatar/manifest.json"), manifest_before);
}

/// A step that records the avatar state at that moment of the reply.
fn capture(core: &Arc<Core>, seen: &Arc<Mutex<Vec<AvatarState>>>) -> Step {
    let (core, seen) = (core.clone(), seen.clone());
    Step::Do(Arc::new(move || {
        seen.lock().unwrap().push(core.avatar_state())
    }))
}

async fn send(core: &Core, provider: &FakeProvider, text: &str) -> VecSink {
    let sink = VecSink::default();
    assert_eq!(
        core.send_message(provider, text, &clock(), &sink).await,
        Ok(None)
    );
    sink
}

#[tokio::test]
async fn the_marker_sets_the_mood_out_of_sight_and_the_state_follows_the_turn() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = Arc::new(f.ready(&provider));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let first = "<mood>concerned</mood>\nLet me check.";
    provider.script(vec![
        vec![
            capture(&core, &seen),
            Step::Text("<mood>conc"),
            Step::Text("erned</mood>\nLet me "),
            capture(&core, &seen),
            Step::Text("check."),
            Step::Tools(vec![
                json!({ "type": "text", "text": first }),
                tool_use("t1", "memory_search", json!({ "query": "keys" })),
            ]),
        ],
        vec![
            capture(&core, &seen),
            Step::Text("<mood>happy</mood>\nFound it!"),
            capture(&core, &seen),
            Step::End("end_turn"),
        ],
    ]);
    let sink = send(&core, &provider, "Where are my keys?").await;
    assert_eq!(sink.deltas(), "Let me check.\n\nFound it!");
    let moods: Vec<Mood> = sink
        .events()
        .into_iter()
        .filter_map(|e| match e {
            ChatEvent::Mood { mood } => Some(mood),
            _ => None,
        })
        .collect();
    assert_eq!(moods, [Mood::Concerned, Mood::Happy]);
    let states: Vec<(Activity, Mood, u32)> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|s| (s.activity, s.mood, s.voice))
        .collect();
    assert_eq!(
        states,
        [
            (Activity::Thinking, Mood::Neutral, 0),
            (Activity::Speaking, Mood::Concerned, 7),
            (Activity::Thinking, Mood::Concerned, 13),
            (Activity::Speaking, Mood::Happy, 22),
        ]
    );
    let end = core.avatar_state();
    assert_eq!((end.activity, end.mood), (Activity::Idle, Mood::Happy));

    // Principle III: stored byte for byte; avatar FR-009: never in visible text.
    let replies: Vec<Value> = f
        .transcript()
        .into_iter()
        .filter(|l| l["role"] == "assistant")
        .map(|l| l["content"][0]["text"].clone())
        .collect();
    assert_eq!(replies, [first, "<mood>happy</mood>\nFound it!"]);
    let history = core.chat_history(None, 10).unwrap().messages;
    assert_eq!(history.last().unwrap().text, "Let me check.\n\nFound it!");

    // avatar FR-011: a reply without a marker is neutral.
    send(&core, &provider, "Thanks").await;
    assert_eq!(core.avatar_state().mood, Mood::Neutral);
}

#[tokio::test]
async fn however_a_turn_ends_the_character_is_idle_and_listening_waits_for_it() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = Arc::new(f.ready(&provider));
    let idle = |c: &Core| c.avatar_state().activity == Activity::Idle;

    provider.script(vec![vec![Step::Text("<mood>sad</mood>\nWai"), Step::Hang]]);
    let stop = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        core.stop_reply();
    };
    let (sent, ()) = tokio::join!(send(&core, &provider, "one"), stop);
    drop(sent);
    assert!(idle(&core), "stopped");
    for script in [
        vec![Step::Text("No."), Step::End("refusal")],
        vec![Step::Fail(ProviderError::Unreachable, None)],
    ] {
        provider.script(vec![script]);
        core.send_message(&provider, "two", &clock(), &VecSink::default())
            .await
            .unwrap();
        assert!(idle(&core));
    }

    // avatar FR-004: listening is cleared when a turn starts and ignored while it runs.
    core.set_listening(true);
    assert_eq!(core.avatar_state().activity, Activity::Listening);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let listen = {
        let core = core.clone();
        Step::Do(Arc::new(move || core.set_listening(true)))
    };
    provider.script(vec![vec![
        listen,
        capture(&core, &seen),
        Step::Text("Hi."),
        Step::End("end_turn"),
    ]]);
    core.retry_reply(&provider, &clock(), &VecSink::default())
        .await
        .unwrap();
    assert_eq!(seen.lock().unwrap()[0].activity, Activity::Thinking);
    assert!(idle(&core));
}

#[tokio::test]
async fn the_character_sleeps_while_it_dreams_and_wakes_for_a_message() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = Arc::new(f.ready(&provider));
    provider.script(vec![vec![
        Step::Text("<mood>happy</mood>\nNight!"),
        Step::End("end_turn"),
    ]]);
    send(&core, &provider, "good night").await;
    f.write(
        "memory/2026-11-12.md",
        "# 2026-11-12\n\n- 09:00 Training for the half marathon. <!-- from:companion src:1-0 -->\n",
    );
    let seen = Arc::new(Mutex::new(Vec::new()));
    // The dream's first call hangs until the message pauses the sweep (dreaming SC-008).
    provider.script(vec![
        vec![capture(&core, &seen), Step::Hang],
        vec![
            capture(&core, &seen),
            Step::Text("<mood>playful</mood>\nAwake!"),
            Step::End("end_turn"),
        ],
    ]);
    let now = clock();
    let (dreamt, _) = tokio::join!(core.dream_now(&provider, &now), async {
        tokio::task::yield_now().await;
        send(&core, &provider, "still up?").await
    });
    dreamt.unwrap();
    let states: Vec<(Activity, Mood)> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|s| (s.activity, s.mood))
        .collect();
    assert_eq!(
        states,
        [
            (Activity::Sleeping, Mood::Happy),
            (Activity::Thinking, Mood::Happy)
        ]
    );
    let end = core.avatar_state();
    assert_eq!((end.activity, end.mood), (Activity::Idle, Mood::Playful));
}

#[tokio::test]
async fn a_new_context_asks_for_the_marker_and_an_older_one_keeps_its_rules() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    send(&core, &provider, "one").await;
    let new = provider.requests().pop().unwrap();
    assert!(new.system[0].contains("Start every reply with your mood on its own line"));
    let context = f
        .transcript()
        .into_iter()
        .find(|l| l["type"] == "context")
        .unwrap();
    assert_eq!(context["harness"], "mood-2");

    // A context from before keeps the rules it started with, byte for byte (Principle III).
    let mut old = context.clone();
    old["harness"] = "world-1".into();
    f.write("sessions/main.jsonl", &format!("{old}\n"));
    let core = f.core();
    send(&core, &provider, "two").await;
    let request = provider.requests().pop().unwrap();
    let rules = include_str!("../src/chat/harness-world-1.md");
    assert_eq!(request.system[0], rules);
}
