# World journey

Built.

**Refinement, 2026-11-14**: "The world is a visual map of the user's diary. It reflects what really happened to the user in the real world. That is why we need to add more common things that represent the daily objects of human activities. There will be a tooltip for each object that shows the keyword of that day."

This is the second part of World Building (`docs/requirements.md` section 6.12, delivery slice 10, WLD-9 and WLD-10), revised. In the first part (the world page) the world grows from a random seed. In this part the world becomes a picture of the user's days: each night the companion works out what happened in the user's day from what the user told it, and an everyday object that stands for it is added to the map, carrying the day's keyword. The reasons are recorded and shown, and the companion knows its own world.

## User stories

### US1. The world maps the user's days (P1)

On Tuesday the user tells Steve they ran ten kilometres before work and baked a cake for their sister's birthday. Overnight, while Steve dreams, he settles on what happened in the user's day: a run, and the birthday. For each, one everyday object is drawn: a pair of running shoes with the keyword "10 km run", and a cake with the keyword "sister's birthday". They stand together, beside the grass and flowers drawn earlier that day. On a day when Steve cannot work out what happened (the provider is unreachable, or the user said nothing about their day), the world still gets its bits of scenery and nothing else.

**Why this priority**: This is the main idea: the world is a visual diary of what really happened to the user. Everything else in this feature shows or uses what this story records.

**How to check**: With a scripted provider and simulated dates, hold a conversation in which the user reports two things they did, let the night's sweep run, and confirm the world gained one everyday object per topic from the right subject, each with its keyword, placed beside that day's scenery; confirm that nothing the user did not say became a topic, that the day stayed within its limit, and that with the provider unreachable only scenery was drawn and chat was unaffected.

**Acceptance Scenarios**:

1. **Given** a day with conversation has ended, **When** the night's dream sweep runs, **Then** the companion names up to three topics for that day, each something the user said they did, that happened to them, or that was on their mind, with a keyword, one short line, and one or two subjects from the app's fixed list.
2. **Given** a topic, **When** its object is chosen, **Then** the app picks one everyday object by weighted chance from the dictionary items filed under the topic's subjects, and draws it beside the scenery of that day.
3. **Given** the companion suggested something the user did not do, or the two talked about something imagined, **When** topics are named, **Then** none of that becomes a topic.
4. **Given** a day already has its topic objects, **When** another sweep runs, **Then** nothing more is drawn for that day.
5. **Given** the provider is unreachable or the user said nothing about their day, **When** the sweep runs, **Then** no topic object is drawn, the scenery from the day's first message stays as it is, and no error appears in the conversation.
6. **Given** the app was closed for the night, **When** the sweep catches up later, **Then** the days it missed that had conversation get their topics then, for at most the seven most recent such days.
7. **Given** a sweep is interrupted while naming topics, **When** the app next runs, **Then** the world and the journey are exactly as before the sweep, and the day is done at the next sweep; if the app stopped while writing, the next sweep completes what was left.

### US2. Read the map as a diary (P1)

The user opens Steve's world and moves the pointer over the running shoes. A tooltip shows "10 km run" and the date. The flowers next to them show the same date and that day's keywords. Under the picture the list is now a diary: for each day, the keywords, a line about what happened, and the objects drawn for it. The user selects the cake and reads the line about the birthday, opens "why this one" to see the other objects that could have been drawn and their chances, and follows a link to that day's notes in the memory browser.

**Why this priority**: The user's complaint is that the journey is invisible. A map of the diary only works if every object says which day and what it stands for. It needs story 1 to have something to show.

**How to check**: Seed a world with several days of topics, open the world, and confirm every object shows a tooltip with its date and the day's keywords on hover, on keyboard focus and on tap; that the list shows each day's keywords and lines beside the objects drawn for them; that selecting an object shows its story; that the chances shown match the record; and that the whole view works with the keyboard and a screen reader.

**Acceptance Scenarios**:

1. **Given** an object drawn for a topic, **When** the user points at it or moves keyboard focus to it, **Then** a tooltip shows its name, its date and the day's keywords, its own keyword first.
2. **Given** scenery drawn on a day that has topics, **When** the user points at it, **Then** the tooltip shows its name, its date and that day's keywords.
3. **Given** an object from a day without topics, **When** the user points at it, **Then** the tooltip shows its name and date only.
4. **Given** days with topics, **When** the user opens the world, **Then** the list shows one entry per day, newest first, with each topic's keyword and line, the object drawn for it, and the scenery drawn that day.
5. **Given** an object drawn for a topic, **When** the user opens "why this one", **Then** they see the candidates that were considered with their chances, which one was picked, and whether it came from a related subject.
6. **Given** a day's entry, **When** notes exist for that day, **Then** the entry offers a way to open them in the memory browser.
7. **Given** the most recent day with topics, **When** the view opens, **Then** the companion's line says what happened that day and what was drawn for it, using only what the record holds.

### US3. The same days don't fill the world with the same thing (P2)

For three weeks the user tells Steve about little else than work. The world does not fill with laptops. The first work day brings a laptop, the next a briefcase, then a desk lamp, and now and then something from a neighbouring subject, such as a coffee cup. An object that is already in the world is less likely each time, and one drawn in the last few days less likely still. Whatever is drawn, the keyword still says what the day was really about.

**Why this priority**: Without it a steady life produces a monotonous map, which defeats the point of story 1. It is a rule on top of story 1's choice and can be tuned separately.

**How to check**: Simulate 30 days with the same single subject and confirm the spread of objects; repeat with the same inputs and confirm the same picks; recompute the recorded chances and confirm they match; confirm no object from an unrelated subject was ever drawn.

**Acceptance Scenarios**:

1. **Given** an item already drawn several times, **When** a topic of its subject comes up again, **Then** its chance is lower than that of an item of the same subject not yet drawn.
2. **Given** an item drawn in the last seven days, **When** it is a candidate, **Then** its chance is lower still.
3. **Given** any topic, **When** its object is picked, **Then** the candidates are items of the topic's subjects and, with a lower chance, items of subjects marked as related. An item of an unrelated subject is never picked.
4. **Given** the same world, day and topics, **When** the pick is repeated, **Then** the result is the same.

### US4. Steve knows his world (P2)

The user sees the cake and asks Steve about it in chat. Steve knows: he drew it on Tuesday night for the sister's birthday, and it stands by the pond east of his home. He can say what is in his world, what he drew most recently and for what, and what the user's days have mostly been made of. He does not invent things that are not there.

**Why this priority**: Once objects stand for the user's days, a Steve who cannot talk about them in conversation would contradict his own world (PER-8). It depends on story 1 for the stories.

**How to check**: Seed a world with stories, start a fresh context, ask what was drawn most recently and why, and confirm the answer matches the record; confirm that the conversation's earlier requests are unchanged when the world grows.

**Acceptance Scenarios**:

1. **Given** a context starts, **When** the companion is given what it knows, **Then** that includes a short description of its world: how much is drawn and since when, what lies in each direction from the home, the most recent objects with their keywords and lines, and what the world is mostly made of.
2. **Given** a sweep drew topic objects overnight, **When** the user sends their next message, **Then** the companion knows what it drew and why, and nothing earlier in the conversation was changed to tell it.
3. **Given** a world of any size, **When** the description is made, **Then** it stays within its size budget.
4. **Given** a context begun before this feature, **When** it continues, **Then** it carries on unchanged until a new context starts.

### US5. Watch it grow, and see what the days are made of (P3)

The user drags a time control back to the first week and sees the world as it was: a home and some grass. They let it play forward and watch the months fill in with the things they did. Beside the picture a strip shows what their days have been made of: a third work, a fifth running, some cooking, some family. This mix, not an id, is what makes this world theirs and Steve's.

**Why this priority**: It turns the record into something the user can take in at a glance. It uses only what stories 1 and 2 already store.

**How to check**: Open a seeded world, move the time control with the mouse and the keyboard, confirm the picture shows exactly the items drawn up to that day, and confirm the makeup matches a count of the record; repeat with reduced motion on.

**Acceptance Scenarios**:

1. **Given** a world, **When** the user picks a past day with the time control, **Then** the picture shows the world as it was at the end of that day.
2. **Given** the time control, **When** the user plays forward, **Then** the world fills in day by day up to today, and holds still between steps when reduced motion is on.
3. **Given** topic objects in the world, **When** the view opens, **Then** it shows the share of each subject among them, as text and as a picture.
4. **Given** the world view, **When** it opens, **Then** it no longer shows the companion's id.

### US6. Take an entry back (P3)

