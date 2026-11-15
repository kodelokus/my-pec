mod common;

use common::{FakeProvider, Fixture, KEY, MemoryStore, OTHER_KEY, clock, models};
use pec_core::AppError;
use pec_core::credentials::{KeyCheckOutcome, KeyStatus};
use pec_core::llm::ProviderError;
use pec_core::onboarding::{SetupCompletion, SetupDraft, SetupState, SetupStep};

fn draft(status: pec_core::onboarding::AppStatus) -> SetupDraft {
    match status.setup {
        SetupState::InProgress { draft } => draft,
        SetupState::Complete => panic!("setup is complete"),
    }
}

/// Every file under the data root, read as bytes, so no format can hide the key.
fn all_files(root: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(all_files(&path));
        } else {
            out.push((path.clone(), std::fs::read(&path).unwrap()));
        }
    }
    out
}

#[tokio::test]
async fn a_key_and_the_defaults_make_a_companion() {
    let fx = Fixture::new();
    let core = fx.core();

    let outcome = core
        .submit_key(&FakeProvider::accepting(), &format!("  {KEY}\n"))
        .await
        .unwrap();
    assert_eq!(
        outcome,
        KeyCheckOutcome::Accepted {
            last_four: "WXYZ".into()
        }
    );
    assert_eq!(fx.store.stored().as_deref(), Some(KEY));
    assert_eq!(fx.state()["models"]["items"][0]["id"], "claude-opus-5-5");

    let companion = core.complete_setup(SetupCompletion::New, &clock()).unwrap();
    assert_eq!(
        (companion.name.as_str(), companion.character_id.as_str()),
        ("Pec", "pip")
    );

    let defaults = SetupDraft::default();
    assert_eq!(fx.read("SOUL.md"), defaults.soul.text);
    // The id and birth date setup adds are the world feature's, checked in world_identity.rs.
    assert!(
        fx.read("IDENTITY.md")
            .starts_with("# Identity\n\n- Name: Pec\n- Avatar: pip\n")
    );
    assert_eq!(
        fx.read("USER.md"),
        "# About the user\n\n- Call them: \n- Preferred language: \n"
    );
    assert_eq!(fx.read("MEMORY.md"), "# Memory\n");
    let manifest: serde_json::Value =
        serde_json::from_str(&fx.read("avatar/manifest.json")).unwrap();
    // The avatar feature: version 2; without a bundled model, the still portrait.
    assert_eq!(
        manifest,
        serde_json::json!({ "version": 2, "type": "static", "character": "pip" })
    );

    let state = fx.state();
    assert_eq!(state["setup"]["complete"], true);
    assert_eq!(state["setup"]["draft"], serde_json::Value::Null);
    assert_eq!(state["firstHelloPending"], true);
    assert_eq!(core.app_status().unwrap().setup, SetupState::Complete);

    // onboarding SC-006: the key is in no file the app wrote.
    for (path, bytes) in all_files(fx.root()) {
        assert!(
            !String::from_utf8_lossy(&bytes).contains(KEY),
            "key found in {}",
            path.display()
        );
    }
}

#[tokio::test]
async fn a_key_that_is_not_accepted_is_not_stored() {
    let cases = [
        (Err(ProviderError::KeyRejected), KeyCheckOutcome::Rejected),
        (Err(ProviderError::NoCredit), KeyCheckOutcome::NoCredit),
        (
            Err(ProviderError::Unreachable),
            KeyCheckOutcome::Unreachable,
        ),
    ];
    for (answer, expected) in cases {
        let fx = Fixture::new();
        let outcome = fx
            .core()
            .submit_key(&FakeProvider::answering(answer), KEY)
            .await
            .unwrap();
        assert_eq!(outcome, expected);
        assert_eq!(fx.store.stored(), None, "{expected:?}");
    }
}

