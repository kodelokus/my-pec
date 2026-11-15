//! The memory browser's commands: list, read, save, search and undo (memory interfaces, memory R13, R15).

use super::files::{MEMORY_BUDGET, MemoryPath, PROFILE_BUDGET, daily_dates, fingerprint};
use super::search::without_comment;
use crate::{AppError, Core, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum FileKind {
    Soul,
    Profile,
    Memory,
    Daily,
    /// The dream diary.
    Diary,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemoryFile {
    pub path: String,
    pub kind: FileKind,
    pub chars: u32,
    pub budget: Option<u32>,
    /// False for a file that isn't valid UTF-8: it is shown but not indexed or opened.
    pub readable: bool,
}

/// A bullet or `- Key: value` line, for the reader and Forget.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct MemoryEntry {
    pub line: u32,
    pub text: String,
    pub id: Option<String>,
    pub at: Option<String>,
    pub from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemoryDoc {
    pub path: String,
    pub text: String,
    /// Sent back with a save, so a change made meanwhile is never saved over (memory FR-029).
    pub hash: String,
    pub entries: Vec<MemoryEntry>,
    pub can_undo: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum SaveOutcome {
    Saved {
        hash: String,
    },
    /// The file changed since it was read; nothing was written.
    Conflict {
        text: String,
        hash: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum UndoOutcome {
    Undone,
    /// The user changed the file since the companion's write; undo again with `force` to restore anyway.
    Edited,
    Nothing,
}

/// What to forget in a file (memory FR-027).
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum ForgetTarget {
    Line {
        line: u32,
    },
    /// A long-term memory entry by its id.
    Entry {
        id: String,
    },
    /// A whole daily note.
    File,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ForgetOutcome {
    /// The forgotten text is still in what the companion works with until a fresh context starts.
    pub in_current_context: bool,
}

/// A search result in the memory browser.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchHit {
    pub path: String,
    pub day: Option<String>,
    pub first_line: u32,
    pub last_line: u32,
    pub text: String,
}

/// The `key:value` fields of a line's trailing `<!-- … -->`.
pub(crate) fn comment_field(line: &str, key: &str) -> Option<String> {
    let start = line.find("<!--")?;
    line[start + 4..]
        .trim_end()
        .trim_end_matches("-->")
        .split_whitespace()
        .find_map(|word| word.strip_prefix(key)?.strip_prefix(':'))
        .map(Into::into)
}

/// The bullets of a memory file, without their bookkeeping comments.
pub(crate) fn entries(text: &str) -> Vec<MemoryEntry> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let item = without_comment(line).trim_start().strip_prefix("- ")?;
            Some(MemoryEntry {
                line: i as u32 + 1,
                text: item.trim().to_owned(),
                id: comment_field(line, "id"),
                at: comment_field(line, "at"),
                from: comment_field(line, "from"),
            })
        })
        .collect()
}

impl Core {
    /// Reads a memory file as text: a missing file is empty, a file that isn't UTF-8 is refused.
    pub(crate) fn read_memory(&self, path: &MemoryPath) -> Result<Option<String>> {
        match std::fs::read(self.workspace_dir().join(path.rel())) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| AppError::invalid("unreadable")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(AppError::Internal {
                message: e.to_string(),
            }),
        }
    }

    /// memory FR-025: the soul, profile and long-term memory, then the daily notes, newest first.
    pub fn memory_files(&self) -> Result<Vec<MemoryFile>> {
        let days = daily_dates(&self.workspace_dir());
        let files = [
            (MemoryPath::Soul, FileKind::Soul, Some(8_000)),
            (MemoryPath::Profile, FileKind::Profile, Some(PROFILE_BUDGET)),
            (MemoryPath::Memory, FileKind::Memory, Some(MEMORY_BUDGET)),
            (MemoryPath::Diary, FileKind::Diary, None),
        ]
        .into_iter()
        .chain(
            days.into_iter()
                .map(|d| (MemoryPath::Daily(d), FileKind::Daily, None)),
        );
        Ok(files
            .map(|(path, kind, budget)| {
                let text = self.read_memory(&path);
                MemoryFile {
                    path: path.rel(),
                    kind,
                    chars: text.as_ref().map_or(0, |t| {
                        t.as_deref().unwrap_or_default().chars().count() as u32
                    }),
                    budget: budget.map(|b| b as u32),
                    readable: text.is_ok(),
                }
            })
            .collect())
    }

    /// memory FR-025, FR-028: a file with its entries and whether a companion change can be undone.
    pub fn memory_read(&self, path: &str) -> Result<MemoryDoc> {
        let parsed = MemoryPath::parse(path)?;
        let text = match (self.read_memory(&parsed)?, &parsed) {
            (Some(text), _) => text,
            (None, MemoryPath::Daily(_)) => return Err(AppError::invalid("path")),
            (None, _) => String::new(),
        };
        let rel = parsed.rel();
        Ok(MemoryDoc {
            // The soul and the diary are read as text: nothing in them is an entry to forget.
            entries: if matches!(parsed, MemoryPath::Soul | MemoryPath::Diary) {
                Vec::new()
            } else {
                entries(&text)
            },
            hash: fingerprint(&text),
            can_undo: self.with_index(|i| i.latest_preimage(&rel))?.is_some(),
            path: rel,
            text,
        })
    }

    /// memory FR-025, FR-029: writes exactly the user's text, unless the file changed since they opened it. The user's
    /// own edit keeps no preimage; their editor keeps that history.
    pub fn memory_save(&self, path: &str, text: &str, base_hash: &str) -> Result<SaveOutcome> {
        let parsed = MemoryPath::parse(path)?;
        self.edit_memory(&parsed, |current| {
            if fingerprint(current) != base_hash {
                let conflict = SaveOutcome::Conflict {
                    text: current.to_owned(),
                    hash: fingerprint(current),
                };
                return Ok((current.to_owned(), conflict));
            }
            let saved = SaveOutcome::Saved {
                hash: fingerprint(text),
            };
            Ok((text.to_owned(), saved))
        })
    }

    /// memory FR-026: up to 20 hits over long-term memory and the daily notes, after the freshness check.
    pub fn memory_search(&self, query: &str) -> Result<Vec<SearchHit>> {
        if !(1..=500).contains(&query.trim().chars().count()) {
            return Err(AppError::invalid("query"));
        }
        let (hits, _) = self.search_memory(query, None, 20, true)?;
        Ok(hits
            .into_iter()
            .map(|h| SearchHit {
                path: h.path,
                day: h.day,
                first_line: h.first as u32,
                last_line: h.last as u32,
                text: h.text,
            })
            .collect())
    }

    /// memory FR-027, Principle IV: removes the target from its file, the index and every preimage holding it, and
    /// records the messages it came from so later dreaming skips them (memory R14). The webview confirms first.
    pub fn memory_forget(
        &self,
        path: &str,
        target: ForgetTarget,
        base_hash: &str,
    ) -> Result<ForgetOutcome> {
        let parsed = MemoryPath::parse(path)?;
        let allowed = match (&parsed, &target) {
            (MemoryPath::Soul | MemoryPath::Diary, _) => false,
            (MemoryPath::Daily(_), ForgetTarget::File) | (_, ForgetTarget::Line { .. }) => true,
            (MemoryPath::Memory, ForgetTarget::Entry { .. }) => true,
            _ => false,
        };
        if !allowed {
            return Err(AppError::invalid("target"));
        }
        let rel = parsed.rel();
        let removed: Vec<String> = if target == ForgetTarget::File {
            let _guard = self.memory.write.lock().unwrap_or_else(|e| e.into_inner());
            let current = self
                .read_memory(&parsed)?
                .ok_or_else(|| AppError::invalid("path"))?;
            if fingerprint(&current) != base_hash {
                return Err(AppError::invalid("changed"));
            }
            let file = self.workspace_dir().join(&rel);
            std::fs::remove_file(&file).map_err(|_| AppError::not_writable(&file))?;
            current.lines().map(str::to_owned).collect()
        } else {
            self.edit_memory(&parsed, |current| {
                if fingerprint(current) != base_hash {
                    return Err(AppError::invalid("changed"));
                }
                let lines: Vec<&str> = current.lines().collect();
                let at = match &target {
                    ForgetTarget::Line { line } => {
                        (*line as usize).checked_sub(1).filter(|i| *i < lines.len())
                    }
                    ForgetTarget::Entry { id } => lines
                        .iter()
                        .position(|l| comment_field(l, "id").as_deref() == Some(id)),
                    ForgetTarget::File => None,
                }
                .ok_or_else(|| AppError::invalid("target"))?;
                let mut kept = lines.clone();
                let gone = kept.remove(at).to_owned();
                let mut next = kept.join("\n");
                if current.ends_with('\n') && !next.is_empty() {
                    next.push('\n');
                }
                Ok((next, vec![gone]))
            })?
        };
        let texts: Vec<String> = removed
            .iter()
            .map(|l| {
                let item = without_comment(l).trim();
                item.strip_prefix("- ").unwrap_or(item).trim().to_owned()
            })
            .filter(|t| !t.is_empty())
            .collect();
        let workspace = self.workspace_dir();
        self.with_index(|i| {
            i.index_file(&workspace, &rel)?;
            i.forget_preimages(&rel, &texts)
        })?;
        let sources: Vec<String> = removed
            .iter()
            .filter_map(|l| comment_field(l, "src"))
            .collect();
        // journey FR-042: the stories of the days the text came from go too.
        let mut days: Vec<String> = match &parsed {
            MemoryPath::Daily(date) => vec![date.clone()],
            _ => Vec::new(),
        };
        let parts: Vec<&str> = sources.iter().flat_map(|s| s.split(',')).collect();
        days.extend(
            parts
                .iter()
                .filter_map(|p| p.strip_prefix("memory/")?.get(..10).map(Into::into)),
        );
        days.extend(self.message_days(&parts)?);
        days.sort();
        days.dedup();
        let stories = self.forget_days(&days)?;
        let in_current_context =
            self.record_forgotten(sources, &texts)? || self.seen_in_context(&stories)?;
        Ok(ForgetOutcome { in_current_context })
    }

    /// memory FR-009, FR-028: puts back the file as it was before the companion's most recent change, byte for byte.
    pub fn memory_undo(&self, path: &str, force: bool) -> Result<UndoOutcome> {
        let parsed = MemoryPath::parse(path)?;
        if !matches!(parsed, MemoryPath::Profile | MemoryPath::Memory) {
            return Err(AppError::invalid("path"));
        }
        let rel = parsed.rel();
        self.edit_memory(&parsed, |current| {
            let Some((id, before, after)) = self.with_index(|i| i.latest_preimage(&rel))? else {
                return Ok((current.to_owned(), UndoOutcome::Nothing));
            };
            if fingerprint(current) != after && !force {
                return Ok((current.to_owned(), UndoOutcome::Edited));
            }
            self.with_index(|i| i.delete_preimage(id))?;
            Ok((before, UndoOutcome::Undone))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_bullets_without_their_bookkeeping() {
        let text = "# Memory\n\n## Health\n- Allergic to peanuts. <!-- id:m_8f3a at:2026-11-14 from:user src:1-0 -->\n\
                    Free text.\n- Hand-written.\n";
        assert_eq!(
            entries(text),
            [
                MemoryEntry {
                    line: 4,
                    text: "Allergic to peanuts.".into(),
                    id: Some("m_8f3a".into()),
                    at: Some("2026-11-14".into()),
                    from: Some("user".into()),
                },
                MemoryEntry {
                    line: 6,
                    text: "Hand-written.".into(),
                    id: None,
                    at: None,
                    from: None,
                },
            ]
        );
    }
}
