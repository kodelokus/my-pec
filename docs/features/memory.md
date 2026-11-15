# Memory

Built.

## User stories

### US1. The companion keeps what matters (P1)

The user tells the companion that their sister's name is Rina, that they now prefer to be called Sam instead of Samuel, and, later, "remember that I'm allergic to peanuts". During the conversation the companion also jots down what the day was about. A week later, in a fresh context, the companion calls them Sam, knows about the allergy, and can say what they talked about yesterday.

**Why this priority**: "Steve remembers" is the product's second promise. Without durable memory every fresh context starts from zero, and nothing later (compaction, dreaming) has anything to work on.

**How to check**: Tell the companion a profile fact, an explicit "remember that...", and some everyday news; start a fresh context; confirm the profile and long-term memory files hold the first two, today's daily note holds the news, and the companion knows all three without being reminded.

**Acceptance Scenarios**:

1. **Given** a conversation, **When** the user says "remember that..." followed by a fact, **Then** the fact is in long-term memory before the companion's reply ends, and the reply shows that it was remembered.
2. **Given** the profile says the user is called Samuel, **When** the user says they prefer Sam now, **Then** the profile says Sam, the old value is gone from the profile, and the reply shows that the profile was updated.
3. **Given** an ordinary conversation about the user's day, **When** something worth keeping comes up, **Then** the companion adds a short note to today's daily note on its own, without the user asking.
4. **Given** facts saved in earlier days, **When** a fresh context starts, **Then** the companion has the profile, long-term memory, and today's and yesterday's daily notes, and uses them without being told again.
5. **Given** a fact the companion just saved, **When** the user asks about it later in the same context, **Then** the companion answers from it and the app does not resend the file.
6. **Given** the companion's write fails (for example the disk is full), **When** the reply is shown, **Then** it says the memory could not be saved, and the conversation continues normally.

### US2. Older memories come back when they matter (P1)

Three weeks ago the user mentioned that their dog Bimo was afraid of thunder. Tonight there is a storm and they say so. Without being asked, the companion asks how Bimo is doing. When the user asks "what did I say about the job interview last month?", the companion looks it up and answers.

**Why this priority**: Only the last two days of notes are loaded into a context. Everything older is reachable only through search and recall, so without them memory stops at yesterday.

**How to check**: Seed a daily note from weeks ago, start a fresh context, send a message that relates to it without naming it, and confirm the reply uses it; then ask directly about an older topic and confirm the companion finds and quotes it.

**Acceptance Scenarios**:

1. **Given** an older daily note relevant to the user's message, **When** the user sends that message, **Then** the most relevant memory snippets are attached to it automatically and the companion can use them in its reply.
2. **Given** no stored memory relevant to the message, **When** the user sends it, **Then** nothing is attached.
3. **Given** the user asks about something from the past, **When** the companion replies, **Then** it can search its memory itself and answer from what it finds, saying when and where it was noted.
4. **Given** memories written in Indonesian, Japanese or another language, **When** the user's message or the companion's search uses words from them, **Then** they are found.
5. **Given** auto-recall is switched off in settings, **When** the user sends a message, **Then** nothing is attached automatically, and the companion can still search when it decides to.
6. **Given** a fresh install with no download beyond the app, **When** memory is searched, **Then** search works.

### US3. See and edit what the companion knows (P2)

The user opens the memory browser from the main window. They read the companion's soul, their own profile, the long-term memory and the daily notes by date. They fix a wrong detail in their profile, tighten a line in the soul, and search for "Bimo" to see everything noted about their dog. They notice the companion changed their profile in a way they don't like and undo that change. Once, they open the folder to look at the files in their own editor.

**Why this priority**: The user owns the companion's mind (Principle I). Memory that can't be seen or corrected isn't trustworthy, and the principles require every automatic rewrite to be reversible from the interface. It depends on stories 1 and 2 producing something to browse.

**How to check**: Open the memory browser, view each file kind, edit and save the profile, send a message and confirm the companion uses the edit, undo a change the companion made, and open the folder.

**Acceptance Scenarios**:

1. **Given** the main window, **When** the user opens the memory browser, **Then** they can view the soul, the profile, long-term memory and each daily note by date.
2. **Given** a file open in the browser, **When** the user edits and saves it, **Then** the file on disk holds exactly their text, and from the next message the companion uses the edited content.
3. **Given** the soul is open for editing, **When** the user types, **Then** they see how much of its size budget is used, as in the setup wizard.
4. **Given** the companion changed the profile or long-term memory, **When** the user chooses to undo that change, **Then** the file returns to exactly what it was before the change.
5. **Given** the user searches the browser for a word, **When** results appear, **Then** each shows the file, the date and the matching text, and opens that place in the file.
6. **Given** the browser, **When** the user chooses "Open folder", **Then** the workspace folder opens in the system file manager.

### US4. Forget something (P2)

The user once told the companion about a friend's private health issue and now regrets it. In the memory browser they find the note and choose Forget. The app asks them to confirm and says what will be removed. Afterwards the companion no longer knows it, and the note is gone from search.

**Why this priority**: The companion hears private things. Being able to take something back is essential to trust (CTL-2), and the principles make forgetting a hard rule.

**How to check**: Save a memory, forget it from the browser, and confirm it is gone from the file, from search and recall, and that the companion no longer knows it in a fresh context.

**Acceptance Scenarios**:

1. **Given** an entry in long-term memory, a line in the profile, or a note in a daily file, **When** the user chooses Forget, **Then** the app asks for confirmation, showing the text that will be removed.
2. **Given** the user confirms, **When** forgetting completes, **Then** the text is gone from the file, search and auto-recall no longer return it, and it is not loaded into any future context.
3. **Given** the user cancels, **When** the dialog closes, **Then** nothing has changed.
4. **Given** a forgotten memory came from something said in the conversation, **When** forgetting completes, **Then** those messages stay in the conversation, marked so that later dreaming skips them. If they are in the current context, the app says the companion may still see them until a fresh context starts, and offers to start one.
5. **Given** a whole daily note, **When** the user forgets it, **Then** the whole file is removed after confirmation.

### US5. The files stay the source of truth (P3)

The user keeps their workspace in a text editor alongside the app. They add a line to yesterday's daily note and a new section to the long-term memory. Within moments, search finds the new lines. One day the search index is damaged; the app rebuilds it without losing anything, and chat keeps working meanwhile. The long-term memory grows past its budget, and the app warns them and suggests tidying it.

**Why this priority**: It protects the promise that the files are the mind (Principle I, MEM-8, MEM-9). Most users won't notice it while things work, which is why it follows the visible stories.

**How to check**: Edit a daily note in an outside editor and confirm search finds the new text within seconds; delete the index, relaunch, and confirm search returns the same results once it is rebuilt and chat works throughout; grow the long-term memory past its budget and confirm the warning.

**Acceptance Scenarios**:

1. **Given** the user edits a memory file in an outside editor, **When** they search a few seconds later, **Then** the new text is found and removed text is not.
2. **Given** the search index is missing or damaged, **When** the app starts, **Then** it rebuilds the index from the files, chat stays usable meanwhile, and once the rebuild ends searches return the same results as before.
3. **Given** the profile, long-term memory or a loaded daily note is over its size budget, **When** a context starts, **Then** the file stays whole on disk, only the budgeted part is given to the companion, and the app tells the user which file was shortened and offers to open it in the memory browser.
4. **Given** long-term memory is full, **When** the companion is asked to remember something, **Then** the fact is kept in today's daily note instead, and the companion and the app tell the user that long-term memory needs tidying.

## Edge cases

