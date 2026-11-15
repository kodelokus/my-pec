# Chat runtime

Built.

## User stories

### US1. Talk with the companion in character (P1)

A user who has finished setup types a message in the main window and presses Enter. The companion's reply appears word by word as it is written, in the voice and personality of its soul, addressing the user the way their profile says. The reply is formatted: paragraphs, lists, emphasis and code read cleanly. The user keeps talking, and the companion follows the conversation.

**Why this priority**: Conversation is the product. Every later feature (memory, compaction, dreaming, the avatar) builds on a working turn.

**How to check**: With a completed setup and a valid key, send several messages and confirm each reply streams in, follows the conversation, reflects the soul and the user's profile, and renders formatting.

**Acceptance Scenarios**:

1. **Given** the main window with a stored key, **When** the user types a message and presses Enter, **Then** the message appears in the conversation at once and the companion's reply starts appearing progressively, without waiting for the whole reply.
2. **Given** a soul that describes a particular voice, **When** the companion replies, **Then** the reply follows that voice, uses the companion's name from the identity file, and calls the user what their profile says.
3. **Given** a profile that names a preferred language, **When** the user writes, **Then** the companion replies in that language unless the user writes in another one.
4. **Given** a reply that contains headings, lists, emphasis, links, tables or code, **When** it is shown, **Then** it is rendered as formatted text. Embedded HTML is shown as text, not executed. Remote images are not loaded.
5. **Given** a conversation in progress, **When** the user refers to something said a few turns earlier, **Then** the companion answers with that earlier context.
6. **Given** the user asks what day or time it is, **When** the companion replies, **Then** it states the user's current local date and time correctly.
7. **Given** the user asks whether they are talking to a person, **When** the companion replies, **Then** it says honestly that it is an AI, in character.
8. **Given** the composer, **When** the user presses Shift+Enter, **Then** a new line is inserted instead of sending.

### US2. The conversation survives restarts (P1)

The user talks with the companion, quits the app from the tray, and opens it the next day. The whole conversation is still there, in order, and they can scroll back to the very first message. When they send a new message, the companion continues from where they left off.

**Why this priority**: The second product promise ("Steve remembers") starts with never losing the conversation. A chat that forgets on restart is not a companion.

**How to check**: Exchange a few messages, quit the app, relaunch it, and confirm the full conversation is shown and the companion continues with that context.

**Acceptance Scenarios**:

1. **Given** a conversation with several exchanges, **When** the user quits and relaunches the app, **Then** the main window shows the whole conversation in order.
2. **Given** a relaunched app, **When** the user sends a message that refers to the previous session, **Then** the companion answers with that context.
3. **Given** a long history, **When** the user scrolls up, **Then** they can reach the very first message, including messages the companion no longer has in its working context.
4. **Given** the app is forcibly closed while a reply is being written, **When** it is relaunched, **Then** the user's last message is still shown, and the app offers to retry the reply that was interrupted.
5. **Given** the conversation is stored on disk, **When** the user opens the companion's folder, **Then** the conversation is in a readable file there and nowhere else.

### US3. The first hello (P2)

When the user finishes the setup wizard and the main window opens for the first time, the companion greets them in character before they type anything. The greeting uses the companion's name and, if the user gave one, the name they want to be called.

**Why this priority**: It completes the first-run journey (ONB-4) that slice 1 left pending, and it is the user's first proof that the soul works. It depends on story 1.

**How to check**: Finish setup on a clean install and confirm a greeting appears in character without any input, exactly once.

**Acceptance Scenarios**:

1. **Given** setup has just completed and a first hello is pending, **When** the main window opens, **Then** the companion's greeting streams into the conversation without the user typing anything.
2. **Given** the user entered what they want to be called, **When** the greeting is written, **Then** it uses that name.
3. **Given** the greeting was shown, **When** the app is relaunched, **Then** no second greeting is sent.
4. **Given** no connection when the main window first opens, **When** the connection returns and the user presses Retry, **Then** the greeting is sent then. The pending greeting is not lost.
5. **Given** the greeting has not arrived yet, **When** the user types and sends a message first, **Then** the companion answers that message, and no separate greeting follows.

### US4. Stop and regenerate (P2)

The companion starts a long reply the user doesn't need, and they stop it halfway. Later, a reply misses the point, and they ask for another version of it.

**Why this priority**: It gives the user control over cost and quality. Chat works without it, so it follows stories 1 and 2.

**How to check**: Stop a reply mid-stream and confirm it ends at once and stays visible as stopped; then regenerate the last reply and confirm a new version replaces it in the conversation.

**Acceptance Scenarios**:

1. **Given** a reply is being written, **When** the user presses Stop (or Escape in the composer), **Then** the reply stops within a second, the text written so far stays visible marked as stopped, and the user can send a new message.
2. **Given** the last message in the conversation is a reply from the companion, **When** the user chooses Regenerate, **Then** a new reply to the same message is written and shown in place of the previous one.
3. **Given** a reply was regenerated, **When** the user continues the conversation, **Then** the companion builds on the new version, not the replaced one.
4. **Given** the user has sent another message after a reply, **When** they look at that earlier reply, **Then** Regenerate is not offered for it.

### US5. Problems are explained and nothing typed is lost (P2)

The user sends a message while their Wi-Fi is down. The app says plainly that Claude can't be reached, keeps the message, and offers Retry. Later the service is busy, and the app waits briefly and retries on its own before asking the user.

**Why this priority**: Errors happen in every real session. Losing a heartfelt message to a network blip would break trust in the companion.

**How to check**: Send a message with the network disconnected, confirm the message is kept and a plain explanation with Retry appears, reconnect, press Retry and confirm the reply arrives.

**Acceptance Scenarios**:

1. **Given** no connection, **When** the user sends a message, **Then** the message stays in the conversation, the app explains that Claude can't be reached right now, and offers Retry.
2. **Given** the provider is rate-limiting or overloaded, **When** a reply is requested, **Then** the app waits and retries automatically for a short while, shows that it is waiting, and only then asks the user to retry.
3. **Given** the stored key is no longer accepted or the account has no credit, **When** a reply is requested, **Then** the app says which of the two it is, keeps the message, and offers to enter another key. Once a working key is stored, the user can retry the message.
4. **Given** the provider declines to answer a message, **When** that happens, **Then** the app says the companion couldn't answer that message, in plain words, and the conversation stays usable.
5. **Given** a reply fails after part of it was shown, **When** the failure is reported, **Then** the part already shown stays visible and is marked as incomplete, and Retry asks for a complete reply.
6. **Given** any failure, **When** the user looks at the composer, **Then** nothing they had typed but not yet sent has been cleared.

### US6. Soul and profile edits take effect (P3)

The user opens `SOUL.md` in a text editor and makes the companion more playful, then returns to the chat. From the next message on, the companion sounds more playful, without restarting the app or losing the conversation.

**Why this priority**: SOUL-3 requires an edit to take effect from the next message. In this slice edits are made outside the app; an in-app editor comes with the memory browser.

**How to check**: Mid-conversation, change the soul in a text editor, send a message, and confirm the reply reflects the change while the earlier conversation is kept.

**Acceptance Scenarios**:

