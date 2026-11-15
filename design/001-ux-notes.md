# 001 App shell and onboarding: UX notes

Design file: [001-app-shell-onboarding.figcraft](001-app-shell-onboarding.figcraft) (FigCraft native format).
Source of truth: [the onboarding page](../docs/features/onboarding.md) and section 6.11 of [requirements.md](../docs/requirements.md).

**The design is at its second version: the companion leads each screen** (Direction A, decision D11, chosen on 2026-11-13). The webview implements it since 2026-11-14. "State of the webview" below records the choices made while building it.

## How the file is organised

| Page | What is on it |
|---|---|
| Foundations | `Theme` variable collection (Light and Dark modes, 18 colour tokens). `Dimensions` collection with two modes, Default and Soft (spacing `space/1`–`space/7`, radius `sm/md/lg/full`, type sizes, font families). Text styles `Type/*` (Segoe UI, Cascadia Mono for code and the soul editor, `Type/Voice` 16/24 for speech bubbles), paint styles `Color/*`, effect styles `Elevation/Card`, `Elevation/Dialog` and a two-tone `Focus ring`. One specimen frame per colour mode. |
| Components | Icon (Segoe Fluent Icons glyph), Button (Primary / Secondary / Danger / Subtle × Default / Focus / Disabled), Icon button, Text field, Text area, Select, Copy field, Companion line (Side / Stacked), Step dots, Alert (Error / Warning / Info / Success), Template option, Character option, Portrait / Pip (imported from `src/assets/characters/pip.svg`), Budget meter, Spinner, Loading (Inline / Panel), Dialog (Confirm / Notice), Window title bar, App bar (settings only), Wizard footer (First / Middle / Last), Tray menu. Each has a description that names the requirement it serves. |
| App shell | Frames at 960 × 720, the app's default window size: main window states, startup loading, tray menu, settings states. |
| Onboarding | Frames 01–21 left to right in flow order, with a prototype flow named "Onboarding: install to companion" starting at 01. |
| Exploration – compact window | Three frames for Direction B, a small frameless window. Not applied. See the last section. |

Everything is auto layout and every screen is built from component instances. Colours and component corner radii are bound to variables. Setting the `Theme` mode to Dark on any frame renders it in dark mode. All screens use the `Soft` mode of `Dimensions`, which sets the radii to 8 / 14 / 20. Spacing uses the values of the `space/*` scale but is not bound to the variables.

Removed in the second version, because no screen uses them any more: Step item, Step indicator, Empty state, and the Main and Wizard variants of App bar.

**No mobile frames.** The spec and plan target Windows 11 desktop only.

## Frame map: Onboarding page

Every wizard step opens with step dots and the companion's line in a speech bubble (PER-1, PER-2). The lines are listed under "The companion's lines".

