# My Pec — Requirements

**Status:** Draft v0.3 for review · **Date:** 2026-11-14 · **Companion document:** [architecture.md](architecture.md)

## 1. What My Pec is

My Pec is a desktop app that gives you one persistent AI companion, named Pec by default. You install it, paste an API key, give the companion a SOUL (who it is) and an avatar (how it looks), and start talking. From then on the companion keeps its own MEMORY: it takes notes while you talk, compacts long conversations without losing what matters, and consolidates the day's notes into long-term memory while you are away. The memory design follows OpenClaw's: Markdown files as the source of truth, a memory flush before compaction, and a nightly "dreaming" sweep.

The product makes three promises:

1. **Seamless.** No account, no server, no config files. Installer, key, soul, avatar, first reply.
2. **Pec remembers.** What you told the companion last week is still there next week, without you managing anything.
3. **It is yours.** Everything lives on your machine as plain Markdown that you can read, edit and delete. Conversation content goes only to the LLM API you configured.

## 2. Scope of v1

**In scope**

- Windows desktop app with a one-click installer and a first-run wizard.
- Bring-your-own API key, stored in the operating system's credential protection.
- SOUL setup and an animated Live2D avatar, both changeable later.
- One continuous text conversation with streaming replies.
- The full memory loop: daily notes, memory search, compaction with a pre-compaction flush, and nightly dreaming with a dream diary.
- Full user control: browse, edit, forget, export and wipe everything the companion knows.
- A world the companion draws for itself, a little on each day you talk, from a dictionary of small drawings that ships with the app.

**Out of scope for v1** (the architecture leaves room for each)

- Voice input and output.
- 3D (VRM) avatars.
- Proactive check-ins, where the companion starts a conversation on its own.
- Tools beyond memory and the bundled extensions (D18): web browsing, file access, MCP servers.
- More than one companion, more than one device, cloud sync, a mobile app.
- Messaging channels (WhatsApp, Telegram and so on). My Pec is a local app, not a gateway.

## 3. Assumptions

These are the defaults this draft is written against. Section 9 records which are decided and which are still assumed.

| # | Assumption |
|---|---|
| A1 | "The API key" is an Anthropic Claude API key. Other providers can be added behind the same interface later. |
| A2 | Windows 11 is the first platform. macOS follows. Linux is not planned yet. |
| A3 | One user, one companion, one device. |
| A4 | The app has no backend of its own. Background work such as dreaming happens only while the app is running, normally in the system tray. |
| A5 | v1 is text chat. The avatar is a Live2D character that moves with mood and activity. |

## 4. Concepts

| Term | Meaning |
|---|---|
| Companion | The single AI persona the user talks to. Default name: Pec. |
| SOUL | `SOUL.md`. The companion's personality, voice, values and boundaries, in plain Markdown. Loaded into every request. |
| Identity | `IDENTITY.md`. Name, pronouns, the avatar reference, and the companion's id and birth date. |
| User profile | `USER.md`. Stable facts and preferences about the user, written as current truths and replaced in place when they change. |
| Long-term memory | `MEMORY.md`. A short, curated list of durable facts and standing decisions. Not a transcript. |
| Daily notes | `memory/YYYY-MM-DD.md`. Append-only notes the companion writes during the day. |
| Transcript | The complete conversation, kept on disk forever unless the user deletes it. |
| Compaction | Replacing older conversation turns with one summary so the conversation fits the context budget. The full transcript stays on disk. |
| Memory flush | A silent turn just before compaction in which the companion saves anything durable to the daily note. |
| Dreaming | A background sweep that turns short-term material into long-term memory in three phases: light, REM, deep. |
| Dream diary | `DREAMS.md`. A short narrative entry per sweep, written for the user to read. Never used as a memory source. |
| Companion id | A short random identifier, unique to one companion, created on the device. |
| DNA | The companion's colours, the lay of its land, and which items it favours. Worked out from the id, never stored. |
| Item dictionary | The small drawings that ship with the app. Each has a permanent id. Everything in a world comes from it. |
| World | `world/`. Everything the companion has drawn, one item per line, in the order it was drawn. Append-only. |

## 5. User journeys

**J1. First run.** The user runs the installer and the app opens on a wizard. They paste an API key, which is checked on the spot. They pick a soul template and tweak the name. They pick a character for the avatar. Pec says hello in character. Every step except the key has a working default, so pressing Next four times is enough.

**J2. An ordinary day.** The user opens the chat from the tray and talks. Pec answers in character, knows what day it is, and brings up relevant things from earlier conversations without being asked. When the user says something worth keeping, Pec notes it down quietly.

