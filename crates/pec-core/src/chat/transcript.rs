//! `workspace/sessions/main.jsonl`: one entry per line, only ever appended to (chat data 1).

use crate::memory::MemoryMark;
use crate::{AppError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Provenance {
    User,
    Companion,
    /// Tool results (architecture 6.3): never promoted to long-term memory automatically.
    Untrusted,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ReplyState {
    Complete,
    Stopped,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum Entry {
    Context {
        model: String,
        files: BTreeMap<String, String>,
        harness: String,
        /// Today's and yesterday's daily notes when the context started, already cut to budget (memory R8).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        notes: Option<BTreeMap<String, String>>,
        /// The compaction that opened this context (compaction R9).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        compaction: Option<String>,
        /// The diary entry of the dream this context was opened after (dreaming R10).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dream: Option<String>,
        /// The world description this context was given (journey R9).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        world: Option<WorldSnapshot>,
        /// The extensions that were on when the context started; missing means none (extensions R2).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        extensions: Vec<String>,
        /// What the companion is told about extensions, after the harness: the SKILL.md of each one that was on
        /// and a line for each one that was off. Kept here so an app update never changes it (Principle III).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skills: Option<String>,
        /// The first message of the turns carried over, as text, from the context before: set when the person
        /// caught up and there was nothing to compact.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        carried: Option<String>,
    },
    Message {
        role: Role,
        provenance: Provenance,
        content: Vec<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        state: Option<ReplyState>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stop_reason: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<Value>,
        /// User messages: the index of the block holding auto-recalled memory (memory FR-021).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        recalled: Option<usize>,
        /// User messages: the newest journey day the world notice it carries told about (journey R10).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        world: Option<String>,
        /// Tool-result messages: what the tools wrote, shown on the turn's reply (memory FR-008).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        marks: Vec<MemoryMark>,
    },
    Notice {
        file: String,
        content: String,
        message: String,
    },
    Replaced {
        target: String,
    },
    /// The companion changed a loaded file; counts as seen, so no notice follows (memory R7).
    Written {
        file: String,
        content: String,
        /// The tool-result message reporting the write; absent for the identity lines world core adds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// The user forgot memories that came from these messages; holds no text (memory R14).
    Forgotten {
        messages: Vec<String>,
    },
    /// Older turns were summarized; counts only when a `context` entry naming it follows (compaction R5, R9).
    #[serde(rename_all = "camelCase")]
    Compaction {
        summary: String,
        first_kept: String,
        tokens_before: u64,
        kept_tokens: u64,
        model: String,
        usage: CompactionUsage,
        flush: FlushRecord,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        focus: Option<String>,
    },
}

/// The provider usage of a compaction's two calls, as received.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct CompactionUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flush: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FlushOutcome {
    Saved,
    #[serde(rename = "none")]
    Nothing,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct FlushRecord {
    pub outcome: FlushOutcome,
    #[serde(default)]
    pub notes: Vec<FlushNote>,
}

/// A note the flush saved to the day's note: its local time and text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct FlushNote {
    pub time: String,
    pub text: String,
}

/// What a context was told about the companion's world, and the newest journey day it covers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorldSnapshot {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub through: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Line {
    pub id: String,
    pub at: u64,
    #[serde(flatten)]
    pub entry: Entry,
}

/// The conversation as read at launch, plus what was appended since.
pub(crate) struct Transcript {
    path: PathBuf,
    pub lines: Vec<Line>,
    /// Lines that could not be read (chat FR-018).
    pub skipped: usize,
    /// The file ends in a half-written line, so the next append starts on a new line.
    torn: bool,
    last: (u64, u32),
}

impl Transcript {
    pub fn open(path: PathBuf) -> Result<Self> {
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => {
                return Err(AppError::Internal {
                    message: format!("{}: {e}", path.display()),
                });
            }
        };
        let (mut lines, mut skipped) = (Vec::new(), 0);
        for raw in String::from_utf8_lossy(&bytes).lines() {
            if raw.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Line>(raw) {
                Ok(line) => lines.push(line),
                Err(_) => skipped += 1,
            }
        }
        // Ids stay unique even if the clock has gone back since the last entry.
        let last = lines
            .last()
            .and_then(|l: &Line| {
                let (ms, n) = l.id.split_once('-')?;
                Some((ms.parse().ok()?, n.parse().ok()?))
            })
            .unwrap_or((0, 0));
        Ok(Self {
            path,
            lines,
            skipped,
            torn: !bytes.is_empty() && !bytes.ends_with(b"\n"),
            last,
        })
    }

    /// A sortable id: Unix milliseconds plus a counter for entries in the same millisecond.
    pub fn next_id(&mut self, at: u64) -> String {
        self.last = if at <= self.last.0 {
            (self.last.0, self.last.1 + 1)
        } else {
            (at, 0)
        };
        format!("{}-{}", self.last.0, self.last.1)
    }

    /// Appends one entry and syncs it to disk before returning (chat FR-013, FR-015).
    pub fn append(&mut self, line: Line) -> Result<()> {
        self.append_all(vec![line])
    }

    /// Appends entries in one write and one sync, so they land together or not at all (compaction R5).
    pub fn append_all(&mut self, lines: Vec<Line>) -> Result<()> {
        let mut bytes = Vec::new();
        if self.torn {
            bytes.push(b'\n');
        }
        for line in &lines {
            bytes.extend(serde_json::to_vec(line).expect("an entry serializes"));
            bytes.push(b'\n');
        }
        write(&self.path, &bytes).map_err(|_| AppError::not_writable(&self.path))?;
        self.torn = false;
        self.lines.extend(lines);
        Ok(())
    }
}

fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file: File = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Transcript {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("main.jsonl");
        std::fs::write(&path, text).unwrap();
        Transcript::open(path).unwrap()
    }

    #[test]
    fn a_feature_002_file_reads_as_before() {
        let t = read(concat!(
            r#"{"id":"1-0","at":1,"type":"context","model":"m","files":{"SOUL.md":"s"},"harness":"chat-1"}"#,
            "\n",
            r#"{"id":"2-0","at":2,"type":"message","role":"user","provenance":"user","content":[],"text":"Hi"}"#,
            "\n",
            r#"{"id":"3-0","at":3,"type":"message","role":"assistant","provenance":"companion","content":[],"state":"complete"}"#,
            "\n",
            r#"{"id":"4-0","at":4,"type":"replaced","target":"3-0"}"#,
            "\n",
        ));
        assert_eq!(t.skipped, 0);
        assert!(matches!(
            &t.lines[0].entry,
            Entry::Context { notes: None, .. }
        ));
        assert!(matches!(
            &t.lines[1].entry,
            Entry::Message { recalled: None, .. }
        ));
        assert!(matches!(&t.lines[3].entry, Entry::Replaced { target } if target == "3-0"));
    }

    #[test]
    fn the_memory_entries_round_trip_unchanged() {
        let lines = [
            r##"{"id":"1-0","at":1,"type":"context","model":"m","files":{},"harness":"memory-1","notes":{"2026-11-14":"# 2026-11-14\n"}}"##,
            r#"{"id":"2-0","at":2,"type":"message","role":"user","provenance":"untrusted","content":[]}"#,
            r#"{"id":"3-0","at":3,"type":"message","role":"user","provenance":"user","content":[],"text":"Hi","recalled":1}"#,
            r##"{"id":"4-0","at":4,"type":"written","file":"MEMORY.md","content":"# Memory\n","message":"2-0"}"##,
            r##"{"id":"4-1","at":4,"type":"written","file":"IDENTITY.md","content":"# Identity\n"}"##,
            r#"{"id":"5-0","at":5,"type":"forgotten","messages":["3-0"]}"#,
        ];
        let t = read(&(lines.join("\n") + "\n"));
        assert_eq!(t.skipped, 0);
        for (line, raw) in t.lines.iter().zip(lines) {
            let written: Value = serde_json::to_value(line).unwrap();
            assert_eq!(written, serde_json::from_str::<Value>(raw).unwrap());
        }
    }

    #[test]
    fn the_compaction_entries_round_trip_unchanged() {
        let lines = [
            r###"{"id":"9-0","at":9,"type":"compaction","summary":"## Where we are\n- Lisbon","firstKept":"7-0","tokensBefore":43120,"keptTokens":7480,"model":"m","usage":{"flush":{"input_tokens":3},"summary":{"output_tokens":5}},"flush":{"outcome":"saved","notes":[{"time":"21:40","text":"Rina visits."}]},"focus":"the formula"}"###,
            r#"{"id":"9-1","at":9,"type":"context","model":"m","files":{},"harness":"memory-1","compaction":"9-0"}"#,
            r#"{"id":"10-0","at":10,"type":"compaction","summary":"s","firstKept":"9-2","tokensBefore":1,"keptTokens":0,"model":"m","usage":{},"flush":{"outcome":"none","notes":[]}}"#,
        ];
        let t = read(&(lines.join("\n") + "\n"));
        assert_eq!(t.skipped, 0);
        for (line, raw) in t.lines.iter().zip(lines) {
            let written: Value = serde_json::to_value(line).unwrap();
            assert_eq!(written, serde_json::from_str::<Value>(raw).unwrap());
        }
    }

    #[test]
    fn entries_appended_together_start_on_a_new_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("main.jsonl");
        std::fs::write(
            &path,
            r#"{"id":"1-0","at":1,"type":"replaced","target":"0-0"}{"half"#,
        )
        .unwrap();
        let mut t = Transcript::open(path.clone()).unwrap();
        let line = |id: &str| Line {
            id: id.into(),
            at: 2,
            entry: Entry::Forgotten {
                messages: Vec::new(),
            },
        };
        t.append_all(vec![line("2-0"), line("2-1")]).unwrap();
        let again = Transcript::open(path).unwrap();
        assert_eq!((again.lines.len(), again.skipped), (2, 1));
        assert_eq!(again.lines[1].id, "2-1");
    }
}
