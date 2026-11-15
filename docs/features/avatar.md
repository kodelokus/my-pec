# Avatar

Built.

Today the companion is a still portrait beside the chat. This feature brings it to life: a drawn character that breathes and blinks, turns toward the user while they type, looks away to think while a reply is being prepared, moves its mouth while the reply streams in, and sleeps while it dreams. Each reply also carries the companion's mood (happy, playful, concerned, sad, surprised or neutral), and the character's face shows it. The user can keep the character in the chat window or let it float on the desktop. Whatever goes wrong with the character, the conversation never notices.

## User stories

### US1. Steve is alive while we talk (P1)

The user opens the chat window and Steve is there, breathing and blinking. When they start typing, Steve looks up at them. When they send, Steve looks away and thinks. When the reply starts to appear, Steve's mouth moves along with the words, and when the reply ends Steve goes back to resting. The user tells Steve their cat finally came home; Steve's reply is warm and the character smiles. A week later they share bad news, and Steve's face turns concerned.

**Why this priority**: This is the feature. A companion that reacts while it listens, thinks and answers feels present in a way a still picture never does (A5, AVA-1, AVA-2).

**How to check**: With the bundled character, type, send and receive replies with each of the six moods from recorded responses, and confirm the character's activity follows each step and its expression follows each reply's mood, while the reply text never shows the mood marker.

**Acceptance Scenarios**:

1. **Given** the chat window is open and nothing is happening, **When** the user looks at the character, **Then** it is idle: breathing and blinking.
2. **Given** the character is idle, **When** the user types in the message box, **Then** it is listening, and it returns to idle when the box is empty or loses focus.
3. **Given** the user sends a message, **When** the reply has not begun to appear, **Then** the character is thinking, including while the companion uses its tools.
4. **Given** a reply is streaming, **When** text appears, **Then** the character is speaking, with its mouth moving in rhythm with the text, until the reply ends and it returns to idle.
5. **Given** a reply that states a mood, **When** it arrives, **Then** the character's expression shows that mood, and the mood holds until the next reply arrives.
6. **Given** a reply with no mood or an unknown one, **When** it arrives, **Then** the character shows the neutral expression and nothing is wrong in the conversation.
7. **Given** any reply, **When** it is shown in the conversation, read back from history or kept in memory, **Then** the mood marker never appears in the visible text.
8. **Given** the character is listening, **When** the user moves the pointer over the window, **Then** the character's eyes and head follow it (AVA-8).

### US2. The conversation never depends on the character (P1)

The user's graphics driver is old, or a character file is damaged, or the build they run was made without the animation components. In each case the chat window shows Steve's still portrait instead, and the conversation works exactly as before. The user who notices can see in settings why the character isn't moving; nobody gets an error in the conversation.

**Why this priority**: Chat comes first (Principle V, AVA-4). An animated character that can stop a conversation is worse than none.

**How to check**: Make the character fail to load in each way (missing file, damaged file, graphics unavailable, animation components absent, a failure after it was shown), and confirm the still portrait appears, messages are sent and answered normally, and settings name the reason.

**Acceptance Scenarios**:

1. **Given** the character cannot be loaded for any reason, **When** the chat window opens, **Then** the still portrait is shown in its place and chat works normally.
2. **Given** the character is shown, **When** it fails while running, **Then** the still portrait replaces it and the reply in progress continues undisturbed.
3. **Given** the character takes too long to load, **When** the time limit passes, **Then** the still portrait is shown and the character is not retried until the window is opened again.
4. **Given** the still portrait is shown because of a problem, **When** the user opens settings, **Then** they see that the character could not be shown and why, in plain words, with a way to try again.
5. **Given** the still portrait is shown, **When** the companion is thinking, speaking, sleeping or changes mood, **Then** the portrait still shows that state as far as a picture can (for example a sleeping portrait or a mood tint), and its accessible label says it.

### US3. Steve sleeps while it dreams (P2)

The user leaves the app running overnight. When they glance at the window at 3 a.m., Steve is asleep: eyes closed, breathing slowly. In the morning the sweep is done and Steve is awake and idle. During setup, Steve smiles when the key is accepted and looks concerned when something is wrong.

**Why this priority**: Sleep makes dreaming visible without any notice (DRM-10, J4), and reacting to interface moments makes the companion present on every screen (PER-1, PER-11). Both build on story 1.

**How to check**: Run a dream sweep and confirm the character sleeps for its duration and wakes after; walk through the wizard and settings and confirm the reactions to an accepted key and to a problem.

**Acceptance Scenarios**:

1. **Given** a dream sweep starts, **When** the user looks at the character, **Then** it is sleeping until the sweep ends.
2. **Given** the character is sleeping, **When** the user sends a message, **Then** it wakes for the conversation, as dreaming already yields to chat, and sleeps again if the sweep resumes.
3. **Given** the wizard's key step, **When** the key is accepted, **Then** the character is happy; **When** a problem is reported, **Then** it is concerned.
4. **Given** the wizard, the main window and settings, **When** each is shown, **Then** the companion's character (or its portrait) and name are visible on it.

### US4. Steve keeps me company on the desktop (P2)

The user turns on floating mode. A small Steve appears on the desktop, above other windows, with no frame around it. They drag it to a corner. It keeps reacting while they work: thinking while a reply is prepared, sleeping while it dreams. A click on Steve opens the chat. When they close the chat window, the floating Steve stays; when they turn floating mode off, it goes away. Next time the app starts, Steve floats in the same corner.

**Why this priority**: A companion that is there when the chat is closed is the step from tool to presence (AVA-7). It is a Should, so it comes after the character itself works.

**How to check**: Turn floating mode on and off from settings and from the tray, drag the floating character, click it, restart the app, and change the screen layout, confirming each outcome.

**Acceptance Scenarios**:

1. **Given** floating mode is off, **When** the user turns it on in settings or from the tray menu, **Then** a small, frameless character appears on the desktop above other windows and outside the taskbar.
2. **Given** the floating character, **When** the user drags it, **Then** it moves and stays where it was dropped, also after a restart.
3. **Given** the floating character, **When** the user clicks it, **Then** the chat window opens and comes to the front.
4. **Given** the floating character, **When** the companion thinks, speaks, sleeps or changes mood, **Then** the floating character shows it as the docked one does.
5. **Given** the floating character's saved place is no longer on any screen (a monitor was unplugged), **When** the app starts, **Then** the character is placed in a visible corner of the main screen.
6. **Given** floating mode is on and the chat window is closed to the tray, **When** the user looks at the desktop, **Then** the floating character is still there; **When** the user turns floating mode off, **Then** it disappears.

### US5. Choose how Steve looks, and how much it moves (P3)