1. **Given** a conversation in progress, **When** the user edits the soul, identity, profile or long-term memory file in a text editor and then sends a message, **Then** the companion's reply reflects the edited content.
2. **Given** such an edit, **When** the reply is written, **Then** the earlier conversation is kept and the companion still knows what was said.
3. **Given** an edited file, **When** the companion replies, **Then** the app has not changed the file.

## Edge cases

- **No key is stored.** The conversation history is shown and can be scrolled. The composer explains that conversation needs an API key and offers to add one. The first hello waits until a key is stored.
- **The app starts offline.** The history is shown. Sending is possible, and the message is kept with Retry, as in story 5.
- **The conversation reaches its context budget.** Until compaction arrives in a later feature, the app tells the user that the conversation has reached its working size and offers to start a fresh context. The full history stays visible and on disk. The fresh context starts with the soul, identity, profile and long-term memory and none of the earlier turns. The companion's earlier turns are not trimmed silently.
- **The provider reports the conversation is too long** before the budget is reached. The app treats this as reaching the context budget.
- **The user changes the chat model in settings mid-conversation.** The change takes effect when the next fresh context starts. Settings says so and offers to start a fresh context now.
- **The user sends an empty or whitespace-only message.** Nothing is sent.
- **The user pastes a very long message.** It is sent as written, up to a limit stated in plain words in the composer when it is exceeded.
- **The user quits from the tray while a reply is being written.** The reply stops, the text written so far is kept as stopped, and the app quits.
- **The user closes the window (to the tray) while a reply is being written.** The reply keeps being written and is complete when the window is reopened.
- **A second message while a reply is being written.** The user can type, but sending waits until the reply finishes or is stopped.
- **The transcript file was edited or damaged outside the app.** Lines that can't be read are skipped and kept on disk unchanged. The app says how many were skipped and never rewrites the file.
- **The provider sends content the app doesn't recognize** (for example a new kind of block). It is saved exactly as received, sent back in later turns, and not shown. If the app could not put a block together from its streamed pieces, the block is saved but not sent back.
- **A workspace file loaded into context is over its size budget.** The file is left whole on disk, only the budgeted part is used, and the app tells the user which file was shortened.
- **The clock or time zone changes** (travel, daylight saving). Each message carries the time at which it was sent.
- **A link in a reply is clicked.** It opens in the user's default browser. Links never open on their own.

## Requirements

**Conversation**

- **FR-001**: The main window MUST show one continuous conversation with the companion, with no threads, sessions or conversation list for the user to manage.
- **FR-002**: Users MUST be able to send a message from a composer at the bottom of the conversation. Enter MUST send and Shift+Enter MUST insert a new line. Empty and whitespace-only messages MUST NOT be sent.
- **FR-003**: The companion's reply MUST appear progressively as it is generated. The first words MUST be shown as soon as they arrive, without waiting for the reply to finish.
- **FR-004**: Replies MUST be rendered as formatted text (paragraphs, headings, lists, emphasis, links, tables, quotes and code). The rendering MUST be sanitized: embedded HTML is shown as text, scripts never run, and remote images are not loaded. Links MUST open in the user's default browser only when clicked.
- **FR-005**: While a reply is being written, the composer MUST stay editable, sending MUST wait until the reply ends, and the conversation MUST show that the companion is writing.

**In character and in context**

- **FR-006**: Every request MUST give the companion its harness rules, its identity, its soul, the user's profile and its long-term memory, so that replies follow the soul's voice and values, use the companion's name and address the user as their profile says.
- **FR-007**: The companion MUST know the user's current local date, time and time zone for every message the user sends.
- **FR-008**: The companion MUST be told by its harness rules to be honest about being an AI and MUST NOT use engagement tricks (NFR-SEC-6).
- **FR-009**: Each request in a conversation MUST include the conversation so far within the current context, so the companion follows what was said earlier, including across restarts.
- **FR-010**: Within one context, earlier turns MUST be sent unchanged, in the same order, and the companion's previous replies MUST be sent exactly as they were received. New content MUST only be added after them (Principle III).
- **FR-011**: When the soul, identity, profile or long-term memory file changes on disk during a context, the next message MUST carry a notice with the updated content, so the change takes effect from that message. The earlier conversation MUST be kept. The app MUST NOT write to those files.
- **FR-012**: Each workspace file given to the companion MUST respect its size budget. Over budget, the file MUST stay whole on disk, only the budgeted part MUST be used, and the app MUST tell the user which file was shortened.

**Persistence**

- **FR-013**: Every message the user sends MUST be saved to the conversation file in the companion's workspace before it is sent to the provider (Principle V).
- **FR-014**: Every reply MUST be saved to the conversation file exactly as received when it ends, including stopped and incomplete replies, marked as such.
- **FR-015**: The conversation file MUST only ever be appended to. No message may be edited or removed from it by this feature.
- **FR-016**: On launch, the main window MUST show the full conversation from the file, in order, and users MUST be able to scroll back to the first message, including turns no longer in the companion's working context.
- **FR-017**: If the app ended while a reply was being written, the next launch MUST show the user's last message and MUST offer to retry the interrupted reply.
- **FR-018**: Lines of the conversation file that can't be read MUST be skipped without changing the file, and the app MUST say how many were skipped.
- **FR-019**: Content the provider sends that the app does not recognize MUST be saved exactly as received and MUST NOT be shown. It MUST be sent back in later turns exactly as received, except a block the app could not put together from its pieces, which is saved but not sent back.

**First hello**

- **FR-020**: When the main window opens with a first hello pending and a key stored, the companion MUST send a greeting in character without user input. The greeting MUST use the companion's name and, when given, the name the user wants to be called.
- **FR-021**: The first hello MUST be sent at most once. It MUST stay pending until it has been shown in full, and it MUST be cancelled if the user sends a message first.

**Stop and regenerate**

- **FR-022**: Users MUST be able to stop a reply while it is being written, with a Stop button and with Escape in the composer. The reply MUST stop within one second and the text written so far MUST stay visible, marked as stopped.
- **FR-023**: Users MUST be able to regenerate the companion's last reply while it is the last message in the conversation. The new reply MUST answer the same message and MUST take the previous reply's place in the conversation shown and in the companion's context. The replaced reply MUST stay in the conversation file, marked as replaced.

**Errors and recovery**

- **FR-024**: When a reply can't be obtained, the app MUST keep the user's message, explain the problem in plain words and offer Retry. Each of these MUST have its own message: no connection, provider busy or rate-limiting, key no longer accepted, no credit, the provider declined to answer, and the conversation is too long.
- **FR-025**: When the provider is busy or rate-limiting, the app MUST retry automatically with increasing waits, honoring any wait the provider asks for, for up to 60 seconds, and MUST show that it is waiting before asking the user to retry.
- **FR-026**: When the key is no longer accepted or the account has no credit, the app MUST offer to enter another key and MUST let the user retry the kept message once a working key is stored.
- **FR-027**: A reply that fails after part of it was shown MUST keep the shown part visible, marked as incomplete. Retry, automatic or by the user, MUST request a complete reply to the same message, which takes the incomplete reply's place in the conversation shown and in the companion's context, as a regenerated reply does (FR-023).
- **FR-028**: While the app runs, no error, retry, stop or switch to settings and back may clear text the user typed in the composer but has not sent. (Keeping it across a quit is not required; see Assumptions.)
- **FR-029**: Without a stored key, the conversation MUST be shown and scrollable, the composer MUST say that conversation needs an API key, and MUST offer to add one.

