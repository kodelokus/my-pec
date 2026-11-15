//! `state/pec.db`: the search index derived from the memory files, and the preimages that power undo
//! (memory data 4, memory R9 to R13). Everything but `preimages`, `recall_events` and `dream_runs`
//! (dreaming) can be dropped and rebuilt.

use super::files::{daily_dates, fingerprint};
use super::search::{
    Hit, blend, chunks, fts_query, meaning_score, prepare, rank, terms, without_comment,
};
use crate::{Core, Result};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashMap;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// One search re-indexes at most this many bytes of changed files, so a huge outside edit never holds up a
/// message; the following searches catch up (memory R12).
pub(crate) const REFRESH_BYTES: i64 = 512 * 1024;
/// Once the index is this big, terms in more than half the chunks are too common to search by (memory R11).
const COMMON_AFTER: i64 = 50;

/// Bumped whenever the derived tables change shape; a mismatch rebuilds them and keeps `preimages`.
const SCHEMA_VERSION: &str = "2";
/// search R4: the nearest chunks by meaning a search considers.
const MEANING_CANDIDATES: usize = 50;

const DERIVED: &str = "
    CREATE TABLE files (
        path TEXT PRIMARY KEY,
        size INTEGER NOT NULL,
        modified INTEGER NOT NULL,
        readable INTEGER NOT NULL
    );
    CREATE TABLE chunks (
        id INTEGER PRIMARY KEY,
        path TEXT NOT NULL,
        first_line INTEGER NOT NULL,
        last_line INTEGER NOT NULL,
        text TEXT NOT NULL,
        day TEXT,
        text_hash TEXT NOT NULL
    );
    CREATE INDEX chunks_path ON chunks (path);
    CREATE INDEX chunks_text_hash ON chunks (text_hash);
    CREATE TABLE chunk_vectors (
        text_hash TEXT NOT NULL,
        model TEXT NOT NULL,
        vector BLOB NOT NULL,
        PRIMARY KEY (text_hash, model)
    );
    CREATE VIRTUAL TABLE chunks_fts USING fts5 (body, tokenize = 'unicode61 remove_diacritics 2');
    CREATE VIRTUAL TABLE chunks_vocab USING fts5vocab (chunks_fts, 'row');
";

const DROP_DERIVED: &str = "
    DROP TABLE IF EXISTS files;
    DROP TABLE IF EXISTS chunks;
    DROP TABLE IF EXISTS chunk_vectors;
    DROP TABLE IF EXISTS chunks_vocab;
    DROP TABLE IF EXISTS chunks_fts;
";

pub(crate) struct Index {
    pub(crate) db: Connection,
}

impl Index {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let db = Connection::open(path)?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        let check: String = db.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        if check != "ok" {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
                Some(check),
            ));
        }
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS preimages (
                 id INTEGER PRIMARY KEY,
                 path TEXT NOT NULL,
                 before TEXT NOT NULL,
                 after_hash TEXT NOT NULL,
                 message TEXT NOT NULL,
                 at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS recall_events (
                 id INTEGER PRIMARY KEY,
                 path TEXT NOT NULL,
                 line INTEGER NOT NULL,
                 text_hash TEXT NOT NULL,
                 query_hash TEXT NOT NULL,
                 coverage REAL NOT NULL,
                 day TEXT NOT NULL,
                 source TEXT NOT NULL,
                 at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS recall_events_key ON recall_events (path, text_hash);
             CREATE TABLE IF NOT EXISTS dream_runs (
                 id INTEGER PRIMARY KEY,
                 night TEXT NOT NULL,
                 started_at INTEGER NOT NULL,
                 ended_at INTEGER,
                 since INTEGER NOT NULL,
                 phase TEXT NOT NULL,
                 checkpoint TEXT NOT NULL,
                 added INTEGER NOT NULL DEFAULT 0,
                 merged INTEGER NOT NULL DEFAULT 0,
                 superseded INTEGER NOT NULL DEFAULT 0,
                 fell_back INTEGER NOT NULL DEFAULT 0,
                 usage TEXT NOT NULL DEFAULT '{}',
                 committed INTEGER NOT NULL DEFAULT 0,
                 undone INTEGER NOT NULL DEFAULT 0
             );",
        )?;
        let version: Option<String> = db
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        if version.as_deref() != Some(SCHEMA_VERSION) {
            db.execute_batch(DROP_DERIVED)?;
            db.execute_batch(DERIVED)?;
            db.execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
                params![SCHEMA_VERSION],
            )?;
        }
        Ok(Self { db })
    }
}