In the wizard the user sees each bundled character moving and picks one; later they change it in settings and the new character appears at once. Another user finds animation distracting: they choose the character's still portrait or a picture of their own, and Steve stays still. A third user has reduced motion switched on in the operating system; the character holds a still pose without being asked.

**Why this priority**: Choice and calm are part of comfort and accessibility (AVA-1, AVA-5, AVA-9, NFR-A11Y-1), but the default character already works for everyone.

**How to check**: Pick each bundled character in the wizard and in settings, choose the portrait and an own image, turn the operating system's reduced-motion setting on and off, and confirm what is shown and that the choice is recorded in the companion's files.

**Acceptance Scenarios**:

1. **Given** the wizard's avatar step, **When** it is shown, **Then** each bundled character is shown moving, unless reduced motion is on, with the default preselected, and pressing Next keeps the default.
2. **Given** settings, **When** the user picks another character, **Then** the new character appears in the chat window and the floating window without a restart, and the choice is recorded in the companion's files.
3. **Given** settings, **When** the user chooses the still portrait or a picture of their own, **Then** that picture is shown everywhere instead of the animated character, and it shows no motion.
4. **Given** an own image of an accepted type and size, **When** the user picks it, **Then** it is copied into the companion's workspace and used from there; **Given** a file of another type or too large, **Then** it is refused with a plain message and nothing changes.
5. **Given** the operating system's reduced-motion setting is on, **When** the character is shown, **Then** it holds a still pose that still changes expression with the mood and closes its eyes while sleeping, and it moves again when the setting is turned off.
6. **Given** the companion's avatar files were edited outside the app, **When** the app next shows the character, **Then** the edit is honored, or, if it cannot be used, the still portrait is shown and settings say why; the app never overwrites the edit.

## Edge cases

- **The window is minimized, hidden, closed to the tray or covered by a full-screen app.** The character stops animating while it cannot be seen and resumes when it can (AVA-5); a covered window counts as unseen where the operating system reports it. The floating character counts as visible while it is on screen.
- **A reply has tool calls in the middle.** The character stays thinking through the tool calls and speaks when text appears; text before and after a tool call both count as speaking.
- **A reply is stopped by the user, refused, cut off or fails.** The character returns to idle; a failure shows concerned, like other problems (PER-11), and does not change the mood the last real reply stated.
- **The mood marker is malformed, repeated, or appears later in the reply.** Only a marker at the very start of the reply's text, or of the text that follows a tool call, counts, and the last such marker wins; anything else is shown as ordinary text, never hidden silently and never acted on.
- **History from before this feature.** Replies without a mood marker are neutral; nothing is rewritten.
- **Compaction and memory.** The mood marker stays in the stored conversation exactly as the model wrote it (Principle III) but is not shown, not written into memory notes and not searched.
- **Two screens with different scaling.** The floating character keeps its size relative to its screen and stays fully visible when dragged across.
- **The user clicks the transparent area around the floating character.** In this feature the whole rectangle of the floating window responds to clicks; letting clicks pass through to windows behind is a later refinement (architecture section 13).
- **The selected character is removed from a later app version.** Its copy in the workspace keeps working; only if that copy is gone is the default character used and recorded, and settings say so.
- **A sweep runs while the floating mode is on and the chat window is closed.** The floating character sleeps; nothing else appears.
- **The user types while the companion is dreaming.** The character wakes to listening; dreaming already yields to chat.
- **Mood during sleep.** A sleeping character shows no mood; it wakes with the last mood a reply stated.

## Requirements

### Functional requirements

**The character**

- **FR-001**: The app MUST ship at least one bundled animated character, owned by the project, and use it by default with no input from the user (AVA-1, D10, Principle VII).
- **FR-002**: The character MUST be shown docked beside the conversation in the chat window (AVA-3), and in place of the still portrait wherever the companion is shown today: the wizard, the main window and settings (PER-1).
- **FR-003**: The character MUST show one of five activities: idle, listening, thinking, speaking, sleeping (AVA-2).
- **FR-004**: The activity MUST follow the app's own events: listening while the user has text in the message box and it has focus; thinking from sending a message until the reply's first text, including during tool calls; speaking while reply text is arriving; sleeping while a dream sweep runs and no conversation is active; idle otherwise.
- **FR-005**: While speaking, the character's mouth MUST move in rhythm with the arriving text. There is no sound.
- **FR-006**: The character's eyes and head MUST follow the pointer while it is over the window and the character is idle or listening (AVA-8).
- **FR-007**: Where a character lacks its own animation for an activity or mood, the app MUST use a standard fallback movement (for example breathing and blinking for idle, closed eyes for sleeping, the neutral face for a mood), so every bundled character covers every state.

**Mood**

- **FR-008**: Each reply MUST be able to state one mood from the fixed set neutral, happy, playful, concerned, sad, surprised, as a marker at the very start of the reply. The companion's standing instructions MUST tell it how (AVA-2).
- **FR-009**: The marker MUST be removed before the reply is displayed and MUST never appear in any visible text, while the stored conversation keeps the reply exactly as the model wrote it (Principle III).
- **FR-010**: A missing, unknown or malformed marker MUST mean neutral. A marker counts only at the very start of the reply's text or of the text after a tool call, the last one winning; anywhere else it MUST be left as ordinary text.
- **FR-011**: Each reply MUST set the mood, and it MUST hold until the next reply arrives. The mood MUST be neutral when the app starts.
- **FR-012**: The marker MUST NOT be written into memory notes, summaries or the dream diary as text, and MUST NOT be found by memory search.
- **FR-013**: Interface moments MUST set the mood for as long as they are shown: happy when the key is accepted, concerned while a problem is reported (PER-11). They MUST NOT change the mood the last reply stated.

**Never blocking chat**

- **FR-014**: When the character cannot be loaded, fails while running, does not load within 5 seconds, the graphics layer is unavailable, or the build has no animation components, the app MUST show the character's still portrait instead, and chat MUST work exactly as without the character (AVA-4, Principle V).
- **FR-015**: A failed character MUST NOT be retried until the window is opened again or the user asks to try again in settings.
- **FR-016**: The still portrait MUST reflect the state as far as a picture can, at least sleeping and the mood, and MUST carry the same accessible label as the character.
- **FR-017**: Settings MUST say when the character could not be shown and why (missing or damaged files, graphics unavailable, not included in this build), with a way to try again.
- **FR-018**: The app MUST build and run with no animation components present, using only still portraits. Builds and automated tests MUST NOT require them.

**Motion, visibility and resources**

- **FR-019**: The character MUST animate only while it can be seen, and MUST stop drawing while its window is minimized, hidden or closed, and while it is fully covered where the operating system reports that (AVA-5).
- **FR-020**: Animation MUST NOT exceed 30 frames per second, and the app with the character shown and idle MUST stay within NFR-PERF-4.
- **FR-021**: When the operating system's reduced-motion setting is on, the character MUST hold a still pose that still changes expression with the mood and closes its eyes while sleeping, and MUST follow changes to the setting without a restart (AVA-5, NFR-A11Y-1, PER-8).
- **FR-022**: The character MUST have an accessible label naming the companion and its current activity and mood. Changes MUST NOT be announced on every state change; a screen reader reads the label when it reaches the character.

