//! The one conversation with the companion: commands, the reply loop, retries (chat interfaces).

pub(crate) mod context;
pub(crate) mod mood;
pub(crate) mod reply;
pub(crate) mod transcript;
mod view;

pub use context::LocalClock;
pub use transcript::ReplyState;
pub use view::{ChatMessage, ChatPage, MessageKind};

use crate::avatar::{Mood, Turn};
use crate::compaction::RESERVE_TOKENS;
use crate::config::now;
use crate::llm::{Effort, LlmProvider, ModelInfo, ProviderError, TurnRequest, effort_for};
use crate::memory::MemoryMark;
use crate::memory::files::MEMORY_BUDGET;
use crate::memory::search::{RECALL_HITS, select_recall};
use crate::onboarding::field;
use crate::{AppError, Core, Result};
use context::{replaced, text_block};
use serde::Serialize;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::Notify;
use transcript::{Entry, Line, Provenance, Role, Transcript};
use ts_rs::TS;
use view::{turn_view, visible};

pub const MAX_MESSAGE_CHARS: usize = 32_000;
/// chat FR-025: how long a busy provider is retried before the user is asked.
const RETRY_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum RetryReason {
    /// The app ended while the reply was being written (chat FR-017).
    Interrupted,
    Failed {
        error: ProviderError,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChatState {
    pub replying: bool,
    pub can_regenerate: bool,
    pub pending_retry: Option<RetryReason>,
    pub first_hello_pending: bool,
    /// The context reached its working size at a turn boundary: the webview asks for a compaction (compaction R2).
    pub compaction_due: bool,
    /// The provider still says the conversation is too long: offer retry or a fresh context (compaction FR-022).
    pub context_full: bool,
    pub model_change_pending: bool,
    pub shortened: Vec<String>,
    pub skipped_lines: u32,
    /// The current context was started before memory existed and has no memory tools (memory R2).
    pub context_outdated: bool,
    /// The current context began under older rules, or without an extension that is on now; the next one gets
    /// them (Principle III), and the chat offers to start it.
    pub update_pending: bool,
    /// Long-term memory is at its size budget (memory FR-011).
    pub memory_full: bool,
    /// Relevant memory is attached to each message (memory FR-020).
    pub auto_recall: bool,
    /// A dream sweep is running (dreaming FR-025).
    pub dreaming: bool,
    /// The webview chimes when a reply is done.
    pub sound: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum ChatEvent {
    Started {
        #[serde(rename = "userMessage")]
        user_message: Option<ChatMessage>,
    },
    Delta {
        text: String,
    },
    /// The reply's mood marker was read; it is never in the text (avatar FR-008).
    Mood {
        mood: Mood,
    },
    /// The provider is busy; text streamed since `text` was replaced and a retry follows (chat FR-025).
    Waiting {
        seconds: u32,
        /// The turn's text from calls that finished before this one.
        text: String,
    },
    /// The provider said the conversation is too long; it is being compacted before one retry (compaction R10).
    Compacting,
    /// A memory tool wrote something; the turn continues (memory FR-008).
    MemoryWritten {
        mark: MemoryMark,
    },
    Finished {
        message: Option<ChatMessage>,
        state: ChatState,
    },
    Failed {
        error: ProviderError,
        partial: Option<ChatMessage>,
        state: ChatState,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub enum SendRefusal {
    Busy,
    Empty,
    TooLong,
    NoKey,
}

/// Where a reply's events go: a Tauri channel in the app, a vector in tests.
pub trait ChatSink: Send + Sync {
    fn send(&self, event: ChatEvent);
}

/// The running reply and its stop signal, one at a time (chat R4); the running compaction and the signal
/// that asks it to stop (compaction R3).
#[derive(Default)]
pub(crate) struct ChatRuntime {
    transcript: Mutex<Option<Transcript>>,
    pub(crate) active: Mutex<Option<Arc<Notify>>>,
    pub(crate) compacting: Mutex<Option<Arc<Notify>>>,
    /// The running dream sweep and the signal that pauses it (dreaming R4).
    pub(crate) dreaming: Mutex<Option<Arc<Notify>>>,
    pub(crate) last_failure: Mutex<Option<ProviderError>>,
}

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// Clears the running-reply slot however the reply ends.
struct Active<'a> {
    core: &'a Core,
    stop: Arc<Notify>,
}

impl Drop for Active<'_> {
    fn drop(&mut self) {
        *lock(&self.core.chat.active) = None;
    }
}

/// Links open in the default browser only for these schemes (chat R12).
pub fn is_openable_link(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    ["http://", "https://", "mailto:"]
        .iter()
        .any(|scheme| lower.starts_with(scheme) && lower.len() > scheme.len())
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// The message that opened the current turn: the last one the user typed, or the hidden hello request.
fn turn_start(lines: &[Line]) -> Option<usize> {
    let gone = replaced(lines);
    lines.iter().rposition(|l| {
        matches!(
            l.entry,
            Entry::Message {
                role: Role::User,
                provenance: Provenance::User | Provenance::System,
                ..
            }
        ) && !gone.contains(l.id.as_str())
    })
}

/// The last line that decides whether a reply is owed: a user message, a reply, or a fresh context. A context a
/// compaction opened carries its kept turns over, so it doesn't count (compaction R8).
fn last_turn(lines: &[Line]) -> Option<&Line> {
    let gone = replaced(lines);
    lines.iter().rev().find(|l| match &l.entry {
        Entry::Message { .. } => !gone.contains(l.id.as_str()),
        Entry::Context { compaction, .. } => compaction.is_none(),
        _ => false,
    })
}

/// Whether the conversation owes a reply: its last turn wasn't answered, was cut off, or stopped between a tool
/// call and its result.
pub(crate) fn owed(lines: &[Line]) -> bool {
    match last_turn(lines).map(|l| &l.entry) {
        Some(Entry::Message {
            role: Role::User, ..
        }) => true,
        Some(Entry::Message {
            state, stop_reason, ..
        }) => *state == Some(ReplyState::Incomplete) || stop_reason.as_deref() == Some("tool_use"),
        _ => false,
    }
}

/// The current context's size: what the provider reported for its latest reply (chat R10).
pub(crate) fn context_size(lines: &[Line]) -> u64 {
    context::current(lines)
        .iter()
        .rev()
        .find_map(|l| match &l.entry {
            Entry::Message {
                usage: Some(usage), ..
            } => Some(reply::context_tokens(usage)),
            _ => None,
        })
        .unwrap_or(0)
}

impl Core {
    pub(crate) fn transcript<T>(&self, f: impl FnOnce(&mut Transcript) -> Result<T>) -> Result<T> {
        let mut slot = lock(&self.chat.transcript);
        if slot.is_none() {
            let path = self.workspace_dir().join("sessions").join("main.jsonl");
            let opened = Transcript::open(path)?;
            // A crash between a compaction's commit and its notes is finished now; it never blocks chat (compaction R5).
            let _ = self.finish_pending(&opened.lines);
            *slot = Some(opened);
        }
        f(slot.as_mut().expect("opened above"))
    }

    pub(crate) fn append(&self, t: &mut Transcript, entry: Entry) -> Result<Line> {
        let at = now_ms();
        let line = Line {
            id: t.next_id(at),
            at,
            entry,
        };
        t.append(line.clone())?;
        Ok(line)
    }

    fn begin(&self) -> Option<Active<'_>> {
        let mut slot = lock(&self.chat.active);
        if slot.is_some() {
            return None;
        }
        let stop = Arc::new(Notify::new());
        *slot = Some(stop.clone());
        // Chat comes first: a running compaction stops at its next step (compaction R3).
        if let Some(compaction) = lock(&self.chat.compacting).as_ref() {
            compaction.notify_one();
        }
        if let Some(dream) = lock(&self.chat.dreaming).as_ref() {
            dream.notify_one();
        }
        Some(Active { core: self, stop })
    }

    /// Starts the first context if none exists (chat FR-031).
    fn ensure_context(&self, t: &mut Transcript, clock: &LocalClock) -> Result<()> {
        if !t
            .lines
            .iter()
            .any(|l| matches!(l.entry, Entry::Context { .. }))
        {
            self.open_context(t, clock)?;
        }
        Ok(())
    }

    /// The time block of a new message, with the lines of the extensions the current context recorded, such as the
    /// weather (weather R5). Stored with the message, so a rebuilt request is the same (Principle III).
    fn with_ext_lines(&self, time: &serde_json::Value, lines: &[Line]) -> serde_json::Value {
        let recorded = match context::current(lines).first().map(|l| &l.entry) {
            Some(Entry::Context { extensions, .. }) => extensions.as_slice(),
            _ => &[],
        };
        let extra = self.ext_context_lines(recorded, time::OffsetDateTime::now_utc());
        match time["text"].as_str() {
            Some(text) if !extra.is_empty() => {
                let body = text.strip_suffix("</context>").unwrap_or(text);
                text_block(&format!("{body}\n{}</context>", extra.join("\n")))
            }
            _ => time.clone(),
        }
    }

    /// A context starts from the files, and today's and yesterday's notes by the user's clock (memory FR-014).
    fn open_context(&self, t: &mut Transcript, clock: &LocalClock) -> Result<()> {
        let entry = self.context_entry(clock, None, &[], None)?;
        self.append(t, entry).map(drop)
    }

    /// Catching up when there is nothing to compact: a new context with today's rules and extensions that replays
    /// every turn the current one holds as text, like the turns a compaction keeps.
    pub fn start_context_carrying(&self, clock: &LocalClock) -> Result<ChatState> {
        context::time_block(clock)?;
        let Some(_active) = self.begin() else {
            return Err(AppError::invalid("busy"));
        };
        self.transcript(|t| {
            let first = context::held(&t.lines).0.first().map(|l| l.id.clone());
            let mut entry = self.context_entry(clock, None, &[], None)?;
            if let Entry::Context { carried, .. } = &mut entry {
                *carried = first;
            }
            self.append(t, entry).map(drop)
        })?;
        *lock(&self.chat.last_failure) = None;
        self.state_with(false)
    }

    /// A `context` entry snapshotting the files and notes now; `compaction` names the compaction that opens it,
    /// and `staged` are note lines it adds to today's note, already in the snapshot (compaction R5).
    pub(crate) fn context_entry(
        &self,
        clock: &LocalClock,
        compaction: Option<String>,
        staged: &[String],
        dream: Option<&str>,
    ) -> Result<Entry> {
        let workspace = self.workspace_dir();
        let today = context::local_now(clock).date;
        Ok(Entry::Context {
            model: self.settings()?.chat.model,
            files: context::read_files(&workspace),
            harness: context::HARNESS_VERSION.into(),
            notes: Some(context::read_notes(&workspace, &today, staged)),
            compaction,
            dream: dream.map(Into::into),
            world: self.world_snapshot(),
            extensions: self.extensions_on(),
            skills: Some(self.ext_section()),
            carried: None,
        })
    }

    pub fn chat_state(&self) -> Result<ChatState> {
        let replying = lock(&self.chat.active).is_some();
        self.state_with(replying)
    }

    pub(crate) fn state_with(&self, replying: bool) -> Result<ChatState> {
        let settings = self.settings()?;
        let first_hello_pending = self.state()?.first_hello_pending;
        let last_failure = *lock(&self.chat.last_failure);
        let compacting = lock(&self.chat.compacting).is_some();
        self.transcript(|t| {
            let last = last_turn(&t.lines);
            let owed = owed(&t.lines);
            let reply_last = matches!(
                last.map(|l| &l.entry),
                Some(Entry::Message {
                    role: Role::Assistant,
                    stop_reason,
                    ..
                }) if stop_reason.as_deref() != Some("tool_use")
            );
            let current = context::current(&t.lines);
            let used = context_size(&t.lines);
            let (context_model, harness, recorded) = match current.first().map(|l| &l.entry) {
                Some(Entry::Context {
                    model,
                    harness,
                    extensions,
                    ..
                }) => (Some(model.as_str()), harness.as_str(), Some(extensions)),
                _ => (None, context::HARNESS_VERSION, None),
            };
            let memory_full = std::fs::read_to_string(self.workspace_dir().join("MEMORY.md"))
                .is_ok_and(|m| m.chars().count() >= MEMORY_BUDGET);
            Ok(ChatState {
                replying,
                can_regenerate: !replying && reply_last,
                pending_retry: (!replying && owed).then_some(match last_failure {
                    Some(error) => RetryReason::Failed { error },
                    None => RetryReason::Interrupted,
                }),
                first_hello_pending,
                compaction_due: !replying
                    && !compacting
                    && !owed
                    && used > 0
                    && used >= settings.context.budget() - RESERVE_TOKENS,
                context_full: last_failure == Some(ProviderError::ContextTooLong),
                model_change_pending: context_model.is_some_and(|m| m != settings.chat.model),
                shortened: [
                    context::shortened(&context::seen(&t.lines)),
                    context::shortened_notes(&t.lines),
                ]
                .concat(),
                skipped_lines: t.skipped as u32,
                // Only a context from before memory lacks the tools; later versions carry on unchanged.
                context_outdated: harness == "chat-1",
                update_pending: harness != "chat-1"
                    && recorded.is_some_and(|r| {
                        harness != context::HARNESS_VERSION || self.extensions_pending(r)
                    }),
                memory_full,
                auto_recall: settings.recall.auto,
                dreaming: lock(&self.chat.dreaming).is_some(),
                sound: settings.chat.sound,
            })
        })
    }

    /// chat FR-016: pages of visible messages, oldest first within a page.
    pub fn chat_history(&self, before: Option<&str>, limit: u32) -> Result<ChatPage> {
        if !(1..=200).contains(&limit) {
            return Err(AppError::invalid("limit"));
        }
        self.transcript(|t| {
            let messages = visible(&t.lines);
            let end = match before {
                Some(id) => messages
                    .iter()
                    .position(|m| m.id == id)
                    .ok_or_else(|| AppError::invalid("before"))?,
                None => messages.len(),
            };
            let start = end.saturating_sub(limit as usize);
            Ok(ChatPage {
                messages: messages[start..end].to_vec(),
                has_more: start > 0,
            })
        })
    }

    /// chat FR-002, FR-013: the user message is on disk before the request is sent.
    pub async fn send_message(
        &self,
        provider: &impl LlmProvider,
        text: &str,
        clock: &LocalClock,
        sink: &impl ChatSink,
    ) -> Result<Option<SendRefusal>> {
        if text.trim().is_empty() {
            return Ok(Some(SendRefusal::Empty));
        }
        if text.chars().count() > MAX_MESSAGE_CHARS {
            return Ok(Some(SendRefusal::TooLong));
        }
        let time = context::time_block(clock)?;
        self.start(provider, clock, sink, |t| {
            // chat FR-021: speaking first cancels the greeting.
            if self.state()?.first_hello_pending {
                self.update_state(|s| s.first_hello_pending = false)?;
            }
            self.ensure_context(t, clock)?;
            let files = context::read_files(&self.workspace_dir());
            let changed = context::changed(&context::seen(&t.lines), &files);
            let at = now_ms();
            let id = t.next_id(at);
            let mut content = vec![self.with_ext_lines(&time, &t.lines)];
            // memory FR-019, FR-021: relevant memory goes between the time and the text, marked as recalled. A failing
            // index means no recall, never a failed message (memory FR-024).
            let recalled = if self.settings()?.recall.auto {
                self.recall(&t.lines, text, clock).ok().flatten()
            } else {
                None
            };
            let recalled = recalled.map(|block| {
                content.push(block);
                content.len() - 1
            });
            // Journey R10: what was drawn overnight, told once, appended with this message.
            let world = self.world_notice(&t.lines)?.map(|(notice, through)| {
                content.push(text_block(&notice));
                through
            });
            for (file, text) in changed {
                content.push(context::notice_block(&file, &text));
                let entry = Entry::Notice {
                    file,
                    content: text,
                    message: id.clone(),
                };
                self.append(t, entry)?;
            }
            content.push(text_block(text));
            let line = Line {
                id,
                at,
                entry: Entry::Message {
                    role: Role::User,
                    provenance: Provenance::User,
                    content,
                    text: Some(text.into()),
                    zone: Some(clock.zone.clone()),
                    state: None,
                    stop_reason: None,
                    model: None,
                    usage: None,
                    recalled,
                    world,
                    marks: Vec::new(),
                },
            };
            t.append(line)?;
            Ok(visible(&t.lines).pop())
        })
        .await
    }

    /// The recall block for a message: the best hits not already in the context, if any qualify (memory R11).
    fn recall(
        &self,
        lines: &[Line],
        text: &str,
        clock: &LocalClock,
    ) -> Result<Option<serde_json::Value>> {
        let today = context::local_now(clock).date;
        let (hits, terms) = self.search_memory(text, Some(&today), 20, false)?;
        // Dreaming R1: the relevant daily-note hits are evidence for dreaming, shown or not.
        let relevant: Vec<_> = hits
            .iter()
            .filter(|h| h.relevant(terms))
            .take(RECALL_HITS)
            .cloned()
            .collect();
        let _ = self.record_recalls(text, &relevant, &today, "auto");
        let chosen = select_recall(hits, terms, &context::loaded(lines));
        Ok(context::recall_block(&chosen))
    }

    /// chat FR-017, FR-027: asks again for the reply the conversation is owed.
    pub async fn retry_reply(
        &self,
        provider: &impl LlmProvider,
        clock: &LocalClock,
        sink: &impl ChatSink,
    ) -> Result<Option<SendRefusal>> {
        context::time_block(clock)?;
        self.start(provider, clock, sink, |t| {
            let entry = match last_turn(&t.lines).map(|l| (&l.id, &l.entry)) {
                Some((
                    _,
                    Entry::Message {
                        role: Role::User, ..
                    },
                )) => None,
                Some((
                    id,
                    Entry::Message {
                        state: Some(ReplyState::Incomplete),
                        ..
                    },
                )) => Some(Entry::Replaced { target: id.clone() }),
                // The app ended between a tool call and its result: say so rather than run it twice (memory R3).
                Some((
                    _,
                    Entry::Message {
                        role: Role::Assistant,
                        stop_reason: Some(stop),
                        content,
                        ..
                    },
                )) if stop == "tool_use" => {
                    let calls = context::tool_calls(content);
                    let interrupted = "This tool was interrupted before its result was saved. It may or may not have run.";
                    Some(Entry::Message {
                        role: Role::User,
                        provenance: Provenance::Untrusted,
                        content: context::tool_results(
                            calls.iter().map(|(id, ..)| (id.as_str(), interrupted, true)),
                        ),
                        text: None,
                        zone: None,
                        state: None,
                        stop_reason: None,
                        model: None,
                        usage: None,
                        recalled: None,
                        world: None,
                        marks: Vec::new(),
                    })
                }
                _ => return Err(AppError::invalid("retry")),
            };
            if let Some(entry) = entry {
                self.append(t, entry)?;
            }
            Ok(None)
        })
        .await
    }

    /// chat FR-023: the last reply is replaced by a new answer to the same request.
    pub async fn regenerate_reply(
        &self,
        provider: &impl LlmProvider,
        clock: &LocalClock,
        sink: &impl ChatSink,
    ) -> Result<Option<SendRefusal>> {
        context::time_block(clock)?;
        self.start(provider, clock, sink, |t| {
            if !matches!(
                last_turn(&t.lines).map(|l| &l.entry),
                Some(Entry::Message {
                    role: Role::Assistant,
                    ..
                })
            ) {
                return Err(AppError::invalid("regenerate"));
            }
            // The whole turn goes: from its first reply on (chat R4).
            let gone = replaced(&t.lines);
            let start = turn_start(&t.lines).map_or(0, |i| i + 1);
            let target = t.lines[start..]
                .iter()
                .find(|l| {
                    matches!(
                        l.entry,
                        Entry::Message {
                            role: Role::Assistant,
                            ..
                        }
                    ) && !gone.contains(l.id.as_str())
                })
                .map(|l| l.id.clone())
                .ok_or_else(|| AppError::invalid("regenerate"))?;
            self.append(t, Entry::Replaced { target })?;
            Ok(None)
        })
        .await
    }

    /// chat FR-020, FR-021: the greeting in character after setup, sent at most once.
    pub async fn start_first_hello(
        &self,
        provider: &impl LlmProvider,
        clock: &LocalClock,
        sink: &impl ChatSink,
    ) -> Result<Option<SendRefusal>> {
        let time = context::time_block(clock)?;
        if !self.state()?.first_hello_pending {
            return Err(AppError::invalid("hello"));
        }
        let profile =
            std::fs::read_to_string(self.workspace_dir().join("USER.md")).unwrap_or_default();
        self.start(provider, clock, sink, |t| {
            let spoken = t.lines.iter().any(|l| {
                matches!(
                    l.entry,
                    Entry::Message {
                        provenance: Provenance::User,
                        ..
                    }
                )
            });
            if spoken {
                return Err(AppError::invalid("hello"));
            }
            self.ensure_context(t, clock)?;
            // A hello request left unanswered by an earlier attempt is answered, not repeated.
            if last_turn(&t.lines).is_some_and(context::hidden) {
                return Ok(None);
            }
            let entry = Entry::Message {
                role: Role::User,
                provenance: Provenance::System,
                content: vec![time, context::hello_block(field(&profile, "Call them"))],
                text: None,
                zone: Some(clock.zone.clone()),
                state: None,
                stop_reason: None,
                model: None,
                usage: None,
                recalled: None,
                world: None,
                marks: Vec::new(),
            };
            self.append(t, entry)?;
            Ok(None)
        })
        .await
    }

    /// The steps every reply shares: a key, the one running-reply slot, the entries `prepare` appends,
    /// then the reply itself.
    async fn start(
        &self,
        provider: &impl LlmProvider,
        clock: &LocalClock,
        sink: &impl ChatSink,
        prepare: impl FnOnce(&mut Transcript) -> Result<Option<ChatMessage>>,
    ) -> Result<Option<SendRefusal>> {
        if self.secrets.get().is_none() {
            return Ok(Some(SendRefusal::NoKey));
        }
        let Some(active) = self.begin() else {
            return Ok(Some(SendRefusal::Busy));
        };
        // Avatar FR-004: thinking from sending until the first text, idle however the turn ends.
        let _turn = Turn::start(&self.avatar);
        let user_message = self.transcript(prepare)?;
        // World: a message the user sent, already on disk, grows the world once a day. Growth never fails or
        // delays the message; a failure is tried again at the next one (world FR-025).
        if user_message.is_some() {
            let _ = self.grow_today(&context::local_now(clock).date);
        }
        sink.send(ChatEvent::Started { user_message });
        self.reply(provider, sink, &active, clock).await?;
        Ok(None)
    }

    /// chat FR-022: also ends a wait between retries.
    pub fn stop_reply(&self) {
        if let Some(stop) = lock(&self.chat.active).as_ref() {
            stop.notify_one();
        }
    }

    /// CTL-2: records the messages forgotten memories came from, holding none of their text, so later dreaming
    /// skips them. Returns whether the forgotten text is still in the current context (memory R14).
    /// The app changed a loaded file itself (world core's identity lines): counts as seen, so the next message
    /// carries no notice (world FR-039). Nothing to record before the first context.
    pub(crate) fn record_written(&self, file: &str, content: String) -> Result<()> {
        self.transcript(|t| {
            if context::current(&t.lines).is_empty() {
                return Ok(());
            }
            let entry = Entry::Written {
                file: file.into(),
                content,
                message: None,
            };
            self.append(t, entry).map(drop)
        })
    }

    /// The local day of the conversation's first message, from its time block (world R2).
    pub(crate) fn first_conversation_day(&self) -> Result<Option<String>> {
        self.transcript(|t| {
            Ok(t.lines.iter().find_map(|l| match &l.entry {
                Entry::Message {
                    role: Role::User,
                    provenance: Provenance::User | Provenance::System,
                    content,
                    ..
                } => content.first()?["text"]
                    .as_str()
                    .and_then(context::date_of_time_block),
                _ => None,
            }))
        })
    }

    pub(crate) fn record_forgotten(&self, messages: Vec<String>, texts: &[String]) -> Result<bool> {
        self.transcript(|t| {
            let in_current = seen_in(&t.lines, &messages, texts);
            self.append(t, Entry::Forgotten { messages })?;
            Ok(in_current)
        })
    }

    /// Whether any of `texts` is in what the companion works with now (journey FR-043).
    pub(crate) fn seen_in_context(&self, texts: &[String]) -> Result<bool> {
        self.transcript(|t| Ok(seen_in(&t.lines, &[], texts)))
    }

    /// The local days of the messages with these ids, from their time blocks.
    pub(crate) fn message_days(&self, ids: &[&str]) -> Result<Vec<String>> {
        self.transcript(|t| {
            Ok(t.lines
                .iter()
                .filter(|l| ids.contains(&l.id.as_str()))
                .filter_map(|l| match &l.entry {
                    Entry::Message { content, .. } => content.first()?["text"]
                        .as_str()
                        .and_then(context::date_of_time_block),
                    _ => None,
                })
                .collect())
        })
    }

    /// chat FR-030, FR-031: the companion starts again from its files; the history stays.
    pub fn start_fresh_context(&self, clock: &LocalClock) -> Result<ChatState> {
        context::time_block(clock)?;
        let Some(_active) = self.begin() else {
            return Err(AppError::invalid("busy"));
        };
        self.transcript(|t| self.open_context(t, clock))?;
        *lock(&self.chat.last_failure) = None;
        self.state_with(false)
    }

    /// Quitting stops a running reply and waits briefly for it to be saved.
    pub async fn shutdown(&self) {
        self.stop_reply();
        for _ in 0..40 {
            if lock(&self.chat.active).is_none() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    pub(crate) fn request(&self) -> Result<TurnRequest> {
        let cached = self.state()?.models.items;
        self.transcript(|t| request_from(self, &t.lines, &cached))
    }
}

/// The current context's request, as the next turn would send it (Principle III).
pub(crate) fn request_from(
    core: &Core,
    lines: &[Line],
    cached: &[ModelInfo],
) -> Result<TurnRequest> {
    let (model, files, version, world, extensions, recorded) = context::current(lines)
        .first()
        .and_then(|l| match &l.entry {
            Entry::Context {
                model,
                files,
                harness,
                world,
                extensions,
                skills,
                ..
            } => Some((
                model.clone(),
                files.clone(),
                harness.clone(),
                world.clone(),
                extensions.clone(),
                skills.clone(),
            )),
            _ => None,
        })
        .ok_or_else(|| AppError::invalid("context"))?;
    let (harness, mut tools, skills) = context::harness(&version);
    // Extensions R2: memory tools, then the tools of each extension the context recorded.
    tools.extend(core.ext_specs(&extensions));
    // The extension text the context recorded; contexts from before it was recorded rebuild it (ext-2, talk-1).
    let harness = if skills {
        let section = recorded.unwrap_or_else(|| core.ext_skills(&extensions));
        format!("{harness}\n{section}")
    } else {
        harness.to_owned()
    };
    Ok(TurnRequest {
        effort: effort_for(cached, &model, Effort::Low),
        system: context::system(&harness, &files, world.as_ref().map(|w| w.text.as_str())),
        messages: context::messages(lines),
        tools,
        model,
        format: None,
    })
}

/// Whether the messages or texts are in what the companion sees: the context's snapshot and lines, its kept turns
/// and its summary (compaction R13).
fn seen_in(lines: &[Line], messages: &[String], texts: &[String]) -> bool {
    let current = context::current(lines);
    let (held, _) = context::held(lines);
    let summary = context::opened_by(lines).and_then(|o| o.summary);
    let seen = serde_json::to_string(&(current, &held, summary)).unwrap_or_default();
    messages.iter().any(|id| {
        current
            .iter()
            .chain(held.iter().copied())
            .any(|l| &l.id == id)
    }) || texts.iter().any(|text| {
        let escaped = serde_json::to_string(text).unwrap_or_default();
        seen.contains(escaped.trim_matches('"'))
    })
}

/// The user's local date from a validated clock.
pub(crate) fn local_today(clock: &LocalClock) -> Result<String> {
    context::time_block(clock)?;
    Ok(context::local_now(clock).date)
}

pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(now() * 1000, |d| d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::is_openable_link;

    #[test]
    fn only_web_and_mail_links_open() {
        for url in [
            "https://example.com/a?b=c",
            "http://localhost:3000",
            "mailto:sam@example.com",
        ] {
            assert!(is_openable_link(url), "{url}");
        }
        for url in [
            "file:///C:/Windows/system32",
            "javascript:alert(1)",
            "data:text/html,hi",
            "/relative",
            "",
            "https://",
            "https://exa mple.com",
        ] {
            assert!(!is_openable_link(url), "{url}");
        }
    }
}