**Context size**

- **FR-030**: When the conversation reaches its context budget, or the provider reports it is too long, the app MUST tell the user and MUST offer to start a fresh context. Earlier turns MUST NOT be dropped from the companion's context without the user choosing this.
- **FR-031**: A fresh context MUST start with the harness rules, identity, soul, profile and long-term memory, and none of the earlier turns. The full conversation MUST stay visible and in the conversation file, with a divider marking where the fresh context began.
- **FR-032**: A change of chat model MUST take effect when the next fresh context starts. Settings MUST say so and MUST offer to start a fresh context now.

**Accessibility and privacy**

- **FR-033**: The conversation, composer, Stop, Regenerate and Retry MUST be usable with the keyboard alone. New messages from the companion MUST be announced to screen readers once complete, not word by word.
- **FR-034**: Conversation content MUST be sent only to the configured Claude API. This feature MUST add no other network destination (Principle II).

### Key entities

- **Conversation**: The single, continuous exchange between the user and the companion. Stored as an append-only file in the workspace and never split into threads.
- **Message**: One entry in the conversation, from the user or the companion. It records who wrote it, when (with the local time zone), its content exactly as sent or received, and its state: complete, stopped, incomplete or replaced.
- **Context**: The part of the conversation the companion currently works with. It starts with the companion's harness rules, identity, soul, profile and long-term memory, then grows only by appending messages. A fresh context starts on first run, when the user chooses one, or (later) after compaction.
- **Notice**: An entry added to the conversation by the app rather than by the user or the companion, for example "the soul was edited" or "a fresh context started". Notices are shown as dividers or small notes.
- **First hello**: A pending greeting recorded at the end of setup and cleared once the greeting has been shown in full or the user has spoken first.

## Success criteria

- **SC-001**: A new user sees the companion's first in-character greeting within 3 minutes of launching the installer, having typed nothing but the API key (product SC-1).
- **SC-002**: On a normal home connection, the first words of a reply appear within 5 seconds of sending for at least 90% of messages.
- **SC-003**: The app's own preparation before a message is sent takes no more than 0.15 seconds for 95% of messages, so the wait the user feels is the provider's.
- **SC-004**: After a forced shutdown at a random moment, 100% of messages the user sent are still in the conversation on relaunch.
- **SC-005**: With a history of 10,000 messages, the main window is usable within 3 seconds of launch and scrolling back stays smooth.
- **SC-006**: Stop ends a reply within 1 second in at least 95% of attempts.
- **SC-007**: In a conversation of 20 messages within one context, at least 80% of the input of every turn after the first is billed at the provider's cached rate.
- **SC-008**: In a blind review of 20 replies per bundled soul template, reviewers match at least 80% of replies to the right template.
- **SC-009**: During a capture of a full chat session, the app's own connections go only to the Claude API.

## Assumptions

- **Depends on slice 1.** Setup, the workspace files, the stored key, the model setting and the pending first hello come from onboarding.
- **No memory tools yet.** The companion reads its long-term memory and profile but can't write to them, take notes, search or recall in this slice. Those arrive with the memory core (slice 3). Daily notes are therefore not loaded yet.
- **No compaction yet.** A conversation that reaches its context budget needs the user to start a fresh context (FR-030). Compaction (slice 4) replaces this with automatic summarizing. The default budget is the starting value from the architecture, 48,000 tokens.
- **No avatar states yet.** The portrait from slice 1 stays still. Moods and activities arrive with the avatar (slice 6), so replies carry no mood tag in this slice.
- **Text only.** Image input, a global hotkey, proactive check-ins and voice are out of scope (CHAT-6 to CHAT-9).
- **No usage meter yet.** Token use is not shown in this slice. The usage meter is part of release readiness (slice 8).
- **Stopped replies stay in the context.** A stopped or incomplete reply is part of what the companion sees afterwards, as text, so the companion knows it was cut off.
- **Unsent composer text** is kept while the app runs, including across switches to settings. Keeping it across a quit is not required.
- **Message length limit.** A single message is limited to 32,000 characters, comfortably within the context budget.
- **Default chat model** is the one chosen in settings (default `claude-opus-5-5`), as in onboarding.
- **Network.** The only destination is the Claude API, already used by onboarding for the key check and the model list. Conversation content now goes there too.

## Decisions

Each section records a decision, why it was made, and what was rejected. API facts come from the Claude API reference bundled with Claude Code (cached 2026-10-06) and the recorded model list from onboarding. Facts marked **(record)** are confirmed by contract fixtures recorded from the real API, as was done for the model list.

### R1. The chat request

**Decision**: `POST /v1/messages` with `stream: true`, `max_tokens: 16000`, API version `2026-10-21` (already pinned). The `thinking` field is omitted. `output_config.effort` is `"low"` when the model's capabilities list effort. Sampling parameters are never sent. A model on the server-side fallback list also gets `fallbacks: "default"` and the header `anthropic-beta: server-side-fallback-2026-07-01`.

**Rationale**:

- On `claude-opus-5-5` thinking is always on. Omitting `thinking` runs adaptive thinking, and `{type: "disabled"}` is a 400. On older models in the picker (Opus 4.8, Haiku 4.5) omitting it means no thinking, which is also valid. Omitting it is the one shape every listed model accepts.
- Effort is the only control on Opus 5.5, and its default there is `medium`. The reference recommends `low` for chat and for time to first token (SC-002). Haiku 4.5 rejects `effort`, so the client sends it only when the model's `capabilities.effort.supported` is true. The Models API returns `capabilities`, which onboarding already fetches and caches (see the recorded `ok.json`).
- `max_tokens` must leave room for thinking as well as the reply. 16,000 is ample for chat at low effort and keeps a runaway reply bounded.
- On API version `2026-10-21`, `temperature`, `top_p` and `top_k` are a 400 on every model.
- The reference asks code for `claude-opus-5-5` to opt into the server-side refusal fallback by default. `"default"` routes by refusal category and needs no pinned model. It is accepted only on some models, so `llm/claude.rs` keeps the list (`claude-opus-5-5`, `claude-opus-5`, `claude-sonnet-5-5`, `claude-fable-5`, `claude-fable-5-1`).

**Alternatives considered**: `thinking: {type: "adaptive"}` sent explicitly (rejected by models older than 4.6). A fixed effort for every model (a 400 on Haiku 4.5). No fallback (rejected: the reference makes it the default, and a refused companion reply with no recovery is a worse experience).

### R2. Reading the stream without new dependencies

**Decision**: Read the response body with `reqwest::Response::chunk()` and parse server-sent events with a small parser in `llm/sse.rs`: split on blank lines, keep `event:` and `data:` lines, and carry a partial event across chunks. A chunk that doesn't arrive within 90 seconds counts as a dropped connection.

**Rationale**: `chunk()` is in the `reqwest` version already used, so the `stream` feature and `futures` are not needed. The SSE subset the API uses (named events, single-line JSON `data`) takes about 40 lines. The API sends `ping` events while the model thinks, so a 90-second silence means the connection is gone.

