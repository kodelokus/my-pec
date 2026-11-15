//! The memory files, their formats and their one writer (memory data 1, memory R5, R6).

use crate::workspace::write_atomic;
use crate::{AppError, Core, Result};

/// A memory file the browser and the tools may name. Anything else is refused (memory interfaces).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MemoryPath {
    Soul,
    Profile,
    Memory,
    Daily(String),
    /// `DREAMS.md`: the person's to read and edit; never a memory source (dreaming FR-021).
    Diary,
}

impl MemoryPath {
    pub fn parse(path: &str) -> Result<Self> {
        match path {
            "SOUL.md" => Ok(Self::Soul),
            "USER.md" => Ok(Self::Profile),
            "MEMORY.md" => Ok(Self::Memory),
            "DREAMS.md" => Ok(Self::Diary),
            _ => path
                .strip_prefix("memory/")
                .and_then(|name| name.strip_suffix(".md"))
                .filter(|date| is_date(date))
                .map(|date| Self::Daily(date.into()))
                .ok_or_else(|| AppError::invalid("path")),
        }
    }

    /// The workspace-relative path.
    pub fn rel(&self) -> String {
        match self {
            Self::Soul => "SOUL.md".into(),
            Self::Profile => "USER.md".into(),
            Self::Memory => "MEMORY.md".into(),
            Self::Daily(date) => format!("memory/{date}.md"),
            Self::Diary => "DREAMS.md".into(),
        }
    }
}

/// `YYYY-MM-DD` naming a real calendar day.
pub(crate) fn is_date(text: &str) -> bool {
    let b = text.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let number = |r: std::ops::Range<usize>| -> Option<u32> {
        text.get(r)
            .filter(|s| s.bytes().all(|c| c.is_ascii_digit()))?
            .parse()
            .ok()
    };
    let (Some(year), Some(month), Some(day)) = (number(0..4), number(5..7), number(8..10)) else {
        return false;
    };
    (1..=days_in_month(year, month)).contains(&day)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    }
}

/// Days since 1970-01-01 for a valid `YYYY-MM-DD` (Howard Hinnant's days-from-civil), for ages in days.
pub(crate) fn day_number(date: &str) -> Option<i64> {
    if !is_date(date) {
        return None;
    }
    let n = |r: std::ops::Range<usize>| date[r].parse::<i64>().unwrap_or(1);
    let (month, day) = (n(5..7), n(8..10));
    let year = n(0..4) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let doy = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    Some(era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468)
}

/// The calendar day before a valid `YYYY-MM-DD`.
pub(crate) fn previous_day(date: &str) -> String {
    let n = |r: std::ops::Range<usize>| date[r].parse::<u32>().unwrap_or(1);
    let (mut year, mut month, mut day) = (n(0..4), n(5..7), n(8..10));
    if day > 1 {
        day -= 1;
    } else {
        (year, month) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        day = days_in_month(year, month);
    }
    format!("{year:04}-{month:02}-{day:02}")
}

/// The dates of the validly named daily notes, newest first. Other files in `memory/` are not daily notes.
pub(crate) fn daily_dates(workspace: &std::path::Path) -> Vec<String> {
    let mut days: Vec<String> = std::fs::read_dir(workspace.join("memory"))
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let name = e.ok()?.file_name().into_string().ok()?;
            let date = name.strip_suffix(".md")?;
            is_date(date).then(|| date.to_owned())
        })
        .collect();
    days.sort_unstable_by(|a, b| b.cmp(a));
    days
}

/// A stable 64-bit FNV-1a hash in hex: the same text gives the same value in every app version.
pub fn fingerprint(text: &str) -> String {
    let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{hash:016x}")
}

/// Size budgets in characters (architecture section 15, memory FR-011, FR-015).
pub(crate) const PROFILE_BUDGET: usize = 4_000;
pub(crate) const MEMORY_BUDGET: usize = 12_000;

/// Who a memory write came from (memory FR-002). Untrusted and system content has no variant, so it can't be written
/// to memory at all (memory FR-013, Principle IV).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Origin {
    User,
    Companion,
}

