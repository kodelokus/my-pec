# Extensions

Built.

Until now the companion could talk, remember and dream, but it couldn't do anything for the person outside the conversation. This feature adds extensions: small, bundled pieces of usefulness that the companion can use when asked and that the person can also use directly. The first two are sticky notes and alarms. "Put that on a sticky" leaves a note on the desktop. "Remind me at 7 to call Mum" makes Windows ring at seven, even with the chat window closed. Everything an extension keeps is a plain file in the companion's folder, like the rest of its mind.

## User stories

### US1. "Remind me at 7 to call Mum" (P1)

The person is chatting in the afternoon and says "remind me at 7 to call Mum". Pec answers that it will, and the reply shows that an alarm was set for 19:00 today. The person closes the window; My Pec keeps running in the tray. At seven, a Windows notification appears: "Call Mum", from My Pec, with Pec's name. The person clicks Snooze, and it rings again ten minutes later; this time they click Done. The next day they ask "what reminders do I have?", and Pec lists what is left.

**Why this priority**: A reminder that rings at the right moment is the clearest proof that the companion is useful beyond conversation, and it's the case the person can't get from a chat window alone (D18).

**How to check**: With recorded replies that call the alarm tools, ask for a reminder a few minutes ahead, close the window to the tray, and confirm the notification appears on time with the right text, that Snooze rings again after the snooze time and Done clears it, and that the alarm list reflects each step.

**Acceptance Scenarios**:

1. **Given** the alarms extension is on, **When** the person asks the companion for a reminder at a time, **Then** an alarm is saved for that local time with the person's words as its label, and the reply shows which alarm was set (date, time, label).
2. **Given** an ambiguous time such as "at 7" in the afternoon, **When** the alarm is set, **Then** it is the next 7 o'clock that has not yet passed (19:00 today), and the companion says the exact date and time it chose.
3. **Given** a saved alarm and the app running, with the window open, hidden in the tray or minimized, **When** its time arrives, **Then** a Windows notification shows the label and the companion's name, within one minute of the time.
4. **Given** a notification is showing, **When** the person chooses Snooze, **Then** it rings again after the snooze time; **When** they choose Done or dismiss it, **Then** a one-time alarm is finished and a repeating one moves to its next time.
5. **Given** the person clicks the notification itself, **When** it opens, **Then** the main window comes to the front with the alarm list.
6. **Given** the person asks the companion what is set, to move an alarm or to cancel one, **When** it answers, **Then** it uses the saved alarms, and every change it makes is shown on its reply the way memory writes are.

### US2. Sticky notes on the desktop (P1)

The person and Pec plan a weekend trip. "Put the packing list on a sticky," they say. A small note appears on the desktop with the list, and the reply shows that a note was made. Later the person ticks off items by editing the note directly, adds "charger" at the bottom, and drags the note to the corner of the screen. Next morning, after a restart, the note is where they left it, with their edits. When they ask Pec "what's still on my packing list?", Pec reads the note as it is now.

**Why this priority**: Sticky notes are the simplest visible artifact the companion can leave in the person's world, and they prove the boundary works both ways: the companion writes them, the person edits them, and both see the same file (Principle I).

**How to check**: With recorded replies that call the note tools, create a note through the companion and one by hand, edit and move them, restart, and confirm the text, place and colour are kept; then edit the note's file in a text editor and confirm both the desktop note and the companion see the change.

**Acceptance Scenarios**:

1. **Given** the notes extension is on, **When** the person asks the companion to note something down, **Then** a note is saved with that text, appears on the desktop, and the reply shows that a note was made.
2. **Given** the main window, **When** the person creates a note themselves, **Then** an empty note appears on the desktop ready to type in.
3. **Given** a note on the desktop, **When** the person types in it, **Then** the text is saved without any save button, and is never lost when the app or the computer stops.
4. **Given** a note, **When** the person moves it, changes its colour or turns "keep on top" on or off, **Then** that is kept across restarts; a note whose place is now off every screen comes back onto the main screen.
5. **Given** notes exist, **When** the person asks the companion about them, **Then** it reads the notes as they are now, including the person's own edits.
6. **Given** a note, **When** the person hides it, **Then** it leaves the desktop but stays in the notes list in the main window, where it can be shown again; **When** they delete it and confirm, **Then** its file is moved into the notes folder's `deleted` folder, not erased.

### US3. Choose which extensions are on (P2)

The person doesn't want notes on their desktop. In settings, under Extensions, they see Sticky notes and Alarms, each with a sentence on what it does, what the companion can do with it and where its files live. They turn Sticky notes off: the desktop notes disappear, and when they ask Pec to note something, Pec says notes are turned off and where to turn them on. Their note files stay in the folder untouched. Turning it on again brings the notes back where they were.

**Why this priority**: Usefulness must never be imposed. Each extension adds windows, notifications and companion abilities, and the person decides which they want (Principle VII, PER-8).

**How to check**: Turn each extension off and on, and confirm its windows, notifications and companion tools follow, its files are untouched, and the other extension is unaffected.

**Acceptance Scenarios**:

1. **Given** a fresh install, **When** setup ends, **Then** both extensions are on, with no extra setup step.
2. **Given** an extension is turned off, **When** the person looks around the app, **Then** its windows, panels and notifications are gone, and its files remain exactly as they were.
3. **Given** an extension is turned off, **When** the companion is asked to use it, **Then** it says the extension is off and where to turn it on, and changes nothing.
4. **Given** alarms are off, **When** an alarm's time passes, **Then** nothing rings; when alarms are turned on again, alarms whose time passed meanwhile are shown as missed, not rung.

### US4. Alarms ring even after a restart (P2)

The person sets a reminder for tomorrow morning and shuts the computer down. In the morning they sign in and My Pec starts by itself, because when they set their first alarm it asked "Alarms only ring while My Pec is running. Start My Pec when you sign in?" and they said yes. The alarm rings on time. Another day the computer was asleep at the alarm's time; when it wakes, My Pec says the reminder was missed, and when it was due, instead of pretending it's on time.

