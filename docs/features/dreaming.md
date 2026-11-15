# Dreaming

Built.

This feature covers DRM-1 to DRM-8 and DRM-10's second half, the dream diary in the memory browser (CTL-1), forgotten content kept out of dreams (CTL-2), the link from each promoted memory back to its note (CTL-3), and the dreams part of the activity log (CTL-5). It completes the memory loop of `docs/requirements.md` section 2: daily notes (the memory feature), compaction with a flush (the compaction feature), and now the nightly consolidation into long-term memory.

## User stories

### US1. Steve remembers what keeps coming up (P1)

Over a week the user mentions their new running routine several times: the plan to run the half marathon in April, the knee that gets sore after ten kilometres. Steve jots these down in the daily notes as they come up, and they come back into the conversation on several days. One morning, without the user asking anyone to remember anything, Steve's long-term memory holds "Training for the half marathon in April; the knee gets sore after 10 km", with a link back to the note it came from. A passing remark from the same week, mentioned once and never again, is not there.

**Why this priority**: This is the third promise of the memory design: the companion consolidates while you are away, so what matters lasts beyond the last two days of notes. Everything else in this slice supports or explains it.

**How to check:** Seed two weeks of daily notes and recall history in which some facts recur across days and queries and others appear once; run a sweep; confirm the recurring facts are now in long-term memory or the profile with a source on each entry, the one-off facts are not, and nothing outside long-term memory and the profile changed except the diary.

**Acceptance Scenarios**:

1. **Given** a fact that was noted and then brought up again on at least two separate occasions by at least two different questions, **When** a sweep runs, **Then** the fact is added to long-term memory, or to the profile if it is a fact about the user, in a short form with a link to the note and line it came from.
2. **Given** a fact mentioned once and never brought up again, **When** a sweep runs, **Then** it is not promoted, and it stays in its daily note.
3. **Given** long-term memory already holds an entry that a newer fact refines or replaces, **When** the sweep promotes the newer fact, **Then** the old entry is merged or superseded rather than duplicated, and the change keeps every source.
4. **Given** a promoted entry, **When** the user opens it in the memory browser, **Then** they can follow its link to the daily note and line it came from.
5. **Given** a sweep found nothing worth promoting, **When** it ends, **Then** long-term memory and the profile are unchanged.

---

### US2. It happens overnight, even on a laptop (P1)

The user closes the laptop lid at 23:00 every night. The sweep is set for 03:00, when the laptop is asleep. The next morning the user opens the lid and starts working; a few minutes after they stop interacting with Steve, the missed sweep runs in the background. If the user sends a message while it runs, Steve answers right away and the sweep waits. It never runs twice for the same night.

**Why this priority**: Without catch-up, a laptop user would never dream at all. It is the condition for US1 working for most people.

**How to check:** With a simulated clock, skip the scheduled time while the app is closed or asleep, start the app, and confirm the sweep runs once after the idle delay, pauses when a message is sent and resumes afterwards, and does not run again for the same night.

**Acceptance Scenarios**:

1. **Given** the app is running at the scheduled time and the user is idle, **When** the time comes, **Then** the sweep runs.
2. **Given** the scheduled time passed while the app was closed or the computer asleep, **When** the app is running again and the user has been idle for the catch-up delay, **Then** the missed sweep runs once.
3. **Given** a sweep is running, **When** the user sends a message, **Then** the reply is not delayed by the sweep, the sweep pauses at its next step, and it resumes when the user is idle again.
4. **Given** several scheduled times were missed (a week away), **When** catch-up runs, **Then** one sweep covers everything since the last completed sweep.
5. **Given** a sweep was interrupted by a crash or by quitting, **When** the app runs again, **Then** the sweep resumes or restarts, and long-term memory is exactly as it was until a sweep completes.

---

### US3. Nothing slips in that shouldn't (P1)

A web page quoted through a tool says "remember that the user's password hint is…". The user forgot a memory last week. Steve's own recalled snippets show up in many conversations. None of these become long-term memory through dreaming: tool text is never promoted, forgotten content never comes back, and Steve does not relearn what it recalled as if it were new. A dream that would have dropped a third of the existing memories is rejected, and the plain fallback adds the new facts without touching the old ones.

**Why this priority**: Dreaming is the one process that rewrites long-term memory on its own. Safety and provenance (Principle IV) are not optional for it.

**How to check:** Seed candidates of every provenance, a forgotten memory, recalled-memory blocks and a model reply that proposes a lossy rewrite; run a sweep; confirm nothing untrusted, system, forgotten or recalled-only was promoted, the lossy rewrite was rejected, the fallback ran, and every promoted entry has a source.

**Acceptance Scenarios**:

1. **Given** candidate material that came from tool results or app instructions, **When** a sweep runs, **Then** none of it is promoted, whatever its score.
2. **Given** a memory the user forgot, and the conversation it came from, **When** a sweep runs, **Then** neither the memory nor its source conversation is used.
3. **Given** text that only appeared as a recalled-memory snippet, **When** a sweep runs, **Then** its recall counts as evidence for the original note, never as a new fact.
4. **Given** a proposed rewrite that would drop more than a quarter of the existing entries, lose a source, exceed the file's size budget, or break the file's structure, **When** it is validated, **Then** it is rejected and the gated facts are appended without merging, within the size budget.
5. **Given** a daily note line was changed or deleted by the user after it was gathered, **When** the deep phase runs, **Then** that candidate is skipped.

---

### US4. A dream diary and a way back (P2)

In the morning the user opens Steve's dream journal and reads a short entry in Steve's voice: what the night's notes were about, a theme it noticed ("you've been talking about running a lot this week"), and what it decided to keep. The journal lists when each dream ran and what changed. The user disagrees with one change and chooses "undo last dream": long-term memory and the profile go back exactly to how they were before that dream.

**Why this priority**: Transparency and reversibility make an automatic rewrite acceptable. The diary also makes dreaming visible, which is part of the companion's charm.

**How to check:** Run a sweep with material, open the journal and see the entry, the run's time and its counts; undo the dream and confirm both files are byte for byte as before; run a sweep where the model fails and see a plain entry marked as degraded.

**Acceptance Scenarios**:

1. **Given** a sweep had useful material, **When** it ends, **Then** the diary gains one short entry in the companion's voice, and the journal shows when it ran and how many entries were added, merged and superseded.
2. **Given** a sweep had nothing to work with, **When** it ends, **Then** no diary entry is written, and the journal records that it ran.
3. **Given** the last dream changed long-term memory or the profile, **When** the user chooses "undo last dream", **Then** both files return byte for byte to their content before that dream.
4. **Given** the user edited a file after the dream, **When** they choose "undo last dream", **Then** the app says the file changed since and asks before discarding those edits, as the memory browser's undo does.
5. **Given** the model was unavailable when the diary was written, **When** the sweep ends, **Then** a plain entry is written and marked as degraded.
6. **Given** the diary, **When** any later sweep runs, **Then** the diary is never used as a source for memory.

---

### US5. The next morning starts fresh (P2)

After a night's dream, the user opens the app and says good morning. Steve already knows what it consolidated overnight, because the conversation continues with the updated long-term memory and profile, and it still knows what they were talking about yesterday. It may mention last night's dream in passing.