impl Origin {
    fn name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Companion => "companion",
        }
    }
}

/// The user's local date and time at the moment of a write: `YYYY-MM-DD` and `HH:MM`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LocalNow {
    pub date: String,
    pub time: String,
}

/// Where a write landed. `kept_in_note`: the file was full, so the fact went to today's note (memory FR-011).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Landed {
    pub path: String,
    pub line: usize,
    pub kept_in_note: bool,
    /// The whole profile or long-term memory after the write, for the conversation's `written` entry.
    pub content: Option<String>,
}

/// A daily note line: `- HH:MM text <!-- from:origin src:message -->` (memory data 1).
pub(crate) fn note_line(time: &str, text: &str, origin: Origin, src: &str) -> String {
    format!("- {time} {text} <!-- from:{} src:{src} -->", origin.name())
}

/// The text of a daily note line, between its time and its comment.
pub(crate) fn note_text(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("- ")?;
    let time = rest.get(..5)?.as_bytes();
    if time[2] != b':' || !time[..2].iter().chain(&time[3..]).all(u8::is_ascii_digit) {
        return None;
    }
    let rest = rest.get(5..)?.strip_prefix(' ')?;
    Some(rest.split(" <!--").next().unwrap_or(rest).trim())
}

/// Appends `line` at the end, after a `# date` heading when the file is new. Returns the text and the line number.
pub(crate) fn append_line(current: &str, heading: &str, line: &str) -> (String, usize) {
    let mut next = if current.trim().is_empty() {
        format!("# {heading}\n\n")
    } else if current.ends_with('\n') {
        current.to_owned()
    } else {
        format!("{current}\n")
    };
    next.push_str(line);
    next.push('\n');
    let number = next.lines().count();
    (next, number)
}

/// Adds a bullet at the end of its `## topic` section, or a new section at the end of the file (memory R5).
pub(crate) fn add_entry(current: &str, topic: &str, entry: &str) -> (String, usize) {
    let lines: Vec<&str> = current.lines().collect();
    let heading = lines.iter().position(|l| {
        l.strip_prefix("## ")
            .is_some_and(|t| t.trim().eq_ignore_ascii_case(topic.trim()))
    });
    let Some(heading) = heading else {
        let mut next = current.trim_end().to_owned();
        if !next.is_empty() {
            next.push_str("\n\n");
        }
        next.push_str(&format!("## {}\n{entry}\n", topic.trim()));
        let number = next.lines().count();
        return (next, number);
    };
    let end = lines[heading + 1..]
        .iter()
        .position(|l| l.starts_with("# ") || l.starts_with("## "))
        .map_or(lines.len(), |i| heading + 1 + i);
    let at = (heading + 1..end)
        .rev()
        .find(|&i| !lines[i].trim().is_empty())
        .map_or(heading + 1, |i| i + 1);
    let mut out: Vec<&str> = lines[..at].to_vec();
    out.push(entry);
    out.extend(&lines[at..]);
    (out.join("\n") + "\n", at + 1)
}

/// Replaces the value of the first `- Key: value` line whose key matches, or appends one (memory R5).
pub(crate) fn set_profile(current: &str, key: &str, value: &str) -> (String, usize) {
    let mut lines: Vec<String> = current.lines().map(str::to_owned).collect();
    let found = lines.iter().position(|l| {
        l.trim()
            .strip_prefix("- ")
            .and_then(|rest| rest.split_once(':'))
            .is_some_and(|(k, _)| k.trim().eq_ignore_ascii_case(key.trim()))
    });
    match found {
        Some(i) => {
            let existing = lines[i]
                .trim()
                .strip_prefix("- ")
                .and_then(|r| r.split_once(':'));
            let name = existing.map_or(key, |(k, _)| k.trim()).to_owned();
            lines[i] = format!("- {name}: {value}");
            (lines.join("\n") + "\n", i + 1)
        }
        None => append_line(
            current,
            "About the user",
            &format!("- {}: {value}", key.trim()),
        ),
    }
}