- **The user edits a file in the browser while the companion writes to it.** The app does not overwrite either change silently. It tells the user the file changed and lets them reload it or keep their version, which is then saved on top.
- **The user edits a file in an outside editor while the companion writes to it.** The companion's write re-reads the file first. If it changed, the write is applied to the new content or, if that is not possible, skipped and reported, never applied over the user's edit.
- **The companion's own write mid-context** does not count as an outside edit. No "file changed" notice is added for it, because the write is already in the conversation.
- **A day ends during a context.** Notes after midnight go to the new day's file. Notes from days that started after the context began are reached through search and recall, not loaded.
- **The user asks the companion in chat to forget something.** The companion says it can't forget on its own, and offers to open the memory browser at the right place.
- **The user asks the companion to change its own soul.** The companion has no way to write the soul. It suggests the change and points the user to the memory browser.
- **Content that arrives through a tool** (for example something the companion's own search returned) is marked untrusted in the conversation. The companion's own writes record it, or the user when they asked, as the origin. Nothing marked untrusted is promoted to long-term memory automatically by later processing.
- **Recalled snippets already in the context.** Snippets already loaded at the start of the context (the profile, long-term memory, today's and yesterday's notes) are not attached again.
- **A daily note is missing, renamed or not in the expected format.** Files that don't follow the date naming are not loaded as daily notes and are not changed. Files that can't be read are skipped by search and shown as unreadable in the memory browser.
- **The conversation started before this feature existed.** That context continues without memory tools. The app tells the user that memory starts with a fresh context and offers to start one.
- **The user moves the workspace.** Memory files move with it, and the index is rebuilt for the new location.
- **A very large outside edit** (for example pasting thousands of notes). Indexing runs in the background and does not slow chat. Search covers the new text once indexing finishes.
- **A memory file has a hand-written structure the app doesn't expect.** The app reads it as text, never rewrites it to fit, and appends new entries at the end.

## Requirements

**Memory files**

- **FR-001**: The companion's memory MUST be kept in the workspace as human-readable files: the user profile, long-term memory, and one daily note file per local date, alongside the conversation file from chat (MEM-1).
- **FR-002**: Each long-term memory entry MUST record when it was written and where it came from (the conversation message or file it was taken from) and its origin: the user, the companion, untrusted content, or the system (MEM-10).
- **FR-003**: Each note the companion adds to a daily file MUST record its time and origin. Daily note files MUST only be appended to by the app.

**Writing memory**

- **FR-004**: The companion MUST be able to add a note to today's daily note on its own during conversation, without the user asking (MEM-2).
- **FR-005**: When the user explicitly asks the companion to remember something, the fact MUST be written to long-term memory before the reply ends (MEM-3).
- **FR-006**: When the user states a profile fact (such as what to call them, their language, or a preference), the companion MUST be able to update the profile so the new value replaces the old one in place (MEM-4).
- **FR-007**: The companion MUST have no way to change the soul. Its harness rules MUST tell it to suggest soul changes to the user instead.
- **FR-008**: Every write by the companion MUST be shown to the user on the reply that made it, naming what was written and where, with a way to open it in the memory browser.
- **FR-009**: Before each change the companion makes to the profile or long-term memory, the previous content MUST be kept, so that the change can be undone from the memory browser and the file restored byte for byte (Principle IV).
- **FR-010**: A write MUST NOT overwrite an edit the user made since the companion last read the file. The write MUST be applied on top of the current content or skipped and reported.
- **FR-011**: When long-term memory is at its size budget, a request to remember MUST still keep the fact, in today's daily note, and the user MUST be told that long-term memory needs tidying.
- **FR-012**: A write that fails MUST be reported to the companion and shown to the user, and MUST NOT break or block the conversation (Principle V).
- **FR-013**: Content from an untrusted or system origin MUST NOT be written to the profile or long-term memory automatically (Principle IV, NFR-SEC-5).

**Loading memory into a context**

- **FR-014**: Every context MUST start with the profile and long-term memory, and with today's and yesterday's daily notes (MEM-5).
- **FR-015**: Each loaded file MUST respect its size budget. Over budget, the file MUST stay whole on disk and only the budgeted part MUST be loaded: the beginning of the profile and of long-term memory, and the most recent part of a daily note. The app MUST tell the user which file was shortened and offer to open it (MEM-9).
- **FR-016**: Changes the companion makes during a context MUST NOT trigger a "file changed" notice. Changes the user makes, in the memory browser or an outside editor, to the soul, profile or long-term memory MUST take effect from the next message, as in chat.

**Search and recall**

- **FR-017**: The companion MUST be able to search its memory (long-term memory and all daily notes) by keyword, and to read a memory file or part of one. Results MUST name the file, date and place of each match (MEM-6, keyword part).
- **FR-018**: Keyword search MUST work on a fresh install with nothing downloaded, and MUST find text in languages other than English, including languages written without spaces between words.
- **FR-019**: Before each user message is sent, the most relevant memory snippets MUST be attached to it automatically, within a small budget, and only when they are relevant enough. Snippets already loaded at the start of the context MUST NOT be attached again (MEM-7).
- **FR-020**: Users MUST be able to switch auto-recall off and on in settings. The change MUST take effect from the next message. Search by the companion MUST keep working when auto-recall is off.
- **FR-021**: Snippets attached by auto-recall MUST be stored in the conversation marked as recalled, so that later processing can tell them apart from what the user typed.

**Files are the source of truth**

- **FR-022**: The search index MUST be derived from the files alone. Deleting it MUST lose no memory, and the app MUST rebuild it from the files when it is missing or damaged (MEM-8, NFR-REL-2).
- **FR-023**: Edits the user makes to memory files outside the app MUST be reflected in search within 5 seconds and MUST NOT be overwritten.
- **FR-024**: Indexing, rebuilding and searching MUST NOT block or slow the conversation. If the index is unavailable, chat MUST keep working, and the companion MUST be told that search is temporarily unavailable (Principle V).

**Memory browser**

- **FR-025**: Users MUST be able to open a memory browser from the main window to view and edit the soul, the profile, long-term memory and each daily note, and to open the workspace folder in the system file manager (CTL-1). Saved edits MUST be written to the file exactly as typed.
- **FR-026**: Users MUST be able to search memory in the browser and open each result at its place in its file.
- **FR-027**: Users MUST be able to forget a long-term memory entry, a profile line, a note in a daily file, or a whole daily file. Forgetting MUST ask for confirmation showing what will be removed. Once confirmed, the content MUST be removed from the file and the index, MUST NOT be loaded into any future context or returned by search or recall, and MUST be recorded so that later dreaming excludes it (CTL-2, Principle IV). The conversation messages the memory came from MUST stay in the conversation and its file, marked as forgotten so that later dreaming skips them. If those messages are in the current context, the app MUST tell the user that the companion may still see them until a fresh context starts, and MUST offer to start one.
- **FR-028**: Users MUST be able to undo each change the companion made to the profile or long-term memory, from the browser, restoring the previous content exactly. If the user edited the file since, the app MUST say so and ask before undoing.
- **FR-029**: When a file open in the browser changes on disk, the browser MUST tell the user and MUST NOT save over the change without their choice.
- **FR-030**: The memory browser MUST be usable with the keyboard alone and with a screen reader, and its text MUST come from the app's string table, as elsewhere.

**Privacy**

- **FR-031**: Memory content MUST be sent only to the configured Claude API as part of the conversation. This feature MUST add no network destination and no download (Principle II).

### Key entities

- **User profile**: Stable facts and preferences about the user, written as current truths. A changed fact replaces the old one in place.
- **Long-term memory**: A short, curated list of durable facts and standing decisions, grouped by topic. Each entry has a stable identity, its date, its source and its origin.
- **Daily note**: One file per local date, appended to by the companion during the day. Each note has its time and origin.
- **Memory snippet**: A passage of a memory file returned by search or auto-recall, with its file, date and place.
- **Origin (provenance)**: Who a piece of memory came from: the user, the companion, untrusted content (such as a tool result) or the system. Only user and companion content can become long-term memory automatically.
- **Previous version**: The content of the profile or long-term memory before a change by the companion, kept so the change can be undone.
- **Forgotten record**: What the user asked to forget and when, kept so that the content stays out of search, recall, future contexts and dreaming.
- **Search index**: A derived, rebuildable copy of the memory files arranged for fast search. Never a source of truth.

## Success criteria

- **SC-001**: In a scripted suite of 30 facts told to the companion across simulated days, at least 27 are recalled correctly in a fresh context a week later, without dreaming (product SC-2).
- **SC-002**: In a scripted suite of 20 explicit "remember that..." requests, at least 19 are in long-term memory when the reply ends.
- **SC-003**: After the user changes a profile fact, the profile holds only the new value in 100% of a scripted set of 10 changes.
- **SC-004**: Memory search returns results within 0.1 seconds for 95% of queries over 10,000 indexed passages, and the app's preparation before a message is sent, recall included, stays within 0.15 seconds for 95% of messages.
- **SC-005**: An edit made in an outside editor is found by search within 5 seconds in 95% of attempts.
- **SC-006**: After the index is deleted and rebuilt, a fixed set of queries returns the same results as before, and no memory is lost.
- **SC-007**: A forgotten memory appears in 0 searches, recalls or loaded contexts afterwards.
- **SC-008**: Undoing a change by the companion restores the previous file byte for byte in 100% of attempts.
- **SC-009**: In a test set of queries in Indonesian, Japanese and English, at least 95% find the matching note.
- **SC-010**: A user can find a specific memory in the browser and forget it within 1 minute.
- **SC-011**: During a capture of a full session that uses memory, the app's own connections go only to the Claude API.

## Clarifications

### Session 2026-11-14

- Q: When a memory is forgotten, what happens to the conversation it came from? → A: Memory only. It is removed from the memory files and the index. The source messages stay in the conversation, marked so that dreaming skips them. If they are in the current context, the app says so and offers a fresh context. Redacting conversation messages is out of scope.

## Assumptions

- **Depends on slice 2.** The conversation, contexts, the "file changed" notice, size budgets for loaded files and the settings screen come from chat. The profile and long-term memory files already exist from setup (the onboarding page).
- **Keyword search only.** Search by meaning, the local embedding model and its download arrive with semantic search (slice 7). The search and recall in this slice keep working unchanged when it does.
- **No dreaming yet.** Nothing is promoted from daily notes to long-term memory automatically in this slice; long-term memory grows only through explicit requests and the user's edits. The dream diary, its view in the browser, and "where did this come from" links in the browser (CTL-3) arrive with dreaming (slice 5). Sources and origins are recorded now so dreaming can use them.
- **No compaction yet.** A context starts on first run or when the user starts a fresh one, as in chat. Compaction (slice 4) will start contexts too, and memory loading will apply to them unchanged.
- **Budgets.** Starting values from the architecture: profile 4,000 characters, long-term memory 12,000, soul 8,000. Today's and yesterday's daily notes are loaded up to 4,000 characters each. Auto-recall attaches at most 3 snippets and 2,000 characters per message, and is on by default.
- **Undo** covers changes the companion made. The user's own edits in the browser are not tracked for undo; their editor or the file system keeps their history.
- **Forgetting in chat** is not a companion tool. The companion points the user to the memory browser.
- **Identity, avatar and model** stay in settings as in the onboarding page and are not part of the memory browser.
- **Off-the-record mode, export and wipe, and the activity log** (CTL-4 to CTL-6) are out of scope.
- **Daily notes use the user's local date** at the moment the note is written.

## Decisions

Each section records a decision, why it was made, and what was rejected. API facts come from the Claude API reference bundled with Claude Code 2.1.309 (`shared/tool-use-concepts.md`, `shared/prompt-caching.md`, `shared/model-migration.md`). Facts marked **(record)** are confirmed by contract fixtures the owner records, as in chat.

### R1. Five memory tools, and the request that carries them

**Decision**: The request gains a `tools` list with the five tools of architecture section 5.4: `memory_search`, `memory_get`, `memory_note`, `memory_remember` and `profile_update`. Each has `strict: true`, `additionalProperties: false` and `eager_input_streaming: true`. `tool_choice` is not sent (the default, `auto`). The list is a constant, serialized in a fixed order, and is the same for every request in a context. Input schemas and descriptions are in Interfaces (Memory tools in the chat request).

**Rationale**:

- `strict: true` guarantees schema-valid arguments under `auto`. The reference states it needs `additionalProperties: false`.
- The reference makes `eager_input_streaming: true` the default for streamed requests with client tools. Inputs here are short, so the gain is small, but the core validates every input anyway (the principles, security gates), which is the condition the reference sets. That this combination is accepted alongside `strict` is **(record)**.
- `tool_choice` `any` and `tool` are a 400 on `claude-opus-5-5`. `auto` is the default and needs no field.
- Tools render before `system`, so the existing breakpoint on the last system block caches tools and system together. Changing the tool list invalidates the whole cache and, on current models, replayed thinking. The list therefore belongs to the context, like the system prompt (R2).
- Descriptions say when to call each tool, which the reference recommends for recent Opus models.

**Alternatives considered**: Generic file read and write tools (rejected by architecture 5.4: each kind of write needs its own validation, budget and undo). Forcing a note each turn (a 400 on Opus 5.5, and costly). Sending tools only when needed (breaks the prefix).

### R2. Contexts from before this feature

**Decision**: The `context` entry's existing `harness` field selects both the harness rules and the tool list. Contexts with `chat-1` (chat) keep their old harness text and no tools for as long as they last. New contexts get `memory-1`, with the new harness text and the five tools. `ChatState` gains `contextOutdated`, true when the current context's harness is not the newest, and the main window offers a fresh context, as it does for a model change.

**Rationale**: Principle III forbids changing the system prompt or tools mid-context, and changing them would invalidate replayed thinking. Starting a fresh context automatically would drop the user's recent turns without their choice, which chat FR-030 does not allow. Keeping the old harness text is one constant.

**Alternatives considered**: Snapshotting the harness text into each `context` entry (still needs the `chat-1` text for existing files, so it adds a format change for nothing). An automatic fresh context on upgrade.

### R3. The tool loop

**Decision**: A reply becomes a turn of one or more model calls. When a call ends with `stop_reason: "tool_use"`, the core:

1. saves the assistant message exactly as received (state `complete`);
2. runs every `tool_use` block in order, validating each input first; an invalid input or a failed tool gives a `tool_result` with `is_error: true`;
3. saves one user message with provenance `untrusted` holding all the `tool_result` blocks, followed by notice blocks for workspace files the user changed meanwhile (R7);
4. sends the next request, built from the file as before.

The loop ends when a call ends with any other stop reason. After 8 calls in one turn, the tool results say the limit is reached and ask for a final answer. Stop, retry after `Busy`, and failures behave per call exactly as in chat. If the app ends between steps 1 and 3, the conversation ends with an unanswered `tool_use`; `pendingRetry` is set, and Retry first saves `tool_result` blocks with `is_error: true` saying the tool was interrupted and may or may not have run, then continues. A tool runs only after the assistant message is saved, and its result is saved right after it runs.

Tools run sequentially on the reply's task. Each is a local file or index operation measured in milliseconds, so running them in parallel would add locking for no visible gain.

**Rationale**: The reference requires all results of one response in one user message. Saving each step keeps the file the single source of the request (Principle III) and keeps the message the user sent safe (Principle V). A cap bounds cost on a runaway loop. Retrying a write could duplicate a note, so an interrupted tool is reported, not repeated.

**Alternatives considered**: Re-running interrupted tools (a note or memory could be written twice). Holding results in memory until the turn ends (a crash would leave the file unable to produce a valid request).

### R4. What the user sees of a turn

**Decision**: The assistant messages and tool-result messages of one turn are shown as one companion message. Its text is the text of all the turn's assistant messages in order. `ChatMessage` gains `memory: MemoryMark[]`, one per write tool call in the turn: what was written (`note`, `remembered`, `profile`), the file, a short excerpt, and whether it succeeded. `memory_search` and `memory_get` are not shown. During streaming, `ChatEvent` gains `MemoryWritten { mark }`. Regenerate and Retry replace the whole turn: every line from the first assistant message after the user's message. Memory written by a replaced turn stays written; the user can undo or forget it.

**Rationale**: FR-008 asks that every write be visible on the reply that made it. Showing searches would be noise. Undoing writes on regenerate would turn a cheap action into a file rewrite the user didn't ask for.

**Alternatives considered**: Showing tool calls as separate bubbles (noisy). Rolling back writes on regenerate (surprising, and a second undo path).

### R5. Memory file formats

**Decision**:

- **Daily note** `memory/YYYY-MM-DD.md`, created on first write with a `# YYYY-MM-DD` heading. Each note is one bullet: `- 21:40 Rina is visiting next week. <!-- from:companion src:m_1794571234000_3 -->`. The time is the user's local time; `src` is the id of the user message that started the turn.
- **Long-term memory** `MEMORY.md`: one bullet per entry under a `## Topic` heading, as in architecture 4.1, with the date and origin added: `- Allergic to peanuts. <!-- id:m_8f3a at:2026-11-14 from:user src:m_1794571234000_3 -->`. A new entry goes at the end of its topic's section; a missing topic is added at the end of the file. `from` is `user` when the tool is called for an explicit request (the tool's `requested_by_user` input) and `companion` otherwise.
- **Profile** `USER.md`: the `- Key: value` lines the onboarding page writes. `profile_update` replaces the value of the first line whose key matches case-insensitively, or adds a line at the end.
- A file whose structure the app doesn't expect is read as text. New content is appended at the end; nothing is reformatted (see Edge cases).

