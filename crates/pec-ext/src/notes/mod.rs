//! Sticky notes: one Markdown file each in `notes/`, with their place, colour and visibility as bookkeeping in
//! `state/notes.json` (extensions R7, R8; extensions data 1, 2; extensions interfaces).

use pec_core::ext::{ExtDir, Extension, Tick, ToolAnswer, write_atomic};
use pec_core::llm::ToolSpec;
use pec_core::memory::{MarkKind, MemoryMark, SaveOutcome, fingerprint};
use pec_core::{AppError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;
use time::{OffsetDateTime, PrimitiveDateTime};
use ts_rs::TS;

/// extensions FR-016.
pub const MAX_CHARS: usize = 4_000;
const FULL: &str = "That note is full (4,000 characters). Suggest a new note instead.";
const ID_CHARS: &str = "23456789abcdefghjkmnpqrstuvwxyz";

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum NoteColour {
    #[default]
    Yellow,
    Green,
    Blue,
    Pink,
    Grey,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NoteSummary {
    pub file: String,
    pub first_line: String,
    pub shown: bool,
    pub colour: NoteColour,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NoteDoc {
    pub file: String,
    pub text: String,
    /// Sent back with a save, so a change made meanwhile is never saved over (extensions FR-008).
    pub hash: String,
    pub colour: NoteColour,
    pub on_top: bool,
}

/// A note window's place and size in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Place {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// One note's bookkeeping; lost, the note is hidden, yellow and placed by the default rule.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Style {
    shown: bool,
    on_top: bool,
    colour: NoteColour,
    #[serde(flatten)]
    place: Option<Place>,
}

type Styles = BTreeMap<String, Style>;

/// `<slug>-<id>.md`, directly in `notes/` (extensions data 1).
fn valid_name(file: &str) -> bool {
    let Some((slug, id)) = file.strip_suffix(".md").and_then(|s| s.rsplit_once('-')) else {
        return false;
    };
    let slug_ok = (1..=32).contains(&slug.len())
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    slug_ok && id.len() == 6 && id.chars().all(|c| ID_CHARS.contains(c))
}

/// The slug of a new note: its first line, lowercased, letters and digits joined by `-`, at most 32 characters.
fn slug(text: &str) -> String {
    let first = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default();
    let words: Vec<String> = first
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let mut slug = String::new();
    for word in words {
        let next = if slug.is_empty() {
            word
        } else {
            format!("{slug}-{word}")
        };
        if next.len() > 32 {
            break;
        }
        slug = next;
    }
    if slug.is_empty() { "note".into() } else { slug }
}

fn first_line(text: &str) -> String {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    match line.char_indices().nth(80) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line.into(),
    }
}

/// The path of an existing note, or `InvalidInput("file")`.
fn path(dir: &ExtDir, file: &str) -> Result<std::path::PathBuf> {
    let path = dir.file(file)?;
    if !valid_name(file) || !path.is_file() {
        return Err(AppError::invalid("file"));
    }
    Ok(path)
}

fn too_long(text: &str) -> bool {
    text.chars().count() > MAX_CHARS
}

/// Every note, shown or hidden, newest first (extensions FR-015).
pub fn list(dir: &ExtDir) -> Vec<NoteSummary> {
    let styles: Styles = dir.read_state();
    let mut notes: Vec<(std::time::SystemTime, NoteSummary)> = std::fs::read_dir(dir.place())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let file = entry.file_name().to_string_lossy().into_owned();
            let meta = entry
                .metadata()
                .ok()
                .filter(|m| m.is_file() && valid_name(&file))?;
            let text = std::fs::read_to_string(entry.path()).ok()?;
            let style = styles.get(&file).cloned().unwrap_or_default();
            let summary = NoteSummary {
                first_line: first_line(&text),
                shown: style.shown,
                colour: style.colour,
                file,
            };
            Some((meta.modified().ok()?, summary))
        })
        .collect();
    notes.sort_by_key(|n| std::cmp::Reverse(n.0));
    notes.into_iter().map(|(_, n)| n).collect()
}