One night Steve drew a medicine bottle with the keyword of a friend's illness. The user would rather not have that on screen. They choose Forget on it, confirm, and the keyword and the line are gone. The bottle stays, now just a bottle with its date. Later they decide they do not want their days drawn at all for a while and switch it off; the world keeps getting its scenery.

**Why this priority**: The keywords and lines come from the user's private life and are shown in a view someone else might see. The user has to be able to take one back (CTL-2, Principle IV). It follows the stories that create them.

**How to check**: Forget a topic and confirm its keyword and line are gone from the tooltips, the list, the record and the companion's next context while the object remains; forget a memory in the memory browser and confirm that day's topics are gone too; switch topic drawing off and confirm sweeps name no topics.

**Acceptance Scenarios**:

1. **Given** a topic in the journey, **When** the user chooses Forget, **Then** the app asks for confirmation, showing the keyword and the line that will be removed.
2. **Given** the user confirms, **When** forgetting completes, **Then** the keyword and the line are gone from the tooltips, the list and the record, the object remains without a story, and later contexts do not mention the story.
3. **Given** the user forgets a memory in the memory browser, **When** forgetting completes, **Then** the topics of the day that memory came from are removed in the same way.
4. **Given** topic drawing is switched off, **When** a sweep runs, **Then** no topics are named, nothing is sent to the model for the world, and scenery continues.

## Edge cases

- **The user tells about another day** ("yesterday I…", "last week we…"). The topic is drawn on the day it was told. Its line may say when it happened.
- **A plan, not an event** ("I'm flying to Tokyo next week"). It may be a topic because it was on the user's mind. Its line says it is a plan and does not say it happened.
- **The user only asks questions and says nothing about their life.** The day has no topic unless something clearly occupied them. It gets scenery only.
- **What happened fits no subject on the list.** The topic is filed under the catch-all subject and gets a neutral everyday object. Its keyword still says what it was.
- **Dreaming is off.** No topics are named. The world grows by scenery alone. "Dream now" still names topics for days that have ended, if topic drawing is on.
- **"Dream now" during the day.** Today is not finished, so it gets no topics yet. Earlier days without topics get theirs.
- **A sweep covers many days** (the app was closed for weeks, or dreaming was off). Only the seven most recent days with conversation get topics. Older ones stay as they are and are not revisited.
- **Days from before this feature.** They are never given topics. The app does not go back through old conversations.
- **A very long day.** The app reads a bounded part of the day's conversation, spread across the day, with what the user said first.
- **The model returns a subject that is not on the list, or a keyword or line that is empty or too long.** That topic is dropped. If none is left, the day has no topics.
- **Two topics on one day lead to the same item.** The second pick already counts the first as drawn, so the same item twice in one day is unlikely but allowed.
- **No room beside the day's scenery, or no suitable land nearby.** The object is placed on the nearest suitable free cell, however far.
- **"Undo last dream"** (the dreaming page) restores memory and the profile. It does not remove what was drawn or its story; the user forgets a story separately.
- **The journey record is deleted or edited by hand.** Objects stay where they are. Stories the record no longer holds are not shown and are not worked out again. Unreadable lines are skipped and left untouched.
- **A forgotten story is still in the current context.** The app says the companion may still mention it until a fresh context starts, and offers to start one, as when a memory is forgotten.
- **A sensitive topic.** Keywords and lines appear only in tooltips, in the list and when an object is selected. They are never standing labels on the picture. The user can forget one.
- **The conversation is not in English.** Keywords and lines are in the language the user wrote in. Subject and item names come from the app's string table.
- **A touch screen, or no pointer.** The tooltip opens on tap and on keyboard focus, and closes on a second tap, on Escape, or when focus moves on.
- **The world has only its home.** The description given to the companion says so, and the makeup is empty.
- **An older version of the app opens this world.** It shows the objects it knows, shows the others as placeholders, and ignores what it does not know about their reasons.

## Requirements

### Functional requirements

**Naming what happened in the day**

- **FR-001**: During each dream sweep (the dreaming page), for each local day that has ended, had at least one message from the user, and has no topics yet, the companion MUST name up to three topics: what the user did, what happened to them, or what was on their mind that day, as the user told it (WLD-9).
- **FR-002**: Each topic MUST have a keyword of at most three words and 30 characters, one line of at most 120 characters in the companion's voice, and one or two subjects from the app's fixed list. A topic with an unknown subject, or with a keyword or line that is empty or too long, MUST be dropped. A day MAY end with no topic.
- **FR-003**: A topic MUST rest on something the user said that day about their own life. What only the companion said, and anything imagined or hypothetical, MUST NOT become a topic. A plan MUST be worded as a plan. Keywords and lines MUST be in the language the user wrote in (PER-8).
- **FR-004**: Topics MUST be named only from what dreaming is allowed to read: that day's real conversation and notes. Recalled-memory snippets, tool results, hidden app requests, the dream diary and forgotten content MUST NOT be used (DRM-6, CTL-2).
- **FR-005**: One sweep MUST name topics for at most the seven most recent such days. Older days MUST be left without topics and MUST NOT be revisited. Days before this feature was installed MUST NOT be given topics.
- **FR-006**: Naming topics MUST add at most one model request to a sweep and MUST read a bounded amount of each day. It MUST follow the sweep's rules: it never runs during a reply, pauses when the user sends a message, and never delays a reply (Principle V).
- **FR-007**: A sweep that is interrupted or fails while naming topics MUST leave the world and the journey record exactly as they were, and the day MUST be tried again at the next sweep while it is among the seven most recent. Writing starts only once every pick is known; if the app stops while writing, the next sweep MUST complete the writing, adding only what is missing and never removing anything (Principle I).
- **FR-008**: Topic drawing MUST be on by default and MUST have its own switch in settings. When it is off, no topics are named and nothing is sent to the model for the world.

**From topics to objects**

- **FR-009**: For each topic the app MUST draw one everyday object, picked by weighted chance. The model MUST NOT choose an item and MUST NOT produce a drawing.
- **FR-010**: The candidates for a pick MUST be the dictionary items filed under the topic's subjects and, with a lower chance, items filed under subjects the list marks as related to them. An item of an unrelated subject MUST NOT be picked.
- **FR-011**: An item's chance MUST fall each time that item has already been drawn in this world, and MUST fall further when it was drawn in the last seven days. The same rule MUST apply to scenery.
- **FR-012**: A pick MUST be decided only by the world's seed, the day, the topics and the world so far. The same four MUST always give the same result, so that picks can be tested exactly (NFR-TEST-1).
- **FR-013**: A topic object MUST be placed on land that suits it, beside the scenery drawn that same day when there is room, and otherwise as near to what is already drawn as a free place allows. It MUST NOT move or hide anything drawn earlier.
- **FR-014**: Stages (WLD-7) MUST apply to scenery only. Topic objects MUST be available from the world's first day.
- **FR-015**: A day MUST gain at most three topic objects and at most six items in all, scenery included. The number MUST NOT depend on how many messages were sent (WLD-5).
- **FR-016**: Scenery MUST continue to be drawn at the day's first message, with no model call, whether or not topics can be named.

**Everyday objects**

- **FR-017**: The dictionary MUST gain everyday objects: common things people use or meet in their daily activities, such as a cup, a laptop, a pair of running shoes, a suitcase or an umbrella. They MUST be drawn in the same style as the rest of the dictionary and follow its rules: a permanent id, a name from the string table, and no removal once shipped (WLD-3).
- **FR-018**: The app MUST ship a fixed list of subjects that covers everyday activities and occasions, including one catch-all subject. Each subject MUST have at least three everyday objects. An item MAY belong to several subjects. The list MUST say which subjects are related. A subject, once shipped, MUST NOT be removed or change its meaning.

**The journey record**

- **FR-019**: For each day given topics, the app MUST record, in the workspace beside the world file and in a human-readable form: the day, each topic's keyword, line and subjects, and for each pick the candidates with their chances, the item chosen, and whether it came from a related subject.
- **FR-020**: The line of each topic object in the world file MUST say that it was drawn for a topic, and for which one.
- **FR-021**: The app MUST only add to the journey record, except to carry out a forget request. Edits the user makes outside the app MUST be honored and never overwritten. Unreadable lines MUST be skipped and left untouched (Principle I).
- **FR-022**: A day MUST be either fully drawn and recorded, or not at all.
- **FR-023**: The journey record MUST be part of the workspace for every purpose: backup, move, and later export and wipe.

**Showing the diary on the map**

