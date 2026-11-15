//! Every rule and every file of My Pec. No Tauri dependency (Principle VI).

pub mod avatar;
pub mod chat;
pub mod compaction;
mod config;
pub mod credentials;
pub mod dream;
mod error;
pub mod ext;
pub mod llm;
pub mod memory;
pub mod model_choice;
pub mod onboarding;
mod workspace;
pub mod world;

pub use error::{AppError, Result};

use credentials::SecretStore;
use std::path::PathBuf;
use std::sync::Mutex;

/// The data root and the credential store. Each command is a method on it.
pub struct Core {
    root: PathBuf,
    secrets: Box<dyn SecretStore>,
    // Commands run on several threads; state.json is read, changed and written under this lock.
    state_lock: Mutex<()>,
    chat: chat::ChatRuntime,
    memory: memory::MemoryRuntime,
    avatar: avatar::AvatarRuntime,
    ext: ext::ExtRuntime,
    /// The bundled characters, when the app has any (avatar R5).
    characters: Option<PathBuf>,
    /// When this core started, in Unix milliseconds: idleness counts from it at launch (dreaming R3).
    started: u64,
}

impl Core {
    pub fn new(root: impl Into<PathBuf>, secrets: Box<dyn SecretStore>) -> Self {
        Self {
            root: root.into(),
            secrets,
            state_lock: Mutex::new(()),
            chat: chat::ChatRuntime::default(),
            memory: memory::MemoryRuntime::default(),
            avatar: avatar::AvatarRuntime::default(),
            ext: ext::ExtRuntime::default(),
            characters: None,
            started: chat::now_ms(),
        }
    }

    /// The folder of the bundled characters, `characters/<id>/` (avatar R5).
    pub fn with_characters(mut self, dir: impl Into<PathBuf>) -> Self {
        self.characters = Some(dir.into());
        self
    }
}