/// An indexed file as found on disk: workspace-relative path, size and modified time in milliseconds.
type Found = (String, i64, i64);

/// `MEMORY.md` and the validly named daily notes, long-term memory first, then the newest notes.
fn memory_files(workspace: &Path) -> Vec<Found> {
    let stat = |rel: String| -> Option<Found> {
        let meta = std::fs::metadata(workspace.join(&rel)).ok()?;
        let modified = meta
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_millis() as i64;
        meta.is_file().then_some((rel, meta.len() as i64, modified))
    };
    std::iter::once("MEMORY.md".to_owned())
        .chain(
            daily_dates(workspace)
                .iter()
                .map(|d| format!("memory/{d}.md")),
        )
        .filter_map(stat)
        .collect()
}

impl Index {
    /// Brings the index up to date with the files: drops removed ones and re-indexes changed ones, within
    /// `budget` bytes (memory R12).
    pub fn refresh(&mut self, workspace: &Path, budget: i64) -> rusqlite::Result<()> {
        let found = memory_files(workspace);
        let known: HashMap<String, (i64, i64)> = self
            .db
            .prepare("SELECT path, size, modified FROM files")?
            .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?
            .collect::<rusqlite::Result<_>>()?;
        for path in known.keys() {
            if !found.iter().any(|(rel, ..)| rel == path) {
                self.reindex(path, None)?;
            }
        }
        let mut left = budget;
        for (rel, size, modified) in found {
            if known.get(&rel) == Some(&(size, modified)) {
                continue;
            }
            if left < budget && size > left {
                break;
            }
            left -= size;
            let bytes = std::fs::read(workspace.join(&rel)).unwrap_or_default();
            self.reindex(&rel, Some((size, modified, String::from_utf8(bytes).ok())))?;
        }
        Ok(())
    }