- **FR-024**: Pointing at any object in the picture, or moving keyboard focus to it, MUST show a tooltip with the object's name, the date it was drawn, and the keywords of that day, with the object's own keyword first. For a day without topics the tooltip MUST show the name and date only.
- **FR-025**: The tooltip MUST also open on tap, MUST stay while the pointer is on it or its object, MUST close on Escape without moving the pointer or focus, and MUST be read by a screen reader. Keywords MUST NOT be shown as standing labels on the picture.
- **FR-026**: The list in the world view MUST become a diary: one entry per day, newest first, with each topic's keyword and line beside the object drawn for it, and the scenery drawn that day. Days without topics MUST list their items as before.
- **FR-027**: Selecting a topic object in the picture MUST show its name, its date, its keyword and its line.
- **FR-028**: For each topic object the view MUST be able to show why it was picked: the recorded candidates and chances, the item chosen, and whether it came from a related subject.
- **FR-029**: Each day's entry MUST offer a way to open that day's notes in the memory browser, when notes for that day exist.
- **FR-030**: The view MUST offer a time control that shows the world as it was at the end of any past day and can play forward to today. It MUST work with the keyboard, and MUST step without animation when reduced motion is on.
- **FR-031**: The view MUST show what the world is made of: the share of each subject among its topic objects, as text and as a picture. This mix is worked out from the world file and the record whenever it is needed and MUST NOT be stored.
- **FR-032**: The companion's line at the top of the view MUST say what happened on the most recent day with topics and what was drawn for it, using only what the record holds (PER-8).
- **FR-033**: The view MUST no longer show the companion's id. The id stays recorded and keeps seeding colours, land and chance, so no existing world changes. The view's plain explanation of how the world grows MUST say that objects for a day are drawn during the following night's dream.
- **FR-034**: Everything this feature adds to the view MUST be usable with the keyboard alone and with a screen reader, MUST NOT rely on colour alone, and MUST take its fixed text, including subject and item names, from the app's string table. Keywords and lines MUST be shown as plain text (NFR-A11Y-1, NFR-I18N-1).
- **FR-035**: A problem reading the journey record MUST NOT stop the picture or the list of items from showing, and MUST NOT affect the conversation.

**The companion knows its world**

- **FR-036**: Every new context MUST give the companion a description of its world within a budget of 1,200 characters: how much is drawn and since when, what lies in each direction from the home, the most recent topic objects with their keywords and lines, and the makeup (WLD-10).
- **FR-037**: The description MUST NOT change during a context, and a rebuilt request MUST equal the one that was sent. Contexts begun before this feature MUST continue unchanged (Principle III).
- **FR-038**: From the first message after a sweep that drew topic objects, the companion MUST know what it drew and why. Telling it MUST NOT change anything earlier in the conversation.
- **FR-039**: The companion's rules MUST tell it that this is its own world, which it drew from the user's days, and that it may state about it only what it was given.
- **FR-040**: The description MUST NOT include forgotten stories.

**Taking an entry back**

- **FR-041**: Users MUST be able to forget a topic from the journey. Forgetting MUST ask for confirmation, showing the keyword and the line. Once confirmed, the keyword, the line and the subjects MUST be removed from the record, the object MUST stay and be shown without a story, and the story MUST NOT appear in any tooltip or any later context (CTL-2, Principle IV).
- **FR-042**: When the user forgets a memory (the memory page), the topics of the day that memory came from MUST be removed in the same way.
- **FR-043**: If a forgotten story is part of the current context, the app MUST say that the companion may still mention it until a fresh context starts, and MUST offer to start one.

**No pressure, privacy, structure**

- **FR-044**: Drawing topic objects MUST NOT produce notifications, badges or reminders. The app MUST NOT show a countdown to the next sweep and MUST NOT offer to draw a pick again (NFR-SEC-6, PER-8).
- **FR-045**: This feature MUST send nothing anywhere but the provider the user chose, and MUST add no download (Principle II).
- **FR-046**: Naming, picking, placing, recording, describing and forgetting MUST be decided in the core and MUST be testable with the scripted provider and an injected clock (Principle VI).

### Key entities

