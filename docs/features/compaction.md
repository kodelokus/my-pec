# Compaction

Built.

This feature covers CMP-1 to CMP-8 (`docs/requirements.md` section 6.7) and product success criterion SC-3. It replaces the stopgap of the chat feature, in which a conversation that reached its working size had to be restarted by the user with a fresh context (chat FR-030, FR-032).

## User stories

### US1. The conversation just keeps going (P1)

The user talks with Steve every evening for weeks. They never see a warning that the conversation is too long and never have to start over. In the middle of planning a trip, the oldest part of the conversation quietly leaves what Steve works with, and Steve carries on: it still knows they are planning the trip to Lisbon, that it promised to suggest three restaurants, that the flight is on 14 March at 07:10, and that the user was a bit stressed about it. The user can still scroll back through every word they ever exchanged.

**Why this priority**: Without this, every long conversation eventually stops and asks the user to start over, which breaks the promise of one continuous companion. It is the core of the slice.

**How to check:** Run a scripted conversation long enough to pass the working size several times, with a fact, a commitment and an open question planted early in each stretch; confirm the conversation never stops or shows an error, that after each compaction the companion continues the active topic and still has the planted details, and that the full history stays visible and on disk.

**Acceptance Scenarios**:

1. **Given** a conversation that has reached its working size, **When** the current reply finishes, **Then** older turns are replaced by one summary in what the companion works with, the most recent turns are kept word for word, and the user did nothing to cause it.
2. **Given** a compaction has happened, **When** the user sends their next message, **Then** the companion continues the active topic without asking the user to repeat it, and keeps the names, dates, numbers and paths from before exactly as written.
3. **Given** a compaction has happened, **When** the user scrolls back, **Then** every earlier message is still there and readable, and a thin divider marks where the compaction happened.
4. **Given** a compaction is under way, **When** the user sends a message, **Then** the message is sent and answered without waiting for the compaction, and the compaction runs again after that reply.
5. **Given** the companion's profile, long-term memory or soul changed since the context began, **When** a compaction happens, **Then** the companion continues with the current files.

---

### US2. Nothing important is lost on the way (P1)

Before the older part of the conversation leaves Steve's view, Steve gets a quiet moment to write down anything worth keeping in today's daily note: the user's sister is visiting next Tuesday, the new job starts on the 3rd. None of this shows up in the conversation. A week later, long after that stretch was summarized, Steve still knows about the visit because it is in the note.

**Why this priority**: A summary is short by design. The flush is what turns "the conversation was compacted" into "the companion still remembers", and it is what OpenClaw's memory design relies on.

**How to check:** Plant durable facts that the companion has not yet noted, push the conversation past its working size, and confirm that today's daily note gained lines for them with their origin and source, that nothing appeared in the conversation, and that a later compaction does not write the same lines again.

**Acceptance Scenarios**:

1. **Given** a compaction is about to happen, **When** it runs, **Then** the companion first gets one silent chance to append durable facts to today's daily note, and no message, mark or text from it appears in the conversation.
2. **Given** a line the flush would write is already in today's note, **When** the flush runs, **Then** the line is not written again.
3. **Given** the flush fails or writes nothing, **When** compaction continues, **Then** the compaction still completes.
4. **Given** the flush wrote notes, **When** the compaction is abandoned or fails before it completes, **Then** none of the flush's notes are saved, and memory is exactly as it was before the compaction started.
5. **Given** the flush wrote notes and the compaction completed, **When** the next context opens, **Then** those notes are part of today's note the companion starts with, and each one records that the companion wrote it and which conversation it came from.

---

### US3. The provider says the conversation is too long (P2)

The user pastes a long document. The provider answers that the conversation is now too long, before the working size was reached. Instead of an error, Steve compacts and tries again, and the reply arrives. Only if that also fails does the user see a plain message, with a way to try again or start a fresh context.

**Why this priority**: Rare in normal use, but without it a single long paste turns into a dead end.

**How to check:** With the fake provider answering "too long" once, send a message and confirm the app compacts and retries once and the reply arrives with no error shown; with the provider answering "too long" every time, confirm the user sees one plain message, their message is kept, and they can retry or start a fresh context.

**Acceptance Scenarios**:

1. **Given** the provider reports that the conversation is too long, **When** the user's message was being answered, **Then** the app compacts and retries that message once, automatically, and the user sees no error if the retry succeeds.
2. **Given** the automatic retry also fails, or no summary can be produced, **When** the user looks at the conversation, **Then** they see a plain explanation, their message is still there, and they can retry or start a fresh context.
3. **Given** no valid summary can be produced, **When** compaction gives up, **Then** what the companion works with stays exactly as it was, nothing is dropped, and compaction is tried again at the next turn boundary.

---

### US4. Compact now, and see where it happened (P3)

The user has spent an hour debugging a spreadsheet with Steve and wants to move on to something else. They choose "compact now" and add a short focus note, "keep the formula we settled on". The conversation tidies itself, a divider appears, and the user can open the divider to read what Steve kept in its summary.

**Why this priority**: A comfort and transparency feature. Automatic compaction already keeps the conversation going without it.

**How to check:** Choose "compact now" with a focus note, confirm a divider appears, the summary reflects the focus note, and opening the divider shows the summary text; choose "compact now" with nothing to compact and confirm the app says so and changes nothing.

**Acceptance Scenarios**:

1. **Given** the conversation has turns older than the recent ones that are always kept, **When** the user chooses "compact now", optionally with a focus note, **Then** a compaction runs, a divider appears, and the summary gives weight to the focus note.
2. **Given** a divider marks a compaction, **When** the user opens it, **Then** they can read the summary the companion continued with.
3. **Given** there is nothing older than the recent turns that are always kept, **When** the user chooses "compact now", **Then** the app says there is nothing to compact yet and changes nothing.
4. **Given** a reply is being written, **When** the user looks for "compact now", **Then** it is not available until the reply finishes.

---

### US5. Choose the working size and the model that compacts (P3)

A user who wants to spend less sets the conversation's working size lower and picks a cheaper model for compaction in settings. Another user, who changes the chat model, chooses "switch now" and the conversation compacts onto the new model instead of starting over.

**Why this priority**: The defaults work for everyone (Principle VII). These settings are cost levers for users who care.

**How to check:** Lower the working size in settings and confirm compaction happens at the new size; pick another model for compaction and confirm the flush and summary use it while chat keeps its own model; change the chat model, choose "switch now", and confirm the conversation compacts and continues on the new model with its recent turns.

**Acceptance Scenarios**:

1. **Given** settings, **When** the user changes the working size within the allowed range, **Then** the next compaction happens at the new size.
2. **Given** settings, **When** the user picks a model for compaction, **Then** the flush and the summary use that model, and chat keeps the chat model.
3. **Given** the user changed the chat model, **When** they choose "switch now", **Then** the conversation compacts and continues on the new model with its summary and recent turns, rather than starting a fresh context with none of them.
4. **Given** a new install, **When** the user never opens these settings, **Then** compaction works with the default working size and the chat model.

---

## Edge cases

