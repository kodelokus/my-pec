# App shell and onboarding

Built.

## User stories

### US1. From installer to a configured companion with only a key (P1)

A person who has never used mySteve runs the installer, pastes their Claude API key when the app opens, and presses Next through the remaining steps without changing anything. They end up in the main window, looking at a companion named Steve with a portrait, and the companion's files exist on their machine.

**Why this priority**: This is the product's first promise: install, paste a key, and you have a companion. Every later feature starts from the state this story produces.

**How to check**: On a clean Windows machine, run the installer as a standard user, paste a valid key, accept every default, and confirm the main window appears and the companion's files exist and are readable.

**Acceptance Scenarios**:

1. **Given** a machine where mySteve has never been installed, **When** a user without administrator rights runs the installer, **Then** the app installs for that user with no other software required and opens on the key step of the setup wizard.
2. **Given** the key step, **When** the user pastes a valid key, **Then** the step reports the key as accepted and lets the user continue.
3. **Given** an accepted key, **When** the user presses Next on each remaining step without entering anything, **Then** setup completes and the main window shows the default companion name and the default portrait.
4. **Given** setup has just completed, **When** the user looks in the companion's workspace folder, **Then** the soul, identity, user profile, an empty long-term memory and the avatar choice are present as readable text files, and a first hello is recorded as pending.
5. **Given** setup was completed earlier, **When** the user launches the app again, **Then** it opens on the main window and not on the wizard.
6. **Given** the wizard at any step, **When** the user works with the keyboard alone, **Then** they can complete setup, and a screen reader announces every control.

### US2. Key problems are explained and recoverable (P2)

A person pastes a key that does not work, or has no connection, or has an account without credit. The app tells them which of these it is, in plain language, and lets them fix it without starting over.

**Why this priority**: The key step is the only mandatory step and the only one that can fail for reasons outside the app. A vague failure here ends the first-run experience.

**How to check**: Run the key step with an invalid key, with the network disconnected, and against an account the provider reports as having no credit. Confirm that each produces its own message and a way forward, and that nothing is stored.

**Acceptance Scenarios**:

1. **Given** the key step, **When** the user submits a key the provider does not accept, **Then** the app says the key was not accepted, stores nothing, and lets the user enter another.
2. **Given** the key step and no network connection, **When** the user submits a key, **Then** the app says it could not reach the service, makes clear the key itself was not judged, and offers a retry.
3. **Given** the key step, **When** the provider reports that the account has no credit, **Then** the app says the key is genuine but the account has no credit, and tells the user what to do about it.
4. **Given** the key step, **When** the user submits an empty or obviously incomplete value, **Then** the app says so immediately without contacting the service.
5. **Given** a key check that has not answered after 10 seconds, **When** the wait ends, **Then** the app treats it as the service being unreachable and offers a retry.

### US3. Give the companion a personality, a name and a face (P2)

A person who wants more than the defaults picks a personality template, rewrites parts of it, renames the companion, chooses a different portrait, and tells the companion what to call them.

**Why this priority**: Giving the companion a soul and an avatar is what makes it theirs. It is not required to reach a working companion, which is why it sits below the first story.

**How to check**: Go through the wizard changing every optional value, complete setup, and confirm that each choice appears in the main window or in the workspace files.

**Acceptance Scenarios**:

1. **Given** the soul step, **When** the user picks a personality template, **Then** its text is shown and can be edited freely.
2. **Given** edited soul text, **When** the user picks a different template, **Then** the app asks before discarding the edits.
3. **Given** the soul step, **When** the soul text grows beyond its size budget, **Then** the indicator shows it is over, explains that the text beyond the budget will not be used in conversation, and still lets the user continue.
4. **Given** the soul step, **When** the user clears the name field and continues, **Then** the default name is used.
5. **Given** the avatar step, **When** the user picks one of the bundled characters, **Then** that character's portrait is the one shown in the main window after setup.
6. **Given** the "about you" step, **When** the user enters what they want to be called and a preferred language, **Then** both appear in the user profile file after setup.
7. **Given** the "about you" step, **When** the user skips it, **Then** setup completes and the user profile exists with those values unset.

### US4. Quit the wizard and pick it up later (P2)

A person starts setup, gets interrupted, and closes the app. When they open it again, they are where they left off, and nothing they entered is gone.

**Why this priority**: An interrupted setup that restarts from zero, or worse leaves a broken half-companion, is a common reason people abandon an app.

**How to check**: Quit the app at each wizard step in turn, relaunch, and confirm the wizard reopens at that step with earlier entries intact and that the main window never appears before setup is complete.

**Acceptance Scenarios**:

1. **Given** the user has had a key accepted and is on the soul step, **When** they quit and relaunch, **Then** the wizard opens on the soul step, the key is still accepted, and any soul text they typed is still there.
2. **Given** the user quit before any key was accepted, **When** they relaunch, **Then** the wizard opens on the key step and no key is stored.
3. **Given** setup has not been completed, **When** the app is launched by any means, **Then** the wizard is shown and the main window is not.
4. **Given** the user is on a later step, **When** they go back to an earlier step, **Then** their earlier entries are shown and nothing entered on later steps is lost.

### US5. The companion lives in the tray (P3)

A person closes the window and the companion stays available from the system tray. Starting the app a second time brings back the same window instead of opening another copy.

**Why this priority**: A companion is meant to be around all day, and later features (overnight memory work) depend on the app staying alive. It is not needed to finish setup.

**How to check**: With setup complete, close the window, reopen it from the tray, launch the app again from the Start menu, and quit from the tray, confirming one running copy throughout.

**Acceptance Scenarios**:

1. **Given** the main window is open, **When** the user closes it, **Then** the app keeps running and its icon stays in the tray.
2. **Given** the user closes the window for the first time, **When** the window disappears, **Then** the app tells them once that it is still running and how to quit.
3. **Given** the app is running in the tray, **When** the user chooses to open it from the tray icon, **Then** the window reappears as they left it.
4. **Given** the app is already running, **When** the user launches it again, **Then** the existing window comes to the front and no second copy starts.
5. **Given** the app is running, **When** the user chooses quit from the tray icon, **Then** the app exits completely.