- **Topic**: One thing from the user's day, as the user told it: something they did, something that happened to them, or something on their mind. It has a keyword, a short line in the companion's voice, and one or two subjects. Up to three per day.
- **Keyword**: At most three words that name a topic, such as "10 km run" or "sister's birthday". Shown in tooltips.
- **Subject**: One of a fixed list of everyday activities and occasions (work, a run, cooking, a birthday and so on) that topics and dictionary items are both filed under, with a note of which subjects are related. Not the same as a dreaming theme, which is a pattern across days.
- **Everyday object**: A dictionary item that depicts a common thing from daily life and is filed under one or more subjects.
- **Topic object**: An everyday object drawn for a topic. Its line in the world file names the topic it was drawn for.
- **Scenery**: An item drawn at a day's first message with no model call, as in the world page (there called ambient). It forms the land the diary sits on.
- **Pick**: One weighted-chance choice: the candidates with their chances, the item chosen, and whether it came from a related subject.
- **Journey record**: The day-by-day account of topics and picks, kept in the workspace beside the world file.
- **Makeup (the world's DNA)**: The share of each subject among a world's topic objects: what the user's days have been made of. Read from the world, never stored.
- **Seed**: The random value a world starts from, today the companion's id. It decides colours, the lay of the land, taste and the roll of chance, and is no longer shown.
- **World description**: The short text about its world that the companion is given at the start of a context.

## Success criteria

- **SC-001**: In a scripted set of 20 days of conversation, each with one clear thing the user did, the first topic named for the day is that thing, with a fitting keyword and subject, on at least 18 days (checked with the real provider before release).
- **SC-002**: In a scripted set of 20 days in which the companion suggests things the user does not do and the two discuss imagined events, 0 topics come from those (checked with the real provider before release).
- **SC-003**: After 30 simulated days of the same single subject, no item makes up more than 40% of the topic objects, at least 5 different items appear, and 0 come from an unrelated subject.
- **SC-004**: For the same world, day and topics, 100 repeated runs produce the same picks and the same record, byte for byte.
- **SC-005**: For 200 sampled picks, the chances in the record match a recomputation from the world as it was, in 100% of cases.
- **SC-006**: Interrupting a sweep at any step before writing leaves the world file and the journey record byte for byte as they were, and stopping the app at any point while writing leaves days that the next sweep completes with no line twice and none removed, in 100% of attempts.
- **SC-007**: No day gains more than 3 topic objects or more than 6 items, across a simulated year that includes days with 500 messages.
- **SC-008**: A message sent while topics are being named starts its reply with the same 0.15 seconds of the app's own work as without it (NFR-PERF-2), and a sweep makes at most one more model request than it does without this feature.
- **SC-009**: With the provider unreachable for three nights, chat and the world view work, no error about the world appears, and the days get their topics at the first sweep after it returns.
- **SC-010**: 100% of the objects in the picture show a tooltip with their date on hover, on keyboard focus and on tap, and every object from a day with topics shows that day's keywords. A user can find what any object stands for in one action and why it was picked within two.
- **SC-011**: In 10 scripted fresh contexts, the companion states what it drew most recently and for what, matching the record, in at least 9. The description stays within 1,200 characters for worlds of 10 and of 5,000 items.
- **SC-012**: Within a context, the requests already sent are unchanged by world growth, in 100% of a scripted run.
- **SC-013**: After a topic is forgotten, its keyword and line appear in 0 places: the tooltips, the list, the record, and the description given to later contexts.
- **SC-014**: Using only the keyboard and a screen reader, a user can reach each object's tooltip and each day's keywords, lines, objects and chances, and operate the time control.
- **SC-015**: After ten years of daily conversation the journey record is smaller than 5 MB.
- **SC-016**: During a capture of a session that includes a sweep with topic drawing, the app's connections go only to the provider, and the world produces 0 notifications or badges.

## Assumptions

- **Depends on the world and dreaming pages.** The world, its file, the dictionary, scenery and the world view come from the world page. The nightly sweep, its catch-up, its pausing for chat, what it may read, its switch, its model setting, "dream now" and the journal's token counts come from the dreaming page. Forgetting and the memory browser come from the memory page, and new contexts from the chat and compaction pages.
- **The day ends with the dream.** "End of the day" is the night's dream sweep (03:00 by default), so a day's topic objects appear the next morning.
- **The diary is what the user told.** The companion only knows the user's day from the conversation. A day the user did not talk about, or did not describe, leaves no topic. Nothing is taken from outside the conversation.
- **The companion names, the app picks.** The model names topics, gives them keywords and files them under subjects. Which object is drawn is the app's weighted chance. The dictionary is therefore never sent to the model, and the choice can be tested exactly. This replaces WLD-9's wording that the model chooses item ids.
- **The keyword carries the truth, the object carries the picture.** Because chance picks among several fitting objects, an object is a likeness of the day and not an exact record. The keyword and the line say what really happened.
- **Topics do not replace scenery.** Scenery keeps the world growing on days without topics and forms the land between the diary's objects. Its tooltips show its day's keywords too, so the whole map reads by day.
- **Starting values**, to be tuned in planning: 2 or 3 scenery items at the first message; up to 3 topics a day; a chance divided by one plus the number of times the item was already drawn, and halved again if drawn in the last seven days; a quarter of the chance for items of related subjects; the five likeliest candidates recorded per pick, with the rest as one remainder; a keyword of at most three words and 30 characters; a line of at most 120 characters; a world description of at most 1,200 characters.
- **Starting list of subjects**, about 40, to be tuned in planning: work, a meeting, study, an exam, the commute, driving, a trip, a walk, a run, cycling, swimming, sport and the gym, cooking, eating out, coffee and tea, shopping, chores, sleep and rest, illness and the doctor, family, children, friends, love, a pet, reading, writing, music, a film or show, a game, making things, gardening, a call, money, the home, a celebration, a wedding, a loss, reflection, the weather, and something else.
- **The dictionary has to grow a lot.** the world page's items are scenery. This feature needs about 100 everyday objects in the same style, so that each subject has at least three. This is the largest single cost of the feature. The art is the project's own.
- **The id becomes a hidden seed.** It stays in the identity file and the world file, so nothing drawn changes. It is no longer shown, and "DNA" now means what the world is made of. This revises WLD-2 and WLD-8 and world FR-003 and FR-030.
- **A description, not the map.** The companion is given a short description of its world, not a character grid of every cell. A grid of a year-old world would cost thousands of tokens in every context and is hard for a model to read. A compact code of the whole map, with a short code per item and its grid cell, is the right form for sharing a world and is left for that later feature.
- **Stories are not memories.** A keyword or line is never used as a memory source, is never promoted to long-term memory, and is not input to later dreams. The day's notes stay the diary's text; the journey links to them.
- **Out of scope**: friends between companions and any connection to another device; a public key as the companion's address; a compact code of the world for sharing; the user choosing, placing or adding objects or keywords by hand; drawing during the conversation itself; any change to the companion's own look; saving the world as a picture.
- **Product documents to update with this feature's plan.** `docs/requirements.md` (WLD-2, WLD-3, WLD-8, WLD-9, the name and dependencies of slice 10, and decisions on the diary idea and on who picks the object) and `docs/architecture.md` (the journey record in the workspace layout, the world description in the request layout). No change to the principles is needed: the record lives in `world/`, which Principle I already covers, and nothing new leaves the machine.
- **Platform.** Windows 11 first, as for the rest of the app.

## Clarifications
### Session 2026-11-14

- Q: Should the day's topics be named by an end-of-day job of its own, or in the nightly dream sweep? → A: In the dream sweep of the dreaming page. The feature reuses dreaming's schedule, catch-up, pausing for chat, switch and model setting, and waits for dreaming to be built. A separate job is not built.
- Q: What is the world for? → A: It is a visual map of the user's diary. It reflects what really happened to the user, as they told the companion. Objects are common things from daily activities, the dictionary gains many of them, and every object has a tooltip with the keyword of its day.
- Q: Should the diary draw only what happened, or also what was on the user's mind, such as plans and worries? → A: Both. A plan or a worry is worded as one and never as something that happened (FR-001, FR-003).

## Decisions

Each section records a decision, why it was made, and what was rejected. The design builds on the world page (the world file, the dictionary, the seeded generator, the world view), the dreaming page (the dream sweep, its checkpoints, pausing, the dreaming model and the run's usage), the memory page (forgetting, the memory browser, file notices) and the chat and compaction pages (context entries and the request layout).

### R1. Where the day's topics are named

**Decision**: A new phase of the dream sweep, `world`, after the diary and before the roll: `light → rem → deep → committing → diary → world → roll → done`. The diary's early exit for a night without material goes to `world` instead of `done`, because days from earlier nights may still be waiting. The phase sends one request for every day that is due (R2), with the dreaming model at low effort, through the sweep's `ask`, so a message stops it (`Paused`, resumed at `world`), and its usage is stored in the run as `world`.

When the request fails (unreachable, refused, key rejected), the phase writes nothing and moves on; the run still completes, and the days stay due for the next sweep (FR-007). When topic drawing is off (R14), the phase is skipped without a request (FR-008).

**Rationale**: The clarification puts the day's end in the dream sweep, which already has the schedule, catch-up, idleness, pausing for chat, the model setting and "dream now" (FR-006). After the diary, the memory part of the sweep is finished and committed, so a world failure can never touch memory. Before the roll, a sweep that also rolls the context gives the new context the world as it now is (R9).

**Alternatives considered**: Before the light phase (a failure there would delay memory consolidation, and the roll would still be needed). Inside the REM request (one request fewer, but it mixes two outputs with different rules and inputs and makes the REM contract harder to test). A job of its own (rejected by the clarification).

### R2. Which days get topics

**Decision**: A day is **due** when all of these hold:

- it is before the local date of the sweep's clock (today has not ended, so "dream now" during the day never names today);
- it is on or after `state.json world.journeySince`, the local date when `prepare_world` first ran with this feature, so days before the feature never get topics (FR-005);
- it is later than `world.journeyDay`, the newest day already considered (the newest day in the journey record when the state has none);
- it has at least one message the user typed (provenance `user`, not forgotten), found from the messages' time blocks (`date_of_time_block`).

Of the due days, the seven most recent are sent. When the model answers, every day it was asked about gets a journey line, an empty one if it named no topics, and `journeyDay` moves to the newest of them. Older days that missed the batch are then behind `journeyDay` and are never revisited. When the request fails, nothing changes: the days are asked again at the next sweep while they are among the seven most recent. Neither field ever moves back. Without `journeySince` (the world view has never opened), no day is due: the journey has not begun.

**Rationale**: A batch always covers the newest due days, so everything at or before `journeyDay` has been considered or was left out on purpose. A day whose line the user deleted by hand is behind `journeyDay` too, so its story is not worked out again and its objects are not drawn twice (Edge cases). Moving `journeyDay` only on success keeps FR-005's "not revisited" without losing days to an unreachable provider (SC-009).

**Alternatives considered**: "No journey line" as the test (a hand-deleted line would be redrawn past the daily cap). Keeping a list of done days in `state.json` (grows forever). Using the sweep's `since` (a failed night would be lost).

### R3. The topics request

**Decision**: A structured-output request (the client's existing `format`) with:

- **system**: the companion's identity and soul block (as the diary does) and the topics instructions (`world/prompts/topics.md`): name up to three things per day that the person said they did, that happened to them or that were on their mind; only what the person said about their own life; never what only the companion said, nothing imagined or hypothetical; a plan or worry worded as one; keyword and line in the person's language; the line in the companion's voice, addressed to the person.
- **user**: for each day, newest first, the date and what the person wrote that day: their messages in time order, each cut to 400 characters, at most 3,000 characters a day. When a day has more, the first message is always kept and the rest are taken evenly across the day. Then that day's daily-note lines written with provenance `user` or `companion` (not forgotten), at most 1,000 characters. Companion replies, recalled-memory blocks, tool results, system and untrusted content, the diary and forgotten messages are never included (FR-004).
- **schema**: `{days: [{day, topics: [{keyword, line, subjects: [subject id]}]}]}`, with the subject ids as an enum and their English names in a description, `maxItems` 3 for topics and 2 for subjects.

The answer is validated in code (FR-002): a day not asked about is dropped; topics beyond three are dropped; each topic needs one or two distinct known subjects, a keyword of one to three words and 1 to 30 characters, and a line of 1 to 120 characters on one line, after trimming and removing control characters and markup brackets. A topic that fails is dropped.

**Rationale**: Only the person's own words go in, so what only the companion said cannot become a topic by construction (FR-003, SC-002). Notes add what a compaction flush or the companion wrote down from a long day. Spreading the sample across the day keeps a 500-message day bounded (edge case "a very long day", SC-007) without favouring its end. The subject list is short enough for an enum. Validation in code makes the model's answer untrusted input, as everywhere else.

**Alternatives considered**: Replaying the day's whole conversation (unbounded, and includes the companion's own suggestions). Sending the dictionary and letting the model choose items (rejected in the Assumptions: the app picks, R5).

### R4. Subjects and everyday objects

**Decision**: Two data files in `crates/steve-core/src/world/`:

- `subjects.json`: the fixed list, about 40, each `{"id": "running", "related": ["walking", "sport"]}`. Ids are lowercase words joined by hyphens. One is the catch-all `other`. Names come from the webview's string table and from an English table in the core for the model.
- `items.json`: everyday objects are added with `"subjects": [...]` and the new land kind `ground` (any land but water); a few use `water`. Their stage is 1, and stages don't apply to them (FR-014).

Scenery picks only items **without** subjects, and topic picks only items **with** subjects, so an everyday object never appears without a day's topic and a tree never stands for one. A unit test checks that every subject has at least three items, every `related` id and item subject exists, no subject is related to itself, and no shipped subject or item id is removed (fixtures `shipped-ids.txt` and a new `shipped-subjects.txt`).

The drawings are about 100 new 32 × 32 SVGs in `src/assets/world/`, in the existing flat style. They use the six palette variables, plus two fixed neutrals added to the palette style, `--w-light` and `--w-dark`, for things that are white or black in life (a cup, a laptop screen).

**Style (changed on 2026-11-14 at the owner's request)**: every world item, scenery included, is drawn as pixel art in a cartoon style: a 16 × 16 grid of 2-unit squares (`shape-rendering="crispEdges"`), flat palette colours, and a dark outline (`--w-dark`) around each shape. The existing drawings were converted by sampling each one onto the grid, snapping every square to the nearest palette colour and adding the outline, so all 171 items share one style and still take the companion's palette.

**Rationale**: Data, not code (lean-code), like the existing dictionary. Keeping the two pools apart keeps the map truthful (the refinement). `ground` lets everyday objects stand anywhere dry, which placement beside the day's scenery needs.

**Alternatives considered**: Tagging scenery with subjects too (a flower for "gardening" would blur scenery and diary). Free colours per drawing (breaks the companion's palette).

### R5. The pick

**Decision**: For a topic with subjects S, the candidates are the items filed under any subject in S (direct) and the items filed under a subject that one of S marks as related but under none of S (related). An item's weight is:

```text
preference(seed, item)          0.5 to 2.0, the world page's taste
× 1 / (1 + times drawn)          in this world so far, including earlier picks of the same night
× 0.5 if drawn in the last 7 days (on or after the topic's day minus 6)
× 0.25 if related
```

Its chance is its weight over the total. The roll uses the world page's SplitMix64 seeded with `seed(companion id, "topic/<day>/<n>")`, where n is the topic's position that day. The same rule (times drawn and the last seven days) now multiplies scenery weights too (FR-011), and scenery becomes 2 or 3 items a day (FR-015). The record keeps the five likeliest candidates with their chances, rounded to three decimals, the drawn item's own chance when it is not among them, and the rest as one share.

**Rationale**: The seed, the day, the topics and the world so far decide everything (FR-012, SC-004), so a pick can be recomputed and checked (SC-005). Dividing by the count spreads a steady subject over its items (SC-003), and the related quarter adds variety without ever leaving the topic's neighbourhood (FR-010). Reusing the world page's generator and taste keeps one source of randomness per world.

**Changed behavior**: Scenery from now on draws 2 or 3 items and counts repeats, so `tests/fixtures/world/golden.jsonl` is regenerated. Worlds already drawn don't change, because the file stores what was drawn (WLD-4).

**Alternatives considered**: Uniform choice among fitting items (a long run repeats too often). Excluding items already drawn (a subject's items run out). Recording every candidate (the record grows past SC-015's limit).

### R6. Placement

**Decision**: A topic object goes on a free cell whose land suits it (`ground`: not water; `water`: water), chosen in this order:

1. Free cells within two cells of an item drawn that same day (its scenery, and its earlier topic objects), nearest to those items first.
2. Otherwise, rings around every drawn item at growing distance, nearest first, until a suitable free cell is found (the land around a world always has both kinds within a few rings; the search stops at 64 rings and falls back to the nearest free cell of any land).

Ties break by distance to the home, then y, then x. Size, flip and shade come from the pick's generator. The line's `day` is the topic's day, not the night it was drawn, and `why` is `topic:<day>/<n>`. Nothing drawn earlier is moved (FR-013).

**Rationale**: A day's objects stand together, which is what makes the map read as a diary (FR-013). The ordering is deterministic. the world page's `REACH` of two cells is kept.

### R7. The journey record

**Decision**: `world/journey.jsonl`, appended by the app, one line per day it considered:

```json
{"day":"2026-11-10","topics":[{"keyword":"10 km run","line":"You ran ten kilometres before work.","subjects":["running"],"item":"shoes.running","related":false,"odds":[["shoes.running",0.312],["bottle.water",0.204],["watch.sport",0.18],["cap.sport",0.11],["towel.small",0.09]],"rest":0.104}]}
```

A forgotten topic becomes `{"item":"shoes.running","forgotten":true}`, keeping its place so that `topic:<day>/<n>` still points at the right entry. Unreadable lines are counted and skipped; lines for the same day after the first are ignored. Only forgetting rewrites the file, through the memory writer with its hash check, so an outside edit made since the view was read is never overwritten (FR-021).

Size: about 350 bytes a topic; three topics every day for ten years is about 4 MB (SC-015).

**Rationale**: Beside `world.jsonl` and in the same append-only JSON Lines form (FR-019, FR-023), so backup and move already include it. A line per day makes "a day is done" one lookup, and forgetting one rewrite.

**Alternatives considered**: Putting the story in the world file's lines (a forget would rewrite the world file, and the world file's lines would grow past the world page's 200-byte target). A table in `steve.db` (not a workspace file, against Principle I).

### R8. All or nothing

**Decision**: The `world` phase works in two steps. After the answer is validated, it works out every due day's picks, world lines and journey lines in memory, saves them in the run's checkpoint (`drawing`), and sets the phase to `drawing`. The `drawing` phase appends each planned world line that the world file doesn't already hold, then each planned journey line likewise, sets `state.json world.journeyDay`, and moves on. A run found in `drawing` when a sweep starts (the app stopped between or during the appends) simply runs `drawing` again: it rolls forward, appending only what is missing, and never removes a line.

**Rationale**: Principle I lets the app only append to `world/`, so an interrupted write is completed, not undone. Nothing is written before the model has answered and every pick is known, so a message (stop signal) can only interrupt before any write, leaving both files as they were (SC-006). A crash in the moments between two appends leaves objects whose stories appear at the next sweep; the view shows such objects without a story until then. Checking for each planned line before appending makes the step safe to rerun (Principle V) and keeps scenery that `grow_today` appended in between.

**Alternatives considered**: Removing the planned lines on recovery, as the dreaming page's commit rolls back (the app would remove lines from `world/` on its own, against Principle I). Truncating to recorded lengths (also a removal, and would drop scenery appended meanwhile). A temporary file per day (two files still can't be renamed together).

### R9. What the companion knows of its world

**Decision**: A **world description** of at most 1,200 characters, worked out by the core from the identity, the world file, the journey and the land when a context begins, and saved in the `context` entry as `world: {text, through}` (`through` is the newest journey day it covers). `system()` puts it after `SOUL.md` in the identity block, under `## Your world`. Its parts, in English with keywords and lines as written:

1. how many things are drawn and since when (the birth date);
2. what lies in each direction from the home (north, east, south, west): the land that is most common there and up to two topic objects, by item name (the English name is the id read backwards, `shoes.running` → "running shoes"; a home by its name alone, "cottage");
3. the five most recent topic objects with their day, keyword and line;
4. the makeup: the four largest subjects with their shares.

When it is too long, the oldest of the recent objects go first, then the directions' objects. A world with only its home says so. Contexts begun before this feature have no `world` and keep their request unchanged (FR-037).

**Rationale**: The system prompt is fixed for a context (Principle III) and already holds what the companion knows about itself. A description reads better and costs less than a map (Assumptions). Saving it in the context entry makes a rebuilt request equal the sent one, as for the notes snapshot.

**Alternatives considered**: Rebuilding it per request (breaks Principle III). Putting it in the opener (absent from contexts that have no summary, dream or notes).

### R10. Telling the companion what it drew overnight

**Decision**: When the user sends a message and the current context has a `world` snapshot, the core compares `state.json world.journeyDay` with the newest day the context has been told (its snapshot's `through`, or a later message's `world` field). When the journey is newer, the message carries a notice before the user's text:

```text
<notice>While you dreamt, you drew in your world: running shoes for "10 km run" (Tuesday 10 November): You ran ten kilometres before work.</notice>
```

The message entry records `world: "2026-11-10"`, the day it told through. Forgotten topics are left out. A context without a snapshot gets no notice.

**Rationale**: A notice in the next user message is how the companion already learns of an edited file without changing anything earlier (the memory page). It adds no model request to the sweep (FR-006, FR-038), unlike rolling the context after every night with topics. Checking a state field keeps the send path free of file reads on most messages (SC-008).

**Alternatives considered**: Always rolling the context after topics (adds the compaction's two requests). Changing the system prompt (breaks the prefix).

### R11. The companion's rules

**Decision**: The harness gains a paragraph: the world is the companion's own, drawn from the person's days while it dreams; it may say about it only what its world description and notices tell it; the keywords are the person's private matters, to be mentioned with care. The new text has version `world-1`; the current text stays as `harness-memory-1.md` for contexts that recorded `memory-1` (FR-037, FR-039). A context counts as outdated only when it predates memory (`chat-1`); a `memory-1` context carries on silently under its own rules.

### R12. Forgetting a story

**Decision**:

- **`forget_topic(day, index, baseHash)`** rewrites that day's journey line with the topic replaced by `{item, forgotten: true}`, after the hash check. The world line stays. It returns whether the keyword or line is in what the current context holds (its world snapshot or a world notice), so the webview can offer a fresh context (FR-041, FR-043).
- **Forgetting a memory** (the memory page) also forgets every topic of the days it touches: the date of a daily note when a line or the whole note is forgotten, and the days of an entry's sources (its `src` notes, and the time blocks of its source messages) when a `MEMORY.md` entry is forgotten (FR-042). `ForgetOutcome.inCurrentContext` covers both.

**Rationale**: The record stays append-only except for forgetting, as FR-021 allows. Keeping a placeholder keeps the indexes of the day's other topics and the world lines' references stable.

### R13. The world view

**Decision**: `world_view` returns everything the view needs, worked out by the core (FR-046):

- each item gains `topic` (`day` and `n` when drawn for a topic) and `keywords` (its day's keywords, its own first);
- `journey`: one entry per day, newest first, with the topics (keyword, line, subjects, the drawn item's index, the pick), whether a note exists for that day, and the indexes of the day's scenery;
- `makeup`: subject and share, largest first, each topic object counting one, split evenly over its topic's subjects; forgotten topics don't count;
- `companionId` is removed from the view type (FR-033).

The webview adds: a tooltip for every item on hover, keyboard focus and tap (dismissable with Escape, hoverable, `role="tooltip"` and `aria-describedby`, WCAG 1.4.13); the diary list; a "why this one" disclosure with the odds as a table; a "notes" link that opens the memory browser at `memory/<day>.md`; a time control (a range over the distinct days, a play button, arrow keys; with reduced motion the steps change without transition); the makeup as a text list and a bar whose segments are labelled; the companion's line from the newest day with topics.

**Rationale**: The rules (what a tooltip shows, the makeup) are decided once in the core and tested headless. The view already receives every item; a decade's journey is a few megabytes, read only when the view opens (FR-035: a journey that can't be read gives an empty journey and the picture still shows).

**Alternatives considered**: A separate command for the odds (a second round trip for a few kilobytes). Computing the makeup in the webview (a rule outside the core).

### R14. Settings

**Decision**: `settings.json dreaming.world` (default `true`), shown in the dreaming section of settings as "Draw my days in Steve's world". `DreamingSettings` gains `world`, and `set_dreaming_settings` takes it.

**Rationale**: Topic drawing is part of the sweep and shares its switch's section. One field on an existing command, not a new command (lean-code).

### R15. Product documents

**Decision**: Updated with this plan: `docs/requirements.md` WLD-2, WLD-3, WLD-8, WLD-9 and WLD-10, slice 10's name and dependencies, and two decisions (D14: the world is a map of the user's diary; D15: the companion names topics and the app picks the object). `docs/architecture.md` sections 4.1 (the journey record), 4.3 (its write rule), 5.2 (the world description in the identity block), 8.2 (the world phase).

## Data

What this slice adds to the workspace, the bundled data, the settings and state files, the dream run's checkpoint, the conversation file, and the types that cross to the webview.

### Data 1. Workspace files

#### `world/world.jsonl` (the world page, changed)

Topic objects are appended with `why` naming their topic, and `day` set to the topic's day (R6):

```json
{"item":"shoes.running","x":3,"y":-1,"size":0.9,"flip":false,"shade":1,"day":"2026-11-10","why":"topic:2026-11-10/0"}
```

| Field | Rules |
|---|---|
| `why` | `ambient` for scenery, as before; `topic:<YYYY-MM-DD>/<n>` for a topic object, n being the topic's position (0 to 2) in that day's journey line. |
| `day` | For a topic object, the day of its topic, which is earlier than the night it was drawn. |

A day holds at most 3 topic objects and at most 6 items (FR-015). Older app versions read the line as before; an item id they lack is a placeholder (world FR-015).

#### `world/journey.jsonl` (new)

One line per day the companion considered, appended in day order of each sweep (R7):

```json
{"day":"2026-11-10","topics":[{"keyword":"10 km run","line":"You ran ten kilometres before work.","subjects":["running"],"item":"shoes.running","related":false,"odds":[["shoes.running",0.312],["bottle.water",0.204],["watch.sport",0.18],["cap.sport",0.11],["towel.small",0.09]],"rest":0.104}]}
{"day":"2026-11-11","topics":[]}
{"day":"2026-11-12","topics":[{"item":"cake.birthday","forgotten":true}]}
```

| Field | Rules |
|---|---|
| `day` | `YYYY-MM-DD`, the user's local date. The first line for a day counts; later ones are ignored. |
| `topics` | 0 to 3, in the order named. Empty: the day was considered and has no topic. |
| `keyword` | 1 to 3 words, 1 to 30 characters, plain text, the user's language. |
| `line` | 1 to 120 characters, one line, plain text, the companion's voice. |
| `subjects` | 1 or 2 distinct ids from `subjects.json`. |
| `item` | The dictionary id drawn for the topic. |
| `related` | Whether the item came from a related subject only. |
| `odds` | The five likeliest candidates as `[item, chance]`, chance rounded to 3 decimals, likeliest first, then the drawn item when it is not among them; fewer when there were fewer candidates. |
| `rest` | The combined chance of the other candidates, 3 decimals; 0 when there were none. |
| `forgotten` | `true` for a forgotten topic, which then has only `item` (FR-041). |

Rules: appended only, except that forgetting rewrites the one day's line through the memory writer with its hash check. Unreadable lines are skipped, counted, and left in place. Never a memory source, never indexed, never dream input (Assumptions).

### Data 2. Bundled data (`crates/steve-core/src/world/`)

#### `subjects.json` (new)

```json
[
  {"id": "running", "about": "running or jogging", "related": ["walking", "sport"]},
  {"id": "other", "about": "anything that fits nowhere else", "related": []}
]
```

About 40 subjects (Assumptions). `id`: lowercase words joined by `-`, permanent. `related`: ids from the same list, never itself. One `other`. A shipped subject is never removed and never changes meaning (FR-018). `about`: what the subject covers, in English, given to the model in the schema. The description names a subject by its id with `-` read as a space; the webview's names live in `strings.ts`.

#### `items.json` (changed)

```json
{"id": "shoes.running", "land": "ground", "stage": 1, "subjects": ["running", "sport"]}
```

| Field | Rules |
|---|---|
| `land` | Gains `ground`: any land but water. |
| `subjects` | New, optional. Present only on everyday objects; each id exists in `subjects.json`. |

Every subject has at least three items (FR-018). Scenery is drawn only from items without `subjects`; topics only from items with them (R4). Items have no English name field: the core reads the id backwards for the world description (`shoes.running` → "running shoes", R9).

### Data 3. `settings.json`

```json
{"dreaming": {"enabled": true, "time": "03:00", "model": null, "world": true}}
```

`dreaming.world`: default `true`. When `false`, the sweep's world phase sends nothing and draws nothing (FR-008).

### Data 4. `state.json`

```json
{"world": {"lastDay": "2026-11-14", "journeySince": "2026-11-14", "journeyDay": "2026-11-13"}}
```

| Field | Rules |
|---|---|
| `world.journeySince` | Set to the local date by `prepare_world` when absent, never changed after. Days before it never get topics (R2). |
| `world.journeyDay` | The newest day considered: set to the newest day of each batch the model answered, never moved back. Only later days are due (R2), and the send path compares it with what the context was told (R10). When absent, the newest day in the journey record. |

Both are bookkeeping; the journey record is the truth.

### Data 4a. Older files

Every field this slice adds to an existing file is optional: `why` was already a free string, the `context` and `message` additions and the settings and state fields default when absent, and `journey.jsonl` is new. Files written before this slice read unchanged and need no migration; older app versions ignore what they don't know (the principles, Security and data gates).

### Data 5. Dream run (the dreaming page, changed)

#### Phases

```text
light ─▶ rem ─▶ deep ─▶ committing ─▶ diary ─▶ world ─▶ drawing ─▶ roll ─▶ done
                                         │        │                  ▲
                                         │        └─(no due day, off, provider fails)──▶ roll or done
                                         └─(no material)─▶ world
drawing ──crash──▶ drawing again (only the missing planned lines are appended)
any phase ──message──▶ same phase (paused)
```

`roll` follows only when the deep phase committed, as before.

#### Checkpoint additions

```json
{
  "topics": [{"day": "2026-11-10", "topics": [{"keyword": "10 km run", "line": "…", "subjects": ["running"]}]}],
  "drawing": {"world": ["{…}"], "journey": ["{…}"]}
}
```

| Field | Written by | Rules |
|---|---|---|
| `topics` | `world` | The validated answer, every asked day present (empty days included). |
| `drawing` | `world` | The exact lines to append; a rerun appends the ones the files don't hold yet (R8). |

`dream_runs.usage` gains a `world` key.

### Data 6. Conversation file (`sessions/main.jsonl`)

| Entry | Change |
|---|---|
| `context` | Optional `world: {"text": "…", "through": "2026-11-10"}`: the world description given to this context and the newest journey day it covers (`through` absent when there is none). Absent for contexts begun before this feature. |
| `message` (user) | Optional `world: "2026-11-12"`: the message carries a world notice telling the companion about journey days through that date (R10). |

### Data 7. Types that cross to the webview (generated with `ts-rs`)

```rust
pub struct WorldView {
    pub born: Option<String>,
    pub palette: WorldPalette,
    pub items: Vec<DrawnItem>,
    pub journey: Vec<JourneyDay>,     // newest first
    pub makeup: Vec<SubjectShare>,    // largest first
    pub skipped_lines: u32,           // world file
    pub journey_skipped: u32,         // journey record
    pub journey_hash: String,         // base hash for forget_topic
    // companion_id removed (FR-033)
}

pub struct DrawnItem {
    /* the world page fields */
    pub topic: Option<TopicRef>,      // drawn for a topic
    pub keywords: Vec<String>,        // its day's keywords, its own first; empty for a day without topics
}

pub struct TopicRef { pub day: String, pub n: u32 }

pub struct JourneyDay {
    pub day: String,
    pub topics: Vec<JourneyTopic>,
    pub scenery: Vec<u32>,            // indexes into items
    pub has_notes: bool,              // memory/<day>.md exists
}

pub struct JourneyTopic {
    pub n: u32,
    pub drawn: String,                // item id
    pub item: Option<u32>,            // index into items; None when the world file no longer has it
    pub story: Option<Story>,         // None once forgotten
}

pub struct Story {
    pub keyword: String,
    pub line: String,
    pub subjects: Vec<String>,
    pub related: bool,
    pub odds: Vec<Odds>,
    pub rest: f64,
}

pub struct Odds { pub item: String, pub chance: f64 }

pub struct SubjectShare { pub subject: String, pub share: f64 }   // 0 to 1

pub struct DreamingSettings { /* the dreaming page fields */ pub world: bool }
```

`forget_topic` returns the memory page's `ForgetOutcome { in_current_context }`.

### Data 8. Constants (core)

| Name | Value | Source |
|---|---|---|
| Topics per day | at most 3 | FR-001 |
| Items per day | at most 6 (scenery 2 or 3) | FR-015, R5 |
| Days per sweep | at most 7 | FR-005 |
| Keyword | 1 to 3 words, at most 30 characters | FR-002 |
| Line | at most 120 characters | FR-002 |
| Day input | 3,000 characters of messages (400 each) and 1,000 of notes | R3 |
| Repeat factor | 1 / (1 + times drawn) | FR-011 |
| Recent factor | 0.5 within 7 days | FR-011 |
| Related factor | 0.25 | FR-010 |
| Odds recorded | 5, plus the rest | Assumptions |
| Placement reach | 2 cells; search up to 64 rings | R6 |
| World description | at most 1,200 characters; 5 recent objects; 4 subjects | FR-036, R9 |

## Interfaces

### World Journey Commands Between the Webview and the Core

Same rules as the pages onboarding, chat, memory, world, compaction and dreaming: thin handlers, validation in the core, expected failures as values, each command in `build.rs` and allowed in `src-tauri/capabilities/main.json`, and `src/api/commands.ts` as the only caller. The only network request this feature adds is the topics request inside the dream sweep (see the topics request below).

#### New command

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `forget_topic` | `day: string`, `n: number`, `baseHash: string` | `ForgetOutcome` | Replaces topic `n` of `day` in `world/journey.jsonl` with `{item, forgotten: true}`. `InvalidInput("topic")` when the day has no such topic or it is already forgotten; `InvalidInput("changed")` when the journey changed since `baseHash`, as the memory browser reports it. The world file is not touched. `inCurrentContext` is true when the topic's keyword or line is in the current context's world snapshot or in a world notice it carries. Refused as `InvalidInput("busy")` while a sweep runs. | FR-041, FR-043 |

#### Changes to earlier commands

| Command | Change |
|---|---|
| `world_view` | Returns the `WorldView` of Data 7: items with `topic` and `keywords`, `journey`, `makeup`, `journeySkipped`, `journeyHash`; no `companionId`. A journey that can't be read gives an empty journey, and the rest of the view still returns (FR-035). |
| `prepare_world` | Also sets `state.json world.journeySince` to the local date when it is absent (FR-005). |
| `dream_tick`, `dream_now` | The sweep gains the `world` and `drawing` phases (Data 5). The topics request's usage counts in the run's `tokens`. |
| `dreaming_settings` | `DreamingSettings` gains `world`. |
| `set_dreaming_settings` | Gains `world: boolean` (FR-008). |
| `memory_forget` | Also forgets the topics of every day the forgotten text came from (R12); `inCurrentContext` covers them (FR-042). |
| `send_message`, `retry_reply`, `regenerate_reply` | A new user message in a context with a world snapshot carries a world notice when the journey has days the context hasn't been told (R10, FR-038). |
| `start_fresh_context`, compaction, the after-dream roll | The new `context` entry carries the world description (FR-036). |

#### Webview duties

- The world view shows a tooltip for every item on hover, keyboard focus and tap: name, date, and the day's keywords, the item's own first. It stays while the pointer is on the item or the tooltip, closes on Escape, a second tap or when focus moves, and is the item's accessible description. Keywords are never drawn on the picture (FR-024, FR-025).
- The list is a diary: one section per day, newest first, each topic's keyword and line beside its item, then the day's scenery; "why this one" shows the odds as a table and whether the item came from a related subject; "notes" opens the memory browser at `memory/<day>.md` when `hasNotes` (FR-026 to FR-029).
- A time control picks any day from the first to today and plays forward; the picture shows the items whose `day` is on or before it. Arrow keys move it; with reduced motion nothing transitions (FR-030).
- The makeup shows as a list with percentages and a labelled bar (FR-031). The companion's line names the newest day with topics and what was drawn (FR-032). The id is no longer shown (FR-033).
- "Forget this story" on a topic asks for confirmation showing the keyword and the line, calls `forget_topic`, and when `inCurrentContext` offers `start_fresh_context` as the memory browser does (FR-041, FR-043).
- Settings show "Draw my days in Steve's world" in the dreaming section.

#### Capability file additions

`allow-forget-topic`.

### The Topics Request, the World Description and the World Notice

#### 1. The topics request

One `POST /v1/messages` per sweep at most, sent by the dream sweep's `world` phase through the existing Claude client (the dreaming page's `dream_request`), and only when topic drawing is on and at least one day is due.

| Field | Value |
|---|---|
| `model` | The dreaming model when it is in the cached list, otherwise the chat model (dreaming FR-026). |
| `output_config.effort` | `low`, when the model supports effort. |
| `output_config.format` | `json_schema` with the schema below. |
| `system` | Block 1: the identity and soul block of the companion's files (as the diary request). Block 2: `world/prompts/topics.md`. |
| `messages` | One user message: the days (below). |
| `tools` | None. |

##### Schema

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["days"],
  "properties": {
    "days": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["day", "topics"],
        "properties": {
          "day": {"type": "string", "description": "The date as given, YYYY-MM-DD."},
          "topics": {
            "type": "array",
            "maxItems": 3,
            "items": {
              "type": "object",
              "additionalProperties": false,
              "required": ["keyword", "line", "subjects"],
              "properties": {
                "keyword": {"type": "string", "description": "At most three words, at most 30 characters, in the person's language."},
                "line": {"type": "string", "description": "One sentence of at most 120 characters, in your voice, to the person."},
                "subjects": {"type": "array", "minItems": 1, "maxItems": 2, "items": {"type": "string", "enum": ["work", "meeting", "…", "other"]}, "description": "work: work or a job; …"}
              }
            }
          }
        }
      }
    }
  }
}
```

##### User message

```text
Days to look back on, newest first. Only what the person wrote is shown.