/// A new note, shown (extensions FR-011).
pub fn create(dir: &ExtDir, text: &str) -> Result<NoteSummary> {
    if too_long(text) {
        return Err(AppError::invalid("text"));
    }
    let _guard = dir.lock();
    let file = loop {
        let name = format!(
            "{}-{}.md",
            slug(text),
            pec_core::world::random_chars(6).to_ascii_lowercase()
        );
        if !dir.file(&name)?.exists() {
            break name;
        }
    };
    write_atomic(&dir.file(&file)?, text.as_bytes())?;
    let mut styles: Styles = dir.read_state();
    let style = styles.entry(file.clone()).or_default();
    style.shown = true;
    let colour = style.colour;
    dir.write_state(&styles)?;
    Ok(NoteSummary {
        first_line: first_line(text),
        shown: true,
        colour,
        file,
    })
}

pub fn read(dir: &ExtDir, file: &str) -> Result<NoteDoc> {
    let text = std::fs::read_to_string(path(dir, file)?).map_err(|_| AppError::invalid("file"))?;
    let style = dir.read_state::<Styles>().remove(file).unwrap_or_default();
    Ok(NoteDoc {
        file: file.into(),
        hash: fingerprint(&text),
        text,
        colour: style.colour,
        on_top: style.on_top,
    })
}

/// Saves only when the file is still as it was read (extensions FR-008, FR-014).
pub fn save(dir: &ExtDir, file: &str, text: &str, base_hash: &str) -> Result<SaveOutcome> {
    if too_long(text) {
        return Err(AppError::invalid("text"));
    }
    let path = path(dir, file)?;
    let _guard = dir.lock();
    let current = std::fs::read_to_string(&path).map_err(|_| AppError::invalid("file"))?;
    if fingerprint(&current) != base_hash {
        return Ok(SaveOutcome::Conflict {
            hash: fingerprint(&current),
            text: current,
        });
    }
    write_atomic(&path, text.as_bytes())?;
    Ok(SaveOutcome::Saved {
        hash: fingerprint(text),
    })
}

fn restyle(dir: &ExtDir, file: &str, change: impl FnOnce(&mut Style)) -> Result<()> {
    path(dir, file)?;
    let _guard = dir.lock();
    let mut styles: Styles = dir.read_state();
    change(styles.entry(file.into()).or_default());
    dir.write_state(&styles)
}

pub fn style(
    dir: &ExtDir,
    file: &str,
    colour: Option<NoteColour>,
    on_top: Option<bool>,
) -> Result<NoteDoc> {
    restyle(dir, file, |s| {
        s.colour = colour.unwrap_or(s.colour);
        s.on_top = on_top.unwrap_or(s.on_top);
    })?;
    read(dir, file)
}

pub fn set_shown(dir: &ExtDir, file: &str, shown: bool) -> Result<()> {
    restyle(dir, file, |s| s.shown = shown)
}

pub fn set_place(dir: &ExtDir, file: &str, place: Place) -> Result<()> {
    restyle(dir, file, |s| s.place = Some(place))
}

pub fn place(dir: &ExtDir, file: &str) -> Option<Place> {
    dir.read_state::<Styles>().remove(file)?.place
}

/// The notes to put on the desktop, and whether each stays on top.
pub fn shown(dir: &ExtDir) -> Vec<(String, bool)> {
    list(dir)
        .into_iter()
        .filter(|n| n.shown)
        .map(|n| {
            let on_top = dir
                .read_state::<Styles>()
                .get(&n.file)
                .is_some_and(|s| s.on_top);
            (n.file, on_top)
        })
        .collect()
}