**Rationale**: FR-002 and FR-003 need the time, origin and source of every entry. HTML comments keep the files readable in any Markdown viewer and match the architecture's entry format. The entry id lets dreaming (slice 5) merge or supersede an entry, and lets forget target one. `src` points at the message, which is stable, rather than at a line number in the transcript.

### R6. Writing memory safely

**Decision**: All memory writes go through one function in the new `memory` module, under one lock: read the file, apply the change to what was read, check the size budget, store the previous content when the file is `USER.md` or `MEMORY.md`, then write atomically with the existing `write_atomic`. Because the change is applied to the file as read at that moment, an outside edit is never overwritten (FR-010): the hash check of architecture 4.3 is only needed by jobs that read early and write late, which this slice has none of, except the memory browser (R12). Budget: a `memory_remember` that would take `MEMORY.md` over 12,000 characters writes the fact to today's note instead, with `from:user`, and the tool result says long-term memory is full (FR-011). `ChatState.memoryFull` tells the main window. A `profile_update` that would take `USER.md` over 4,000 characters falls back the same way, to today's note. Principle IV requires an automatic rewrite to pass validation or fall back to appending, and a refusal would do neither.

**Rationale**: One writer per file (architecture 4.3, `lean-code`: each store is the only writer of its data). Reading at write time is the simplest way to honor outside edits.

