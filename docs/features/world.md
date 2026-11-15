# World

Built.

World Building is delivered in two parts (`docs/requirements.md` section 6.12). This page is the first, delivery slice 9: the world exists, belongs to one companion, and grows without any model call. It covers WLD-1 to WLD-8. The second part, slice 10, arrives with dreaming and lets the companion draw things that come from what was talked about (WLD-9, WLD-10).

## User stories

### US1. A world of its own (P1)

The user finishes setup and meets Steve. From the main window they open Steve's world. It is almost empty: a small home in the middle of a blank canvas, in colours that belong to this Steve. The view shows Steve's id and the day Steve was born. A friend who installs the app gets a different id, different colours and a different home. Someone who already had a companion before this feature opens the app after the update and finds that their companion now has an id, a birth date and a home too, without doing anything.

**Why this priority**: The id, the birth date and the world file are what everything else in World Building builds on. On their own they already give each companion something that is visibly its own.

**How to check:** Finish setup in two fresh workspaces, open the world in each, and confirm that each shows a different id, a home, and a different look; relaunch and confirm each world is unchanged. Launch the app on a workspace made before this feature and confirm the companion gets an id, a birth date and a home with no prompt.

**Acceptance Scenarios**:

1. **Given** setup has just finished, **When** the user opens the world from the main window, **Then** they see a blank canvas with the companion's home at the centre, the companion's id, and its birth date.
2. **Given** two companions with different ids, **When** their worlds are compared, **Then** they differ in colours, and as they grow, in where water, meadow, woods and rock lie and in which items appear most.
3. **Given** a companion created before this feature, **When** the app starts, **Then** the companion gets an id, a birth date and a home without any prompt, and the conversation under way is not interrupted.
4. **Given** a companion with a world, **When** the app is closed and opened again, **Then** the world, the id and the birth date are exactly as before.
5. **Given** the world view is open, **When** the user goes back to the conversation, **Then** the conversation is as they left it, including text they had typed but not sent.

---

### US2. The world grows a little on days you talk (P1)

The user chats with Steve in the evening. When they next open the world, a few new things are there: a tuft of grass, two flowers, a pebble near the home. The same happens on other days they talk. Whether they send one message or two hundred on a day, the world gains the same small handful. Then they are away for three weeks. When they come back, the world is exactly as they left it. Nothing has withered, nothing tells them how long they were gone, and the day they return adds the same small handful as any other day.

**Why this priority**: Growth over time is the feature. The rules around it (a small daily amount, no decay, no pressure) are what keep it compatible with the product's promise not to use engagement tricks (NFR-SEC-6, PER-8).

**How to check:** With simulated dates, send a message on each of several days and confirm the world gains a small number of items once per day; send many messages on one day and confirm the count stays within the daily limit; skip 60 days and confirm the world file is unchanged, no notice about the gap appears anywhere, and the next day with a message adds an ordinary day's items.

**Acceptance Scenarios**:

1. **Given** a day on which the user has sent no message yet, **When** they send one, **Then** the world gains a small number of new items for that day, placed near what is already there.
2. **Given** the day's items have been drawn, **When** the user sends more messages that day, **Then** no further items are drawn.
3. **Given** days on which the user sent no message, **When** the user next opens the app, **Then** the world is unchanged and no items are added for the days missed.
4. **Given** any length of absence, **When** the user returns, **Then** no item has been removed, faded or changed, and nothing in the app mentions the absence, a streak, or days missed.
5. **Given** the user sends a message, **When** the day's items are being decided and saved, **Then** the message is sent and the reply streams with no added delay.
6. **Given** saving the day's items fails (for example the disk is full), **When** the reply is shown, **Then** the conversation continues normally with no error about the world, and the app tries again at the next message that day.

---

### US3. The world evolves (P2)

In the first days Steve draws only small things: grass, flowers, pebbles. After a couple of weeks, bushes, trees and rocks start to appear, and a pond forms where this Steve's land is wet. After a couple of months there are built things too: a fence by the home, a lantern, a small boat on the water. Nothing drawn earlier is replaced. The world gets richer as it gets bigger.

**Why this priority**: Stages are what make the world feel like it is growing up and not only getting larger. They depend on stories 1 and 2 and can be tuned after those work.

**How to check:** Simulate growth day by day and confirm that only first-stage items appear at the start, that later-stage items begin to appear once the world has passed each stage's size, and that every earlier item is still in place.

**Acceptance Scenarios**:

1. **Given** a new world, **When** it grows, **Then** only items from the first stage are drawn.
2. **Given** a world that has passed a stage's size, **When** it grows, **Then** items from that stage can be drawn, alongside items from earlier stages.
3. **Given** a world at any stage, **When** it grows, **Then** each item is drawn where it belongs: water items where the land is wet, trees in the woods, built things near the home.
4. **Given** a world at any stage, **When** the user opens it, **Then** no countdown or progress bar towards the next stage is shown, and any mention of a stage states only what has already happened (for example that trees have started to appear).

---

### US4. Look around and read the world (P2)

The user opens the world after a few months. The whole world fits the window at first. They zoom in on the pond and drag to look around. They select the lantern and see its name and the day it was drawn. They switch to a list, which shows everything Steve has drawn, newest first, grouped by day. A user who relies on a screen reader opens the same view, hears a summary of the world, moves through the list with the keyboard, and gets the same information.

**Why this priority**: A world that can only be glanced at stops being interesting once it is large, and a picture with no text alternative would shut some users out (NFR-A11Y-1). It follows the stories that create something to look at.

**How to check:** Open a seeded world of a few thousand items; zoom, pan and select with mouse and with keyboard alone; open the list and confirm it holds every item with its name and date; run the view with a screen reader and with reduced motion switched on.

**Acceptance Scenarios**:

1. **Given** a world of any size, **When** the view opens, **Then** the whole world is fitted to the window.
2. **Given** the world view, **When** the user zooms or pans with the mouse, the touchpad or the keyboard, **Then** the view follows, and one action returns it to the whole world.
3. **Given** an item in the world, **When** the user selects it, **Then** they see its name and the date it was drawn.
4. **Given** the world view, **When** the user switches to the list, **Then** every drawn item is listed with its name and date, grouped by day, newest first, and selecting one shows where it is in the world.
5. **Given** a screen reader, **When** the world view opens, **Then** it announces a text summary: whose world it is, how many things have been drawn, and what was drawn most recently.
6. **Given** the operating system's reduced-motion setting is on, **When** the view opens or changes, **Then** nothing animates.
7. **Given** the picture of the world cannot be shown, **When** the user opens the view, **Then** the list still works and the conversation is unaffected.

