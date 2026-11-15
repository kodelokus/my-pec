# Weather

Built.

Pec lives on the person's screen, but until now it had no idea what the day is like outside. This feature adds a third extension, Weather. The person tells My Pec their city once. From then on Pip shares the weather with them: rain falls around it and it ducks under a leaf, the sun shines and it squints, the moon comes out at night. Pec knows it too, so "should I go for a run?" gets an answer that fits the sky. It is the first extension that goes online, so it stays off until the person turns it on, and it sends nothing but the city they typed and that city's coordinates.

## User stories

### US1. Pip feels the weather (P1)

The person opens settings, turns on Weather under Extensions, and types "Jakarta". The app shows a few matching places; they pick "Jakarta, DKI Jakarta, Indonesia". Back in the main window, rain is falling around Pip and Pip holds a leaf over its head. A small line near Pip reads "Rain · 26°". In the evening the rain stops, the sky clears, and a moon and a few stars appear above Pip. The next morning the sun is out and Pip squints happily in its rays.

**Why this priority**: This is the feeling the person asked for: a companion that shares their day, not just their conversation. It works on its own, even if the companion never mentions the weather.

**How to check**: With recorded weather answers for each condition, turn the extension on, choose a place, and confirm that the weather around the character, Pip's reaction and the short weather line match the recorded condition, temperature and time of day; then switch the recorded answer and confirm the picture follows on the next refresh.

**Acceptance Scenarios**:

1. **Given** the Weather extension is off (as it is on a new install), **When** the person turns it on, **Then** settings asks for a city before anything is sent anywhere.
2. **Given** the person has typed a city name, **When** they search, **Then** they see up to five matching places, each with its region and country, and choose one; nothing else about them is sent.
3. **Given** a chosen place, **When** its current weather has been fetched, **Then** within ten seconds the weather is drawn above and around the character in the main window, and a short line shows the condition and the temperature.
4. **Given** the weather is shown, **When** it is daytime at the place, **Then** clear skies show sun rays, cloudy skies clouds, rain or drizzle falling drops, thunderstorms rain and an occasional flash, snow falling flakes and fog a haze; **When** it is night at the place, **Then** clear skies show a moon and stars instead of the sun.
5. **Given** the character is Pip, **When** the weather is shown, **Then** Pip reacts to it: it squints in bright sun, shelters under a leaf in rain, flinches at a thunder flash and shivers in the cold, while still showing its mood and what it is doing (listening, thinking, speaking).
6. **Given** the floating character is on, **When** the weather is shown, **Then** the floating character has the same weather and reactions, without the weather line and without getting in the way of dragging or clicking it.

### US2. Pec knows the weather in conversation (P1)

It's raining in the person's city. They write "mau jogging sore ini, gimana?" Pec answers that it's raining there right now, suggests waiting or running indoors, and does it without any visible lookup. Later, when the rain has stopped and the person asks again, Pec's answer follows the new weather.

**Why this priority**: A companion that can see the rain but talks as if the sun were out would break the feeling of the first story. Telling the companion the weather with each message is cheap and keeps the conversation true to the picture.

**How to check**: With a recorded weather answer and the fake model, send a message and confirm the request carries the current weather in the line with the local time; change the recorded weather, send another message, and confirm only the new message carries the new weather and nothing earlier in the conversation changed.

**Acceptance Scenarios**:

1. **Given** the weather is known and recent, **When** the person sends a message, **Then** the companion is told the place, the condition, the temperature and whether it is day or night there, together with the local date and time it already receives.
2. **Given** the weather is unknown, too old or the extension is off, **When** the person sends a message, **Then** the companion is told nothing about the weather, and it doesn't guess when asked; it says it doesn't know.
3. **Given** the weather changes between two messages, **When** the person sends the second, **Then** only that message carries the new weather; what the companion was told earlier is never rewritten (Principle III).
4. **Given** the weather is known, **When** the person talks about something unrelated, **Then** the companion doesn't bring up the weather unprompted in every reply; it mentions it when it matters to what they are talking about.