| Frame | Shows | Spec |
|---|---|---|
| 01 Launch / Loading | Status loading before the app knows whether setup is complete | FR-002, SC-009 |
| 02 Key / Empty | The companion asks for the key. Key address with Copy, key field with a hint about where the key is stored, Check key, Next disabled | US1 AC1, FR-010, FR-005 |
| 03 Key / Incomplete value | The line says the key is incomplete. Field in its error state, "Fix the key" next to Check key. No service call | US2 AC4, PER-6 |
| 04 Key / Checking | Field disabled, button reads "Checking…", spinner with "This can take up to 10 seconds." Times out to 06 after 10 s | FR-011, US2 AC5, SC-003 |
| 05 Key / Rejected | The line says Claude didn't accept the key. Nothing stored. "Enter another key" next to Check key | US2 AC1, FR-012, FR-013 |
| 06 Key / Unreachable | The line says the key was not judged. The button reads "Retry". Also covers timeout and a dropped connection | US2 AC2, AC5, edge case "connection drops" |
| 07 Key / No credit | The line says the key is genuine and the account has no credit. The button reads "Try again" | US2 AC3, FR-012 |
| 08 Key / Credential storage unavailable | The line says Credential Manager refused the key and nothing is kept elsewhere. "Try again" | FR-015, edge case |
| 09 Key / Accepted | Plain status "Key accepted (ends in 7Qx2)." with "Use a different key". Next enabled | US1 AC2, FR-014 |
| 10 Key / Existing companion found | The line asks whether to keep the found companion. "Keep this companion" and "Start fresh" | FR-008, edge case "companion already exists" |
| 11 Key / Start fresh – confirm | Plain confirm dialog before moving the workspace to a backup folder | FR-008 |
| 12 Soul / Default | The bubble quotes the chosen template's sample line. Five templates on the left (Warm friend preselected). Name, editable text and budget meter on the right | US3 AC1, FR-016–FR-019, PER-9 |
| 13 Soul / Edited – switch template confirm | Plain confirm dialog before discarding edits | US3 AC2, FR-017 |
| 14 Soul / Over budget | The line says the text beyond the budget won't be used. Meter and count in warning colour. Next still enabled | US3 AC3, FR-019 |
| 15 Soul / Name cleared | Empty name field with the hint "Leave empty to use Steve." | US3 AC4, FR-018 |
| 16 Avatar / Default | Pip preselected as a still portrait | US3 AC5, FR-020 |
| 17 About you / Empty | The companion asks both questions. Both fields optional. Back, "Skip for now", "Meet Steve" | US3 AC7, FR-021, SC-002, PER-10 |
| 18 About you / Filled | "Sam", "English" | US3 AC6 |
| 19 About you / Finishing | Fields and actions disabled, inline progress | FR-022, FR-023 |
| 20 About you / Setup failed | The line names the folder that could not be written. "Retry" | Edge case "workspace cannot be written" |
| 21 Done / Main window | Greeting by time of day and name, "Steve is here", the first line inside the conversation area, Settings as a round icon button | US1 AC3, FR-024, PER-5 |

### Prototype

Happy path (US1, SC-002): 01 → 02 → Check key → 04 → (3 s) → 09 → Next → 12 → Next → 16 → Next → 17 → "Meet Steve" or "Skip for now" → 19 → (1.5 s) → 21. That is four confirmations after the key is accepted.

Branches you trigger from the keyboard while presenting:

- On **02**, press `0` for the incomplete-key problem (03).
- On **04**, press `1` for rejected (05), `2` for unreachable (06), `3` for no credit (07), `4` for storage unavailable (08). Each problem's button leads back to 02 or 04.
- On **09**, press `E` for the existing-companion branch (10). From there, Keep goes to 21 and Start fresh goes to 11, then to 12.
- On **12**, click Curious scholar to see the discard confirm (13), click the soul text for over budget (14), or click the name field for an empty name (15).
- On **17**, click the first field to see it filled (18).
- On **19**, press `F` for the setup problem (20). Retry goes back to 19.

Back buttons go to the previous step (FR-004, US4 AC4).

## Frame map: App shell page

| Frame | Shows | Spec |
|---|---|---|
| Main / Ready | Greeting, "Steve is here", the companion's first line in the conversation area, Settings icon button | FR-002, FR-024, US1 AC3, AC5, PER-5 |
| Main / No key | "Steve is waiting for a key". Line "I can't talk until you add an API key." with "Add a key" | FR-032, US6 AC4 |
| Main / Offline | "Steve is offline". The line says Claude can't be reached, with "Retry" | FR-033, SC-009, edge case "starts with no network" |
| Main / Key no longer accepted | "Steve can't reply". The line says Claude no longer accepts the key, with "Enter another key". Not presented as a connection problem | FR-036, US6 AC7 |
| Main / No credit | "Steve can't reply". The line says the account has no credit, with "Enter another key" | FR-036, edge case "stored key stops working" |
| Main / Files recreated | The line names the recreated files | Edge case "workspace file is missing" |
| Main / Close notice (first close) | In-window notice in the companion's voice, explaining the tray and how to quit, before hiding | FR-026, US5 AC1, AC2, PER-10 |
| App / Loading status | Window while the app status loads | FR-002 |
| Tray / Menu open | Tray context menu: Open mySteve, Settings, Quit | FR-025, FR-027, US5 AC3, AC5 |
| Settings / Key stored | "A key is stored, ending in 7Qx2.", Replace key, Remove key; model picker. The app bar shows the companion's name and portrait | US6 AC1, FR-014, FR-028, PER-1 |
| Settings / Replace key – entering | Field with focus, Check key | US6 AC2 |
| Settings / Replace key – checking | Disabled field and actions, "Checking…" | FR-011 |
| Settings / Replace key – rejected | Error alert; old key stays and says so | US6 AC3, FR-028 |
| Settings / Key replaced | Success, new last four | US6 AC2 |
| Settings / Remove key – confirm | Confirm dialog, focus on Cancel | US6 AC4, FR-028 |
| Settings / No key stored | Empty field and Check key | FR-032, US6 AC4 |
| Settings / Model list out of date | Current choice plus info "This list may be out of date." | US6 AC6, FR-031 |
| Settings / Model replaced (fallback) | Info naming the replaced model | US6 AC8, FR-030 |
| Settings / Model picker open | Provider list, default (Claude Opus 5.5, `claude-opus-5-5`) selected | US6 AC5, FR-029 |