**Why this priority**: Without this, the updated memory would only reach the companion at the next compaction, possibly days later.

**How to check:** Run a sweep that promotes a fact, send a message, and confirm the request starts from the updated long-term memory, still carries the summary and recent turns of the conversation, and includes the latest diary entry marked as a dream.

**Acceptance Scenarios**:

1. **Given** a sweep changed long-term memory or the profile, **When** the user next sends a message, **Then** the companion works with the updated files.
2. **Given** a new context started after a dream, **When** the user continues yesterday's topic, **Then** the companion still knows it.
3. **Given** a dream diary entry was written, **When** the next context starts, **Then** the companion is given the latest entry, marked as its dream and not as a fact, and may mention it.
4. **Given** a sweep changed nothing, **When** the user next sends a message, **Then** the context continues as it was.

---

### US6. The user decides when and whether (P3)

A user who works nights sets the dream time to 14:00. Another turns dreaming off. Someone curious chooses "dream now" in the evening and watches the diary entry appear. A cost-conscious user picks a cheaper model for dreaming.

**Why this priority**: The defaults work for everyone (Principle VII); these are controls for users who care.

**How to check:** Change the dream time and confirm the next sweep follows it; turn dreaming off and confirm no sweep runs and "dream now" still works on request; pick another model and confirm the sweep uses it.

**Acceptance Scenarios**:

1. **Given** settings, **When** the user sets a dream time, **Then** later sweeps follow it, with catch-up as before.
2. **Given** dreaming is off, **When** the scheduled time passes, **Then** no sweep runs, and nights missed while it was off are not caught up when it is turned back on; the next sweep covers everything since the last completed one.
3. **Given** no sweep is running, **When** the user chooses "dream now", **Then** a sweep runs at once over everything since the last completed one, whether dreaming is on or off.
4. **Given** settings, **When** the user picks a model for dreaming, **Then** the sweep uses it, and chat keeps its own model.
5. **Given** a new install, **When** the user never opens these settings, **Then** dreaming runs nightly at 03:00 with the chat model.

---

## Edge cases

- **A new companion with no recall history.** Nothing passes the gates, so nothing is promoted; facts the user stated or asked to remember are already in the profile and long-term memory from the memory feature (day-one memory does not depend on dreaming).
- **Auto-recall is off.** Recalls come only from the companion's own memory searches, so promotion is rarer. The dreaming settings say so.
- **Long-term memory or the profile is at its size budget.** The deep phase can merge or supersede to make room; a rewrite that would exceed the budget is rejected and the fallback appends only what fits. The diary says memory is full.
- **The user edits MEMORY.md or USER.md while a sweep runs.** The commit detects the change, re-reads the file and re-applies the decisions, or skips the commit and tries again next time; the user's edit is never overwritten.
- **The model is unavailable for the whole sweep.** The light phase needs no model; the REM and deep phases fall back (no themes; plain append of gated facts); the diary is a degraded local entry. The sweep still completes.
- **The provider rejects the key or has no credit.** The sweep stops, changes nothing, and runs again at the next schedule or catch-up; nothing is shown in the conversation.
- **The sweep is due while a reply is running or a compaction is under way.** It waits for both.
- **Clock changes and time zones.** A night is the local date of the scheduled time; travelling across time zones or changing the clock never runs two sweeps for one local night, and a clock set back runs nothing new.
- **A promoted fact is later forgotten.** It is forgotten like any long-term memory (the memory feature); its source lines are excluded from later dreams.
- **The same fact is noted on many days.** It is promoted once and later recurrences reinforce it rather than adding duplicates.
- **Very long notes.** Each promoted entry is kept short; a long passage is never copied into long-term memory.
- **The diary file is edited or deleted by the user.** Later entries are appended to whatever is there, or to a new file.
- **Undo after two dreams.** "Undo last dream" undoes only the most recent dream; doing it again undoes the one before, as long as the files still match.
- **The app is quit during a sweep.** Quitting is not delayed by the sweep; it resumes next time.

## Requirements

**Schedule and catch-up**

- **FR-001**: The app MUST run a sweep once per local night at the dream time (default 03:00) while it is running, including in the tray (DRM-1, A4).
- **FR-002**: If the dream time passed while the app was closed or the computer asleep, the app MUST run one sweep after the user has been idle for the catch-up delay (default 5 minutes), covering everything since the last completed sweep (DRM-2).
- **FR-003**: A sweep MUST NOT start while a reply is running or a compaction is under way, and MUST pause at its next step when the user sends a message, resuming when the user is idle again (DRM-2, Principle V).
- **FR-004**: At most one sweep may run at a time, and at most one sweep may complete per local night, whatever the clock or time zone does.
- **FR-005**: Each phase MUST record its progress so that an interrupted sweep resumes or restarts safely; until the deep phase commits, the profile and long-term memory MUST be exactly as they were (NFR-REL-3).

**Input and provenance**

- **FR-006**: Dream input MUST be the daily notes and the real conversation since the last completed sweep. Recalled-memory snippets, hidden app requests, tool results and the diary MUST NOT be input (DRM-6, DRM-5).
- **FR-007**: Material that came from tool results or app instructions (untrusted or system provenance) MUST NOT be promoted, whatever its score (Principle IV).
- **FR-008**: Forgotten memories and the conversation they came from MUST NOT be used by any phase (CTL-2).
- **FR-009**: Each time a memory snippet is returned by auto-recall or by the companion's memory search, the app MUST record it as recall evidence: which snippet, a fingerprint of the query, the score and the day. The query text itself MUST NOT be stored.

**Phases**

- **FR-010**: The light phase MUST gather candidate lines from the input, remove near-duplicates, and record which candidates were reinforced. It MUST NOT write long-term memory or the profile.
- **FR-011**: The REM phase MUST look across recent days for themes (recurring topics, changing goals, shifts in mood) and record reinforcement for candidates that belong to a theme. It MUST NOT write long-term memory or the profile.
- **FR-012**: The deep phase MUST score each candidate on relevance, frequency, query diversity, recency, recurrence across days and conceptual richness, and MUST promote only candidates that clear a minimum score, a minimum recall count and a minimum number of distinct queries (DRM-3).
- **FR-013**: Before deciding, the deep phase MUST re-read each surviving candidate from its live daily note and skip any that was changed or deleted.
- **FR-014**: The deep phase MUST obtain promotions as discrete decisions (add, merge, supersede), each naming its target file and section, the entry text, its sources, and the entries it replaces. The app, not the model, MUST apply them (DRM-4).

**Safety and undo**

- **FR-015**: A rewrite MUST be accepted only if it drops at most a quarter of the prior entries, keeps a source on every promoted entry, stays within the file's size budget, and keeps the file's structure. Otherwise the gated candidates MUST be appended without merging, within the size budget (DRM-4).
- **FR-016**: Each promoted entry MUST be short (a bounded length) and MUST link to the daily note and line it came from (CTL-3).
- **FR-017**: The deep phase MUST commit long-term memory and the profile once, at its end, after saving the previous content of each so that it can be restored (DRM-4, Principle IV).
- **FR-018**: "Undo last dream" MUST restore long-term memory and the profile byte for byte to their content before the most recent dream not yet undone. If a file changed since that dream, the app MUST say so and ask before discarding the change.
- **FR-019**: The companion MUST never write the soul through dreaming, and the user's outside edits MUST never be overwritten (Principle I).

