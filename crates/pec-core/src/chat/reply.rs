//! The reply loop: a turn of one or more model calls with memory tools run between them; stop, retry a busy
//! provider, save the outcome (chat R4, R11; memory R3).

use super::mood::MoodParser;
use super::transcript::{Entry, Line, Provenance, ReplyState, Role};
use super::{
    Active, ChatEvent, ChatSink, LocalClock, RETRY_WINDOW, context, lock, now_ms, turn_start,
    turn_view,
};
use crate::avatar::{Mood, TurnPhase};
use crate::compaction::{Ask, CompactOutcome};
use crate::llm::{LlmProvider, ProviderError};
use crate::memory::files::LocalNow;
use crate::memory::tools::{ToolOutcome, failed};
use crate::{AppError, Core, Result};
use context::{hidden, text_block};
use serde_json::Value;
use std::time::Duration;

/// A turn ends after this many model calls that asked for tools (memory R3).
const MAX_CALLS: u32 = 8;

impl Core {
    /// Saves a reply and returns its id.
    fn save_reply(
        &self,
        content: Vec<Value>,
        state: ReplyState,
        stop_reason: Option<String>,
        usage: Option<Value>,
        model: &str,
    ) -> Result<String> {
        self.transcript(|t| {
            let entry = Entry::Message {
                role: Role::Assistant,
                provenance: Provenance::Companion,
                content,
                text: None,
                zone: None,
                state: Some(state),
                stop_reason,
                model: Some(model.into()),
                usage,
                recalled: None,
                world: None,
                marks: Vec::new(),
            };
            Ok(self.append(t, entry)?.id)
        })
    }

    fn replace(&self, target: &str) -> Result<()> {
        self.transcript(|t| {
            self.append(
                t,
                Entry::Replaced {
                    target: target.into(),
                },
            )
            .map(drop)
        })
    }

    /// Whether the turn has no reply yet: its opening message is the last message (compaction R10).
    fn awaits_first_reply(&self) -> Result<bool> {
        self.transcript(|t| {
            let gone = context::replaced(&t.lines);
            Ok(t.lines
                .iter()
                .rev()
                .find(|l| matches!(l.entry, Entry::Message { .. }) && !gone.contains(l.id.as_str()))
                .is_some_and(context::is_turn_start))
        })
    }

    /// Whether the turn just answered was opened by the hidden hello request.
    fn answered_hello(&self) -> Result<bool> {
        self.transcript(|t| Ok(turn_start(&t.lines).is_some_and(|i| hidden(&t.lines[i]))))
    }

    /// Sends the turn's final event and records how it ended.
    fn finish(&self, sink: &impl ChatSink, failure: Option<ProviderError>) -> Result<()> {
        *lock(&self.chat.last_failure) = failure;
        let state = self.state_with(false)?;
        let message = self.transcript(|t| Ok(turn_view(&t.lines)))?;
        sink.send(match failure {
            None => ChatEvent::Finished { message, state },
            Some(error) => ChatEvent::Failed {
                error,
                partial: message,
                state,
            },
        });
        Ok(())
    }