---

### US5. The world is a file you own (P3)

The user opens the workspace folder and finds the world as a plain text file, one line per thing drawn. They delete a line for a rock they don't like, and the rock is gone the next time they open the world. Months later the app is updated with eighty new items. Their world looks exactly as it did the day before, and the new items only show up in what Steve draws from then on. When they back up or move the workspace, the world goes with it.

**Why this priority**: It protects the promise that everything the companion is lives in files the user owns (Principle I). Most users will not notice it while things work, which is why it follows the visible stories.

**How to check:** Edit the world file in an outside editor (remove a line, add a line with an unknown item id, add a broken line) and confirm the view reflects the edit and the app changes none of those lines; grow a world, add items to the dictionary, and confirm every earlier item is unchanged; back up and move the workspace and confirm the world comes along.

**Acceptance Scenarios**:

1. **Given** the user removes or changes a line in the world file outside the app, **When** the world is next opened, **Then** the view shows the file as it now is, and the app does not put the old line back.
2. **Given** a line names an item the app does not know, **When** the world is shown, **Then** that item appears as a neutral placeholder, is listed as an unknown item, and its line is left untouched.
3. **Given** a line the app cannot read, **When** the world is shown, **Then** the line is skipped, left untouched in the file, and the rest of the world is shown.
4. **Given** a world drawn with an earlier version of the app, **When** a later version adds items to the dictionary or refines how an item is drawn, **Then** every item already drawn stays the same item in the same place.
5. **Given** the workspace is backed up or moved, **When** the copy is opened, **Then** the world is there in full.

---

## Edge cases

- **The id is missing or was changed by hand.** The identity file decides whenever it holds a readable id (FR-038). If the user changed the id, the app uses the new one: what was drawn stays where it is, and colours and future growth follow the new id. The id recorded in the world file is left as it was, since that file is only ever added to. If the id is missing or unreadable, the app restores it from the world's own record when it can, and otherwise gives the companion a new id and tells the user, as it does when it recreates another workspace file.
- **The world file was deleted.** The app starts a new world with the home and tells the user the file was recreated. It does not try to redraw what was there.
- **The home's line was removed by the user.** The world is shown without a home. The app does not add one back, and growth continues around what is there, or from the centre if nothing is.
- **The user added items by hand, far from everything else.** They are shown where the file says. Growth continues from the world as it now is.
- **A conversation runs past midnight.** A message sent after midnight counts for the new local day.
- **The clock or time zone changes.** Items are drawn only for a day later than every day already in the world (FR-019), so going back in time adds nothing, and a day that already has its items never gets more. Moving the clock forward can add a day's items early, and the days after it then wait until the clock catches up. The app does not guard against this: it is the user's own world.
- **The user deletes the day's items.** If the user removes today's lines in an editor and then sends another message, nothing is drawn again: today is no later than the newest day still in the world, or, if the user removed every line of today, the next day with a message draws as usual. The app never puts removed lines back.
- **A message is saved but no reply arrives** (offline, a provider error, the reply was stopped). The day still counts as a day with a conversation.
- **The app is open across several days with no message.** Nothing is drawn.
- **The id is added to an existing companion while a conversation is under way.** The conversation is not interrupted and no notice about a changed file is shown because of it (FR-039). The companion learns its id and birth date when its next fresh context starts.
- **An older version of the app opens a world drawn by a newer one.** Items it does not know are shown as placeholders and their lines are kept.
- **A very large world** (many thousands of items). The view may take longer to open and may show detail only when zoomed in, but it opens, and the list still works.
- **No API key, or no network.** The world view opens and works. The world does not grow, because no message can be sent.
- **Setup is not finished.** There is no id and no world yet, and no way to open the world view.
- **A small window.** The view stays usable at the app's smallest window size. The list can replace the picture when there is not enough room.

## Requirements

**Identity**

- **FR-001**: Every companion MUST have a unique id, created on the device when setup finishes, with no network request. A companion from before this feature MUST be given one the first time the app runs, without the user doing anything (WLD-1).
- **FR-002**: Every companion MUST have a birth date: the day setup finished. For a companion from before this feature it MUST be the day its conversation began when the app can tell, and otherwise the day the id is given.
- **FR-003**: The id and the birth date MUST be recorded with the companion's identity in the workspace, where the user can read them, and MUST be shown in the world view. The id MUST be short enough to read aloud and easy to copy.
- **FR-004**: The id MUST be random. It MUST NOT be derived from the user's name, the companion's name, the API key, the device or any account.
- **FR-038**: When the app adds the id and birth date to an existing identity file, it MUST add them as new lines at the end and leave every other byte of the file as it was. If the file already holds a readable id, the app MUST use it and add nothing (onboarding FR-035). The identity file's id MUST take precedence over the id recorded in the world file, which is used only to restore a missing one.
- **FR-039**: Adding the id and birth date to the identity file is the app's own change, not the user's. It MUST NOT trigger a "file changed" notice in the conversation, and it takes effect from the next fresh context. This narrows chat FR-011 ("the app MUST NOT write to those files") for these two lines only, as memory FR-016 did for the companion's own memory writes. Any other change to the identity file still follows FR-011.

**DNA**

- **FR-005**: The id alone MUST decide the companion's DNA: its colours, the lay of its land (where water, meadow, woods and rock lie), and which items it favours. The DNA MUST be worked out from the id whenever it is needed and MUST NOT be stored (WLD-2).
- **FR-006**: Two companions with different ids MUST differ visibly in their worlds: in colours from the first day, and in layout and choice of items as they grow.
- **FR-007**: The lay of the land MUST NOT be shown in advance. It becomes visible only through what is drawn on it.

**Item dictionary**