### US6. Change the key or the model later (P3)

A person opens settings to replace their key, remove it, or pick a different model from the list the provider offers.

**Why this priority**: Keys get rotated and users will want to trade cost against quality, but neither is part of first-run success.

**How to check**: With setup complete, replace the key with another valid one, try to replace it with an invalid one, remove it, add it back, and change the model, confirming each result survives a restart.

**Acceptance Scenarios**:

1. **Given** a stored key, **When** the user opens settings, **Then** they see that a key is present and its last four characters, and never the full key.
2. **Given** a stored key, **When** the user submits a new valid key, **Then** the new key replaces the old one.
3. **Given** a stored key, **When** the user submits a new key that is not accepted, **Then** the old key stays in place and the app explains why the new one was rejected.
4. **Given** a stored key, **When** the user removes it and confirms, **Then** the key is gone, the app stays open, and the main window says conversation is unavailable until a key is added, with a direct way to add one.
5. **Given** a stored key and a connection, **When** the user opens the model setting, **Then** the list comes from the provider with a default already selected, and the user's choice is kept across restarts.
6. **Given** no connection, **When** the user opens the model setting, **Then** they see their current choice and the last known list, with a note that the list may be out of date.
7. **Given** a stored key the provider no longer accepts, **When** the app opens, **Then** it says the key is no longer accepted, stays usable, and offers to enter another key.
8. **Given** a chosen model the provider no longer offers, **When** the user opens the model setting, **Then** the default model is selected and a notice names the model that was replaced.

## Edge cases

- **A companion already exists.** If a workspace is found while setup is not complete, the wizard offers a choice once the key is accepted. Keeping the companion skips the soul, avatar and "about you" steps and changes no file. Starting fresh asks for confirmation, then moves the existing workspace to a dated backup folder before the new one is created. No existing file is ever overwritten or deleted.
- **The key has stray whitespace.** Leading and trailing spaces and line breaks around a pasted key are ignored.
- **The connection drops during the key check.** This is treated as the service being unreachable, not as a bad key.
- **Protected credential storage is unavailable or refuses the key.** The app says so and does not store the key anywhere else.
- **The workspace location cannot be written** (disk full, no permission). Setup does not complete, the app explains why, and nothing is left half-written.
- **A workspace file is missing at launch** after setup was completed. The app recreates that file from its default, leaves every other file untouched, and tells the user.
- **The user edits a workspace file in a text editor.** The app uses the edited content the next time it reads the file and does not overwrite it.
- **The chosen model is no longer offered.** The app falls back to the default model and tells the user.
- **The app is launched a second time while the wizard is open.** The wizard comes to the front.
- **The user quits during a key check.** No key is stored.
- **The window is closed during the wizard.** The app stays in the tray, and reopening it shows the wizard at the same step.
- **The app starts with no network after setup.** It opens normally and states that the service cannot be reached.
- **The stored key stops working.** If the provider no longer accepts the stored key, or reports no credit, the app says which of the two it is, stays usable, and offers to enter another key. It does not report this as a connection problem.
- **The stored key disappears during setup.** If the key is deleted outside the app while the wizard is on a later step, the wizard returns to the key step and keeps the other entries.

## Requirements

**Install and launch**

- **FR-001**: A standard Windows user without administrator rights MUST be able to install the app with a single installer and no other software to install first.
- **FR-002**: The first launch after installation MUST open the setup wizard. Every launch after setup is complete MUST open the main window.
- **FR-003**: Only one copy of the app MUST run per user. Launching it again MUST bring the existing window to the front.

**Setup wizard**

- **FR-004**: The wizard MUST present its steps in this order: key, soul, avatar, about you. It MUST show which step the user is on and allow going back without losing entries.
- **FR-005**: The key step MUST be the only mandatory step. Every other step MUST have defaults such that continuing without input completes setup.
- **FR-006**: Wizard progress and entries MUST be saved as the user makes them, so that quitting at any moment and relaunching resumes at the same step with every entry intact, including text typed on the current step.
- **FR-007**: Until the wizard is complete, the app MUST treat setup as unfinished: the main window is not shown and no partially written companion is treated as ready.
- **FR-008**: If a companion workspace already exists while setup is not complete, the wizard MUST offer to keep it or to start fresh. Keeping it MUST change no workspace file. Starting fresh MUST ask for confirmation and MUST move the existing workspace to a dated backup folder before creating the new one. No existing file may be overwritten or deleted (see also FR-035).
- **FR-009**: Every wizard step and every settings control MUST be usable with the keyboard alone and MUST be labeled for screen readers.

**API key**

- **FR-010**: The key step MUST accept a pasted or typed key, ignore surrounding whitespace, and show the address where a key can be created, as text the user can copy.
- **FR-011**: The app MUST check the key with the provider as soon as it is submitted, at no cost to the user, and MUST give a result within 10 seconds or report that the service could not be reached.
- **FR-012**: The app MUST give a distinct plain-language message for each of: the key was not accepted, the account has no credit (when the provider reports it), and the service could not be reached. Each message MUST come with a way to retry or enter another key.
- **FR-013**: A key MUST be stored only after the provider has accepted it. It MUST be stored in the operating system's protected credential storage for the current user and MUST NOT be written to any other file, log, report or export.
- **FR-014**: Once stored, the full key MUST NOT be displayed again. The app MAY show that a key is present and its last four characters.
- **FR-015**: If protected credential storage is unavailable, the app MUST say so and MUST NOT fall back to any less protected storage.

**Soul and identity**

- **FR-016**: The soul step MUST offer at least four personality templates and a blank one, with one template preselected.
- **FR-017**: The soul text MUST be freely editable after a template is picked. Switching template after editing MUST ask before discarding the edits.
- **FR-018**: The soul step MUST let the user set the companion's name, between 1 and 40 characters, defaulting to "Steve". An empty name MUST revert to the default.
- **FR-019**: The soul step MUST show how much of the soul's size budget the text uses. When the text is over budget, the app MUST warn that the excess will not be used in conversation and MUST still let the user continue.

**Avatar**

- **FR-020**: The avatar step MUST show the bundled characters as still portraits, with one preselected, and MUST record the user's choice. At least one character MUST be available.