/// An entry id unique in the file: `m_` and 4 to 8 hex characters, then a counter if needed.
pub(crate) fn entry_id(current: &str, seed: &str) -> String {
    let hash = fingerprint(seed);
    [4, 6, 8]
        .iter()
        .map(|n| format!("m_{}", &hash[..*n]))
        .chain((2..).map(|i| format!("m_{}{i}", &hash[..8])))
        .find(|id| !current.contains(&format!("id:{id} ")))
        .expect("an unused id exists")
}

impl Core {
    /// memory FR-003, FR-004: a timed note at the end of today's daily note.
    pub(crate) fn add_note(
        &self,
        now: &LocalNow,
        text: &str,
        origin: Origin,
        src: &str,
    ) -> Result<Landed> {
        let path = MemoryPath::Daily(now.date.clone());
        let line = note_line(&now.time, text, origin, src);
        let number = self.edit_memory(&path, |current| {
            let (next, number) = append_line(current, &now.date, &line);
            Ok((next, number))
        })?;
        Ok(Landed {
            path: path.rel(),
            line: number,
            kept_in_note: false,
            content: None,
        })
    }

    /// memory FR-005, FR-011: an entry in long-term memory, or today's note when long-term memory is full.
    pub(crate) fn remember(
        &self,
        now: &LocalNow,
        text: &str,
        topic: &str,
        origin: Origin,
        src: &str,
    ) -> Result<Landed> {
        self.rewrite(&MemoryPath::Memory, now, text, origin, src, |current| {
            let id = entry_id(current, &format!("{text}{}{src}", now.date));
            let entry = format!(
                "- {text} <!-- id:{id} at:{} from:{} src:{src} -->",
                now.date,
                origin.name()
            );
            add_entry(current, topic, &entry)
        })
    }

    /// memory FR-006: the profile's value for `key` replaced in place, or today's note when the profile is full.
    pub(crate) fn update_profile(
        &self,
        now: &LocalNow,
        key: &str,
        value: &str,
        src: &str,
    ) -> Result<Landed> {
        let fact = format!("{}: {value}", key.trim());
        self.rewrite(
            &MemoryPath::Profile,
            now,
            &fact,
            Origin::User,
            src,
            |current| set_profile(current, key, value),
        )
    }

    /// An automatic rewrite of the profile or long-term memory: checked against the budget with a fall back to appending to today's note, and preceded by its preimage
    /// (Principle IV, memory R6, R13).
    fn rewrite(
        &self,
        path: &MemoryPath,
        now: &LocalNow,
        fact: &str,
        origin: Origin,
        src: &str,
        change: impl FnOnce(&str) -> (String, usize),
    ) -> Result<Landed> {
        let budget = match path {
            MemoryPath::Profile => PROFILE_BUDGET,
            _ => MEMORY_BUDGET,
        };
        let written = self.edit_memory(path, |current| {
            let (next, number) = change(current);
            if next.chars().count() > budget {
                return Ok((current.to_owned(), None));
            }
            self.save_preimage(&path.rel(), current, &next, src)?;
            Ok((next.clone(), Some((number, next))))
        })?;
        match written {
            Some((line, content)) => Ok(Landed {
                path: path.rel(),
                line,
                kept_in_note: false,
                content: Some(content),
            }),
            None => {
                let mut landed = self.add_note(now, fact, origin, src)?;
                landed.kept_in_note = true;
                Ok(landed)
            }
        }
    }

    /// The one writer of memory files: reads the file as it is now, applies `change` to that, and writes the
    /// result atomically, so an outside edit is never overwritten (memory FR-010). A missing file reads as empty.
    pub(crate) fn edit_memory<T>(
        &self,
        path: &MemoryPath,
        change: impl FnOnce(&str) -> Result<(String, T)>,
    ) -> Result<T> {
        self.edit_workspace(&path.rel(), change)
    }

