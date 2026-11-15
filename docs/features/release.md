# Release readiness (planned, not built yet)

Planned: not built yet.

Everything My Pec does now works on the owner's machine. This feature makes it something strangers can install, trust and keep. The main way in is the Microsoft Store: a package the Store signs and keeps up to date, published by Kodelokus. For people who want the open-source build, each GitHub release also offers a direct download with a public record of where it was built from. On top of that come a meter that shows what the companion costs in API usage, consent before anything is sent to the AI provider, a way to report a bad reply, the ability to take all your data with you or erase it, and public pages that say truthfully what the app does.

## User stories

### US1. Install My Pec from the Microsoft Store (P1)

A friend searches the Microsoft Store for "My Pec", sees it published by Kodelokus with screenshots, a description and an age rating, and clicks Install. There is no SmartScreen warning and no administrator prompt. My Pec opens on its wizard; launch at login, alarm notifications with My Pec's own name and icon, sticky notes on the desktop and the Live2D characters all work as they do in the direct download.

**Why this priority**: The Store is the channel that gives a trusted publisher and automatic updates for free, and the Company developer account it needs is already verified. A companion that holds someone's thoughts must not start with a warning about an unknown publisher.

**How to check**: Build the Store package from a version tag, install it locally in developer mode, and walk through the wizard, chat, notes, alarms (with the computer signed out and back in), weather and a Live2D character; then submit it to the Store and confirm it passes certification.

**Acceptance Scenarios**:

1. **Given** a version tag, **When** the release workflow runs, **Then** it builds a Store package of that tag's source, identified as Kodelokus's My Pec, ready to upload to Partner Center.
2. **Given** the package installed from the Store, **When** the person runs My Pec, **Then** every feature of the direct download works: the wizard, chat, memory, dreaming, the avatar including Live2D characters, the floating character, sticky notes, alarms and their notifications, weather and launch at login.
3. **Given** the person turns on launch at login, **When** they sign in to Windows, **Then** My Pec starts in the tray, through the mechanism Windows provides to packaged apps, and they can also turn it off in Windows' own startup settings.
4. **Given** an alarm rings, **When** the notification shows, **Then** it carries My Pec's name and icon.
5. **Given** the Store listing, **When** a person reads it, **Then** it describes what My Pec does, that it needs their own Claude API key and sends their conversations to Anthropic with it, links the privacy policy, and shows an age rating.

### US2. Clear consent, and a way to report a bad reply (P1)

During setup, right after the person pastes their key, My Pec says plainly that their messages, the companion's memory of them and their notes will be sent to Anthropic's Claude API with their own key, links the privacy policy, and asks them to agree before anything is sent. Later, Pec says something hurtful. The person opens the reply's menu, chooses "Report this reply", reads what will be shared, and sends the report; nothing is sent until they confirm.

**Why this priority**: The Store requires consent before personal data goes to a third party and a way to report inappropriate AI-generated content. Both are also simply right for every person, whichever way they installed.

**How to check**: On a fresh profile, confirm no request reaches the provider before consent is given, that declining keeps the person on the step with the reason and a way back, and that the report action shows exactly what would be shared and does nothing until confirmed.

**Acceptance Scenarios**:

1. **Given** a fresh install, **When** the person reaches the key step, **Then** before the key is checked they see what will be sent, to whom and why, with a link to the privacy policy, and must agree to continue; the key check itself happens only after they agree.
2. **Given** the person declines, **When** they stay on the step, **Then** nothing is sent anywhere and the companion explains that it can't talk without it.
3. **Given** an existing companion from before this release, **When** the app starts after updating, **Then** it asks for the same consent once before the next request.
4. **Given** a reply from the companion, **When** the person chooses to report it, **Then** they see the reply's text and an optional comment field, and nothing is shared until they confirm; confirming opens the project's report page with the reply filled in, and nothing else from the conversation is included.

### US3. Updates that arrive on their own (P1)

A month after installing from the Store, a new version of My Pec comes out. Windows updates it in the background through the Store. When the person next opens My Pec, a small note says it was updated to 0.2.0 and links to what's new. Their conversation, memory, settings and key are exactly as they were. Someone who downloaded the open-source build from GitHub instead sees a quiet note that a newer version exists, with a link to download it.

**Why this priority**: A companion that lives on the desktop for months must get fixes without the person hunting for them (SYS-2), and an update must never put their data at risk.

**How to check**: Install one version of the Store package, then a newer one as an update, and confirm the data folder and the key are untouched and the "updated" note appears once with its release notes; for the direct download, serve a newer release from a local address and confirm the note and its link.