Settings keep plain wording and alert boxes (PER-2, PER-7). Only their shapes are softer.

Requirements with no screen of their own: FR-001 (installer), FR-003 and US5 AC4 (a second launch brings the existing window forward), FR-034 (no telemetry), FR-035 (honouring hand edits), and US1 AC4 (files on disk).

## The companion's lines

Interface lines use one warm, neutral first-person voice, whatever soul is chosen (PER-3). A problem line gives the facts first and keeps its button (PER-6). All of them are in `src/strings.ts`, except "Main, ready": the chat runtime (spec 002) replaced it with the companion's first hello.

| Where | Line |
|---|---|
| 02 Key | "Hi! I'm almost here. I think with Claude, so I need your own Claude API key before we can talk. You can create one at the address below." |
| 03 Incomplete | "That doesn't look like a complete key. Paste the whole key, without spaces in the middle." |
| 04 Checking | "One moment, I'm checking your key with Claude." |
| 05 Rejected | "Claude didn't accept this key. Check that you copied all of it, or create a new one." |
| 06 Unreachable | "I couldn't reach Claude, so your key hasn't been checked yet. Check your internet connection, then I'll try again." |
| 07 No credit | "This key is genuine, but the account has no credit. Add credit in your Claude account, then I'll try again." |
| 08 Storage unavailable | "Your key was accepted, but Windows Credential Manager wouldn't store it. I won't keep it anywhere else." |
| 09 Accepted | "That worked, thank you. Next, let's decide who I am." |
| 10, 11 Existing companion | "I found Steve here from an earlier setup. Do you want to keep Steve, or start fresh?" |
| 12, 13, 15 Soul | "As a warm friend I'd sound like this: “…” Pick another personality to hear it, or rewrite me below." |
| 14 Over budget | "This is more than I can use. Text beyond 8,000 characters won't be used in conversation. You can still continue." |
| 16 Avatar | "Choose how I look. Right now there is one look, Pip, and it's already selected." |
| 17, 18 About you | "Two quick things, both optional. What should I call you? And which language would you like us to talk in?" |
| 19 Finishing | "One moment, I'm getting everything ready." |
| 20 Setup failed | "I couldn't save my files to (folder). Check that the folder can be written, then I'll try again." |
| Main, ready | "I'm all set up. We can't chat just yet: conversation arrives in the next version of mySteve, and I'll say hello first." |
| Main, no key | "I can't talk until you add an API key." |
| Main, offline | "I can't reach Claude right now, so I can't reply. Check your internet connection and I'll try again." |
| Main, key no longer accepted | "Claude no longer accepts your API key, so I can't reply. Enter another key and we can carry on." |
| Main, no credit | "Your Claude account has no credit left, so I can't reply. Add credit to the account, or enter another key." |
| Main, files recreated | "These files were missing, so I recreated them from defaults: (file names)." |
| Close notice | "I'll be right here in the system tray. To quit, right-click my icon and choose Quit." |