- **FR-008**: Everything in a world MUST come from a dictionary of items that ships with the app. Each item MUST have a permanent id, a display name, the kind of place it belongs to, and the stage from which it can be drawn (WLD-3).
- **FR-009**: This slice MUST ship at least 50 items in one consistent, simple drawing style, made by the project, including at least one home. Later versions MAY add items up to several hundred without any change to these requirements.
- **FR-010**: Once an item has shipped, its id MUST NOT be removed, reused for something else, or change what it depicts. How an item is drawn MAY be refined.
- **FR-011**: Item names MUST come from the app's string table so they can be translated (NFR-I18N-1).

**The world file**

- **FR-012**: The world MUST be kept in the workspace as a human-readable file with one drawn item per line (WLD-4). Each line MUST record which item it is, where it is, how it is varied (size, facing, shade), the local date it was drawn, and why it was drawn. In this slice the reason is always "ambient"; the second part of World Building adds a reference to a memory. The file MUST also record the id of the companion it belongs to, so that the id can be restored if the identity loses it.
- **FR-013**: The app MUST only add lines to the world file. It MUST NOT move, change or remove an item once drawn.
- **FR-014**: What is shown MUST come from the lines in the file, not from working the world out again. An update to the app or its dictionary MUST NOT change any item already drawn.
- **FR-015**: Edits the user makes to the world file outside the app MUST be reflected the next time the world is opened and MUST NOT be overwritten. A line that names an unknown item MUST be shown as a neutral placeholder. A line that cannot be read MUST be skipped. Both MUST be left untouched in the file.
- **FR-016**: Adding to the world file MUST be safe against a crash or power loss: at most the items being written are lost, and earlier lines are never damaged (NFR-REL-1).
- **FR-017**: The world MUST be part of the workspace for every purpose: it is included when the workspace is backed up or moved, and later when it is exported or wiped.

**Growth**