### US3. Online only by choice, and quiet when offline (P2)

The person turns Weather on, and settings says plainly what goes online: the city name they type, to look it up, and that place's coordinates, to get its weather, both to Open-Meteo, and nothing else. On a train with no signal, the laptop goes offline: Pip keeps showing the last weather it knew for a while, with no error message. After a few hours without an update the weather quietly disappears from around Pip, and comes back once the laptop is online again. When the person turns Weather off, the weather vanishes at once and the app stops going online for it; the chosen place stays in its file, ready if they turn it on again.

**Why this priority**: This is the first extension that reaches the network. The person must decide that, understand exactly what is sent, and never be bothered by its failures (Principle II, PER-8).

**How to check**: Capture network traffic while turning the extension on, choosing a place and letting it refresh, and confirm only the two named destinations are reached and only with the typed name or the coordinates; cut the network and confirm the last weather stays, no error appears and it disappears once too old; turn the extension off and confirm no further requests.

**Acceptance Scenarios**:

1. **Given** a new install, **When** the app runs, **Then** it makes no weather request of any kind until the person turns the extension on and chooses a place.
2. **Given** the extension is on, **When** the person reads its settings, **Then** they see which service is used, exactly what is sent and when, where the place is kept, the time of the last update, and the data credit the service asks for.
3. **Given** the weather can't be fetched (offline, the service down or refusing), **When** a refresh fails, **Then** no notice, alert or error appears outside settings, the last known weather keeps showing while it is less than three hours old, and chat is unaffected.
4. **Given** the last known weather is three hours old or older, **When** the window shows the character, **Then** no weather is drawn and the companion is told nothing about the weather, until a fresh update arrives.
5. **Given** the extension is on, **When** the person turns it off, **Then** the weather disappears from every window at once, no further requests are made, and the place file is left untouched.
6. **Given** a city that the lookup can't find, **When** the person searches, **Then** settings says no place was found and nothing else changes.

### US4. Every character and every person can enjoy it (P3)

The person uses a Live2D character instead of Pip. The weather is drawn above and around it just the same, though the character itself doesn't react. A person who has turned on "reduce motion" in Windows sees a still picture of the weather (a sun, a cloud, a few drops) instead of falling rain and flashes. A screen reader announces the weather line once when the weather changes, not every animation frame.

**Why this priority**: Weather is a pleasure, not a function, so it must never cost anyone comfort or access (PER-8), and it shouldn't depend on which character the person picked.

**How to check**: Switch to a Live2D character and confirm the weather layer shows with no character reactions; turn on reduced motion and confirm nothing moves; check with a screen reader that the weather line is real text and is announced only when it changes.

**Acceptance Scenarios**:

1. **Given** a Live2D character, **When** the weather is shown, **Then** the weather layer is drawn above and around it, and the character's own motions are unchanged.
2. **Given** reduced motion is on, **When** the weather is shown, **Then** it is a still picture with no falling, flashing or drifting.
3. **Given** a screen reader, **When** the weather changes, **Then** the weather line is announced once; the drawn weather itself is hidden from assistive technology.
4. **Given** any weather, **When** it is drawn, **Then** the character's face stays visible and every control near the character remains readable and usable.

## Edge cases

- The person types a city with many places of the same name ("Springfield"): the list shows each with its region and country, at most five, and nothing is chosen until they pick one.
- The person edits the place file by hand: the next refresh uses the new place if it can be read; if it can't, no weather is shown and settings says the place couldn't be read, leaving the file as it is.
- The chosen place is in another time zone from the computer: day and night follow sunrise and sunset at the place, not the computer's clock; the local time the companion is told stays the computer's.
- The computer wakes from sleep: the weather refreshes soon after waking, rather than showing hours-old weather until the next scheduled refresh.
- The service limits or refuses requests: the app waits longer before trying again and never retries more than once every ten minutes.
- The weather is known but the character is asleep while the companion dreams: the weather is still drawn, and Pip sleeps through it rather than reacting.
- The person changes the place: the old weather disappears at once and the new place's weather is fetched immediately.
- Turning the extension off and on again within a few minutes reuses the last weather if it is still recent, without a new request.
- The weather line and the temperature use the units the person chose (Celsius by default).
- A context that started before the weather was known, or before this feature existed, keeps its system prompt unchanged; the weather only ever arrives in new messages.