**About you**

- **FR-021**: The "about you" step MUST let the user say what the companion should call them and their preferred conversation language. Skipping it MUST leave both unset.

**Completion**

- **FR-022**: Completing the wizard MUST create the companion's workspace with the soul, the identity (name and avatar choice), the user profile (seeded from "about you"), an empty long-term memory, and the avatar choice, all as plain text files a person can open and edit.
- **FR-023**: Completing the wizard MUST record that the companion's first hello is pending, so that the conversation feature can deliver it.
- **FR-024**: After completion, the main window MUST show the companion's name and chosen portrait and the area where conversation will take place.

**App shell**

- **FR-025**: While the app runs, an icon MUST be present in the system tray with actions to open the app, open settings, and quit.
- **FR-026**: Closing the window MUST leave the app running in the tray. The first time this happens, the app MUST tell the user it is still running and how to quit.
- **FR-027**: Choosing quit from the tray MUST exit the app completely.

**Settings**

- **FR-028**: Settings MUST show whether a key is stored, and MUST let the user replace it or remove it. A replacement MUST be accepted by the provider before it takes the old key's place. Removal MUST ask for confirmation.
- **FR-029**: Settings MUST let the user choose a model from a list obtained from the provider, with a default preselected. The choice MUST persist across restarts.
- **FR-030**: If the chosen model is no longer offered, the app MUST fall back to the default and tell the user which model was replaced.
- **FR-031**: If the model list cannot be refreshed, settings MUST show the current choice and the last known list, with a note that the list may be out of date.

**Degraded states and privacy**

- **FR-032**: With no key stored, the app MUST still open, show the companion, and state that conversation is unavailable until a key is added, with a direct way to add one.
- **FR-033**: With no network connection, the app MUST still open, state that the service cannot be reached, and offer a retry.
- **FR-034**: The only data this feature sends off the device MUST be the key check and the model list request to the provider. It MUST send no usage data or analytics.
- **FR-035**: The app MUST honor edits the user makes to workspace files outside the app and MUST NOT silently overwrite them (see also FR-008).
- **FR-036**: If the provider no longer accepts the stored key, or reports that the account has no credit, the app MUST say which of the two it is, MUST stay usable, and MUST offer a direct way to enter another key. It MUST NOT present this as the service being unreachable.

### Key entities

- **Companion workspace**: The folder that holds everything the companion is. It belongs to the user and contains only plain text files in this feature.
- **Soul** (`SOUL.md`): The companion's personality, voice, values and boundaries as free text. It has a size budget.
- **Identity** (`IDENTITY.md`): The companion's name and which avatar it uses.
- **User profile** (`USER.md`): What the companion should call the user and their preferred language. It starts with whatever "about you" supplied.
- **Long-term memory** (`MEMORY.md`): Created empty by this feature. Later features fill it.
- **Avatar choice**: Which bundled character represents the companion. In this feature a character is a name and a still portrait.
- **Personality template**: A named, ready-made soul text the user can start from.
- **API key**: The user's secret for the provider. The app exposes only whether one is stored and its last four characters.
- **Model choice**: The model the companion will use, picked from the provider's list, with a default.
- **Setup progress**: Which wizard step the user is on, what they have entered so far, whether setup is complete, and whether the first hello is still pending.

## Success criteria

- **SC-001**: A new user with a valid key gets from starting the installer to the main window in under 2 minutes, typing nothing but the key.
- **SC-002**: After the key is accepted, setup can be finished with no more than four confirmations and no typing.
- **SC-003**: On a working connection, 95% of key checks receive the provider's answer within 5 seconds.
- **SC-004**: In scripted tests, an unaccepted key, a no-credit report and an unreachable service each produce their own message 100% of the time, and none of them stores a key.
- **SC-005**: Quitting at any wizard step and relaunching returns the user to that step with entries intact in 100% of tested steps.
- **SC-006**: After setup, a search of every file the app has written and all of its logs finds the key zero times.
- **SC-007**: Every workspace file created by setup opens as readable text in a standard text editor.
- **SC-008**: Launching the app again while it is running results in a single running copy in 20 of 20 attempts.
- **SC-009**: With the network disconnected, the app opens to a usable window in under 5 seconds and states what is unavailable within 10 seconds.
- **SC-010**: In a test with five first-time users who have a key, at least four complete setup without help.
- **SC-011**: A network capture taken during setup and settings use shows that the app's own process and the pages it shows connect to the provider and to nothing else. Connections the WebView2 runtime makes on its own are outside this criterion (see Assumptions).

## Assumptions

- **Platform.** Windows 11 is the only platform in scope for this feature.
- **Provider.** The key is for the Claude service, the only provider in v1. Obtaining a key happens outside the app, which only points to where one can be created.
- **The key check needs a connection.** A key must be accepted by the provider before the user can leave the key step. With no connection the user retries later, which the resumable wizard allows.
- **Credit may not be visible at the key check.** The free check confirms the key is genuine. It may not reveal that the account has no credit. The no-credit message is shown wherever the provider first reports that condition, which may be during the first conversation in the next feature.
- **Uninstalling removes only the app.** The workspace, the app's saved state and the stored key all stay in place, so after a reinstall the app opens straight to the main window with the same companion.
- **When an existing companion is offered back.** The choice in FR-008 arises only when a workspace exists but the app's saved state does not say setup is complete, for example after the state folder was deleted or a workspace was copied in from a backup.
- **Workspace location.** The workspace is created in the app's default data location. Moving it is outside this feature.
- **Soul budget.** The size budget comes from the product configuration, currently 8,000 characters.
- **Templates and characters are content.** The wording of the personality templates and the choice of bundled characters are decided in the Decisions. Which characters can be bundled depends on the open licensing decision D10 in `docs/requirements.md`.
- **Out of scope, delivered by later features:**
  - the conversation itself, including the companion's first in-character hello
  - animated avatar rendering
  - memory search, compaction and dreaming
  - automatic updates and signed installers
  - launch at login
  - moving, exporting or wiping the workspace