## 2026-11-10 (Tuesday)
What they wrote:
- 07:12 Just back from a 10 km run before work!
- 19:40 Baked a cake for my sister's birthday tomorrow.
Your notes from that day:
- 19:42 Sister's birthday is on 11 November.
```

Rules: days newest first; at most 7. Each message is cut to 400 characters with "…"; at most 3,000 characters of messages a day, the first message always kept, the rest taken evenly across the day in time order. Note lines with provenance `user` or `companion` only, their comments removed, at most 1,000 characters. Nothing else from the conversation.

##### Validation of the answer

A day not asked about, or asked about twice, is dropped after its first answer. For each topic: trim; remove control characters and `<`, `>`; the keyword must have 1 to 3 words and 1 to 30 characters; the line 1 to 120 characters with no line break; the subjects 1 or 2 distinct known ids. A topic that fails is dropped; topics after the third are dropped. A day asked about but missing from the answer counts as answered with no topics.

##### Outcomes

| Provider answer | Result |
|---|---|
| Valid reply | Every asked day gets a journey line (R2). |
| Stopped by a message | Paused at `world`; nothing written. |
| Unreachable, refusal, rate limit, key rejected, no credit | Nothing written; the run goes on to `roll` or `done`; the days stay due. |
| Unparsable JSON | As unreachable. |

Recorded fixtures: `crates/steve-core/tests/fixtures/messages/topics.sse` (written by hand, noted in the fixtures README until the owner records it).

#### 2. The world description

Saved in each new `context` entry (`world.text`) and placed in the system prompt's identity block after `SOUL.md`:

```text
## Your world