**Why this priority**: An alarm that silently never rings is worse than none. The app must be running to ring, and it must be honest about the times it couldn't be (SYS-5, NFR-SEC-6).

**How to check**: Turn on launch at login, restart the computer and confirm My Pec starts in the tray without a window; put the computer to sleep across an alarm's time and confirm it is reported as missed with its time when the computer wakes.

**Acceptance Scenarios**:

1. **Given** launch at login is off, **When** the person saves their first alarm, **Then** they are asked once whether My Pec should start when they sign in, with the reason; their answer is kept and can be changed in settings.
2. **Given** launch at login is on, **When** the person signs in to Windows, **Then** My Pec starts in the tray without opening its window, and alarms ring from then on.
3. **Given** an alarm's time passed while the app was not running or the computer was asleep, **When** the app runs again, **Then** one notification lists the missed alarms with their times, they are marked missed in the alarm list, and a repeating alarm moves on to its next time.
4. **Given** an alarm missed by more than a day, **When** it is reported, **Then** it is reported once and never rung late.

### US5. Notes and alarms are the person's files (P3)

The person opens the companion's folder and finds a `notes` folder with one Markdown file per note and an `alarms.md` file listing alarms one per line in plain words. They fix a typo in an alarm's label and change its time in a text editor. The alarm list and the companion both show the change, and it rings at the new time. They type a line the app can't read; the app leaves that line alone and says in the alarm list that one line couldn't be read.

**Why this priority**: Files are the mind (Principle I). Notes and alarms are part of the companion's world, so they must be readable, editable and safe to back up like the rest.

**How to check**: Edit, add and break note and alarm files by hand while the app runs, and confirm every valid change is picked up, nothing written by hand is overwritten, and invalid lines are reported and left alone.

**Acceptance Scenarios**:

1. **Given** the app is running, **When** a note or alarm file is changed outside the app, **Then** the change is shown within 15 seconds and used by the companion from then on.
2. **Given** an alarm line the app can't read, **When** the alarms are loaded, **Then** that line is left in the file untouched, is not rung, and the alarm list says how many lines couldn't be read.
3. **Given** the app and the person change the same file at nearly the same time, **When** the app writes, **Then** it never overwrites a change it hasn't read; it keeps the person's version and applies its own change on top or reports a conflict.

## Edge cases

- The person asks for a reminder with no time ("remind me to buy milk"): the companion asks when, or offers a note instead; it never invents a time.
- A time that falls in a clock change, or a time zone change while travelling: alarms are kept in local wall-clock time and ring at that local time in the zone the computer is set to when it comes due.
- Many alarms due at the same minute ring as one notification listing them.
- The companion is asked to set many alarms or notes at once: it can, one tool call each, within the usual limit on tool calls per reply.
- Windows notifications are turned off for My Pec: the alarm is still marked as rung, and the alarm list says once that Windows is holding back its notifications. With Focus on, Windows keeps the notification for later, and the alarm shows as rung in the list.
- A note's text is very long: notes are capped at a size that fits a desktop note (a few thousand characters); the companion is told when a note is full.
- The notes or alarms file is deleted while the app runs: the extension shows an empty list and recreates nothing until something new is saved.
- A conversation context started before extensions existed, or before an extension was turned on, keeps the tools it started with; the companion gets the new tools from the next fresh context or compaction (Principle III), and turning an extension off takes effect at once by refusing its tools.
- The companion must not set alarms or write notes the person didn't ask for; it may suggest one and wait for a yes (NFR-SEC-6, no proactive behaviour).
- Two My Pec windows can't both open the same note: a note is shown in at most one window.

## Requirements

**Extensions in general**

- **FR-001**: The app MUST ship two bundled extensions, Sticky notes and Alarms, both on by default and needing no setup (Principle VII).
- **FR-002**: Each extension MUST be able to give the companion tools for the conversation, keep its data only as plain files in its own place in the companion's workspace, and add its own windows, panels or notifications.
- **FR-003**: An extension MUST NOT read or write anything outside its own place in the workspace and its own bookkeeping file, MUST NOT reach the network, and MUST NOT see the API key (Principle II).
- **FR-004**: Settings MUST list the extensions with what each does, what the companion can do with it and where its files are, and let the person turn each on or off. Turning one off MUST hide its windows and stop its notifications and tools at once, and MUST leave its files untouched.
- **FR-005**: Every change the companion makes through an extension's tools MUST be shown on its reply, like memory writes, with a way to open the note or the alarm list.
- **FR-006**: The companion MUST use extension tools only when the person asks or agrees, never on its own initiative (NFR-SEC-6).
- **FR-007**: New conversation contexts MUST carry the tools of the extensions that are on; a running context keeps the tools it started with, and a call to the tool of an extension that is now off MUST be refused with a message the companion can pass on (Principle III).
- **FR-008**: Extension files MUST be written atomically, MUST honour edits made outside the app, and MUST never overwrite a change the app hasn't read (Principle I).
- **FR-009**: Extension data MUST NOT be used as memory: it is not indexed by memory search, not given to dreaming and not promoted to long-term memory. The companion reads it only through the extension's own tools.
- **FR-010**: A failing extension MUST NOT block or break chat (Principle V): its tools return an error the companion can explain, and its windows close without affecting the main window.

**Sticky notes**