- **The WebView2 runtime has its own traffic.** The window is drawn by the Microsoft WebView2 runtime that ships with Windows. On each launch its browser process connects to Microsoft services by itself, outside the pages the app shows, and the documented browser flags do not turn this off (measured 2026-11-13, Checks SC-011). FR-034 and SC-011 cover what the app sends, not the runtime's own traffic.
- **Known limitation until installers are signed.** Windows may show a warning before running the installer. The timing in SC-001 excludes dismissing that warning.

## Decisions

Sources are listed at the end.

### Environment found on the development machine

| Tool | Version |
|---|---|
| Rust (`rustc`, `cargo`) | 1.99.1 |
| Node.js | 24.8.0 |
| pnpm | 10.34.8 |
| git | 2.46.2 (the project folder is not a repository yet) |
| WebView2 runtime | 153.0 |

### R1. Installer

- **Decision**: Tauri's NSIS installer with its default install mode.
- **Rationale**: By default the NSIS installer installs for the current user only, into `%LOCALAPPDATA%`, and does not ask for administrator rights. That is exactly FR-001. The default WebView2 mode (`downloadBootstrapper`) installs the runtime only when it is missing, and Windows 11 ships it.
- **Alternatives considered**: `perMachine` and `both` need administrator rights. MSI (WiX) is aimed at managed deployments.

### R2. Single instance

- **Decision**: `tauri-plugin-single-instance`, registered before any other plugin, with a callback that shows, unminimizes and focuses the main window.
- **Rationale**: It is the maintained Tauri plugin, it needs no capability entry because it has no JavaScript API, and its documentation says it must be the first plugin registered. It requires Rust 1.90 or later, and 1.99 is installed.
- **Alternatives considered**: A hand-rolled named mutex. More code to own for the same result.

### R3. Tray and close-to-tray

- **Decision**: Tauri's built-in tray (`tray-icon` feature) with a menu of Open, Settings and Quit. A close request on the window is intercepted: the window is hidden and the process keeps running. The first-close notice is shown inside the window before it hides.
- **Rationale**: The tray is part of Tauri 2 and needs no plugin. Showing the notice in the window avoids adding a notification or dialog plugin for one message.
- **Alternatives considered**: A native notification through `tauri-plugin-notification`. An extra plugin and an extra permission for a one-time message.

### R4. Restricting which commands the webview may call

- **Decision**: Declare the app's commands in `build.rs` through `tauri_build::AppManifest::commands`, and allow them for the main window in `src-tauri/capabilities/main.json`.
- **Rationale**: By default every command registered with the invoke handler is callable from every window. The principles (Principle II and the security gates) requires an explicit allow-list. `AppManifest::commands` is the documented way to turn the default off.
- **To confirm during implementation**: the exact permission identifier generated for each command. The capabilities page does not state the format.

### R5. Generating TypeScript types from Rust

- **Decision**: `ts-rs` 12 for the types. One small handwritten module, `src/api/commands.ts`, wraps the nine `invoke` calls using those generated types.
- **Rationale**: `tauri-specta` would generate the command wrappers too, but its Tauri 2 line has no stable release. The newest is 2.0.0-rc.29, published on 2026-11-13, the third release candidate in four weeks. `ts-rs` is stable. The principles require the types to be generated, which this does. The verification script regenerates them and fails if the committed files differ.
- **Alternatives considered**: `tauri-specta` 2.0.0-rc. Rejected for now because of the churn. Revisit when 2.0 is released as stable.

### R6. Storing the API key

- **Decision**: `keyring-core` 1.0 with `windows-native-keyring-store` 1.1, which stores a generic credential in Windows Credential Manager. Persistence is set to `Local`. All access goes through one `SecretStore` object in the shell.
- **Rationale**: The `keyring` crate was restructured in version 4 and now recommends this pair for applications that pick their own store. The store's default persistence is `Enterprise`, which roams with a domain profile. `Local` keeps the key on this machine. The store documents that operations on one entry from different threads may not run in order, so access is funneled through one place.
- **Alternatives considered**: The umbrella `keyring` crate with its `v1` feature. It works but is the compatibility path. Tauri's Stronghold plugin needs its own password and solves a different problem.
- **To confirm during implementation**: the exact `Entry` method signatures, read from the crate documentation rather than from memory.

### R7. Talking to the provider

- **Decision**: One request, `GET https://api.anthropic.com/v1/models`, with the headers `x-api-key` and `anthropic-version: 2026-10-21`. It validates the key and returns the model list in the same call. The client is hand-written with `reqwest` (rustls) in the core's `llm` module. One attempt, 10-second timeout, no automatic retry.
- **Rationale**: There is no official Claude SDK for Rust. Listing models is not billed, which satisfies "at no cost to the user" (FR-011). The Models endpoint behaves the same on API version `2026-10-21` as on earlier versions. Retry with backoff is shared machinery that the chat slice will need. This slice has a Retry button, so building it now would be code with no caller.
- **Error mapping**: classified by the error `type` in the response body, then by status.

  | Provider response | Outcome shown to the user |
  |---|---|
  | 200 | Key accepted |
  | 401 `authentication_error` | Key not accepted |
  | 403 `permission_error` | Key not accepted |
  | 402 `billing_error` | Key is genuine, account has no credit |
  | 429, 529, any 5xx, timeout, connection failure | Service could not be reached |

- **Finding that affects an assumption**: the API has a dedicated billing error (402). The Assumptions say missing credit might not be visible at the key check. It is still not known whether the free model-list call returns 402 for an account without credit, so the assumption stands and the mapping is built and tested either way.
- **To confirm when recording fixtures**: the pagination field names of the list response.

### R8. Where data lives

