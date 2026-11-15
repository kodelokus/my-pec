//! What every bundled extension shares: the trait, its confined folder, whether it is on, its tools in a
//! conversation and the shell's tick (extensions R1 to R4), and the one path to the network an extension
//! may have (weather R1). The core never names a concrete extension.

use crate::llm::ToolSpec;
use crate::memory::MemoryMark;
use crate::memory::tools::{ToolOutcome, failed};
use crate::{AppError, Core, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use time::{OffsetDateTime, PrimitiveDateTime};
use ts_rs::TS;

pub use crate::workspace::write_atomic;

/// A bundled extension. Its rules live in `pec-ext`; it reaches only its own place through `ExtDir`.
pub trait Extension: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn about(&self) -> &'static str;
    /// Its SKILL.md: how the companion uses it, written after the harness of each context that records it.
    fn skill(&self) -> &'static str;
    /// Its place in the workspace: a folder ending in `/`, or one file.
    fn place(&self) -> &'static str;
    fn tools(&self) -> Vec<ToolSpec>;
    /// Runs one of its tools at the person's local time. Never fails: a problem is an `is_error` answer.
    fn run(&self, tool: &str, input: &Value, dir: &ExtDir, now: PrimitiveDateTime) -> ToolAnswer;
    /// The shell's tick, every 15 seconds while the extension is on (extensions R4).
    fn tick(&self, now: OffsetDateTime, dir: &ExtDir) -> Tick;
    /// Whether it is on before the person has chosen. One that goes online starts off (Principle II).
    fn default_on(&self) -> bool {
        true
    }
    /// The hosts its spec names, the only ones `ExtDir::fetch` reaches for it (weather R1).
    fn hosts(&self) -> &'static [&'static str] {
        &[]
    }
    /// A line for the `<context>` block of each new message, in contexts that recorded it (weather R5).
    fn context_line(&self, _dir: &ExtDir, _now: OffsetDateTime) -> Option<String> {
        None
    }
}

/// Why a request got no answer. Either way the extension carries on without it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchError {
    /// Not a host the extension named, or the extension is off: nothing was sent.
    Refused,
    /// Offline, a timeout, an error status, or an answer that isn't small JSON.
    Failed,
}

/// One GET answering JSON: the core's only path to the network for extensions. Tests give the core a fake, as
/// they do the LLM (weather R1).
pub trait Fetch: Send + Sync {
    fn get(&self, url: &str, query: &[(&str, String)]) -> std::result::Result<Value, FetchError>;
}

/// The real client: 5 seconds at most, answers of at most 64 KB. Blocking, so it is used only off the async
/// runtime, on the tick thread or a blocking command thread.
pub struct Http(reqwest::blocking::Client);

const FETCH_LIMIT: usize = 64 * 1024;

impl Default for Http {
    fn default() -> Self {
        Self(
            reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("the HTTP client builds"),
        )
    }
}

impl Fetch for Http {
    fn get(&self, url: &str, query: &[(&str, String)]) -> std::result::Result<Value, FetchError> {
        let answer = self
            .0
            .get(url)
            .query(query)
            .send()
            .map_err(|_| FetchError::Failed)?;
        if !answer.status().is_success() {
            return Err(FetchError::Failed);
        }
        let bytes = answer.bytes().map_err(|_| FetchError::Failed)?;
        if bytes.len() > FETCH_LIMIT {
            return Err(FetchError::Failed);
        }
        serde_json::from_slice(&bytes).map_err(|_| FetchError::Failed)
    }
}

/// A tool call's `tool_result` and the mark it leaves on the reply (extensions R3).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolAnswer {
    pub content: String,
    pub is_error: bool,
    pub mark: Option<MemoryMark>,
}

impl ToolAnswer {
    pub fn ok(content: impl Into<String>, mark: Option<MemoryMark>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
            mark,
        }
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
            mark: None,
        }
    }
}