    /// Runs a reply's tool calls in order and saves their results in one `untrusted` message, followed by notices
    /// for files the user changed meanwhile (memory R3, R7). Past the call limit, each call is refused instead.
    fn answer_tools(
        &self,
        sink: &impl ChatSink,
        calls: &[(String, String, Value)],
        now: &LocalNow,
        refuse: bool,
    ) -> Result<()> {
        let workspace = self.workspace_dir();
        let (id, src, notices) = self.transcript(|t| {
            let changed =
                context::changed(&context::seen(&t.lines), &context::read_files(&workspace));
            let src = turn_start(&t.lines).map(|i| t.lines[i].id.clone());
            Ok((t.next_id(now_ms()), src.unwrap_or_default(), changed))
        })?;
        let outcomes: Vec<(&str, ToolOutcome)> = calls
            .iter()
            .map(|(tool_id, name, input)| {
                let outcome = if refuse {
                    failed("You have used your tools too many times in this reply. Answer the person now, without tools.")
                } else {
                    let at = crate::ext::local_time(&now.date, &now.time);
                    self.run_ext_tool(name, input, at)
                        .unwrap_or_else(|| self.run_tool(name, input, now, &src))
                };
                if let Some(mark) = &outcome.mark {
                    sink.send(ChatEvent::MemoryWritten { mark: mark.clone() });
                }
                (tool_id.as_str(), outcome)
            })
            .collect();
        self.transcript(|t| {
            let mut content = context::tool_results(
                outcomes
                    .iter()
                    .map(|(id, o)| (*id, o.content.as_str(), o.is_error)),
            );
            for (file, text) in notices {
                content.push(context::notice_block(&file, &text));
                let entry = Entry::Notice {
                    file,
                    content: text,
                    message: id.clone(),
                };
                self.append(t, entry)?;
            }
            for (file, text) in outcomes.iter().filter_map(|(_, o)| o.written.clone()) {
                let entry = Entry::Written {
                    file,
                    content: text,
                    message: Some(id.clone()),
                };
                self.append(t, entry)?;
            }
            t.append(Line {
                id: id.clone(),
                at: now_ms(),
                entry: Entry::Message {
                    role: Role::User,
                    provenance: Provenance::Untrusted,
                    content,
                    text: None,
                    zone: None,
                    state: None,
                    stop_reason: None,
                    model: None,
                    usage: None,
                    recalled: None,
                    world: None,
                    marks: outcomes
                        .iter()
                        .filter_map(|(_, o)| o.mark.clone())
                        .collect(),
                },
            })
        })
    }

    /// Sends the mood a marker set, and text the person may see after the turn's earlier calls' text, which the
    /// character speaks (chat FR-005, FR-008).
    fn show(
        &self,
        sink: &impl ChatSink,
        seen: &mut String,
        separate: bool,
        (out, mood): (String, Option<Mood>),
    ) {
        if let Some(mood) = mood {
            self.avatar.mood_read(mood);
            sink.send(ChatEvent::Mood { mood });
        }
        if out.is_empty() {
            return;
        }
        if seen.is_empty() && separate {
            sink.send(ChatEvent::Delta {
                text: "\n\n".into(),
            });
        }
        seen.push_str(&out);
        self.avatar.spoke(out.chars().count());
        sink.send(ChatEvent::Delta { text: out });
    }