- **Decision**: `%APPDATA%\mySteve\workspace\` for the companion's files and `%APPDATA%\mySteve\state\` for `settings.json` and `state.json`. The shell resolves the root and passes it to the core.
- **Rationale**: Matches `docs/architecture.md` section 4 and decision D6. The core never asks the operating system or Tauri for paths, which keeps it shell-independent and lets tests point it at a temporary folder.
- **Open item for the owner**: Tauri needs a reverse-domain bundle identifier. The project uses `com.mysteve.desktop` as a placeholder. It should be settled before any public release because changing it later changes how Windows identifies the app.

### R9. Format of the small workspace files

- **Decision**: `IDENTITY.md` and `USER.md` are Markdown lists of `- Key: value` lines, read with a few lines of parsing.
- **Rationale**: Readable and editable by hand, close to OpenClaw's style, and no YAML parser dependency for two or three fields.
- **Alternatives considered**: YAML front matter. It needs a parser dependency and is easier for a person to break.

### R10. The avatar in this slice

- **Decision**: Bundled characters are still portraits shipped with the user interface, keyed by a character id. Setup records the chosen id in `IDENTITY.md` and in `avatar/manifest.json` with type `static`. No model files are copied into the workspace yet.
- **Rationale**: The requirements ask only that the choice is recorded and the portrait shown. Loading model files from the workspace belongs to the avatar slice, which is the first consumer of that machinery.
- **Open item for the owner**: the portrait itself. Until decision D10 (which character may be bundled) is made, the slice ships one original placeholder portrait.

### R11. Front-end tooling

- **Decision**: React, TypeScript and Vite at their current stable versions, pinned by the lockfile. Biome for formatting and linting. CSS modules. No router and no state library.
- **Rationale**: Six screens chosen by one status value do not need routing or a store. Biome is one tool and one config file where ESLint with Prettier is several.
- **Alternatives considered**: ESLint with Prettier, a router, a state library. None has a need in this slice.

### R12. Test tooling

- **Decision**:
  - Core: `cargo test`, with `tempfile` for temporary workspaces and `wiremock` to replay recorded provider responses to the real HTTP client.
  - User interface: Vitest with Testing Library.
  - End to end: WebdriverIO with `@wdio/tauri-service`, using its `tauri-driver` route on Windows, against a small local server that replays the same recorded responses.
- **Rationale**: Tauri's documentation now recommends WebdriverIO with that service, which keeps the Edge WebDriver in step with the installed WebView2. Replaying through a local server exercises the real request, including its headers, without ever calling the real API.
- **The one test-only seam**: the provider's base URL can be overridden by an environment variable only in builds compiled with the `e2e` cargo feature. Release builds have no override.

### R13. The verification script

- **Decision**: `scripts/verify.ps1` is the single entry point. Without arguments it runs the fast gates. With `-E2E` it also builds the app and runs the end-to-end specs.
- **Fast gates, in order**:
  1. `cargo fmt --check`
  2. `cargo clippy` on the whole workspace with warnings as errors
  3. a check that `steve-core`'s dependency tree does not contain `tauri` (Principle VI)
  4. `cargo test` on the whole workspace, which also regenerates the TypeScript types
  5. a check that the regenerated types match the committed ones
  6. Biome check, TypeScript type check, Vitest
- **Rationale**: The `test-audit` skill expects one script that runs everything once per commit. PowerShell matches the Windows-first platform.

### R14. Deliberately not built in this slice

Each of these is in the architecture but has no caller yet. The slice that first uses it builds it.

- SQLite, the search index and embeddings
- the memory writer queue and the hash check before rewrites (this slice writes each file once, and never over an existing file)
- retry with backoff, streaming and the Messages API
- a logging framework (the key type refuses to print itself, which covers the "no secret in logs" gate)
- the asset protocol and loading avatar files from the workspace
- an internationalization library (all strings sit in one module, ready for it)

### Sources

- Tauri: Windows installer — https://v2.tauri.app/distribute/windows-installer/
- Tauri: single instance plugin — https://v2.tauri.app/plugin/single-instance/
- Tauri: system tray — https://v2.tauri.app/learn/system-tray/
- Tauri: capabilities — https://v2.tauri.app/security/capabilities/
- Tauri: WebDriver testing — https://v2.tauri.app/develop/tests/webdriver/
- tauri-specta versions — https://crates.io/crates/tauri-specta
- ts-rs versions — https://crates.io/crates/ts-rs
- keyring — https://docs.rs/keyring/latest/keyring/
- keyring-core — https://docs.rs/keyring-core/latest/keyring_core/
- windows-native-keyring-store — https://docs.rs/windows-native-keyring-store/latest/windows_native_keyring_store/
- Claude API documentation (Models API, errors, API versions) — https://platform.claude.com/docs

## Data

All Rust types below live in `steve-core`. The ones that cross to the webview are exported to TypeScript by `ts-rs`.

### Data 1. Files in the workspace

Root: `%APPDATA%\mySteve\workspace\`. Created when setup completes (FR-022). Each file is created only if it does not exist. Setup never writes over an existing file (FR-008, FR-035). When the user chooses to start fresh over an existing companion, the whole workspace folder is first moved to `%APPDATA%\mySteve\workspace-backup-<date>-<time>\`.

| File | Content written by setup | Notes |
|---|---|---|
| `SOUL.md` | The soul text exactly as the user left it | Stored whole even when over budget (FR-019). |
| `IDENTITY.md` | `# Identity`, then `- Name: <name>` and `- Avatar: <character id>` | Read back at launch to show the name and portrait. |
| `USER.md` | `# About the user`, then `- Call them: <value>` and `- Preferred language: <value>` | Values are empty when "about you" was skipped (FR-021). |
| `MEMORY.md` | `# Memory` and nothing else | Later slices fill it. |
| `avatar/manifest.json` | `{ "type": "static", "character": "<character id>" }` | The avatar slice adds the `live2d` type. |

`IDENTITY.md` and `USER.md` use one `- Key: value` line per field. Unknown lines are preserved and ignored, so a person can add notes.

### Data 2. Files in the state folder