**Acceptance Scenarios**:

1. **Given** My Pec installed from the Store, **When** a newer version is published, **Then** it is installed through the Store, and My Pec never downloads or installs updates by itself.
2. **Given** My Pec was updated, **When** the person opens it, **Then** a note shows the new version once, with its release notes readable in the app.
3. **Given** an update, **When** My Pec starts, **Then** every file, setting and the stored key are as they were; files from an older version are migrated and stay readable.
4. **Given** the direct download, **When** a newer release exists, **Then** the app shows a quiet note with the version and a link to its release page, at most once a day; the person can turn this check off.

### US4. Knowing what it costs (P2)

The person uses their own API key and wonders what Pec costs them. Settings shows a usage meter: tokens and an estimated cost for today and this month, split between conversation and background work (compaction and dreaming). They set a soft daily limit of US$1. One busy day the meter passes it: the night's dream is skipped, and before their next message the chat says the daily limit is reached and asks whether to go on today. They say yes and keep talking.

**Why this priority**: The person pays the provider directly, so cost must be visible (KEY-5, NFR-COST-3). A soft limit protects them from surprises without ever cutting off a conversation unasked.

**How to check**: With recorded usage on the fake model, confirm the meter adds up tokens and estimated cost per day and per month, chat and background separately; set a limit below the day's use and confirm dreaming is skipped and chat asks before sending.

**Acceptance Scenarios**:

1. **Given** conversation, compaction and dreaming have used the API, **When** the person opens the usage meter, **Then** it shows input, output and cached tokens and an estimated cost for today and this month, for chat and for background work.
2. **Given** a model whose price the app doesn't know, **When** its usage is shown, **Then** its tokens are counted and its cost is marked as unknown rather than guessed.
3. **Given** a soft daily limit, **When** the day's estimated cost reaches it, **Then** dreaming is skipped for the rest of the day, and the next message asks once whether to continue chatting today before sending. Agreeing covers chat only; compaction keeps running, because it keeps later messages cheaper.
4. **Given** no limit is set (the default), **When** usage grows, **Then** nothing is paused and nothing is asked.

### US5. Taking your data with you, or erasing it (P2)

The person wants a backup before trying something new. In settings they choose "Export everything" and get one archive file holding the companion's whole mind: personality, profile, memory, notes, the conversation, its world, extension files and settings, but not the API key. Months later they decide to stop using My Pec. They choose "Erase everything", read exactly what will be removed, type the companion's name to confirm, and the app removes all its data, the stored key and its sign-in entry, then closes. Before uninstalling from the Store, settings reminds them that uninstalling also removes the companion's data and offers an export.

**Why this priority**: The person owns the companion's mind (Principle I, CTL-4). Leaving must be as easy and complete as arriving, and a Store uninstall must never silently take their companion with it.

**How to check**: Export a workspace with data in every area and confirm the archive holds every file and no key; erase and confirm the data, the credential entry and the sign-in entry are gone and nothing else on the machine changed.

**Acceptance Scenarios**:

1. **Given** any amount of data, **When** the person exports, **Then** they choose where to save one archive containing the workspace and the settings, with the date in its name, and the API key is never in it.
2. **Given** an export is running, **When** a reply, compaction or dream is running too, **Then** the archive is consistent: no file is caught half-written.
3. **Given** the person chooses to erase, **When** they confirm by typing the companion's name, **Then** the app removes its data, the stored key and the launch-at-login entry, closes every window and quits; it offers an export first.
4. **Given** something can't be removed, **When** erasing, **Then** the app says exactly what is left and where, rather than claiming it is all gone.
5. **Given** My Pec installed from the Store, **When** the person reads the data section in settings or the privacy policy, **Then** it says that uninstalling the Store app removes the companion's data and that exporting first keeps it.

### US6. What the app says about itself is true (P3)

Before installing, a person reads the privacy policy linked from the Store listing. It lists every place My Pec connects to: the Claude API with their key, the one-time search model download if they agree, Open-Meteo while Weather is on, the release check in the direct download, and the report page only when they report a reply. The README tells them how to install from the Store or download directly, how to verify a direct download, and what it costs. Settings links to the same policy and shows the version.

**Why this priority**: Local-first is the promise (Principle II). A page that forgets a connection, as the privacy page does today with Weather, breaks that promise as surely as code would. The Store also requires a privacy policy.

**How to check**: Compare the privacy policy's list of connections with a network capture of a full session with every feature on, and confirm they match; read the README's install and verify steps on a clean machine.

**Acceptance Scenarios**:

1. **Given** the privacy policy, **When** compared with every network destination named in the specs, **Then** each one is listed with what is sent and when, and nothing else is.
2. **Given** the README, **When** a person follows it, **Then** they can install from the Store or download directly, verify a direct download, and know that My Pec has no cost of its own beyond their API usage.
3. **Given** the app, **When** the person opens settings, **Then** it links to the privacy policy and shows the app's version and how it was installed (Store or direct download).

## Edge cases

- The Store package and the direct download are installed on the same machine: they share no files, so each has its own companion; the README and the Store listing say so.
- Someone moves from the direct download to the Store: they export from one and restore the archive into the other by hand, as the README explains.
- The Store rejects a submission: the reasons are fixed in a new version; nothing about the direct download is affected.
- A Store update needs a newer data format: the app migrates files on first start, keeping older files readable, and a failed migration leaves the old files untouched (the quality gates in docs/principles.md).
- The release check of the direct download has no network: it fails silently and tries again the next day.
- The person reports a reply while offline: the report page opens when they are back online, or they can copy the text; nothing is queued in the background.
- The usage meter after a restart: totals are kept. If the database itself must be rebuilt, past usage is lost and the meter says from which day it counts.
- The day changes at midnight local time: today's meter and the soft limit start again.
- The price of a model changes: estimates for past days keep the price recorded then.
- Export to a full disk or a folder without write access: the person is told, and no partial archive is left behind.
- Erase while a note window is open or an alarm is due: windows close and nothing rings afterwards.

## Requirements

**The Store package**

- **FR-001**: Releases MUST be built from a version tag of the repository by the project's build service, never on a developer's machine, and MUST produce a Microsoft Store package identified as Kodelokus's My Pec, ready to submit.
- **FR-002**: The Store package MUST keep every feature of the direct download working, including the Live2D characters, desktop sticky notes, the floating character, alarm notifications with My Pec's name and icon, weather and launch at login (through the startup mechanism Windows gives packaged apps).
- **FR-003**: The Store package MUST NOT download or install updates by itself; the Store updates it.
- **FR-004**: The Store listing MUST describe the app, say that it needs the person's own Claude API key and sends their conversations to Anthropic, link the privacy policy, and carry an age rating; the package MUST include My Pec's license, the Live2D terms for the bundled runtime and the third-party notices.

**The direct download**

- **FR-005**: Each GitHub release MUST also offer the direct-download installer built by the same workflow, with its checksum and a verifiable public record of the source and workflow it was built from; the README MUST say it is unsigned and how to verify it.
- **FR-006**: The direct download MUST check about once a day whether a newer release exists and show a quiet note with a link to it; the person MUST be able to turn the check off. It MUST send nothing about the person.

**Consent and reporting**