    /// Streams the turn the conversation is owed: model calls, with tools run between them, retrying a busy
    /// provider per call (chat R11, memory R3).
    pub(super) async fn reply(
        &self,
        provider: &impl LlmProvider,
        sink: &impl ChatSink,
        active: &Active<'_>,
        clock: &LocalClock,
    ) -> Result<()> {
        let key = self.secrets.get().ok_or_else(|| AppError::invalid("key"))?;
        let now = context::local_now(clock);
        let (mut waited, mut attempt, mut calls) = (Duration::ZERO, 0u32, 0u32);
        let mut overflowed = false;
        // The text of this run's finished calls, as shown while streaming.
        let mut shown = String::new();
        loop {
            let request = self.request()?;
            // The call's text as received, and as shown: without its mood marker (avatar interfaces).
            let (mut text, mut seen) = (String::new(), String::new());
            let mut marker = MoodParser::default();
            let separate = !shown.is_empty();
            let outcome = {
                let mut on_text = |delta: &str| {
                    text.push_str(delta);
                    self.show(sink, &mut seen, separate, marker.feed(delta));
                };
                tokio::select! {
                    result = provider.stream(&key, &request, &mut on_text) => Some(result),
                    () = active.stop.notified() => None,
                }
            };
            self.show(sink, &mut seen, separate, marker.finish());
            let failure = match outcome {
                None => {
                    if !text.is_empty() {
                        let blocks = vec![text_block(&text)];
                        self.save_reply(blocks, ReplyState::Stopped, None, None, &request.model)?;
                    }
                    return self.finish(sink, None);
                }
                Some(Ok(reply)) if reply.stop_reason.as_deref() == Some("refusal") => {
                    // A refusal's partial output is discarded, not kept as a reply.
                    ProviderError::Declined
                }
                Some(Ok(reply)) if reply.stop_reason.as_deref() == Some("tool_use") => {
                    calls += 1;
                    self.avatar.turn(TurnPhase::Thinking);
                    let tools = context::tool_calls(&reply.content);
                    self.save_reply(
                        reply.content,
                        ReplyState::Complete,
                        reply.stop_reason,
                        reply.usage,
                        &reply.model,
                    )?;
                    if !text.is_empty() {
                        if !shown.is_empty() {
                            shown.push_str("\n\n");
                        }
                        shown.push_str(&text);
                    }
                    self.answer_tools(sink, &tools, &now, calls >= MAX_CALLS)?;
                    if calls > MAX_CALLS {
                        // The companion kept calling tools after being told to stop; the turn ends here.
                        return self.finish(sink, None);
                    }
                    (waited, attempt) = (Duration::ZERO, 0);
                    continue;
                }
                Some(Ok(reply)) => {
                    let state = if reply.stop_reason.as_deref() == Some("max_tokens") {
                        ReplyState::Incomplete
                    } else {
                        ReplyState::Complete
                    };
                    let hello = state == ReplyState::Complete && self.answered_hello()?;
                    self.save_reply(
                        reply.content,
                        state,
                        reply.stop_reason,
                        reply.usage,
                        &reply.model,
                    )?;
                    if hello {
                        self.update_state(|s| s.first_hello_pending = false)?;
                    }
                    return self.finish(sink, None);
                }
                // Compaction R10: too long on the turn's first call compacts inline and retries once.
                Some(Err(failure))
                    if failure.error == ProviderError::ContextTooLong
                        && !overflowed
                        && text.is_empty()
                        && self.awaits_first_reply()? =>
                {
                    overflowed = true;
                    sink.send(ChatEvent::Compacting);
                    let outcome = self
                        .compact(provider, &key, clock, Ask::default(), None)
                        .await?;
                    if outcome == CompactOutcome::Compacted {
                        continue;
                    }
                    ProviderError::ContextTooLong
                }
                Some(Err(failure)) => {
                    let partial = (!text.is_empty())
                        .then(|| {
                            let blocks = vec![text_block(&text)];
                            let model = &request.model;
                            self.save_reply(blocks, ReplyState::Incomplete, None, None, model)
                        })
                        .transpose()?;
                    if failure.error == ProviderError::Busy {
                        let wait = failure
                            .wait
                            .unwrap_or_else(|| Duration::from_secs(1 << attempt.min(5)));
                        if waited + wait <= RETRY_WINDOW {
                            if let Some(partial) = &partial {
                                self.replace(partial)?;
                            }
                            let seconds = wait.as_secs().max(1) as u32;
                            sink.send(ChatEvent::Waiting {
                                seconds,
                                text: shown.clone(),
                            });
                            let stopped = tokio::select! {
                                () = tokio::time::sleep(wait) => false,
                                () = active.stop.notified() => true,
                            };
                            if stopped {
                                return self.finish(sink, None);
                            }
                            waited += wait;
                            attempt += 1;
                            continue;
                        }
                    }
                    failure.error
                }
            };
            return self.finish(sink, Some(failure));
        }
    }
}

/// The size of the context after a reply, from the provider's own counts (chat R10).
pub(crate) fn context_tokens(usage: &Value) -> u64 {
    [
        "input_tokens",
        "cache_read_input_tokens",
        "cache_creation_input_tokens",
        "output_tokens",
    ]
    .iter()
    .filter_map(|f| usage[*f].as_u64())
    .sum()
}
