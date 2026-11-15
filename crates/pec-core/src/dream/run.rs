//! A sweep's row in `dream_runs`: the night, the phase reached and the checkpoint each phase writes, so a paused or
//! interrupted sweep resumes where it stopped (dreaming R4, dreaming data 2 and 5).

use crate::world::journey::NamedDay;
use crate::{Core, Result};
use rusqlite::{OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Phase {
    Light,
    Rem,
    Deep,
    Committing,
    Diary,
    /// Journey: naming the ended days' topics.
    World,
    /// Journey: appending the planned world and journey lines.
    Drawing,
    Roll,
    Done,
    Failed,
}

impl Phase {
    fn name(self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(Into::into))
            .unwrap_or_default()
    }

    fn parse(text: &str) -> Self {
        serde_json::from_value(Value::String(text.into())).unwrap_or(Self::Failed)
    }
}

/// Where a candidate's text was written: a daily note line, and the fingerprint of that line's text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Source {
    pub path: String,
    pub line: usize,
    pub text_hash: String,
}

/// A line the light phase gathered (dreaming R5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Candidate {
    pub id: usize,
    pub text: String,
    /// `user` or `companion`; nothing else is gathered.
    pub provenance: String,
    pub sources: Vec<Source>,
    /// The days the line was written on.
    pub days: Vec<String>,
    pub reinforced: u32,
    pub terms: usize,
}

/// Something the person typed since the last sweep, with its local time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Said {
    pub at: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Theme {
    pub title: String,
    pub summary: String,
    pub candidates: Vec<usize>,
}

/// One consolidation decision (dreaming interfaces).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Op {
    pub op: String,
    pub topic: String,
    pub text: String,
    pub key: String,
    pub sources: Vec<usize>,
    pub replaces: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Checkpoint {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    #[serde(default)]
    pub messages: Vec<Said>,
    #[serde(default)]
    pub themes: Vec<Theme>,
    #[serde(default)]
    pub decisions: Vec<Op>,
    /// The candidates that passed the gates, for the fallback.
    #[serde(default)]
    pub gated: Vec<usize>,
    /// The entries the dream wrote, for the diary.
    #[serde(default)]
    pub kept: Vec<String>,
    #[serde(default)]
    pub diary: Option<String>,
    /// Journey: the validated topics of every day asked about.
    #[serde(default)]
    pub topics: Vec<NamedDay>,
    /// Journey: the exact lines to append (dreaming R8).
    #[serde(default)]
    pub drawing: Option<Drawing>,
}

/// The world and journey lines a night's topics give, appended by the drawing phase.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Drawing {
    pub world: Vec<String>,
    pub journey: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Run {
    pub id: i64,
    pub night: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    /// Unix seconds: the person's messages from then on are this sweep's.
    pub since: i64,
    pub phase: Phase,
    pub checkpoint: Checkpoint,
    pub added: u32,
    pub merged: u32,
    pub superseded: u32,
    pub fell_back: bool,
    pub usage: BTreeMap<String, Value>,
    pub committed: bool,
    pub undone: bool,
}

const COLUMNS: &str = "id, night, started_at, ended_at, since, phase, checkpoint, added, merged, superseded, \
                       fell_back, usage, committed, undone";

/// A JSON column; an unreadable one reads as empty rather than losing the run.
fn json<T: serde::de::DeserializeOwned + Default>(text: String) -> T {
    serde_json::from_str(&text).unwrap_or_default()
}

fn read(row: &Row) -> rusqlite::Result<Run> {
    Ok(Run {
        id: row.get(0)?,
        night: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        since: row.get(4)?,
        phase: Phase::parse(&row.get::<_, String>(5)?),
        checkpoint: json(row.get(6)?),
        added: row.get(7)?,
        merged: row.get(8)?,
        superseded: row.get(9)?,
        fell_back: row.get(10)?,
        usage: json(row.get(11)?),
        committed: row.get(12)?,
        undone: row.get(13)?,
    })
}

impl Core {
    pub(crate) fn start_run(&self, night: &str, since: i64) -> Result<Run> {
        let started = crate::config::now() as i64;
        let id = self.with_index(|i| {
            i.db.execute(
                "INSERT INTO dream_runs (night, started_at, since, phase, checkpoint) VALUES (?1, ?2, ?3, 'light', '{}')",
                params![night, started, since],
            )?;
            Ok(i.db.last_insert_rowid())
        })?;
        Ok(Run {
            id,
            night: night.into(),
            started_at: started,
            ended_at: None,
            since,
            phase: Phase::Light,
            checkpoint: Checkpoint::default(),
            added: 0,
            merged: 0,
            superseded: 0,
            fell_back: false,
            usage: BTreeMap::new(),
            committed: false,
            undone: false,
        })
    }

    /// Writes the whole run back: its phase and checkpoint move together.
    pub(crate) fn save_run(&self, run: &Run) -> Result<()> {
        let checkpoint = serde_json::to_string(&run.checkpoint).expect("a checkpoint serializes");
        let usage = serde_json::to_string(&run.usage).expect("usage serializes");
        self.with_index(|i| {
            i.db.execute(
                "UPDATE dream_runs SET ended_at = ?2, phase = ?3, checkpoint = ?4, added = ?5, merged = ?6, \
                 superseded = ?7, fell_back = ?8, usage = ?9, committed = ?10, undone = ?11 WHERE id = ?1",
                params![
                    run.id,
                    run.ended_at,
                    run.phase.name(),
                    checkpoint,
                    run.added,
                    run.merged,
                    run.superseded,
                    run.fell_back,
                    usage,
                    run.committed,
                    run.undone
                ],
            )
            .map(drop)
        })
    }

    /// The newest runs first.
    pub(crate) fn runs(&self, limit: usize) -> Result<Vec<Run>> {
        self.with_index(|i| {
            i.db.prepare(&format!(
                "SELECT {COLUMNS} FROM dream_runs ORDER BY id DESC LIMIT ?1"
            ))?
            .query_map(params![limit as i64], read)?
            .collect()
        })
    }

    /// The latest run that is neither done nor failed: one to resume.
    pub(crate) fn unfinished_run(&self) -> Result<Option<Run>> {
        self.with_index(|i| {
            i.db.query_row(
                &format!(
                    "SELECT {COLUMNS} FROM dream_runs WHERE phase NOT IN ('done', 'failed') ORDER BY id DESC LIMIT 1"
                ),
                [],
                read,
            )
            .optional()
        })
    }

    /// The latest run that completed: what the next sweep starts after.
    pub(crate) fn last_completed_run(&self) -> Result<Option<Run>> {
        self.with_index(|i| {
            i.db.query_row(
                &format!(
                    "SELECT {COLUMNS} FROM dream_runs WHERE phase = 'done' ORDER BY id DESC LIMIT 1"
                ),
                [],
                read,
            )
            .optional()
        })
    }
}