- **FR-018**: A new world MUST start with one item, the companion's home, at the centre. It is drawn when the companion gets its id.
- **FR-019**: On each local day on which the user sends at least one message, the companion MUST draw a small number of items, once. Items MUST be drawn only for a local day later than every day the world has already grown (the home's own line marks when the world began, not a day of growth) and later than the last day items were drawn; that is what "once" means, whatever the clock or the user's edits do. The number MUST stay within a daily limit and MUST NOT depend on how many messages were sent or how long they were (WLD-5).
- **FR-020**: The companion MUST NOT draw anything for a day without a message from the user, and MUST NOT draw anything later to make up for past days.
- **FR-021**: The world MUST NOT decay. No item is ever removed, faded, damaged or marked because time passed or because the user was away (WLD-6).
- **FR-022**: Each new item MUST be placed within a short, fixed distance of an item already there, so the world grows outward from the home. Which item is drawn where MUST follow the DNA and the kind of land at that place. A new item MUST NOT be placed so that it fully hides an earlier one.
- **FR-023**: The dictionary MUST be divided into at least three stages (WLD-7). A stage's items become available once the world has reached that stage's size. Reaching a stage MUST NOT remove or replace anything drawn earlier.
- **FR-024**: What is drawn on a given day MUST be decided only by the id, the date and the world as it is. The same three MUST always give the same result, so that growth can be tested exactly (NFR-TEST-1).
- **FR-025**: Growth MUST NOT call the language model, MUST NOT add to any request sent to it, and MUST NOT delay sending a message or showing a reply. A failure to grow MUST NOT show an error in the conversation and MUST be retried at the next message that day (Principle V).

**The world view**

- **FR-026**: Users MUST be able to open the world from the main window and return to the conversation as they left it, including unsent text (WLD-8).
- **FR-027**: The view MUST open with the whole world fitted to the window. Users MUST be able to zoom and pan with the mouse, the touchpad and the keyboard, and return to the whole world with one action.
- **FR-028**: Users MUST be able to select an item and see its name and the date it was drawn.
- **FR-029**: The companion MUST lead the view, as on every other screen (PER-1, PER-2): its portrait, its name, and a line in its first-person voice that states facts from the world file, such as how many things it has drawn and what it drew most recently.
- **FR-030**: The view MUST show the companion's id and birth date, and MUST explain in plain, neutral wording how the world grows and that it never shrinks.
- **FR-031**: The view MUST offer a list of every drawn item with its name and date, grouped by day, newest first. Selecting an item in the list MUST show where it is in the world. The list MUST hold everything the picture shows.
- **FR-032**: The view MUST be usable with the keyboard alone and with a screen reader, MUST provide a text summary of the world, MUST NOT rely on colour alone to tell the user anything, MUST hold still when the reduced-motion setting is on, and MUST take its text from the app's string table (NFR-A11Y-1, PER-8).
- **FR-033**: A problem showing the world MUST NOT block or break the conversation. If the picture cannot be shown, the list MUST still work.
- **FR-034**: With the world view closed, the feature MUST NOT animate anything or use processor time while idle (NFR-PERF-4).

**No pressure**

- **FR-035**: The world MUST NOT produce notifications, badges or reminders. The app MUST NOT show streaks, days in a row, days missed, time since the last conversation, or a countdown or progress bar towards the next stage (NFR-SEC-6, PER-8).
- **FR-036**: Lines in the companion's voice about the world MUST state only what the world file backs up. They MUST NOT claim feelings about the world, about the user's absence, or about how often the user talks to it.

**Privacy**

- **FR-037**: This feature MUST add no network destination and no download. The world file and the DNA MUST NOT be sent anywhere (Principle II).

### Key entities

- **Companion id**: A short random identifier, unique to one companion, recorded with its identity. It is the seed of the DNA.
- **Birth date**: The local date the companion came to be. Recorded with the id.
- **DNA**: The companion's colours, the lay of its land, and its preferences among items. Always worked out from the id, never stored.
- **Item**: One entry in the bundled dictionary: a small drawing with a permanent id, a name, the kind of place it belongs to, and a stage.
- **Item dictionary**: All items that ship with a version of the app. It only grows.
- **Drawn item**: One thing the companion drew: which item, where, how it is varied, the date, and the reason. One line of the world file.
- **World**: Everything one companion has drawn, in the order it was drawn. Kept as one append-only file in the workspace.
- **Stage**: A step in the world's growth that makes more kinds of items available, reached by size.

## Success criteria

- **SC-001**: Across 1,000 generated companions grown over the same 30 days of conversation, no two worlds are the same, and every bundled colour set and every kind of land appears in at least one.
- **SC-002**: For a fixed id and a fixed sequence of days, 100 repeated runs produce the same world file, byte for byte.
- **SC-003**: After a simulated year of growth followed by an update that adds items to the dictionary, 100% of the items drawn before the update are the same item in the same place.
- **SC-004**: Over a simulated 60-day absence the world file does not change, the app shows 0 notifications, badges or lines about the absence, and the first day back adds no more items than the daily limit.
- **SC-005**: A day with 500 messages adds the same number of items as that day would with 1 message.
- **SC-006**: The app's own work before a message is sent stays within 0.15 seconds for 95% of messages, with the world growing (NFR-PERF-2).
- **SC-007**: The world view shows a world of 2,000 items within 1 second of being opened and a world of 5,000 items within 2 seconds, on a mid-range laptop. Zooming and panning respond within 0.1 seconds at both sizes.
- **SC-008**: With the world view closed, the app's idle footprint in the tray still meets NFR-PERF-4.
- **SC-009**: After ten years of daily conversation the world file is smaller than 5 MB.
- **SC-010**: Using only the keyboard and a screen reader, a user can open the world, hear its summary, and reach the name and date of any item. 100% of the items in the picture are in the list.
- **SC-011**: During a capture of a full session that includes world growth and the world view, the app's own connections go only to the Claude API, and this feature causes 0 requests to it.
- **SC-012**: A new user can see their companion's world one action after setup ends, with nothing to configure.
- **SC-013**: An edit made to the world file in an outside editor is shown the next time the world is opened in 100% of attempts, and the app changes none of the edited lines.

## Assumptions

- **Depends on slices 1 and 2, and on slice 3 for one detail.** The identity file, the workspace, the string table and the companion's portrait come from the onboarding feature. "A day with a conversation" is read from the conversation of the chat feature. The entry point in the main window follows the memory browser of the memory feature. Beyond that entry point, the feature needs nothing from memory, compaction or dreaming.
- **Second part, with dreaming (slice 10, after slice 5).** Items chosen from the day's memories, each linked to the memory it came from, arrive with dreaming (WLD-9). So does removing a drawn item in the app, which forgetting a memory will require (Principle IV). The "why it was drawn" field exists now so that part needs no change to the file.
- **The companion does not talk about its world in this slice** (WLD-10 comes later). Nothing about the world is added to the conversation. The identity file is already given to the companion at the start of each context, so from its next fresh context it knows its own id and birth date and nothing more.
- **The companion's own look does not change.** Stages change what the companion can draw, not its portrait or avatar.
- **Layout.** The world is a map that grows outward from the home, not a strip where position stands for time.
- **Starting values**, tuned in the Decisions: 3 to 6 items on a day with a conversation; four stages, beginning at 0, 10, 70 and 260 drawn items; an id of 12 characters from an alphabet without look-alike characters. At 6 items a day, ten years is about 22,000 lines, so SC-009 holds if a line stays under about 200 bytes; the plan sets the line format with that limit and tests it.
- **Where the world file records the companion's id** (FR-012) is settled in the Decisions: the first line. `docs/architecture.md` section 4.1 says so.
- **No setting.** The world is always on. It costs no tokens, sends nothing, and is only visible in its own view.
- **The user edits the world through the file** in this slice, with the existing "open folder" action. Placing, moving or removing items inside the app is out of scope.
- **No sharing.** Saving the world as a picture, sharing it, or comparing worlds between people is out of scope.
- **The art is the project's own.** All items are drawn for the project in one style, so no third-party licence applies.
- **Very large worlds.** Showing only the part of the world in view, which worlds beyond roughly 10,000 items may need, is deferred. The world file already records each item's position, so that work needs no change to the file.
- **The world file is a workspace file under the principles.** Principle I covers `world/`: the app only appends to it, and what was drawn is stored and never worked out again. `docs/requirements.md` (section 6.12, decisions D12 and D13, slices 9 and 10) and `docs/architecture.md` (section 4.1) were updated in the same change.
- **Platform.** Windows 11 first, as for the rest of the app.

## Decisions

Each section records a decision, why it was made, and what was rejected. This slice calls no model, so no API facts are involved. It builds on the onboarding page (identity, setup, file recreation), the chat page (the conversation and its notices) and the memory page (the memory writer, the stable hash, the `written` entry, the memory browser's entry point).

### R1. The companion id

**Decision**: 12 characters from the 31-character alphabet `23456789ABCDEFGHJKMNPQRSTUVWXYZ` (no `0`, `1`, `I`, `L`, `O`), written in three groups of four: `K7Q2-M9TX-3HPA`. That is about 59 bits. The randomness comes from the standard library's `RandomState`, whose keys the operating system seeds per process: two fresh `RandomState` hashers each give 64 random bits.

**Rationale**: FR-003 asks for an id short enough to read aloud and easy to copy; groups of four do both, and an alphabet without look-alikes avoids misreading. 59 bits make a clash between two companions vanishingly unlikely for a personal app. FR-004 requires randomness, not derivation; `RandomState` is OS-seeded, so no dependency is needed (the principles, Simplicity). The id is an identity, not a secret, so cryptographic strength isn't required.

**Alternatives considered**: `getrandom` or `rand` (a dependency for 64 bits). A UUID (36 characters, hard to read aloud). A hash of the time and process id (not random in the sense of FR-004).

### R2. Where the id and birth date live, and how they get there

**Decision**:

- **New companions**: setup's `IDENTITY.md` gains `- Id: K7Q2-M9TX-3HPA` and `- Born: 2026-11-14` after the avatar line. `complete_setup` gains the user's `LocalClock`, as the chat commands already have, for the local birth date.
- **Companions from before this feature**: a new command, `prepare_world(clock)`, called by the main view when it opens, adds the two lines at the end of `IDENTITY.md` and leaves every other byte as it was (FR-038). If `IDENTITY.md` already has a readable `Id`, nothing is added. If it has none but the world file records one (R5), that id is restored. The birth date is the local date of the first message in the conversation, read from that message's time block (`<context>Local time: Friday 13 November 2026, …`, chat R8), or today when there is no conversation.
- **No notice** (FR-039): after the write, if a context is running, the conversation file gets a `written` entry for `IDENTITY.md`, as the companion's own writes do in memory R7. `seen` then matches the file, so the next message carries no notice; the system prompt keeps its snapshot until the next fresh context.

**Rationale**: The core has no local time zone (chat R8), so the birth date needs the webview's clock; a command called when the main view opens runs on the first launch after the update without the user doing anything (US1, scenario 3). The time block is the one record of the user's local date for old messages; its format is the app's own and fixed. Reusing `written` keeps one mechanism for "the app changed a loaded file".

**Alternatives considered**: Adding the id in `app_status` (no clock, so no local birth date). Storing the id only in `state.json` (FR-003 wants it with the identity, where the user can read it). A `notice` with the new identity (FR-039 forbids it, and it would cost tokens for nothing).

### R3. One writer for workspace files the app changes

**Decision**: the memory feature's `edit_memory` becomes a thin wrapper over `edit_workspace(rel, change)`, the same read-apply-write under the same lock. `IDENTITY.md` and the world file are written through it or, for the world file's appends, through an append under the same lock.

**Rationale**: `lean-code`: one writer, no parallel helper. `MemoryPath` stays the browser's allow-list, so the browser can't write `IDENTITY.md` or the world file.

### R4. DNA

**Decision**: a seeded generator (SplitMix64) whose seed is the memory feature's stable FNV-1a hash of the id plus a purpose label. From it:

- **Palette**: one of 8 bundled palettes, each a named set of 6 colours (ground, water, leaf, bark, stone, accent), made a little lighter or darker per companion (a factor of 0.92 to 1.08). The webview applies them as CSS custom properties on the world picture.
- **Land**: a value-noise field over the grid, two octaves, seeded by the id, with id-dependent thresholds that split it into `water`, `meadow`, `woods` and `rock`. The cell at the home and its eight neighbours are always `meadow`.
- **Preferences**: a weight between 0.5 and 2.0 per item id, from the seed and the item id.

DNA is computed when needed and never written anywhere (FR-005). The land is used only to choose what grows where; the view never shows it (FR-007).

**Rationale**: SplitMix64 and value noise are each a few lines with no dependency, and are fully deterministic across platforms, which the golden tests need (FR-024, NFR-TEST-1). The memory feature's hash is already guaranteed stable across app versions.

**Alternatives considered**: `rand` with a seeded generator (a dependency, and its algorithms may change between versions, which would change every world grown afterwards). Perlin or simplex noise (more code for no visible gain at this grid size). Storing the DNA (FR-005 forbids it).

### R5. The world file

**Decision**: `workspace/world/world.jsonl`, JSON Lines.

- The first line names the companion: `{"world":1,"companion":"K7Q2-M9TX-3HPA","born":"2026-11-14"}`. It is written once, when the file is created, and never changed (FR-012, FR-038).
- Every other line is one drawn item:
  `{"item":"flower.daisy","x":3,"y":-2,"size":0.9,"flip":false,"shade":1,"day":"2026-11-14","why":"ambient"}`.
  `x` and `y` are integer grid cells with the home at `0,0`. `size` is 0.8–1.2 in steps of 0.1, `flip` mirrors the drawing, `shade` (0–2) picks a lighter or darker tone of the item's colour. `why` is `"ambient"` in this slice; slice 10 adds a memory reference in a new field.
- One item per cell, so no item can hide another (FR-022).
- A typical line is about 110 bytes: at 6 items a day, ten years is about 2.4 MB, under SC-009's 5 MB. A test fails if a line exceeds 200 bytes.
- Appends use append mode and `sync_data`, starting on a new line if the file ends with a half-written one, as the conversation file does (chat R6). A crash loses at most the line being written (FR-016).
- Reading skips lines that aren't valid JSON or lack `item`, `x`, `y` or `day`, counts them, and never rewrites the file (FR-015). An unknown `item` id is kept and shown as a placeholder.

**Rationale**: The conversation file already uses JSON Lines with the same append and skip rules, so the reader and writer pattern is proven. Readable key names keep the file human-readable (Principle I) and still fit the size budget. A header line is the simplest place for the companion's id; `docs/architecture.md` section 4.1 is updated to say the first line names the companion.

**Alternatives considered**: Short keys (`"i"`, `"d"`) to halve the size (less readable, and the budget is met anyway). Storing the id inside the home's line (lost if the user deletes the home). Free coordinates (needs overlap tests and makes the file harder to edit by hand).

### R6. Growth

**Decision**: `grow(id, today, world, dictionary) -> Vec<Item>`, a pure function:

1. Draw nothing unless `today` is later than every `day` of the world's grown items (the home's line doesn't count, so the first message on setup day draws) and later than the last drawn day kept in `state.json` (R7). That is FR-019's "once".

The free cells within reach are found once per day and kept up to date as items are placed, and each item's preference weight is worked out once, so a day's growth stays cheap in a world of tens of thousands of items. Growth skips a companion that has no id or no world file yet: `prepare_world` creates both when the main view opens, before any message can be sent.
2. The day's generator is seeded with the hash of the id and `today`. The count is 3 to 6.
3. The stage is set by the number of items in the world: stage 1 from 0, 2 from 10, 3 from 70, 4 from 260 (Assumptions).
4. For each item: the free cells within 2 cells of any drawn item are the candidates, in a fixed order (by distance from the home, then `y`, then `x`). One is picked at random, weighted towards the home. The item is picked from the dictionary entries whose stage is reached and whose land matches the cell's land (or is `any`), never a home (a world has the one home it starts with); `near-home` items only within 4 cells of the home. Each entry's chance is its DNA preference weight. If nothing fits a cell, the next candidate is tried.
5. Size, flip and shade come from the same generator.

The same id, date and world always give the same items (FR-024). An empty world, for example after the user deleted every line, grows from the centre.

**Rationale**: FR-022 asks for outward growth near existing items, FR-023 for stages, FR-020 and FR-021 for no catch-up and no decay; a pure function of the three inputs makes all of them testable exactly. Candidates in a fixed order make the result independent of hash-map ordering.

**Alternatives considered**: Growth scaled by messages (FR-019 forbids it). Making up missed days (FR-020 forbids it). Asking the model (FR-025 forbids it).

### R7. When growth runs, and the last drawn day

**Decision**: `send_message` grows the world right after the user's message is saved, before the request is built, using the message's clock. The hidden first-hello request does not count (it is not from the user). The growth step is skipped when `today` is not later than `state.json`'s `world.lastDay`, so most messages never read the world file. When it runs, it reads the world file, grows, appends, and sets `lastDay`. A failure is logged without content and leaves `lastDay` unchanged, so the next message that day tries again (FR-025). Any failure is swallowed: the message is sent regardless.

**Rationale**: Principle V: the user's message is saved first, and growth (a few milliseconds of file work once a day) never delays the reply. `lastDay` in `state.json` is bookkeeping, which the principles allow outside the workspace; it makes "once" hold even when the user deletes the day's lines (edge cases), and spares every other message a file read. The world file stays the source of truth for what was drawn.

**Alternatives considered**: Growing on a timer (draws on days without a message). Growing after the reply ends (a stopped or failed reply would skip a day that counts, edge cases). Keeping `lastDay` only in memory (a restart on the same day would draw twice once the user deleted that day's lines).

### R8. The item dictionary

**Decision**: `crates/steve-core/src/world/items.json` is the single list: for each item its permanent id (`kind.name`, for example `flower.daisy`), its land (`meadow`, `water`, `woods`, `rock`, `any`, `near-home`) and its stage (1–4). The core embeds it with `include_str!`. The drawings are `src/assets/world/<id>.svg`, and the names are in `src/strings.ts` under `t.world.items`, keyed by id (FR-011). A Vitest test imports `items.json` and fails if any item lacks a drawing or a name; a Rust test fails if an id is duplicated or if an id from a committed snapshot of earlier ids is missing (FR-010).

This slice ships 56 items: 2 homes; stage 1 (14: grasses, flowers, pebbles, mushrooms, clover); stage 2 (16: bushes, saplings, ferns, reeds, a pond, rocks, logs); stage 3 (14: trees, boulders, a stream piece, lily pads, a bench, a well); stage 4 (10: fence, lantern, mailbox, boat, bridge, garden bed, path stones, windmill, signpost, birdhouse).

**Rationale**: The core needs land and stage to grow (R6); the webview needs only drawings and names. One JSON file read by both keeps them from drifting without generated code for data. Stable ids with a snapshot test enforce FR-010.

**Alternatives considered**: The list as Rust constants plus a TypeScript copy (two sources). Shipping the drawings inside the core (the core has no reason to hold pictures, and the webview can't use them from there).

### R9. Drawing style and the picture

**Decision**: every item is an SVG on a 32 × 32 grid, built only from simple shapes (circles, ellipses, rectangles with rounded corners, short paths), with flat fills and no strokes, gradients or text. Colours are CSS custom properties (`var(--w-leaf)` and so on), so the companion's palette (R4) colours every item and `shade` picks a lighter or darker variant. The world view renders one `<svg>`: each kind of item used is a `<symbol>`, each drawn item a `<use>` placed at its cell, sized, flipped and shaded. Zoom and pan change the `viewBox`; nothing animates (FR-032, FR-034). An unknown item id renders as a neutral rounded square.

**Rationale**: FR-009 asks for one consistent, simple style made by the project. Inline symbols let CSS custom properties reach every item, which image tags can't. `<use>` keeps a 5,000-item world one element per item (SC-007). Changing the `viewBox` is cheap and needs no animation library.

**Alternatives considered**: A canvas (no accessible items, harder to hit-test, and the list would be the only way to select). Image tags per item (no palette, one request per item). A map library (a dependency for pan and zoom on one picture).

### R10. The world view

**Decision**: a third icon button in the main window, beside Memory and Settings, opens a view built like the memory browser: an app bar with Back, the companion's portrait and a first-person line (FR-029), for example "I've drawn 42 things. Most recently, two daisies and a pebble." Below it:

- **The picture**, fitted to the window when it opens (FR-027): wheel or pinch to zoom, drag to pan, `+`/`-`, arrow keys and `0` ("Whole world") on the keyboard. Each item is focusable in reading order (by day, then position) and named "Daisy, drawn 14 November 2026"; selecting one shows its name and date (FR-028).
- **The list** (FR-031): every item, grouped by day, newest first; choosing one centres it in the picture and selects it. Unknown items appear as "Unknown item (id)". The list replaces the picture when the window is narrow.
- **About this world**: the id (with a copy button), the birth date, and one neutral sentence on how the world grows and that it never shrinks (FR-030). No stage, progress, streak or countdown is shown (FR-035).

The view reads the world through `world_view()`, which returns the companion's id and birth date, the palette, the drawn items with their dates, and the count of skipped lines. The conversation component stays mounted, so returning to it keeps unsent text (FR-026), as with the memory browser.

**Rationale**: The memory browser established the pattern (memory R15). Focusable items and the list together meet NFR-A11Y-1 and SC-010. The companion's line counts and names only what the file holds (FR-036).

### R11. A deleted or damaged world file

**Decision**: `prepare_world` recreates a missing world file with its header and the home, dated today, and reports it in its result so the main view shows the existing "files were missing, so I recreated them" line (edge cases). It never redraws what was there. A home line the user removed is never added back; growth continues from what is there, or from the centre.

**Rationale**: The onboarding feature already tells the user when it recreates a workspace file; the world joins the same message.

### R12. Test strategy

**Decision**:

- **Core unit tests**: id format and alphabet; SplitMix64 and noise golden values; palette and land differ between two fixed ids; the world file round-trips and skips bad lines without counting good ones; stage thresholds; candidate order; `items.json` has unique ids and contains every id in the committed snapshot (FR-010); every line stays under 200 bytes.
- **Golden test**: a fixed id grown over a fixed sequence of 40 days produces a committed world file, byte for byte (SC-002, FR-024).
- **Core integration tests** on a temporary workspace: setup writes the id, birth date, header and home; `prepare_world` adds two lines to a hand-edited `IDENTITY.md` and keeps every other byte, uses an existing id, restores a missing id from the world file, and writes a `written` entry so the next message has no notice (FR-038, FR-039); growth happens once on the first message of a day, not for the hello, not twice after the day's lines are deleted, not after the clock goes back, and the same after 1 or 500 messages (SC-004, SC-005); a 60-day gap adds one ordinary day; a failed append (world folder made a file) doesn't fail the message and retries; a missing world file is recreated.
- **Diversity test**: 1,000 ids grown over 30 days give 1,000 different files, and every palette and land kind appears (SC-001). Ignored in CI if it takes more than a few seconds in debug.
- **UI tests**: every item in `items.json` has a drawing and a name; the view fits the world, zooms and pans by keyboard, selects an item and shows its name and date, the list groups by day, the summary line counts items, reduced motion changes nothing, and no stage or streak text appears.
- **End-to-end**: after setup, open the world from the main window, see the home and the id, and return to the conversation with its unsent text.

**Rationale**: The principles' test gates and the `test-audit` skill: growth is pure, so its contract is owned by unit and golden tests; integration tests cover the file and identity rules; the UI is tested through behaviour.

### R13. Deliberately not built in this slice

- Items drawn from memories, and removing an item when its memory is forgotten (slice 10).
- The companion talking about its world (WLD-10), or any world line in the request.
- Changes to the companion's look (WLD-11), saving the world as a picture (WLD-12), sharing.
- Placing, moving or removing items in the app; the user edits the file.
- Showing only the visible part of very large worlds.
- A setting to turn the world off.

## Data

Files and types from the onboarding, chat and memory pages are unchanged except where noted. Rust definitions are the source; the TypeScript types in `src/bindings/` are generated from them.

### Data 1. `IDENTITY.md` — gains two lines

```markdown
# Identity

- Name: Steve
- Avatar: pip
- Id: K7Q2-M9TX-3HPA
- Born: 2026-11-14
```

| Line | Rule |
|---|---|
| `- Id:` | Three groups of four characters from `23456789ABCDEFGHJKMNPQRSTUVWXYZ`, joined by `-` (R1). Read case-insensitively; an unreadable value counts as missing. |
| `- Born:` | `YYYY-MM-DD`, the local date setup finished, or for an older companion the local date of its first message (R2). An unreadable value is shown as unknown and never rewritten. |

Setup writes both lines. For an existing file, `prepare_world` appends them at the end, leaving every other byte unchanged, and only when `Id` is missing (FR-038).

### Data 2. `world/world.jsonl` — new

One JSON object per line, appended only (FR-013).

#### First line: the companion

| Field | Type | Meaning |
|---|---|---|
| `world` | number | Format version, `1`. |
| `companion` | string | The companion's id when the file was created. Used only to restore a missing id (FR-038). |
| `born` | string | The birth date when the file was created. |

#### Every other line: a drawn item

| Field | Type | Rule |
|---|---|---|
| `item` | string | A dictionary id, `kind.name`. An unknown id is kept and shown as a placeholder (FR-015). |
| `x`, `y` | integer | Grid cell, home at `0,0`. At most one item per cell (FR-022). |
| `size` | number | 0.8 to 1.2 in steps of 0.1. |
| `flip` | boolean | Drawn mirrored. |
| `shade` | integer | 0, 1 or 2: the item's colour, lighter, or darker. |
| `day` | string | `YYYY-MM-DD`, the local day it was drawn. |
| `why` | string | `"ambient"` in this slice. Slice 10 adds memory-drawn items with an extra field. |

Example: `{"item":"flower.daisy","x":3,"y":-2,"size":0.9,"flip":false,"shade":1,"day":"2026-11-14","why":"ambient"}` (about 110 bytes; every line stays under 200, SC-009).

**Validation on read**: a line that isn't valid JSON, or lacks `item`, `x`, `y` or `day`, is skipped and counted, and the file is never rewritten (FR-015). Missing `size`, `flip`, `shade` or `why` take their defaults (1.0, false, 0, `"ambient"`). A first line that isn't a header is read as an item.

**Writing**: appended in append mode with `sync_data`, on a new line if the file ends with a half-written one (FR-016), through the core's workspace writer lock (R3).

### Data 3. The conversation file

`written` (memory Data) may now name `IDENTITY.md`. Its `message` field becomes optional and is left out for this write, which belongs to no tool-result message. Older files read the same.

### Data 4. State

`state/state.json` gains:

```json
{ "world": { "lastDay": "2026-11-14" } }
```

The last local day items were drawn (R7). Missing means "never": the world file's newest `day` decides alone. Bookkeeping only; the world file is the source of what was drawn.

### Data 5. The item dictionary (bundled, not in the workspace)

`crates/steve-core/src/world/items.json`, an array:

| Field | Type | Rule |
|---|---|---|
| `id` | string | Permanent, unique, `kind.name`, lowercase. Never removed or reused (FR-010). |
| `land` | string | `meadow`, `water`, `woods`, `rock`, `any` or `near-home`. |
| `stage` | number | 1 to 4. Stage 1 from 0 drawn items, 2 from 10, 3 from 70, 4 from 260. |
| `home` | boolean | Optional; true for items that can be a companion's home. A home is drawn only when the world is created, never by growth. |

Each id has a drawing at `src/assets/world/<id>.svg` and a name at `t.world.items[id]`.

### Data 6. DNA (computed, never stored)

| Part | From | Used for |
|---|---|---|
| Palette | the id's seed: one of 8 palettes and a small hue shift | Colours of the picture (sent to the webview as six colours). |
| Land | value noise seeded by the id; home and its neighbours are `meadow` | Which items grow where. Never sent to the webview (FR-007). |
| Preferences | a weight 0.5–2.0 per item id | How likely each item is. |

### Data 7. Types

#### WorldView (core → webview)

| Field | Type | Meaning |
|---|---|---|
| `companionId` | string | From `IDENTITY.md`. |
| `born` | string \| null | From `IDENTITY.md`; null when unreadable. |
| `palette` | `WorldPalette` | `{ ground, water, leaf, bark, stone, accent }`, hex colours. |
| `items` | `DrawnItem[]` | In file order. |
| `skippedLines` | number | Unreadable lines left in the file. |

`DrawnItem`: `{ item: string, known: boolean, x: number, y: number, size: number, flip: boolean, shade: number, day: string }`.

#### PrepareOutcome (core → webview)

`{ recreated: string[] }`: workspace files `prepare_world` had to recreate, today only ever `world/world.jsonl`. Shown with the existing "files were missing" line.

### Data 8. State transitions

```
setup finished ─► IDENTITY.md with Id + Born, world.jsonl with header + home (day = birth)
older companion, first launch ─► prepare_world: Id + Born appended (or restored), world.jsonl created if missing, written entry if a context runs
user message saved ─► today > lastDay and > newest day in file? ─► grow 3–6 items, append, lastDay = today
                                                              └─► no: nothing
world.jsonl deleted ─► prepare_world: header + home, dated today; reported as recreated
```

## Interfaces

### World Commands Between the Webview and the Core

These commands extend the surfaces of the onboarding, chat and memory pages. The same rules apply: thin handlers, validation in the core, expected failures as values, each command declared in `build.rs` and allowed in `src-tauri/capabilities/main.json`, and `src/api/commands.ts` as the only caller. This slice adds no network request (FR-037).

#### New commands

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `prepare_world` | `clock: LocalClock` | `PrepareOutcome` | Valid only after setup is complete (otherwise `InvalidInput("setup")`). Ensures the companion has an id and birth date (appending them to `IDENTITY.md`, or restoring the id from the world file) and a world file (recreating header and home if missing). Writes a `written` entry when it changed `IDENTITY.md` while a context runs. Idempotent: a second call changes nothing. Called by the main view when it opens. | FR-001 to FR-004, FR-018, FR-038, FR-039 |
| `world_view` | — | `WorldView` | Reads `IDENTITY.md` and the world file, computes the palette. Writes nothing. An absent world file gives an empty list. | FR-026 to FR-033 |

#### Changes to earlier commands

| Command | Change |
|---|---|
| `complete_setup` | Gains `clock: LocalClock`. A new companion's `IDENTITY.md` gets `Id` and `Born` (the clock's local date), and `world/world.jsonl` is created with its header and the home. `AdoptExisting` writes nothing, as before; `prepare_world` handles that companion when the main view opens. |
| `send_message` | After the user's message is saved, grows the world for the clock's local day if that day is later than `lastDay` and every day in the world file (R6, R7). Any growth failure is swallowed; the reply is never delayed or failed by it. The first-hello, retry and regenerate commands never grow the world. |

#### Capability file additions

`allow-prepare-world`, `allow-world-view`.

## Checks


### Prerequisites

- Onboarding, chat and memory set up, or a clean machine to run the wizard.
- Rust 1.90 or later, Node.js 22 or later, pnpm 10.
- A Claude API key for the scenarios that send messages. The automated checks never use one, and growth itself never calls the API.

### Inspect the files

- `%APPDATA%\mySteve\workspace\IDENTITY.md`: `- Id:` and `- Born:` lines.
- `%APPDATA%\mySteve\workspace\world\world.jsonl`: a header line, then one item per line (Data 2).
- `%APPDATA%\mySteve\state\state.json`: `world.lastDay`.

### Manual validation scenarios

| # | Story | Steps | Expected |
|---|---|---|---|
| 1 | US1 | Finish the wizard on a clean machine and open the world from the main window. | A canvas with the home in the middle, the id in groups of four and today as the birth date, in one action after setup (SC-012). |
| 2 | US1 | Do the same on a second clean machine (or a second data root). | A different id and visibly different colours. |
| 3 | US1 | On a machine with a companion from the memory feature, install this build and launch it mid-conversation. | `IDENTITY.md` gains two lines at the end and is otherwise unchanged; a world file exists with a home. The next message shows no "IDENTITY.md updated" note. |
| 4 | US1 | Relaunch, open the world. | Id, birth date and world unchanged. |
| 5 | US1 | Type a message without sending, open the world, go back. | The typed text is still there. |
| 6 | US2 | Send a message. Open the world. | 3 to 6 new items near the home, dated today. |
| 7 | US2 | Send ten more messages today. Open the world. | Nothing new. |
| 8 | US2 | Set the system date forward one day, send one message. Then forward another 30 days and send one message. | Each adds one ordinary day's items; nothing is added for the skipped days, and nothing anywhere mentions the gap. |
| 9 | US2 | Set the system date back two days and send a message. | Nothing is drawn. |
| 10 | US2 | With the network off, send a message on a new day. | The message is kept with Retry, and the day's items are drawn. |
| 11 | US2 | Make `world` a file instead of a folder, send a message on a new day. Restore it and send another. | The first message works with no world error; the second draws the day's items. |
| 12 | US3 | Use the test helper to grow a world for 300 simulated days, then open it. | Small plants first, then bushes, trees and rocks, then built things near the home and water items on wet land. Nothing earlier replaced. No progress bar or countdown. |
| 13 | US4 | Open the 300-day world. Zoom with the wheel, pan by dragging, press `+`, `-`, the arrows and `0`. | The view follows within a blink; `0` fits the whole world (SC-007). |
| 14 | US4 | Tab to an item and select it; switch to the list and choose an entry. | The item's name and date are shown; the list groups by day, newest first, and its entry centres the item. |
| 15 | US4 | With Narrator on and reduced motion set in Windows, open the world. | The summary is read; every item is reachable by keyboard with its name and date; nothing animates (SC-010). |
| 16 | US4 | Make the window as small as the app allows. | The list replaces the picture and still works. |
| 17 | US5 | In a text editor, delete a rock's line and add a line with `"item":"thing.unknown"` and a broken line. Open the world. | The rock is gone, a placeholder "Unknown item" appears, the broken line is skipped and reported, and the app changed none of those lines (SC-013). |
| 18 | US5 | Delete today's item lines and send another message today. | Nothing is drawn again. |
| 19 | US5 | Delete `world.jsonl` and relaunch. | A new world with only the home; the app says the file was recreated. |
| 20 | US5 | Delete the `- Id:` line from `IDENTITY.md` and relaunch. | The id comes back from the world file, appended at the end. |
| 21 | Edge | Delete the home's line, send a message on a new day. | The home is not added back; items grow from what is there. |
| 22 | SC-011 | Capture network traffic for a session that grows the world and opens the view. | Only `api.anthropic.com`, and only for chat. |
| 23 | SC-008 | Leave the app in the tray for an hour with the world view closed. | The idle footprint still meets NFR-PERF-4. |

### Release-readiness checks (not run per commit)

| Check | How | Pass |
|---|---|---|
| SC-001 diversity | `cargo test -p steve-core --release --lib -- --ignored --nocapture a_thousand_companions` | 1,000 different worlds; every palette and land kind used. |
| SC-007 view speed | Open seeded worlds of 2,000 and 5,000 items on a mid-range laptop. | Within 1 s and 2 s; zoom and pan within 0.1 s. |
| SC-009 size | The ten-year simulation in the same ignored test. | Under 5 MB. |