**J3. A very long conversation.** After hours of talk the conversation outgrows its budget. Pec saves loose ends to the daily note, the older turns are summarized, and the chat carries on. The user sees at most a thin divider in the timeline.

**J4. Overnight.** While the user is away the companion dreams. The avatar sleeps. Notes from recent days are sorted, recurring themes are found, and the strongest items are promoted to long-term memory. In the morning the dream diary has a new entry and Pec starts the day with the updated memory.

**J5. Checking and correcting.** The user opens the memory browser, reads what Pec believes, fixes a wrong fact, deletes something they regret sharing, and can see which conversation each memory came from.

**J6. A world of its own.** On days the user talks with Pec, Pec draws a few small things in its world. Over weeks the world fills in around Pec's home, in colours and a layout no other companion has. The user opens it now and then to look around. When they have been away, it is exactly as they left it.

## 6. Functional requirements

Priority: **M** is required for v1. **S** should follow in v1.x. **C** could come later.

### 6.1 Install and onboarding

| ID | Pri | Requirement |
|---|---|---|
| ONB-1 | M | One installer per operating system. Per-user install with no administrator rights and no separate runtime to install. |
| ONB-2 | M | First-run wizard with the steps key, soul, avatar, optional "about you", first hello. Only the key step is mandatory. Every other step has a default that works. |
| ONB-3 | M | The wizard can be resumed. Quitting midway never leaves the app half-configured. |
| ONB-4 | M | When the wizard finishes, the companion sends the first message in character. |
| ONB-5 | S | A conversational alternative to the soul form, in which the companion and the user work out its personality together in the first chat. |
| ONB-6 | C | Import an existing OpenClaw workspace (`SOUL.md`, `USER.md`, `MEMORY.md`, `memory/`). |

### 6.2 API key and model

| ID | Pri | Requirement |
|---|---|---|
| KEY-1 | M | The user pastes a key and it is validated immediately with a call that costs nothing. Invalid key, no credit and no network each get their own plain-language message. |
| KEY-2 | M | The key is stored encrypted with the operating system's credential protection. It is never written in plaintext, never logged, never shown in full again, and never returned to the user interface once entered. |
| KEY-3 | M | A model picker filled from the provider's model list, with a default already selected. |
| KEY-4 | M | The key can be replaced or removed in settings. Without a key the app still opens and lets the user browse memory and history. |
| KEY-5 | S | A usage meter showing tokens and estimated cost per day and per month, with an optional soft daily limit that pauses background work first and warns before chat. |
| KEY-6 | C | More providers (OpenAI-compatible endpoints, local models) through the same provider interface. |

### 6.3 SOUL

| ID | Pri | Requirement |
|---|---|---|
| SOUL-1 | M | The soul is the plain Markdown file `SOUL.md`. Name, pronouns and avatar reference live in `IDENTITY.md`. |
| SOUL-2 | M | Soul creation from a gallery of at least four templates plus a blank one. The text is fully editable. |
| SOUL-3 | M | The soul shapes every reply. An edit takes effect from the next message. |
| SOUL-4 | M | The companion never changes its own soul. |
| SOUL-5 | M | A size budget with a visible indicator, because the soul is sent with every request. |
| SOUL-6 | S | A soul editor with a live preview ("say hello as this soul"). |
| SOUL-7 | S | The companion may propose a soul edit. The user sees it as a diff and approves or rejects it. |

### 6.4 Avatar

| ID | Pri | Requirement |
|---|---|---|
| AVA-1 | M | The avatar is an animated Live2D character. v1 ships with at least one bundled character, chosen in the wizard and changeable later. |
| AVA-2 | M | The avatar shows what the companion is doing: idle, listening, thinking, speaking, sleeping (while dreaming). It also shows a mood from a small fixed set: neutral, happy, playful, concerned, sad, surprised. |
| AVA-3 | M | The avatar is docked in the chat window. |
| AVA-4 | M | Avatar problems never block chat. If the character fails to load or the graphics layer is unavailable, the chat continues with a static portrait. |
| AVA-5 | M | The character animates only while it is visible, and holds a still pose when the operating system's reduced-motion setting is on. |
| AVA-6 | S | Import a Live2D model of the user's own, with a screen that maps its expressions and motions onto the companion's moods and activities. Shipping this in a public build depends on Live2D's licensing review (D9). |
| AVA-7 | S | A floating mode: a small always-on-top avatar on the desktop that can be dragged and clicked to open the chat. |
| AVA-8 | S | The character's eyes and head follow the pointer. |
| AVA-9 | S | A plain picture as the avatar, for users who prefer one. |
| AVA-10 | C | VRM 3D avatars, and lip-sync to speech once voice exists. |