/// Something due: an alarm to ring or one that was missed.
#[derive(Debug, Clone, PartialEq)]
pub struct Due {
    pub key: String,
    pub label: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    /// `HH:MM`.
    pub time: String,
}

/// What one extension's tick found.
#[derive(Debug, Default, PartialEq)]
pub struct Tick {
    pub ring: Vec<Due>,
    pub missed: Vec<Due>,
    /// Its files changed since the previous tick (extensions R9).
    pub changed: bool,
}

/// What the shell shows after a tick (extensions interfaces "The shell's tick").
#[derive(Debug, Default, PartialEq)]
pub struct ExtTick {
    pub ring: Vec<Due>,
    pub missed: Vec<Due>,
    pub changed: Vec<String>,
}

/// An extension as settings lists it (extensions FR-004).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub about: String,
    /// Where its files are, for the person to find them.
    pub folder: String,
    pub on: bool,
}

/// Launch at login: whether it is on, read from Windows by the shell, and whether the person was asked (extensions
/// FR-026, extensions FR-027).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Autostart {
    pub on: bool,
    pub offered: bool,
}

/// `extensions.<id>` in `settings.json`: on unless the person turned it off (extensions FR-001).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExtensionSetting {
    pub on: bool,
}

/// An extension's own place in the workspace and its own bookkeeping file, `state/<id>.json`, and nothing else
/// (extensions FR-003, extensions R1).
pub struct ExtDir<'a> {
    place: PathBuf,
    folder: bool,
    state: PathBuf,
    lock: &'a Mutex<()>,
    on: bool,
    hosts: &'static [&'static str],
    fetch: &'a OnceLock<Box<dyn Fetch>>,
}

impl ExtDir<'_> {
    /// The place itself: the folder, or the one file.
    pub fn place(&self) -> &Path {
        &self.place
    }

    /// A path inside a folder place: plain names only, no `..`, no absolute path and no link.
    pub fn file(&self, rel: &str) -> Result<PathBuf> {
        let plain = Path::new(rel)
            .components()
            .all(|c| matches!(c, Component::Normal(_)));
        if !self.folder || rel.is_empty() || !plain {
            return Err(AppError::invalid("path"));
        }
        let path = self.place.join(rel);
        if path.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
            return Err(AppError::invalid("path"));
        }
        Ok(path)
    }

    /// The bookkeeping, or its default when it is missing or damaged (extensions data 2: safe to lose).
    pub fn read_state<T: DeserializeOwned + Default>(&self) -> T {
        std::fs::read(&self.state)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn write_state<T: Serialize>(&self, value: &T) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(value).expect("state serializes");
        write_atomic(&self.state, &bytes)
    }

    /// A GET to `https://<host><path>`, only to a host the extension names and only while it is on (Principle II).
    /// The path is plain: letters, digits, `/`, `-`, `_` and `.`.
    pub fn fetch(
        &self,
        host: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> std::result::Result<Value, FetchError> {
        let plain = path.starts_with('/')
            && path
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "/-_.".contains(c));
        if !self.on || !self.hosts.contains(&host) || !plain {
            return Err(FetchError::Refused);
        }
        let fetch = self.fetch.get_or_init(|| Box::new(Http::default()));
        fetch.get(&format!("https://{host}{path}"), query)
    }

    /// The writer's lock: files are read, changed and written under it (extensions FR-008).
    pub fn lock(&self) -> MutexGuard<'_, ()> {
        self.lock.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// The person's local time from a chat clock's date and `HH:MM`.
pub(crate) fn local_time(date: &str, time: &str) -> Option<PrimitiveDateTime> {
    let number = |s: &str| s.parse::<u8>().ok();
    let (y, m, d) = (
        date.get(..4)?.parse().ok()?,
        number(date.get(5..7)?)?,
        number(date.get(8..10)?)?,
    );
    let day = time::Date::from_calendar_date(y, m.try_into().ok()?, d).ok()?;
    let at = time::Time::from_hms(number(time.get(..2)?)?, number(time.get(3..5)?)?, 0).ok()?;
    Some(PrimitiveDateTime::new(day, at))
}

