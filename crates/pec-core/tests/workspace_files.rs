mod common;

use common::{Fixture, KEY, clock};
use pec_core::AppError;
use pec_core::onboarding::{SetupCompletion, SetupDraft, SetupState};
use std::fs;

fn completed() -> Fixture {
    let fx = Fixture::new();
    fx.store_key(KEY);
    fx.core()
        .complete_setup(SetupCompletion::New, &clock())
        .unwrap();
    fx
}

#[test]
fn a_hand_edit_is_honored_and_never_overwritten() {
    let fx = completed();
    let edited = "# Identity\n\nMy notes.\n- Name: Bob\n- Avatar: pip\n";
    fs::write(fx.workspace("IDENTITY.md"), edited).unwrap();
    fs::write(fx.workspace("USER.md"), "- Call them: Sam\n").unwrap();

    let status = fx.core().app_status().unwrap();

    assert_eq!(status.companion.unwrap().name, "Bob");
    assert_eq!(status.call_me.as_deref(), Some("Sam"));
    assert_eq!(fx.read("IDENTITY.md"), edited);
}

#[test]
fn a_failed_write_leaves_setup_unfinished_and_nothing_half_written() {
    let fx = Fixture::new();
    fx.store_key(KEY);
    // A file where the workspace folder should be: nothing can be created inside it.
    fs::write(fx.root().join("workspace"), "in the way").unwrap();

    let err = fx
        .core()
        .complete_setup(SetupCompletion::New, &clock())
        .unwrap_err();

    assert!(
        matches!(err, AppError::WorkspaceNotWritable { .. }),
        "{err:?}"
    );
    assert!(matches!(
        fx.core().app_status().unwrap().setup,
        SetupState::InProgress { .. }
    ));
    let entries: Vec<_> = fs::read_dir(fx.root())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries, ["workspace"]);
}

#[test]
fn an_existing_companion_can_be_kept_or_replaced() {
    let original = "# Identity\n\n- Name: Old Friend\n- Avatar: pip\n";
    let existing = || {
        let fx = Fixture::new();
        fx.store_key(KEY);
        fs::create_dir_all(fx.workspace("")).unwrap();
        fs::write(fx.workspace("IDENTITY.md"), original).unwrap();
        fs::write(fx.workspace("SOUL.md"), "old soul").unwrap();
        fx
    };

    let fx = existing();
    let status = fx.core().app_status().unwrap();
    assert!(status.existing_workspace);
    assert_eq!(status.companion.unwrap().name, "Old Friend");
    let refused = fx.core().complete_setup(SetupCompletion::New, &clock());
    assert_eq!(
        refused,
        Err(AppError::InvalidInput {
            field: "mode".into()
        })
    );

    let fx = existing();
    assert_eq!(
        fx.core()
            .complete_setup(SetupCompletion::AdoptExisting, &clock())
            .unwrap()
            .name,
        "Old Friend"
    );
    assert_eq!(fx.read("IDENTITY.md"), original);
    assert_eq!(fx.read("SOUL.md"), "old soul");
    assert!(!fx.workspace("MEMORY.md").exists());
    assert_eq!(fx.state()["firstHelloPending"], false);

    let fx = existing();
    assert_eq!(
        fx.core()
            .complete_setup(SetupCompletion::ReplaceExisting, &clock())
            .unwrap()
            .name,
        "Pec"
    );
    let backups: Vec<_> = fs::read_dir(fx.root())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("workspace-backup-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        fs::read_to_string(backups[0].join("IDENTITY.md")).unwrap(),
        original
    );
    assert_eq!(fx.read("SOUL.md"), SetupDraft::default().soul.text);
    assert_eq!(fx.state()["firstHelloPending"], true);
}

#[test]
fn a_deleted_file_comes_back_and_the_others_are_untouched() {
    let fx = completed();
    fs::write(fx.workspace("SOUL.md"), "my own soul").unwrap();
    fs::write(fx.workspace("IDENTITY.md"), "- Name: Bob\n- Avatar: pip\n").unwrap();
    fs::remove_file(fx.workspace("MEMORY.md")).unwrap();
    fs::remove_file(fx.workspace("avatar/manifest.json")).unwrap();

    let status = fx.core().app_status().unwrap();

    assert_eq!(
        status.recreated_files,
        ["MEMORY.md", "avatar/manifest.json"]
    );
    assert_eq!(fx.read("MEMORY.md"), "# Memory\n");
    assert_eq!(fx.read("SOUL.md"), "my own soul");
    assert_eq!(fx.read("IDENTITY.md"), "- Name: Bob\n- Avatar: pip\n");
    assert!(fx.core().app_status().unwrap().recreated_files.is_empty());
}