Root: `%APPDATA%\mySteve\state\`. Both files are written atomically (temporary file, then rename). A missing file means defaults.

#### `settings.json` — what the user chose

| Field | Type | Default | Rule |
|---|---|---|---|
| `chat.model` | string | `claude-opus-5-5` | Must be an id from the model list. If it is no longer offered, it falls back to the default, and to the first listed model if the default is also absent (FR-030). |

#### `state.json` — bookkeeping

| Field | Type | Default | Purpose |
|---|---|---|---|
| `setup.complete` | bool | false | Decides wizard or main window at launch (FR-002, FR-007). |
| `setup.draft` | SetupDraft | defaults | Wizard progress, saved whenever an entry or the step changes (FR-006). Dropped when setup completes. |
| `firstHelloPending` | bool | false | Set to true when setup completes with a new companion (FR-023). Not set when an existing companion is kept. The chat slice clears it. |
| `closeNoticeShown` | bool | false | Whether the "still running in the tray" notice has been shown (FR-026). |
| `models.items` | ModelInfo[] | empty | The last model list received (FR-031). |
| `models.fetchedAt` | timestamp | none | When that list was received. |

### Data 3. The credential

| Property | Value |
|---|---|
| Store | Windows Credential Manager, generic credential |
| Service | `mySteve` |
| Account | `claude-api-key` |
| Persistence | `Local` |
| Written | Only after the provider has accepted the key (FR-013) |

The key appears in no file. In memory it is held in an `ApiKey` type whose debug and display output is masked.

### Data 4. Types

#### SetupStep

`Key`, `Soul`, `Avatar`, `AboutYou`. In that order (FR-004).

#### SetupDraft

| Field | Type | Default | Validation |
|---|---|---|---|
| `step` | SetupStep | `Key` | Cannot be past `Key` unless a key is stored. |
| `soul.templateId` | string | the first template | Must be a known template id or `blank`. |
| `soul.text` | string | the template's text | Any length. Over budget is allowed (FR-019). |
| `soul.name` | string | `Steve` | Trimmed. 1 to 40 characters. Empty becomes the default (FR-018). |
| `characterId` | string | the first character | Must be a known character id (FR-020). |
| `about.callMe` | string or none | none | Trimmed. Empty becomes none. |
| `about.language` | string or none | none | Trimmed. Empty becomes none. |

#### SetupState

`InProgress { draft }` or `Complete`.

State transitions:

```text
            key accepted          Next             Next              Next / Skip
   Key  ───────────────▶  Soul  ───────▶  Avatar  ───────▶  AboutYou  ───────────▶  Complete
    ▲                      │ ▲              │ ▲                 │
    └──────── Back ────────┘ └──── Back ────┘ └────── Back ─────┘