    /// The one writer of the workspace files the app changes (world R3): the memory files, and
    /// the identity lines world core adds. Same read-apply-write, same lock.
    pub(crate) fn edit_workspace<T>(
        &self,
        rel: &str,
        change: impl FnOnce(&str) -> Result<(String, T)>,
    ) -> Result<T> {
        let _guard = self.memory.write.lock().unwrap_or_else(|e| e.into_inner());
        let file = self.workspace_dir().join(rel);
        let current = match std::fs::read(&file) {
            Ok(bytes) => String::from_utf8(bytes).map_err(|_| AppError::invalid("unreadable"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(_) => return Err(AppError::not_writable(&file)),
        };
        let (next, out) = change(&current)?;
        if next != current {
            write_atomic(&file, next.as_bytes())?;
        }
        Ok(out)
    }

    /// Appends whole lines to a workspace file under the writer's lock, starting on a new line if the file ends
    /// with a half-written one, and syncs them to disk (world FR-016).
    pub(crate) fn append_workspace(&self, rel: &str, lines: &[String]) -> Result<()> {
        use std::io::{Read, Seek, SeekFrom, Write};
        let _guard = self.memory.write.lock().unwrap_or_else(|e| e.into_inner());
        let path = self.workspace_dir().join(rel);
        let failed = || AppError::not_writable(&path);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|_| failed())?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)
            .map_err(|_| failed())?;
        let mut bytes = Vec::new();
        if file.metadata().map_err(|_| failed())?.len() > 0 {
            let mut last = [0u8];
            file.seek(SeekFrom::End(-1)).map_err(|_| failed())?;
            file.read_exact(&mut last).map_err(|_| failed())?;
            if last[0] != b'\n' {
                bytes.push(b'\n');
            }
        }
        for line in lines {
            bytes.extend_from_slice(line.as_bytes());
            bytes.push(b'\n');
        }
        file.write_all(&bytes).map_err(|_| failed())?;
        file.sync_data().map_err(|_| failed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_memory_files_can_be_named() {
        for (path, parsed) in [
            ("SOUL.md", MemoryPath::Soul),
            ("USER.md", MemoryPath::Profile),
            ("MEMORY.md", MemoryPath::Memory),
            ("DREAMS.md", MemoryPath::Diary),
            (
                "memory/2026-11-14.md",
                MemoryPath::Daily("2026-11-14".into()),
            ),
            (
                "memory/2024-02-29.md",
                MemoryPath::Daily("2024-02-29".into()),
            ),
        ] {
            assert_eq!(MemoryPath::parse(path), Ok(parsed.clone()), "{path}");
            assert_eq!(parsed.rel(), path);
        }
        for path in [
            "IDENTITY.md",
            "memory/2026-13-01.md",
            "memory/2026-02-29.md",
            "memory/notes.md",
            "memory/+026-11-14.md",
            "../USER.md",
            "C:\\x\\USER.md",
            "memory/2026-11-14.md/..",
            "memory/../USER.md",
            "",
        ] {
            assert_eq!(
                MemoryPath::parse(path),
                Err(AppError::invalid("path")),
                "{path}"
            );
        }
    }

    #[test]
    fn the_fingerprint_is_stable() {
        assert_eq!(fingerprint(""), "cbf29ce484222325");
        assert_eq!(fingerprint("a"), "af63dc4c8601ec8c");
        assert_ne!(fingerprint("# Memory\n"), fingerprint("# Memory\n\n"));
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;
    use crate::credentials::{ApiKey, SecretStore, StoreUnavailable};

    struct NoSecrets;

    impl SecretStore for NoSecrets {
        fn get(&self) -> Option<ApiKey> {
            None
        }
        fn set(&self, _: &ApiKey) -> std::result::Result<(), StoreUnavailable> {
            Err(StoreUnavailable)
        }
        fn delete(&self) -> std::result::Result<(), StoreUnavailable> {
            Ok(())
        }
    }

    fn now() -> LocalNow {
        LocalNow {
            date: "2026-11-14".into(),
            time: "21:40".into(),
        }
    }

    fn core(dir: &tempfile::TempDir, files: &[(&str, &str)]) -> Core {
        for (name, text) in files {
            let path = dir.path().join("workspace").join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Core::new(dir.path(), Box::new(NoSecrets))
    }

    fn read(dir: &tempfile::TempDir, name: &str) -> String {
        std::fs::read_to_string(dir.path().join("workspace").join(name)).unwrap_or_default()
    }

    #[test]
    fn a_note_is_a_timed_line_with_its_origin_and_source() {
        let dir = tempfile::tempdir().unwrap();
        let core = core(&dir, &[]);
        let landed = core
            .add_note(&now(), "Rina visits next week.", Origin::Companion, "9-0")
            .unwrap();
        assert_eq!(landed.path, "memory/2026-11-14.md");
        assert_eq!(landed.line, 3);
        let second = core
            .add_note(&now(), "Second.", Origin::User, "9-1")
            .unwrap();
        assert_eq!(second.line, 4);
        assert_eq!(
            read(&dir, "memory/2026-11-14.md"),
            "# 2026-11-14\n\n- 21:40 Rina visits next week. <!-- from:companion src:9-0 -->\n\
             - 21:40 Second. <!-- from:user src:9-1 -->\n"
        );
    }

    #[test]
    fn an_entry_goes_to_the_end_of_its_topic_or_a_new_topic_at_the_end() {
        let file = "# Memory\n\n## Health\n- Old. <!-- id:m_1 -->\n\n## Work\n- Job.\n";
        let (next, line) = add_entry(file, "health", "- New.");
        assert_eq!(
            next,
            "# Memory\n\n## Health\n- Old. <!-- id:m_1 -->\n- New.\n\n## Work\n- Job.\n"
        );
        assert_eq!(line, 5);
        let (next, line) = add_entry("# Memory\n", "Pets", "- Bimo.");
        assert_eq!(next, "# Memory\n\n## Pets\n- Bimo.\n");
        assert_eq!(line, 4);
        // A hand-written file keeps every byte; the entry is added at the end.
        let hand = "My own notes\nwithout headings";
        let (next, _) = add_entry(hand, "Pets", "- Bimo.");
        assert!(next.starts_with(hand));
        assert!(next.ends_with("## Pets\n- Bimo.\n"));
    }

    #[test]
    fn a_profile_value_is_replaced_in_place_or_appended() {
        let file = "# About the user\n\n- Call them: Samuel\n- Preferred language: English\n";
        let (next, line) = set_profile(file, "call them", "Sam");
        assert_eq!(
            next,
            "# About the user\n\n- Call them: Sam\n- Preferred language: English\n"
        );
        assert_eq!(line, 3);
        let (next, line) = set_profile(file, "Lives in", "Bandung");
        assert!(next.ends_with("- Preferred language: English\n- Lives in: Bandung\n"));
        assert_eq!(line, 5);
    }

    #[test]
    fn entry_ids_are_unique_in_the_file() {
        let first = entry_id("", "seed");
        assert!(first.starts_with("m_") && first.len() == 6);
        let taken = format!("- a <!-- id:{first} -->");
        let second = entry_id(&taken, "seed");
        assert_ne!(first, second);
    }

    #[test]
    fn a_full_file_keeps_the_fact_in_todays_note() {
        let dir = tempfile::tempdir().unwrap();
        let full_memory = format!("# Memory\n\n## A\n- {}\n", "x".repeat(MEMORY_BUDGET - 20));
        let full_profile = format!("- Notes: {}\n", "y".repeat(PROFILE_BUDGET - 15));
        let core = core(
            &dir,
            &[("MEMORY.md", &full_memory), ("USER.md", &full_profile)],
        );
        let landed = core
            .remember(
                &now(),
                "Allergic to peanuts.",
                "Health",
                Origin::User,
                "1-0",
            )
            .unwrap();
        assert!(landed.kept_in_note);
        assert_eq!(landed.path, "memory/2026-11-14.md");
        let landed = core
            .update_profile(&now(), "Lives in", "Bandung", "1-0")
            .unwrap();
        assert!(landed.kept_in_note);
        assert_eq!(read(&dir, "MEMORY.md"), full_memory);
        assert_eq!(read(&dir, "USER.md"), full_profile);
        let note = read(&dir, "memory/2026-11-14.md");
        assert!(note.contains("Allergic to peanuts. <!-- from:user"));
        assert!(note.contains("Lives in: Bandung"));
    }
}