## Requirements

**The extension**

- **FR-001**: The app MUST ship a third bundled extension, Weather, that is off until the person turns it on, unlike Sticky notes and Alarms (Principle II: a feature that adds a network destination is off until the person turns it on).
- **FR-002**: Weather MUST give the companion no tools; it reaches the conversation only through the weather line of FR-016.
- **FR-003**: The chosen place MUST be kept as one plain file in the companion's workspace, readable and editable in any text editor, holding the place's name, region, country and coordinates; the last known weather and its time are bookkeeping, kept outside the workspace and safe to lose.
- **FR-004**: Turning Weather off MUST stop every request at once, remove the weather from every window and from what the companion is told, and leave the place file untouched.

**Choosing a place**

- **FR-005**: Settings MUST let the person type a city name, search, and choose one of at most five matching places, each shown with its region and country. Nothing is saved or fetched until they choose.
- **FR-006**: Settings MUST say, while the extension is on: the service used, exactly what is sent (the typed name when searching, the chosen place's coordinates when refreshing) and when, where the place is kept, when the weather was last updated, the units, and the data credit the service asks for.
- **FR-007**: The person MUST be able to choose Celsius or Fahrenheit; Celsius is the default.

**Network**

- **FR-008**: Weather MUST reach only two destinations, both named here: Open-Meteo's place lookup (`geocoding-api.open-meteo.com`) with the typed name only, and Open-Meteo's weather (`api.open-meteo.com`) with the chosen place's coordinates only. Requests MUST carry no key, account, identifier, cookie or other data about the person.
- **FR-009**: Requests MUST be made by the core, never by the webview (Principle II), and MUST NOT be made while the extension is off or no place is chosen.
- **FR-010**: The weather MUST be fetched when the person chooses a place, then about every 30 minutes while the app runs, and soon after the computer wakes, but never more than once every ten minutes; after a failure the app MUST wait longer before trying again.

**The weather**

- **FR-011**: The current weather MUST consist of a condition (clear, partly cloudy, cloudy, fog, drizzle, rain, heavy rain, thunderstorm, snow), the temperature, whether it is day or night at the place, and the time it was observed.
- **FR-012**: The last known weather MUST be kept and shown while it is less than three hours old; older weather MUST be neither drawn nor told to the companion.
- **FR-013**: A failed lookup or refresh MUST NOT show any notice, alert or error outside the extension's settings, and MUST NOT affect chat (Principle V).

**Drawing it**

- **FR-014**: While the weather is known and recent, it MUST be drawn above and around the character in the main window and on the floating character: sun rays on a clear day, a moon and stars on a clear night, clouds, falling drops for drizzle and rain (heavier for heavy rain), rain with an occasional flash for thunderstorms, falling flakes for snow and a haze for fog. It MUST NOT hide the character's face, and on the floating character it MUST NOT get in the way of dragging or clicking.
- **FR-015**: Pip MUST react to the weather while keeping its mood and activity: squinting in sun, sheltering under a leaf in drizzle, rain and thunderstorms, flinching at a thunder flash, and shivering when it is cold (below 10 °C). Asleep, Pip doesn't react. Live2D characters get the weather drawn around them but no reactions in this release.
- **FR-016**: The main window MUST show a short weather line near the character, such as "Rain · 26°", as real text that a screen reader announces once when it changes; the drawn weather MUST be hidden from assistive technology (PER-8).
- **FR-017**: With reduced motion on, the weather MUST be a still picture: nothing falls, flashes or drifts (PER-8).

**The conversation**

- **FR-018**: While the weather is known and recent, each message the person sends MUST tell the companion the place, the condition, the temperature in the chosen units and whether it is day or night there, alongside the local date and time it already receives. When the weather isn't known or recent, or the extension is off, nothing about weather is added.
- **FR-019**: The weather MUST arrive only in new messages; nothing the companion was told earlier is changed when the weather changes (Principle III).
- **FR-020**: The companion MUST be told how to use the weather: mention it when it matters to the conversation, don't bring it up in every reply, and when nothing about weather was given, say it doesn't know rather than guess.
- **FR-021**: The weather MUST NOT be stored as memory by the app itself, indexed by memory search or given to dreaming; the companion may still note something about it, like anything else the person talks about.

**Changes to existing requirements**

- **FR-022**: EXT-2 ("an extension never reaches the network") MUST change to: an extension may reach the network only through the core, only while it is on, and only the destinations its spec names. SC-5 ("nothing leaves the machine except to the configured LLM endpoint and the update server") MUST list the Weather destinations as allowed while Weather is on.

## Key entities

- **Place**: the one place whose weather is shown: its name, region, country and coordinates, kept as a plain file in the workspace that the person can read and edit.
- **Current weather**: the condition, temperature, day or night and the time it was observed, for the chosen place; bookkeeping that is safe to lose and is considered gone after three hours.
- **Weather line**: the short text near the character and the line the companion receives with each message, both made from the current weather.
- **Weather layer**: what is drawn above and around the character for each condition and time of day, still when reduced motion is on.
- **Pip's reaction**: how Pip responds to each condition (squint, shelter, flinch, shiver), shown together with its mood and activity.

## Success criteria

- **SC-001**: After choosing a place while online, the weather appears around the character within 10 seconds.
- **SC-002**: When the weather at the place changes, the picture and what the companion is told follow within 40 minutes while the app runs, and within 2 minutes of the computer waking.
- **SC-003**: A network capture of a full day with Weather on shows requests only to the two named destinations, carrying only the typed city name or the chosen coordinates, and no more than one weather request every 10 minutes; with Weather off, a capture shows no weather request at all.
- **SC-004**: During a day offline, no weather error, notice or alert ever appears outside settings, chat works as before, and the weather disappears from the picture no later than 3 hours after the last update.
- **SC-005**: Asked 10 weather-dependent questions ("do I need an umbrella?", "is it a good evening for a walk?") with recorded weather, the companion's answers agree with the given weather in all 10, and with Weather off it says it doesn't know in all 10.
- **SC-006**: With rain animating, the app's processor and memory use while idle stay within the budgets for an idle app (NFR-PERF); with reduced motion on, nothing on screen moves.
- **SC-007**: A person who has never used the extension can turn it on, find their city and see the weather around Pip in under one minute, without help.

## Assumptions

- Open-Meteo is used because it needs no account or key, so the person sets nothing up beyond a city (Principle VII). Its data is free for non-commercial use under an attribution license; whether My Pec's distribution counts as commercial must be checked before release, and settings shows the credit the license asks for.
- One place at a time is enough for this release; several places, forecasts and weather warnings are out of scope.
- The place comes only from what the person types. No location service, IP address or automatic detection is used.
- "Cold" for Pip's shiver is below 10 °C, a threshold that suits both tropical and temperate places; it can be tuned later without changing the requirements.
- The weather line in the conversation is a short factual line; the companion's soul decides how it talks about the weather.
- The three-hour limit balances showing something on a short offline stretch against showing weather that is clearly wrong.
- Refreshing about every 30 minutes keeps requests far below the service's free limits for one person.

## Dependencies

- Principle II (docs/principles.md) lists the allowed outbound connections; it already lets a feature add a destination that its spec names and that is off until the person turns it on. The plan must confirm whether the list in its first rule needs a matching amendment.
- The extension framework of the extensions page (turning extensions on and off, the place in the workspace, bookkeeping in `state/`, the tick).
- The character drawing and Pip's moods and activities of the avatar page, including the floating character. Pip's artwork is being changed in parallel, so building the reactions waits until that work is committed.
- The local date and time line the companion receives with each message (chat runtime, CHAT-5).

## Decisions

### R1. Network through the core, limited to the hosts an extension declares

**Decision**: The core owns every request. `pec_core::ext` gains a `Fetch` trait (one method: a GET returning JSON) held by the `Core`, and `ExtDir` gains `fetch(host, path, query)`. An extension declares its hosts (`fn hosts(&self) -> &'static [&'static str]`, empty by default); `ExtDir::fetch` refuses any other host, anything but HTTPS and any call while the extension is off. The real `Fetch` uses `reqwest` with its `blocking` feature, a 5-second timeout and a 64 KB response limit; tests give the core a fake that answers from recorded JSON (`Core::with_fetch`), for the same reason the LLM is faked: no test may reach the network.

**Rationale**: Principle II keeps the webview off the network and says every destination is named and off until turned on; doing it in one place in the core makes both checkable by one test. `reqwest` is already the core's client, so `blocking` is a feature flag, not a new crate. The tick thread and the search command are plain threads, where a blocking client is the simplest fit.

**Alternatives considered**: Fetching in the webview (forbidden by Principle II). An async trait method (the extension tick is synchronous, R4). `ureq` (a second HTTP client for the same job).

### R2. Open-Meteo requests

**Decision**:

- Place lookup: `GET https://geocoding-api.open-meteo.com/v1/search?name=<typed name>&count=5&format=json`. The answer's `results` give `name`, `admin1` (region), `country`, `latitude`, `longitude`.
- Current weather: `GET https://api.open-meteo.com/v1/forecast?latitude=<lat>&longitude=<lon>&current=temperature_2m,weather_code,is_day`. The answer's `current` gives `time`, `temperature_2m` (°C), `weather_code` (WMO), `is_day` (0 or 1).
- No `language`, `timezone` or other parameter is sent: nothing beyond the typed name or the coordinates (FR-008). Coordinates are sent as stored, rounded to two decimals (about a kilometre), which is enough for weather and says less about where exactly the person picked.
- Temperatures are always fetched in Celsius and converted for display, so changing units needs no request.
- Settings shows the credit the data license asks for: "Weather data by Open-Meteo.com" (CC BY 4.0).

**Rationale**: No account or key, so no setup beyond a city (Principle VII). The free tier allows far more than one person's requests (R4).

**Alternatives considered**: Services that need a key (a second key contradicts the Technology Constraints' spirit and Principle VII). Windows' own weather (no public API).

**Open before release**: Open-Meteo's free service is for non-commercial use. Whether a free, open-source app published by a company counts must be checked; if not, the paid plan or another service is a change to this research only (the Assumptions).

### R3. Conditions from weather codes

**Decision**: A fixed table maps WMO weather codes to the conditions of the requirements:

| Codes | Condition |
|---|---|
| 0, 1 | clear |
| 2 | partly cloudy |
| 3 | cloudy |
| 45, 48 | fog |
| 51, 53, 55, 56, 57 | drizzle |
| 61, 63, 66, 80, 81 | rain |
| 65, 67, 82 | heavy rain |
| 71, 73, 75, 77, 85, 86 | snow |
| 95, 96, 99 | thunderstorm |

Any other code is treated as no weather (nothing drawn, nothing told), not as an error.

### R4. When to refresh

**Decision**: The weather extension's `tick` (every 15 s, like alarms) refreshes when the extension is on, a place is readable, and one of these holds: there is no weather for this place yet; the last success is 30 minutes old; the place changed; or the computer just woke (more than two minutes of real time since the last tick, the same signal alarms use). It never fetches within 10 minutes of the last attempt, and after a failure it waits 10, 20, 40 and then at most 120 minutes. The fetch runs inside the tick, with the 5-second timeout of R1; Weather comes after Alarms in `pec_ext::all()`, so an alarm due in the same tick is collected first, and the shell rings it at most 5 seconds late, well within the one minute alarms allow.

**Rationale**: About 48 requests a day, far under the service's limits; the wake rule meets SC-002's two minutes; no extra thread or timer in the shell.

**Alternatives considered**: A separate timer thread in the shell (more moving parts, and Principle VI keeps the rules in the core). Fetching only when the window is shown (the companion still needs the weather in chat while the window is hidden).

### R5. Telling the companion

**Decision**: The `Extension` trait gains `fn context_line(&self, dir: &ExtDir, now: OffsetDateTime) -> Option<String>` (none by default). When a message is sent, the core adds the line of each extension that the current context recorded and that is still on, inside the `<context>` block after the local time, for example: `Weather in Jakarta, Indonesia: rain, 26 °C, daytime.` The line is part of the stored message, so a rebuilt request is identical (Principle III). Weather's `SKILL.md` tells the companion what the line means and how to use it (FR-020); it is recorded in each context with the other SKILL.md text, so no harness version change is needed. A context that started before Weather was on gets no weather line and offers to catch up, as for any extension.

**Rationale**: No tool call, no extra request, nothing rewritten. Restricting the line to recorded extensions keeps the instructions and the line together: the companion never gets a weather line it wasn't told how to read.

**Alternatives considered**: A `weather_now` tool (the requirements ask for no tool; it costs a round trip per question). Putting the weather in the system prompt (changes mid-epoch, Principle III).

### R6. Drawing the weather

**Decision**: A `WeatherLayer` React component draws each condition with inline SVG and CSS animation (only `transform` and `opacity`), positioned over the avatar's box with `pointer-events: none` and `aria-hidden`. `AvatarView` takes an optional `weather` prop; only the main window and the floating window pass it, so the portraits beside chat lines stay plain. Rain uses at most 24 drops and snow 16 flakes; animations pause while the page is hidden; with reduced motion (the hook `AvatarView` already uses) the layer is a still picture. The main window shows the weather line as text with `role="status"`, announced once when it changes. Day and night come from the service's `is_day` for the place.

**Rationale**: No new dependency; CSS animation of a few elements keeps an idle app within NFR-PERF (SC-006). One component serves Pip and Live2D characters alike.

**Alternatives considered**: A canvas particle system (more code, harder to make accessible and still). Drawing the weather inside each renderer (would repeat it for Live2D).

### R7. Pip's reactions through the renderer interface

**Decision**: `Shown` (what `AvatarView` passes to `AvatarRenderer.setState`) gains `weather: Reaction | null`, where a reaction is `squint`, `shelter`, `flinch` or `shiver`, chosen in one place from the condition, the temperature and day or night (sun on a clear day: squint; drizzle, rain, heavy rain: shelter; thunderstorm: shelter, and flinch for a moment at each flash; below 10 °C: shiver, combined with shelter if it also rains). Pip's renderer shows it through a data attribute and its stylesheet (a leaf over the head, narrowed eyes, a shake), together with mood and activity; asleep, it shows none. The portrait and Live2D renderers ignore it. This waits for the parallel rework of Pip's artwork (`pip.ts`, `pip.css`, `pip.svg`) to be committed.

**Rationale**: Principle VI: the interface drives the avatar only through `AvatarRenderer`; one optional field keeps every renderer working unchanged.

### R8. Settings

**Decision**: Under Extensions, the Weather row, once on, opens a small panel: a city field with a Search button, up to five results to choose from, the chosen place, Celsius or Fahrenheit, the last update time ("Updated 14:05", or "Couldn't update; trying again later" in plain text, never an alert), what is sent and to whom, where the place is kept, and the data credit. Search runs on a blocking thread through the core (R1); the panel sends one search at a time.

### R9. Documents and the principles

**Decision**: Principle II's first rule lists the allowed outbound connections; it gains "and destinations that a feature's spec names, only while the person has that feature on", which the rule after it already assumes. This widens the rule, so it is a MINOR amendment, 1.2.1 to 1.3.0, for the owner to approve before building. `docs/requirements.md` changes EXT-2 and SC-5 (FR-022) and adds the weather items to section 6.13; `docs/architecture.md` adds Weather to its extensions section, `weather.md` to the workspace files, and the network rule of R1.

## Data

### Data 1. `weather.md` (workspace)

The chosen place, written by settings and editable by hand.

```markdown
# Weather

- Place: Jakarta, DKI Jakarta, Indonesia
- Latitude: -6.21
- Longitude: 106.85
- Units: celsius
```

| Field | Rule |
|---|---|
| Place | 1 to 120 characters, one line: name, then region and country when known, joined by ", ". |
| Latitude | A number from -90 to 90, at most two decimals. |
| Longitude | A number from -180 to 180, at most two decimals. |
| Units | `celsius` or `fahrenheit`; missing means `celsius`. |

- Other lines are kept as they are when settings rewrites the file; the four fields are matched by their label, in any order, case-insensitively.
- A file without a readable Place, Latitude and Longitude means no place: nothing is fetched, settings says the place couldn't be read, and the file is left untouched until the person chooses a place.
- Written atomically under the writer's lock, like the other extension files.

### Data 2. `state/weather.json` (bookkeeping, safe to lose)

```json
{
  "for": "-6.21,106.85",
  "now": { "code": 61, "temperatureC": 26.3, "day": true, "observed": "2026-11-15T14:00", "fetched": 1794731100 },
  "lastTry": 1794731100,
  "failures": 0,
  "lastTick": 1794731115
}
```

| Field | Meaning |
|---|---|
| `for` | The coordinates the weather belongs to; a different place in `weather.md` makes `now` stale at once. |
| `now` | The last weather fetched: WMO code, temperature in Celsius, day or night, the service's observation time, and when it was fetched (Unix seconds). |
| `lastTry` | When the last request was made, successful or not (R4's 10-minute floor). |
| `failures` | Failures in a row, for the wait after a failure (R4). |
| `lastTick` | Unix seconds of the last tick, to notice that the computer woke (R4). |

`now` counts as recent while `fetched` is less than three hours ago (FR-012).

### Data 3. Types shared with the webview (generated)

```ts
type WeatherCondition = "clear" | "partlyCloudy" | "cloudy" | "fog" | "drizzle" | "rain" | "heavyRain" | "snow" | "thunderstorm";
type Units = "celsius" | "fahrenheit";

/** A place found by a search, or the chosen one. */
type Place = { name: string; region: string | null; country: string | null; latitude: number; longitude: number };

/** The weather to draw and to say; absent when not known or not recent. */
type WeatherView = {
  place: string;              // "Jakarta, DKI Jakarta, Indonesia"
  condition: WeatherCondition;
  temperature: number;        // rounded, in `units`
  units: Units;
  day: boolean;
};

/** What the Weather panel in settings shows. */
type WeatherSettings = {
  place: Place | null;
  units: Units;
  unreadable: boolean;        // weather.md exists but its place can't be read
  updated: string | null;     // local "HH:MM" of the last success, if recent
  failing: boolean;           // the last attempt failed
};
```

### Data 4. Pip's reaction (webview only)

`Reaction = "squint" | "shelter" | "flinch" | "shiver"`, a list of zero to three (a cold thunderstorm has shelter, flinch and shiver), chosen from a `WeatherView` (R7) and passed to the renderer in `Shown.weather`.

### Data 5. The context line

Inside the `<context>` block of a new message, after the local time, while the weather is recent and the current context recorded Weather:

```
Weather in Jakarta, DKI Jakarta, Indonesia: rain, 26 °C, daytime.
```

The condition is in plain English words (`clear`, `partly cloudy`, `cloudy`, `fog`, `drizzle`, `rain`, `heavy rain`, `snow`, `thunderstorm`), the temperature in the chosen units, and `daytime` or `night`.

## Interfaces

### Commands, events and requests

Same rules as the earlier features (onboarding to extensions): thin handlers, validation in `pec-ext` and the core, expected failures as values, each command in `build.rs` and allowed in the capability file of each window that may call it, and `src/api/commands.ts` as the only caller. Commands that reach the network run on a blocking thread, never on the main thread.

#### New commands

| Command | Windows | Input | Output | Behavior | Requirements |
|---|---|---|---|---|---|
| `weather_now` | main, floating | — | `WeatherView \| null` | The weather to draw, or null when Weather is off, no place is chosen, or the last weather isn't recent. No request is made. | FR-012, FR-014 |
| `weather_settings` | main | — | `WeatherSettings` | What the Weather panel shows. | FR-006 |
| `weather_search` | main | `name` | `Place[] \| null` | Looks the name up (1 to 100 characters, trimmed) and returns at most five places. `InvalidInput("name")` for an empty or overlong name, `InvalidInput("weather")` while Weather is off, null when the service can't be reached, an empty list when nothing matches. The panel sends one search at a time. | FR-005, FR-008 |
| `weather_choose` | main | `place: Place` | `WeatherSettings` | Validates the coordinates, writes `weather.md` (keeping its other lines), forgets the old weather and fetches the new place's weather at once. | FR-003, FR-005, FR-010 |
| `weather_units` | main | `units` | `WeatherSettings` | Writes the units to `weather.md`; no request. | FR-007 |

The existing `extensions` and `set_extension` commands list and switch Weather like the other extensions; Weather's default is off. Turning it off makes `weather_now` return null at once and stops requests.

#### Events

`ext_changed` with `"weather"` after a refresh changed the weather, after `weather_choose`, after `set_extension` for Weather, and when the weather stops being recent. Windows that draw the weather call `weather_now` again.

#### Capabilities

`main.json` allows all five commands. `floating.json` allows `weather_now` only. Note windows get none.

#### Requests to Open-Meteo (the only network destinations of this feature)

| When | Request | Sent |
|---|---|---|
| The person searches | `GET https://geocoding-api.open-meteo.com/v1/search?name=<name>&count=5&format=json` | The typed name. |
| A refresh is due (R4) | `GET https://api.open-meteo.com/v1/forecast?latitude=<lat>&longitude=<lon>&current=temperature_2m,weather_code,is_day` | The chosen place's coordinates, two decimals. |

No headers beyond what the HTTP client sends by default, no cookies, no key. `ExtDir::fetch` refuses any other host and any request while Weather is off. Answers are read as JSON of at most 64 KB; anything unexpected is treated as a failed refresh (FR-013).

#### The context line

When a message is sent and the current context recorded Weather (R5), the `<context>` block ends with the line of Data 5 while the weather is recent. Weather's `SKILL.md` explains the line to the companion.

## Checks

How to check this slice works. The automated part never reaches the network; the scenarios use the real service on the owner's machine.

### Manual scenarios (owner's machine)

1. On a fresh profile, run the app for five minutes with a network capture: no request to Open-Meteo.
2. Turn Weather on, search "Jakarta", choose it: the weather appears around Pip within 10 seconds, with the weather line. Settings shows what is sent, the update time and the credit.
3. Ask "do I need an umbrella today?": the answer fits the weather shown, with no tool call on the reply.
4. Turn on the floating character: it has the same weather and stays draggable and clickable.
5. Turn on reduced motion in Windows: the weather holds still.
6. Disconnect the network for four hours: no error appears; after three hours the weather disappears; reconnecting brings it back within 30 minutes.
7. Sleep the computer for an hour: after waking, the weather updates within two minutes.
8. Edit `weather.md` to another city's coordinates: the next refresh shows that place; break the file: no weather, and settings says the place couldn't be read.
9. Turn Weather off with a network capture running: the weather disappears and no further request is made.
10. Switch to a Live2D character: the weather is drawn around it, with no reactions.

### Release checks (owner)

| Check | How | Pass |
|---|---|---|
| SC-003 | Network capture of a day with Weather on, then an hour with it off. | Only the two named hosts, only the name or the coordinates, at most one weather request in 10 minutes; nothing while off. |
| SC-006 | Task Manager with rain animating and the window idle. | Within NFR-PERF's idle budget. |
| License | Open-Meteo terms for the way My Pec is distributed. | Allowed, or a plan for the paid tier or another service. |