```

- Leaving `Key` requires an accepted key. Every other step can be left with its defaults (FR-005).
- Going back keeps every entry (FR-004).
- If a key is already stored when the wizard starts at `Key`, the step shows it as accepted and lets the user continue.
- If the saved draft is past `Key` but no key is stored, the wizard returns to `Key` and keeps the other entries.
- If a workspace already exists when the key is accepted, the user chooses. Keeping the companion jumps from `Key` straight to `Complete` and writes no workspace file. Starting fresh continues through the remaining steps, and at completion the existing workspace is moved to the backup folder before the new files are created (FR-008).
- `Complete` is final. Removing the key later does not reopen the wizard (FR-032).

#### SetupCompletion

How `complete_setup` finishes the wizard.

| Mode | When | Effect |
|---|---|---|
| `New` | No workspace exists | Creates the five files from the draft. Sets `firstHelloPending`. Rejected as `InvalidInput` if a workspace exists. |
| `AdoptExisting` | A workspace exists and the user keeps it | Writes no workspace file. Does not set `firstHelloPending`. |
| `ReplaceExisting` | A workspace exists and the user starts fresh | Moves the workspace to the backup folder, then behaves as `New`. |

#### KeyStatus

`Absent` or `Present { lastFour }`. This is all the webview ever learns about the key (FR-014).

#### KeyCheckOutcome

| Variant | Meaning | Key stored? |
|---|---|---|
| `Accepted { lastFour }` | The provider accepted the key | Yes |
| `Incomplete` | Empty, shorter than 20 characters, or contains whitespace after trimming the ends. Decided locally, with no request. | No |
| `Rejected` | The provider did not accept the key | No |
| `NoCredit` | The provider reported a billing problem | No |
| `Unreachable` | No answer within 10 seconds, a connection failure, or a server-side error | No |
| `StorageUnavailable` | The key was accepted but the credential store refused it (FR-015) | No |

When a key is already stored and a replacement is submitted, any outcome other than `Accepted` leaves the old key in place (FR-028).

#### CompanionSummary

| Field | Type | Source |
|---|---|---|
| `name` | string | `IDENTITY.md` |
| `characterId` | string | `IDENTITY.md` |

#### AppStatus

| Field | Type | Purpose |
|---|---|---|
| `setup` | SetupState | Wizard or main window. |
| `key` | KeyStatus | Whether conversation can be offered (FR-032). |
| `companion` | CompanionSummary or none | Present when setup is complete, or when an earlier workspace was found. |
| `existingWorkspace` | bool | True when setup is not complete but a workspace exists (FR-008). |
| `recreatedFiles` | string[] | Workspace files that were missing at launch and were recreated from defaults. |

#### SetupOptions

| Field | Type |
|---|---|
| `templates` | SoulTemplate[] (at least four, plus `blank`) |
| `characters` | Character[] (at least one) |
| `soulBudgetChars` | number (8,000) |
| `defaultName` | string (`Steve`) |
| `nameMaxChars` | number (40) |

#### SoulTemplate

`id`, `title`, `description`, `text`. The texts are Markdown files embedded in the core.

#### Character

`id`, `name`. The portrait is a user-interface asset looked up by `id`.

#### ModelInfo and ModelList

| Type | Fields |
|---|---|
| ModelInfo | `id`, `displayName` |
| ModelList | `models: ModelInfo[]`, `selected: string`, `stale: bool`, `problem: ProviderError or none`, `fallbackFrom: string or none` |

- `stale` is true when the list shown did not come from the provider in this call: either no refresh was asked for, or the refresh failed (FR-031).
- `problem` is set only when a refresh was attempted and failed. `Unreachable` makes the main window say the service cannot be reached (FR-033). `KeyRejected` and `NoCredit` make it say which and offer to enter another key (FR-036).
- `fallbackFrom` holds the id of the previously chosen model when the choice fell back to the default (FR-030). It is reported on the call that makes the change.

### Data 5. Traits

| Trait | Declared in | Implementations | Why a trait |
|---|---|---|---|
| `LlmProvider` (this slice: `list_models` only) | `steve-core::llm` | The Claude client. A fake in tests. | The principles require the core to depend on the trait, and tests must never reach the real API. |
| `SecretStore` (`get`, `set`, `delete`) | `steve-core::credentials` | Windows Credential Manager, in the shell. In-memory, in tests. | The real store exists only on Windows and writes to the developer's own credential manager. |

## Interfaces

### Commands Between the Webview and the Core

This is the complete surface the main window may call in this slice. Every command is declared in `build.rs` and allowed in `src-tauri/capabilities/main.json`. Nothing else is reachable from the webview.

#### Rules that apply to every command

- Handlers are thin: deserialize, call `steve-core`, map the result. All validation happens in the core.
- No command returns the API key or any part of it beyond the last four characters.
- Expected failures are values in the result type. Unexpected failures return `AppError`.
- The TypeScript types are generated from the Rust types. `src/api/commands.ts` is the only place that calls `invoke`.

#### Commands

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `app_status` | — | `AppStatus` | Reads `state.json`, the credential store and, when a workspace exists, `IDENTITY.md`. Recreates a missing workspace file from its default when setup is complete, and lists it in `recreatedFiles`. If the saved draft is past `Key` while no key is stored, returns it at `Key`. Makes no network request. | FR-002, FR-007, FR-008, FR-032 |
| `setup_options` | — | `SetupOptions` | Returns the templates, the bundled characters and the limits. | FR-016, FR-018, FR-019, FR-020 |
| `submit_key` | `key: string` | `KeyCheckOutcome` | Trims the value. Returns `Incomplete` without a request when it is obviously not a key. Otherwise asks the provider for the model list with a 10-second limit. On success, stores the key, saves the model list as the last known list, and returns `Accepted`. On any other outcome stores nothing and leaves an existing key in place. | FR-010 to FR-015, FR-028 |
| `remove_key` | — | `KeyStatus` | Deletes the credential and returns `Absent`. The webview asks for confirmation before calling it. | FR-028, FR-032 |
| `save_setup_draft` | `draft: SetupDraft` | — | Validates and saves the draft to `state.json`. Rejects a step past `Key` when no key is stored. The webview calls it whenever an entry or the step changes. | FR-004, FR-006 |
| `complete_setup` | `mode: SetupCompletion` | `CompanionSummary` | `New` creates each workspace file from the saved draft and is rejected as `InvalidInput` when a workspace already exists. `AdoptExisting` writes no workspace file. `ReplaceExisting` moves the existing workspace to a dated backup folder, then creates the new files. All three set `setup.complete` and drop the draft. `New` and `ReplaceExisting` also set `firstHelloPending`. | FR-008, FR-022, FR-023, FR-024 |
| `model_list` | `refresh: bool` | `ModelList` | With `refresh` true and a stored key, asks the provider and saves the result. If that fails, returns the last known list with `stale` true and `problem` set to the reason. With `refresh` false, returns the last known list with `stale` true and no problem. Applies the fallback rule when the selected model is no longer listed and reports it in `fallbackFrom`. The main view calls it with `refresh` true when it opens. | FR-029, FR-030, FR-031, FR-033, FR-036 |
| `select_model` | `id: string` | `ModelList` | Saves the choice to `settings.json` if the id is in the list. | FR-029 |
| `hide_to_tray` | — | — | Marks the close notice as shown and hides the window. | FR-026 |

#### Events sent to the webview

| Event | When | The webview responds by |
|---|---|---|
| `close_notice` | The user closes the window and the notice has never been shown | Showing the "still running in the tray" notice, then calling `hide_to_tray` when the user confirms. |
| `open_settings` | The user picks Settings in the tray menu | Showing the settings view. |

After the notice has been shown once, a close request hides the window directly and no event is sent.

#### AppError

| Kind | Meaning | What the user sees |
|---|---|---|
| `WorkspaceNotWritable` | A workspace or state file could not be created or written | An explanation that setup could not save, with the folder path. Nothing is left half-written. |
| `InvalidInput` | The payload failed validation in the core | The field is marked. This indicates a bug in the interface, not a user mistake. |
| `Internal` | Anything else | A generic failure message with a retry. |

#### Tray menu

| Item | Effect |
|---|---|
| Open mySteve | Shows, unminimizes and focuses the main window. A left click on the tray icon does the same. |
| Settings | Shows the window and sends `open_settings`. |
| Quit | Exits the process (FR-027). |

Launching the app while it is running has the same effect as "Open mySteve" (FR-003).

### The Provider's Model List

This is the only request the app sends off the device in this slice (FR-034). It lives entirely in `steve-core::llm::claude`. Nothing outside that module names a header, a path or a wire field.

#### Request

| Part | Value |
|---|---|
| Method and URL | `GET https://api.anthropic.com/v1/models` |
| Header `x-api-key` | The key being checked, or the stored key |
| Header `anthropic-version` | `2026-10-21` (pinned) |
| Timeout | 10 seconds for the whole request (FR-011) |
| Retries | None. The user retries. |
| Cost | Not billed |

If the response says more pages exist, the client follows them until the list is complete.

#### Response fields the app reads

| Field | Used for |
|---|---|
| `data[].id` | The model id stored in settings |
| `data[].display_name` | The label in the model picker |

Every other field is ignored and tolerated, including fields the client has never seen.

#### Outcomes

| Provider response | `ProviderError` / result | `KeyCheckOutcome` | In `model_list` |
|---|---|---|---|
| 200 with a list | models | `Accepted` | fresh list, `stale: false` |
| 401, or error type `authentication_error` | `KeyRejected` | `Rejected` | last known list, `stale: true` |
| 403, or error type `permission_error` | `KeyRejected` | `Rejected` | last known list, `stale: true` |
| 402, or error type `billing_error` | `NoCredit` | `NoCredit` | last known list, `stale: true` |
| 429, 529, any 5xx | `Unreachable` | `Unreachable` | last known list, `stale: true` |
| Timeout or connection failure | `Unreachable` | `Unreachable` | last known list, `stale: true` |
| 200 with a body that cannot be read as a model list | `Unreachable` | `Unreachable` | last known list, `stale: true` |

Classification uses the error `type` in the response body first and falls back to the status code.