#[tokio::test]
async fn an_incomplete_value_never_reaches_the_provider() {
    let fx = Fixture::new();
    let provider = FakeProvider::accepting();
    let outcome = fx.core().submit_key(&provider, "abc").await.unwrap();
    assert_eq!(outcome, KeyCheckOutcome::Incomplete);
    assert_eq!(provider.calls(), 0);
}

#[tokio::test]
async fn a_credential_store_that_refuses_leaves_the_key_nowhere() {
    let fx = Fixture::with_store(MemoryStore {
        refuse: true,
        ..Default::default()
    });
    let outcome = fx
        .core()
        .submit_key(&FakeProvider::accepting(), KEY)
        .await
        .unwrap();
    assert_eq!(outcome, KeyCheckOutcome::StorageUnavailable);
    assert_eq!(fx.store.stored(), None);
    for (path, bytes) in all_files(fx.root()) {
        assert!(
            !String::from_utf8_lossy(&bytes).contains(KEY),
            "key found in {}",
            path.display()
        );
    }
}

#[tokio::test]
async fn a_customized_draft_is_written_as_entered() {
    let fx = Fixture::new();
    let core = fx.core();
    fx.store_key(KEY);
    let soul = format!("# Soul\n\nBe gentle.  \n{}", "x".repeat(9_000));
    let mut d = SetupDraft {
        step: SetupStep::AboutYou,
        ..Default::default()
    };
    d.soul.template_id = "blank".into();
    d.soul.text.clone_from(&soul);
    d.soul.name = "  Ada ".into();
    d.about.call_me = Some(" Sam ".into());
    d.about.language = Some("Bahasa Indonesia".into());
    core.save_setup_draft(d).unwrap();

    let companion = core.complete_setup(SetupCompletion::New, &clock()).unwrap();

    assert_eq!(companion.name, "Ada");
    assert_eq!(fx.read("SOUL.md"), soul);
    assert!(fx.read("IDENTITY.md").contains("- Name: Ada\n"));
    assert_eq!(
        fx.read("USER.md"),
        "# About the user\n\n- Call them: Sam\n- Preferred language: Bahasa Indonesia\n"
    );
}

#[test]
fn the_wizard_resumes_where_it_was_left() {
    for step in [
        SetupStep::Key,
        SetupStep::Soul,
        SetupStep::Avatar,
        SetupStep::AboutYou,
    ] {
        let fx = Fixture::new();
        fx.store_key(KEY);
        let mut d = SetupDraft {
            step,
            ..Default::default()
        };
        d.soul.text = format!("typed on {step:?}");
        d.about.call_me = Some("Sam".into());
        fx.core().save_setup_draft(d.clone()).unwrap();

        // A new Core reads everything back from disk, as a relaunch does.
        let status = fx.core().app_status().unwrap();
        assert_eq!(draft(status), d, "{step:?}");
    }
}

#[test]
fn without_a_key_the_wizard_stays_on_the_key_step() {
    let fx = Fixture::new();
    let core = fx.core();
    let status = core.app_status().unwrap();
    assert_eq!(status.key, KeyStatus::Absent);
    assert_eq!(draft(status).step, SetupStep::Key);

    let mut d = SetupDraft {
        step: SetupStep::Soul,
        ..Default::default()
    };
    assert_eq!(
        core.save_setup_draft(d.clone()),
        Err(AppError::InvalidInput {
            field: "step".into()
        })
    );
    assert_eq!(
        core.complete_setup(SetupCompletion::New, &clock()),
        Err(AppError::InvalidInput {
            field: "step".into()
        })
    );

    // The key disappears while the wizard is past the key step: back to Key, entries kept.
    fx.store_key(KEY);
    d.soul.text = "kept".into();
    core.save_setup_draft(d).unwrap();
    core.remove_key().unwrap();
    let back = draft(core.app_status().unwrap());
    assert_eq!(
        (back.step, back.soul.text.as_str()),
        (SetupStep::Key, "kept")
    );
}