/// Moves the file into `notes/deleted/`; nothing is erased (extensions FR-015).
pub fn delete(dir: &ExtDir, file: &str) -> Result<()> {
    let from = path(dir, file)?;
    let _guard = dir.lock();
    let stem = file.trim_end_matches(".md");
    let to = (1..)
        .map(|n| {
            if n == 1 {
                format!("deleted/{file}")
            } else {
                format!("deleted/{stem}-{n}.md")
            }
        })
        .map(|name| dir.file(&name))
        .find(|p| p.as_ref().map_or(true, |p| !p.exists()))
        .expect("a free name")?;
    std::fs::create_dir_all(dir.file("deleted")?).map_err(|_| AppError::not_writable(&to))?;
    std::fs::rename(&from, &to).map_err(|_| AppError::not_writable(&from))?;
    let mut styles: Styles = dir.read_state();
    styles.remove(file);
    dir.write_state(&styles)
}

fn specs() -> Vec<ToolSpec> {
    let note = json!({ "type": "string", "description": "A note's file name, from note_list." });
    let object = |properties: Value, required: &[&str]| json!({ "type": "object", "properties": properties, "required": required, "additionalProperties": false });
    vec![
        ToolSpec {
            name: "note_create",
            description: "Put a new sticky note on the person's desktop. Only when the person asks, or agrees to \
                          your suggestion.",
            input_schema: object(
                json!({ "text": { "type": "string", "description": "The note, Markdown, 1 to 4,000 characters. The first line is its title." } }),
                &["text"],
            ),
        },
        ToolSpec {
            name: "note_list",
            description: "List the person's sticky notes: file name, shown or hidden, first line.",
            input_schema: object(json!({}), &[]),
        },
        ToolSpec {
            name: "note_read",
            description: "Read a sticky note as it is now, with the person's own edits.",
            input_schema: object(json!({ "note": note }), &["note"]),
        },
        ToolSpec {
            name: "note_write",
            description: "Add to a sticky note or replace its text, when the person asks.",
            input_schema: object(
                json!({
                    "note": note,
                    "text": { "type": "string", "description": "The text to add or the new text, 1 to 4,000 characters." },
                    "mode": { "type": "string", "enum": ["append", "replace"] }
                }),
                &["note", "text", "mode"],
            ),
        },
    ]
}

fn mark(kind: MarkKind, file: &str, text: &str) -> Option<MemoryMark> {
    Some(MemoryMark {
        kind,
        path: format!("notes/{file}"),
        line: None,
        excerpt: first_line(text),
        ok: true,
    })
}

fn run(tool: &str, input: &Value, dir: &ExtDir) -> ToolAnswer {
    let text = input["text"].as_str().unwrap_or_default();
    let note = input["note"].as_str().unwrap_or_default();
    let missing = || {
        ToolAnswer::error(format!(
            "There is no note named {note}. List the notes first."
        ))
    };
    match tool {
        "note_list" => {
            let lines: Vec<String> = list(dir)
                .into_iter()
                .map(|n| {
                    format!(
                        "- {} ({}): {}",
                        n.file,
                        if n.shown { "shown" } else { "hidden" },
                        n.first_line
                    )
                })
                .collect();
            ToolAnswer::ok(
                if lines.is_empty() {
                    "No notes yet.".into()
                } else {
                    lines.join("\n")
                },
                None,
            )
        }
        "note_read" => match read(dir, note) {
            Ok(doc) => ToolAnswer::ok(doc.text, None),
            Err(_) => missing(),
        },
        "note_create" if text.trim().is_empty() => {
            ToolAnswer::error("The input was not valid: text must be 1 to 4,000 characters.")
        }
        "note_create" if too_long(text) => ToolAnswer::error(FULL),
        "note_create" => match create(dir, text) {
            Ok(n) => ToolAnswer::ok(
                format!("Made the note \"{}\".", n.first_line),
                mark(MarkKind::NoteMade, &n.file, text),
            ),
            Err(_) => ToolAnswer::error("The note can't be written right now."),
        },
        "note_write" => {
            let Ok(doc) = read(dir, note) else {
                return missing();
            };
            let next = match input["mode"].as_str() {
                Some("append") if doc.text.is_empty() => text.to_owned(),
                Some("append") => format!("{}\n{text}", doc.text.trim_end_matches('\n')),
                Some("replace") => text.to_owned(),
                _ => {
                    return ToolAnswer::error(
                        "The input was not valid: mode must be append or replace.",
                    );
                }
            };
            if text.trim().is_empty() {
                return ToolAnswer::error(
                    "The input was not valid: text must be 1 to 4,000 characters.",
                );
            }
            if too_long(&next) {
                return ToolAnswer::error(FULL);
            }
            match save(dir, note, &next, &doc.hash) {
                Ok(SaveOutcome::Saved { .. }) => ToolAnswer::ok(
                    format!("Updated the note \"{}\".", first_line(&next)),
                    mark(MarkKind::NoteChanged, note, &next),
                ),
                _ => ToolAnswer::error(
                    "The person is editing that note right now. Try again in a moment.",
                ),
            }
        }
        _ => ToolAnswer::error(format!("There is no tool named {tool}.")),
    }
}