In `model_list`, a failed refresh also returns the reason as `problem` (`Unreachable`, `KeyRejected` or `NoCredit`), so the main window can tell a connection problem from a key that has stopped working (FR-036).

#### Recorded fixtures

One fixture per outcome that changes behavior, kept in `crates/steve-core/tests/fixtures/models/`. They are replayed to the real client by a local server, so the test also proves the two headers are sent.

| Fixture | Proves |
|---|---|
| `ok.json` | A list is parsed and unknown fields are tolerated |
| `ok-paged-1.json`, `ok-paged-2.json` | Pagination is followed |
| `401.json` | Rejected key |
| `403.json` | Key without permission |
| `402.json` | Billing problem |
| `429.json`, `529.json`, `500.json` | Transient service problems map to unreachable |
| `malformed.txt` | An unreadable success body does not store a key |

Timeout and connection failure need no fixture: one test points the client at a listener that never answers with a short limit, and one at a closed port.

Fixtures are recorded once from the real API by hand, with the key removed, and never refreshed by CI. No test reaches the real API.

#### End-to-end override

Builds compiled with the `e2e` cargo feature read the base URL from an environment variable, so the end-to-end run can point the app at a local server that replays these fixtures. Release builds do not contain the override.

## Checks

### Prerequisites

- Windows 11 with the WebView2 runtime (ships with the operating system)
- Rust 1.90 or later, Node.js 24, pnpm 10
- A Claude API key for the manual scenarios. The automated checks never use one.

### Setup

```bash
pnpm install
```

### Run in development

```bash
pnpm tauri dev
```

### Build the installer

```bash
pnpm tauri build
```

Expected: an installer under `target/release/bundle/nsis/`.

### Start from a clean state

To repeat a first-run scenario, quit the app from the tray, delete `%APPDATA%\mySteve\`, and remove the `mySteve` entry in Windows Credential Manager.

### Manual validation scenarios

Each scenario maps to a user story in the user stories above.

| # | Story | Steps | Expected |
|---|---|---|---|
| 1 | US1 | On a clean state, run the installer as a standard user. Paste a valid key. Press Next on each remaining step. | No administrator prompt. The key step reports the key as accepted. The main window shows "Steve" and the default portrait. Total time under 2 minutes. |
| 2 | US1 | Open `%APPDATA%\mySteve\workspace\` in a text editor. | `SOUL.md`, `IDENTITY.md`, `USER.md`, `MEMORY.md` and `avatar\manifest.json` exist and read as plain text. |
| 3 | US1 | Search `%APPDATA%\mySteve\` for the key's text. | No match. Credential Manager has a `mySteve` entry. |
| 4 | US2 | On a clean state, submit a made-up key of plausible length. | "Key not accepted" message. Credential Manager has no entry. Another key can be entered. |
| 5 | US2 | Disconnect the network and submit a valid key. Reconnect and press Retry. | "Could not reach the service" message that does not blame the key. After reconnecting, the key is accepted. |
| 6 | US2 | Submit an empty value, then `abc`. | An immediate message that this is not a complete key. No network request. |
| 7 | US3 | Pick each template. Edit the text, then pick another template. Clear the name. Paste text beyond the budget. | Each template's text appears. A confirmation precedes discarding edits. The name falls back to "Steve". The indicator warns and Next still works. |
| 8 | US3 | Choose a different character and fill in "about you". Finish. | The main window shows the chosen portrait. `USER.md` contains both values. |
| 9 | US4 | Quit from the tray on the soul step after typing some text. Relaunch. | The wizard opens on the soul step with the text intact. The key is still accepted. |
| 10 | US4 | Quit on the key step before any key is accepted. Relaunch. | The wizard opens on the key step. Credential Manager has no entry. |
| 11 | US5 | Close the window. Confirm the notice. Reopen from the tray. Close again. | The first close shows the notice once. The second close hides the window with no notice. The tray icon stays throughout. |
| 12 | US5 | Launch the app again from the Start menu while it is running. | The existing window comes to the front. Task Manager shows one mySteve process. |
| 13 | US5 | Choose Quit in the tray menu. | The process ends and the tray icon disappears. |
| 14 | US6 | Open settings. Replace the key with a made-up one, then with a valid one. | Only the last four characters are ever shown. The made-up key is rejected and the old key still works. The valid key replaces it. |
| 15 | US6 | Remove the key and confirm. | The app stays open. The main window says conversation is unavailable and offers to add a key. The wizard does not reappear. |
| 16 | US6 | Open the model setting online, pick another model, restart. Then open it offline. | The list comes from the provider with a default selected. The choice survives the restart. Offline, the last list is shown with an "may be out of date" note. |
| 17 | Edge | After setup, delete `MEMORY.md` and relaunch. | The file is recreated, the others are untouched, and the app says which file it recreated. |
| 18 | Edge | After setup, edit the name in `IDENTITY.md` in a text editor and relaunch. | The main window shows the edited name. |
| 19 | Edge | Uninstall, then install again. | The app opens straight to the main window with the same companion and the same key. Uninstalling removed neither the workspace, the saved state nor the key. |
| 20 | Edge | Quit. Delete `%APPDATA%\mySteve\state\` only. Relaunch and choose "Keep this companion". | The wizard opens with the stored key shown as accepted, then offers to keep the companion or start fresh. Keeping it changes no workspace file and shows the main window. |
| 21 | Edge | Repeat scenario 20, but choose "Start fresh", confirm, and finish the wizard. | The old workspace is moved to a dated backup folder next to it. A new workspace is created from the wizard's entries. |
| 22 | SC-009 | After setup, quit, disconnect the network and launch. Then reconnect and press Retry. | The window is usable within 5 seconds. Within 10 seconds it says the service cannot be reached. After reconnecting, Retry clears the message. |
| 23 | Accessibility | Complete the wizard, then open settings, with the keyboard only and Narrator on. | Every control in the wizard and in settings is reachable and announced. |

### What "done" looks like

- Scenarios 1 to 23 pass on a clean Windows 11 machine.
- The success criteria SC-001 to SC-011 above are met. SC-010 (five first-time users) is a release-readiness check, not a merge gate.