#[test]
fn the_close_notice_is_remembered() {
    let fx = Fixture::new();
    assert!(!fx.core().close_notice_shown().unwrap());
    fx.core().mark_close_notice_shown().unwrap();
    assert!(fx.core().close_notice_shown().unwrap());
}

// NFR-I18N-1: English until the person picks another language the app has text for.
#[test]
fn the_interface_language_is_remembered_and_only_a_known_one() {
    let fx = Fixture::new();
    assert_eq!(fx.core().language(), "en");
    assert_eq!(fx.core().set_language("id").unwrap(), "id");
    assert_eq!(fx.core().language(), "id");
    assert!(fx.core().set_language("xx").is_err());
    assert_eq!(fx.core().language(), "id");
}

#[tokio::test]
async fn replacing_and_removing_the_key() {
    let fx = Fixture::new();
    let core = fx.core();
    core.submit_key(&FakeProvider::accepting(), KEY)
        .await
        .unwrap();
    core.complete_setup(SetupCompletion::New, &clock()).unwrap();

    let rejecting = FakeProvider::answering(Err(ProviderError::KeyRejected));
    assert_eq!(
        core.submit_key(&rejecting, OTHER_KEY).await.unwrap(),
        KeyCheckOutcome::Rejected
    );
    assert_eq!(fx.store.stored().as_deref(), Some(KEY));

    core.submit_key(&FakeProvider::accepting(), OTHER_KEY)
        .await
        .unwrap();
    assert_eq!(
        core.key_status(),
        KeyStatus::Present {
            last_four: "QRST".into()
        }
    );

    assert_eq!(core.remove_key().unwrap(), KeyStatus::Absent);
    let status = core.app_status().unwrap();
    assert_eq!(
        (status.key, status.setup),
        (KeyStatus::Absent, SetupState::Complete)
    );
}

#[tokio::test]
async fn a_failed_refresh_returns_the_last_known_list_and_the_reason() {
    for problem in [
        ProviderError::Unreachable,
        ProviderError::KeyRejected,
        ProviderError::NoCredit,
    ] {
        let fx = Fixture::new();
        let core = fx.core();
        let provider = FakeProvider::accepting();
        core.submit_key(&provider, KEY).await.unwrap();

        provider.set(Err(problem));
        let list = core.model_list(&provider, true).await.unwrap();
        assert_eq!(
            list.models,
            models(&["claude-opus-5-5", "claude-sonnet-5-5"])
        );
        assert_eq!((list.stale, list.problem), (true, Some(problem)));
    }
}

#[tokio::test]
async fn the_model_choice_follows_the_providers_list() {
    let fx = Fixture::new();
    let core = fx.core();
    let provider = FakeProvider::accepting();
    core.submit_key(&provider, KEY).await.unwrap();

    let fresh = core.model_list(&provider, true).await.unwrap();
    assert_eq!(
        (fresh.selected.as_str(), fresh.stale, fresh.problem),
        ("claude-opus-5-5", false, None)
    );

    assert_eq!(
        core.select_model("nope"),
        Err(AppError::InvalidInput { field: "id".into() })
    );
    core.select_model("claude-sonnet-5-5").unwrap();
    assert_eq!(
        fx.core()
            .model_list(&provider, false)
            .await
            .unwrap()
            .selected,
        "claude-sonnet-5-5"
    );

    // The chosen model is withdrawn: the default takes its place and the change is reported once.
    provider.set(Ok(models(&["claude-opus-5-5"])));
    let changed = core.model_list(&provider, true).await.unwrap();
    assert_eq!(
        (changed.selected.as_str(), changed.fallback_from.as_deref()),
        ("claude-opus-5-5", Some("claude-sonnet-5-5"))
    );
    assert_eq!(
        core.model_list(&provider, true)
            .await
            .unwrap()
            .fallback_from,
        None
    );
}