/// The extensions and the lock their files are written under.
#[derive(Default)]
pub(crate) struct ExtRuntime {
    all: Vec<Box<dyn Extension>>,
    write: Mutex<()>,
    /// Made on first use, off the async runtime; tests set a fake with `with_fetch`.
    fetch: OnceLock<Box<dyn Fetch>>,
}

impl Core {
    /// The bundled extensions, as the shell builds them with `pec_ext::all()`.
    pub fn with_extensions(mut self, all: Vec<Box<dyn Extension>>) -> Self {
        self.ext.all = all;
        self
    }

    fn find(&self, id: &str) -> Result<&dyn Extension> {
        self.ext
            .all
            .iter()
            .find(|e| e.id() == id)
            .map(AsRef::as_ref)
            .ok_or_else(|| AppError::invalid("id"))
    }

    /// Extensions' requests go through `fetch` instead of the network: the shell's end-to-end builds and tests.
    pub fn with_fetch(self, fetch: Box<dyn Fetch>) -> Self {
        let _ = self.ext.fetch.set(fetch);
        self
    }

    pub fn extension_on(&self, id: &str) -> bool {
        let default = self.find(id).is_ok_and(|e| e.default_on());
        self.settings()
            .map(|s| s.extensions.get(id).map_or(default, |e| e.on))
            .unwrap_or(default)
    }

    pub fn extensions(&self) -> Result<Vec<ExtensionInfo>> {
        let workspace = self.workspace_dir();
        Ok(self
            .ext
            .all
            .iter()
            .map(|e| ExtensionInfo {
                id: e.id().into(),
                name: e.name().into(),
                about: e.about().into(),
                folder: workspace
                    .join(e.place().trim_end_matches('/'))
                    .display()
                    .to_string(),
                on: self.extension_on(e.id()),
            })
            .collect())
    }

    /// extensions FR-004: on or off at once; its files are never touched.
    pub fn set_extension(&self, id: &str, on: bool) -> Result<Vec<ExtensionInfo>> {
        self.find(id)?;
        let mut settings = self.settings()?;
        settings
            .extensions
            .insert(id.into(), ExtensionSetting { on });
        self.save_settings(&settings)?;
        self.extensions()
    }