- **FR-011**: The person MUST be able to create, edit, colour, move, hide, show and delete notes; the companion MUST be able to create a note, list notes, read a note and add to or replace a note's text.
- **FR-012**: Each note MUST be one Markdown file in the workspace's notes folder, readable and editable in any text editor; its place on screen, colour, "keep on top" and whether it's shown are kept alongside it.
- **FR-013**: Shown notes MUST appear as small frameless windows on the desktop, outside the taskbar, that stay while the main window is closed to the tray and come back where they were after a restart. A note off every screen MUST be moved onto the main screen.
- **FR-014**: Typing in a note MUST save it within a second of the last keystroke, with no save action.
- **FR-015**: The main window MUST have a notes list showing every note, shown or hidden, from which a note can be opened, shown, hidden or deleted. Deleting MUST ask for confirmation and move the file into `notes/deleted/`, from where the person can move it back.
- **FR-016**: A note MUST hold at most 4,000 characters; the companion MUST be told when a note is full.

**Alarms**

- **FR-017**: The person MUST be able to create, edit, snooze, finish and delete alarms from the alarm list; the companion MUST be able to set, list, move and cancel alarms.
- **FR-018**: An alarm MUST have a label, a local date and time, and a repeat: once, every day, on chosen weekdays, or every week.
- **FR-019**: Alarms MUST be kept in one plain file in the workspace, one alarm per line, in a form a person can read and edit.
- **FR-020**: While the app runs, in any window state, an alarm MUST ring within one minute of its time with a Windows notification showing its label and the companion's name, and offering Snooze and Done.
- **FR-021**: Snooze MUST ring again after 10 minutes. Done or dismissing MUST finish a one-time alarm and move a repeating one to its next time.
- **FR-022**: Clicking a notification MUST bring the main window to the front on the alarm list.
- **FR-023**: Alarms whose time passed while the app was not running, the computer was asleep or alarms were off MUST be reported once as missed, with their times, and MUST NOT be rung late.
- **FR-024**: When the person asks for a time without a date, the companion MUST choose the next such time that hasn't passed and say the exact date and time; when there is no time at all, it MUST ask rather than guess.
- **FR-025**: Lines in the alarms file that can't be read MUST be left untouched and not rung, and the alarm list MUST say how many there are.

**Launch at login**

- **FR-026**: Settings MUST let the person start My Pec when they sign in to Windows; it MUST be off by default (SYS-5).
- **FR-027**: When the person saves their first alarm while launch at login is off, the app MUST ask once whether to turn it on, explaining that alarms ring only while My Pec runs and that it also lets the companion dream overnight.
- **FR-028**: Started at sign-in, My Pec MUST open in the tray without showing its window, and show the notes that were shown.

## Key entities

- **Extension**: A bundled piece of usefulness with a name, a description, on/off state, the tools it gives the companion, and its own place in the workspace.
- **Note**: A short Markdown text with a colour, a place on screen, whether it is kept on top and whether it is shown; one file per note.
- **Alarm**: A label, a local date and time, a repeat rule, and its state (waiting, snoozed until, done or missed, and ringing only while the app runs); one line per alarm in the alarms file.
- **Extension change mark**: What an extension tool did during a reply (note made, note changed, alarm set, moved or cancelled), shown on the reply.

## Success criteria

- **SC-001**: In 20 scripted requests like "remind me at 7 to call Mum" with recorded replies, every alarm is saved at the intended local time, and the notification appears within one minute of it in 100% of runs with the window open, hidden in the tray or minimized.
- **SC-002**: A person can set a reminder by asking the companion in one message and see it confirmed on the reply, without opening any other screen.
- **SC-003**: Notes survive 10 restarts in a row with their text, place, colour and visibility unchanged; a note edited in a text editor shows the edit within 3 seconds in a focused note window and within 15 seconds elsewhere.
- **SC-004**: With power cut while typing in a note, no more than the last second of typing is lost, and the note file is never left half-written.
- **SC-005**: An alarm missed while the computer slept is reported within 1 minute of waking, once, and is never rung as if on time.
- **SC-006**: Turning an extension off removes its windows and notifications within 1 second and leaves its files byte for byte unchanged.
- **SC-007**: With 50 notes and 100 alarms, opening the main window, the notes list and the alarm list each feels instant (under 300 ms), and the app in the tray stays within NFR-PERF-4.
- **SC-008**: A network capture of a day with notes, alarms and launch at login shows no destination but the provider.
- **SC-009**: A new user still completes setup with the key as the only input, and the companion can set an alarm and make a note in the first conversation.

## Assumptions

- **Bundled only.** Extensions are part of the app and of this repository, behind a boundary that would let them move to their own repository later (D18). Third-party extensions, runtime plugins, an extension store and extensions that reach the network are out of scope.
- **Where the files live.** Notes live in `notes/` and alarms in `alarms.md` inside the companion's workspace, so they are backed up, moved and exported with it. A deleted note's file is moved into `notes/deleted/` rather than erased; the app never empties that folder.
- **Time.** Alarms are wall-clock times in the computer's current time zone, as the rest of the app reads the person's clock. Snooze is 10 minutes, not configurable in this release.
- **Notifications.** Alarms use Windows' own notifications, which follow the person's Focus and notification settings; a sound is whatever Windows plays. The app does not play its own sound.
- **Repeat rules.** Once, every day, chosen weekdays and every week cover the common cases; monthly and yearly alarms, and calendars, are out of scope.
- **No proactive behaviour.** The companion uses the tools only when asked or after the person agrees to a suggestion; proactive check-ins (CHAT-8) remain out of scope.
- **Shared window work.** Desktop notes reuse the frameless, outside-the-taskbar window approach and the placement rule of the avatar's floating mode (the avatar page), each note in a window that can do only what a note needs.
- **Launch at login.** This feature brings SYS-5 into scope as a setting and the offer on the first alarm; the onboarding wizard itself is unchanged, so setup still needs only the key.
- **Not memory.** Notes and alarms are not memory: the companion reaches them only through their tools, and they are left out of memory search, dreaming and compaction summaries. A note's content enters the conversation only when the companion reads it with a tool.
- **Dependencies.** Builds on chat and tools (the chat and memory pages), the marks shown on replies (the memory page), harness versions per context (the world, journey and avatar pages) and the floating window (the avatar page).

