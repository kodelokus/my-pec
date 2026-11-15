//! The deep phase's commit: each changed file written once, after its preimage, under a commit marker that rolls a
//! half-done commit back; and "undo last dream" (dreaming R8, R9; dreaming FR-017, FR-018).

use super::UndoDreamOutcome;
use super::run::{Phase, Run};
use crate::chat::lock;
use crate::memory::files::fingerprint;
use crate::{AppError, Core, Result};
use std::collections::BTreeMap;

fn message(run: &Run) -> String {
    format!("dream:{}", run.id)
}

impl Core {
    /// Writes each file whose dream text differs from `prior`. A file changed since the decisions were made is
    /// rebuilt from its current text by `rebuild` (validated there), or left as it is tonight (Principle I).
    pub(crate) fn commit_dream(
        &self,
        run: &mut Run,
        prior: &BTreeMap<&str, String>,
        next: &BTreeMap<&str, String>,
        rebuild: &dyn Fn(&str, &str) -> Option<String>,
    ) -> Result<()> {
        run.phase = Phase::Committing;
        self.save_run(run)?;
        let tag = message(run);
        let mut changed = false;
        for (rel, target) in next {
            let before = &prior[rel];
            if target == before {
                continue;
            }
            changed |= self.edit_workspace(rel, |current| {
                let target = if current == before {
                    target.clone()
                } else {
                    match rebuild(rel, current) {
                        Some(text) => text,
                        None => return Ok((current.to_owned(), false)),
                    }
                };
                if target == current {
                    return Ok((current.to_owned(), false));
                }
                self.save_preimage(rel, current, &target, &tag)?;
                Ok((target, true))
            })?;
        }
        run.committed = changed;
        run.phase = Phase::Diary;
        self.save_run(run)
    }

    /// A run found mid-commit: each file the commit wrote goes back to its preimage, and the deep phase runs again.
    pub(crate) fn roll_back(&self, run: &mut Run) -> Result<()> {
        for (id, path, before, after_hash) in self.preimages_of(&message(run))? {
            self.edit_workspace(&path, |current| {
                let restored = if fingerprint(current) == after_hash {
                    before.clone()
                } else {
                    current.to_owned()
                };
                Ok((restored, ()))
            })?;
            self.drop_preimage(id)?;
        }
        run.committed = false;
        run.phase = Phase::Deep;
        self.save_run(run)
    }

    /// dreaming FR-018: the files of the latest dream not yet undone go back to how they were before it, byte for byte.
    /// A file changed since is reported, and restored only with `force`.
    pub fn undo_last_dream(&self, force: bool) -> Result<UndoDreamOutcome> {
        if lock(&self.chat.dreaming).is_some() {
            return Err(AppError::invalid("busy"));
        }
        let Some(mut run) = self
            .runs(1_000)?
            .into_iter()
            .find(|r| r.committed && !r.undone)
        else {
            return Ok(UndoDreamOutcome::NothingToUndo);
        };
        let preimages = self.preimages_of(&message(&run))?;
        let workspace = self.workspace_dir();
        let files: Vec<String> = preimages
            .iter()
            .filter(|(_, path, _, after)| {
                let now = std::fs::read_to_string(workspace.join(path)).unwrap_or_default();
                fingerprint(&now) != *after
            })
            .map(|(_, path, _, _)| path.clone())
            .collect();
        if !files.is_empty() && !force {
            return Ok(UndoDreamOutcome::Changed { files });
        }
        for (id, path, before, _) in preimages {
            self.edit_workspace(&path, |_| Ok((before.clone(), ())))?;
            self.drop_preimage(id)?;
        }
        run.undone = true;
        self.save_run(&run)?;
        Ok(UndoDreamOutcome::Undone)
    }
}