### 6.5 Chat

| ID | Pri | Requirement |
|---|---|---|
| CHAT-1 | M | One continuous conversation with streaming replies and Markdown rendering. The user never has to manage threads. |
| CHAT-2 | M | History survives restarts. The user can scroll back through everything, including turns the model no longer sees after compaction. |
| CHAT-3 | M | Stop a reply while it is being generated. Regenerate the last reply. |
| CHAT-4 | M | Network errors, rate limits, provider overload and refusals are shown in plain language with a retry. The user's typed message is never lost. |
| CHAT-5 | M | The companion knows the current local date and time. |
| CHAT-6 | S | Image input by paste or drag and drop. |
| CHAT-7 | S | A global hotkey that shows or hides the chat. |
| CHAT-8 | C | Proactive check-ins with strict limits the user controls. |
| CHAT-9 | C | Voice input and output. |
| CHAT-10 | M | Replies read like a person messaging someone close: short by default, in the everyday spoken form of the user's language, with no filler openings, closing offers or formatting in ordinary chat. A reply of a few short plain paragraphs shows as a few bubbles, like messages sent in a row. The soul still decides the voice. |

### 6.6 Memory

| ID | Pri | Requirement |
|---|---|---|
| MEM-1 | M | Memory is layered and stored as files: `USER.md`, `MEMORY.md`, `memory/YYYY-MM-DD.md`, `DREAMS.md`, and the transcript. |
| MEM-2 | M | The companion writes daily notes on its own during conversation. |
| MEM-3 | M | When the user explicitly asks ("remember that..."), the fact becomes durable immediately, without waiting for a dream. |
| MEM-4 | M | Profile facts such as the user's name or a stated preference update `USER.md` directly, replacing the old value. |
| MEM-5 | M | `USER.md` and `MEMORY.md` are loaded at the start of every context. Today's and yesterday's daily notes are loaded when a new context starts. Older notes are reached through search. |
| MEM-6 | M | The companion can search memory by keyword and by meaning. Keyword search works with no extra download. Semantic search switches on when a local embedding model is available. |
| MEM-7 | M | Auto-recall: the most relevant memory snippets are attached to the user's message automatically, within a small budget. It can be switched off. |
| MEM-8 | M | Files are the source of truth. The search index is derived and can be rebuilt at any time. Edits the user makes in any text editor are picked up. |
| MEM-9 | M | Always-loaded files have size budgets. Over budget, the file stays whole on disk, the loaded copy is truncated, and the app warns the user. |
| MEM-10 | M | Every memory candidate records where it came from: the user, the companion, or untrusted content such as a tool result. Untrusted content is never promoted to long-term memory automatically. |

### 6.7 Compaction

| ID | Pri | Requirement |
|---|---|---|
| CMP-1 | M | When the conversation nears its context budget, older turns are summarized into one entry and recent turns are kept word for word. The conversation continues with no action from the user. |
| CMP-2 | M | Before compaction a silent memory flush lets the companion save durable context to today's daily note. The flush only appends, is never visible in chat, and its failure never blocks compaction. |
| CMP-3 | M | The full transcript stays on disk. Compaction changes only what the model sees. |
| CMP-4 | M | The summary keeps the active topic or request, open commitments, the tone of the relationship, and identifiers (names, dates, numbers, paths) exactly as written. |
| CMP-5 | M | If the provider reports that the context is too long, the app compacts and retries once automatically. |
| CMP-6 | M | If no valid summary is produced, the history stays as it was and compaction is retried later. History is never dropped. |
| CMP-7 | S | A "compact now" action with an optional focus note, and a thin divider in the timeline where a compaction happened. |
| CMP-8 | S | The context budget and the model used for compaction are configurable. |

### 6.8 Dreaming