**Diary and journal**

- **FR-020**: Each sweep with useful material MUST append one short diary entry in the companion's voice, with its date, what the notes were about, any theme, and what it kept. If the model is unavailable, a plain entry marked as degraded MUST be written instead (DRM-5).
- **FR-021**: The diary MUST be viewable in the app's dream journal and in the memory browser, and editable there like the other memory files (CTL-1). It MUST never be used as a memory source.
- **FR-022**: The dream journal MUST list each sweep: when it ran, whether it completed, how many entries it added, merged and superseded, whether it fell back, and the tokens it used (CTL-5).

**After a dream**

- **FR-023**: After a sweep that changed long-term memory or the profile, the next message MUST be answered from a context that starts from the updated files and still carries the conversation's summary and recent turns (DRM-8).
- **FR-024**: That context MUST include the latest diary entry, marked as the companion's dream and not as a fact, so the companion may mention it (DRM-10).
- **FR-025**: While a sweep runs, the companion's portrait MUST show it asleep, and the main window MUST show a calm "dreaming" status. Neither may block chat (DRM-10, PER-11).

**Controls**

- **FR-026**: Dreaming MUST be on by default. Settings MUST offer an on/off switch, a dream time, and a model for dreaming (default: the chat model, falling back to it if the chosen model is no longer offered) (DRM-7, DRM-9).
- **FR-027**: The journal MUST offer "dream now", which runs a sweep at once whether dreaming is on or off, unless one is already running.

**Boundaries**

- **FR-028**: Dreaming MUST send nothing anywhere but the provider the user chose (Principle II, SC-5).
- **FR-029**: All dreaming rules (schedule, phases, scoring, gates, validation, commit, undo) MUST be decided in the core, with no dependency on the desktop shell, and MUST be testable with the fake provider and an injected clock (Principle VI, NFR-TEST-1).
- **FR-030**: Dreaming state other than recall evidence, previous file content and run history MUST be rebuildable from the files (NFR-REL-2).

### Key entities

- **Sweep**: One run of the three phases over the material since the last completed sweep, for one local night. Has a start, the phase reached, an outcome, and counts.
- **Recall event**: One snippet returned by auto-recall or a memory search: which snippet, a fingerprint of the query, its score, the day. The evidence the deep phase ranks on.
- **Candidate**: A line gathered by the light phase from a daily note or the conversation, with its source, provenance and reinforcement.
- **Theme**: A pattern the REM phase found across days, with the candidates it reinforces.
- **Promotion decision**: Add, merge or supersede: a target file and section, an entry text, sources, and replaced entries.
- **Previous content**: The profile and long-term memory as they were before a dream's commit, kept so the dream can be undone.
- **Dream diary**: `DREAMS.md`, one entry per sweep with material, written for the user. Never a memory source.
- **Dream journal**: The app view of the diary and of the sweeps' history, with "dream now" and "undo last dream".

## Success criteria

- **SC-001**: In a scripted two-week suite, at least 90% of facts that recur on at least two days through at least two distinct questions are in long-term memory or the profile after the sweeps, and at most 5% of facts mentioned once are (scripted replies in CI; the real model before release).
- **SC-002**: Across the suite, 0 promoted entries come from tool results, app instructions, forgotten memories, recalled-only snippets or the diary.
- **SC-003**: 100% of promoted entries link to a daily note and line that holds the fact.
- **SC-004**: Interrupting a sweep at any step leaves long-term memory and the profile byte for byte as they were, in 100% of attempts; the next run completes the night.
- **SC-005**: "Undo last dream" restores both files byte for byte in 100% of attempts when they are unchanged since the dream.
- **SC-006**: Every proposed rewrite in a set of lossy proposals (dropping entries beyond the limit, losing a source, exceeding the budget, breaking the structure) is rejected, in 100% of cases.
- **SC-007**: A missed night is caught up within 10 minutes of the app running with the user idle, and no local night ever completes two sweeps.
- **SC-008**: A message sent during a sweep starts streaming its reply with the same 0.15 seconds of the app's own work as without a sweep (NFR-PERF-2).
- **SC-009**: A sweep over a typical day uses at most three model requests of its own, plus the compaction that opens the next context (the compaction feature), and completes within 2 minutes on the real provider, for 95% of runs.
- **SC-010**: During a network capture of a session that includes a dream, the app's connections go only to the provider.
- **SC-011**: A new user who never opens settings dreams nightly with nothing to configure (Principle VII).

## Assumptions