## Decisions

Each section records a decision, why it was made, and what was rejected. The design builds on memory (memory tools, marks on replies, tool results as `untrusted` messages), world, journey and avatar (harness versions recorded per context), dreaming (the webview tick and the core's scheduling rules) and avatar (frameless windows outside the taskbar, the placement rule, per-window capabilities).

### R1. The extension boundary

**Decision**: The core defines one trait, `Extension`, in a new module `pec-core/src/ext/`. An extension has an id, a short description for settings, the tool specs it gives the companion, a function that runs one of its tool calls against its own folder, and a function that answers the shell's tick. The two extensions live in their own crate, `crates/pec-ext` (`notes` and `alarms` modules), which depends on `pec-core` for the trait, the atomic write and the shared types. The shell builds the core with `Core::with_extensions(pec_ext::all())`. The core never names a concrete extension.

The core owns everything shared: whether an extension is on (`settings.json`), the tool list a new context records, routing a tool call to its extension, the marks on the reply, and confining each extension to its own place through an `ExtDir` handle that opens only the extension's workspace place (a folder such as `notes/`, or one file such as `alarms.md`) and its own bookkeeping file, `state/<id>.json`.

**Rationale**: D18 asks for a boundary strict enough that extensions could move to their own repository later. A crate edge is a boundary the compiler enforces: `pec-ext` can't reach the core's private modules, the transcript or the key. `lean-code` allows a trait with two implementations, and there are two.

**Alternatives considered**: Extension code inside `pec-core` behind a module (nothing stops it reaching into memory or chat internals). A separate repository with a git submodule (rejected in D18 while the API is still moving). Runtime plugins such as WASM (out of scope; needed only for third-party extensions).

### R2. Tools and the append-only rule

**Decision**: A new harness version, `ext-1`, adds a short section "Your extensions" to the rules: the companion may use extension tools only when the person asks or agrees, says what it did, and never invents a time. A new context records, besides its harness version, the ids of the extensions that were on (`Entry::Context.extensions`). The request rebuilds the tool list from that record, in a fixed order: memory tools, then each recorded extension's tools. Turning an extension on mid-context changes nothing until the next fresh context or compaction. Turning one off takes effect at once: its tool calls are answered with an error result saying the extension is off and where to turn it on, and the tool list stays as it was (FR-007).

**Rationale**: Principle III forbids changing the tool list within an epoch. Recording the ids keeps old contexts exactly reproducible, as harness versions already do.

**Alternatives considered**: Always sending every extension's tools and refusing the off ones (a person who never wanted alarms would still pay for their tool definitions in every request). Opening a fresh context when an extension is toggled (loses the conversation's working context for a setting change).

### R3. Marks on the reply

**Decision**: `MarkKind` gains five kinds: `NoteMade`, `NoteChanged`, `AlarmSet`, `AlarmChanged` and `AlarmCancelled` (the existing `Note` means a daily memory note and keeps that meaning). A mark's `path` names the note file or `alarms.md`, and its excerpt the note's first line or the alarm's time and label. The webview's existing mark row opens the note window or the alarm list.

**Rationale**: FR-005 asks for changes to show "like memory writes"; the mark path already exists end to end (memory FR-008).

**Alternatives considered**: A separate event stream for extension changes (a second mechanism for the same job).

### R4. When alarms ring: a timer in the shell, the rules in the core

**Decision**: The shell runs one background thread that wakes every 15 seconds and calls `core.ext_tick(now)`, where `now` is the local time with its UTC offset, read with `time::OffsetDateTime::now_local` (`time` is already in the dependency tree through Tauri; `chrono`, assumed here at first, is not). The offset lets the rule tell real elapsed time from a clock change. The core asks each extension that is on what is due. The alarms extension compares each alarm's next time with `now` and with the previous tick's time: due in this interval → ring; due before the previous tick by more than 2 minutes (the computer slept or the app was closed) → missed. The tick returns what to show: alarms to ring, missed alarms to report. The thread does not depend on any window.

**Rationale**: Chromium throttles timers in hidden pages to once a minute after 5 minutes, and nothing documents what WebView2 does under `window.hide()`; the dream tick tolerates that, alarms can't (SC-001: within one minute). The rule (what is due, what is missed, the next repeat) is pure and testable headless (Principle VI); only the clock and the timer are in the shell.