    /// The extension's confined place, for the shell's commands on it.
    pub fn ext_dir(&self, id: &str) -> Result<ExtDir<'_>> {
        let ext = self.find(id)?;
        Ok(ExtDir {
            place: self.workspace_dir().join(ext.place().trim_end_matches('/')),
            folder: ext.place().ends_with('/'),
            state: self.state_dir().join(format!("{}.json", ext.id())),
            lock: &self.ext.write,
            on: self.extension_on(ext.id()),
            hosts: ext.hosts(),
            fetch: &self.ext.fetch,
        })
    }

    /// Asks each extension that is on what is due and what changed. Takes no conversation lock (Principle V).
    pub fn ext_tick(&self, now: OffsetDateTime) -> ExtTick {
        let mut out = ExtTick::default();
        for ext in self.ext.all.iter().filter(|e| self.extension_on(e.id())) {
            let Ok(dir) = self.ext_dir(ext.id()) else {
                continue;
            };
            let tick = ext.tick(now, &dir);
            out.ring.extend(tick.ring);
            out.missed.extend(tick.missed);
            if tick.changed {
                out.changed.push(ext.id().into());
            }
        }
        out
    }

    /// extensions FR-027: the launch-at-login question is asked once; `on` is what Windows says.
    pub fn autostart(&self, on: bool) -> Result<Autostart> {
        Ok(Autostart {
            on,
            offered: self.state()?.autostart_offered,
        })
    }

    pub fn mark_autostart_offered(&self, on: bool) -> Result<Autostart> {
        self.update_state(|s| s.autostart_offered = true)?;
        self.autostart(on)
    }

    /// The ids of the extensions that are on, recorded by a new context (extensions R2).
    pub(crate) fn extensions_on(&self) -> Vec<String> {
        self.ext
            .all
            .iter()
            .filter(|e| self.extension_on(e.id()))
            .map(|e| e.id().into())
            .collect()
    }

    /// The tools of the recorded extensions, in the recorded order.
    pub(crate) fn ext_specs(&self, ids: &[String]) -> Vec<ToolSpec> {
        ids.iter()
            .filter_map(|id| self.find(id).ok())
            .flat_map(|e| e.tools())
            .collect()
    }

    /// The SKILL.md of each recorded extension, in the recorded order, or a line saying none is on.
    pub(crate) fn ext_skills(&self, ids: &[String]) -> String {
        let skills: Vec<&str> = ids
            .iter()
            .filter_map(|id| self.find(id).ok())
            .map(|e| e.skill().trim_end())
            .collect();
        if skills.is_empty() {
            "No extension is turned on.\n".into()
        } else {
            skills.join("\n\n") + "\n"
        }
    }

    /// What a new context records about extensions: the SKILL.md of each one that is on, then a short entry for
    /// each one that is off, so the companion can say where to turn it on.
    pub(crate) fn ext_section(&self) -> String {
        let (on, off): (Vec<_>, Vec<_>) =
            self.ext.all.iter().partition(|e| self.extension_on(e.id()));
        let mut parts: Vec<String> = on.iter().map(|e| e.skill().trim_end().to_owned()).collect();
        parts.extend(off.iter().map(|e| {
            format!(
                "## {} (turned off)\n\n{} It is turned off, so you have no tools for it. If the person asks for \
                 it, tell them they can turn it on in Settings, under Extensions.",
                e.name(),
                e.about()
            )
        }));
        if on.is_empty() {
            parts.insert(0, "No extension is turned on.".into());
        }
        parts.join("\n\n") + "\n"
    }

    /// The lines the recorded extensions that are still on add to a new message's `<context>` block.
    pub(crate) fn ext_context_lines(
        &self,
        recorded: &[String],
        now: OffsetDateTime,
    ) -> Vec<String> {
        recorded
            .iter()
            .filter(|id| self.extension_on(id))
            .filter_map(|id| {
                let ext = self.find(id).ok()?;
                ext.context_line(&self.ext_dir(id).ok()?, now)
            })
            .collect()
    }

    /// Whether an extension that is on now isn't in the current context, which gets it only in a fresh one
    /// (Principle III).
    pub(crate) fn extensions_pending(&self, recorded: &[String]) -> bool {
        self.extensions_on().iter().any(|id| !recorded.contains(id))
    }

    /// Runs an extension's tool, or `None` when no extension has a tool of that name. A tool of an extension that
    /// is now off is refused, and nothing changes (extensions FR-007).
    pub(crate) fn run_ext_tool(
        &self,
        name: &str,
        input: &Value,
        now: Option<PrimitiveDateTime>,
    ) -> Option<ToolOutcome> {
        let ext = self
            .ext
            .all
            .iter()
            .find(|e| e.tools().iter().any(|t| t.name == name))?;
        if !self.extension_on(ext.id()) {
            return Some(failed(format!(
                "{} is turned off. The person can turn it on in Settings, under Extensions.",
                ext.name()
            )));
        }
        let (Some(now), Ok(dir)) = (now, self.ext_dir(ext.id())) else {
            return Some(failed("The person's clock is unavailable right now."));
        };
        let answer = ext.run(name, input, &dir, now);
        Some(ToolOutcome {
            content: answer.content,
            is_error: answer.is_error,
            mark: answer.mark,
            written: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ext_dir_opens_only_its_own_place_and_state() {
        let root = tempfile::tempdir().unwrap();
        let lock = Mutex::new(());
        let fetch = OnceLock::new();
        let notes = ExtDir {
            place: root.path().join("workspace/notes"),
            folder: true,
            state: root.path().join("state/notes.json"),
            lock: &lock,
            on: true,
            hosts: &[],
            fetch: &fetch,
        };
        assert!(notes.file("a-k7q2mx.md").is_ok());
        assert!(notes.file("deleted/a-k7q2mx.md").is_ok());
        for bad in [
            "",
            "../alarms.md",
            "..",
            "/etc/x.md",
            "C:\\x.md",
            "./x/../../y",
        ] {
            assert_eq!(notes.file(bad), Err(AppError::invalid("path")), "{bad}");
        }
        let alarms = ExtDir {
            place: root.path().join("workspace/alarms.md"),
            folder: false,
            ..notes
        };
        assert_eq!(alarms.file("x.md"), Err(AppError::invalid("path")));
        // The bookkeeping is the extension's own file, and its default when lost or damaged.
        alarms.write_state(&vec![1, 2]).unwrap();
        assert_eq!(alarms.read_state::<Vec<i32>>(), vec![1, 2]);
        std::fs::write(root.path().join("state/notes.json"), "{damaged").unwrap();
        assert_eq!(alarms.read_state::<Vec<i32>>(), Vec::<i32>::new());
    }

    /// Answers every request with the URL and query it was asked for.
    struct Echo;

    impl Fetch for Echo {
        fn get(
            &self,
            url: &str,
            query: &[(&str, String)],
        ) -> std::result::Result<Value, FetchError> {
            Ok(serde_json::json!({ "url": url, "query": query }))
        }
    }

    // weather FR-008, FR-009, R1.
    #[test]
    fn an_ext_dir_reaches_only_its_own_hosts_over_https_while_on() {
        let root = tempfile::tempdir().unwrap();
        let (lock, fetch) = (Mutex::new(()), OnceLock::new());
        let _ = fetch.set(Box::new(Echo) as Box<dyn Fetch>);
        let dir = ExtDir {
            place: root.path().join("workspace/weather.md"),
            folder: false,
            state: root.path().join("state/weather.json"),
            lock: &lock,
            on: true,
            hosts: &["api.example.com"],
            fetch: &fetch,
        };
        assert_eq!(
            dir.fetch("api.example.com", "/v1/now", &[("lat", "1.5".into())]),
            Ok(
                serde_json::json!({ "url": "https://api.example.com/v1/now", "query": [["lat", "1.5"]] })
            )
        );
        for (host, path) in [
            ("other.example.com", "/v1/now"),
            ("api.example.com", "@evil.example.com/x"),
            ("api.example.com", "/x?y=1"),
            ("api.example.com", "//evil.example.com"),
        ] {
            let refused = dir.fetch(host, path, &[]);
            if path == "//evil.example.com" {
                // A plain path still goes only to the named host.
                assert_eq!(
                    refused.unwrap()["url"],
                    "https://api.example.com//evil.example.com"
                );
            } else {
                assert_eq!(refused, Err(FetchError::Refused), "{host}{path}");
            }
        }
        let off = ExtDir { on: false, ..dir };
        assert_eq!(
            off.fetch("api.example.com", "/v1/now", &[]),
            Err(FetchError::Refused)
        );
    }

    #[test]
    fn extension_settings_default_on_and_keep_unknown_ids() {
        let text = r#"{"extensions":{"notes":{"on":false},"later":{"on":true}}}"#;
        let settings: crate::config::Settings = serde_json::from_str(text).unwrap();
        assert!(!settings.extensions["notes"].on);
        assert!(!settings.extensions.contains_key("alarms"));
        let back = serde_json::to_string(&settings).unwrap();
        assert!(back.contains(r#""later":{"on":true}"#));
    }

    #[test]
    fn a_chat_clock_reads_as_local_time() {
        let at = local_time("2026-11-14", "19:05").unwrap();
        assert_eq!(at.to_string(), "2026-11-14 19:05:00.0");
        assert_eq!(local_time("2026-13-01", "19:05"), None);
        assert_eq!(local_time("2026-11-14", "25:00"), None);
    }
}