| ID | Pri | Requirement |
|---|---|---|
| DRM-1 | M | A background sweep, daily at 03:00 local time by default, runs three phases in order: light (stage and deduplicate), REM (themes and reflection), deep (score and promote). Only the deep phase writes long-term memory. |
| DRM-2 | M | If the computer was off or asleep at the scheduled time, the sweep runs at the next idle moment. It never runs during a reply, and chat always takes priority over it. |
| DRM-3 | M | Promotion is gated by a minimum score, a minimum recall count and a minimum number of distinct queries. The score combines relevance, frequency, query diversity, recency, recurrence across days, and conceptual richness. |
| DRM-4 | M | Promotions are discrete decisions (add, merge, supersede), each traceable to its source lines. Every rewrite is validated and reversible: the previous file is kept and "undo last dream" restores it. |
| DRM-5 | M | Each sweep with useful material writes a short dream diary entry in the companion's voice. The diary is viewable in the app and is never used as a memory source. |
| DRM-6 | M | Only real conversation is dream input. Background turns and recalled-memory snippets are excluded, so the companion cannot relearn its own recalls as new facts. |
| DRM-7 | M | Dreaming is on by default, with an on/off switch, a schedule setting and a "dream now" action. |
| DRM-8 | S | After a dream, the next conversation starts with the updated memory. |
| DRM-9 | S | The model used for dreaming is configurable, and batch processing can be used to lower cost. |
| DRM-10 | S | The avatar sleeps while dreaming, and the companion may mention the dream in its morning greeting. |

### 6.9 Control and transparency

| ID | Pri | Requirement |
|---|---|---|
| CTL-1 | M | A memory browser to view and edit the soul, the user profile, long-term memory, daily notes and the dream diary, plus an "open folder" action. |
| CTL-2 | M | Forget: the user can delete specific memories. Deleted items are removed from the files, from the search index and from future dream input. Deletion asks for confirmation. |
| CTL-3 | M | Each long-term memory links back to the note and date it came from. |
| CTL-4 | M | Export all data as one archive. Wipe all data. |
| CTL-5 | S | An activity log of compactions, flushes and dreams: when they ran, what changed, tokens used. |
| CTL-6 | S | "Off the record" mode: nothing said while it is on is written to notes, summarized, or used by dreaming. |

### 6.10 App shell

| ID | Pri | Requirement |
|---|---|---|
| SYS-1 | M | A tray icon, close-to-tray behavior, and a single running instance. |
| SYS-2 | M | Automatic updates with visible release notes. |
| SYS-3 | M | Settings for everything above, with safe defaults. No setting is required to start. |
| SYS-4 | M | Without a network connection the app still opens for browsing memory and history, and the chat clearly shows that it is offline. |
| SYS-5 | M | Launch at login: a setting, off by default, offered once when the first alarm is saved, with the explanation that alarms ring only while My Pec runs and that it lets the companion dream overnight. Started at sign-in, My Pec waits in the tray (feature 010). |
| SYS-6 | S | Signed installers, so that Windows SmartScreen does not warn on install. Required before any public release. |

### 6.11 Personal experience

**Added on 2026-11-13.** These came out of the design work in `design/001-app-shell-onboarding.figcraft` and `design/001-ux-notes.md`. Decision D11 chose the direction: the companion leads each screen inside the current window. The design shows PER-1 to PER-10, except the tray tooltip. The wording below is still open for review, and PER-12 waits for the avatar slice.

My Pec should feel like meeting someone, not like configuring a tool. These requirements apply to every screen. Where one conflicts with clarity, accessibility or honesty, NFR-A11Y-1 and NFR-SEC-6 win.