**Alternatives considered**: The webview tick that dreaming uses (throttling, and it stops if the page reloads). OS-scheduled tasks or wake timers (heavier, need elevated setup, and would ring with the app closed, which the requirements don't promise). Absolute UTC instants computed when an alarm is saved (wrong after a clock change or travel; the spec asks for wall-clock time).

### R5. Notifications with Snooze and Done

**Decision**: The shell shows alarms with `tauri-winrt-notification` (new dependency, Windows only): `Toast::new(app_id)`, the label as the title, the companion's name as the text, `Scenario::Alarm` so it stays until dismissed, and two buttons, `snooze` and `done`. `on_activated` reports `Some("snooze")`, `Some("done")` or `None` for a click on the toast itself (open the alarm list); `on_dismissed` counts as Done (FR-021). The app id is the bundle identifier `com.kodelokus.mypec` in installed builds, and `Toast::POWERSHELL_APP_ID` when running from `target/` (development and the end-to-end build), as Tauri's own notification plugin does. Missed alarms use one ordinary toast listing them. Activation is handled in-process while My Pec runs; no COM activator is registered, because the app must be running for alarms to ring at all.

**Rationale**: The Tauri notification plugin's desktop side uses `notify-rust`; its docs call actions mobile-only, and its source says buttons appear only while a listener exists, with no alarm scenario. `tauri-winrt-notification` is maintained by the Tauri organization, exposes buttons, activation arguments, dismissal reasons and the alarm scenario directly, and is confined to the shell (Principle VI; platform code isolated).

**Alternatives considered**: `tauri-plugin-notification` (uncertain button support on Windows, no alarm scenario). An in-app alarm window (not visible when the person is in another app with My Pec in the tray, and adds a window per alarm).

**To check on the owner's machine** (Checks): that the NSIS installer's Start menu shortcut carries the AppUserModelID so toasts show My Pec's name and icon, and that button activation reaches the running app.

### R6. Launch at login

**Decision**: `tauri-plugin-autostart` (new dependency) with the argument `--autostart`. The core keeps the person's answer to the first-alarm offer (`state.json` `autostartOffered`); whether launch at login is on is read from the plugin, not stored twice. Started with `--autostart`, the shell does not show the main window; the floating character and shown notes appear as usual (FR-028). Settings gains a switch in the Extensions section (it serves alarms and dreaming alike).

**Rationale**: SYS-5 is a "should" whose implementation is the plugin's whole job. Reading the state from the system avoids a setting that can disagree with Windows when the person changes it in Task Manager.

**Alternatives considered**: Writing the `Run` registry key ourselves (the plugin does exactly this, maintained). An onboarding step (the requirements keep setup to the key only).

### R7. Notes on the desktop

**Decision**: Each shown note is a window labelled `note-<id>`, built like the floating character (avatar R8): frameless, outside the taskbar, not always on top unless the note says so, 260 × 260 logical pixels by default and resizable, loading the app's page with an initialization script that names the note. Its capability, `capabilities/note.json`, allows only reading and saving its own note, setting its colour, place, size and "keep on top", hiding it, and dragging the window. Placement reuses the core's `place` rule. Typing saves 700 ms after the last keystroke (FR-014 allows one second). Windows are built only from async commands or the setup hook (the avatar page's finding).

**Rationale**: Sharing the floating window's approach was the point of D18's note. A window per note is what desktop sticky notes are.

**Cost**: each WebView2 window adds a renderer process, roughly 20 to 40 MB. NFR-PERF-4 (150 MB in the tray) cannot hold with many notes shown at once. This is accepted and recorded here: notes the person hides cost nothing, and Checks measure memory with 1, 5 and 10 notes shown. A later change could draw several notes in one transparent window if this matters in practice.

**Alternatives considered**: One large transparent window holding every note (click-through and focus are hard to get right on Windows). Notes only inside the main window (not what "sticky notes on the desktop" means).

### R8. The files

**Decision**:

- **Notes**: `workspace/notes/<slug>-<id>.md`, the note's text and nothing else, so it reads cleanly in any editor. The id is 6 characters from the same alphabet as the world id; the slug comes from the first line when the note is created and never changes. Colour, place, size, "keep on top" and shown are bookkeeping in `state/notes.json`, keyed by file name: device-specific and safe to lose (a lost entry shows the note hidden, in the default colour). Deleted notes move to `workspace/notes/deleted/`.
- **Alarms**: `workspace/alarms.md`, one alarm per line, written so a person can read and edit it:

  ```markdown
  # Alarms

  - [ ] 2026-11-15 19:00 Call Mum
  - [ ] every day 07:30 Take the vitamins
  - [ ] Mon Wed Fri 06:45 Run
  - [ ] every Sat 10:00 Laundry
  - [x] 2026-11-14 08:00 Dentist
  - [!] 2026-11-13 21:00 Water the plants
  ```

  `[ ]` waiting, `[x]` done, `[!]` missed. A repeating alarm stays `[ ]`; when it last rang, its snooze and the last tick's time are bookkeeping in `state/alarms.json`, keyed by the line's text, so an edited line simply starts fresh. Lines that don't match are left untouched and counted (FR-025). The app rewrites the file only to add, change or remove its own lines, under the writer's lock, after re-reading it (Principle I; FR-008).

**Rationale**: Principle I: the files carry only what a person would write; window positions and ring history are bookkeeping, like `state.json` today. A Markdown checklist is the most familiar plain-text form for a list of reminders.

**Alternatives considered**: Front matter in each note for colour and place (clutters the note in an editor and changes the file every time it is dragged). JSON for alarms (not something a person edits comfortably). One file per alarm (dozens of tiny files for a list).

### R9. Outside edits

**Decision**: Extensions read their files on every use, as the memory browser does. The core's existing file watch is not extended; instead the shell's 15-second tick also asks each extension whether its files changed (modification time and size), and the webview is told with an `ext_changed` event so open lists and note windows reload (SC-003: within 3 seconds while a note window is focused is met by the note window re-reading on focus; within 15 seconds otherwise). A note window with unsaved typing never reloads over it; if the file changed underneath, it saves its text as a new note beside the original and says so.

**Rationale**: Principle I forbids silently overwriting an outside edit; reading on use and a cheap change check cover it without a new watcher dependency.

**Alternatives considered**: A file system watcher crate (a new dependency and threads for two small files). Polling every second (needless work in the tray).

### R10. Not memory

**Decision**: `notes/` and `alarms.md` are outside the memory folders, so the index, dreaming and the memory browser already ignore them; this plan adds them to the list of workspace files the memory module skips explicitly, with a test. Tool results from extension tools are `untrusted` messages like memory tool results, so nothing in them can be promoted (Principle IV). The compaction summary already reads only user messages, replies and marks.

**Rationale**: FR-009.

### R11. Principles and product documents

- **Principle I** lists the workspace files that are the companion's mind. Notes and alarms are things the companion and the person made together, so `notes/` and `alarms.md` belong in that list. Amended in 1.2.1 (PATCH), approved by the owner on 2026-11-14.
- **Principle VI** limits the shell to "windows, the tray, command handlers, capability configuration, secret storage and updates". Notifications, the alarm timer's clock and launch at login are shell concerns of the same kind. Amended in 1.2.1 (PATCH), approved by the owner on 2026-11-14: the list gains notifications, launch at login and the clock for the extension tick, and the extensions' rules may live in a crate that depends on `pec-core` alone.
- **Principle II**: no network destination is added.
- `docs/requirements.md`: SYS-5 moves from "should" to in scope with this slice; a new section 6.13 "Extensions" holds EXT-1 to EXT-n from this page. `docs/architecture.md`: a new section on extensions (the trait, the crate, the tick, the files), section 4.1 (workspace files), section 14 (repository layout: `crates/pec-ext`), section 15 (snooze, tick interval, note size).

## Data

What this slice adds to the workspace, the state folder, the settings file, the transcript and the types that cross to the webview. No database table changes.

### Data 1. Workspace

```text
workspace/
  notes/
    packing-list-k7q2mx.md     one note: its text only
    deleted/                   notes the person deleted, never emptied by the app
  alarms.md                    every alarm, one per line
```

#### A note file

The note's Markdown text, at most 4,000 characters (FR-016). The file name is `<slug>-<id>.md`: the slug is the first line at creation, lowercased, letters and digits joined by `-`, at most 32 characters (`note` when empty); the id is 6 characters from the world id's alphabet. The name never changes after creation, so bookkeeping keyed by it survives edits.

#### `alarms.md`

```markdown
# Alarms

- [ ] 2026-11-15 19:00 Call Mum
- [ ] every day 07:30 Take the vitamins
- [ ] Mon Wed Fri 06:45 Run
- [ ] every Sat 10:00 Laundry
- [x] 2026-11-14 08:00 Dentist
- [!] 2026-11-13 21:00 Water the plants
```

| Part | Rules |
|---|---|
| Box | `[ ]` waiting, `[x]` done, `[!]` missed. Repeating alarms are always `[ ]`. |
| When | `YYYY-MM-DD` (once), `every day`, one or more of `Mon Tue Wed Thu Fri Sat Sun` separated by spaces (chosen weekdays), or `every <weekday>` (weekly). |
| Time | `HH:MM`, 24-hour, local wall-clock time. |
| Label | The rest of the line, 1 to 200 characters, one line. |

Lines that start with `#` or are empty are kept as they are. Any other line that doesn't match is left untouched, never rung, and counted (FR-025). The app writes the file only to add, change or remove a line it parsed, after reading the file again under the writer's lock (FR-008). Done and missed one-time alarms older than 30 days are removed by the app the next time it writes the file.

### Data 2. State folder (bookkeeping, safe to lose)

| File | Holds | If lost |
|---|---|---|
| `state/notes.json` | Per note file name: `shown`, `onTop`, `colour` (`yellow`, `green`, `blue`, `pink`, `grey`), `x`, `y`, `width`, `height` (physical pixels, `null` until placed) | Notes come back hidden, yellow, placed by the default rule |
| `state/alarms.json` | Per alarm line text: `rang` (the local date and time it last rang), `snoozedUntil`; and `lastTick` (the previous tick's local date and time, and its Unix time to tell sleep from a clock change) | Repeating alarms count from now; nothing is reported as missed for the time before |
| `state.json` | `autostartOffered: bool` (the first-alarm offer was made) | The offer is made again once |

### Data 3. `settings.json`

```json
{ "extensions": { "notes": { "on": true }, "alarms": { "on": true } } }
```

Both default to `true` (FR-001). Unknown extension ids are kept and ignored.

### Data 4. Transcript

`Entry::Context` gains `extensions: Vec<String>`, the ids of the extensions that were on when the context started, in order. Missing means none (every context written before this slice). Harness version `ext-1` is recorded by contexts started after this slice.

### Data 5. Alarm state machine

```text
waiting ──time comes, app running, alarms on──▶ ringing ──Snooze──▶ snoozed ──10 min──▶ ringing
   │                                              │
   │                                              └─Done / dismissed─▶ done (once) | waiting, next time (repeating)
   └──time passed > 2 min before this tick (asleep, closed, alarms off)──▶ missed (once) | waiting, next time (repeating)
```

The next time of a repeating alarm is the first time after `max(rang, now)` that matches its rule. Ringing and snoozed live only in the running app and `state/alarms.json`; a restart while ringing reports the alarm as missed if its time is more than 2 minutes past.

### Data 6. Constants

| Name | Value | Source |
|---|---|---|
| Tick | every 15 seconds | R4 |
| Missed after | 2 minutes late | R4 |
| Snooze | 10 minutes | FR-021 |
| Note size | 4,000 characters | FR-016 |
| Note save delay | 700 ms after the last keystroke | FR-014 |
| Note window | 260 × 260 logical pixels by default, resizable from 160 × 120 | R7 |
| Old alarms kept | 30 days | section 1 |

### Data 7. Types that cross to the webview (generated with `ts-rs`)

```rust
pub struct ExtensionInfo { pub id: String, pub name: String, pub about: String, pub folder: String, pub on: bool }

pub struct NoteSummary { pub file: String, pub first_line: String, pub shown: bool, pub colour: NoteColour }
pub struct NoteDoc { pub file: String, pub text: String, pub hash: String, pub colour: NoteColour, pub on_top: bool }
pub enum NoteColour { Yellow, Green, Blue, Pink, Grey }

pub enum Repeat { Once { date: String }, Daily, Weekdays { days: Vec<Weekday> }, Weekly { day: Weekday } }
pub enum AlarmState { Waiting, Snoozed, Done, Missed }
pub struct Alarm { pub key: String, pub label: String, pub time: String, pub repeat: Repeat, pub state: AlarmState, pub next: Option<String> }
pub struct AlarmList { pub alarms: Vec<Alarm>, pub unreadable: u32, pub notifications_held: bool }

pub struct ExtTick { pub ring: Vec<Alarm>, pub missed: Vec<Alarm>, pub changed: Vec<String> }
```

An alarm's `key` is the first 8 hex characters of the SHA-256 of its line in `alarms.md` after the box, so Done doesn't change it. A hand edit to the line gives the alarm a new key.

`MarkKind` gains `NoteMade`, `NoteChanged`, `AlarmSet`, `AlarmChanged`, `AlarmCancelled`. `ExtTick` does not cross to the webview; the shell turns it into notifications and an `ext_changed` event.

## Interfaces

### Commands, windows and events

Same rules as the earlier features (onboarding to avatar): thin handlers, validation in the core, expected failures as values, each command in `build.rs` and allowed in the capability file of each window that may call it, and `src/api/commands.ts` as the only caller. Commands that build windows are `async` (the avatar page: a window built in a synchronous command never loads). No network destination is added.

#### New commands

| Command | Windows | Input | Output | Behavior | Requirements |
|---|---|---|---|---|---|
| `extensions` | main | — | `ExtensionInfo[]` | The bundled extensions and whether each is on. | FR-004 |
| `set_extension` | main | `id`, `on: boolean` | `ExtensionInfo[]` | Saves the setting. Turning notes off closes every note window; turning it on shows the notes that were shown. | FR-004 |
| `autostart` | main | — | `{ on: boolean, offered: boolean }` | Whether My Pec starts at sign-in, read from the system. | FR-026 |
| `set_autostart` | main | `on: boolean` | `{ on, offered }` | Turns launch at login on or off and records that the offer was answered. | FR-026, FR-027 |
| `notes` | main | — | `NoteSummary[]` | Every note, shown or hidden, newest first. | FR-015 |
| `new_note` | main, note | `text?` | `NoteSummary` | Creates a note and shows it: empty from the main window; a note window passes its text when its file changed underneath it (R9). | FR-011 |
| `note_read` | main, note | `file` | `NoteDoc` | A note window reads only its own file. | FR-011 |
| `note_save` | main, note | `file`, `text`, `baseHash` | `SaveOutcome` | Saves when the file still has `baseHash`; otherwise `Conflict` and nothing is written (the window keeps the person's text as a new note). Over 4,000 characters: `InvalidInput("text")`. | FR-008, FR-014, FR-016 |
| `note_style` | main, note | `file`, `colour?`, `onTop?` | `NoteDoc` | Saves colour and "keep on top"; the window follows. | FR-011 |
| `show_note` | main | `file`, `shown: boolean` | `NoteSummary` | Opens or closes the note's window and remembers it. | FR-011, FR-015 |
| `hide_note` | note | — | — | The note window hides itself. | FR-011 |
| `delete_note` | main | `file` | — | Moves the file to `notes/deleted/` and closes its window. The webview asks first. | FR-015 |
| `alarms` | main | — | `AlarmList` | Every alarm, with the count of unreadable lines and whether Windows is holding back notifications. | FR-017, FR-025 |
| `save_alarm` | main | `key?`, `label`, `time`, `repeat` | `AlarmList` | Adds (no key) or changes an alarm. `InvalidInput` for a bad time, date, weekday or label. | FR-017, FR-018 |
| `delete_alarm` | main | `key` | `AlarmList` | Removes the alarm's line. | FR-017 |
| `alarm_action` | main | `key`, `action: "snooze" \| "done"` | `AlarmList` | The same as the notification's buttons, from the list. | FR-021 |

`file` is a note file name; the core accepts only names of existing files directly in `notes/` matching `<slug>-<id>.md`. A note window's commands are checked against the file its window was opened for, by window label (`note-<file name without .md>`), in the shell.

#### Changes to earlier commands

| Command | Change |
|---|---|
| `send_message`, `retry_reply`, `regenerate_reply`, `start_first_hello` | A new context records the extensions that are on and their tools (R2). Extension tool calls run and add marks like memory tools. |
| `chat_history` | Marks of the new kinds appear on replies. |
| `complete_setup` | Unchanged: both extensions are on by default with nothing to choose. |

#### The shell's tick

A thread in the shell calls `core.ext_tick(local_now)` every 15 seconds from startup to quit, whatever the windows are doing. It shows one notification for everything in `ring` (R5): a single alarm as below, several as one `N reminders` notification. If `missed` is not empty, it shows one notification listing them. If `changed` is not empty, it emits `ext_changed` with the extension ids to every window.

#### Notifications

| Notification | Title | Text | Buttons | Click |
|---|---|---|---|---|
| Alarm | the label | `<name> · HH:MM` | Snooze, Done | open the main window on the alarm list |
| Several alarms | `N reminders` | each label with its time, up to five, then "and N more" | Snooze, Done (each acts on all of them) | open the main window on the alarm list |
| Missed | `Missed reminders` | each label with its time, up to five, then "and N more" | — | open the alarm list |

Snooze calls `alarm_action(key, "snooze")` in the core for each alarm in the notification; Done, and the person dismissing it, call `alarm_action(key, "done")`. Windows hiding or timing out a notification changes nothing. The app id is `com.kodelokus.mypec` when installed and the PowerShell id when run from `target/`.

#### Windows

| Window | Label | Capability | Options |
|---|---|---|---|
| Note | `note-<file name without .md>` | `capabilities/note.json`: `core:event:default`, `core:window:allow-start-dragging`, `core:window:allow-start-resize-dragging`, `allow-note-read`, `allow-note-save`, `allow-note-style`, `allow-hide-note`, `allow-new-note` | frameless, outside the taskbar, resizable from 160 × 120, on top only when the note says so, 260 × 260 by default, placed by the core's `place` rule; its place and size saved 500 ms after the last move |

The main window's capability gains the main-window commands above. The autostart plugin's own permissions (`autostart:allow-enable` and the rest) are **not** granted to any window: launch at login changes only through `set_autostart`, so the app also records that the offer was answered.

#### Events

| Event | Payload | Sent when |
|---|---|---|
| `ext_changed` | `string[]` (extension ids) | the tick saw an extension's files change, or a tool call changed them |
| `open_alarms` | — | a notification was clicked |

#### Webview duties

- Main view header: a Notes button and an Alarms button, beside World and Dreams.
- A notes list and an alarm list, each a view like the world and the dream journal.
- Settings: an Extensions section with each extension's switch, description and folder, and the launch-at-login switch.
- After the first alarm is saved (by the person or the companion), if `autostart.offered` is false, a one-time question in the companion's line with Yes and Not now, calling `set_autostart`.
- The note window: the text as an editable area, a small bar on hover with colour, "keep on top" and hide; saving 700 ms after the last keystroke.

### Extension tools

The tools the companion is given for each extension that was on when its context started, after the memory tools, in this order. Same conventions as the memory tools (memory Interfaces): strict JSON schemas, results as text the companion reads, errors as `is_error` results with a sentence it can pass on, each change shown as a mark on the reply.

#### 1. What the companion is told (`ext-1` harness)

Added after "Your world":

> # Your extensions
>
> You have tools for the person's sticky notes and alarms, when they are turned on. Use them only when the person asks, or after you suggest it and they say yes. After using one, say briefly what you did. For an alarm, say the exact day and time you chose; if they gave no time, ask for one. Never set an alarm or write a note on your own.

#### 2. Sticky notes (`notes`)

| Tool | Input | Result | Mark |
|---|---|---|---|
| `note_create` | `text` (1 to 4,000 characters) | `Made the note "<first line>".` | `NoteMade` |
| `note_list` | — | One line per note: file name, shown or hidden, first line. `No notes yet.` when empty. | — |
| `note_read` | `note` (a file name from `note_list`) | The note's text. | — |
| `note_write` | `note`, `text`, `mode`: `append` or `replace` | `Updated the note "<first line>".` | `NoteChanged` |

`note_create` shows the new note on the desktop. `note_write` with `append` adds the text on a new line. A note that would pass 4,000 characters is refused: `That note is full (4,000 characters). Suggest a new note instead.`

#### 3. Alarms (`alarms`)

| Tool | Input | Result | Mark |
|---|---|---|---|
| `alarm_set` | `label` (1 to 200 characters), `time` (`HH:MM`), `repeat`: `once`, `daily`, `weekdays`, `weekly`; `date` (`YYYY-MM-DD`, `once` only, optional); `days` (weekday names, `weekdays` and `weekly` only) | `Set "<label>" for <Weekday d Month> at HH:MM.` or `… every day at HH:MM.` | `AlarmSet` |
| `alarm_list` | — | One line per waiting alarm, soonest first, with its key, next time and label. `No alarms set.` when empty. | — |
| `alarm_move` | `alarm` (a key from `alarm_list`), and any of `time`, `date`, `repeat`, `days`, `label` | `Moved "<label>" to …` | `AlarmChanged` |
| `alarm_cancel` | `alarm` | `Cancelled "<label>".` | `AlarmCancelled` |

A `once` alarm without `date` is the next occurrence of `time` that hasn't passed by the user's clock (FR-024); the result always states the date. A `once` alarm in the past is refused: `That time has already passed. Ask which day they mean.` A key that no longer matches a line (the person edited it, or it was cancelled) is refused: `That alarm has changed. List the alarms again.`

#### 4. When an extension is off

Every tool of an extension that is off answers, with `is_error`: `<Extension> is turned off. The person can turn it on in Settings, under Extensions.` Nothing is changed.

#### 5. Provenance

Tool results are stored as `untrusted` messages, like memory tool results, so nothing an extension returns can be promoted to memory (Principle IV). Notes and alarms are never indexed, dreamt about or summarized (FR-009).

## Checks

How to check this slice works. The automated part runs everywhere; the scenarios need a real Windows session because notifications, windows on the desktop and launch at login can't be faked.

### Manual scenarios (owner's machine, installed build)

1. Ask "remind me in 3 minutes to stretch". The reply names the time and shows the mark. Close the window to the tray. A notification rings within a minute of the time, with My Pec's name and icon, not PowerShell's.
2. Click Snooze: it rings again 10 minutes later. Click Done: the alarm shows done in the list.
3. Click the body of a notification: the main window comes to the front on the alarm list.
4. Set an alarm 5 minutes ahead and put the computer to sleep for 10 minutes. On waking, one "Missed reminders" notification lists it; the list shows it as missed; it never rings.
5. Turn Focus on, let an alarm come due, turn Focus off: the list shows it rang. Turn notifications off for My Pec in Windows settings and let an alarm come due: the list shows it rang and says once that Windows is holding back its notifications.
6. Ask "put my shopping list on a sticky: eggs, rice, tea". A note appears on the desktop. Type in it, drag it, make it blue, keep it on top. Restart: it comes back the same.
7. Open `notes/` in a text editor and change the note: the note window shows the change when focused, and "what's on my shopping list?" answers from the new text.
8. Edit `alarms.md`: change a time, add a line by hand, add a line that makes no sense. The list shows the first two and says one line couldn't be read; the bad line is still in the file.
9. Turn Sticky notes off in settings: notes leave the desktop, the files stay. Ask for a note: the companion says notes are off and where to turn them on.
10. Save the first alarm with launch at login off: the question appears once. Say yes, sign out and in: My Pec starts in the tray without its window, and shown notes reappear.
11. With 1, 5 and 10 notes shown and the main window in the tray, read My Pec's total memory in Task Manager (R7 records the expected cost).

### Release checks (owner)

| Check | How | Pass |
|---|---|---|
| SC-001 | 20 reminders asked of the companion with the real model, window in the tray. | Every alarm at the intended local time; every notification within a minute. |
| SC-004 | Cut power while typing in a note (or kill the process). | At most the last second lost; the file is never half-written. |
| SC-007 | 50 notes and 100 alarms. | Lists open instantly; tray memory within NFR-PERF-4 with notes hidden. |
| SC-008 | Network capture of a day with notes, alarms and launch at login. | Only the provider. |