**Alternatives considered**: A background writer queue (no background job exists yet; tools run on the reply task, under a lock).

### R7. No notice for the companion's own writes

**Decision**: The transcript gains a `written` entry, `{ file, content, message }`, appended after every write to `USER.md` or `MEMORY.md`, where `content` is the file after the write. `seen` (chat R8) takes `written` entries into account, so the next message doesn't report the companion's own change as an edit. The notice check also runs when the tool-result message is built, before tools run, so a user edit made meanwhile is still announced in that message rather than absorbed by the `written` entry.

**Rationale**: FR-016. The companion already knows what it wrote; a notice would cost tokens and suggest the user edited the file.

### R8. Loading daily notes when a context starts

**Decision**: The `context` entry gains `notes`: the content of today's and yesterday's daily notes at the moment the context starts, keyed by date, each cut to its last 4,000 characters (whole lines). The request opens with a user message built from this snapshot: `<memory>Your daily notes from yesterday and today (your own notes, not typed by the person):</memory>` followed by each note. The user's first message in the context follows it; the API joins consecutive user messages into one turn. The local date comes from the clock of the command that starts the context, so `start_fresh_context` gains a `clock` argument. Days that begin after the context started are reached through search and recall (see Edge cases).

**Rationale**: Architecture 5.2 puts today's and yesterday's notes in an epoch opener at the start of `messages`, where slice 4's compaction summary will join them. Snapshotting them keeps a rebuilt request identical after a restart (Principle III), exactly as chat does for the system files. The most recent part of a note matters most, so the tail is kept.