| ID | Pri | Requirement |
|---|---|---|
| PER-1 | M | The companion is present on every screen. Its portrait (later its avatar) and its name are visible in the wizard, the main window and settings. No screen is only a form. |
| PER-2 | M | Text that asks the user for something, or reports how the companion is doing, is written in the companion's first-person voice and shown as a speech bubble next to the portrait ("What should I call you?"). Control labels, security facts and settings stay in plain, neutral wording. |
| PER-3 | M | Interface lines use one warm, neutral first-person voice whatever the soul says, so they can be translated (NFR-I18N-1) and never contradict a soul the user wrote. Only real replies come from the model in character. |
| PER-4 | M | The interface uses the names it knows: the companion's name wherever it refers to the companion, and the user's "call me" name in greetings once it is set. Every line also works when that name is not set. |
| PER-5 | M | The main window opens with a greeting that follows the local time of day ("Good evening, Sam") and a line saying whether the companion is available, instead of a toolbar with a title. |
| PER-6 | M | Problems are told in the companion's voice with the facts first. Each message still says which problem it is and what to do, in the same distinct cases as KEY-1 and CHAT-4, and keeps its action button. Two different problems never read the same. |
| PER-7 | M | Shapes and colour are soft: rounded controls, the accent tint as a background wash, and no toolbar-style header in the wizard or the main window. Settings may stay conventional. |
| PER-8 | M | Personality never costs clarity or access. Every in-voice line is real text that a screen reader announces, bubble text meets contrast requirements, and motion follows the reduced-motion setting. Nothing personal is used to pressure the user: no guilt, no streaks, no invented urgency (NFR-SEC-6). In-voice lines never claim feelings or memories the app cannot back up. |
| PER-9 | S | Each personality template ships one short sample line in its own voice. Picking a template shows that line at once, with no request to the provider. SOUL-6 remains the live, model-written preview. |
| PER-10 | S | Small moments mark progress. Setup ends with "Meet Pec" (the companion's name) instead of "Finish". The first close-to-tray notice is said by the companion ("I'll be right here in the tray"). The tray tooltip reads "Pec is here". |
| PER-11 | S | The portrait reacts to interface moments with the moods of AVA-2: happy when the key is accepted, concerned when something is wrong, asleep while dreaming. Until the animated avatar exists, the still portrait is enough. |
| PER-12 | S | The default window is a compact, portrait-shaped companion window (about 480 × 720) that the user can make larger. Depends on D11. |

### 6.12 World

**Added on 2026-11-14.** Each companion slowly draws a world of its own. The idea comes from virtual pets, and takes only half of it: the companion and its world grow with time and conversation, but nothing is ever lost through neglect (D12). WLD-1 to WLD-8 are detailed in [features/world.md](features/world.md). WLD-9 and WLD-10, revised on 2026-11-14 so that the world becomes a visual map of the user's diary (D14), are detailed in [features/journey.md](features/journey.md).

| ID | Pri | Requirement |
|---|---|---|
| WLD-1 | M | Every companion has a unique, random id and a birth date, created on the device and recorded in `IDENTITY.md`. A companion from before this feature gets both the first time the app runs. |
| WLD-2 | M | The id is a hidden seed: it decides the companion's colours, the lay of its land, which items it favours and the roll of chance, worked out whenever needed and never stored. The world's DNA is what it is made of: the share of each subject among the objects drawn for the user's days (D14). |
| WLD-3 | M | The companion has a world: a blank canvas with its home at the centre, which grows outward as a map. Everything in it comes from a dictionary of small drawings that ships with the app: scenery, and everyday objects filed under a fixed list of subjects (everyday activities and occasions). Each item and subject has a permanent id, and both lists only grow. |
| WLD-4 | M | The world is kept in the workspace (`world/`) as a human-readable file that the app only appends to, one drawn item per line. What was drawn is stored, not worked out again, so an update to the app or its dictionary never changes the past. Edits the user makes to the file are honored. |
| WLD-5 | M | On each local day with at least one message from the user, the companion draws a small number of items, once. The number is capped and does not depend on how much was said. Drawing uses no model call and never delays a reply. |
| WLD-6 | M | The world never decays, and absence costs nothing. Nothing is removed or changed because time passed, missed days are never made up, and the app shows no streaks, countdowns, badges or reminders about the world (NFR-SEC-6, PER-8). |
| WLD-7 | M | The dictionary is divided into stages. More kinds of items become available as the world grows, and nothing drawn earlier is replaced. |
| WLD-8 | M | A world view opened from the main window shows the whole world with zoom and pan, a tooltip for each item with its name, its date and the keywords of its day, the birth date, a diary list of everything drawn as a full text alternative (NFR-A11Y-1), why each object was picked, the world as it was on any past day, and what it is made of. The id is not shown. A problem with the world never blocks chat. |
| WLD-9 | S | During dreaming the companion names up to three topics for each ended day the user talked about (what they did, what happened to them, or what was on their mind, as they told it), each with a keyword, a line and one or two subjects. For each topic the app picks an everyday object by weighted chance, less likely the more it was drawn, and draws it beside that day's scenery. The model never chooses an item or produces a drawing (D15). Each pick and its reasons are recorded in the workspace beside the world file. The user can forget a topic's story, and forgetting a memory forgets the stories of its day; the object stays (CTL-2). |
| WLD-10 | S | The companion knows its world in conversation, through a short description given at the start of a context and a notice after a night that drew something. |
| WLD-11 | C | The companion's own look changes as its world grows. |
| WLD-12 | C | The world can be saved as a picture. |

### 6.13 Extensions

**Added on 2026-11-14** (D18). Small, bundled pieces of usefulness the companion can use when asked and the user can also use directly. Each one keeps its data as plain files in the workspace. Detailed in [features/extensions.md](features/extensions.md); Weather in [features/weather.md](features/weather.md).

| ID | Pri | Requirement |
|---|---|---|
| EXT-1 | M | Two bundled extensions, sticky notes and alarms, both on by default with nothing to set up (Weather, EXT-8, starts off). Settings lists them with what each does and where its files are, and turns each on or off at once without touching its files. |
| EXT-2 | M | An extension reaches only its own place in the workspace and its own bookkeeping file, and never the key. It reaches the network only through the core, only while it is on, and only the destinations its spec names. A failing extension never blocks chat. |
| EXT-3 | M | The companion uses an extension's tools only when the user asks or agrees, and each change shows on its reply. A context keeps the tools it started with; a tool of an extension turned off since is refused. |
| EXT-4 | M | Extension data is not memory: not indexed, dreamt about or summarized. |
| EXT-5 | M | Sticky notes: one Markdown file each in `notes/`, at most 4,000 characters, shown as small frameless windows on the desktop that save as the user types, keep their place, colour and "keep on top", and come back after a restart. A notes list shows every note; a deleted note moves to `notes/deleted/`. |
| EXT-6 | M | Alarms: one line each in `alarms.md`, once, every day, on chosen weekdays or every week, at local wall-clock time. While the app runs, an alarm rings within a minute with a Windows notification offering Snooze (10 minutes) and Done; alarms due together ring as one. Alarms missed while the app was closed or the computer slept are reported once and never rung late. Lines the app can't read are kept and counted. |
| EXT-7 | M | Edits made to note and alarm files outside the app are picked up and never overwritten. |
| EXT-8 | M | Weather: a third extension, off until the user turns it on. The user types a city and picks one of up to five matches; its current weather from Open-Meteo is drawn above and around the character, with a short weather line, and Pip reacts to it. Only the typed name or the place's coordinates are sent. The place is `weather.md`. |
| EXT-9 | M | Weather refreshes about every 30 minutes and after the computer wakes, never more than once in 10 minutes; a failure is silent outside settings, and weather older than 3 hours is neither drawn nor told. |
| EXT-10 | M | While the weather is known and recent, each new message tells the companion the weather in its `<context>` block, in contexts that recorded Weather; nothing earlier changes. |

## 7. Quality requirements

### Privacy and security

| ID | Requirement |
|---|---|
| NFR-SEC-1 | Local-first. All data stays on the user's disk. The only outbound connections are the configured LLM API, the update check, and a one-time download of the embedding model. |
| NFR-SEC-2 | No telemetry or analytics unless the user opts in. |
| NFR-SEC-3 | The user enters the API key in the user interface, which hands it to the background process once and keeps no copy. After that only the background process holds it, and the user interface can ask whether a key exists and nothing more. |
| NFR-SEC-4 | The user interface runs sandboxed, loads no remote content, and reaches the rest of the app only through a fixed, typed set of calls. |
| NFR-SEC-5 | Durable memory is protected against poisoning: content that did not come from the user or the companion cannot be promoted, and every automatic rewrite can be undone. |
| NFR-SEC-6 | The companion is honest about being an AI. The app uses no engagement tricks such as guilt-inducing notifications. |

### Reliability and data integrity

| ID | Requirement |
|---|---|
| NFR-REL-1 | Memory files are written atomically. The transcript and the world file are append-only. A crash or power loss costs at most the message in flight, or the world items being written. |
| NFR-REL-2 | The search index and all dreaming state can be rebuilt from the files. |
| NFR-REL-3 | Background jobs are checkpointed and safe to rerun. A crash mid-compaction or mid-dream leaves memory exactly as it was before the job started. |
| NFR-REL-4 | The previous version of `SOUL.md`, `USER.md` and `MEMORY.md` is kept before every automatic rewrite. |

### Performance

| ID | Requirement |
|---|---|
| NFR-PERF-1 | Cold start to a usable chat window in 3 seconds or less on a mid-range laptop. |
| NFR-PERF-2 | Work done locally before a request is sent (context assembly and recall) takes 150 ms or less at the 95th percentile. |
| NFR-PERF-3 | Memory search returns in 100 ms or less at the 95th percentile with 10,000 indexed chunks. |
| NFR-PERF-4 | In the tray with no window open, the app uses 150 MB of memory or less and close to no CPU. With the avatar on screen and idle, it uses 400 MB or less and animates at no more than 30 frames per second. |

### Cost

| ID | Requirement |
|---|---|
| NFR-COST-1 | The request layout is designed for prompt caching: stable content first, changing content last, history only ever appended to. |
| NFR-COST-2 | The context budget is set well below the model's maximum on purpose, and always-loaded files are bounded, so the cost of a turn stays predictable. |
| NFR-COST-3 | The user can see what the app is spending and can choose cheaper models for chat and for background work. |

### Portability, accessibility, maintainability

| ID | Requirement |
|---|---|
| NFR-PORT-1 | All memory and conversation logic is independent of the desktop shell and runs headless under test. |
| NFR-A11Y-1 | Full keyboard navigation, screen-reader labels, and avatar animation that respects the operating system's reduced-motion setting. |
| NFR-I18N-1 | The companion converses in any language the model supports. Search handles non-English text. Interface strings are externalized for translation. The interface is available in English and Indonesian, chosen in settings; English until the user chooses. |
| NFR-TEST-1 | Context assembly is deterministic and covered by golden tests. Compaction and dreaming are tested against a fake LLM. World growth is decided only by the id, the date and the world so far, and is covered by golden tests. |

## 8. Success criteria

| # | Criterion | How it is measured |
|---|---|---|
| SC-1 | A new user gets the first in-character reply within 3 minutes of launching the installer, having typed nothing but the API key. | Timed walkthrough on a clean Windows 11 machine. |
| SC-2 | Facts told to the companion are still known a week later in a fresh context. | Scripted suite of 30 facts across simulated days. At least 27 are recalled correctly. |
| SC-3 | Compaction is invisible. The companion continues the active topic without asking the user to repeat it. | Scripted long conversations. At least 95% continue correctly, with no user-visible error. |
| SC-4 | Dreaming never loses memory. | Automated checks: a rewrite that drops more than 25% of prior entries is rejected, and undo restores the previous file byte for byte. |
| SC-5 | Nothing leaves the machine except to the configured LLM endpoint and the update server, and, while the user has Weather on, Open-Meteo's place lookup and weather. | Network capture during a full session, including a dream. |
| SC-6 | The idle footprint meets NFR-PERF-4. | Measured after 1 hour idle in the tray. |

## 9. Decisions

### Decided on 2026-11-13

| # | Decision | Choice | Consequence |
|---|---|---|---|
| D1 | Provider scope for v1 | Claude only, behind a provider interface | One key and one tested path keeps setup seamless. The interface keeps other providers cheap to add. |
| D2 | Desktop shell | Tauri 2 | A small installer and low memory use. The core is written in Rust, and because there is no official Claude SDK for Rust, the project maintains its own API client. |
| D3 | Avatar for v1 | Live2D | An animated character from day one. Live2D's licensing terms apply at release (D9, D10). |
| D8 | Where the core logic runs under Tauri | Rust, in the same process as the shell | No second runtime in the installer. The alternative, a bundled Node sidecar with the official Claude SDK, was rejected. |
| D11 | How personal the window feels (section 6.11) | The companion leads each screen inside the current window (Direction A in `design/001-ux-notes.md`) | Only wording, layout and style values change in the webview. A compact frameless window (Direction B, PER-12) is revisited with the avatar slice, because it needs custom window chrome and overlaps with floating mode (AVA-7). |

### Decided on 2026-11-14

| # | Decision | Choice | Consequence |
|---|---|---|---|
| D12 | How the companion's world grows (section 6.12) | A small, capped amount on days with a conversation, with no decay, no catch-up for missed days and no model call | The growth half of a virtual pet is kept and the neglect half is left out, because NFR-SEC-6 and PER-8 rule out guilt, streaks and invented urgency. Chatting more does not grow the world faster. |
| D13 | What the world looks like, and what evolves | A map that grows outward from the companion's home. The world evolves; the companion's own look does not | The whole world reads at a glance as one identity. Stages for the avatar would multiply the art and depend on the open Live2D questions (D9, D10), so they stay a later option (WLD-11). |
| D14 | What the world stands for (2026-11-14) | A visual map of the user's diary: everyday objects for what really happened to the user, as they told it, each with the keyword of its day | The world means something to the user, not only to the companion. The dictionary needs many everyday objects, and the id stops being shown: what makes a world unique is what it is made of. |
| D15 | Who chooses the object for a topic | The companion names the topic and its subjects; the app picks the object by seeded weighted chance | The dictionary is never sent to the model, picks can be tested exactly, and a steady life does not fill the world with one object. The keyword carries what happened; the object is a likeness. |
| D16 | How My Pec is released, and under what license | Open source on GitHub under the MIT License, copyright Kodelokus. Installers are attached to GitHub Releases; the Microsoft Store comes later | No store fees, signing certificate or store review stand in the way of the first release. The installer is unsigned until signing is set up, so users meet a SmartScreen warning. GPL-3.0 was rejected because Live2D's agreements forbid putting Cubism code under a license like it. Cubism Core and the Cubism Web Framework stay under Live2D's licenses and are never committed (architecture 9.5). Kodelokus's sales are under ¥10M a year, so Live2D's small-business exemption covers releasing it. |
| D17 | The product's name and home | My Pec, at https://github.com/kodelokus/my-pec. The rename is done before the first public push | Nobody has installed it yet, so the app identifier, data folder and credential entry can change without a migration. Specs 001 to 009 keep the old name as history. |
| D18 | Usefulness beyond chat | An extension system: each extension gives the companion tools, keeps its data as plain files in the workspace, and may add its own window, panel or notification. The first two are sticky notes and alarms. They live in the main repository, as their own crate and UI folder behind a strict boundary | The companion becomes the way to use everyday tools ("remind me at 7"). A separate repository would add paired pull requests and submodule bumps while the extension API is still changing, so the split waits until the API settles or outside authors want to write extensions, which would also need runtime plugins. Alarms must fire with the window closed, so launch at login (SYS-5) and Windows notifications move into scope. Sticky notes share the always-on-top window work with the avatar's floating mode (AVA-7). |
| D19 | How people get access to a model | First release: each person uses their own Claude API key (KEY-1), on the Microsoft Store as an MSIX package and as a direct download ([features/release.md](features/release.md)). Next: a hosted subscription, "My Pec Cloud", offered beside the own-key option as another provider behind the same interface | The Store accepts apps that need an API key from a Company account, which Kodelokus has, so the first release needs no server. A subscription removes the key step for most people but needs accounts, billing and a server of the project's own, which Principles II and VII rule out today: it comes with its own feature page and an amendment to the principles. Earning money also ends Live2D's small-business exemption above ¥10M a year and needs Open-Meteo's paid plan. |

### Raised by those choices, still open

| # | Decision | Recommendation | Why |
|---|---|---|---|
| D9 | User-imported Live2D models in public builds | Build the feature behind a flag, and ask Live2D whether the app counts as an Expandable Application before switching it on | Individuals and small businesses are exempt from Live2D's release license, except for Expandable Applications, which need Live2D's review and a special agreement at least a month before release. An app that loads any user's model may count as one. Live2D generally does not approve a fully free Expandable Application, so the open-source release (D16) keeps import off. |
| D10 | The bundled default character | An original model owned by the project | Live2D's sample models carry their own terms, and it is not confirmed that one may be bundled in a published app. |

D9 and D10 matter only if My Pec is released to other people. For personal use, Live2D requires no license. My Pec will be released (D16), so both must be settled before the avatar ships.

### Assumed, not yet confirmed

| # | Decision | Recommendation | Why |
|---|---|---|---|
| D4 | First platform | Windows 11, then macOS | Matches the development machine. |
| D5 | Voice in v1 | No | It roughly doubles the surface (speech recognition, speech synthesis, lip-sync) without touching the memory loop that defines the product. |
| D6 | Where the workspace lives | App data folder, with an "open folder" action and an option to move it | The Documents folder is often synced by OneDrive, which interferes with frequent small writes. |
| D7 | Default chat model | `claude-opus-5-5`, user-selectable | Best quality by default. Cost is the user's call through the model picker (KEY-3, NFR-COST-3). |

## 10. Suggested delivery slices

Each slice is described in its own page under [features/](features/) and ends in something a user can try.

| # | Slice | Delivers | Depends on |
|---|---|---|---|
| 1 | App shell and onboarding | Installer, wizard, key storage, model picker | — |
| 2 | Chat runtime | Streaming chat in character, persistent transcript, soul in context | 1 |
| 3 | Memory core | Memory files, memory tools, keyword search, auto-recall, memory browser | 2 |
| 4 | Compaction | Budget tracking, memory flush, summary, new context after compaction | 3 |
| 5 | Dreaming | Scheduler with catch-up, three phases, promotion, dream diary, undo | 3 |
| 6 | Avatar | Live2D character, moods and activity states, static fallback, floating mode | 2 |
| 7 | Semantic search | Local embedding model, hybrid ranking | 3 |
| 8 | Release readiness | Auto-update, signing, usage meter, export and wipe | all |
| 9 | World core | Companion id and birth date, item dictionary, world file, daily growth with no model call, world view | 2 (3 for the entry point) |
| 10 | World journey | The day's topics named during dreaming, an everyday object picked for each and recorded with its reasons, tooltips and a diary list in the world view, the companion knows its world | 5, 9 |

Slices 4, 5 and 6 are independent of each other once slice 3 (or 2, for the avatar) is done. Slice 9 needs only slice 2, apart from opening its view from beside the memory browser of slice 3, and is numbered after the others so that their numbers stay as they were. Each slice's page in [features/](features/) is named after what it delivers, not its number: slice 9 is [world](features/world.md) and slice 10 [journey](features/journey.md).