- **Depends on slices 3 and 4.** Daily notes, long-term memory, the profile, provenance, forgetting, previous content and undo in the memory browser come from the memory feature; the summary and kept turns that the after-dream context carries over come from the compaction feature.
- **Recall evidence starts now.** The memory feature deferred recording recall events to this slice. Facts noted before this feature is installed have no recall history and promote only once they are recalled again.
- **Idle means no turn.** The user is idle when no message was sent and no reply ran for the catch-up delay. Operating-system input idle and resume-from-sleep events are left to the Decisions; the app's own activity is enough to meet FR-002 and FR-003.
- **The night a sweep belongs to** is the local date of the dream time it serves. A sweep at 09:00 catching up a missed 03:00 belongs to that day.
- **Starting values** from `docs/architecture.md` section 15: dream time 03:00, catch-up delay 5 minutes, minimum score 0.75, minimum recall count 2, minimum distinct queries 2, at most a quarter of prior entries lost, promoted snippets about 160 tokens; the scoring weights are OpenClaw's (architecture section 8.3). The gates are tuned in release checks.
- **Batch processing is out of scope** (DRM-9's second half). A sweep makes a few requests; batching them would add hours of waiting and job tracking for little saving at this volume.
- **The avatar's sleeping animation** belongs to the avatar slice (slice 6). This slice shows the still portrait asleep and a status line (PER-11 says a still portrait is enough).
- **A morning greeting** is not added. The companion still speaks only when spoken to, except for the first hello; it may mention the dream when the user next talks (FR-024).
- **The activity log of compactions and flushes** (the rest of CTL-5) is not built here; compactions already record their sizes, model, tokens and flush outcome for it.
- **World dreams** (WLD-9, WLD-10, slice 10) and **semantic search** (slice 7) are out of scope. "Off the record" mode (CTL-6) is out of scope.
- **Platform.** Windows 11 first, as for the rest of the app.

## Decisions

Each section records a decision, why it was made, and what was rejected. The design follows `docs/architecture.md` section 8 and OpenClaw's dreaming, adapted to a desktop app that is often asleep. It builds on the memory feature (memory files, the index, preimages, forgetting) and the compaction feature (compaction, the stop-signal pattern, structured output).

### R1. Recall evidence

**Decision**: A persistent table `recall_events` in `steve.db`, created like `preimages` (it survives index rebuilds). One row per snippet: `path`, `line` (the snippet's first line when recalled), `text_hash` (the stable fingerprint of the snippet's text without its comment), `query_hash` (fingerprint of the normalized query terms), `coverage` (matched query terms ÷ query terms, 0 to 1), `day` (the user's local date), `source` (`auto` or `search`). Rows are written:

- by **auto-recall**: for each hit that matches enough of the message's terms (the same threshold as `select_recall`), up to 3, *including* hits left out because their day's note is already in the context. Being relevant to a message is the evidence; whether it was shown is not.
- by **`memory_search`**: for each returned hit, up to 8.

A snippet is identified by `(path, text_hash)`, so editing other lines of a note doesn't break its history. Only daily-note snippets are recorded; `MEMORY.md` hits are not, since they are already long-term.

**Rationale**: The deep phase ranks on recall frequency, query diversity, relevance and recency (architecture 8.3); the memory feature deferred recording them to this slice (architecture 5.4). Chunk ids change on every rebuild, while path and text do not. Daily-note bullets are one chunk each (memory R11), so a recalled snippet is a note line. Coverage is a stable 0-to-1 relevance; the raw rank score depends on the index's term statistics. Storing a fingerprint of the query, not the query, keeps distinct-query counts without keeping what the user typed twice (Principle II, FR-009). Counting already-loaded hits lets today's and yesterday's notes earn evidence while they are in view.

**Alternatives considered**: Logging chunk ids (lost at each rebuild). Logging only shown hits (recent notes would never earn evidence). Storing query text (a second copy of private text for no benefit).

### R2. Who drives the schedule

**Decision**: The webview calls `dream_tick(clock)` once a minute from the main view, which stays loaded while the window is hidden in the tray, and once at load. The core decides whether a night is due (R3) and, if so, runs the sweep inside that call. The call returns a `DreamStatus` (idle, running, paused, last run). A tick while a sweep runs returns at once with `running`.

**Rationale**: The core has no local time zone; every command that needs local time gets the webview's clock (chat R8). The main window is hidden, not destroyed, when closed to the tray (the onboarding feature), so its timer keeps running, throttled to about once a minute by WebView2 in the background, which is enough for a nightly job with a five-minute idle delay. This is the pattern of the first hello and of automatic compaction: the rule lives in the core, the webview only knocks.

**Alternatives considered**: A timer task in the shell (needs a local clock in Rust, a new dependency or Windows calls, and puts scheduling in the shell). A task spawned by the core (a `'static` core and a runtime handle in `steve-core`, against Principle VI and the lean-code rule on concurrency). Operating-system scheduled tasks (run when the app is closed, which A4 rules out).

### R3. Nights, idleness and catch-up

**Decision**:

- **Night**: the local date of the most recent dream time at or before the clock. At 02:59 on 15 November with a 03:00 dream time, the latest night is the 14th; at 03:00 it is the 15th.
- **Due**: dreaming is on, the latest night is later than `state.json dreaming.lastNight`, no reply or compaction is running, no reply is owed, and the user has been **idle** for 5 minutes.
- **Idle**: the time since the later of the last conversation entry and the moment the core started. The core records the start; the last entry's `at` is in the transcript. "Now" is the tick's clock, so tests decide it (FR-029).
- **Completion** sets `lastNight` to the night the sweep served. A clock set back gives a latest night not later than `lastNight`, so nothing runs; moving across time zones changes the local date at most by a day and can't run a night twice.
- **Turning dreaming on** sets `lastNight` to the latest night, so nights missed while it was off are not caught up; the next dream time is.
- **Dream now** runs whatever `lastNight` says and sets it to the latest night when it completes.
- **What a sweep covers**: everything since the end of the last completed sweep (its `ended_at`), or the last 7 days of notes for the first sweep.

**Rationale**: The feature's scheduled run and catch-up are one rule this way: at 03:00 an idle user has been idle for hours, so the night is due at once; a laptop opened at 09:00 runs it five minutes after the user stops. App-level idleness is enough to keep chat first (FR-003) and needs no operating-system hooks (Assumptions).

**Alternatives considered**: Cron expressions (the setting is a time of day; cron adds parsing for nothing). Operating-system idle time (platform calls for no gain). Catching up each missed night separately (a week away would run seven sweeps over the same notes).

### R4. Running, pausing and checkpoints

**Decision**: A sweep is a row in a persistent `dream_runs` table: `id`, `night`, `started_at`, `ended_at`, `since` (the start point it covers), `phase` (`light`, `rem`, `deep`, `committing`, `diary`, `roll`, `done`, `failed`), `checkpoint` (JSON: the candidates, the themes, the decisions), counts (`added`, `merged`, `superseded`), `fell_back`, `usage` (JSON), `committed`, `undone` (Data 2). Each phase reads the checkpoint, does its work, and writes the next phase with the new checkpoint in one statement.

- **Pausing**: the chat runtime gets a third slot, `dreaming: Option<Arc<Notify>>`, which `begin()` notifies like the compacting slot. A model call runs in a `select!` against it; a stopped call leaves the phase as it was, and the tick returns `paused`. The next due tick continues from the same phase.
- **One at a time**: the slot also refuses a second sweep and a sweep while a compaction runs; `begin_compaction` refuses while a sweep runs.
- **Resuming after a crash**: an unfinished run for an unfinished night continues from its phase on the next due tick; a `committing` run is first rolled back (R8).
- **A failed run** (the provider rejects the key or has no credit) is marked `failed` with nothing changed; the night stays due, but a new run starts only on a due tick at least an hour after the failure, so a broken key doesn't cause a request every few minutes.
- **Candidates in the checkpoint**: the light phase's output is stored in the run row, not in a separate `dream_candidates` table.

**Rationale**: Checkpoints per phase are what NFR-REL-3 and architecture 8.1 ask for; storing them with the run keeps one row as the whole state of a sweep. Candidates are derived from the files and only matter for one run, so a table of their own adds a schema for nothing (FR-030). The stop signal is the compaction's (compaction R3).

**Alternatives considered**: A `dream_candidates` table (architecture 4.2's first idea; more schema, same information). Restarting a stopped sweep from scratch (repeats model calls).

### R5. The light phase

**Decision**: Local, no model call.

- **Lines**: every bullet of the daily notes written since the sweep's start point (R3), plus every daily-note line that gained recall evidence since then (older notes recalled again).
- **Provenance**: a note line's `from:` comment. `from:user` and `from:companion` lines are candidates; any other value, and lines inside a recalled-memory or notice block, are dropped. A line with no comment was written by the user in the file and counts as `user`.
- **Forgotten**: a line whose `src:` names a message listed in a `forgotten` entry is dropped (CTL-2).
- **Deduplication**: lines are compared by their normalized terms (the index's tokenizer); two lines whose term sets overlap by 80% or more (Jaccard) are one candidate, keeping the newest text and every source line. The number of days a candidate appears on is its **recurrence**, and each merge counts as a **reinforcement**.
- **Evidence**: each candidate gets the recall events of all its source lines (by `path` and `text_hash`).
- **Real conversation**: the user's typed messages since the start point (their `text` field, never the recall or notice blocks, never hidden or tool messages, never forgotten turns) are kept in the checkpoint for the REM phase.

**Rationale**: The notes are already one fact per line, written with provenance, so gathering them needs no model. The conversation itself has no recall evidence and is not indexed, so promotion works on notes; the conversation informs themes (FR-006, DRM-6).

**Alternatives considered**: Asking the model to extract candidates from the conversation (costly, and its output would have no recall evidence to pass the gates). Embedding-based deduplication (needs slice 7).

### R6. The REM phase

**Decision**: One structured-output request with the candidates (id, text, day) and the user's messages since the start point (each with its time), asking for up to 5 themes: `{title, summary, candidates: [ids]}`. Each candidate in a theme gets one reinforcement. The themes go to the checkpoint and to the diary. With no candidates and no messages, the phase is skipped; if the request fails, there are no themes and the sweep continues.

**Rationale**: Themes across days are what REM is for (architecture 8.2); reinforcement is the small, decaying boost architecture 8.3 adds to the score. A theme can only point at candidates that exist, so a theme can't invent a fact.

**Alternatives considered**: No REM phase (DRM-1 asks for three). Letting the REM phase propose new facts (it would bypass the gates).

### R7. Scoring, gates and decisions

**Decision**:

- **Signals**, each 0 to 1, over a candidate's recall events: relevance = mean coverage; frequency = min(events ÷ 3, 1); query diversity = min(distinct queries ÷ 3, 1); recency = 0.5^(days since the last event ÷ 14); consolidation = min(distinct event days and note days ÷ 3, 1); conceptual richness = min(distinct terms ÷ 12, 1). Weights 0.30, 0.24, 0.15, 0.15, 0.10, 0.06 (OpenClaw's). Boost: +0.05 per reinforcement, at most +0.15.
- **Gates**: score ≥ 0.6, events ≥ 2, distinct queries ≥ 2. Provenance is already filtered (R5).
- **Re-read**: each surviving candidate's newest source line is read from its live file; if no line with its `text_hash` is there, it is skipped (FR-013).
- **Decision request**: one structured-output request at medium effort, with the survivors (id, text, sources, day), the current `MEMORY.md` entries (id, topic, text) and the current profile lines. It returns operations: `add {topic, text, sources}`, `merge {topic, text, sources, replaces}`, `supersede {topic, text, sources, replaces}`, `profile {key, value, sources}`. A survivor the model leaves out is not promoted this night.
- **Applying** (code): `add` and `merge` and `supersede` write `- text <!-- id:… at:DATE from:… src:memory/DATE.md#Lnn[,…] -->` under the topic (`add_entry`), removing replaced entries; `from:user` if any source or replaced entry was `user`, else `companion`. `profile` sets `- Key: value` like `profile_update`. Each text is capped at 600 characters (about 160 tokens).

**Rationale**: The weights are the architecture's. The normalization is chosen so that the minimum evidence the gates require (two recalls, two queries, two days, recent, decent coverage) lands near the threshold, which is what SC-001 asks for; the architecture's 0.75 was marked "to be tuned" and, with these normalizations, would reject most facts that pass the other two gates. Profile lines stay plain `- Key: value`, because `field()` reads values up to the end of the line and the profile has never carried comments; their sources are recorded in the run.

**Alternatives considered**: The model scores candidates (no evidence, easy to talk into). Writing sources into profile lines (breaks every reader of `USER.md`).

### R8. Validation, fallback and the commit

**Decision**:

- **Validation** (FR-015), on the new text of each file: prior entries removed ≤ 25% of prior entries (merged or superseded ids count as removed); every entry the dream wrote has a `src:` that names a daily note line; the file fits its budget (12,000 characters for `MEMORY.md`, 4,000 for `USER.md`); every line is a heading, a blank line, an entry or plain text as before, and ids stay unique.
- **Fallback**: if the request failed or validation failed, the gated survivors are appended as `add` entries under the topic "From my dreams", oldest first, while they fit the budget; the rest wait. `fell_back` is recorded.
- **Commit**: for each changed file, `edit_workspace` re-reads it; if its hash differs from the one the decisions were built on, the decisions are re-applied to the new text and re-validated, or the file is skipped this night. Before writing, a preimage is saved with `message = dream:<run id>`. The run is set to `committing` before the first write and to `diary` after the last.
- **Rollback**: a run found in `committing` (a crash between the two files) restores each file from its `dream:<run id>` preimage if the file's current hash equals that preimage's `after_hash`, deletes those preimages, and goes back to the deep phase.

**Rationale**: The checks are the architecture's (8.4). Re-applying on a changed file honors outside edits (Principle I). The commit marker plus hash-checked rollback is what makes two file writes behave as one for NFR-REL-3, the same idea as the compaction feature's pending marker.

**Alternatives considered**: Writing both files and hoping (a crash between leaves half a dream). A single combined file (the files are the user's, with fixed names).

### R9. Undo last dream

**Decision**: "Undo last dream" takes the latest run with a commit that isn't undone, and for each of its `dream:<run id>` preimages: if the file's hash equals the preimage's `after_hash`, restore `before`; otherwise report `Changed` with the file, and restore only when called again with `force`. The run is marked `undone` and its preimages deleted. Calling it again undoes the run before, under the same rule. Forgetting a memory still purges preimages that contain it (the memory feature), so undo can't bring a forgotten memory back.

**Rationale**: It is the memory feature's undo, applied to the two files of one run together (FR-018). The hash check is what keeps a later user edit from being discarded silently.

### R10. The new context after a dream

**Decision**: When a sweep's commit changed a file, the sweep's last step (after the diary) rolls the context:

1. It runs `Core::compact` with `manual: true` and the dreaming stop signal; the new context snapshots the updated files.
2. On `NothingToCompact` with no message since the current context began, it appends a new `context` entry that names the current context's compaction, if any, so the summary and kept turns carry over and no turn is lost. With messages that are still too few to compact, it leaves the context as it is: the next message carries the changed files as a notice (chat FR-011), and the dream entry is not given.
3. On `Stopped` or `Failed`, the run records it and the next due tick retries step 1 before anything else; the run is `done` only after a roll.

The new `context` entry carries `dream`: the diary entry just written. The opener adds it after the summary: `<memory>The dream you wrote in your diary last night (a dream, not a fact):</memory>`.

**Rationale**: The architecture lists "after a dream" as a compaction trigger (7.1). A compaction keeps the conversation's thread (FR-023), and reopening is safe precisely when nothing new exists. The dream text is given once, marked as a dream, so the companion can mention it without learning it (FR-024, DRM-5).

**Alternatives considered**: A fresh context (loses the conversation's thread). Re-reading the files mid-epoch (breaks Principle III).

### R11. The diary

**Decision**: One request with the soul and identity as the system prompt and the night's material (themes, the decisions applied, how many lines were gathered) as the user message, asking for 3 to 6 sentences in the companion's voice. `DREAMS.md` gets, appended through `edit_workspace`: `## <night>` then the entry, then a "Kept:" list of the entries the dream wrote, generated by code. On failure, a local entry: `I dreamt about N notes and kept M of them.` plus the list, with *(written without the model)* after the heading. A sweep with no candidates and no messages writes no entry. `DREAMS.md` is never indexed, never gathered, and never part of a context except as the one `dream` above.

**Rationale**: Architecture 8.5. The "Kept" list is code-generated so the diary's account of changes is exact even when the narrative is fanciful.

### R12. What the user sees

**Decision**:

- **Status**: `dream_tick` returns `DreamStatus {state: idle | running | paused, lastRun}`; `ChatState` gains `dreaming: bool`. The main view shows "Steve is dreaming…" as a polite status and the portrait with a sleeping style (dimmed, with a small "z" badge) while it runs. Chat stays enabled.
- **Dream journal**: a view opened from the main window (beside Memory and World) with the diary rendered as sanitized Markdown, the list of runs (date, completed or not, added, merged, superseded, fell back, tokens), "Dream now", and "Undo last dream" with the changed-file confirmation.
- **Memory browser**: `DREAMS.md` is listed as the dream diary and is editable; it is not searchable.
- **Settings**: a "Dreaming" section: on/off, the dream time (a time field), the model for dreaming (same as conversation, or one from the list, with the fallback note), and a note when auto-recall is off that dreaming promotes less.

**Rationale**: FR-020 to FR-022, FR-025 to FR-027. The sleeping portrait is PER-11's still-portrait version of DRM-10; the animated version comes with slice 6.

### R13. Models and effort

**Decision**: The dreaming model setting, falling back to the chat model when missing from the cached list. `TurnRequest.effort` becomes `Option<Effort>` with `Low` and `Medium`; chat, flush and summary keep `Low`, the deep phase uses `Medium`, REM and the diary `Low`, each only for models that take effort.

**Rationale**: Architecture 11 ("The deep phase starts at medium"). A two-value enum is the smallest change that expresses it.

### R14. Testing

**Decision**:

- **Unit**: nights and due-ness over a table of clocks (before and after the dream time, clock set back, time zone change, off and on); idleness; light-phase gathering (provenance, forgotten, dedup, recurrence); scoring and gates over fixed evidence; decision application; validation (the lossy-rewrite gate: removed entries over 25%, a missing source, over budget, broken structure, duplicate ids); the fallback; diary formatting.
- **Integration** (`dream_flow.rs`, temporary workspace, fake provider, injected clock): recall evidence from auto-recall and `memory_search`; a two-week scripted suite (SC-001, SC-002, SC-003); a sweep with no material; pausing on a message and resuming; each crash point including a half-done commit (SC-004); undo, undo after an edit, undo twice (SC-005); an outside edit during the sweep; the after-dream context with the dream entry; dream now with dreaming off; settings.
- **Contract**: the REM and deep request bodies (structured output, medium effort).
- **Webview**: Vitest for the journal, the status and portrait, settings; one end-to-end spec on the replay server.
- **Release checks** (owner, real provider): the two-week suite against the real model, sweep time (SC-009), a network capture with a dream (SC-010).

**Rationale**: The principles' dreaming gate (lossy rewrite rejected, undo exact) is covered at unit and integration level; the rest proves each requirement once at the cheapest boundary (`test-audit`).

## Data

What this slice adds to the workspace files, `steve.db`, the settings and state files, the conversation file, and the types that cross to the webview.

### Data 1. Workspace files

#### `MEMORY.md` entries written by a dream

The memory feature's entry format, with a `src:` naming daily note lines (architecture 4.1):

```markdown
## Running
- Training for the half marathon in April; the knee gets sore after 10 km. <!-- id:m_8f3a at:2026-11-15 from:companion src:memory/2026-11-10.md#L4,memory/2026-11-12.md#L7 -->
```

Rules: text at most 600 characters, one line; `id` unique in the file (`entry_id`); `at` is the night; `from:user` if any source line or replaced entry was `from:user`, else `companion`; `src:` one or more `memory/YYYY-MM-DD.md#Lnn` separated by commas. The fallback uses the topic `From my dreams`.

#### `USER.md` lines written by a dream

`- Key: value`, replacing the value of a matching key or appended, exactly as `profile_update` does. No comment (R7).

#### `DREAMS.md` (new)

```markdown
# Dream diary

## 2026-11-15
Running kept coming up this week, the plan for April, and that knee. I tucked it away properly. Mostly I dreamt of
rain on the window while you talked about the new job.

Kept:
- Training for the half marathon in April; the knee gets sore after 10 km.

## 2026-11-16 *(written without the model)*
I dreamt about 12 notes and kept 1 of them.

Kept:
- Starts the new job on 3 December.
```

Rules: created with the `# Dream diary` heading when missing; one `## <night>` section per sweep with material, appended; "Kept:" lists the entries the dream wrote, or is left out when it kept none. User edits are kept; later entries are appended to whatever is there. Never indexed, never gathered.

### Data 2. `state/steve.db`

Both tables are persistent: created with `CREATE TABLE IF NOT EXISTS` beside `preimages`, never dropped by an index rebuild.

#### `recall_events` (new)

| Column | Type | Rules |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | |
| `path` | TEXT | `memory/YYYY-MM-DD.md` only |
| `line` | INTEGER | The snippet's first line when recalled |
| `text_hash` | TEXT | `fingerprint` of the line without its comment, trimmed |
| `query_hash` | TEXT | `fingerprint` of the query's normalized terms, sorted and joined |
| `coverage` | REAL | Matched terms ÷ query terms, 0 to 1 |
| `day` | TEXT | The user's local date |
| `source` | TEXT | `auto` or `search` |
| `at` | INTEGER | Unix seconds |

Index on `(path, text_hash)`.

#### `dream_runs` (new)

| Column | Type | Rules |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | |
| `night` | TEXT | `YYYY-MM-DD` the sweep serves |
| `started_at`, `ended_at` | INTEGER | Unix seconds; `ended_at` null until `done` or `failed` |
| `since` | INTEGER | Unix seconds: the start point of what it covers |
| `phase` | TEXT | `light`, `rem`, `deep`, `committing`, `diary`, `roll`, `done`, `failed` |
| `checkpoint` | TEXT | JSON, section 5 |
| `added`, `merged`, `superseded` | INTEGER | Counts of applied decisions |
| `fell_back` | INTEGER | 0 or 1 |
| `usage` | TEXT | JSON: `{rem, deep, diary}` provider usage objects |
| `committed` | INTEGER | 0 or 1: the commit changed a file |
| `undone` | INTEGER | 0 or 1 |

#### `preimages` (existing)

Dream commits add one row per changed file with `message = 'dream:<run id>'`.

### Data 3. `settings.json`

```json
{"dreaming": {"enabled": true, "time": "03:00", "model": null}}
```

| Field | Rules |
|---|---|
| `dreaming.enabled` | Default `true`. |
| `dreaming.time` | `HH:MM`, 24-hour, default `03:00`. |
| `dreaming.model` | A model id or `null` (the chat model). Falls back to the chat model when not in the cached list. |

### Data 4. `state.json`

```json
{"dreaming": {"lastNight": "2026-11-15"}}
```

`dreaming.lastNight`: the night of the last completed sweep, or the latest night when dreaming was turned on (R3). Absent means none yet.

### Data 5. Run checkpoint (JSON in `dream_runs.checkpoint`)

```json
{
  "candidates": [
    {"id": 1, "text": "Training for the half marathon in April.", "provenance": "companion",
     "sources": [{"path": "memory/2026-11-10.md", "line": 4, "textHash": "…"}],
     "days": ["2026-11-10", "2026-11-12"], "reinforced": 2, "terms": 7}
  ],
  "messages": [{"at": "Friday 13 November 2026, 21:40", "text": "I ran 10 km today"}],
  "themes": [{"title": "Running", "summary": "…", "candidates": [1]}],
  "decisions": [{"op": "add", "topic": "Running", "text": "…", "key": "", "sources": [1], "replaces": []}],
  "gated": [1],
  "kept": ["…"],
  "diary": "…"
}
```

Each phase adds its part: light writes `candidates` and `messages`; REM writes `themes` and adds reinforcement; deep writes `decisions`, `gated` (the candidates that passed the gates) and `kept`; diary writes `diary`. The commit compares each file with the text the decisions were built on (R8).

### Data 6. Conversation file (`sessions/main.jsonl`)

`context` gains optional `dream`: the diary entry text the context was opened with after a dream (R10). Absent otherwise.

### Data 7. Types that cross to the webview (generated with `ts-rs`)

```rust
pub enum DreamState { Idle, Running, Paused }

pub struct DreamStatus {
    pub state: DreamState,
    pub last_run: Option<DreamRun>,
}

pub struct DreamRun {
    pub id: i64,
    pub night: String,
    pub started_at: u64,          // Unix seconds
    pub ended_at: Option<u64>,
    pub completed: bool,
    pub added: u32,
    pub merged: u32,
    pub superseded: u32,
    pub fell_back: bool,
    pub tokens: u64,              // input + output of the run's own requests
    pub undone: bool,
}

pub struct DreamJournal {
    pub diary: String,            // DREAMS.md as it is, or empty
    pub runs: Vec<DreamRun>,      // newest first, at most 30
    pub can_undo: bool,
}

#[serde(tag = "kind")]
pub enum UndoDreamOutcome { Undone, NothingToUndo, Changed { files: Vec<String> } }

pub struct DreamingSettings {
    pub enabled: bool,
    pub time: String,             // HH:MM
    pub model: Option<String>,
    pub model_missing: bool,
}

pub struct ChatState { /* … */ pub dreaming: bool }
pub enum MemoryPath { /* … */ Diary }    // DREAMS.md, in the memory browser (not exported; FileKind gains Diary)
```

### Data 8. Constants (core)

| Name | Value | Source |
|---|---|---|
| Catch-up idle delay | 5 minutes | architecture 15 |
| First sweep look-back | 7 days | R3 |
| Dedup overlap | 0.8 (Jaccard of terms) | R5 |
| Themes per sweep | at most 5 | R6 |
| Min score / recalls / distinct queries | 0.6 / 2 / 2 | R7 |
| Weights | 0.30, 0.24, 0.15, 0.15, 0.10, 0.06 | architecture 8.3 |
| Reinforcement boost | +0.05 each, at most +0.15 | R7 |
| Max prior entries removed | 25% | architecture 15 |
| Promoted entry length | 600 characters | R7 |
| Recall events per message / per search | 3 / 8 | R1 |

### Data 9. State transitions of a run

```text
(due tick) ─▶ light ─▶ rem ─▶ deep ─▶ committing ─▶ diary ─▶ roll ─▶ done
                 any phase ──message──▶ same phase (paused; resumes on the next due tick)
                 committing ──crash──▶ rollback ─▶ deep
                 deep (nothing gated) ─▶ diary (or done when there was no material)
                 provider rejects the key ─▶ failed (nothing changed; next night or catch-up retries)
```

## Interfaces

### Dreaming Commands Between the Webview and the Core

Same rules as the onboarding, chat, memory, world and compaction pages: thin handlers, validation in the core, expected failures as values, each command in `build.rs` and allowed in `src-tauri/capabilities/main.json`, and `src/api/commands.ts` as the only caller. The only network requests are the REM, deep and diary requests and the after-dream compaction, sent to the Claude API by the core (see Interfaces, the REM, deep and diary requests).

#### New commands

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `dream_tick` | `clock: LocalClock` | `DreamStatus` | Async. Called by the main view once at load and once a minute. If a night is due (R3) and nothing else runs, runs or resumes the sweep and returns when it is done or paused. Returns at once with `Running` while a sweep runs, and with `Idle` when nothing is due. Without a key, never due. Never fails because of the provider; a failed run is recorded and retried on a due tick at least an hour later. | FR-001 to FR-005, FR-023 |
| `dream_now` | `clock: LocalClock` | `DreamStatus` | Async. Runs a sweep at once over everything since the last completed one, whether dreaming is on or off. Refused as `InvalidInput("busy")` while a reply, a compaction or a sweep runs. | FR-027 |
| `undo_last_dream` | `force: boolean` | `UndoDreamOutcome` | Restores `MEMORY.md` and `USER.md` from the latest committed run not yet undone (R9). A file changed since gives `Changed` unless `force`. `NothingToUndo` when no run qualifies. Refused while a sweep runs. | FR-018 |
| `dream_journal` | — | `DreamJournal` | Reads `DREAMS.md` and the last 30 runs. No network request. | FR-021, FR-022 |
| `dreaming_settings` | — | `DreamingSettings` | Reads `settings.json` and the cached model list. | FR-026 |
| `set_dreaming_settings` | `enabled: boolean`, `time: string`, `model: string \| null`, `clock: LocalClock` | `DreamingSettings` | `time` must be `HH:MM` 00:00 to 23:59 (`InvalidInput("time")`); `model` `null` or in the cached list (`InvalidInput("model")`). Turning dreaming on sets `lastNight` to the latest night by `clock` (R3). | FR-026 |

#### Changes to earlier commands

| Command | Change |
|---|---|
| `chat_state` | `ChatState` gains `dreaming` (a sweep is running). |
| `send_message`, `retry_reply`, `regenerate_reply`, `start_first_hello` | Starting any of them also stops a running sweep at its next step (R4); they never wait for it. Auto-recall records recall evidence (R1). |
| `compact_conversation` | Refused as `InvalidInput("busy")` while a sweep runs; the sweep's own roll uses the core directly. |
| `memory_files`, `memory_read`, `memory_save` | `DREAMS.md` is listed with kind `Diary`, readable and savable like the other files; it has no undo and no forget. `memory_search` never returns it. |
| Companion tool `memory_search` | Records recall evidence for the daily-note hits it returns (R1). |
| Companion tool `memory_get` | Refuses `DREAMS.md`, as it refuses `SOUL.md`: the diary is never a memory source (FR-021). |

#### Webview duties

- The main view calls `dream_tick(localClock())` on load and every 60 seconds, never two at once, and shows `Running` as the dreaming status and the sleeping portrait.
- The dream journal shows the diary and the runs; "Dream now" calls `dream_now`; "Undo last dream" calls `undo_last_dream(false)` and on `Changed` asks before calling it with `true`.
- Settings show the dreaming section; changing it calls `set_dreaming_settings` with the clock.

#### Capability file additions

`allow-dream-tick`, `allow-dream-now`, `allow-undo-last-dream`, `allow-dream-journal`, `allow-dreaming-settings`, `allow-set-dreaming-settings`.

### The REM, Deep and Diary Requests to the Claude API

All three use the existing `POST /v1/messages` through the same client (chat Interfaces): streaming, headers, error classification, refusal handling, fallback list. Every wire detail stays in `crates/steve-core/src/llm/`. The model is the dreaming model (falling back to the chat model). No `tools` field on any of them.

#### Effort

`TurnRequest.effort` becomes a level. The body sends `output_config.effort` as `"low"` or `"medium"`, only for models that take effort:

| Request | Effort |
|---|---|
| Chat, flush, compaction summary | low (unchanged) |
| REM, diary | low |
| Deep decisions | medium |

#### REM: themes

`output_config.format` schema:

```json
{
  "type": "object",
  "properties": {
    "themes": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "title": { "type": "string" },
          "summary": { "type": "string" },
          "candidates": { "type": "array", "items": { "type": "integer" } }
        },
        "required": ["title", "summary", "candidates"],
        "additionalProperties": false
      }
    }
  },
  "required": ["themes"],
  "additionalProperties": false
}
```

System: `dream/prompts/rem.md` (find up to 5 themes across days; only candidates by id; no new facts). User message: the candidates as `[id] (day) text` lines, then the person's messages as `[time] text` lines. The core keeps at most 5 themes and drops ids that aren't candidates.

#### Deep: decisions

`output_config.format` schema:

```json
{
  "type": "object",
  "properties": {
    "operations": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "op": { "type": "string", "enum": ["add", "merge", "supersede", "profile"] },
          "topic": { "type": "string" },
          "text": { "type": "string" },
          "key": { "type": "string" },
          "sources": { "type": "array", "items": { "type": "integer" } },
          "replaces": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["op", "topic", "text", "key", "sources", "replaces"],
        "additionalProperties": false
      }
    }
  },
  "required": ["operations"],
  "additionalProperties": false
}
```

System: `dream/prompts/deep.md` (consolidate the candidates into long-term memory as discrete operations; short entries; keep identifiers exactly; `profile` only for facts about the person that fit a `Key: value`; `replaces` only ids from the current entries; leave out what isn't durable). User message: the candidates as `[id] text (sources)`, the current `MEMORY.md` entries as `[m_id] (topic) text`, the profile lines as `Key: value`.

What the core does with the reply: drops operations whose sources aren't gated candidates or whose `replaces` aren't current ids; ignores `key` except for `profile` and `topic` for `profile`; then applies and validates (R7, R8).

#### Diary

No `format`. System: the identity and soul as in the chat request's second system block, then `dream/prompts/diary.md` (write tonight's dream diary entry in your own voice, 3 to 6 sentences, for the person to read; you may be playful, but say what you kept truthfully). User message: the themes, how many notes were gathered, and the entries kept. The reply's text is the entry; an empty reply, a refusal or a failure gives the degraded local entry.

#### Contract tests

`tests/claude_contract.rs` gains: a request with `Medium` effort sends `output_config.effort: "medium"`; with `format` and `Medium`, both are sent. Existing cases keep passing with `Low`.

## Checks


### Prerequisites

- Onboarding, chat, memory, world and compaction set up, or a clean machine to run the wizard.
- Rust 1.98 or later, Node.js 22 or later, pnpm 10.
- A Claude API key for the manual scenarios and the release checks. The automated checks never use one.
- To see promotions quickly in manual runs: talk about one topic on several days (or set the system date forward between conversations), asking about it in different words so auto-recall brings the note back, then use "Dream now".

### Inspect the files

- `%APPDATA%\mySteve\workspace\MEMORY.md`: dream entries with `src:memory/…md#L…`.
- `%APPDATA%\mySteve\workspace\DREAMS.md`: one `## <night>` section per sweep with material.
- `%APPDATA%\mySteve\state\steve.db`: `recall_events`, `dream_runs` (any SQLite browser).
- `%APPDATA%\mySteve\state\state.json`: `dreaming.lastNight`.
- `%APPDATA%\mySteve\workspace\sessions\main.jsonl`: after a dream, a `context` entry with `dream`.

### Manual validation scenarios

| # | Story | Steps | Expected |
|---|---|---|---|
| 1 | US1 | Over three days, mention a fact once each day and ask about it in different words on two of them. Choose "Dream now". | The fact is in `MEMORY.md` with a `src:` to a note line; the journal shows 1 added. |
| 2 | US1 | Mention another fact once, never again. Dream now. | Not promoted. |
| 3 | US1 | In the memory browser, open the promoted entry's source. | The note and line it came from. |
| 4 | US1 | Change the fact (for example, a new race date) and repeat scenario 1. | The old entry is superseded, not duplicated. |
| 5 | US2 | Leave the app in the tray overnight with the computer awake. | A sweep ran near 03:00 (journal). |
| 6 | US2 | Close the lid at night; open it in the morning; use the app, then stop for 5 minutes. | The missed sweep runs once, after the idle delay. |
| 7 | US2 | Start "Dream now" and send a message right away. | The reply streams at once; the status shows paused or running; the sweep completes later. |
| 8 | US2 | Kill the app (Task Manager) during a sweep. Relaunch and wait. | `MEMORY.md` and `USER.md` are unchanged until the sweep completes; the night completes once. |
| 9 | US3 | Have the companion search memory for a page-like text through a tool result (or seed one), then dream. | Nothing from it is promoted. |
| 10 | US3 | Forget a memory, then dream. | It is not promoted again, and its source conversation is not used. |
| 11 | US3 | Edit `MEMORY.md` in an editor while a sweep runs. | Your edit is kept. |
| 12 | US4 | Open the dream journal after a sweep with material. | A short entry in the companion's voice, a "Kept" list, and the run's time and counts. |
| 13 | US4 | Choose "Undo last dream". | `MEMORY.md` and `USER.md` are exactly as before the dream. |
| 14 | US4 | Dream, edit `MEMORY.md`, then undo. | The app asks before discarding your edit. |
| 15 | US4 | Turn the network off, dream now. | A plain diary entry marked "written without the model"; facts that passed the gates are appended under "From my dreams". |
| 16 | US5 | After a dream that promoted a fact, say good morning and continue yesterday's topic. | The companion knows the promoted fact and the topic; it may mention its dream. |
| 17 | US6 | Set the dream time to an hour from now and leave the app idle. | A sweep runs at that time. |
| 18 | US6 | Turn dreaming off for two nights, then on. | No catch-up of those nights; the next dream time runs. |
| 19 | US6 | Pick another model for dreaming, dream now. | The run used it (provider logs or usage); chat keeps its model. |
| 20 | Edge | A new install: talk for an hour, dream now. | Nothing promoted (no recall history yet); the journal says the sweep ran. |

### Release checks (real provider, owner)

| Check | How | Pass |
|---|---|---|
| SC-001 | The two-week scripted suite against the real model. | At least 90% of recurring facts promoted; at most 5% of one-off facts. |
| SC-002, SC-003 | Same suite: provenance and sources of every promoted entry. | 0 from tool results, instructions, forgotten, recalled-only or the diary; 100% with a valid source. |
| SC-009 | Time each sweep in the suite. | 95% within 2 minutes. |
| SC-010 | Network capture of a session including a dream. | Only the provider. |