/// A fingerprint of the notes folder: names, sizes and times (extensions R9).
fn signature(folder: &Path) -> Option<String> {
    let mut parts: Vec<String> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            Some(format!(
                "{:?}{}{:?}",
                e.file_name(),
                meta.len(),
                meta.modified().ok()?
            ))
        })
        .collect();
    parts.sort();
    Some(fingerprint(&parts.concat()))
}

/// The sticky notes extension.
#[derive(Default)]
pub struct Notes {
    seen: Mutex<Option<String>>,
}

impl Extension for Notes {
    fn id(&self) -> &'static str {
        "notes"
    }

    fn name(&self) -> &'static str {
        "Sticky notes"
    }

    fn about(&self) -> &'static str {
        "Small notes on your desktop that you or the companion write. The companion can make, list, read and add to \
         them when you ask."
    }

    fn skill(&self) -> &'static str {
        include_str!("SKILL.md")
    }

    fn place(&self) -> &'static str {
        "notes/"
    }

    fn tools(&self) -> Vec<ToolSpec> {
        specs()
    }

    fn run(&self, tool: &str, input: &Value, dir: &ExtDir, _now: PrimitiveDateTime) -> ToolAnswer {
        run(tool, input, dir)
    }

    fn tick(&self, _now: OffsetDateTime, dir: &ExtDir) -> Tick {
        let signature = signature(dir.place());
        let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
        let changed = *seen != signature;
        *seen = signature;
        Tick {
            changed,
            ..Tick::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_names_are_made_and_checked_strictly() {
        assert_eq!(slug("Packing list: Bali!\n- shirts"), "packing-list-bali");
        assert_eq!(slug("\n\n  "), "note");
        assert_eq!(slug("ÜBER café"), "ber-caf");
        assert_eq!(slug(&"word ".repeat(20)).len(), 29);
        for good in ["packing-list-k7q2mx.md", "note-222222.md"] {
            assert!(valid_name(good), "{good}");
        }
        for bad in [
            "packing-list-k7q2m.md",
            "Packing-k7q2mx.md",
            "k7q2mx.md",
            "a-k7q2m0.md",
            "a-k7q2mx.txt",
            "../a-k7q2mx.md",
        ] {
            assert!(!valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn a_note_saves_only_over_what_was_read_and_moves_to_deleted() {
        let (_root, core) = crate::testing::core();
        let dir = core.ext_dir("notes").unwrap();
        let made = create(&dir, "Packing list\n- shirts").unwrap();
        assert!(made.shown && made.file.starts_with("packing-list-"));
        let doc = read(&dir, &made.file).unwrap();
        assert!(matches!(
            save(&dir, &made.file, "Packing list\n- hats", &doc.hash),
            Ok(SaveOutcome::Saved { .. })
        ));
        // A stale hash is a conflict and writes nothing.
        let outcome = save(&dir, &made.file, "Overwritten", &doc.hash).unwrap();
        assert!(
            matches!(outcome, SaveOutcome::Conflict { ref text, .. } if text == "Packing list\n- hats")
        );
        assert_eq!(
            save(&dir, &made.file, &"a".repeat(4_001), "x"),
            Err(AppError::invalid("text"))
        );
        style(&dir, &made.file, Some(NoteColour::Blue), Some(true)).unwrap();
        assert_eq!(list(&dir)[0].colour, NoteColour::Blue);
        // Lost bookkeeping: the note comes back hidden and yellow.
        std::fs::remove_file(_root.path().join("state/notes.json")).unwrap();
        assert_eq!(
            (list(&dir)[0].shown, list(&dir)[0].colour),
            (false, NoteColour::Yellow)
        );
        delete(&dir, &made.file).unwrap();
        assert!(list(&dir).is_empty());
        assert!(
            dir.file(&format!("deleted/{}", made.file))
                .unwrap()
                .is_file()
        );
        assert_eq!(read(&dir, "nope-k7q2mx.md"), Err(AppError::invalid("file")));
    }

    #[test]
    fn the_companion_makes_lists_reads_and_writes_notes() {
        let (_root, core) = crate::testing::core();
        let dir = core.ext_dir("notes").unwrap();
        let ext = Notes::default();
        let now = time::macros::datetime!(2026-11-14 18:00);
        let said = |tool: &str, input: Value| {
            let answer = ext.run(tool, &input, &dir, now);
            (
                answer.content,
                answer.is_error,
                answer.mark.map(|m| (m.kind, m.path, m.excerpt)),
            )
        };
        assert_eq!(said("note_list", json!({})).0, "No notes yet.");
        let (made, error, mark) = said("note_create", json!({ "text": "Packing list\n- shirts" }));
        assert_eq!(
            (made.as_str(), error),
            ("Made the note \"Packing list\".", false)
        );
        let (kind, path, excerpt) = mark.unwrap();
        assert_eq!(
            (kind, excerpt.as_str()),
            (MarkKind::NoteMade, "Packing list")
        );
        let file = path.strip_prefix("notes/").unwrap().to_owned();
        assert_eq!(
            said("note_list", json!({})).0,
            format!("- {file} (shown): Packing list")
        );

        // The person's own edit is what the companion reads.
        std::fs::write(
            dir.file(&file).unwrap(),
            "Packing list\n- shirts\n- charger",
        )
        .unwrap();
        assert_eq!(
            said("note_read", json!({ "note": file })).0,
            "Packing list\n- shirts\n- charger"
        );
        let (answer, _, mark) = said(
            "note_write",
            json!({ "note": file, "text": "- hat", "mode": "append" }),
        );
        assert_eq!(answer, "Updated the note \"Packing list\".");
        assert_eq!(mark.unwrap().0, MarkKind::NoteChanged);
        assert_eq!(
            read(&dir, &file).unwrap().text,
            "Packing list\n- shirts\n- charger\n- hat"
        );
        said(
            "note_write",
            json!({ "note": file, "text": "Groceries", "mode": "replace" }),
        );
        assert_eq!(read(&dir, &file).unwrap().text, "Groceries");

        let full = "a".repeat(3_995);
        assert_eq!(
            said(
                "note_write",
                json!({ "note": file, "text": full, "mode": "append" })
            ),
            (FULL.into(), true, None)
        );
        assert_eq!(
            said("note_create", json!({ "text": "a".repeat(4_001) })),
            (FULL.into(), true, None)
        );
        assert_eq!(
            said("note_read", json!({ "note": "gone-k7q2mx.md" })).0,
            "There is no note named gone-k7q2mx.md. List the notes first."
        );
    }
}