**Alternatives considered**: `reqwest` `stream` feature with `eventsource-stream` (two dependencies for 40 lines). A long overall timeout (would cut off long replies).

### R3. Events and content blocks

**Decision**: The client handles `message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`, `ping` and `error`. Each content block is assembled as raw JSON from its `content_block_start` object:

- a delta whose fields besides `type` are strings is merged by appending each string to the field of the same name (`text_delta.text`, `thinking_delta.thinking`, `signature_delta.signature`);
- `input_json_delta.partial_json` is concatenated and parsed into `input` at `content_block_stop`;
- `citations_delta.citation` is pushed onto `citations`;
- any other delta is kept in the order it arrived under the block's `"_unmerged"` field, and the block is marked so that it is never replayed (it is stored, FR-019).

Text deltas are also passed to the caller as they arrive. `message_delta` carries `stop_reason`, `stop_details` and the final `usage`.

**Rationale**: Principle III and the principles' technology constraints require assistant content to be stored and replayed exactly as received. Merging by field name covers today's string deltas and future ones of the same shape without code changes. A delta the client can't merge would make a replayed block wrong, so such a block is stored but not sent back.

**Alternatives considered**: Typed structs for every block (loses unknown blocks). Storing only the text (breaks thinking replay and prompt caching).

### R4. Stop, one reply at a time, and quitting

**Decision**: The core runs at most one reply. A reply runs in a `tokio::select!` between the provider future and a cancel signal (`tokio::sync::Notify`). Stop fires the signal. The provider future is dropped, which closes the connection, and the text received so far is saved as a stopped reply. A second send while a reply runs is refused with `Busy`. Quitting from the tray stops the reply and waits up to two seconds for it to be saved.

**Rationale**: Checking a flag between chunks can't meet one second (SC-006) while the model is thinking, because no chunk arrives then. Selecting on cancellation stops at once. Tauri already runs a tokio runtime, so the core needs only tokio's `sync`, `time` and `macros` features.

**Alternatives considered**: `tokio-util` `CancellationToken` (a dependency for one `Notify`). An `AtomicBool` checked per chunk (too slow while thinking).

### R5. Streaming to the webview

**Decision**: Commands that start a reply take a Tauri `ipc::Channel<ChatEvent>`. The core reports through a `ChatSink` trait that the shell implements over the channel and tests implement over a `Vec`.

**Rationale**: A channel is ordered, typed and scoped to one call, which an app-wide event is not. The trait keeps the core free of Tauri (Principle VI) and is the test boundary.

**Alternatives considered**: Global `emit` events (unordered across calls, need manual filtering). Polling (latency).

### R6. The conversation file

**Decision**: `workspace/sessions/main.jsonl`, one JSON object per line, appended with `O_APPEND`, then `sync_data`. Entry types: `context` (a fresh context: the model and the snapshot of the files it starts from), `message` (a user or companion message with its content exactly as sent or received), `notice` (a workspace file changed: its new content) and `replaced` (a regenerated reply). Each entry has an id and a timestamp. A line that doesn't parse, including a half-written last line after a crash, is skipped and counted, never rewritten (FR-018).

**Rationale**: This is the transcript format of `docs/architecture.md` section 4.1, plus `context` and `replaced`, which this slice needs. The `context` entry stores the system prompt's inputs, so a context rebuilt after a restart sends exactly the same system prompt even if the files were edited while the app was closed (Principle III). Edits since then arrive as notices (R8). `replaced` keeps the file append-only when a reply is regenerated.

**Alternatives considered**: SQLite (Principle I: the transcript is a workspace file). Rewriting a line in place to mark it replaced (breaks append-only). Re-reading the files at startup for the system prompt (a hand edit while the app was closed would change the cached prefix and invalidate thinking blocks).

### R7. Building the request from the file

**Decision**: The request is rebuilt from the last `context` entry and every entry after it:

- `system`: the harness rules, then `IDENTITY.md` and `SOUL.md`, then `USER.md` and `MEMORY.md`, each in its own text block, from the `context` snapshot. `cache_control: {type: "ephemeral"}` goes on the last system block, and top-level `cache_control` caches the conversation tail.
- user messages: their content as sent.
- companion replies: complete replies as received, minus any `thinking`, `redacted_thinking` and `tool_use` blocks that come before a `fallback` block. Stopped and incomplete replies are replayed as their `text` blocks only, and not at all when they have none. Replaced replies and blocks marked unreplayable (R3) are left out.
- consecutive user messages (after a failed or stopped reply) are sent as they are. The API combines consecutive messages of one role into one turn.

**Rationale**: The same entries always produce the same request, so every request extends the previous one (Principle III, the snapshot test gate). The reference states that removing turns from the middle invalidates later thinking blocks, while dropping the final reply and asking again (regenerate) sends a request equal to the original one. A stopped reply's thinking block has no signature and can't be replayed, so the text-only form is used every time, which keeps it stable. The fallback rule is the reference's rule for echoing a turn after a mid-output fallback.

**Alternatives considered**: Merging consecutive user messages client-side (unnecessary, and it rewrites the last message). Replaying stopped replies whole (an unsigned thinking block is a 400).

### R8. Time, notices and the first hello

**Decision**:

- **Time**: the webview passes a `LocalClock { iso, zone }` (ISO 8601 with offset, and the IANA zone from `Intl`) with each command that sends. The core validates it and adds a first text block to the user message: `<context>Local time: Friday 13 November 2026, 21:40 (Asia/Jakarta, UTC+07:00)</context>`.
- **Notices**: before each send, the core compares `IDENTITY.md`, `SOUL.md`, `USER.md` and `MEMORY.md` with the latest content the context has seen (the `context` snapshot or a later notice). For each changed file it appends a `notice` entry and adds a block to the same user message: `<notice>The user edited SOUL.md. From now on, follow this version:</notice>` followed by the new content.
- **First hello**: a user message with provenance `system`, hidden in the conversation, asking for a greeting in character that uses the companion's name and what the user wants to be called.

**Rationale**: Rust's standard library has no local time zone. The webview's clock is the user's clock, passing it keeps the core free of a time zone dependency, and it is the injected clock the test strategy calls for. Notices go in the user turn rather than a mid-conversation `system` message, because that role is a 400 on some models in the picker (Claude Sonnet 5), and a user-turn block works on all of them. Keeping notices and the hello in the conversation file means a rebuilt request matches the one sent.

**Alternatives considered**: `jiff` or `chrono` in the core (a dependency for one string). Re-snapshotting the system prompt on an edit (breaks the prefix). Mid-conversation `role: "system"` notices (not accepted by every listed model).

### R9. Size budgets for loaded files

**Decision**: `SOUL.md` 8,000 characters, `USER.md` 4,000 and `MEMORY.md` 12,000, as in architecture section 15. `IDENTITY.md` is small by design and has no budget. Over budget, the first N characters are used and the file's name is reported in `ChatState.shortened`.

**Rationale**: FR-012 and MEM-9. Characters, not tokens, so the check is local and deterministic, and it matches the soul budget the wizard shows.

### R10. Context budget