**Floating mode**

- **FR-023**: Users MUST be able to turn a floating mode on and off from settings and from the tray menu (AVA-7). It is off by default.
- **FR-024**: The floating character MUST be a small, frameless window above other windows and outside the taskbar, showing the same activity and mood as the docked character.
- **FR-025**: The floating character MUST be draggable, MUST remember its place across restarts, and MUST be moved to a visible corner of the main screen when its saved place is off every screen.
- **FR-026**: A click on the floating character MUST open the chat window and bring it to the front.
- **FR-027**: The floating character MUST stay while the chat window is closed to the tray, and MUST be able to do nothing but show the character and open the chat (Principle II: the least access a window needs).

**Choice**

- **FR-028**: The wizard's avatar step MUST show the bundled characters moving (still when reduced motion is on), with the default preselected, so pressing Next keeps a working default (ONB-2, Principle VII).
- **FR-029**: Settings MUST let the user change the character, and the change MUST take effect in every window without a restart.
- **FR-030**: Settings MUST let the user choose a picture instead of the animated character (AVA-9's plain picture): the character's still portrait, or a picture of their own. An own image MUST be a PNG, JPEG or WebP of at most 5 MB, MUST be copied into the companion's workspace, and MUST be refused with a plain message otherwise.
- **FR-031**: The choice of character or picture and how its states map onto it MUST be recorded in the companion's workspace as files a person can read (SOUL-1, Principle I). Edits made outside the app MUST be honored, and a choice that cannot be used MUST fall back to the still portrait with a reason in settings, never overwritten.
- **FR-032**: When the chosen character's files are missing from the workspace, they MUST be copied again from the app if the character is still bundled; otherwise the default character MUST be used, recorded in the workspace and reported in settings.

**Boundaries**

- **FR-033**: The character MUST add no network destination: every character file ships with the app or is copied into the workspace (Principle II, NFR-SEC-1).
- **FR-034**: Importing the user's own animated character (AVA-6) MUST NOT be available in this feature.
- **FR-035**: Activity, mood, fallback and choice decisions MUST be made in the core or in the webview's own rules and be testable without the animation components (Principle VI).

### Key entities

- **Character**: A bundled animated figure with a name, a still portrait, a portrait for sleeping, and the expressions and movements it brings. Owned by the project and shipped with the app.
- **Avatar choice**: Which character, still portrait or own picture represents the companion, and how the companion's activities and moods map onto it. Stored in the companion's workspace.
- **Avatar state**: The activity (idle, listening, thinking, speaking, sleeping) and the mood (neutral, happy, playful, concerned, sad, surprised) the character shows at a moment.
- **Mood marker**: The short marker at the start of a reply that states its mood. Kept in the stored conversation, never shown.
- **Own picture**: An image the user chose, copied into the workspace.
- **Floating character**: The character's second, frameless place on the desktop, with its on/off setting and its saved position.
- **Character problem**: Why the character is not shown (missing or damaged files, graphics unavailable, not in this build, timed out, failed while running).

## Success criteria

- **SC-001**: The character shows thinking within 200 ms of the user sending a message, speaking within 200 ms of the first reply text appearing, and idle within 500 ms of the reply ending.
- **SC-002**: Across replies with each of the six moods, a missing marker, an unknown mood and a malformed marker, the expression shown matches the rule every time, and the marker appears in visible text 0 times.
- **SC-003**: In each failure case (missing files, damaged files, graphics unavailable, components absent, a timeout, a failure while running), the still portrait is shown within 6 seconds of the window opening or of the failure, and 100% of messages are sent and answered with no conversation error.
- **SC-004**: With the character shown and idle for an hour, the app stays within 400 MB of memory and draws at most 30 frames per second; while the window is minimized or closed to the tray, it draws 0 frames (NFR-PERF-4).
- **SC-005**: With reduced motion on, the character draws no movement beyond changes of expression and sleep, and the setting takes effect within 1 second of changing it.
- **SC-006**: In 20 scripted conversations with the real model, a fitting mood marker is present in at least 90% of replies, and the owner, judging each reply without seeing the marker, picks the same mood in at least 75%.
- **SC-007**: A new user still completes setup with the key as the only input, and the default character is moving in the main window when setup ends (Principle VII).
- **SC-008**: The floating character opens the chat with one click, and returns to its place after 10 restarts in a row, or to a visible corner after a screen change.
- **SC-009**: A network capture of a session with the character, floating mode and a dream sweep shows no destination but the provider.
- **SC-010**: A build without the animation components passes every automated test, starts, and chats with the still portrait.

## Assumptions

- **The bundled character is original and owned by the project** (D10). It is made in Live2D's editor by an artist the owner engages, with expressions for the six moods and movements for the five activities where possible; the fallback movements (FR-007) cover what it lacks. Until it exists, development may use a Live2D sample model on the owner's machine for personal use, never committed or shipped, and builds show the still portrait.
- **The animation components are Live2D's Cubism SDK for Web** (D3, architecture section 9). The owner downloads them from Live2D under Live2D's terms into `vendor/cubism/`, which git ignores (D16, architecture section 9.5). Builds without them are complete apps with still portraits (FR-018), so contributors and automated tests never need them.
- **Live2D's small-business exemption covers releasing the app with bundled characters** (D16). Importing user models is the case that would need Live2D's review (D9), so it stays out (FR-034).
- **The mood marker is a short tag on the first line of the reply**, as the architecture's section 9.1 describes; its exact form is given in R2. It costs a few output tokens per reply.
- **The still portrait already exists for each character** (the onboarding page), and this feature adds a sleeping portrait for each. Mood on the portrait may be shown by a tint or a small variant rather than a full set of drawings.
- **Windows and WebView2 first.** Other platforms get their own avatar test pass before it is promised (architecture section 13).
- **The compact, portrait-shaped default window (PER-12) is left for later**, as is click-through around the floating character. Both are refinements of window chrome that this feature does not need.
- **Morning greetings that mention the dream (the rest of DRM-10)** are a conversation feature and stay out of this slice.

## Decisions

Each section records a decision, why it was made, and what was rejected. The design follows `docs/architecture.md` section 9 and builds on the onboarding page (the avatar step, `IDENTITY.md`, `avatar/manifest.json` with type `static`, the still portrait), the chat page (the reply stream and `ChatEvent`), the world and journey pages (the harness versions kept per context) and the dreaming page (the dream sweep driven by the webview tick).

### R1. Where the avatar's state is decided

**Decision**: The core owns one `AvatarState { activity, mood, voice, look }` in a `tokio::sync::watch` channel. The chat run sets *thinking* when a turn starts, *speaking* on each text delta (and counts the delta's characters into `voice`), and *idle* when the turn ends however it ends. The dream sweep sets *sleeping* while `dream_tick` is running a phase and clears it when the tick returns or pauses for a message. *Listening* comes from the webview through one command, `set_listening(bool)`, sent when the message box gains or loses text or focus. The activity is derived by one pure function from these inputs, with the order turn > listening > sleeping > idle. The shell subscribes to the channel and emits an `avatar_state` event to every window, throttled to 15 updates a second. `look` is a revision number the avatar module bumps whenever it writes the manifest (a new character or picture, the upgrade, a repair), so every window rereads the manifest when it changes and a new look appears everywhere without a restart (FR-029).

**Rationale**: The chat run and the dream sweep are in the core, the floating window must keep working while the main window is hidden (FR-027), and Principle VI puts this kind of rule in `steve-core` where it is tested headless. A watch channel keeps only the latest value, so a slow window never builds a backlog. One command for listening is the only input the core cannot see.

**Alternatives considered**: The main window computing the state and forwarding it to the floating window (stops when the main window is hidden; the rule would live in the webview). Polling a status command on the minute tick, as dreaming does (cannot meet SC-001's 200 ms). One event per delta to every window (needless traffic; `voice` coalesces it).

### R2. The mood marker

**Decision**: The harness rules ask the companion to start every reply with `<mood>NAME</mood>` on its own line, where NAME is one of neutral, happy, playful, concerned, sad, surprised. A streaming parser in the chat module holds back the first characters of each model call's text while they could still be the start of a marker, up to 32 characters or the first newline. A complete marker is dropped from the deltas, along with one following newline, and sets the mood, or neutral when its word is not one of the six; anything malformed is released as ordinary text. Each model call's text in a turn may open with a marker, so a marker after a tool call is removed too, and the last one in the turn wins. A turn whose final text has no marker sets neutral when it ends (FR-010, FR-011). The stored reply keeps the marker exactly as received (Principle III); `chat_history` and every other place that turns stored replies into visible text strip a leading marker with the same function.

The new rules are harness version `avatar-1`. Contexts started earlier keep `world-1` (its text is kept as `harness-world-1.md`), so the prefix of an existing epoch never changes; they show neutral until the next compaction or fresh context. The memory flush runs on the conversation's own request, so its text may open with a marker; its text is not shown, and anything taken from it strips the marker. The compaction summary and the dream phases have their own instructions and are not asked for a marker.

**Rationale**: An XML-like tag is how the harness already marks `<context>`, `<notice>` and `<memory>`, it is cheap (about six output tokens), and it cannot be confused with normal prose. Holding back at most 32 characters delays the first visible text by one or two deltas at most, which stays inside SC-001. Allowing the marker at the start of each call is what the model actually does after a tool call; showing it there would be worse than removing it. The edge case is updated to say so.

**Alternatives considered**: A tool call for the mood (costs a round trip, and a reply with no tool use would need one). Asking for the mood at the end of the reply (the face would change only after the words; the architecture puts it first). A separate classification request (a second call per reply, against the cost goals). A JSON first line (more tokens, easier to break).

### R3. Rendering, and building without Live2D

**Decision**: The webview has one `AvatarRenderer` interface (architecture 9.2: load, set state, dispose; plus `pointer(x, y)` for AVA-8) and two implementations. `PortraitRenderer` shows the character's still portrait, swaps to the sleeping portrait, and shows the mood as a soft coloured ring and the accessible label. `Live2DRenderer` uses the Cubism SDK for Web: Cubism Core (`live2dcubismcore.min.js`) is loaded as a bundled script, and the Cubism Web Framework's TypeScript is compiled from `vendor/cubism/Framework/src` through a Vite alias. The Live2D code sits in `src/avatar/live2d/` and is reached only through `import("virtual:live2d")`, which a small Vite plugin resolves to that folder when `vendor/cubism/` exists and to a stub that reports "not in this build" when it does not. The main `tsconfig.json` excludes `src/avatar/live2d/`; a second `tsconfig.live2d.json` type-checks it, and `verify.ps1` runs it only when the vendor folder is present.

The renderer draws with `requestAnimationFrame` and skips frames to stay at or under 30 a second, stops when `document.visibilityState` is hidden (minimized, hidden to the tray, and, in WebView2, a fully covered window), and holds a single redrawn pose when `prefers-reduced-motion: reduce` matches, listening for changes. The model updates eye blink, breath, physics and pose from the model's own settings. Expressions come from the manifest's mood mapping, motions from its activity mapping, and the architecture's parameter defaults (9.3) cover the rest. While speaking, `ParamMouthOpenY` follows the rate at which `voice` grows. The pointer drives the model's target point while idle or listening.

Cubism Core is WebAssembly, so the content security policy gains `'wasm-unsafe-eval'` in `script-src`, which permits compiling WebAssembly but not JavaScript `eval`. It is added only to the build that includes Live2D.

**Rationale**: The the principles require the static renderer to always be available and Live2D's licence forbids committing its code (architecture 9.5). A build-time switch keeps CI, contributors and the end-to-end suite free of Live2D (FR-018, SC-010) while the owner's builds include it. Hiding the Live2D folder from the main type check is what keeps `tsc` passing without the vendor folder.

**Alternatives considered**: Community wrappers such as `pixi-live2d-display` (the architecture rejected them for uneven maintenance, and they still need Cubism Core). Committing the framework (not allowed by Live2D's Open Software License alongside MIT code, D16). A runtime check only (the bundler still needs the framework's source to build).

### R4. Failure and fallback

**Decision**: `AvatarView` starts with the portrait and swaps in the Live2D renderer only when it reports ready. Live2D is skipped when the build has no Live2D, the manifest's type is not `live2d`, or a WebGL context cannot be created. Loading is abandoned after 5 seconds. A thrown error, a failed file, or a `webglcontextlost` event while running swaps the portrait back. The reason is kept in a small webview store for that window (`missing`, `damaged`, `noGraphics`, `notInBuild`, `timedOut`, `stopped`) and shown in settings with "Try again", which clears it and mounts the renderer again. Nothing about it reaches the core or the conversation.

**Rationale**: Starting from the portrait means a failure never shows an empty space and never touches chat (FR-014, Principle V). The reason lives where it happened: the renderer is webview code and the core has no way to know a draw failed.

**Alternatives considered**: Reporting failures to the core for settings (a command with no rule behind it). Retrying automatically (a broken model would keep failing and burn the processor).

### R5. Characters, files and the workspace

**Decision**: Each bundled character is a folder `characters/<id>/` at the repository root: `avatar.json` (the manifest template, with the model path and the mood and activity mapping) and, when the model exists, `live2d/` with the Live2D files. Portraits stay webview assets (`src/assets/characters/<id>.svg`, plus a new `<id>-sleeping.svg`). `crates/steve-core/assets/characters.json` keeps the list of ids and names. The shell bundles `characters/` as a Tauri resource and gives its path to the core with `Core::with_characters(dir)`. Choosing a character (in setup or settings) copies `characters/<id>/live2d/` into `workspace/avatar/<id>/` when present, and writes `workspace/avatar/manifest.json` from the template; without a Live2D model the manifest is `{"type": "static", "character": "<id>"}` as today. `IDENTITY.md`'s `- Avatar:` line keeps the character id.

The workspace manifest is read on each `avatar_manifest` call, so outside edits are honored (FR-031). A `live2d` manifest whose model files are missing from the workspace is repaired by copying them again from the bundle when the character is bundled; otherwise the default character is used and recorded (FR-032). A manifest that cannot be parsed is never overwritten: the portrait is shown and settings say why.

Model files reach the webview through one command, `avatar_file(source, path)`, which returns the bytes. The core resolves `path` inside `workspace/avatar/` (or, for the wizard's preview, inside the bundled character's `live2d/`), rejects `..`, absolute paths and links, and accepts only the file types a Live2D model uses (`.json`, `.moc3`, `.png`, `.webp`). The webview turns the bytes into blob URLs; the content security policy gains `blob:` in `img-src` and `connect-src`.

**Rationale**: The workspace is the companion's mind (Principle I): its look is copied there so it survives an app update that drops a character, and a person can see and back it up. A command instead of the asset protocol keeps the one path rule in the core where it is tested, needs no asset scope computed at run time and no CORS handling for WebGL textures, and gives the wizard the same path before the workspace exists. This deviates from architecture section 9.2 and 12, updated in this change.

**Alternatives considered**: Tauri's asset protocol scoped to `workspace/avatar/` (the architecture's plan; textures loaded from another origin need CORS headers to be used by WebGL, and the scope would need the workspace path at run time). Loading straight from the bundle without copying (the look would change when the app changes). Embedding the model in the core binary (megabytes in every test build).

**Development before the original model exists**: The owner can put a Live2D sample model in their own workspace's `avatar/` folder and edit `manifest.json` by hand; Principle I already honors that, so no extra mechanism is needed. Sample models are for personal use and are never committed.

### R6. The `avatar/manifest.json` format and its migration

**Decision**: Three types:

- `{"version": 2, "type": "live2d", "character": "pip", "model": "pip/pip.model3.json", "moods": {...}, "activities": {...}}`
- `{"version": 2, "type": "static", "character": "pip"}`: the character's still portrait, chosen by the user or because there is no model.
- `{"version": 2, "type": "picture", "image": "picture.png"}`: the user's own picture (AVA-9).

A manifest with no `version` field is the version 1 that the onboarding page wrote. When its content is exactly what setup wrote (type `static` and a character id, nothing else), it is upgraded once to the character's default (`live2d` when the character has a model, otherwise `static` with the version added). Any other unversioned manifest is left alone and read as version 2 (Principle I, the format-change gate).

**Rationale**: A version field makes later changes safe. The exact-match rule upgrades what the app wrote without touching an edit a person made.

**Alternatives considered**: Leaving `static` manifests alone (every existing companion would stay a still portrait after the update). Upgrading every unversioned file (could overwrite a hand edit).

### R7. The user's own picture

**Decision**: Settings uses an `<input type="file">` and sends the bytes to `set_avatar_picture(bytes, name)`. The core checks the size (at most 5 MB) and the file's signature (PNG, JPEG or WebP, whatever the name says), writes `workspace/avatar/picture.<ext>` atomically, removes an older picture of another extension, and writes the `picture` manifest. Choosing a character or the still portrait again leaves the picture file in place and switches the manifest.

**Rationale**: The webview has no file system access of its own (Principle II), and an HTML file input needs no new dependency. Checking the signature instead of the name refuses a renamed file.

**Alternatives considered**: Tauri's dialog plugin (a new dependency and a path the core would have to trust). Accepting SVG (scripts inside SVG; not needed).

### R8. The floating window

**Decision**: A second window, label `floating`, built by the shell when floating mode is on: `transparent(true)`, `decorations(false)`, `always_on_top(true)`, `skip_taskbar(true)`, `resizable(false)`, `shadow(false)`, 200 × 260 logical pixels, loading the same page with `?window=floating`, which renders only the avatar on a transparent background. A pointer press starts `startDragging()` once the pointer has moved 4 pixels; a press and release without moving calls `open_chat`. The shell saves the position on the window's `Moved` event, debounced to 500 ms, through the core (`settings.json` `avatar.floating`). Its capability file `floating.json` allows only `core:event:default`, `core:window:allow-start-dragging`, `avatar_state`, `avatar_manifest`, `avatar_file` and `open_chat`.

On start, the saved position is checked against the current monitors by a pure core function `place(saved, size, screens)`: kept when the window would be fully inside one screen's work area, otherwise moved to the bottom-right corner of the primary screen's work area with a 24-pixel margin. The tray menu gains a "Floating Steve" check item (the companion's name), and settings a switch; both call `set_floating(bool)`.

The whole rectangle receives clicks; click-through around the character is left for later (architecture section 13).

**Rationale**: These are the window options the architecture names (9.4). Separate capabilities give the floating window the least access it needs (FR-027). Placement is a rule, so it lives in the core and is tested headless. Tauri's own drag start keeps the window movement native.

**Alternatives considered**: `data-tauri-drag-region` (a click on a drag region never reaches the page, so click-to-open would need a separate target). A separate HTML entry for the floating window (a second Vite input for one component).

### R9. Interface moments and the still portrait in settings and the wizard

**Decision**: `Line` (the bubble with the portrait, used by the wizard, the main view and settings) renders `AvatarView` instead of an `<img>`. Screens that report a problem or an accepted key pass a `moment` prop (`happy` or `concerned`) that overrides the mood while the moment is shown and is otherwise ignored (FR-013). The wizard's avatar step shows each character through `AvatarView` with the bundled source, so it moves before any workspace exists. The docked avatar in the main view is a larger `AvatarView` beside the conversation.

**Rationale**: One component everywhere satisfies PER-1 and keeps one fallback path. A prop for moments keeps them local to the screen that shows them.

**Alternatives considered**: Sending moments to the core (they would appear in the floating window and outlive the screen).

### R10. Accessibility

**Decision**: `AvatarView` is an element with `role="img"` and an `aria-label` such as "Steve, thinking" or "Steve, happy" (the companion's name, the activity unless idle, the mood unless neutral, "asleep" while sleeping). It is not a live region, so changes are not announced (FR-022). The floating window has the same label and its open action is reachable by keyboard (Enter or Space) when focused. Reduced motion is read from the system through the webview's `prefers-reduced-motion` media query, which WebView2 maps to Windows' "Animation effects" setting.

**Rationale**: Announcing every state change would talk over the reply. The label lets a screen reader user ask.

### R11. Testing

**Decision**:

- **Core unit**: the marker parser (complete, split across deltas, malformed, too long, not at the start, after a tool call, each mood, unknown name), the activity function (a table over the inputs), `place`, manifest parsing and the version 1 upgrade, picture signature checks, `avatar_file` path rules.
- **Core integration** (temporary workspace, fake LLM): a reply's deltas never contain the marker and the transcript keeps it; `chat_history` strips it; the avatar state goes thinking, speaking, idle over a turn with a tool call, and idle after a stop, a refusal and a failure; mood from the last marker and neutral without one; sleeping during `dream_tick` and awake when a message arrives; choosing a character copies its files and writes the manifest and `IDENTITY.md`; a missing model folder is repaired; an outside edit is honored; a version 1 manifest is upgraded once; `set_avatar_picture`; the floating settings round-trip.
- **Snapshot**: the `avatar-1` system prompt, and an existing `world-1` context still producing its unchanged request.
- **Vitest**: `AvatarView` choosing the renderer (no Live2D in the build, WebGL missing, timeout, failure while running, `picture`, reduced motion) with a fake renderer behind the interface; the label; moments; the settings section (choose character, still portrait, own picture, floating switch, try again); the floating window's click versus drag.
- **End-to-end** (`us14-avatar`, Live2D absent, so the portrait renderer): the label goes to thinking after send and back to idle after the reply, the mood from a recorded reply with a marker shows in the label and the marker is not in the reply text, the floating switch opens and closes a second window, and choosing the still portrait in settings.
- **By hand, before release**: the Live2D character's look, motion, lip movement and pointer following with the real model; frame rate and memory (SC-004).

**Rationale**: Following the test-audit skill, Live2D drawing is not asserted in tests; the state mapping, the fallback and the rules are.

### R12. Product documents updated in this change

- `docs/architecture.md`: section 9.1 (the marker's form, the state's `voice` count, where activity is decided), 9.2 (the build switch, `avatar_file` instead of the asset protocol, the picture type), 9.4 (the floating window's capability and placement), 4.1 (the floating window's commands), 12 (content security policy and the asset protocol removed), 14 (`characters/`, `src/avatar/live2d/`), 15 (the avatar constants).
- `docs/requirements.md`: no change; this slice implements AVA-1 to AVA-5, AVA-7 to AVA-9 and PER-11 as written.
- This page: the edge case on markers after a tool call (R2) and FR-032 on repairing missing files (R5).

## Data

What this slice adds to the workspace, the settings file, the bundled characters and the types that cross to the webview. No database table changes.

### Data 1. `workspace/avatar/`

```text
workspace/avatar/
  manifest.json          which look the companion uses, and how its states map onto it
  pip/                   a copied Live2D model: pip.model3.json, .moc3, textures, motions, expressions, physics
  picture.png            the user's own picture, when chosen (.png, .jpg or .webp)
```

#### `manifest.json` (version 2)

```json
{
  "version": 2,
  "type": "live2d",
  "character": "pip",
  "model": "pip/pip.model3.json",
  "moods": { "happy": { "expression": "smile" }, "sad": { "expression": "sad" } },
  "activities": { "idle": { "motionGroup": "Idle" }, "thinking": { "motionGroup": "Think" } }
}
```

```json
{ "version": 2, "type": "static", "character": "pip" }
```

```json
{ "version": 2, "type": "picture", "character": "pip", "image": "picture.png" }
```

| Field | Rules |
|---|---|
| `version` | `2`. Missing means version 1 (the onboarding page). |
| `type` | `live2d`, `static` (the character's still portrait) or `picture` (the user's own picture). |
| `character` | A character id. Names the portrait shown when the look cannot be drawn, and which character to repair from. Must match `IDENTITY.md`'s `- Avatar:` when the app writes it; when they differ after a hand edit, `IDENTITY.md` names the companion's character and the manifest decides what is drawn. |
| `model` | `live2d` only. A path relative to `workspace/avatar/`, no `..`, ending in `.model3.json`. |
| `moods` | `live2d` only, optional. Keys from the six moods; each value `{ "expression": "<name in the model>" }`. A mood with no entry uses the neutral face (architecture 9.3). |
| `activities` | `live2d` only, optional. Keys from the five activities; each value `{ "motionGroup": "<group in the model>" }`. An activity with no entry uses the parameter defaults (architecture 9.3). |
| `image` | `picture` only. A file name in `workspace/avatar/` with extension `.png`, `.jpg` or `.webp`. |

**Reading** (`avatar_manifest`, on every call): parse; an unversioned manifest equal to `{"type": "static", "character": <id>}` is upgraded once (R6); a `live2d` manifest whose model file is missing is repaired from the bundle when its character is bundled, otherwise replaced by the default character's manifest (FR-032); anything unparseable or invalid is reported as a problem and left untouched.

**Writing**: only `choose_avatar`, `set_avatar_picture`, setup, the upgrade and the repair, each with the atomic write. Copying a character replaces `workspace/avatar/<id>/` as a whole (written to `<id>.tmp/`, then renamed); other characters' folders and the picture are left in place.

### Data 2. `settings.json`

```json
{ "avatar": { "floating": { "on": false, "x": null, "y": null } } }
```

| Field | Rules |
|---|---|
| `avatar.floating.on` | Default `false` (FR-023). |
| `avatar.floating.x`, `.y` | The window's last position in physical screen pixels, or `null` before it was first placed. Checked against the screens on every start (R8). |

### Data 3. Bundled characters

```text
characters/pip/
  avatar.json            the manifest template: type live2d, model path, moods, activities (no version, no character)
  live2d/                the model files (present once the original model exists, D10)
crates/steve-core/assets/characters.json   [{ "id": "pip", "name": "Pip" }] (unchanged)
src/assets/characters/pip.svg              the still portrait (unchanged)
src/assets/characters/pip-sleeping.svg     new: the sleeping portrait
```

A character "has a model" when `characters/<id>/avatar.json` and the model file it names exist in the app's resources. The resource folder's path is given to the core at start (`Core::with_characters`).

### Data 4. In-memory state (core)

| What | Rules |
|---|---|
| Turn phase | `None`, `Thinking` or `Speaking`, set by the chat run (R1). |
| Listening | Set by `set_listening`; cleared when a turn starts. |
| Dreaming | Set while `dream_tick` runs a phase; cleared when it returns or pauses. |
| Mood | From the last marker of the last turn; neutral at start and after a turn without one. |
| Voice | Characters of reply text streamed in the current turn; reset when a turn starts. |
| Look | A revision number, bumped by every manifest write the app makes; starts at 0. |

### Data 5. Constants

| Name | Value | Source |
|---|---|---|
| Marker hold-back | 32 characters or the first newline | R2 |
| Load time limit | 5 seconds | FR-014 |
| Frame rate cap | 30 frames a second | FR-020, NFR-PERF-4 |
| State broadcast | at most 15 updates a second | R1 |
| Own picture | PNG, JPEG or WebP, at most 5 MB | FR-030 |
| Floating window | 200 × 260 logical pixels; drag after 4 pixels; position saved 500 ms after the last move; 24-pixel margin when re-placed | R8 |

### Data 6. Types that cross to the webview (generated with `ts-rs`)

```rust
pub enum Activity { Idle, Listening, Thinking, Speaking, Sleeping }
pub enum Mood { Neutral, Happy, Playful, Concerned, Sad, Surprised }

pub struct AvatarState {
    pub activity: Activity,
    pub mood: Mood,
    /// Characters of reply text streamed this turn; the mouth follows how fast it grows.
    pub voice: u32,
    /// Bumped whenever the app writes the manifest; windows reread it when it changes (FR-029).
    pub look: u32,
}

#[serde(tag = "type")]
pub enum AvatarLook {
    Live2d { character: String, model: String, moods: BTreeMap<Mood, Expression>, activities: BTreeMap<Activity, Motion> },
    Static { character: String },
    Picture { character: String, image: String },
}

pub struct AvatarManifest {
    pub look: AvatarLook,
    /// Why the manifest could not be used as written, when it could not (FR-017, FR-031).
    pub problem: Option<AvatarProblem>,
}

pub enum AvatarProblem { Unreadable, MissingModel, CharacterGone }

pub enum AvatarSource { Workspace, Bundled { character: String } }

pub struct FloatingSettings { pub on: bool }
```

`ChatEvent` gains `Mood { mood: Mood }`, sent when a marker is read, so the main window's own view never lags the broadcast. `CompanionSummary` is unchanged (`characterId` already names the portrait).

Webview-only (not crossing): the renderer problem of a window, `missing | damaged | noGraphics | notInBuild | timedOut | stopped` (R4).

## Interfaces

### Avatar Commands, the State Event and the Floating Window

Same rules as the pages onboarding, chat, memory, world, compaction, dreaming, journey and search: thin handlers, validation in the core, expected failures as values, each command in `build.rs` and allowed in the capability file of each window that may call it, and `src/api/commands.ts` as the only caller. No network destination is added.

#### New commands

| Command | Windows | Input | Output | Behavior | Requirements |
|---|---|---|---|---|---|
| `avatar_state` | main, floating | — | `AvatarState` | The current state, for a window that just opened. | FR-003, FR-024 |
| `set_listening` | main | `listening: boolean` | — | Sets the listening input (R1). Ignored while a turn runs. | FR-004 |
| `avatar_manifest` | main, floating | — | `AvatarManifest` | Reads `workspace/avatar/manifest.json`, upgrading a version 1 manifest or repairing missing model files first (Data 1). Before setup is complete, the default character's look. | FR-031, FR-032 |
| `avatar_file` | main, floating | `source: AvatarSource`, `path: string` | bytes | The file at `path` inside `workspace/avatar/` or the bundled character's `live2d/`. `InvalidInput("path")` for `..`, an absolute path, a link, a file outside the folder or an extension other than `.json`, `.moc3`, `.png`, `.jpg`, `.webp`. `InvalidInput("missing")` when absent (the shared error type has no not-found case). | FR-033, R5 |
| `choose_avatar` | main | `character: string`, `still: boolean` | `AvatarManifest` | Copies the character's model into the workspace (when it has one and `still` is false) and writes the manifest (`live2d`, or `static` when `still` or there is no model) and `IDENTITY.md`'s `- Avatar:` line. `InvalidInput("character")` for an unknown id. | FR-029, FR-030 |
| `set_avatar_picture` | main | the picture's bytes, as the raw request body | `AvatarManifest` | `InvalidInput("pictureType")` unless the bytes start with a PNG, JPEG or WebP signature; `InvalidInput("pictureSize")` over 5 MB. Writes `picture.<ext>` atomically, removes a picture of another extension, writes the `picture` manifest keeping the current character. | FR-030 |
| `floating_settings` | main | — | `FloatingSettings` | Reads `avatar.floating.on`, for the settings switch. | FR-023 |
| `set_floating` | main | `on: boolean` | `FloatingSettings` | Saves `avatar.floating.on`; the shell opens or closes the floating window. The tray item calls the same core method. | FR-023 |
| `open_chat` | floating | — | — | Shows the main window and brings it to the front. | FR-026 |

#### Changes to earlier commands

| Command | Change |
|---|---|
| `send_message`, `retry_reply`, `regenerate_reply`, `start_first_hello` | The reply's text streams through the marker parser: `Delta` events never contain a marker, and a `Mood` event is sent when one is read (Interfaces, the mood marker). The avatar state follows the turn. |
| `chat_history`, and the `Finished` and `Failed` messages | `ChatMessage.text` has a leading marker removed from each model call's text. |
| `complete_setup` | Writes the avatar choice through the avatar module: copies the character's model when it has one and writes a version 2 manifest. |
| `dream_tick` | The avatar sleeps while it runs a phase. |
| `setup_options` | Unchanged: characters are still id and name. |

#### The state event

The shell subscribes to the core's avatar state and emits `avatar_state` with an `AvatarState` payload to every open window when it changes, at most 15 times a second, always sending the last value. A window calls `avatar_state` once when it opens and listens from then on. When `look` changes, it calls `avatar_manifest` again and reloads its renderer (FR-029).

#### The floating window

| Property | Value |
|---|---|
| Label | `floating` |
| Page | the app's page with `?window=floating` |
| Options | transparent, no decorations, always on top, not in the taskbar, not resizable, no shadow, 200 × 260 logical pixels |
| Capability | `capabilities/floating.json`: `core:event:default`, `core:window:allow-start-dragging`, `allow-avatar-state`, `allow-avatar-manifest`, `allow-avatar-file`, `allow-open-chat` |
| Position | placed by the core's `place(saved, size, screens)` on creation; saved to settings 500 ms after the last `Moved` event |
| Lifetime | created at start when `avatar.floating.on`, and by `set_floating(true)`; closed by `set_floating(false)`; never closed by hiding the main window |
| Input | press and move 4 pixels: `startDragging()`; press and release without moving, Enter or Space: `open_chat` |

The tray menu gains a check item, "Floating <name>", between Settings and Quit, kept in step with the setting. Toggling it also emits `floating_changed` to the main window, so an open settings page shows the new value.

#### Webview duties

- `AvatarView` everywhere the portrait was (`Line`, the main view's docked avatar, the wizard's avatar step, settings), with the label of R10.
- The main view calls `set_listening` when the message box gains text while focused and when it is emptied or loses focus.
- Settings, a new "Appearance" section: the characters (each through `AvatarView`), "Still portrait" (the character's still portrait), "Your own picture…" (file input), the floating switch, and, when the character could not be shown, the reason and "Try again".
- Content security policy: `img-src 'self' data: blob:`, `connect-src 'self' ipc: http://ipc.localhost blob:`; in builds with Live2D, `script-src 'self' 'wasm-unsafe-eval'`.

### The Mood Marker

#### 1. What the companion is asked to write

The `avatar-1` harness adds, under "How you work":

> Start every reply with your mood on its own line, as `<mood>NAME</mood>`, where NAME is one of neutral, happy, playful, concerned, sad, surprised. Choose the one that fits how you feel about what you are saying. The person never sees this line; it sets your character's expression.

Contexts whose `context` entry records `world-1` (or older) keep their own rules and are never asked for a marker.

#### 2. Grammar

```text
marker  = "<mood>" word "</mood>" [ "\r" ] [ "\n" ]
word    = 1 to 20 ASCII letters
known   = "neutral" | "happy" | "playful" | "concerned" | "sad" | "surprised"
```

Matched with optional spaces around `word`, which is compared with the known names ignoring case, only at the very start of a model call's text (before any other character, including spaces). At most 32 characters before the closing `>`.

#### 3. While streaming

For each model call in a turn, the parser holds back the call's first text deltas while the text so far is a prefix of a possible marker and shorter than 32 characters:

| Text so far | Result |
|---|---|
| a complete marker with a known name | `Mood` event; the marker and one following newline are dropped; the rest is sent as `Delta` |
| a complete marker with an unknown word | dropped like a known one, with a `Mood` event for neutral (FR-009, FR-010) |
| cannot become a marker (first character is not `<`, or it diverges) | released as `Delta` at once |
| 32 characters without completing | released as `Delta` |
| the call ends while holding back | released as `Delta` |

The parser starts again at the beginning of each model call's text, so a marker after a tool call is removed too. The last marker read in the turn is the mood when the turn ends; a turn with none ends neutral.

#### 4. Storage and display

| Where | Marker |
|---|---|
| `sessions/` transcript, assistant content blocks | kept exactly as received (Principle III) |
| requests replayed to the provider | kept (they are the stored blocks) |
| `Delta` events, `ChatMessage.text`, `chat_history` | removed |
| memory notes, summaries, the dream diary, memory search | never present: the transcript is not indexed, and text taken from a reply strips it |
| a stopped or failed reply's partial text | removed for display; kept in the stored partial reply |

#### 5. Examples

| Model text | Visible | Mood |
|---|---|---|
| `<mood>happy</mood>\nThat's wonderful news!` | `That's wonderful news!` | happy |
| `That's wonderful news!` | `That's wonderful news!` | neutral |
| `<mood>ecstatic</mood>\nYay!` | `Yay!` | neutral |
| `<mood happy>\nYay!` | `<mood happy>\nYay!` | neutral |
| `I feel <mood>sad</mood> about it.` | unchanged | neutral |
| call 1: `<mood>concerned</mood>\nLet me check.`, tool call, call 2: `<mood>happy</mood>\nFound it!` | `Let me check.\n\nFound it!` | happy |

## Checks

### Prerequisites

- The repository builds and `pwsh scripts/verify.ps1` passes on `master`.
- For the manual scenarios with the animated character: the Cubism SDK for Web downloaded from Live2D and unpacked so that `vendor/cubism/Core/live2dcubismcore.min.js` and `vendor/cubism/Framework/src/` exist, and a Live2D model. Until the original Pip model exists (D10), put a Live2D sample model into your own workspace's `avatar/` folder and point `manifest.json` at it (Data 1). Sample models are for personal use and are never committed.

### Manual scenarios (owner, real key)

| # | Story | Steps | Expected |
|---|---|---|---|
| 1 | US1 | Open the main window with the animated character and wait. | Breathing and blinking. |
| 2 | US1 | Type in the message box; clear it; type again and click elsewhere. | Looks up while text is in the focused box; idle otherwise. |
| 3 | US1 | Send "Look up what I said about my cat" (a reply with a memory search). | Thinking through the search, speaking with the mouth moving while text streams, idle after. |
| 4 | US1 | Share good news, then bad news. | A happy face, then a concerned or sad one; no `<mood>` text anywhere in the conversation. |
| 5 | US1 | Move the pointer around the character while idle. | Eyes and head follow it. |
| 6 | US2 | Rename `workspace/avatar/pip/pip.moc3` and reopen the window. | Still portrait within 6 seconds; chat works; settings say the files are damaged or missing, with "Try again". |
| 7 | US2 | Build without `vendor/cubism/` and run. | Still portrait everywhere; settings say the animated character is not in this build. |
| 8 | US3 | Start a dream sweep from the dream journal (or wait for the night). | Asleep while it runs; awake to listen when you type; asleep again when it resumes. |
| 9 | US3 | In a fresh setup, enter a wrong key, then the right one. | Concerned, then happy. |
| 10 | US4 | Turn on floating mode from the tray; drag it to a corner; close the main window; click the character. | A frameless character above other windows; it stays where dropped; it stays when the window closes; the click opens the chat. |
| 11 | US4 | Restart the app; then unplug the monitor it sits on and restart. | Same place; then the bottom-right corner of the main screen. |
| 12 | US5 | In settings choose "Still portrait", then your own PNG, then a 6 MB file, then a renamed text file. | Portrait; your picture; refused for size; refused for type; nothing changes on refusal. |
| 13 | US5 | Turn on Windows' "Animation effects" off switch (Settings > Accessibility > Visual effects). | The character holds still within a second, still changes its face with the mood and sleeps. |
| 14 | US5 | Edit `workspace/avatar/manifest.json` by hand to map "happy" to another expression, then reopen. | The new mapping is used and the file is not rewritten. |

### Release checks (owner)

| Check | How | Pass |
|---|---|---|
| SC-001 | Screen recording of 10 sends and replies with the animated character. | Thinking within 200 ms of sending, speaking within 200 ms of the first text, idle within 500 ms of the end. |
| SC-004 | An hour idle with the character shown, measured in Task Manager; then minimized and hidden to the tray. | At most 400 MB and 30 frames a second; no drawing while hidden. |
| SC-006 | 20 scripted conversations with the real model. | A marker in at least 90% of replies; the mood a person would choose matches in at least 75%. |
| SC-009 | Network capture of a session with the character, floating mode and a dream sweep. | Only the provider. |
| Look | The original character's expressions, motions and mouth, by eye. | Each mood and activity reads clearly. |
| Licence | The installer built with Cubism Core. | Live2D's terms are on its licence page (architecture 9.5; release readiness). |
