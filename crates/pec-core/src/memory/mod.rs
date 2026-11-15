//! The companion's memory: the files and their writer, the index, and the tools (memory R5 to R15).

pub(crate) mod browser;
mod download;
pub(crate) mod files;
pub(crate) mod index;
pub(crate) mod search;
mod semantic;
pub(crate) mod tools;

use crate::{AppError, Core, Result};
pub use browser::{
    FileKind, ForgetOutcome, ForgetTarget, MemoryDoc, MemoryEntry, MemoryFile, SaveOutcome,
    SearchHit, UndoOutcome,
};
pub use files::fingerprint;
use index::Index;
use semantic::SemanticRuntime;
pub use semantic::{Embedder, EmbedderLoader, SemanticProblem, SemanticState, SemanticStatus};
use serde::Serialize;
use std::sync::Mutex;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, serde::Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MarkKind {
    Note,
    Remembered,
    Profile,
    /// Long-term memory or the profile was full, so the fact went to today's note (memory FR-011).
    KeptInNote,
    /// extensions R3: changes made through an extension's tools.
    NoteMade,
    NoteChanged,
    AlarmSet,
    AlarmChanged,
    AlarmCancelled,
}

/// A write by the companion, shown on the reply that made it (memory FR-008).
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemoryMark {
    pub kind: MarkKind,
    pub path: String,
    pub line: Option<u32>,
    pub excerpt: String,
    pub ok: bool,
}

/// The memory write lock and the lazily opened index.
#[derive(Default)]
pub(crate) struct MemoryRuntime {
    write: Mutex<()>,
    index: Mutex<Option<Index>>,
    /// Semantic search: the model, its vectors and its download.
    semantic: SemanticRuntime,
}

impl Core {
    /// Tests: search by meaning with a stand-in embedder and a local download source, since the real model is a
    /// download (search FR-026).
    pub fn with_semantic(
        mut self,
        loader: EmbedderLoader,
        source: impl Into<String>,
        files: Vec<(String, u64, String)>,
    ) -> Self {
        self.memory.semantic = SemanticRuntime::new(loader, source.into(), files);
        self
    }

    /// The workspace folder, for "Open folder" (memory FR-025).
    pub fn workspace_folder(&self) -> std::path::PathBuf {
        self.workspace_dir()
    }

    /// memory FR-020: auto-recall on or off, from the next message.
    pub fn set_auto_recall(&self, enabled: bool) -> Result<crate::chat::ChatState> {
        let mut settings = self.settings()?;
        settings.recall.auto = enabled;
        self.save_settings(&settings)?;
        self.chat_state()
    }

    /// The chime when a reply is done, on or off.
    pub fn set_sound(&self, enabled: bool) -> Result<crate::chat::ChatState> {
        let mut settings = self.settings()?;
        settings.chat.sound = enabled;
        self.save_settings(&settings)?;
        self.chat_state()
    }

    /// `state/pec.db`. One from before the rename, `steve.db`, is moved once with its WAL files rather than
    /// rebuilt, because it also holds the undo history.
    pub(crate) fn index_path(&self) -> std::path::PathBuf {
        let state = self.state_dir();
        let path = state.join("pec.db");
        if !path.exists() && state.join("steve.db").exists() {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::rename(
                    state.join(format!("steve.db{suffix}")),
                    state.join(format!("pec.db{suffix}")),
                );
            }
        }
        path
    }

    /// Runs `f` on the index, opening it on first use.
    pub(crate) fn with_index<T>(
        &self,
        f: impl FnOnce(&mut Index) -> rusqlite::Result<T>,
    ) -> Result<T> {
        let mut slot = self.memory.index.lock().unwrap_or_else(|e| e.into_inner());
        if slot.is_none() {
            // memory FR-022: a damaged index is deleted and rebuilt from the files; only undo history is lost.
            let path = self.index_path();
            let index = Index::open(&path).or_else(|_| {
                for suffix in ["", "-wal", "-shm"] {
                    let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
                }
                Index::open(&path)
            });
            *slot = Some(index.map_err(internal)?);
        }
        f(slot.as_mut().expect("opened above")).map_err(internal)
    }
}

fn internal(e: rusqlite::Error) -> AppError {
    AppError::Internal {
        message: format!("memory index: {e}"),
    }
}