Other new or changed text: the greeting ("Good morning / afternoon / evening, Sam", or without the name when it isn't set), the presence line ("Steve is here", "Steve is offline", "Steve is waiting for a key", "Steve can't reply"), the key hint ("It's stored only in Windows Credential Manager, never in a file."), the labels "What Steve calls you" and "Language for our conversations", and the buttons "Skip for now" and "Meet Steve".

Sample lines per template (PER-9). They are fixed text, not model output, and the name follows the name field:

| Template | Sample line |
|---|---|
| Warm friend | "Hi, I'm Steve. I'm here to listen, and I'll remember what matters to you." |
| Curious scholar | "Hello, I'm Steve. Ask me anything. I'll probably ask you something back." |
| Cheerful sidekick | "Hey! Steve here. Ready when you are." |
| Calm mentor | "Hello. I'm Steve. Take your time, and we'll think it through together." |
| Blank | "Hi, I'm Steve. Who I am is up to you. Start writing." |

## Accessibility (FR-009, US1 AC6, PER-8)

- The companion's line is real text. The portrait next to it is decorative (`alt=""`). A line that reports a problem is announced with `role=alert`; other lines that change use `role=status`.
- Step dots are still an ordered list labelled "Setup steps". The caption carries the step number and name as text, and the current step is the wide dot, so position is not shown by colour alone.
- The Settings icon button has an accessible name and a tooltip with the same text.
- Buttons have a Focus variant using the shared `Focus ring` style, a 2 px surface gap plus a 2 px focus colour, so it shows on filled buttons too. Text field, Text area and Select show focus with a 2 px focus-colour border. Template and character cards don't have a drawn Focus variant yet. They should take the same `Focus ring` style when focused.
- Fields always have a visible label. A field in its error state references the companion's line (`aria-describedby`), so the message is announced with the field.
- Template and character cards are radio groups with a visible legend ("Personality", "Character").
- The budget count is a `role=status` live region.
- In Confirm dialogs, focus starts on Cancel, as `Confirm.tsx` does today. Notice dialogs focus OK.
- The Spinner should respect `prefers-reduced-motion`.

## State of the webview

### First version: implemented on 2026-11-13

`src/styles.css` carries the `Theme` and `Dimensions` tokens, with the Dark mode under `prefers-color-scheme: dark`. The first version's four proposals are in the webview (full-width wizard footer, two-column soul step, the proposed copy, a Primary button on "Start fresh"), along with the app bars, alerts, cards, radio cards, step indicator, modal dialogs and spinners. Choices made for the open questions:

- 1: the avatar step stays, with Pip preselected.
- 2: the minimum window size is unchanged (480 × 560). The soul step uses two columns from 880 px and one column below that.
- 3: the fallback note names the new model by its display name. The replaced model keeps its id, because it is no longer in the list.
- 9: the dark palette ships.
- 10: the app bar portrait is 48 px.
- 11: settings shows the last known model list when no key is stored.
- 4–8 and 12 are unchanged from the code: free-text language, an in-window close notice, no message when the stored key disappears, both no-credit screens, and resuming without a message.

### Second version: implemented on 2026-11-14

Choices made while building it:

- 18: the conversation fills the card. The companion's replies are bubbles beside a 40 px portrait, not the 88 px one, so a long conversation stays readable. The main window's problem lines and the chat's own failure, interruption and fresh-context offers are said the same way, as the latest entry in the conversation, with their buttons under the bubble.
- 13: one voice, as designed. The chat's failure messages were moved into the same first-person voice.
- 14: problems keep no icon or warning colour; they are announced with `role=alert`.
- 16: the presence line shows in every state. Without a "call me" name the greeting is "Good evening" alone. The name comes from `USER.md` on each status read, so a hand edit is honoured.
- The sample lines (PER-9) live in `strings.ts` with the other interface text, keyed by template id, rather than in the core's template list.
- At windows narrower than 640 px the wizard's portrait shrinks to 56 px and content no longer indents under the bubble. The soul step falls back to one column below 880 px, as before.
- Not done: the tray tooltip "Steve is here" (PER-10) and the portrait's reactions (PER-11, waiting on D10).

What the webview changed to match the design:

1. **Wizard.** Replace the app bar and numbered step indicator with step dots and a caption. Replace each step's heading and intro with the companion's line next to the portrait.
2. **Problems.** On the key step, in setup and in the main window, the problem is told in the companion's line, and the recovery button sits with the step's other buttons. Alert boxes stay only for the accepted-key status and in settings.
3. **Main window.** Replace the app bar with a greeting by local time of day and name, a presence line, and a round Settings icon button. The companion's line sits inside the conversation area.
4. **Soul step.** Order on the right is name, text, budget. The name hint shows only while the name is empty. The over-budget message moves into the companion's line.
5. **Shapes and colour.** Radii 8 / 14 / 20 in place of 4 / 8 / 12, an accent wash from the top of the window, and footer buttons with no bar behind them.
6. **Settings.** Unchanged except for the softer shapes and the companion's name and portrait in the app bar.
7. **Text.** The lines, labels and buttons listed under "The companion's lines". Tests that match the old strings need updating.
8. **Data.** The greeting needs the local time of day. The soul step needs a sample line for each template, which the core's template list does not carry yet.

## Open UX questions

Questions 1–12 are from the first version. Their answers are recorded under "First version" above.

1. **There is only one character.** `characters.json` bundles only Pip until licensing decision D10 is made. Should the avatar step still show with a single preselected choice, or be skipped until there are at least two?
2. **Minimum window size.** What is the minimum window size, and should the soul step fall back to a single scrolling column on narrow windows?
3. **How the model fallback names models.** Ids or display names?
4. **Preferred language as free text.** Would a picker with a free-text fallback avoid values like "eng" or "bahasa"?
5. **Close-to-tray notice.** An in-window dialog with OK, or a Windows toast after hiding?
6. **The stored key disappears during setup** (edge case). The wizard returns to the key step and keeps the other entries, but no message explains why. Should the companion say so?
7. **No credit at the key check.** The free check may never report no credit. Should Main / No credit be treated as the main place this message appears?
8. **Existing companion.** Confirm that the found companion's name can be read before setup completes, and what to show if `IDENTITY.md` itself is missing.
9. **Dark mode.** Should the dark palette ship in this slice?
10. **Portrait size in the main window.**
11. **Settings when no key is stored.** Disable the Model section, or show the last known list?
12. **Resuming the wizard (US4).** Should a one-time "welcome back" line be shown after relaunch?

New with the second version:

13. **One voice or one per template?** The design uses one neutral voice for interface lines (PER-3). A set of lines per template would feel more in character, but multiplies the text to write and translate, and can't cover a soul the user wrote.
14. **Problems without an alert box.** A problem is told only in the bubble, with no warning colour or icon. Is that enough, or should the bubble carry a small status icon so problems can be spotted at a glance?
15. **"Fix the key" and "Enter another key".** They sit as secondary buttons next to Check key. Are they still needed when the field is already marked and focused?
16. **Greeting and presence line.** When "about you" was skipped the greeting has no name. Is the presence line wanted in every state, and is its wording right ("Steve is waiting for a key", "Steve can't reply")?
17. **Existing companion wording.** The companion speaks about "Steve" in the third person while it is, in effect, Steve. Is that clear enough?
18. **Fit with the chat runtime.** Frame 21 assumes the conversation fills the card, with the companion's messages as bubbles next to the portrait. Check this against [the chat page](../docs/features/chat.md) before building either.
19. **Reactions need art.** PER-11 needs happy, concerned and asleep versions of Pip. That depends on decision D10.
20. **Font.** The design stays on Segoe UI. A rounder typeface would add warmth but would have to be bundled with the app.

## Direction B: a compact frameless window (not applied)

Three frames on the page "Exploration – compact window" show the key step, the about-you step and the main window in a 480 × 720 window with no native title bar and the portrait on top (PER-12).

| | Direction A (applied) | Direction B |
|---|---|---|
| What changes | Wording, layout and style values in the webview | Also the window: 480 × 720 by default, no native title bar, own close and settings buttons |
| Cost | No change to the shell | Custom drag region and window buttons. Gives up the native title bar's snapping and keyboard behaviour |
| Risk | Little | The soul step doesn't fit at 480 × 720 without scrolling or growing the window. Overlaps with floating mode (AVA-7) |

D11 leaves Direction B to be revisited with the avatar slice.