- **The working size is reached in the middle of a tool round.** Compaction waits until the whole turn, including every tool call and its result, has finished. It never starts between a tool call and its result.
- **The latest exchange is bigger than the recent part that is always kept**, for example after a long paste. The latest user message and everything after it are always kept whole, even when that is more than the usual recent part.
- **Nothing is older than the recent part.** There is nothing to summarize, so compaction does nothing. If the provider still reports the conversation is too long, the user is told and can retry or start a fresh context (US3).
- **The user sends a message while a compaction runs.** The compaction is abandoned before it changes anything, the message is answered in the current context, and the compaction runs again after that reply. The part of the working size held back for this (the reserve) makes room for the extra turn.
- **The app closes or crashes during a compaction.** Nothing was saved, so the conversation and memory are as they were before it started. Compaction runs again at the next turn boundary.
- **The provider is offline or rejects the key during a compaction.** The compaction fails without changing anything and is tried again at a later turn boundary. Chat behaves as the chat feature describes for the same error.
- **A second compaction.** The new summary covers the previous summary and everything since, so the companion never sees two summaries.
- **Forgotten memories.** Messages the user marked as forgotten (the memory feature) are left out of what is summarized and out of the recent turns the companion continues with. After a compaction, the companion no longer sees them.
- **The context was started before memory existed** (memory R2). A compaction opens the next context with memory tools, like a fresh context would.
- **The summary comes back empty, malformed or too long.** It is asked for once more. If it fails again, nothing changes (US3, scenario 3).
- **The model chosen for compaction is no longer offered.** Compaction falls back to the chat model, and settings say so.
- **The working size is set below what the recent part, the summary and the companion's files need.** The setting does not allow it: the range has a floor that leaves room for them.
- **Soul or profile edited mid-context.** A compaction starts the next context from the current files, so no notice about those edits needs to carry over.
- **Today's daily note is over its size.** The flush still appends, and the next context loads the note cut to its usual size, as the memory feature describes.
- **A day passes between the reply that reached the working size and the next message.** Compaction still runs at the next turn boundary, and the next context loads that day's and the previous day's notes.

## Requirements

**Budget**

- **FR-001**: The app MUST track how much of the working size the current context uses, from the size the provider reports for the latest reply.
- **FR-002**: When the context reaches the working size minus a reserve, the app MUST compact at the next turn boundary with no action from the user (CMP-1). The reserve MUST leave room for a typical turn, and a turn that goes past the working size MUST still be answered.
- **FR-003**: The working size MUST default to 48,000 tokens and MUST be adjustable in settings within a range whose floor leaves room for the recent turns, the summary and the companion's files, and whose ceiling does not exceed the chat model's context window (CMP-8).
- **FR-004**: The chat feature's offer to start a fresh context when the working size is reached MUST be replaced by automatic compaction. Starting a fresh context MUST stay available where the chat and memory features offer it for other reasons, and after compaction has failed (FR-022).

**Turn boundary and yielding**

- **FR-005**: A compaction MUST start only at a turn boundary: never during a reply and never between a tool call and its result.
- **FR-006**: A message the user sends while a compaction is under way MUST NOT wait for it (Principle V). The compaction MUST stop before it saves anything, the message MUST be answered in the current context, and the compaction MUST run again after that reply.
- **FR-007**: A compaction MUST save its result in one step at its end. Until then nothing in the conversation file, the memory files or the companion's context may change. An abandoned, failed or interrupted compaction MUST leave the conversation and memory exactly as they were before it started (NFR-REL-3, Principle V).

**Memory flush**

- **FR-008**: Before summarizing, the app MUST give the companion one silent chance to append durable facts to today's daily note, with only note-writing allowed (CMP-2).
- **FR-009**: Nothing from the flush may appear in the conversation, as a message, a memory mark or any other trace, and the flush MUST NOT become part of what the companion works with.
- **FR-010**: The flush MUST only append. A line already present in today's note MUST NOT be written again.
- **FR-011**: Each note the flush writes MUST record that the companion wrote it and which conversation it came from, like any other note (Principle IV).
- **FR-012**: The flush's notes MUST be saved only when the compaction completes, together with it (FR-007).
- **FR-013**: A flush that fails, times out or writes nothing MUST NOT stop the compaction. Its outcome MUST be recorded with the compaction.

**Cut and summary**

- **FR-014**: The most recent turns, up to a set amount (default 8,000 tokens), MUST be kept word for word. The cut MUST fall where a user message begins, MUST NOT separate a tool call from its result, and MUST always keep the latest user message and everything after it.
- **FR-015**: Everything before the cut, including the previous summary, MUST be summarized into one summary that keeps, in a fixed structure: the active topic and the user's current request; open commitments and unanswered questions; the tone of the relationship and the mood of the conversation; facts from that stretch with names, dates, numbers and paths exactly as written; and pointers to the daily notes that hold more detail (CMP-4).
- **FR-016**: Messages marked as forgotten (memory FR-027) MUST be left out of what is summarized and out of the kept turns the companion continues with.
- **FR-017**: A summary MUST be non-empty, MUST follow the fixed structure, and MUST fit within a size limit (default 8,000 characters). A summary that fails these checks MUST be requested once more. If it fails again, the compaction MUST fail with nothing changed (CMP-6).
- **FR-018**: If there is nothing before the cut, a compaction MUST do nothing.

**New context**

- **FR-019**: A completed compaction MUST open a new context that starts from the current harness rules, identity, soul, profile, long-term memory, and today's and yesterday's daily notes (including what the flush just wrote), followed by the summary and the kept turns.
- **FR-020**: Within a context, what the companion is sent MUST only grow by appended messages, from the context's first message until the next compaction (Principle III). The kept turns MUST be carried into the new context in a form the provider accepts without changing any turn's words.
- **FR-021**: The conversation file MUST keep the complete history. A compaction MUST only append to it, recording the summary, where the kept turns begin, the size before and after, the model used, the tokens spent, and the flush outcome (CMP-3).

**Failure and overflow**

- **FR-022**: If the provider reports that the conversation is too long, the app MUST compact and retry the user's message once, automatically (CMP-5). If the retry fails or no summary can be produced, the app MUST tell the user in plain words, keep their message, and offer to retry or start a fresh context.
- **FR-023**: A failed compaction MUST NOT drop or change any history, and MUST be tried again at a later turn boundary (CMP-6). Repeated failures MUST NOT block chat while the context still fits.

**Manual compaction and visibility**

- **FR-024**: The user MUST be able to choose "compact now" when no reply is being written, with an optional focus note that the summary gives weight to (CMP-7). With nothing to compact, the app MUST say so and change nothing.
- **FR-025**: The conversation MUST show a thin divider where each compaction happened. Opening it MUST show the summary the companion continued with.
- **FR-026**: While a compaction runs, the app MUST show a short, calm status. It MUST NOT block typing or sending (FR-006).

**Model**