    /// Replaces a file's chunks in one transaction. `None` drops the file; unreadable text is recorded and skipped.
    fn reindex(
        &mut self,
        path: &str,
        file: Option<(i64, i64, Option<String>)>,
    ) -> rusqlite::Result<()> {
        let tx = self.db.transaction()?;
        tx.execute(
            "DELETE FROM chunks_fts WHERE rowid IN (SELECT id FROM chunks WHERE path = ?1)",
            params![path],
        )?;
        tx.execute("DELETE FROM chunks WHERE path = ?1", params![path])?;
        tx.execute("DELETE FROM files WHERE path = ?1", params![path])?;
        if let Some((size, modified, text)) = file {
            let day = path
                .strip_prefix("memory/")
                .and_then(|n| n.strip_suffix(".md"));
            for chunk in chunks(text.as_deref().unwrap_or_default(), day.is_none()) {
                tx.execute(
                    "INSERT INTO chunks (path, first_line, last_line, text, day, text_hash) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        path,
                        chunk.first as i64,
                        chunk.last as i64,
                        chunk.text,
                        day,
                        fingerprint(&chunk.text)
                    ],
                )?;
                tx.execute(
                    "INSERT INTO chunks_fts (rowid, body) VALUES (last_insert_rowid(), ?1)",
                    params![prepare(&chunk.text)],
                )?;
            }
            tx.execute(
                "INSERT INTO files (path, size, modified, readable) VALUES (?1, ?2, ?3, ?4)",
                params![path, size, modified, text.is_some()],
            )?;
        }
        // search FR-017: a vector goes with the last chunk that held its text.
        tx.execute(
            "DELETE FROM chunk_vectors WHERE text_hash NOT IN (SELECT text_hash FROM chunks)",
            [],
        )?;
        tx.commit()
    }

    /// Re-indexes one file now, or drops it if it no longer exists.
    pub fn index_file(&mut self, workspace: &Path, rel: &str) -> rusqlite::Result<()> {
        match memory_files(workspace).into_iter().find(|(r, ..)| r == rel) {
            Some((_, size, modified)) => {
                let bytes = std::fs::read(workspace.join(rel)).unwrap_or_default();
                self.reindex(rel, Some((size, modified, String::from_utf8(bytes).ok())))
            }
            None => self.reindex(rel, None),
        }
    }

    /// Deletes a file's preimages that hold any of `texts`, so undo can't bring back what was forgotten.
    pub fn forget_preimages(&self, path: &str, texts: &[String]) -> rusqlite::Result<()> {
        for text in texts {
            self.db.execute(
                "DELETE FROM preimages WHERE path = ?1 AND instr(before, ?2) > 0",
                params![path, text],
            )?;
        }
        Ok(())
    }

    /// The most recent preimage of a file: its id, the content before, and the hash of the content after.
    pub fn latest_preimage(&self, path: &str) -> rusqlite::Result<Option<(i64, String, String)>> {
        self.db
            .query_row(
                "SELECT id, before, after_hash FROM preimages WHERE path = ?1 ORDER BY id DESC LIMIT 1",
                params![path],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
    }

    pub fn delete_preimage(&self, id: i64) -> rusqlite::Result<()> {
        self.db
            .execute("DELETE FROM preimages WHERE id = ?1", params![id])
            .map(drop)
    }

    /// Hits for `query`, best first (memory R11). With `meaning` (each chunk text's cosine to the query, by text
    /// hash), keyword and meaning scores are blended; without it the order is the keyword order (search R4).
    pub fn search(
        &self,
        query: &str,
        today: Option<&str>,
        limit: usize,
        meaning: Option<&HashMap<String, f32>>,
    ) -> rusqlite::Result<(Vec<Hit>, usize)> {
        let mut terms = terms(query);
        let total: i64 = self
            .db
            .query_row("SELECT count(*) FROM chunks", [], |r| r.get(0))?;
        if total >= COMMON_AFTER {
            let mut frequency = self
                .db
                .prepare("SELECT doc FROM chunks_vocab WHERE term = ?1")?;
            let rare: Vec<String> = terms
                .iter()
                .filter(|t| {
                    let docs: i64 = frequency.query_row(params![t], |r| r.get(0)).unwrap_or(0);
                    docs * 2 <= total
                })
                .cloned()
                .collect();
            if !rare.is_empty() {
                terms = rare;
            }
        }
        // A candidate chunk: its id, the hit with its keyword score, and its text hash.
        let row = |r: &rusqlite::Row| -> rusqlite::Result<(i64, Hit, String)> {
            let text: String = r.get(5)?;
            let prepared = prepare(&text).to_lowercase();
            let hit = Hit {
                path: r.get(1)?,
                day: r.get(2)?,
                first: r.get::<_, i64>(3)? as usize,
                last: r.get::<_, i64>(4)? as usize,
                matched: terms
                    .iter()
                    .filter(|t| prepared.contains(t.as_str()))
                    .count(),
                meaning: 0.0,
                // bm25 is lower for better matches; a chunk found only by meaning has none.
                score: -r.get::<_, Option<f64>>(7)?.unwrap_or(0.0),
                text,
            };
            Ok((r.get(0)?, hit, r.get(6)?))
        };
        let mut found: Vec<(i64, Hit, String)> = Vec::new();
        if !terms.is_empty() {
            let mut statement = self.db.prepare(
                "SELECT c.id, c.path, c.day, c.first_line, c.last_line, c.text, c.text_hash, bm25(chunks_fts)
                 FROM chunks_fts JOIN chunks c ON c.id = chunks_fts.rowid
                 WHERE chunks_fts MATCH ?1 ORDER BY bm25(chunks_fts) LIMIT 200",
            )?;
            found = statement
                .query_map(params![fts_query(&terms)], row)?
                .collect::<rusqlite::Result<_>>()?;
        }
        if let Some(meaning) = meaning {
            let mut nearest: Vec<(&String, f32)> = meaning
                .iter()
                .filter(|(_, c)| meaning_score(f64::from(**c)) > 0.0)
                .map(|(h, c)| (h, *c))
                .collect();
            nearest.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
            let mut statement = self.db.prepare(
                "SELECT id, path, day, first_line, last_line, text, text_hash, NULL
                 FROM chunks WHERE text_hash = ?1",
            )?;
            for (hash, _) in nearest.into_iter().take(MEANING_CANDIDATES) {
                for chunk in statement.query_map(params![hash], row)? {
                    let chunk = chunk?;
                    if !found.iter().any(|(id, ..)| *id == chunk.0) {
                        found.push(chunk);
                    }
                }
            }
        }
        let best = found.iter().map(|(_, h, _)| h.score).fold(0.0, f64::max);
        let hits = found
            .into_iter()
            .map(|(_, mut hit, hash)| {
                // Without meaning scores the order stays exactly the keyword order (search FR-020).
                let coverage = match meaning {
                    Some(_) if !terms.is_empty() => hit.matched as f64 / terms.len() as f64,
                    _ => 1.0,
                };
                let keyword = if best > 0.0 {
                    hit.score.max(0.0) / best * coverage
                } else {
                    0.0
                };
                hit.meaning = meaning
                    .and_then(|m| m.get(&hash))
                    .map_or(0.0, |c| meaning_score(f64::from(*c)));
                hit.score = blend(keyword, hit.meaning);
                hit
            })
            .filter(|hit| hit.score > 0.0)
            .collect();
        Ok((rank(hits, today, limit), terms.len()))
    }

    /// Semantic search: every vector of `model`, by text hash.
    pub fn vectors(&self, model: &str) -> rusqlite::Result<HashMap<String, Vec<f32>>> {
        self.db
            .prepare("SELECT text_hash, vector FROM chunk_vectors WHERE model = ?1")?
            .query_map(params![model], |r| {
                Ok((r.get(0)?, decode(&r.get::<_, Vec<u8>>(1)?)))
            })?
            .collect()
    }

    /// Chunk texts with no vector of `model`, one per text: long-term memory first, then the newest notes
    /// (search R8).
    pub fn missing(&self, model: &str, limit: usize) -> rusqlite::Result<Vec<(String, String)>> {
        self.db
            .prepare(
                "SELECT text_hash, min(text) FROM chunks c
                 WHERE NOT EXISTS (
                     SELECT 1 FROM chunk_vectors v WHERE v.text_hash = c.text_hash AND v.model = ?1
                 )
                 GROUP BY text_hash ORDER BY max(day IS NOT NULL), max(day) DESC, min(id) LIMIT ?2",
            )?
            .query_map(params![model, limit as i64], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect()
    }

    /// How many distinct chunk texts have a vector of `model`, out of how many.
    pub fn prepared(&self, model: &str) -> rusqlite::Result<(u32, u32)> {
        self.db.query_row(
            "SELECT (SELECT count(DISTINCT c.text_hash) FROM chunks c
                     JOIN chunk_vectors v ON v.text_hash = c.text_hash AND v.model = ?1),
                    (SELECT count(DISTINCT text_hash) FROM chunks)",
            params![model],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
    }

    /// Stores vectors, each only while some chunk still holds its text, so a note edited or forgotten while it was
    /// being embedded leaves nothing behind (search R8). Returns the ones kept.
    pub fn store_vectors(
        &mut self,
        model: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> rusqlite::Result<Vec<(String, Vec<f32>)>> {
        let tx = self.db.transaction()?;
        let mut kept = Vec::new();
        for (hash, vector) in vectors {
            let stored = tx.execute(
                "INSERT OR REPLACE INTO chunk_vectors (text_hash, model, vector)
                 SELECT ?1, ?2, ?3 WHERE EXISTS (SELECT 1 FROM chunks WHERE text_hash = ?1)",
                params![hash, model, encode(&vector)],
            )?;
            if stored > 0 {
                kept.push((hash, vector));
            }
        }
        tx.commit()?;
        Ok(kept)
    }

    /// Deletes the vectors of every model but `keep`; with none, every vector.
    pub fn drop_vectors(&self, keep: Option<&str>) -> rusqlite::Result<()> {
        self.db
            .execute(
                "DELETE FROM chunk_vectors WHERE ?1 IS NULL OR model <> ?1",
                params![keep],
            )
            .map(drop)
    }
}

/// A vector as stored: little-endian `f32` values (search data 1).
pub(crate) fn encode(vector: &[f32]) -> Vec<u8> {
    vector.iter().flat_map(|v| v.to_le_bytes()).collect()
}

pub(crate) fn decode(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect()
}

/// A recalled daily-note line: the evidence dreaming ranks on (dreaming R1).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Recall {
    pub path: String,
    pub line: usize,
    pub text_hash: String,
    pub query_hash: String,
    pub coverage: f64,
    pub day: String,
    /// Unix seconds.
    pub at: i64,
}

/// The fingerprint a note line is known by in `recall_events`: its text without the bookkeeping comment.
pub(crate) fn line_hash(line: &str) -> String {
    fingerprint(without_comment(line).trim())
}

impl Core {
    /// Keeps the daily-note hits of a query as recall evidence: which line, a fingerprint of the query (never its
    /// text), how much of the query it matched, and the local day (dreaming FR-009).
    pub(crate) fn record_recalls(
        &self,
        query: &str,
        hits: &[Hit],
        day: &str,
        source: &str,
    ) -> Result<()> {
        let mut words = terms(query);
        if words.is_empty() {
            return Ok(());
        }
        words.sort();
        let query_hash = fingerprint(&words.join(" "));
        let at = crate::config::now() as i64;
        self.with_index(|i| {
            for hit in hits.iter().filter(|h| h.day.is_some()) {
                i.db.execute(
                    "INSERT INTO recall_events (path, line, text_hash, query_hash, coverage, day, source, at) \n                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        hit.path,
                        hit.first as i64,
                        fingerprint(hit.text.trim()),
                        query_hash,
                        hit.relevance(words.len()),
                        day,
                        source,
                        at
                    ],
                )?;
            }
            Ok(())
        })
    }

    /// Every recall event, oldest first.
    pub(crate) fn recalls(&self) -> Result<Vec<Recall>> {
        self.with_index(|i| {
            i.db.prepare(
                "SELECT path, line, text_hash, query_hash, coverage, day, at FROM recall_events ORDER BY id",
            )?
            .query_map([], |r| {
                Ok(Recall {
                    path: r.get(0)?,
                    line: r.get::<_, i64>(1)? as usize,
                    text_hash: r.get(2)?,
                    query_hash: r.get(3)?,
                    coverage: r.get(4)?,
                    day: r.get(5)?,
                    at: r.get(6)?,
                })
            })?
            .collect()
        })
    }

    /// Indexes every memory file that changed, with no size limit: run at launch on a background thread, so the
    /// first search finds the index ready (memory R12).
    pub fn sync_memory_index(&self) -> Result<()> {
        let workspace = self.workspace_dir();
        self.with_index(|i| i.refresh(&workspace, i64::MAX))
    }

    /// Searches memory after bringing the index up to date (memory FR-017, FR-023). Returns the hits and the number of
    /// query terms searched by.
    pub(crate) fn search_memory(
        &self,
        query: &str,
        today: Option<&str>,
        limit: usize,
        wait: bool,
    ) -> Result<(Vec<Hit>, usize)> {
        let workspace = self.workspace_dir();
        self.with_index(|i| i.refresh(&workspace, REFRESH_BYTES))?;
        // Semantic search: by meaning too when the model can answer; `wait` lets it load first (search R5, R6).
        let meaning = self.query_meaning(query, wait);
        self.with_index(|i| i.search(query, today, limit, meaning.as_ref()))
    }

    /// The content of the profile or long-term memory before a companion write (memory R13).
    pub(crate) fn save_preimage(
        &self,
        path: &str,
        before: &str,
        after: &str,
        message: &str,
    ) -> Result<()> {
        let at = crate::config::now() as i64;
        self.with_index(|i| {
            i.db.execute(
                "INSERT INTO preimages (path, before, after_hash, message, at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![path, before, fingerprint(after), message, at],
            )
            .map(drop)
        })
    }

    /// The preimages one write saved, by its message (a dream saves `dream:<run id>`): id, path, content before,
    /// fingerprint after (dreaming R8, R9).
    pub(crate) fn preimages_of(&self, message: &str) -> Result<Vec<(i64, String, String, String)>> {
        self.with_index(|i| {
            i.db.prepare(
                "SELECT id, path, before, after_hash FROM preimages WHERE message = ?1 ORDER BY id",
            )?
            .query_map(params![message], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })?
            .collect()
        })
    }

    pub(crate) fn drop_preimage(&self, id: i64) -> Result<()> {
        self.with_index(|i| i.delete_preimage(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_schema_change_rebuilds_the_index_and_keeps_what_cannot_be_derived() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pec.db");
        let index = Index::open(&path).unwrap();
        index
            .db
            .execute_batch(
                "INSERT INTO files VALUES ('MEMORY.md', 1, 1, 1);
                 INSERT INTO preimages (path, before, after_hash, message, at)
                     VALUES ('USER.md', 'old', 'h', 'm', 1);
                 INSERT INTO recall_events (path, line, text_hash, query_hash, coverage, day, source, at)
                     VALUES ('memory/2026-11-14.md', 3, 't', 'q', 0.5, '2026-11-14', 'auto', 1);
                 INSERT INTO dream_runs (night, started_at, since, phase, checkpoint)
                     VALUES ('2026-11-14', 1, 0, 'light', '{}');
                 UPDATE meta SET value = '0' WHERE key = 'schema_version';",
            )
            .unwrap();
        drop(index);

        let index = Index::open(&path).unwrap();
        let count = |table: &str| -> i64 {
            index
                .db
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(count("files"), 0);
        assert_eq!(count("preimages"), 1);
        // Dreaming: recall evidence and run history are not derived from the files either.
        assert_eq!(count("recall_events"), 1);
        assert_eq!(count("dream_runs"), 1);
    }

    /// A workspace with long-term memory and two notes, indexed.
    fn indexed(dir: &Path) -> Index {
        let workspace = dir.join("workspace");
        std::fs::create_dir_all(workspace.join("memory")).unwrap();
        std::fs::write(
            workspace.join("MEMORY.md"),
            "# Memory\n\n## Pets\n- Biscuit is a beagle.\n",
        )
        .unwrap();
        std::fs::write(
            workspace.join("memory/2026-11-01.md"),
            "- 09:00 Old note.\n",
        )
        .unwrap();
        std::fs::write(
            workspace.join("memory/2026-11-05.md"),
            "- 09:00 New note.\n",
        )
        .unwrap();
        let mut index = Index::open(&dir.join("pec.db")).unwrap();
        index.refresh(&workspace, i64::MAX).unwrap();
        index
    }

    #[test]
    fn vectors_are_kept_by_text_only_while_a_chunk_holds_it() {
        let dir = tempfile::tempdir().unwrap();
        let mut index = indexed(dir.path());
        // search R8: long-term memory first, then the newest notes.
        let missing = index.missing("m@1", 10).unwrap();
        let texts: Vec<&str> = missing.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Pets\n- Biscuit is a beagle.",
                "- 09:00 New note.",
                "- 09:00 Old note."
            ]
        );
        assert_eq!(index.prepared("m@1").unwrap(), (0, 3));

        // A vector round-trips as little-endian f32; one for text no chunk holds is never stored.
        let vector = vec![0.25f32, -1.5, 3.0];
        assert_eq!(decode(&encode(&vector)), vector);
        let kept = index
            .store_vectors(
                "m@1",
                vec![
                    (missing[0].0.clone(), vector.clone()),
                    ("gone".into(), vector.clone()),
                ],
            )
            .unwrap();
        assert_eq!(kept.len(), 1);
        assert_eq!(index.vectors("m@1").unwrap()[&missing[0].0], vector);
        assert_eq!(index.prepared("m@1").unwrap(), (1, 3));

        // search FR-017: the vector goes with the last chunk that held its text.
        std::fs::write(dir.path().join("workspace/MEMORY.md"), "# Memory\n").unwrap();
        index
            .refresh(&dir.path().join("workspace"), i64::MAX)
            .unwrap();
        assert!(index.vectors("m@1").unwrap().is_empty());

        // Other models' vectors go when asked; all of them on removal.
        let hash = index.missing("m@1", 1).unwrap()[0].0.clone();
        index
            .store_vectors("m@1", vec![(hash.clone(), vector.clone())])
            .unwrap();
        index.store_vectors("old@0", vec![(hash, vector)]).unwrap();
        index.drop_vectors(Some("m@1")).unwrap();
        assert!(
            index.vectors("old@0").unwrap().is_empty() && index.vectors("m@1").unwrap().len() == 1
        );
        index.drop_vectors(None).unwrap();
        assert!(index.vectors("m@1").unwrap().is_empty());
    }
}