**Alternatives considered**: A fourth system block (works, but diverges from the architecture's layout that compaction builds on). Re-reading notes at each request (changes the prefix).

### R9. The search index

**Decision**: SQLite through `rusqlite` with the `bundled` feature, at `state/steve.db` in WAL mode. Tables: `files` (path, size, modified time, hash), `chunks` (file, first and last line, text) and `chunks_fts`, an FTS5 table over chunk text with the `unicode61` tokenizer and `remove_diacritics 2`. One connection, behind a mutex in the core.

**Rationale**: The principles' technology constraints name SQLite with FTS5 for derived data, and architecture 4.2 names these tables. `bundled` compiles SQLite into the app, so no system library is needed, and its build enables FTS5 **(verify at implementation)**. Keyword search with no download satisfies MEM-6's fallback and Principle VII.

**Alternatives considered**: An in-memory index rebuilt at each launch (slow to start with years of notes, and slice 7 needs SQLite for vectors anyway). Tantivy (a much larger dependency, and not the stack the principles name).

### R10. Languages written without spaces

**Decision**: Before indexing and before querying, runs of characters from scripts written without spaces (Han, Hiragana, Katakana, Hangul, Thai, Lao, Khmer, Myanmar) are rewritten as overlapping two-character tokens: `天気予報` becomes `天気 気予 予報`. A query token of one such character becomes a prefix query. Other text goes to `unicode61` unchanged, which splits on spaces and punctuation and folds case and diacritics, so Indonesian, English and other space-separated languages work as written.

**Rationale**: FR-018 and NFR-I18N-1. `unicode61` treats a run of Han or Kana as one token, so `天気` would not match inside `天気予報`. FTS5's `trigram` tokenizer can't match words of two characters, which are common in Japanese and Chinese. Character bigrams are the standard approach (Lucene's CJK analyzer) and take a few dozen lines with the standard library's character ranges.

**Alternatives considered**: `trigram` (misses two-character words). A morphological analyzer such as Lindera (a large dictionary in the installer, per language).

### R11. Chunks, ranking and auto-recall

**Decision**:

- **Chunks**: Markdown is split at headings, blank lines and bullets into chunks of at most 600 characters, each with its line range. A chunk in `MEMORY.md` is indexed with its topic heading in front. Indexed files are `MEMORY.md` and `memory/YYYY-MM-DD.md`. `USER.md` is always loaded, so it isn't indexed.
- **Query**: the terms of the text are combined with `OR`. Once the index holds at least 50 chunks, terms found in more than half of them are dropped, using FTS5's own term statistics. This removes common words in any language without a stop-word list.
- **Score**: FTS5's `bm25`, multiplied for daily notes by an age factor that halves every 30 days. Of several hits from the same file within five lines, only the best is kept.
- **Auto-recall**: before each user message, the user's text is searched. A hit counts only if it matches at least two distinct query terms, or the only term when the query has one. Hits from the files loaded at the start of the context (the loaded part of `MEMORY.md`, and today's and yesterday's notes) are left out. The top three, up to 2,000 characters together, are added to the user message as one block: `<memory>Possibly relevant notes from your memory (found automatically, not typed by the person):</memory>` followed by each snippet with its date and file. The user message entry records which content block is the recall block (FR-021). `settings.json` gains `recall.auto`, default `true`.
- `memory_search` returns up to 8 hits as text with file, date and line range. `memory_get` returns a file or a line range of at most 200 lines.

**Rationale**: Architecture 6.2 and section 15 (top 3, 2,000 characters). A threshold on raw `bm25` doesn't carry across corpus sizes; matched-term coverage is stable and explainable. The recall block's wording tells the model the snippets are memory, not the user's words, and the entry's marker lets dreaming strip them (architecture 4.1).

**Alternatives considered**: Recalling from `USER.md` (always loaded already). A fixed stop-word list (English-only).

### R12. Keeping the index current without a file watcher

**Decision**: Before every search (tool, recall or browser), the index compares the memory folder's file list with the `files` table by size and modified time, and re-indexes changed files and drops removed ones, each in its own transaction. One search re-indexes at most 512 KB of changed files (always at least one file), so a very large outside edit never holds up a message: the rest is indexed by the following searches, and search covers the new text once they have caught up (see Edge cases). A file that can't be read as UTF-8 is skipped, recorded in `files` so it isn't retried until it changes, and shown as unreadable in the memory browser. At launch, the shell runs a full sync on a background thread so the first search finds the index ready. If the database can't be opened or fails `PRAGMA quick_check`, it is deleted and rebuilt the same way. If indexing fails, search reports itself unavailable and chat continues without recall (FR-024).

**Rationale**: FR-023 asks that outside edits are found within 5 seconds. Checking at query time makes every search current with no watcher thread, no debounce and no new dependency. Listing a folder of a few thousand small files and reading their sizes takes a few milliseconds, within the 0.1-second search budget (SC-004). Architecture 4.3 mentions a file watcher; it is updated to describe this check (the principles: the architecture is updated in the same change).

**Alternatives considered**: The `notify` crate with a debounced worker (a dependency and a thread for a property that a size and time comparison already gives). Indexing only at launch (misses edits made while the app runs).

### R13. Undo

**Decision**: A `preimages` table in `steve.db` holds, for each companion write to `USER.md` or `MEMORY.md`: the file, the content before, a hash of the content after, the turn's user message id, and the time. Undo restores the most recent change of a file; repeating it steps back further. If the file's current hash differs from the stored after-hash (the user edited it since), `memory_undo` returns `Edited` and the browser asks before undoing with `force`. Undo is a user action: it writes no preimage of its own and adds a notice at the next message like any user edit.

**Rationale**: Principle IV and NFR-REL-4 require the previous version before every automatic rewrite, reversible from the interface. A stack is simple to reason about: undoing an older change while keeping later ones would need a merge. Preimages are bookkeeping that can't be derived from files, which the principles allow in the database.

**Alternatives considered**: Preimages as files in the workspace (they would be indexed, loaded or mistaken for memory). Undo for the user's own edits (their editor keeps that history; Assumptions).

### R14. Forgetting

**Decision**: `memory_forget` takes a file and a target: a `MEMORY.md` entry by id or line, a `USER.md` line, a daily-note line, or a whole daily file. It:

1. removes the target from the file (or deletes the file) through the memory writer;
2. re-indexes the file, so search and recall can't return it;
3. deletes every preimage of that file whose content contains the removed text, so undo can't bring it back;
4. appends a `forgotten` entry to the transcript with the source message ids found in the removed lines' `src` (no text), which slice 5's dreaming will skip;
5. returns whether any of those messages are in the current context, so the browser can say the companion may still see them and offer a fresh context (see Clarifications).

The confirmation dialog is the webview's, showing the text to be removed, before the command is sent. A `forgotten` entry is never shown in the conversation.

**Rationale**: Principle IV: forgetting removes content from files, index and future dream input. Keeping a preimage would let undo restore what was forgotten. Recording message ids instead of text keeps the forgotten content out of the record itself. Architecture 4.1 already lists the `forgotten` entry type.

**Alternatives considered**: Redacting the source messages in the conversation file (ruled out in the Clarifications). Keeping the text in the tombstone to filter dreams by content (the record would itself hold what was forgotten).

### R15. The memory browser

**Decision**: A new main-window view, opened from a button beside Settings, with a list (Soul, Profile, Long-term memory, then daily notes newest first), a reader that shows each bullet with a Forget action, an editor (a plain textarea with the budget meter for budgeted files, as in the wizard), a search box, Undo for the profile and long-term memory, and "Open folder". Saving sends the text with the hash it was loaded with; if the file changed meanwhile the core returns `Conflict` with the current text and the browser asks to reload or keep the user's version. "Open folder" calls a command that opens the workspace folder through the opener plugin chat already uses, from Rust. Links from a reply's memory marks open the browser at that file and line.

**Rationale**: CTL-1 and FR-025 to FR-030. A textarea keeps the files exactly as typed and needs no editor dependency. The hash on save is the one place this slice reads early and writes late (R6).

**Alternatives considered**: A rich Markdown editor (a dependency, and it may reformat the file). Editing entries one by one (doesn't cover hand-written structure).

### R16. Test strategy

**Decision**:

- **Core unit tests**: chunking, CJK bigrams, query building, recall selection and threshold, entry formats (note, entry, profile line), budget fallback, the tool input validation table, `seen` with `written` entries.
- **Request snapshot test** (extended): a turn with a tool call and its results, a recall block, the daily-note opener and a notice after a tool result; each request still equals the previous one plus appended messages.
- **Core integration tests** on a temporary workspace with real SQLite and the fake provider: remember, note and profile tools end to end; an outside edit found by the next search; the index deleted and rebuilt with the same results; forget removes from file, index and preimages and writes the tombstone; undo byte for byte and the `Edited` case; interrupted tool recovery; a `chat-1` context keeps no tools.
- **Contract tests**: recorded SSE for a reply with one `tool_use`, with two parallel `tool_use` blocks, and a follow-up after tool results. Hand-written from the documented shapes first **(record)**.
- **UI tests**: the browser's reader, editor conflict, forget confirmation and undo prompt; memory marks on a reply.
- **End-to-end**: the replay server scripts a reply that calls `memory_remember`; the end-to-end spec checks the mark, the file and the browser.
- **Release readiness, not this slice's tests**: the 30-fact recall suite (SC-001), the search latency over 10,000 chunks (SC-004, an ignored benchmark like chat's), and the language test set (SC-009) run as documented in Checks.

**Rationale**: The principles' test gates and the `test-audit` skill: each contract owned once at its cheapest boundary; real SQLite, no fakes of it.

### R17. Deliberately not built in this slice

- Search by meaning, the embedding model and `chunks_vec` (slice 7).
- `recall_events`, `dream_candidates` and `dream_runs`, which only dreaming reads (slice 5). Recalled blocks are marked now so dreaming can strip them.
- Compaction and the memory flush (slice 4).
- "Where did this come from" links in the browser (CTL-3, slice 5). Sources are recorded now.
- The dream diary view (slice 5), off-the-record mode, export and wipe, and the activity log.
- A file watcher (R12), a writer queue thread (R6), parallel tool execution (R3).
- A companion tool that forgets, or one that writes `SOUL.md`.
- Undo for the user's own edits.

### Sources

- Claude API reference bundled with Claude Code 2.1.309: `shared/tool-use-concepts.md` (tool definitions, eager input streaming, tool results), `shared/prompt-caching.md` (render order, tools and cache invalidation), `shared/model-migration.md` and `shared/error-codes.md` (forced tool choice rejected on Opus 5.5, strict tool use).
- `docs/architecture.md` sections 4, 5.2 to 5.4, 6, 13 and 15.
- Chat: R6 to R9, R11 and R16.

## Data

Files and types from the onboarding and chat pages are unchanged except where noted. Rust definitions are the source; the TypeScript types in `src/bindings/` are generated from them.

### Data 1. Workspace memory files

All are plain Markdown, written atomically by the core's memory writer (R6), and readable and editable by the user in any editor (Principle I).

#### `memory/YYYY-MM-DD.md` — daily notes

Created on the first note of a local date. Only ever appended to by the app (FR-003); the user may edit it freely.

```markdown
# 2026-11-14

- 21:40 Sam's sister Rina is visiting next week. <!-- from:companion src:1794583200123-0 -->
- 22:05 Long-term memory was full; kept here: allergic to peanuts. <!-- from:user src:1794584700456-0 -->
```

| Part | Rule |
|---|---|
| Time | The user's local time when the note was written, `HH:MM`. |
| Text | One line, 1–1,000 characters, no line breaks (validated tool input). |
| `from` | `user`, `companion`, `untrusted` or `system` (R5). |
| `src` | The id of the user message that started the turn. |

A file whose name isn't a valid date is not a daily note: it is neither loaded nor indexed nor changed.

#### `MEMORY.md` — long-term memory

The onboarding page creates it as `# Memory\n`. Entries are bullets under `## Topic` headings (architecture 4.1):

```markdown
# Memory

## Health
- Allergic to peanuts. <!-- id:m_8f3a at:2026-11-14 from:user src:1794584700456-0 -->
```

| Part | Rule |
|---|---|
| `id` | `m_` plus 4–8 lowercase hex characters, unique in the file. |
| `at` | The user's local date when the entry was written. |
| `from` | `user` when written for an explicit request, otherwise `companion`. `untrusted` and `system` are refused (FR-013). |
| `src` | The id of the user message that started the turn. |
| Topic | 1–60 characters, one line. A new topic heading is added at the end of the file. |
| Budget | 12,000 characters for the whole file. A new entry that would exceed it goes to today's daily note instead (FR-011). |

Bullets without a comment (hand-written) are valid entries without an id; they can still be forgotten by line.

#### `USER.md` — the profile

The `- Key: value` lines of the onboarding page. `profile_update` replaces the value of the first line whose key matches case-insensitively, or appends `- Key: value` at the end. Key: 1–40 characters, no colon or line break. Value: 1–300 characters, one line. These are automatic rewrites of `USER.md`, so each one stores a preimage and can be undone (R13). Budget: 4,000 characters. An update that would exceed it, or a file whose structure the replacement can't follow, falls back to appending the fact to today's daily note (Principle IV: validate or fall back to appending), with a `keptInNote` mark.

### Data 2. The conversation file

`workspace/sessions/main.jsonl` keeps every entry type of the chat page. Changes:

#### `context` — gains `notes`, new harness version

| Field | Type | Meaning |
|---|---|---|
| `harness` | string | `"memory-1"` for contexts started by this version: the new harness rules and the five tools. `"chat-1"` contexts keep the old rules and no tools (R2). |
| `notes` | object | New, optional. `{ "YYYY-MM-DD": text }` for today's and yesterday's daily notes at the start of the context, each already cut to its last 4,000 characters. Missing in `chat-1` entries. |

#### `message` — new provenance, recall marker, tool turns

| Field | Change |
|---|---|
| `provenance` | Gains `"untrusted"`, used for user messages that carry `tool_result` blocks (architecture 6.3). |
| `recalled` | New, optional, user messages only: the index of the content block holding auto-recalled snippets (FR-021). |
| `marks` | New, optional, tool-result messages only: the `MemoryMark`s of the writes their tools made, shown on the turn's reply (FR-008). |
| `stop_reason` | `"tool_use"` marks an assistant message whose tools must be answered before the turn continues (R3). |

A turn is the lines from the first assistant message after a user-typed or hello message up to the next such message: assistant messages, tool-result messages and the entries between them.

#### `replaced` — now covers a whole turn

`target` is the first assistant message of the turn. Every `message` from the target up to the `replaced` entry is out of the conversation. A file from the chat feature, where the target is the single reply just before `replaced`, reads the same way.

#### `written` — new: the companion changed a loaded file

| Field | Type | Meaning |
|---|---|---|
| `file` | `"USER.md"` \| `"MEMORY.md"` | The file a tool wrote. |
| `content` | string | The whole file after the write. |
| `message` | string | The id of the tool-result message that reports the write. |

`seen` (the chat page) is updated by `written` entries as it is by notices, so no notice follows (FR-016). Not shown in the conversation.

#### `forgotten` — new: the user forgot a memory

| Field | Type | Meaning |
|---|---|---|
| `messages` | string[] | Ids of conversation messages the forgotten lines came from (their `src`). May be empty. |

Holds no forgotten text. Not shown in the conversation. Dreaming (slice 5) skips the listed messages.

**Validation on read**: as in chat. A `context` without `notes`, or a message without `recalled`, is valid.

### Data 3. Settings

`state/settings.json` gains one section:

```json
{ "recall": { "auto": true } }
```

A missing section means `true` (Principle VII).

### Data 4. The index database

`state/steve.db`, SQLite in WAL mode. Everything except `preimages` is derived from the workspace and can be dropped (Principle I, NFR-REL-2).

| Table | Columns | Notes |
|---|---|---|
| `files` | `path` (primary key, workspace-relative), `size`, `modified`, `readable` | One row per indexed file: `MEMORY.md` and valid daily notes. Size and modified time decide whether a file changed (R12); `readable` is false for a file that isn't UTF-8. |
| `chunks` | `id`, `path`, `first_line`, `last_line`, `text`, `day` (date or null) | Chunks of at most 600 characters (R11). `day` drives the age factor. |
| `chunks_fts` | FTS5 over the prepared chunk text, `unicode61 remove_diacritics 2`, rowid equal to `chunks.id` | Prepared text has script runs without spaces split into bigrams (R10). |
| `chunks_vocab` | `fts5vocab` over `chunks_fts` | Term statistics, for dropping terms in more than half the chunks (R11). |
| `preimages` | `id`, `path`, `before`, `after_hash`, `message`, `at` | One row per companion write to `USER.md` or `MEMORY.md` (R13). Rows are deleted by forget when `before` contains removed text. |
| `meta` | `key`, `value` | `schema_version`. A different version drops and rebuilds the derived tables and keeps `preimages`. |

### Data 5. Types

#### Tool inputs (model → core)

Defined with the tool schemas in Interfaces (Memory tools in the chat request). Every input is validated in the core before the tool runs; a failure is a `tool_result` with `is_error: true`.

#### MemoryMark (core → webview)

| Field | Type | Meaning |
|---|---|---|
| `kind` | `"note"` \| `"remembered"` \| `"profile"` \| `"keptInNote"` | What the companion wrote. `keptInNote`: long-term memory or the profile was full, so the fact went to today's note (FR-011). |
| `path` | string | Workspace-relative file. |
| `line` | number \| null | Line where the write landed, for opening the browser there. |
| `excerpt` | string | Up to 120 characters of what was written. |
| `ok` | boolean | False when the write failed (FR-012). |

#### ChatMessage — gains `memory`

`memory: MemoryMark[]`, empty for user messages and for replies that wrote nothing. A companion message now stands for a whole turn; its `text` joins the text of the turn's assistant messages; its `id` is the turn's first assistant message.

#### ChatEvent — gains `MemoryWritten`

`MemoryWritten { mark: MemoryMark }`, sent when a write tool finishes, before the turn continues.

#### ChatState — gains four fields

| Field | Type | Meaning |
|---|---|---|
| `contextOutdated` | boolean | The current context was started by an older harness and has no memory tools (R2). |
| `memoryFull` | boolean | `MEMORY.md` is at or over its budget (FR-011). |
| `autoRecall` | boolean | From settings (FR-020). |
| `shortened` | string[] | Unchanged type; now also lists daily notes cut in the context opener, as `memory/YYYY-MM-DD.md`. |

#### MemoryFile (core → webview)

| Field | Type | Meaning |
|---|---|---|
| `path` | string | `SOUL.md`, `USER.md`, `MEMORY.md` or `memory/YYYY-MM-DD.md`. |
| `kind` | `"soul"` \| `"profile"` \| `"memory"` \| `"daily"` | |
| `chars` | number | Current length. |
| `budget` | number \| null | 8,000, 4,000, 12,000, or null for daily notes. |
| `readable` | boolean | False when the file isn't valid UTF-8. Such a file is not indexed, and `memory_read` on it is `InvalidInput("unreadable")`. |

#### MemoryDoc (core → webview)

| Field | Type | Meaning |
|---|---|---|
| `path` | string | |
| `text` | string | The whole file. |
| `hash` | string | Hash of `text`, sent back with a save (R15). |
| `entries` | `MemoryEntry[]` | Bullets and `- Key: value` lines, for the reader and Forget. |
| `canUndo` | boolean | A preimage exists for this file. |

`MemoryEntry`: `{ line: number, text: string, id: string | null, at: string | null, from: string | null }`. `text` has the trailing comment removed.

#### Outcomes

| Type | Variants |
|---|---|
| `SaveOutcome` | `Saved { hash }` · `Conflict { text, hash }` |
| `ForgetTarget` | `Line { line }` · `Entry { id }` · `File` |
| `ForgetOutcome` | `{ inCurrentContext: boolean }` |
| `UndoOutcome` | `Undone` · `Edited` (the user changed the file since; retry with `force`) · `Nothing` |
| `SearchHit` | `{ path, day: string \| null, firstLine, lastLine, text }` |

### Data 6. State transitions

#### A turn with tools

```
user message saved ─► call ─► stop_reason tool_use ─► assistant saved ─► tools run (each: validate, write, written/preimage, MemoryWritten)
        ▲                                                                      │
        │                                                     tool-result message saved (untrusted, + notices)
        │                                                                      │
        └──────────── next call (at most 8 per turn) ◄─────────────────────────┘
   any other stop_reason ─► assistant saved, turn ends (Finished)
```

Stop, `Busy` retries and failures apply to each call as in chat. A crash after the assistant message and before its tool results leaves `pendingRetry` set; Retry first saves `is_error` results saying the tool was interrupted.

#### A memory file's life

```
absent ─(first note of the day / setup)─► present ─(tool write)─► present + preimage (USER.md, MEMORY.md)
present ─(user edit, browser or editor)─► present (notice at next message, re-indexed at next search)
present ─(undo)─► previous content
present ─(forget line)─► present without it + tombstone   ·   daily note ─(forget file)─► absent + tombstone
```

## Interfaces

### Memory Commands Between the Webview and the Core

These commands extend the surfaces of the onboarding and chat pages (see the chat and onboarding Interfaces). The same rules apply: thin handlers, validation in the core, expected failures as values, each command declared in `build.rs` and allowed in `src-tauri/capabilities/main.json`, and `src/api/commands.ts` as the only caller.

`path` is always workspace-relative and must be one of `SOUL.md`, `USER.md`, `MEMORY.md` or `memory/YYYY-MM-DD.md` with a valid date. Anything else, including `..`, an absolute path or another file, is `InvalidInput("path")`.

#### New commands

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `memory_files` | — | `MemoryFile[]` | Soul, profile and long-term memory first, then daily notes newest first. Reads the folder; no indexing. | FR-025 |
| `memory_read` | `path` | `MemoryDoc` | The whole file, its hash, its entries and whether Undo is available. A missing daily note is `InvalidInput("path")`. | FR-025, FR-028 |
| `memory_save` | `path`, `text`, `baseHash` | `SaveOutcome` | If the file's current hash is `baseHash`, writes `text` exactly (atomically) and returns `Saved`. Otherwise writes nothing and returns `Conflict` with the current text. `text` of the soul, profile or long-term memory may exceed the budget; it is saved and the budget warning follows (FR-015). Writes no preimage: this is the user's edit. | FR-025, FR-029 |
| `memory_search` | `query` (1–500 characters) | `SearchHit[]` | Up to 20 hits over long-term memory and daily notes, after the freshness check (R12). An unusable index is an `Internal` error, and the browser says search is unavailable. | FR-026 |
| `memory_forget` | `path`, `target: ForgetTarget`, `baseHash` | `ForgetOutcome` | Valid only if the file's hash is `baseHash` (otherwise `InvalidInput("changed")`; the browser reloads). Removes the target, re-indexes, deletes preimages holding the removed text, appends a `forgotten` entry (R14). `File` is valid for daily notes only. `Entry` is valid for `MEMORY.md` only. The webview confirms before calling. | FR-027 |
| `memory_undo` | `path`, `force: boolean` | `UndoOutcome` | `USER.md` and `MEMORY.md` only. Restores the most recent preimage and deletes it. `Edited` when the file changed since that write and `force` is false. `Nothing` when no preimage exists. | FR-009, FR-028 |
| `open_workspace_folder` | — | — | Opens the workspace folder in the system file manager through the opener plugin, from Rust. | FR-025 |
| `set_auto_recall` | `enabled: boolean` | `ChatState` | Writes `recall.auto` to `settings.json`. Takes effect from the next message. | FR-020 |

#### Changes to chat commands

| Command | Change |
|---|---|
| `send_message` | Before the user message is saved: auto-recall (when on and the index is available) adds the recall block and the `recalled` marker. The context opener holds today's and yesterday's notes when this message starts the first context. Replies may run the tool loop; the channel also receives `MemoryWritten`. |
| `retry_reply`, `regenerate_reply` | Gain `clock: LocalClock`, so tools that write a daily note know the user's local date and time. `regenerate_reply` replaces the whole last turn. `retry_reply` also answers an unanswered `tool_use` (R3). |
| `start_first_hello` | Unchanged input. The new context carries the notes snapshot. |
| `start_fresh_context` | Gains `clock: LocalClock` for the notes snapshot (R8). |
| `chat_state` | `ChatState` gains `contextOutdated`, `memoryFull` and `autoRecall`. |
| `chat_history` | Companion messages stand for whole turns and carry `memory` marks. |

#### Startup

The shell starts the index sync on a background thread after the core is created (R12). It is not a webview command.

#### Capability file additions

`allow-memory-files`, `allow-memory-read`, `allow-memory-save`, `allow-memory-search`, `allow-memory-forget`, `allow-memory-undo`, `allow-open-workspace-folder`, `allow-set-auto-recall`. No plugin permission is added: the opener is called from Rust only.

### Memory Tools in the Chat Request

This slice changes the chat request (see the chat Interfaces) for contexts with harness `memory-1`. Contexts with harness `chat-1` send exactly the chat request. Every wire detail stays in `crates/steve-core/src/llm/`; the core passes tool specs (name, description, input schema) and the `llm` module adds `strict` and `eager_input_streaming`.

#### Request changes

```json
{
  "model": "claude-opus-5-5",
  "max_tokens": 16000,
  "stream": true,
  "tools": [ "...the five tools below, in this order..." ],
  "system": [
    { "type": "text", "text": "<harness rules memory-1>" },
    { "type": "text", "text": "<IDENTITY.md>\n\n<SOUL.md>" },
    { "type": "text", "text": "<USER.md>\n\n<MEMORY.md>", "cache_control": { "type": "ephemeral" } }
  ],
  "messages": [
    { "role": "user", "content": [
      { "type": "text", "text": "<memory>Your daily notes from yesterday and today (your own notes, not typed by the person):</memory>\n\n## 2026-11-13\n\n..." }
    ] },
    { "role": "user", "content": [
      { "type": "text", "text": "<context>Local time: ...</context>" },
      { "type": "text", "text": "<memory>Possibly relevant notes from your memory (found automatically, not typed by the person):</memory>\n\n- 2026-10-23 (memory/2026-10-23.md, line 4): Bimo hides under the bed when it thunders." },
      { "type": "text", "text": "There's a big storm tonight." }
    ] },
    { "role": "assistant", "content": [ "...", { "type": "tool_use", "id": "toolu_…", "name": "memory_note", "input": { "text": "…" } } ] },
    { "role": "user", "content": [
      { "type": "tool_result", "tool_use_id": "toolu_…", "content": "Noted in memory/2026-11-14.md, line 5." },
      { "type": "text", "text": "<notice>The user edited USER.md. From now on, follow this version:</notice>\n\n…" }
    ] }
  ]
}
```

Rules:

- `tools` is identical for every request of a context, serialized in the order below. `tool_choice` is never sent. `cache_control` stays on the last system block, which caches tools and system together.
- The opener message is present when the context's `notes` snapshot has at least one note; otherwise the request starts with the user's message as in chat.
- The recall block is present only when recall found something, and sits between the time block and the user's text.
- Notice blocks follow the `tool_result` blocks in a tool-result message, never precede them.
- An assistant message with `tool_use` is always followed by a user message answering every `tool_use` id in it.

#### Tools

Every tool is sent as `{ name, description, input_schema, strict: true, eager_input_streaming: true }`. Every schema has `"type": "object"`, `"additionalProperties": false` and lists all its properties in `required`. Properties use only `type` and `description`. Length and range limits are stated in the property descriptions and checked in the core, so the schemas avoid keywords strict mode might not accept. Which keywords strict mode accepts is **(record)**.

| Tool | Input | Description (when to call) | Result |
|---|---|---|---|
| `memory_search` | `query`: string, 1–500 | Search your memory (long-term memory and all daily notes) by keywords. Call it when the person refers to something from the past that is not in your context, or asks what you remember about something. | Up to 8 hits: `- {date or MEMORY.md} ({path}, lines {a}–{b}): {text}`, or "No matches." When the index is unavailable: "Memory search is unavailable right now." (`is_error: true`). |
| `memory_get` | `path`: string; `from_line`: integer ≥ 1; `to_line`: integer ≥ `from_line`, at most 200 lines later | Read a memory file or part of it, usually after a search, to see more context around a hit. | The lines, each prefixed with its number. |
| `memory_note` | `text`: string, 1–1,000, one line | Add a short note to today's daily note. Call it on your own whenever something worth keeping comes up: plans, events, feelings, things the person cares about. Don't note small talk. | "Noted in memory/{date}.md, line {n}." |
| `memory_remember` | `text`: string, 1–500, one line; `topic`: string, 1–60, one line; `requested_by_user`: boolean | Save a durable fact to long-term memory. Call it when the person explicitly asks you to remember something (set `requested_by_user` to true), or for a lasting fact or standing decision that will matter for months. Not for profile facts; use `profile_update` for those. | "Remembered in MEMORY.md under {topic}." or, when full, "Long-term memory is full, so this was kept in memory/{date}.md, line {n}. Suggest the person tidies their long-term memory in the memory browser." |
| `profile_update` | `key`: string, 1–40, no colon; `value`: string, 1–300, one line | Update a fact or preference about the person in their profile, replacing the old value: what to call them, their language, where they live, a stated preference. Call it as soon as the person states or changes such a fact. | "Updated USER.md: {key} is now {value}." or, over the profile's budget, "The profile is full, so this was kept in memory/{date}.md, line {n}. Suggest the person tidies their profile in the memory browser." |

`memory_get` accepts the same paths as the browser commands except `SOUL.md`, which is already in the context.

#### Response handling

| Event or field | Use |
|---|---|
| `content_block_start` with `type: "tool_use"` | A tool call at `index`; `input` arrives through `input_json_delta` and is parsed at `content_block_stop` (chat R3). |
| `stop_reason: "tool_use"` | The core runs the tools and continues the turn (R3). |
| Invalid or truncated `input` | A `tool_result` with `is_error: true`: "The input was not valid: {reason}." Nothing is written. |

Contract fixtures **(record)**: a reply with one `tool_use`, a reply with two `tool_use` blocks, and a final reply after tool results, each as SSE, replayed by `wiremock`.

## Checks

### Prerequisites

- The onboarding and chat pages set up, with a conversation in progress or a clean machine.
- Rust 1.90 or later, Node.js 22 or later, pnpm 10.
- A Claude API key with credit for the manual scenarios. The automated checks never use one.

### Run in development

```bash
pnpm tauri dev
```

### Inspect the files

- `%APPDATA%\mySteve\workspace\MEMORY.md`, `USER.md` and `memory\YYYY-MM-DD.md`: readable Markdown (Data 1).
- `%APPDATA%\mySteve\workspace\sessions\main.jsonl`: `written`, `forgotten` and tool messages appear as the companion works.
- `%APPDATA%\mySteve\state\steve.db`: the index. Deleting it loses nothing but undo history.

### Manual validation scenarios

| # | Story | Steps | Expected |
|---|---|---|---|
| 1 | Edge | Launch with a conversation from the chat page. | The main window offers a fresh context so memory can start. Before accepting, chat works as before. Accept it. |
| 2 | US1 | Say "Remember that I'm allergic to peanuts." | Before the reply ends, `MEMORY.md` has the entry with `from:user` and today's date. The reply shows a "Remembered" mark that opens the browser at the entry. |
| 3 | US1 | Say "Actually, call me Sam from now on." | `USER.md` says `- Call them: Sam`, and the old value is gone. The reply shows a "Profile updated" mark. No "USER.md updated" note follows at the next message. |
| 4 | US1 | Talk about your day for a few messages (plans, a friend's visit). | `memory\<today>.md` gains short timed notes with `from:companion`, without being asked. Marks appear on those replies. |
| 5 | US1 | Start a fresh context and ask "What do you know about me, and what did we talk about today?" | The companion uses Sam, knows the allergy and recalls today's notes. |
| 6 | US2 | Create `memory\<a date three weeks ago>.md` with `- 20:00 Bimo hides under the bed when it thunders.` Start a fresh context and say "There's a big storm tonight." | The companion asks about Bimo. `main.jsonl` shows the user message with a recall block and a `recalled` marker. |
| 7 | US2 | Ask "What did I say about Bimo?" | The companion searches (a `memory_search` call in `main.jsonl`) and answers with the date. |
| 8 | US2 | Add notes in Indonesian (`- 09:00 Rapat dengan klien besok pagi.`) and Japanese (`- 10:00 明日は天気予報で雨です。`). Ask about "rapat" and about "天気". | Both are found. |
| 9 | US2 | Turn auto-recall off in settings and repeat scenario 6's message in a fresh context. | No recall block on the message. Asking directly still makes the companion search. |
| 10 | US3 | Open the memory browser. View the soul, profile, long-term memory and today's note. | Each shows its text, entries and size against its budget. |
| 11 | US3 | Edit the profile in the browser (add `- Lives in: Bandung`), save, then ask "Where do I live?" | The file holds exactly the typed text. The reply says Bandung. A small "USER.md updated" note appears. |
| 12 | US3 | Open the profile in the browser, then in a text editor change a line and save. Back in the browser, edit and save. | The browser says the file changed and offers to reload or keep your version. Nothing is overwritten silently. |
| 13 | US3 | After scenario 3, choose Undo on the profile. | `USER.md` is byte for byte what it was before the change. Undo again steps back further, if there are earlier changes. |
| 14 | US3 | Edit `USER.md` in a text editor after a companion change, then choose Undo. | The browser warns that the file was edited since, and asks before undoing. |
| 15 | US3 | Search the browser for "Bimo". Choose "Open folder". | Hits show file, date and text and open the right place. The workspace folder opens in Explorer. |
| 16 | US4 | Starting from the main window, time how long it takes to find the peanut entry and forget it. Cancel once, then confirm. | Done within 1 minute (SC-010). Cancel changes nothing. Confirm shows the text first, then removes it from `MEMORY.md`. The browser says the companion may still see it until a fresh context and offers one. |
| 17 | US4 | Search the browser and ask the companion about peanuts in a fresh context. Then try Undo on `MEMORY.md`. | No hit, the companion doesn't know, and Undo can't restore the forgotten entry. `main.jsonl` has a `forgotten` entry with no text. |
| 18 | US4 | Forget a whole daily note. | The file is gone after confirmation and no longer listed or found. |
| 19 | US5 | In a text editor, add a line to yesterday's note, then search for it in the browser within 5 seconds. | Found. Remove it and search again: not found. |
| 20 | US5 | Quit, delete `state\steve.db`, relaunch, and chat right away. | Chat works immediately. Searches return the same results as before once the rebuild ends (seconds). |
| 21 | US5 | Make `MEMORY.md` longer than 12,000 characters, then ask the companion to remember something. | The fact goes to today's note with a "kept in today's note" mark, and the main window says long-term memory needs tidying and offers the browser. |
| 22 | US5 | Make today's note longer than 4,000 characters and start a fresh context. | The app says the note was shortened. The file is untouched. |
| 23 | Edge | Ask the companion to change its own soul. | It suggests the change and points to the memory browser; `SOUL.md` is unchanged. |
| 24 | Edge | Ask the companion to forget something. | It explains forgetting happens in the memory browser and nothing is removed. |
| 25 | Edge | End `mysteve.exe` in Task Manager while a reply is calling a tool (ask it to remember several things at once). Relaunch and Retry. | The conversation continues; the companion is told a tool was interrupted. Nothing is written twice by the retry. |
| 26 | Accessibility | With Narrator on, use the memory browser keyboard only: open each file, edit, save, forget with confirmation, undo, search. | Every control is reachable and named, and the confirmation dialog takes focus. |
| 27 | SC-011 | Capture network traffic for a session that remembers, searches and recalls. | `mysteve.exe` connects only to `api.anthropic.com`. |

### Release-readiness checks (not run per commit)

| Check | How | Pass |
|---|---|---|
| SC-001 recall | The scripted suite of 30 facts across simulated days, against the real API, fresh context a simulated week later. | At least 27 recalled correctly. |
| SC-002 remember | 20 scripted "remember that…" requests against the real API, each in its own turn. | At least 19 entries are in `MEMORY.md` when their reply ends. |
| SC-003 profile | 10 scripted profile changes (name, language, city, preferences) against the real API. | `USER.md` holds only the new value after each of the 10. |
| SC-004 latency | `cargo test -p steve-core --release --test memory_index -- --ignored --nocapture search_over_10000_chunks` | 95th percentile 0.1 s or less for search, 0.15 s or less for request preparation with recall. |
| SC-009 languages | The fixed query set in `crates/steve-core/tests/fixtures/languages/` run by the same ignored test. | At least 95% found. |