- **FR-027**: The flush and the summary MUST use the chat model unless the user picks another model for compaction in settings. If the picked model is no longer offered, compaction MUST fall back to the chat model and settings MUST say so (CMP-8).
- **FR-028**: When the user changes the chat model, "switch now" MUST compact onto the new model, keeping the summary and recent turns, instead of starting a fresh context (replacing chat FR-032's offer).

**Boundaries**

- **FR-029**: The flush and the summary MUST be sent only to the provider the user chose (Principle II). No other service may be contacted.
- **FR-030**: All compaction rules (the budget, the cut, the flush, the summary checks and the new context) MUST be decided in the core, with no dependency on the desktop shell, and MUST be testable with the fake provider (Principle VI).

### Key entities

- **Working size**: How much conversation the companion works with before compacting. A cost choice, not a model limit. Set in settings, with a default.
- **Reserve**: The part of the working size held back so that a turn can still be answered while a compaction is due or under way.
- **Context**: What the companion currently works with, as in the chat feature. Starts with its files, then grows only by appended messages, until the next compaction or fresh context.
- **Memory flush**: A silent turn before compaction in which the companion may append durable facts to today's daily note. It is never part of the conversation.
- **Cut**: The point that splits the context into the older part to summarize and the recent turns kept word for word.
- **Kept turns**: The recent turns carried into the new context word for word.
- **Summary**: One structured text that replaces everything before the cut, including any previous summary.
- **Compaction record**: An entry appended to the conversation file: the summary, where the kept turns begin, sizes before and after, the model used, tokens spent and the flush outcome.
- **Divider**: The thin line in the conversation where a compaction happened, which opens to show the summary.

## Success criteria

- **SC-001**: In a suite of scripted long conversations, each passing the working size at least three times, at least 95% of the turns right after a compaction continue the active topic correctly, and 0 show an error to the user (product SC-3). The mechanics (compacting, continuing, no error) are checked in CI with scripted replies; how well the companion continues is measured against the real provider before release.
- **SC-002**: In the same suite, 100% of the names, dates, numbers and paths planted in a summarized stretch appear exactly as written in the summary that replaces it.
- **SC-003**: Durable facts planted in a stretch that is then compacted, and not noted before, are in today's daily note after the compaction in at least 90% of cases in the real-provider suite. In CI, with scripted replies, 100% of the facts the flush returns are saved exactly once.
- **SC-004**: After any number of compactions, the conversation file still holds 100% of the messages ever exchanged, unchanged, and the user can scroll to every one of them.
- **SC-005**: Interrupting a compaction at any step, by a crash, a closed app, a provider failure or an incoming message, leaves the conversation file and every memory file byte for byte as they were before it started, in 100% of attempts.
- **SC-006**: A message sent while a compaction runs starts streaming its reply no later than it would with no compaction running, with the same 0.15 seconds of the app's own work as the chat feature (NFR-PERF-2).
- **SC-007**: Within each context, every request is the previous request plus appended messages, checked by an automated comparison over a conversation with at least two compactions.
- **SC-008**: Over 30 days of daily conversation at the default working size, no context grows past the working size by more than one turn, and no request is rejected for being too long without an automatic compaction and retry.
- **SC-009**: A compaction at the default settings finishes within 30 seconds for 95% of runs on the real provider, and the user can keep chatting the whole time.
- **SC-010**: A new user who never opens settings gets automatic compaction with nothing to configure (Principle VII).

## Assumptions

- **Depends on slices 2 and 3.** Contexts, the conversation file, the fresh context and the divider come from the chat feature. Daily notes, note-writing, provenance, and messages marked as forgotten come from the memory feature.
- **The flush is staged.** Its notes are saved together with the compaction (FR-012), so an abandoned compaction changes no memory, as NFR-REL-3 requires. A flush that has to be redone after an abandoned compaction costs one more small request; that is accepted. If compaction fails repeatedly and the user starts a fresh context instead, that attempt's staged notes are not saved. The conversation itself stays on disk.
- **Yielding instead of waiting.** Principle V says background work yields to an incoming message, and Principle III says compaction runs synchronously at a turn boundary. This feature reads them together: a compaction never runs alongside a turn, a message that arrives during one stops it before it saves anything, and it runs again after that reply. The reserve (default 6,000 tokens) makes room for that extra turn. `docs/architecture.md` section 7.2 step 1 ("no new turn is accepted until it finishes") is updated to match with the plan.
- **After a dream.** Opening a new context after a dream is a compaction trigger in the architecture, but it belongs to dreaming (slice 5).
- **Activity log and usage meter are later.** Each compaction records its sizes, model, tokens and flush outcome (FR-021) so that the activity log (CTL-5) and the usage meter (KEY-5, slice 8) can show them. Neither is built here.
- **Starting values** from `docs/architecture.md` section 15: working size 48,000 tokens, reserve 6,000, kept recent turns 8,000, summary up to 8,000 characters, flush on. The floor of the working-size range is set in the Decisions from the reserve, the kept turns, the summary and the companion's files at their size budgets; about 32,000 tokens is expected.
- **The reserve and the kept size are not settings.** Only the working size and the compaction model are (CMP-8).
- **The focus note is the user's words.** It shapes the summary and is recorded with the compaction, with user provenance.
- **The summary is in the conversation's language.** The fixed structure is for the model and the checks, not shown as a form; the divider shows the summary as text.
- **Redacting conversation messages is out of scope**, as in the memory feature. Forgotten messages are left out of the summary and the kept turns (FR-016) but stay in the conversation file.
- **Platform.** Windows 11 first, as for the rest of the app.

## Decisions

Each section records a decision, why it was made, and what was rejected. API facts come from the Claude API reference bundled with the project's tooling (prompt caching, structured outputs, preserved thinking, model migration notes) as of 2026-11-14. The slice builds on the chat page (contexts, the conversation file, the reply loop, the fresh context) and the memory page (daily notes, `memory_note`, provenance, forgotten messages).

### R1. How big the context is, and when it is due

**Decision**: The size of the current context is the provider's count for its latest reply: `input_tokens + cache_read_input_tokens + cache_creation_input_tokens + output_tokens` of the last saved reply in the context (`reply::context_tokens`, chat R10). A compaction is **due** when that size reaches `budgetTokens − reserveTokens` (48,000 − 6,000 by default), no reply is running, and the conversation owes no reply (the last turn ended normally). Right after a compaction the new context has no reply yet, so its size reads 0 until the next reply reports one.

**Rationale**: The architecture forbids estimating tokens with another vendor's tokenizer (section 11), and the provider already reports the exact size on every reply, for free. The reserve exists so the turn that crosses the threshold, and one more turn sent while a compaction runs (R3), still fit inside the working size (FR-002, SC-008). "Owes no reply" is the turn boundary of FR-005: a turn stopped between a tool call and its result is owed a retry first.

**Alternatives considered**: The token-counting endpoint before every turn (a request per message for a number the reply gives anyway). Counting characters (an estimate the architecture rules out, and wrong for languages written without spaces). Checking before the request instead of after the reply (the size of the coming request isn't known until it is built and counted).

### R2. Who starts an automatic compaction

**Decision**: The core decides; the webview asks. `ChatState` gains `compactionDue`. After every `Finished` event, and once when the conversation view loads, the webview calls a new async command `compact_conversation(clock, focus: null, manual: false)` if the state it just received says `compactionDue`. The core checks again and does nothing if it is not due. "Compact now" and "switch now" call the same command with `manual: true`.

**Rationale**: This is how the first hello already works (`firstHelloPending`, the chat feature): the rule lives in the core, the webview only kicks it off, and the clock the compaction needs (for the flush's note times and the new context's daily notes) comes from the webview like every other clock (chat R8). Running it inside `send_message` after the reply would keep that command's promise open for the whole compaction; the webview clears the composer when the promise resolves, so text typed meanwhile could be lost, which chat FR-028 forbids. A separate command also covers the app closing after the reply that made it due: the next launch's first `chat_state` says `compactionDue` and the view starts it.

**Alternatives considered**: A task the core spawns (needs a `'static` core and a runtime handle in `steve-core`, and a channel back to the webview for the status line; Principle VI and the lean-code rule against unneeded concurrency both argue against). A timer in the shell (puts a rule in the shell).

### R3. Running between turns, and stopping for a message

**Decision**:

- The chat runtime gains a second slot next to the running-reply slot: `compacting: Option<Arc<Notify>>`. Only one compaction runs at a time; a second call while one runs returns `Busy`.
- A compaction starts only if no reply is running. It snapshots what it needs (the context's lines, their count, the files, the settings) and releases every lock.
- `begin()`, which every reply passes through, notifies the compacting slot's signal. The compaction awaits each model call in a `select!` against that signal, and stops as soon as it fires, saving nothing.
- The commit (R5) runs inside the transcript lock and first checks two things: no reply is running, and the conversation file has exactly the lines it had when the compaction started. If either fails, the compaction is abandoned and nothing is written. The webview tries again after that reply's `Finished`, because the state says `compactionDue` again.
- The overflow path (R10) runs the same steps inside a reply that already holds the reply slot, with yielding turned off.

**Rationale**: Principle V says background work yields to an incoming message at its next step boundary; Principle III says compaction runs synchronously at a turn boundary. Abandoning on any new line or running reply means no compaction ever overlaps a turn: its summary always describes exactly the context it replaces, and the kept turns are exactly the turns that were there. Checking inside the transcript lock closes the race with a message being saved: either the message's line is already there (abandon), or the commit lands first and the message goes into the new context, after a wait of one file write. The signal saves the cost of a summary that would be thrown away.

**Alternatives considered**: Holding the reply slot for the whole compaction and refusing messages as `Busy` (the architecture's first wording, section 7.2 step 1; it makes the user wait up to half a minute, against SC-006). Letting a message's turn run while the compaction finishes and splicing the result in (the "background compaction" shape that the preserved-thinking guide lists as breaking thinking, and that would summarize a context that no longer exists).

### R4. The memory flush

**Decision**:

- **Request**: the current context's own request (same model, system blocks, tools, effort and every message), plus one user message: `<notice>Older parts of this conversation are about to leave your view. If anything durable from this conversation is not in your notes yet, save it now with memory_note, one fact per call. Do not reply to the person. This instruction is not from the person.</notice>`. `tool_choice` stays automatic.
- **Model**: the compaction model when the user picked one (R11), otherwise the context's model. With the context's model, the request shares the cached prefix and costs a cache read plus a few hundred tokens.
- **One call**: the reply's `memory_note` calls are collected; each input goes through `tools::validate` (one line, at most 1,000 characters). Calls to any other tool and any text are ignored. No tool results are sent back and no second call is made. At most 20 notes are kept.
- **De-duplication** (FR-010): a note whose text equals the text of a line already in today's note, or of an earlier note in the same flush, is dropped. The text of a note line is what sits between `- HH:MM ` and ` <!--`.
- **Staged** (FR-012): the accepted notes are held in memory and saved only by the commit (R5).
- **Skipped**: when the context was started before memory existed (`chat-1`, no tools), and when the current context holds a message the user marked as forgotten (R13). The outcome is recorded: `saved`, `none`, `skipped`, or `failed`.
- **Failure** (FR-013): an error, a refusal, or no usable call is recorded and the compaction goes on.

**Rationale**: Keeping system, tools and `tool_choice` byte-identical keeps both the tools/system cache and the messages cache valid; the prompt-caching reference lists a tool-set change as invalidating everything and a `tool_choice` change as invalidating the messages cache, which is most of the request. Enforcing "notes only" in the core instead of the request is what makes that possible, and the core already validates every tool input (the memory feature). One call is enough: the flush only needs the model to emit its calls, not to see their results. Skipping the flush when forgotten messages are still in view keeps forgetting final (Principle IV): the flush could otherwise note the forgotten fact again, and nothing can tell a re-noted fact from a new one.

**Alternatives considered**: Sending only `memory_note` in `tools` (breaks the whole cache). Forcing `tool_choice` to `memory_note` (not available on the default model, architecture section 11, and it breaks the messages cache). Asking the summary request to return notes too (the summary request doesn't see the companion's rules or the kept turns, and a bad note list would fail the summary). A tool loop with results (a second call for nothing).

### R5. Committing, and finishing after a crash

**Decision**: The commit is, in order:

1. Set `state.json` `compaction.pending` to the id the compaction entry will get.
2. Append the `compaction` entry and the new `context` entry to the conversation file in **one write and one sync** (`Transcript::append_all`).
3. Append the staged notes to today's daily note through `add_note`, de-duplicated again against the file as it is now.
4. Clear `compaction.pending`.

A compaction counts as done when its `context` entry is on disk. When the conversation file is opened (the first chat command after launch), if `compaction.pending` names an entry:

- If that entry and its `context` entry are both in the file, steps 3 and 4 run again. De-duplication makes this safe.
- Otherwise (a torn write left only part of it), the marker is cleared and the stray `compaction` line, if any, is ignored: a `compaction` entry counts only when a `context` entry naming it follows.

The new context's snapshot of today's note already includes the staged notes, computed in memory, so the companion starts with them even if step 3 has to be finished later.

**Rationale**: FR-007 and NFR-REL-3 want an interrupted compaction to change nothing, but a compaction writes two files and can't do both atomically. Making the conversation file the commit point, with its two lines in one synced write, gives a single moment before which nothing changed. The marker exists so that a crash between steps 2 and 3 finishes the notes, and so that recovery never runs at any other time: a note the user later deletes by hand is never put back (Principle I).

**Alternatives considered**: Writing the notes first (a crash before the conversation file leaves notes from a compaction that never happened). Finishing by comparing the latest compaction's notes with the daily note at every launch (puts back notes the user deleted). Holding the notes in the `context` entry only (the daily note is the source of truth for notes; the conversation file is not searched).

### R6. The cut

**Decision**: The cut is computed over the messages of the current context: the kept turns it started with (R8), then the lines after its `context` entry.

- **Sizes**: the size of everything from a user message to the end is the current size minus the size reported by the last reply before that message (R1). Only user messages after the current context's first reply can be measured this way; earlier positions are treated as too far back to keep.
- **Candidates**: the start of each user message the person typed (or the hidden hello request), never a tool-result message, so a tool call is never separated from its result.
- **Choice**: the earliest candidate whose tail fits `keepRecentTokens` (8,000). If none fits, the latest user message is the cut: it and everything after it are always kept (FR-014).
- **Nothing to compact** (FR-018): if no message of the context lies before the cut, or the context holds nothing new since it began (only the turns the last compaction kept), the compaction does nothing and says so.
- Turns that were replaced (the chat feature) and turns the user marked as forgotten (R13) count for the sizes, but are never kept or summarized.

**Rationale**: Usage differences are the provider's own numbers, so the kept part is measured, not estimated (R1). At the default settings the context holds roughly 20,000 tokens of new conversation when a compaction is due, so the cut always lands among measured positions. Cutting at a user message keeps whole turns, which also keeps every tool call with its result.

**Alternatives considered**: Counting kept turns instead of tokens (one long paste would keep far too much). The token-counting endpoint per candidate (requests for numbers we already have).

### R7. The summary request

**Decision**:

- **Input as text**: one user message that renders the part being summarized as plain text, in order: the previous summary, if any, then each turn as `[Friday 13 November 2026, 21:40] Sam: …` and `Steve: …`, with the companion's memory writes as `(Steve saved to memory: …)` and file edits as `(Sam edited SOUL.md)`. Recalled-memory blocks and tool results are left out; they come from memory, which the companion keeps anyway. Names come from `IDENTITY.md` (`Name`) and `USER.md` (`Call them`), with "the companion" and "the person" when absent. A focus note, if given, follows as `Focus from Sam: …`.
- **System prompt**: a fixed summarizer instruction (`compaction/summary.md`), the same for every compaction, asking for the structure of CMP-4, in the language of the conversation, keeping names, dates, numbers and paths exactly as written, and writing from the companion's side.
- **Structured output**: `output_config.format` with a JSON schema: `thread` (string), `commitments` (array of strings), `relationship` (string), `facts` (array of strings), `notes` (array of dates `YYYY-MM-DD`). Strings are plain text. No tools are sent.
- **Model and effort**: the compaction model, or the chat model; effort `low` where the model takes one, like chat (architecture section 11 says effort is tuned per route by measurement). `max_tokens` stays the client's 16,000.
- **Checks** (FR-017): the reply parses against the schema; `thread` is not empty; every date in `notes` is a valid date; the rendered summary (R9) fits `maxSummaryChars` (8,000). A failed check, an empty reply, a refusal, or `max_tokens` asks once more; a second failure fails the compaction. A provider error fails it at once.

**Rationale**: Rendering the old turns as text, instead of replaying them, lets the summary leave out forgotten turns (R13) without editing the history the API checks, lets it run on another model (FR-027), and keeps thinking blocks out of a request that has no use for them. The structured-output format guarantees parseable JSON on the default model and replaces the prefill that newer models reject; it is the format the architecture names for summaries (section 11). A fixed schema makes the five parts of CMP-4 checkable.

**Alternatives considered**: Replaying the context up to the cut with an instruction appended (cache-friendly, but it can't drop forgotten turns without an edit the preserved-thinking check rejects, and it ties the summary to the chat model). Free text with headings checked by pattern (no guarantee it parses). The API's server-side compaction (R14).

### R8. The kept turns in the new context

**Decision**: Each kept turn is replayed as the user message's content exactly as saved (time block, recalled memory, notices, text), then **one** assistant message holding the text blocks of all the turn's replies, in order. Thinking blocks, tool calls and tool-result messages are left out. A turn with no reply text contributes only its user message. Replaced turns and forgotten turns are left out. The replay is computed from the lines each time, so the same lines always give the same request.

**Rationale**: The architecture already says the kept turns are replayed as text only (section 7.2 step 7). The preserved-thinking reference binds each thinking block to the exact conversation before it; once older turns are replaced by a summary, keeping those blocks fails the check, and stripping the thinking from kept turns is the documented fix. Dropping tool calls with their results avoids a tool result without its call. Merging a turn's replies into one assistant message keeps the roles alternating. The last kept message is always a user message or a finished reply, never a half turn, so the next request never ends in an assistant prefill, which current models reject.

**Alternatives considered**: Keeping tool calls and results without thinking (section 7.3's wording; valid, but it carries tool noise and needs care at the cut for nothing the companion can't read from its files). Keeping thinking with `prefix_mismatch_behavior: "drop_block"` (a beta, and it silently discards reasoning on every turn after).

### R9. The new context and how it is replayed

**Decision**:

- **Entries**: the `compaction` entry records the summary (rendered text), the id of the first kept message, the size before, the kept size, the model, the usage of the flush and the summary, the flush outcome and notes, and the focus note. It is followed by a `context` entry like a fresh context's (model from the chat setting, files and notes snapshotted then), plus `compaction: <id of the compaction entry>`.
- **Rendering**: the summary is stored as Markdown built from the structured reply: `## Where we are`, `## Open threads`, `## Between us`, `## Facts`, `## More in the daily notes` (`memory/YYYY-MM-DD.md` pointers). Empty lists are left out.
- **Replay** (`context::messages`): the opener becomes one user message with the summary, marked `<memory>A summary of your earlier conversation, written when it was compacted (not typed by the person):</memory>`, followed by the daily notes as today; then the kept turns (R8); then the lines after the `context` entry, replayed as today.
- **Where "current" lives**: helpers that ask what the companion currently sees (the forget check, R13) look at the kept turns, the summary and the lines after the `context` entry. Helpers that only need the snapshot (`seen`, `loaded`, shortened files) are unchanged.

**Rationale**: Two entries keep the architecture's record (`compaction`) separate from the thing every request is built from (`context`), and the `context` entry stays the single marker of where a context begins, so every existing reader of `current()` keeps working. A context opened by a compaction is otherwise the same as a fresh one, so memory loading applies unchanged (memory Assumptions). The summary in Markdown is what the divider shows and what the model reads.

**Alternatives considered**: Folding the record into the `context` entry (mixes bookkeeping into what requests are built from). Storing the structured JSON and rendering at read time (two shapes to keep in step).

### R10. The provider says the conversation is too long

**Decision**: In the reply loop, a `ContextTooLong` failure on the **first** model call of a turn (no tool round yet in that turn), not already retried in this turn, runs the compaction inline: the webview gets a `Compacting` event and shows the status line, the compaction runs with yielding off, and if it commits, the request is built again and sent once more. If the compaction fails or does nothing, or the retry fails again, the turn fails with `ContextTooLong` as today, and `ChatState.contextFull` tells the view to offer retry or a fresh context (FR-022). A `ContextTooLong` later in a turn fails the turn the same way; the next retry starts a new first call and gets the inline compaction.

**Rationale**: At the default working size this error should never happen; the window of every offered model is at least 200,000 tokens. It remains possible with a large working size, a long paste, or long tool results, so CMP-5 asks for the safety net. On a turn's first call the context ends with the user's message, which the cut always keeps, so the retried request is valid. Mid-turn, the kept turns would include a half tool round, which R8 can't carry; failing and letting the retry start cleanly is simpler and still recovers on the next attempt.

**Alternatives considered**: Compacting mid-turn too (needs the half turn's tool calls and results in the new context). Retrying more than once (CMP-5 says once).

### R11. Settings

**Decision**:

- `settings.json`: `context.budgetTokens` (exists, default 48,000), new `compaction.model` (absent means the chat model).
- **Range**: the working size is 32,000 to 200,000 tokens, in steps of 1,000. The floor covers the reserve (6,000), the kept turns (8,000), a full summary (8,000 characters, about 3,000 tokens), the system prompt at its file budgets (about 27,000 characters of soul, profile, memory, identity and rules, about 10,000 tokens), the two daily notes (8,000 characters, about 3,000 tokens) and the tools (about 1,500 tokens): about 31,500. The ceiling is the smallest context window among the models the picker can offer.
- **Compaction model**: chosen from the cached model list, or "same as chat". If the chosen model is no longer in the list when a compaction runs, the chat model is used and `compaction_settings` reports `modelMissing` (FR-027).
- **Fixed values** (constants in `compaction/mod.rs`, not settings): reserve 6,000; kept turns 8,000; summary 8,000 characters; flush on.

**Rationale**: CMP-8 names only the budget and the model. A fixed ceiling avoids depending on a context-window field the Models API may not report for every model, and every model in the picker today accepts at least 200,000 tokens. The floor keeps a compaction from being due again right after it finished.

**Alternatives considered**: Exposing the reserve and kept size (more settings, no user benefit). Reading each model's window from the Models API (not in the cached model info of the onboarding feature; a later change if a smaller model ever appears).

### R12. Changing the chat model

**Decision**: "Switch now" in settings calls `compact_conversation(manual: true)`. The new context opens on the newly chosen chat model, with the summary and the kept turns. If there is nothing to compact yet, it falls back to starting a fresh context, as the chat feature did, since there are no older turns to lose. The fresh-context action stays where the chat and memory features offer it: after forgetting something that is still in view, for a context from before memory, and when compaction has failed with the context full.

**Rationale**: The architecture already says "Switch now triggers a compaction" (section 5.1). A new model can't read the old context's thinking blocks anyway, and R8's text-only kept turns are exactly what a new model can read.

**Alternatives considered**: Keeping the fresh context for model switches (loses the recent turns for no reason now that compaction exists).

### R13. Forgotten messages

**Decision**: A turn whose user message is listed in a `forgotten` entry is left out of the summary input (R7) and out of the kept turns (R8). The flush is skipped while the current context, its kept turns or its summary still holds a forgotten message (R4). `record_forgotten`'s check "is the forgotten text still in what the companion sees" also looks at the kept turns and the current summary, so the app still offers a fresh context when needed (memory FR-027).

**Rationale**: Principle IV: forget removes content from everything the companion works with going forward, and a compaction must not carry it into a summary or put it back through a note. Leaving out the whole turn, not just the user's message, keeps the companion's own echo of the fact out too.

**Alternatives considered**: Asking the summarizer to leave the fact out (unreliable, and the fact would still be in its input). Redacting the conversation file (out of scope, the memory feature).

### R14. Not using the API's server-side compaction

**Decision**: Compaction is done by the app, as the architecture designs it. The API's own compaction (a beta that summarizes server-side and returns a compaction block) is not used.

**Rationale**: The summary must be an entry in the conversation file the user owns (Principle I, CMP-3) and must follow CMP-4's structure; the flush must run first and write daily notes; forgotten turns must be kept out; and the compaction model may differ from the chat model. Beta features are also confined to the client and avoided where a plain request does the job (architecture section 11).

**Alternatives considered**: Server-side compaction with custom instructions (its summary lives in the API's response, not in a structure the app checks; no flush; no control over forgotten turns).

### R15. What the user sees

**Decision**:

- `MessageKind` gains `Compacted`. `visible()` puts it just before the first kept message, with the summary as its text. A `context` entry that names a compaction adds no plain divider of its own.
- `ChatState` gains `compactionDue` and `contextFull`. `budgetReached` is removed; its offer of a fresh context becomes the `contextFull` offer (FR-004, FR-022).
- `ChatEvent` gains `Compacting`, sent before an inline compaction (R10).
- `compact_conversation` returns a `CompactOutcome`: `Compacted`, `NothingToCompact`, `Stopped` (a message came in), or `Failed` with the provider error if there was one, plus the new `ChatState`.
- The status line ("Tidying up the older part of our conversation…") shows while `compact_conversation` runs or after a `Compacting` event, as a polite live region. The composer stays enabled.

**Rationale**: The divider belongs where the summarized part ends, which is before the kept turns, not at the end of the file where the entries are written. The rest follows the existing patterns of the chat feature.

### R16. Thinking-binding behavior

**Decision**: This slice does not yet set the API's thinking-binding behavior explicitly (architecture section 7.3). No request ever edits history: each context is append-only (SC-007, tested), and a new context starts with no thinking blocks before its own turns (R8).

**Rationale**: The setting is a beta field with its own header, and setting `thinking` at all changes requests for models that run without thinking. It belongs with a change to the Claude client that records new fixtures (the memory feature has one open). The append-only test is the check that matters, and it runs on every commit.

**Alternatives considered**: Setting `prefix_mismatch_behavior: "error"` now (requires recorded fixtures for every model shape before it can be tested).

### R17. Testing

**Decision**:

- **Unit** (`compaction/cut.rs`, `summary.rs`, `flush.rs`, `chat/context.rs`): table-driven cases for the cut (usage-based tails, the latest message always kept, tool rounds never split, replaced and forgotten turns, nothing to compact); summary parsing, checks, rendering and the size limit; flush note acceptance and de-duplication; text-only replay of kept turns.
- **Integration** (`tests/compaction_flow.rs`, temporary workspace, fake provider, real files): a conversation driven past the working size compacts and continues; the flush's notes land once with provenance; each interruption point (a message during the flush, during the summary, a provider failure, a summary that fails twice, a stopped reply) leaves the conversation file and the daily note byte for byte as before (SC-005); crash recovery with a `pending` marker; overflow compacts inline and retries once; forgotten turns are left out and the flush skipped; "switch now" opens the new model's context; settings range and fallback.
- **Snapshot**: `tests/chat_request.rs` checks that every request extends the previous one within each context, across two compactions (SC-007).
- **Contract**: `tests/claude_contract.rs` checks the request body with `output_config.format` against the documented shape.
- **Webview**: Vitest for the divider opening to show the summary, the status line, automatic compaction after `Finished` when due, compact now with a focus note, and the settings section. One end-to-end spec on the replay server.
- **Release checks** (owner, real provider): the scripted long-conversation suite for SC-001 to SC-003 and SC-009.

**Rationale**: The principles' compaction gate asks for tests of the cut, the kept turns and summary validation; the rest proves each requirement once at the cheapest boundary (test-audit skill). Quality of summaries can only be measured against the real model, which CI never calls.

## Data

What this slice adds to the conversation file, the settings and state files, the daily notes, and the types that cross to the webview. Everything else is as the chat and memory pages define it.

### Data 1. Conversation file entries (`sessions/main.jsonl`)

#### `compaction` (new)

Written once per completed compaction, immediately followed by its `context` entry in the same write (R5). A `compaction` line with no `context` entry naming it after it is ignored.

```json
{"id":"1794600000000-0","at":1794600000000,"type":"compaction",
 "summary":"## Where we are\n- Planning the Lisbon trip; Sam wants three restaurant ideas.\n\n## Open threads\n- Suggest three restaurants near Alfama.\n\n## Between us\nWarm and joking; Sam is a little stressed about the early flight.\n\n## Facts\n- Flight TP1350 on 14 March at 07:10.\n\n## More in the daily notes\n- memory/2026-11-13.md",
 "firstKept":"1794599000000-0",
 "tokensBefore":43120,"keptTokens":7480,
 "model":"claude-opus-5-5",
 "usage":{"flush":{"input_tokens":312,"cache_read_input_tokens":42100,"output_tokens":180},
          "summary":{"input_tokens":21400,"output_tokens":620}},
 "flush":{"outcome":"saved","notes":[{"time":"21:40","text":"Rina visits next Tuesday."}]},
 "focus":"keep the formula we settled on"}
```

| Field | Type | Rules |
|---|---|---|
| `summary` | string | The rendered summary (R9): Markdown, non-empty, at most 8,000 characters (FR-017). |
| `firstKept` | string | The id of the first kept message (a user message). Every kept turn is at or after it. |
| `tokensBefore` | integer | The context's size when the compaction started (R1). |
| `keptTokens` | integer | The measured size of the kept turns (R6). The size after the compaction is reported by the next reply's usage, which is in the file anyway. |
| `model` | string | The model that wrote the summary. |
| `usage` | object | `flush` and `summary`: the provider's usage objects as received; either is absent when that call wasn't made. |
| `flush` | object | `outcome`: `saved`, `none`, `skipped` or `failed`. `notes`: the notes saved, each `{time: "HH:MM", text}`; empty unless `saved`. |
| `focus` | string, optional | The user's focus note for "compact now", up to 500 characters, user provenance. |

#### `context` (changed)

Gains an optional `compaction` field: the id of the `compaction` entry that opened this context. Absent for a first context or a fresh one. Its other fields are snapshotted when the compaction commits, as for a fresh context; `notes` already includes the flush's notes (R5).

```json
{"id":"1794600000000-1","at":1794600000000,"type":"context","model":"claude-opus-5-5",
 "files":{"IDENTITY.md":"…","SOUL.md":"…","USER.md":"…","MEMORY.md":"…"},
 "harness":"memory-1","notes":{"2026-11-13":"…","2026-11-14":"…"},
 "compaction":"1794600000000-0"}
```

#### Which messages a context holds

For a context with a `compaction` field (R8, R9):

1. The opener: the summary, then the daily notes.
2. The kept turns: every message from `firstKept` up to its `compaction` entry, minus replaced turns, forgotten turns, tool-result messages, thinking and tool-call blocks; each turn's reply text merged into one assistant message.
3. Every message after the `context` entry, replayed as the chat feature defines.

A second compaction's `firstKept` can point into the previous context's kept turns; the rule is the same.

### Data 2. Daily notes

Flush notes use the memory feature's line format, with the compaction's local time and the id of the latest user message in the compacted context as their source:

```markdown
- 21:40 Rina visits next Tuesday. <!-- from:companion src:1794599000000-0 -->
```

Rules: only appended; a note whose text matches an existing line's text, or an earlier note of the same flush, is not written (FR-010); at most 20 per flush; one line each, at most 1,000 characters (the memory feature's `memory_note` rules).

### Data 3. `settings.json`

```json
{"chat":{"model":"claude-opus-5-5"},
 "context":{"budgetTokens":48000},
 "compaction":{"model":"claude-haiku-5-5"},
 "recall":{"auto":true}}
```

| Field | Rules |
|---|---|
| `context.budgetTokens` | Exists. 32,000 to 200,000, a multiple of 1,000 (R11). A stored value out of range is read as the nearest end of the range. |
| `compaction.model` | New, optional. A model id; absent or `null` means the chat model. If it is not in the cached model list when a compaction runs, the chat model is used. |

### Data 4. `state.json`

```json
{"compaction":{"pending":"1794600000000-0"}}
```

| Field | Rules |
|---|---|
| `compaction.pending` | New, optional. Set just before the commit writes the conversation file; cleared after the flush's notes are in the daily note. Present at launch means a crash happened in between: the notes are finished if the entries are on disk, and the marker is cleared either way (R5). |

### Data 5. Constants (core, not settings)

| Name | Value | Source |
|---|---|---|
| `RESERVE_TOKENS` | 6,000 | architecture section 15 |
| `KEEP_RECENT_TOKENS` | 8,000 | architecture section 15 |
| `MAX_SUMMARY_CHARS` | 8,000 | architecture section 15 |
| `MAX_FLUSH_NOTES` | 20 | R4 |
| `MAX_FOCUS_CHARS` | 500 | this plan |
| Budget range | 32,000 to 200,000, step 1,000 | R11 |

### Data 6. Types that cross to the webview (generated with `ts-rs`)

```rust
pub enum MessageKind { User, Companion, Divider, Note, Compacted }   // + Compacted: text is the summary

pub struct ChatState {
    // … as before, without budget_reached …
    pub compaction_due: bool,   // R1: the webview should call compact_conversation
    pub context_full: bool,     // the provider still says too long: offer retry or a fresh context
}

pub enum ChatEvent {
    // … as before …
    Compacting,                 // an inline compaction started (overflow, R10)
}

#[serde(tag = "kind")]
pub enum CompactOutcome {
    Compacted,
    NothingToCompact,
    Stopped,                    // a message came in; nothing was saved
    Failed { error: Option<ProviderError> },   // None: the summary failed its checks twice
}

pub struct CompactResult {
    pub outcome: CompactOutcome,
    pub state: ChatState,
}

pub struct CompactionSettings {
    pub budget_tokens: u64,
    pub min_budget: u64,        // 32_000
    pub max_budget: u64,        // 200_000
    pub budget_step: u64,       // 1_000
    pub model: Option<String>,  // None: same as the chat model
    pub model_missing: bool,    // the chosen model is no longer offered; the chat model is used
}
```

### Data 7. Internal types (core only)

```rust
// compaction/summary.rs
struct Summary { thread: String, commitments: Vec<String>, relationship: String, facts: Vec<String>, notes: Vec<String> }

// compaction/cut.rs
struct Cut { first_kept: usize /* index into the context's messages */, kept_tokens: u64 }

// compaction/flush.rs
// chat/transcript.rs, as the conversation file holds them
struct FlushNote { time: String, text: String }
enum FlushOutcome { Saved, Nothing /* "none" */, Skipped, Failed }

// compaction/flush.rs
fn accepted(content: &[Value], today: &str) -> Vec<String>   // the notes a flush reply asks for
```

### Data 8. State transitions

```text
idle ──reply finishes, size ≥ budget − reserve──▶ due
due ──webview calls compact_conversation──▶ compacting
compacting ──flush (optional) → summary ok → commit──▶ idle (new context)
compacting ──message sent / line appended / reply running at commit──▶ due (stopped, nothing saved)
compacting ──summary fails twice / provider error──▶ due (failed, nothing saved; retried after the next reply)
compacting ──nothing before the cut──▶ idle (nothing to compact)
reply ──ContextTooLong on the turn's first call──▶ compacting (inline, no yielding) ──▶ retry once
```

## Interfaces

### Compaction Commands Between the Webview and the Core

These commands extend the surfaces of the onboarding, chat, memory and world pages. The same rules apply: thin handlers, validation in the core, expected failures as values, each command declared in `build.rs` and allowed in `src-tauri/capabilities/main.json`, and `src/api/commands.ts` as the only caller. The only network requests are the flush and the summary, sent to the Claude API by the core (see Interfaces, the flush and summary requests).

#### New commands

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `compact_conversation` | `clock: LocalClock`, `focus: string \| null`, `manual: boolean` | `CompactResult` | Async. `NoKey` is reported as `Failed { error: KeyRejected }` without a request. Refused as `InvalidInput("busy")` while a reply or another compaction runs. `manual: false` does nothing (`NothingToCompact`) unless the core finds the compaction due (R1). `focus` is accepted only with `manual: true`, trimmed, at most 500 characters (`InvalidInput("focus")`). Runs the flush and the summary, then commits (R3 to R5). A message sent meanwhile makes it return `Stopped` with nothing saved. A failure saves nothing. Never shows an error in the conversation itself. | FR-002, FR-005 to FR-021, FR-023, FR-024, FR-027, FR-028 |
| `compaction_settings` | — | `CompactionSettings` | Reads `settings.json` and the cached model list. No network request. | FR-003, FR-027 |
| `set_compaction_settings` | `budgetTokens: number`, `model: string \| null` | `CompactionSettings` | `budgetTokens` must be 32,000 to 200,000 and a multiple of 1,000 (`InvalidInput("budgetTokens")`). `model` must be `null` or an id in the cached model list (`InvalidInput("model")`). Writes `settings.json` atomically. The next compaction uses the new values. | FR-003, FR-027 |

#### Changes to earlier commands

| Command | Change |
|---|---|
| `chat_state` | `ChatState` gains `compactionDue` and `contextFull`; `budgetReached` is removed. `compactionDue` is false while a reply or a compaction runs and while a reply is owed. `contextFull` is true after a turn failed with `ContextTooLong` despite the inline compaction. |
| `chat_history` | Messages gain the kind `Compacted`, placed just before the first kept message, with the summary as `text`. A `context` entry that names a compaction adds no `Divider`. |
| `send_message`, `retry_reply`, `regenerate_reply`, `start_first_hello` | Starting any of them asks a running compaction to stop (R3); they never wait for it. On `ContextTooLong` for a turn's first call they send a `Compacting` event, compact inline and retry once (R10). The events otherwise stay as they are. |
| `start_fresh_context` | Unchanged. Still offered after forgetting something in view, for a context from before memory, and when `contextFull` is true. Clears `contextFull`. |
| `record_forgotten` path (`memory_forget`) | The "still in what the companion sees" answer also checks the kept turns and the current summary (R13). |

#### Webview duties

- After every `Finished` event whose `state.compactionDue` is true, and once when the conversation view loads if `chatState().compactionDue` is true, call `compact_conversation(clock, null, false)`. Show the status line while it runs. The composer stays enabled; sending during it is allowed and stops it.
- After a `Compacting` event, show the status line until the turn's `Finished` or `Failed`.
- "Compact now" opens a small dialog with an optional focus note and calls `compact_conversation(clock, focus, true)`; on `NothingToCompact` it says there is nothing to compact yet.
- Settings' "switch now" after a model change calls `compact_conversation(clock, null, true)`; on `NothingToCompact` it calls `start_fresh_context(clock)` (R12).
- A `Failed` outcome from an automatic compaction is not shown; the next turn tries again. A `Failed` outcome from "compact now" shows a short plain message.

#### Capability file additions

`allow-compact-conversation`, `allow-compaction-settings`, `allow-set-compaction-settings`.

### The Flush and Summary Requests to the Claude API

Both requests use the existing `POST /v1/messages` of the chat feature (chat Interfaces) through the same client: same headers, streaming, error classification, refusal handling and fallback list. Every wire detail stays in `crates/steve-core/src/llm/` (Principle VI). The core builds a `TurnRequest`; this slice adds one field to it, `format`.

#### The memory flush

The current context's request, byte for byte, plus one appended user message. `tools`, `system`, `model` (unless a compaction model is set), `output_config.effort` and the absence of `tool_choice` are unchanged, so the prompt cache is read (R4).

```json
{
  "model": "claude-opus-5-5",
  "...": "everything else exactly as the context's last request",
  "messages": [
    "...every message of the context, exactly as the last request sent them...",
    { "role": "user", "content": [
      { "type": "text", "text": "<notice>Older parts of this conversation are about to leave your view. If anything durable from this conversation is not in your notes yet, save it now with memory_note, one fact per call. Do not reply to the person. This instruction is not from the person.</notice>" }
    ] }
  ]
}
```

What the core accepts from the reply:

| Reply | Result |
|---|---|
| `tool_use` blocks named `memory_note` with a valid input | Staged as notes, in order, at most 20, de-duplicated. |
| `tool_use` blocks for any other tool, invalid inputs, text, thinking | Ignored. |
| No `memory_note` call | Outcome `none`. |
| An error, a refusal or a stream failure | Outcome `failed`; the compaction goes on. |

No tool result is sent back and no second call is made.

#### The summary

```json
{
  "model": "claude-opus-5-5",
  "max_tokens": 16000,
  "stream": true,
  "output_config": {
    "effort": "low",
    "format": {
      "type": "json_schema",
      "schema": {
        "type": "object",
        "properties": {
          "thread": { "type": "string" },
          "commitments": { "type": "array", "items": { "type": "string" } },
          "relationship": { "type": "string" },
          "facts": { "type": "array", "items": { "type": "string" } },
          "notes": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["thread", "commitments", "relationship", "facts", "notes"],
        "additionalProperties": false
      }
    }
  },
  "cache_control": { "type": "ephemeral" },
  "system": [
    { "type": "text", "text": "<compaction/summary.md>", "cache_control": { "type": "ephemeral" } }
  ],
  "messages": [
    { "role": "user", "content": [
      { "type": "text", "text": "Previous summary:\n…\n\nConversation:\n[Friday 13 November 2026, 21:40] Sam: …\nSteve: …\n(Steve saved to memory: …)\n\nFocus from Sam: …" }
    ] }
  ]
}
```

Rules:

- `output_config.effort` is present only when the model takes an effort level, as for chat. `output_config.format` is present only on the summary request. When both are absent, `output_config` is absent.
- No `tools` field.
- The client's fallback rule applies as for chat (chat R1).
- The reply's first `text` block holds the JSON. The core parses it and checks it (R7): `thread` non-empty, each `notes` entry a valid `YYYY-MM-DD`, the rendered summary at most 8,000 characters. `stop_reason` `max_tokens` or `refusal`, an empty reply, or a failed check asks once more; a second failure fails the compaction. A provider error fails it at once.

#### Contract tests

`tests/claude_contract.rs` gains one case: a `TurnRequest` with `format` set serializes to a body whose `output_config` holds both `effort` and `format` as above, and a recorded structured-output reply parses to its text block. The existing fixtures keep passing unchanged, since `format` is absent from chat requests.

## Checks


### Prerequisites

- Onboarding, chat, memory and world set up, or a clean machine to run the wizard.
- Rust 1.98 or later, Node.js 22 or later, pnpm 10.
- A Claude API key for the manual scenarios and the release checks. The automated checks never use one.
- To reach the working size quickly in manual runs: set the working size to its floor, 32,000 tokens, in settings, and paste long text (up to 32,000 characters per message).

### Inspect the files

- `%APPDATA%\mySteve\workspace\sessions\main.jsonl`: after a compaction, a `compaction` entry followed by a `context` entry naming it (Data 1). Every earlier line is unchanged.
- `%APPDATA%\mySteve\workspace\memory\YYYY-MM-DD.md`: flush notes as ordinary `from:companion` lines.
- `%APPDATA%\mySteve\settings.json`: `context.budgetTokens`, `compaction.model`.
- `%APPDATA%\mySteve\state\state.json`: no `compaction.pending` when the app is idle.

### Manual validation scenarios

| # | Story | Steps | Expected |
|---|---|---|---|
| 1 | US1 | With the working size at 32,000, talk about a trip: a city, a flight number, a date and time, and ask the companion to promise three restaurant ideas later. Then paste long text until a compaction happens. | No warning or error. A thin divider appears; a short status line shows while it runs. |
| 2 | US1 | Right after scenario 1, ask "so, where were we?" | The companion continues the trip, still owes the three ideas, and quotes the flight number, date and time exactly. |
| 3 | US1 | Scroll up past the divider. | Every earlier message is there. |
| 4 | US1 | Open the divider. | The summary: where we are, open threads, between us, facts, and pointers to daily notes. |
| 5 | US1 | Make a compaction due, then send a message immediately while the status line shows. | The reply streams at once; the status line goes away; a compaction happens after that reply instead. |
| 6 | US1 | Edit USER.md in the memory browser, then trigger a compaction. | The next reply uses the edited profile; no "USER.md updated" note follows. |
| 7 | US2 | Mention a durable fact the companion doesn't note right away (for example "my sister Rina visits next Tuesday"), then push to a compaction. | Today's daily note gains a line for it with `from:companion`; nothing about it appears in the conversation. |
| 8 | US2 | Push to a second compaction without mentioning it again. | The same line is not written twice. |
| 9 | US2 | Turn the network off just as a compaction starts. | No change to the conversation file or daily notes; the next reply after the network returns compacts. |
| 10 | US3 | Set the working size to 200,000 on a model with a 200,000-token window and paste until the provider reports the conversation is too long. | The status line shows, the reply arrives after the compaction, and no error is shown. |
| 11 | US4 | Choose "compact now" with the focus note "keep the formula we settled on". | A divider; the summary mentions the formula. |
| 12 | US4 | Choose "compact now" right after a compaction. | "Nothing to compact yet"; nothing changes. |
| 13 | US4 | Start a reply and look for "compact now". | Not available until the reply finishes. |
| 14 | US5 | Set the compaction model to a cheaper model, trigger a compaction. | The `compaction` entry's `model` is the cheaper one; chat replies keep the chat model. |
| 15 | US5 | Change the chat model and choose "switch now". | A divider; the next reply's `model` is the new model, and it continues the topic. |
| 16 | US5 | Pick a compaction model, then remove it from the cached list (or wait for it to be withdrawn) and open settings. | Settings say the chat model is used instead; compaction still works. |
| 17 | Edge | Forget a memory that came from an early message, then trigger a compaction. | The summary doesn't mention it; no flush note brings it back. |
| 18 | Edge | Kill the app (Task Manager) during a compaction. Relaunch. | The conversation and notes are as before the compaction, or the compaction is complete with its notes; never half of it. |
| 19 | Edge | Continue a conversation started before memory existed (a `chat-1` context) to a compaction. | The next context has memory tools; no flush ran for the old one. |

### Release checks (real provider, owner)

| Check | How | Pass |
|---|---|---|
| SC-001 | The scripted long-conversation suite: at least 10 conversations, each passing the working size three times, with planted topics, commitments and questions. | At least 95% of turns right after a compaction continue correctly; 0 user-visible errors. |
| SC-002 | Same suite: compare planted names, dates, numbers and paths with each summary. | 100% exact. |
| SC-003 | Same suite: durable facts planted and not noted before the compaction. | At least 90% in today's note after it. |
| SC-009 | Time each compaction in the suite. | 95% within 30 seconds. |