- **FR-007**: Before the first request to the AI provider, including the key check, the app MUST show what is sent (messages, the companion's memory and notes about the person), to whom (Anthropic, with their own key) and why, link the privacy policy, and get the person's agreement; an existing companion MUST be asked once after updating.
- **FR-008**: Declining MUST send nothing anywhere and leave the person able to agree later.
- **FR-009**: Every companion reply MUST offer "Report this reply", which shows what will be shared (that reply and an optional comment) and shares nothing until the person confirms; confirming opens the project's report page with that content filled in.

**Updates**

- **FR-010**: After an update, the app MUST show the new version once, with its release notes readable in the app.
- **FR-011**: An update MUST leave every file, setting and the stored key as they were, migrating older formats and keeping them readable.

**Usage meter**

- **FR-012**: The app MUST record the token usage the provider reports for every call that reports it: chat replies, compaction and dreaming.
- **FR-013**: Settings MUST show input, output and cached tokens and an estimated cost for today and this month, for conversation and for background work, by the person's local day.
- **FR-014**: Estimated cost MUST use a price list shipped with the app and kept per day when recorded; a model without a known price MUST show its tokens and "cost unknown".
- **FR-015**: The person MAY set a soft daily limit (none by default). Once the day's estimate reaches it, dreaming MUST be skipped for the rest of the day, and the next message MUST ask once whether to continue chatting today before sending (KEY-5, NFR-SEC-6: no pressure, just the fact). Compaction keeps running.

**Export and erase**

- **FR-016**: The person MUST be able to export all data to one archive file at a place they choose: the workspace and the settings, never the API key or anything from the credential store (CTL-4, Principle II).
- **FR-017**: Export MUST produce a consistent archive while the app runs: no file is caught half-written, and a failed export leaves no partial file behind.
- **FR-018**: The person MUST be able to erase everything: the data, the stored key and the launch-at-login entry, after reading what will be removed and typing the companion's name; erase MUST offer an export first, close every window and quit. CTL-4's "wipe" is this erase.
- **FR-019**: Erase MUST remove only My Pec's own data and entries, and MUST report anything it couldn't remove and where it is.
- **FR-020**: In the Store package, settings and the privacy policy MUST say that uninstalling removes the companion's data and that exporting first keeps it.

**Paperwork**

- **FR-021**: The privacy policy MUST list every network destination in the specs, what is sent and when (the Claude API, the search model download, Open-Meteo while Weather is on, the direct download's release check, the report page only when the person reports), and MUST be published at a stable address the Store listing links to; settings MUST link to it and show the app's version and how it was installed.
- **FR-022**: The README MUST explain both ways to install, how to verify a direct download, how to move a companion between them, and that My Pec costs nothing beyond the person's own API usage.

## Key entities

- **Store package**: the signed-by-the-Store package of one version, with its identity (Kodelokus, My Pec), listing and age rating.
- **Direct download**: the unsigned installer of the same version on GitHub, with its checksum and build record.
- **Consent**: the person's agreement to send data to the AI provider, with when it was given and the version of the policy it covered.
- **Report**: one reply's text and an optional comment, shared only on confirmation through the project's report page.
- **Usage record**: one request's model, kind (chat or background), local day and the tokens the provider reported, with the price in force that day.
- **Price list**: per model, the price per million input, output and cached tokens, shipped with the app.
- **Export archive**: one file holding the workspace and settings, named with the date.
- **Erase report**: what was removed and anything left behind, with where it is.

## Success criteria

- **SC-001**: The first Store submission passes certification, or every reason it fails is fixed in the next submission; the published listing shows Kodelokus as the publisher.
- **SC-002**: On a clean Windows 11 machine, installing from the Store shows no SmartScreen warning or administrator prompt, and the first in-character reply comes within 3 minutes of clicking Install (requirements SC-1).
- **SC-003**: Every feature passes the same manual walkthrough in the Store package as in the direct download, including alarms after signing out and in.
- **SC-004**: On a fresh profile, a network capture shows no request to the AI provider before consent is given, and a report sends nothing until confirmed.
- **SC-005**: After a Store update and after an export and restore, every workspace file is byte-for-byte unchanged and the companion continues the conversation where it was.
- **SC-006**: Over a recorded week of use, the meter's token totals match the provider's reported usage exactly, and its estimated cost is within 1% of the price list applied by hand.
- **SC-007**: After erase, nothing of My Pec's data, credential entry or sign-in entry remains, and nothing outside them changed.
- **SC-008**: Anyone can confirm a direct download matches its release's checksum and build record in under 5 minutes by following the README.
- **SC-009**: A network capture of a full session with every feature on shows no destination missing from the privacy policy.

## Assumptions

- The Store signs the package, so no code-signing certificate is bought. Tauri doesn't build Store packages itself; the design adds a packaging step to the release workflow.
- The direct download stays unsigned (SmartScreen warns, as D16 accepted) and carries a Sigstore build record through GitHub's artifact attestations, free for public repositories.
- Store installs are updated by the Store. The direct download only tells the person a newer release exists; it doesn't install updates itself, which keeps one update mechanism to maintain.
- Inside a Store package, Windows keeps the app's data in a per-app location and removes it when the app is uninstalled. The plan checks whether the data can be kept across an uninstall; until then, the app says so and offers an export (FR-020).
- The report page is a public page the project runs (for example an issue form on its repository); the app only opens it in the browser with the reply filled in.
- The privacy policy is published at a stable public address before the first submission, which the Store requires.
- The price list covers the Claude models the model picker offers at release time and is updated with each release.
- The export archive is a standard ZIP; restoring is done by hand in this release, as the README explains.
- Kodelokus's Company developer account in Partner Center is already verified (2026-11-14). My Pec is free, so Live2D's small-business exemption still applies.

## Dependencies

- Kodelokus's verified Company account in Partner Center, and an age-rating questionnaire completed there.
- The planned move to `github.com/kodelokus/my-pec` for the direct download, the build record and the public privacy policy and report pages; Store packages can be built and tested before it.
- The existing release workflow (`.github/workflows/release.yml`), which builds the installer on a version tag into a draft release.
- The extensions and weather pages: launch at login and alarm notifications must work inside a package.
- `PRIVACY.md`, which today omits Weather and must list it.
- Requirements D16 (GitHub first, unsigned) changes: the Store becomes the main channel, signed by the Store, and GitHub the direct download; SYS-6 is met by the Store package.