You have drawn 214 things since 2026-03-03, around your cottage.
North: mostly woods, with open laptop and desk lamp. East: mostly meadow, with birthday cake. South: mostly rocks. West: mostly meadow, with running shoes.
Most recently you drew:
- birthday cake for "sister's birthday" (2026-11-10): You baked a cake for your sister's birthday.
- running shoes for "10 km run" (2026-11-10): You ran ten kilometres before work.
Your world is mostly made of: work 34%, running 21%, cooking 12%, family 9%.
```

Rules: at most 1,200 characters; an item named by its id read backwards (`shoes.running` → "running shoes", a home by its name), a subject by its id with `-` as a space; dates as `YYYY-MM-DD`; the recent objects newest first; keywords and lines as written, in quotes; forgotten topics never appear; a world with only its home reads "Your world is just your <home> so far." When over budget, the oldest recent objects go first, then the directions' objects. Directions are the four quarters around the home (by angle); "mostly" is the commonest land among drawn items in that quarter; up to two topic objects per quarter, the most recent first.

#### 3. The world notice

Placed before the user's text in the next user message of a context that has a world snapshot, when `state.json world.journeyDay` is later than what the context has been told:

```text
<notice>While you dreamt, you drew in your world: running shoes for "10 km run" (2026-11-10): You ran ten kilometres before work. birthday cake for "sister's birthday" (2026-11-10): You baked a cake for your sister's birthday.</notice>
```

Rules: the topics of the journey days later than the context's newest told day, oldest first, forgotten ones left out; at most 1,200 characters, keeping the newest; no notice when nothing is left. The message entry records `world` as the newest day told.

## Checks

### Prerequisites

- The pages onboarding, chat, memory, world, compaction and dreaming set up, or a clean machine to run the wizard.
- Rust 1.98 or later, Node.js 22 or later, pnpm 10.
- A Claude API key for the manual scenarios and the release checks.
- To see topics quickly in manual runs: tell Steve about your day, set the system date forward a day (or wait until tomorrow), and use "Dream now" in the dream journal.

### Inspect the files

- `%APPDATA%\mySteve\workspace\world\world.jsonl`: topic objects with `"why":"topic:<day>/<n>"`.
- `%APPDATA%\mySteve\workspace\world\journey.jsonl`: one line per day considered, with keywords, lines, subjects and odds.
- `%APPDATA%\mySteve\state\state.json`: `world.journeySince`, `world.journeyDay`.
- `%APPDATA%\mySteve\settings.json`: `dreaming.world`.
- `%APPDATA%\mySteve\workspace\sessions\main.jsonl`: `context` entries with `world`, user messages with `world`.

### Manual validation scenarios

| # | Story | Do | Expect |
|---|---|---|---|
| 1 | US1 | Tell Steve you ran before work and baked a cake for a birthday. Next day, dream now. | Two everyday objects that fit (for example shoes and a cake), next to that day's scenery, dated the day you told him. |
| 2 | US1 | Let Steve suggest a hobby you don't take up, and talk about an imagined trip. Next day, dream now. | Neither becomes a topic. |
| 3 | US1 | Say only "what's the capital of Peru?" for a day. Next day, dream now. | No topic object; the day's scenery is there. |
| 4 | US1 | Dream now twice. | The second run draws nothing more for that day. |
| 5 | US1 | Turn the network off, dream now. Turn it on, dream now again. | First: nothing drawn and no error. Second: the day gets its topics. |
| 6 | US1 | Dream now during the day. | Today gets no topics; yesterday does. |
| 7 | US1 | Kill the app (Task Manager) during a sweep, relaunch, wait for the next sweep. | The world and journey are whole after the next sweep: a day is fully drawn and recorded once, with nothing removed or repeated. |
| 8 | US2 | Point at a topic object, then at that day's flowers, then at an older tree. | Name, date and keywords (own first); name, date and keywords; name and date only. |
| 9 | US2 | Tab to an object; then tap one on a touch screen; press Escape. | The tooltip opens on focus and tap and closes on Escape without moving focus. |
| 10 | US2 | Open the list. | Days newest first, each topic's keyword and line beside its object, then the scenery. |
| 11 | US2 | Open "why this one" on an object; follow "notes". | The candidates with their chances, the pick marked; the memory browser opens on that day's note. |
| 12 | US3 | Talk about work only, then set the date forward and dream now, ten times. | Several different work objects, now and then a related one; never something unrelated. |
| 13 | US4 | After a night with topics, ask Steve what he drew last night. | He names the objects and what they stand for, as the journey says. |
| 14 | US4 | Start a fresh context and ask what his world is like. | A description that matches the picture: directions, recent objects, what it is mostly made of. |
| 15 | US5 | Drag the time control back to the first week, then play. | The picture shows the world as it was, then fills in day by day. With reduced motion on, it steps without movement. |
| 16 | US5 | Look at the makeup and the "about" section. | Shares of subjects as text and a bar; no id shown. |
| 17 | US6 | Forget a story and confirm. | Keyword and line gone from the tooltip, the list and the journey; the object remains with its date. The app offers a fresh context if Steve was told about it. |
| 18 | US6 | Forget a daily-note line in the memory browser. | That day's topics lose their stories too. |
| 19 | US6 | Turn "Draw my days in Steve's world" off, talk, dream now. | No topics; scenery keeps growing. |
| 20 | Edge | Write in another language for a day, then dream. | Keywords and lines in that language. |
| 21 | Edge | Edit `journey.jsonl` by hand (delete a day's line, add a broken line), open the world. | Objects stay; that day shows no story; the broken line is reported and left. |

### Release checks (real provider, owner)

| Check | How | Pass |
|---|---|---|
| SC-001 | 20 scripted days, each with one clear thing the user did, against the real model. | The first topic is that thing, with a fitting keyword and subject, on at least 18 days. |
| SC-002 | 20 scripted days with companion suggestions and imagined events. | 0 topics from them. |
| SC-011 | 10 fresh contexts after scripted nights. | The companion states its most recent object and what it stands for, matching the journey, in at least 9. |
| SC-016 | Network capture of a session including a sweep with topics. | Only the provider; no notification or badge. |

SC-015 (journey size after ten years) is measured by the ignored release test in the core.