**Decision**: After each reply, the context size is `input_tokens + cache_read_input_tokens + cache_creation_input_tokens + output_tokens` from the reply's usage, stored with the message. When it reaches `context.budgetTokens` (default 48,000, a new setting), `ChatState.budgetReached` is true and the main window offers a fresh context. Sending stays possible. A 400 `invalid_request_error` saying the prompt is too long **(record)** maps to `ContextTooLong`, which shows the same offer.

**Rationale**: The provider's own count is exact (architecture section 11 forbids estimating). Blocking sending at the budget would be stricter than the requirements ask. Opus 5.5's window is 1M tokens, so the budget is a cost guard, not a hard limit.

### R11. Errors and retries

**Decision**: `ProviderError` gains three variants. A failure maps to one of:

| Outcome | From | Behavior |
|---|---|---|
| `Unreachable` | connection failure, timeout, dropped stream | message kept, Retry |
| `Busy` | 429 `rate_limit_error`, 529 `overloaded_error`, 500 `api_error`, an `error` event in the stream | retried automatically, waits 1, 2, 4, 8, 16 seconds or the `retry-after` header, for up to 60 seconds in total, with `ChatEvent::Waiting` each time. Then the message is kept with Retry |
| `KeyRejected` | 401, 403, `authentication_error`, `permission_error` | offer to enter another key |
| `NoCredit` | 402, `billing_error` | offer to enter another key |
| `ContextTooLong` | 400 with "prompt is too long" **(record)** | offer a fresh context |
| `Declined` | `stop_reason: "refusal"` after any fallback | plain message, conversation usable |
| `Invalid` | any other 400, 404, 413 | "something went wrong" with Retry, logged without content |

A failure after partial output first saves the partial as an incomplete entry. Any retry of it, automatic or by the user, then appends `replaced` for that entry before sending, exactly as regenerate does, so the request equals the one that produced the partial and the model never sees the partial and the new reply merged into one turn. `stop_reason: "max_tokens"` saves the reply as incomplete, and Retry treats it the same way.

**Rationale**: The official SDKs retry 408, 409, 429 and 5xx. The project implements the same policy (architecture section 11). Errors are classified by the body's error type first and the status second, as `classify` already does.

### R12. Opening links

**Decision**: A command `open_link(url)` opens `http`, `https` and `mailto` links in the default browser through `tauri-plugin-opener`, called from Rust. The core decides which URLs are allowed. The webview gets no opener permission of its own.

**Rationale**: The webview has no shell access (Principle II), so a link needs a command. An explicit command with a checked scheme is narrower than granting the plugin's URL permission to the webview.

**Alternatives considered**: Links as plain text (poor experience). The opener plugin's JavaScript API with a URL scope (a broader capability than needed).

### R13. Rendering replies

**Decision**: `react-markdown` with `remark-gfm` (tables, strikethrough, task lists). Raw HTML is not enabled, so it renders as text. Custom renderers: `a` calls `open_link` on click, and `img` renders its alt text as a link instead of loading the image. Code blocks are plain monospace, with no syntax highlighting.

**Rationale**: The principles require sanitized Markdown. `react-markdown` builds React elements rather than HTML strings, so there is no `dangerouslySetInnerHTML`. Not loading remote images keeps the content security policy (no remote content) and prevents tracking pixels.

**Alternatives considered**: `marked` with DOMPurify (HTML strings and two libraries). A highlighting library (size, no requirement).

### R14. Long histories

**Decision**: `chat_history(before, limit)` returns pages of 50 messages, newest first. The conversation loads the latest page at launch and the previous page when the user scrolls near the top. The core reads the file once at launch and keeps the parsed entries in memory.

**Rationale**: SC-005 (10,000 messages, usable within 3 seconds). Parsing 10,000 lines takes milliseconds. Rendering 50 Markdown messages at a time is cheap. A virtualized list library isn't needed for this volume.

### R15. Accessibility

**Decision**: The message list is a `log` region. A streaming reply has `aria-busy="true"` and is announced once complete through a polite live region. Stop, Regenerate and Retry are buttons with names. Escape in the composer stops a reply.

**Rationale**: FR-033. Announcing every delta would flood a screen reader.

### R16. Test strategy

**Decision**:

- **Core unit tests**: SSE parser cases (split chunks, multi-line data, ping, error event), block assembly per delta type, request assembly per entry type, time block formatting, budget truncation, error classification table.
- **Request snapshot test**: three turns, a notice, a stopped reply and a regenerate, asserting each request is the previous one plus appended messages (the principles' test gate).
- **Core integration tests**: a `FakeProvider` scripted with events and errors, on a temporary workspace: persist-before-send, stop, crash recovery from a truncated last line, retry after `Busy` with a fake clock for the waits, first hello once, fresh context.
- **Contract tests**: recorded SSE responses for a plain reply, a reply with thinking, a refusal, a prompt-too-long error and an overload error, replayed by `wiremock`. Hand-written from the documented shapes first, then recorded by the owner **(record)**.
- **UI tests**: composer keys, streaming render, Markdown sanitizing, Stop and Regenerate visibility.
- **End-to-end**: the replay server answers `/v1/messages` with recorded SSE.

**Rationale**: The principles' test gates and the `test-audit` skill: each contract is owned once, at its cheapest boundary.

### R17. Deliberately not built in this slice

- Memory tools, daily notes, search and auto-recall (slice 3).
- Compaction and the memory flush (slice 4). Reaching the budget offers a fresh context instead.
- The mood tag and avatar states (slice 6).
- The usage ledger and SQLite (slice 8). Usage is stored with each reply in the conversation file.
- Image input, the global hotkey, check-ins and voice.
- Syntax highlighting and a virtualized list.
- Keeping unsent composer text across a quit.

### Sources

- Claude API reference bundled with Claude Code 2.1.309: `curl/examples.md` (streaming, caching, thinking), `shared/model-migration.md` (Opus 5.5 thinking and effort, refusals and fallbacks, preserved thinking), `shared/error-codes.md`.
- `docs/architecture.md` sections 4, 5, 11, 12, 13 and 15.
- `crates/steve-core/tests/fixtures/models/ok.json` (recorded 2026-11-13) for model capabilities.

## Data

The onboarding page's files and types are unchanged except where noted. Rust definitions are the source; the TypeScript types in `src/bindings/` are generated from them.

### Data 1. The conversation file

`workspace/sessions/main.jsonl`. Created on the first entry. One JSON object per line, appended only (FR-015). Every entry has `type`, `id` (a sortable id: Unix milliseconds plus a counter, e.g. `"1794583200123-0"`) and `at` (Unix milliseconds).

#### `context` — a fresh context begins

| Field | Type | Meaning |
|---|---|---|
| `model` | string | The chat model for this context (FR-032). |
| `files` | object | `{ "IDENTITY.md": text, "SOUL.md": text, "USER.md": text, "MEMORY.md": text }`, as read when the context started, before size budgets are applied. |
| `harness` | string | Version tag of the harness rules (`"chat-1"`), so a later app version can tell which rules a context was built with. |

The first `context` entry is written before the first message. A later one is written when the user starts a fresh context (FR-031). The conversation view shows it as a divider, except the first.

#### `message` — a user or companion message

| Field | Type | Meaning |
|---|---|---|
| `role` | `"user"` \| `"assistant"` | Who wrote it. |
| `provenance` | `"user"` \| `"companion"` \| `"system"` | `system` for the first-hello request (hidden). Matches the architecture's provenance tags. |
| `content` | array | The content blocks exactly as sent (user) or received (assistant), as raw JSON (R3). |
| `text` | string | User messages only: what the user typed, for display. |
| `zone` | string | User messages only: the IANA time zone the message was sent in. |
| `state` | `"complete"` \| `"stopped"` \| `"incomplete"` | Assistant only. `stopped`: the user stopped it (FR-022). `incomplete`: a failure or `max_tokens` cut it off (FR-027). |
| `stop_reason` | string \| null | Assistant only, as received. |
| `model` | string | Assistant only: the model that produced it (it can differ after a fallback). |
| `usage` | object \| null | Assistant only: the `usage` object as received (R10). |

User messages are written before the request is sent (FR-013). Assistant messages are written once, when the reply ends or stops.

#### `notice` — a workspace file changed during a context

| Field | Type | Meaning |
|---|---|---|
| `file` | string | `IDENTITY.md`, `SOUL.md`, `USER.md` or `MEMORY.md`. |
| `content` | string | The file's new content. |
| `message` | string | The id of the user message that carried the notice to the companion. |

#### `replaced` — a regenerated reply

| Field | Type | Meaning |
|---|---|---|
| `target` | string | The id of the assistant message that was replaced (FR-023). |

**Validation on read**: a line that is not valid JSON, has an unknown `type`, or lacks a required field is skipped and counted (FR-018). Unknown extra fields are kept in memory and ignored.

### Data 2. Settings and state

`state/settings.json` gains one section. A missing section means the default.

```json
{
  "chat": { "model": "claude-opus-5-5" },
  "context": { "budgetTokens": 48000 }
}
```

`state/state.json` is unchanged. `firstHelloPending` (from the onboarding page) is cleared when the greeting is saved complete, or when the user sends a message while it is still pending (FR-021).

The model cache in `state.json` gains `supportsEffort` per model (R1), read from `capabilities.effort.supported`. A cached list without the field means `false` until the next refresh.

### Data 3. Derived state (in memory)

The core reads the conversation file once at launch into a `Conversation`:

- `entries`: every valid entry in file order.
- `skipped`: the count of unreadable lines.
- `seen`: per loaded file, the latest content the current context has seen (from the last `context` entry and later notices).
- `active`: the running reply, if any: its cancel signal, the id of the message it answers and the text received so far.

Nothing here is written anywhere else (Principle I).

### Data 4. Types

#### LocalClock (webview → core)

| Field | Type | Rule |
|---|---|---|
| `iso` | string | RFC 3339 local time with offset, e.g. `2026-11-13T21:40:05+07:00`. Rejected as `InvalidInput("clock")` if it doesn't parse. |
| `zone` | string | IANA zone from `Intl`, 1–64 characters of `A–Z a–z 0–9 / _ + -`. Otherwise `InvalidInput("clock")`. |

#### ChatMessage (core → webview)

| Field | Type | Meaning |
|---|---|---|
| `id` | string | Entry id. |
| `kind` | `"user"` \| `"companion"` \| `"divider"` \| `"note"` | `divider` for a fresh context. `note` for a notice ("SOUL.md was updated"). |
| `text` | string | User: what they typed. Companion: the concatenated `text` blocks. Note: the file name. |
| `state` | `"complete"` \| `"stopped"` \| `"incomplete"` \| null | Companion only. |
| `at` | number | Unix milliseconds. |

Hidden entries (the first-hello request, replaced replies) are never sent to the webview.

#### ChatPage

`{ messages: ChatMessage[], hasMore: boolean }`. Oldest first within the page.

#### ChatState (core → webview)

| Field | Type | Meaning |
|---|---|---|
| `replying` | boolean | A reply is running. |
| `canRegenerate` | boolean | The last visible message is a companion reply and nothing is running. |
| `pendingRetry` | `RetryReason` \| null | The last user message has no complete reply: `Interrupted` (the app ended mid-reply, FR-017) or a `ProviderError` from the last attempt, including when an incomplete reply was saved (FR-027). |
| `firstHelloPending` | boolean | From `state.json`. |
| `budgetReached` | boolean | R10. |
| `modelChangePending` | boolean | The chat model in settings differs from the current context's (FR-032). |
| `shortened` | string[] | Files over their size budget in the current context (FR-012). |
| `skippedLines` | number | Unreadable lines in the conversation file (FR-018). |

#### ChatEvent (core → webview, over the call's channel)

| Variant | Payload | When |
|---|---|---|
| `Started` | `{ userMessage: ChatMessage \| null }` | The request is about to be sent. `userMessage` is the saved user message, or null for a retry, regenerate or hello. |
| `Delta` | `{ text: string }` | Reply text arrived. |
| `Waiting` | `{ seconds: number }` | The provider is busy. An automatic retry follows (FR-025). Any text streamed so far in this reply has been saved and replaced, so the webview discards it from the view; the retry streams a fresh reply. |
| `Finished` | `{ message: ChatMessage \| null, state: ChatState }` | The reply ended: saved complete, stopped or incomplete. `message` is null when it was stopped before any text arrived. |
| `Failed` | `{ error: ProviderError, partial: ChatMessage \| null, state: ChatState }` | No complete reply. `partial` is the saved incomplete reply, if text had arrived. |

#### ProviderError (extended)

The onboarding page's `KeyRejected`, `NoCredit` and `Unreachable`, plus `Busy`, `ContextTooLong`, `Declined` and `Invalid` (R11). The onboarding model list never returns the new variants.

#### RetryReason

`Interrupted` or `Provider(ProviderError)`.

#### SendRefusal

The command result when nothing was sent: `Busy` (a reply is running), `Empty` (whitespace only), `TooLong` (over 32,000 characters), `NoKey`.

### Data 5. Traits

#### LlmProvider (extended)

```text
list_models(key) -> Result<Vec<ModelInfo>, ProviderError>                    # from onboarding
stream(key, request: TurnRequest, on_text: &mut dyn FnMut(&str))
    -> Result<Reply, StreamFailure>                                          # new
```

`TurnRequest` holds the model, the system blocks, the message list (raw JSON) and whether effort and the fallback apply. `Reply` holds the assembled content blocks, `stop_reason`, `stop_details`, `usage` and the serving model. `StreamFailure` holds a `ProviderError`, the text received before the failure, and the provider's requested wait, if any. Dropping the future cancels the request (R4).

#### ChatSink

```text
send(event: ChatEvent)
```

Implemented by the shell over a Tauri channel, and by tests over a vector.

### Data 6. State transitions

#### A reply

```text
idle ──send / retry / regenerate / hello──▶ replying
replying ──end_turn──────────────▶ idle   (saved complete)
replying ──max_tokens────────────▶ idle   (saved incomplete)
replying ──Stop / quit───────────▶ idle   (saved stopped, if any text)
replying ──Busy──▶ waiting ──after the wait──▶ replying   (up to 60 s in total; any partial saved incomplete, then replaced)
replying ──any other error───────▶ idle   (saved incomplete if any text; pendingRetry set)
idle ──retry of an incomplete reply──▶ replying   (replaced appended first)
replying ──refusal───────────────▶ idle   (pendingRetry = Declined)
```

#### The first hello

```text
pending ──main window opens with a key──▶ greeting
greeting ──saved complete──▶ done
greeting ──fails or stopped──▶ pending
pending ──user sends a message──▶ done (no greeting)
```

#### Contexts

```text
(no file) ──first send or hello──▶ context 1
context n ──user starts a fresh context──▶ context n+1   (new snapshot, current model)
```

## Interfaces

### Chat Commands Between the Webview and the Core

These commands are added to the onboarding surface (see the onboarding Interfaces), whose rules all apply: thin handlers, validation in the core, expected failures as values, each command declared in `build.rs` and allowed in `src-tauri/capabilities/main.json`, and `src/api/commands.ts` as the only caller.

#### Commands

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `chat_state` | — | `ChatState` | Reads the in-memory conversation, `state.json` and the settings. Makes no network request. | FR-017, FR-018, FR-029, FR-030, FR-032 |
| `chat_history` | `before: string \| null`, `limit: number` (1–200) | `ChatPage` | Returns up to `limit` visible messages before the entry `before` (the latest when null), oldest first. | FR-016 |
| `send_message` | `text: string`, `clock: LocalClock`, `onEvent: Channel<ChatEvent>` | `SendRefusal \| null` | Refuses with `Empty`, `TooLong`, `NoKey` or `Busy` without writing anything. Otherwise: starts the first context if none exists, clears a pending first hello, appends notices for changed files, appends the user message (FR-013), sends `Started`, then streams the reply as described in "A reply" below. Resolves when the reply ends; the events arrive on the channel meanwhile. | FR-002, FR-003, FR-007, FR-011, FR-013, FR-021, FR-024 to FR-028 |
| `retry_reply` | `onEvent: Channel<ChatEvent>` | `SendRefusal \| null` | Valid only when `pendingRetry` is set, or when the last visible message is an incomplete reply. In the second case it first appends `replaced` for that reply. Then requests a reply to the conversation as it stands. File edits reach the companion with the next message the user sends. | FR-017, FR-024, FR-027 |
| `regenerate_reply` | `onEvent: Channel<ChatEvent>` | `SendRefusal \| null` | Valid only when `canRegenerate`. Appends `replaced` for the last reply, then requests a new one with the same request as the replaced reply. | FR-023 |
| `start_first_hello` | `clock: LocalClock`, `onEvent: Channel<ChatEvent>` | `SendRefusal \| null` | Valid only when `firstHelloPending` is true and the conversation has no user message. Starts the first context, appends the hidden hello request and streams the greeting. Clears `firstHelloPending` when the greeting is saved complete. | FR-020, FR-021 |
| `stop_reply` | — | — | Stops the running reply, if any, including while it waits to retry after `Busy`. The reply's own channel receives `Finished`. | FR-022 |
| `start_fresh_context` | — | `ChatState` | Refused as `InvalidInput("busy")` while a reply runs. Appends a `context` entry with a fresh snapshot of the files and the current model. | FR-030, FR-031, FR-032 |
| `open_link` | `url: string` | — | Opens `http`, `https` and `mailto` URLs in the default browser. Any other scheme is `InvalidInput("url")`. | FR-004 |

The webview passes its clock as `{ iso: new Date()` formatted as RFC 3339 with offset, `zone: Intl.DateTimeFormat().resolvedOptions().timeZone }`.

#### A reply

Every command that starts a reply runs the same steps in the core:

1. Build the request from the conversation file (R7).
2. Stream it. Each text delta is sent as `Delta`.
3. On `Busy`, send `Waiting`, wait, and go back to step 2, for up to 60 seconds in total. If text had already arrived, save it as an incomplete reply and append `replaced` for it before going back, so the next attempt sends the same request.
4. Save the outcome and send exactly one final event:
   - `end_turn` (or any stop reason other than those below): save complete, send `Finished`.
   - `max_tokens`: save incomplete, send `Finished`.
   - Stop or quit: save stopped if any text arrived and send `Finished`. With no text, nothing is saved, `Finished` carries no message, and `pendingRetry` is set.
   - `refusal`: send `Failed { error: Declined }`.
   - Any error: save incomplete if any text arrived, then send `Failed`.

The channel is dropped after the final event. If the window was hidden, events still arrive, because the webview stays loaded.

#### Changes to onboarding commands

| Command | Change |
|---|---|
| `select_model` | Unchanged behavior. `ChatState.modelChangePending` reports the difference from the current context. |
| `model_list` | `ModelInfo` gains `supportsEffort` (read from the provider's `capabilities`). |
| `app_status` | Unchanged. The main view calls `chat_state` and `chat_history` after it. |

#### Capability file additions

`allow-chat-state`, `allow-chat-history`, `allow-send-message`, `allow-retry-reply`, `allow-regenerate-reply`, `allow-start-first-hello`, `allow-stop-reply`, `allow-start-fresh-context`, `allow-open-link`. Channels need no extra permission. The opener plugin is called from Rust only, so no `opener:` permission is added.

### The Chat Request to the Claude API

This slice adds one outbound request, `POST /v1/messages`, to the model list from onboarding (see the onboarding Interfaces). Both go to the same host. Every detail below is confined to `crates/steve-core/src/llm/` (Principle VI).

#### Request

```http
POST {base}/v1/messages
x-api-key: <key>
anthropic-version: 2026-10-21
content-type: application/json
anthropic-beta: server-side-fallback-2026-07-01      ← only for models on the fallback list
```

```json
{
  "model": "claude-opus-5-5",
  "max_tokens": 16000,
  "stream": true,
  "output_config": { "effort": "low" },
  "fallbacks": "default",
  "cache_control": { "type": "ephemeral" },
  "system": [
    { "type": "text", "text": "<harness rules>" },
    { "type": "text", "text": "<IDENTITY.md>\n\n<SOUL.md>" },
    { "type": "text", "text": "<USER.md>\n\n<MEMORY.md>", "cache_control": { "type": "ephemeral" } }
  ],
  "messages": [
    { "role": "user", "content": [
      { "type": "text", "text": "<context>Local time: Friday 13 November 2026, 21:40 (Asia/Jakarta, UTC+07:00)</context>" },
      { "type": "text", "text": "Hi Steve!" }
    ] },
    { "role": "assistant", "content": [ "...blocks exactly as received..." ] }
  ]
}
```

Rules:

- `output_config` is present only when the model's cached `supportsEffort` is true. `fallbacks` and the beta header are present only for `claude-opus-5-5`, `claude-opus-5`, `claude-sonnet-5-5`, `claude-fable-5` and `claude-fable-5-1`.
- `thinking`, `temperature`, `top_p`, `top_k` and `tools` are never sent.
- The system blocks come from the current `context` entry's snapshot, cut to their size budgets, and do not change within a context.
- Message content is the stored content, with the replay rules of R7.
- The key is sent in the header only. It never appears in the body, a log line or an error.

#### Response stream

`200` with `content-type: text/event-stream`. Events handled:

| Event | Used for |
|---|---|
| `message_start` | `message.model` (the serving model), initial `usage`. |
| `content_block_start` | A new block at `index`, kept as raw JSON. A block of type `fallback` marks a server-side fallback boundary. |
| `content_block_delta` | Merged into the block at `index` (R3). `text_delta.text` is also passed on as reply text. |
| `content_block_stop` | Finishes the block; parses `input` from concatenated `partial_json`. |
| `message_delta` | `delta.stop_reason`, `delta.stop_details`, final `usage`. |
| `message_stop` | The reply is complete. |
| `ping` | Ignored; resets the 90-second silence timer. |
| `error` | `error.type` classified as below. `overloaded_error` and `api_error` are `Busy`. |

A stream that ends without `message_stop`, or is silent for 90 seconds, is `Unreachable` with the text received so far.

#### Outcomes

| Response | Outcome |
|---|---|
| `message_stop` with `stop_reason` `end_turn`, `stop_sequence` or `pause_turn` | Reply complete. |
| `stop_reason: "max_tokens"` | Reply saved incomplete. |
| `stop_reason: "refusal"` | `Declined` (after any server-side fallback inside the same call). |
| Connection failure, timeout, silence | `Unreachable` |
| 429 `rate_limit_error`, 529 `overloaded_error`, 500 `api_error` | `Busy`, with `retry-after` seconds when present |
| 401, 403, `authentication_error`, `permission_error` | `KeyRejected` |
| 402, `billing_error` | `NoCredit` |
| 400 `invalid_request_error` whose message says the prompt is too long | `ContextTooLong` |
| Any other status | `Invalid` |

The body's `error.type` decides first and the status second, as for the model list.

#### Recorded fixtures

Under `crates/steve-core/tests/fixtures/messages/`, replayed by `claude_contract.rs` and by `e2e/replay-server.ts`. Hand-written from the documented shapes first, then replaced with recordings by the owner, with the key removed:

| File | Shape |
|---|---|
| `reply.sse` | A short text reply ending in `end_turn`, with `ping` events. |
| `thinking.sse` | A thinking block (empty text, with a signature) then text. |
| `refusal.sse` | `stop_reason: "refusal"` with `stop_details`. |
| `fallback.sse` | A `fallback` block followed by text from the fallback model. |
| `overloaded.json` | A 529 `overloaded_error` body. |
| `too-long.json` | A 400 prompt-too-long body. |
| `stream-error.sse` | Some text, then an `error` event. |

## Checks

### Prerequisites

- The app installed and set up through the onboarding wizard, or a clean machine to run it.
- Rust 1.90 or later, Node.js 22 or later, pnpm 10.
- A Claude API key with credit for the manual scenarios. The automated checks never use one.

### Run in development

```bash
pnpm tauri dev
```

### Inspect the conversation file

`%APPDATA%\mySteve\workspace\sessions\main.jsonl`. One JSON object per line (Data 1). It should only ever grow.

### Manual validation scenarios

| # | Story | Steps | Expected |
|---|---|---|---|
| 1 | US3 | Finish the wizard on a clean state, entering "Sam" as what Steve should call you. | Without typing, a greeting streams in, in character, using "Sam". Under 3 minutes from launching the installer (SC-001). |
| 2 | US3 | Quit and relaunch. | The greeting is still there. No second greeting. |
| 3 | US1 | Send "Hi! What day is it today?" | The reply starts within 5 seconds, streams in, and states today's date correctly. |
| 4 | US1 | Ask for a short list, a table and a code sample. | Each renders as formatted text. |
| 5 | US1 | Ask the companion to reply with `<b>bold</b>`, `<script>alert(1)</script>` and a Markdown image from a web address. | HTML shows as text. No script runs. The image is not loaded; its alt text is shown as a link. |
| 6 | US1 | Click a link in a reply. | It opens in the default browser. |
| 7 | US1 | Ask "Are you a real person?" | An honest answer, in character, that it is an AI. |
| 8 | US1 | Type two lines with Shift+Enter, then Enter. | One message with two lines is sent. |
| 9 | US2 | After a few exchanges, quit from the tray and relaunch. Ask about something said earlier. | The whole conversation is shown in order, and the companion answers with that context. |
| 10 | US2 | While a reply is streaming, end `mysteve.exe` in Task Manager. Relaunch. | The last user message is shown with an offer to retry. Retry produces a reply. |
| 11 | US4 | Ask for a long story and press Stop (then repeat with Escape). | The reply stops within a second, stays visible marked as stopped, and a new message can be sent. |
| 12 | US4 | Press Regenerate on the last reply. Then send a message that refers to it. | A new reply replaces the old one. The companion builds on the new one. The old one is still in `main.jsonl` with a `replaced` entry after it. |
| 13 | US5 | Disconnect the network and send a message. Reconnect and press Retry. | The message stays, a plain "can't reach Claude" message with Retry appears, and Retry brings the reply. |
| 14 | US5 | In settings, replace the key with a revoked one, then send a message. | "Key no longer accepted" with an offer to enter another key; the message is kept. After storing a working key, Retry works. |
| 15 | US5 | Type text in the composer, trigger scenario 13, switch to settings and back. | The typed text is still in the composer. |
| 16 | US6 | Mid-conversation, edit `SOUL.md` in a text editor to make the companion speak like a pirate. Send a message. | The reply follows the new soul. The conversation shows a small "SOUL.md updated" note. `SOUL.md` is unchanged by the app. |
| 17 | Edge | Make `MEMORY.md` longer than 12,000 characters and send a message. | The app says `MEMORY.md` was shortened. The file on disk is untouched. |
| 18 | Edge | Set `context.budgetTokens` to 2000 in `settings.json`, relaunch, and chat until the budget is reached. Start a fresh context. | The app offers a fresh context. After starting it, the history stays visible with a divider, and the companion no longer refers to earlier turns but still knows its soul and profile. |
| 19 | Edge | Change the model in settings mid-conversation. | Settings says the change takes effect in a fresh context and offers to start one now. |
| 20 | Edge | Append a line `not json` to `main.jsonl` and relaunch. | The app says one line couldn't be read. The file is unchanged. |
| 21 | Edge | Remove the key in settings. | History is visible and scrollable. The composer says conversation needs a key and offers to add one. |
| 22 | SC-005 | Generate a 10,000-message `main.jsonl` with the test helper, then launch. | The window is usable within 3 seconds and scrolling back is smooth. |
| 23 | SC-007 | Send 20 messages in one context, then read `usage.cache_read_input_tokens` on the replies in `main.jsonl`. | From the second reply on, at least 80% of each reply's input tokens are cache reads. |
| 24 | SC-009 | Capture network traffic for a full session (wizard to chat). | `mysteve.exe` connects only to `api.anthropic.com`. |
| 25 | Accessibility | With Narrator on, send a message, stop one, regenerate one and retry one, keyboard only. | Every control is reachable and named. A finished reply is announced once, not word by word. |
| 26 | SC-003 | Run `cargo test -p steve-core --release --test chat_request -- --ignored --nocapture preparing_a_request_on_10000_messages`. | The printed 95th percentile is 150 ms or less. |

### What "done" looks like

- Scenarios 1 to 26 pass.
- SC-001 to SC-009 are met. SC-008 (blind matching of replies to soul templates) is a release-readiness check, not a merge gate.
