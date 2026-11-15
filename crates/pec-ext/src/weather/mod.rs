//! Weather: the current weather at the place the person chose, from Open-Meteo, drawn around the character and
//! told to the companion with each new message (weather feature). Off until the person turns it on.

mod codes;

pub use codes::{Units, WeatherCondition};

use pec_core::ext::{ExtDir, Extension, FetchError, Tick, ToolAnswer, write_atomic};
use pec_core::llm::ToolSpec;
use pec_core::{AppError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use time::{OffsetDateTime, PrimitiveDateTime};
use ts_rs::TS;

const LOOKUP: &str = "geocoding-api.open-meteo.com";
const FORECAST: &str = "api.open-meteo.com";
const HEADER: &str = "# Weather\n";
/// weather FR-012: older weather is neither drawn nor told.
const RECENT: i64 = 3 * 3600;
/// weather R4: a refresh every 30 minutes, never within 10 of the last try, waiting longer after failures.
const EVERY: i64 = 30 * 60;
const FLOOR: i64 = 10 * 60;
const LONGEST_WAIT: i64 = 120 * 60;
/// More real time than this between two ticks means the computer slept.
const WOKE: i64 = 2 * 60;

/// A place found by a search, or the chosen one (weather data 3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Place {
    pub name: String,
    pub region: Option<String>,
    pub country: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}

impl Place {
    /// "Jakarta, DKI Jakarta, Indonesia".
    pub fn label(&self) -> String {
        [
            Some(&self.name),
            self.region.as_ref(),
            self.country.as_ref(),
        ]
        .into_iter()
        .flatten()
        .filter(|p| !p.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
    }

    /// The coordinates the weather is asked for and kept under: two decimals, about a kilometre (weather R2).
    fn key(&self) -> String {
        format!("{:.2},{:.2}", self.latitude, self.longitude)
    }

    fn valid(&self) -> bool {
        let name = self.label();
        (1..=120).contains(&name.chars().count())
            && !name.contains(['\n', '\r'])
            && (-90.0..=90.0).contains(&self.latitude)
            && (-180.0..=180.0).contains(&self.longitude)
    }
}

/// The weather to draw and to say; absent when it isn't known or recent (weather data 3).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct WeatherView {
    pub place: String,
    pub condition: WeatherCondition,
    pub temperature: i32,
    pub units: Units,
    pub day: bool,
}

/// What the Weather panel in settings shows.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct WeatherSettings {
    pub place: Option<Place>,
    pub units: Units,
    /// `weather.md` exists but its place can't be read.
    pub unreadable: bool,
    /// Local `HH:MM` of the last success, while it is recent.
    pub updated: Option<String>,
    /// The last attempt failed.
    pub failing: bool,
}

/// `weather.md`, as far as it could be read.
struct PlaceFile {
    place: Option<Place>,
    units: Units,
    unreadable: bool,
}

fn field<'a>(text: &'a str, label: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let (key, value) = line.trim().strip_prefix("- ")?.split_once(':')?;
        key.trim().eq_ignore_ascii_case(label).then(|| value.trim())
    })
}

fn read(dir: &ExtDir) -> PlaceFile {
    let Ok(text) = std::fs::read_to_string(dir.place()) else {
        return PlaceFile {
            place: None,
            units: Units::default(),
            unreadable: false,
        };
    };
    let units = match field(&text, "units")
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("fahrenheit") => Units::Fahrenheit,
        _ => Units::Celsius,
    };
    let number = |label| field(&text, label)?.parse::<f64>().ok();
    let place = match (
        field(&text, "place"),
        number("latitude"),
        number("longitude"),
    ) {
        (Some(name), Some(latitude), Some(longitude)) => Some(Place {
            name: name.into(),
            region: None,
            country: None,
            latitude,
            longitude,
        })
        .filter(Place::valid),
        _ => None,
    };
    PlaceFile {
        unreadable: place.is_none(),
        place,
        units,
    }
}

/// Writes the place and units, keeping every other line the person wrote (weather data 1).
fn write(dir: &ExtDir, place: Option<&Place>, units: Units) -> Result<()> {
    let old = std::fs::read_to_string(dir.place()).unwrap_or_else(|_| format!("{HEADER}\n"));
    let mut fields: Vec<(&str, String)> = vec![(
        "Units",
        match units {
            Units::Celsius => "celsius".into(),
            Units::Fahrenheit => "fahrenheit".into(),
        },
    )];
    if let Some(place) = place {
        fields.splice(
            0..0,
            [
                ("Place", place.label()),
                ("Latitude", format!("{:.2}", place.latitude)),
                ("Longitude", format!("{:.2}", place.longitude)),
            ],
        );
    }
    let mut lines: Vec<String> = old.lines().map(Into::into).collect();
    for (label, value) in fields {
        let line = format!("- {label}: {value}");
        let at = lines.iter().position(|l| {
            l.trim()
                .strip_prefix("- ")
                .and_then(|l| l.split_once(':'))
                .is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case(label))
        });
        match at {
            Some(i) => lines[i] = line,
            None => lines.push(line),
        }
    }
    write_atomic(dir.place(), (lines.join("\n") + "\n").as_bytes())
}

/// Bookkeeping in `state/weather.json`, safe to lose (weather data 2).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct State {
    #[serde(rename = "for")]
    place: Option<String>,
    now: Option<Now>,
    last_try: Option<i64>,
    failures: u32,
    last_tick: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Now {
    code: u32,
    temperature_c: f64,
    day: bool,
    observed: String,
    fetched: i64,
}

impl State {
    /// The weather of `place`, while it is recent.
    fn recent(&self, place: &Place, unix: i64) -> Option<&Now> {
        self.now
            .as_ref()
            .filter(|n| self.place.as_deref() == Some(&place.key()) && unix - n.fetched < RECENT)
    }
}

fn view(dir: &ExtDir, unix: i64) -> Option<WeatherView> {
    let file = read(dir);
    let place = file.place?;
    let state: State = dir.read_state();
    let now = state.recent(&place, unix)?;
    Some(WeatherView {
        place: place.label(),
        condition: WeatherCondition::of(now.code)?,
        temperature: file.units.of(now.temperature_c),
        units: file.units,
        day: now.day,
    })
}

/// Asks for the place's current weather and keeps it; a failure keeps the last weather (weather FR-012, FR-013).
fn refresh(dir: &ExtDir, place: &Place, unix: i64) {
    let answer = dir.fetch(
        FORECAST,
        "/v1/forecast",
        &[
            ("latitude", format!("{:.2}", place.latitude)),
            ("longitude", format!("{:.2}", place.longitude)),
            ("current", "temperature_2m,weather_code,is_day".into()),
        ],
    );
    let current = answer.ok().and_then(|v| {
        let c = &v["current"];
        Some(Now {
            code: u32::try_from(c["weather_code"].as_u64()?).ok()?,
            temperature_c: c["temperature_2m"].as_f64()?,
            day: c["is_day"].as_u64()? == 1,
            observed: c["time"].as_str()?.chars().take(16).collect(),
            fetched: unix,
        })
    });
    let _guard = dir.lock();
    let mut state: State = dir.read_state();
    state.last_try = Some(unix);
    match current {
        Some(now) => {
            state.place = Some(place.key());
            state.now = Some(now);
            state.failures = 0;
        }
        None => state.failures += 1,
    }
    let _ = dir.write_state(&state);
}

/// Whether a refresh is due at `unix` (weather R4).
fn due(state: &State, place: &Place, unix: i64, woke: bool) -> bool {
    let wait = match state.failures {
        0 => FLOOR,
        n => (FLOOR << (n - 1).min(4)).min(LONGEST_WAIT),
    };
    if state.last_try.is_some_and(|t| unix - t < wait) {
        return false;
    }
    let ours = state
        .now
        .as_ref()
        .filter(|_| state.place.as_deref() == Some(&place.key()));
    match ours {
        None => true,
        Some(now) => woke || state.failures > 0 || unix - now.fetched >= EVERY,
    }
}

/// weather FR-005: up to five places for a typed name, each with its region and country, or none when the service
/// couldn't be reached. Sends only the name.
pub fn search(dir: &ExtDir, name: &str) -> Result<Option<Vec<Place>>> {
    let name = name.trim();
    if !(1..=100).contains(&name.chars().count()) || name.contains(['\n', '\r']) {
        return Err(AppError::invalid("name"));
    }
    let answer = match dir.fetch(
        LOOKUP,
        "/v1/search",
        &[
            ("name", name.into()),
            ("count", "5".into()),
            ("format", "json".into()),
        ],
    ) {
        Ok(answer) => answer,
        Err(FetchError::Refused) => return Err(AppError::invalid("weather")),
        Err(FetchError::Failed) => return Ok(None),
    };
    let text = |v: &Value| v.as_str().filter(|s| !s.is_empty()).map(String::from);
    Ok(Some(
        answer["results"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .filter_map(|r| {
                Some(Place {
                    name: text(&r["name"])?,
                    region: text(&r["admin1"]),
                    country: text(&r["country"]),
                    latitude: (r["latitude"].as_f64()? * 100.0).round() / 100.0,
                    longitude: (r["longitude"].as_f64()? * 100.0).round() / 100.0,
                })
            })
            .filter(Place::valid)
            .take(5)
            .collect(),
    ))
}

/// weather FR-005, FR-010: saves the place and fetches its weather at once.
pub fn choose(dir: &ExtDir, place: &Place, now: OffsetDateTime) -> Result<WeatherSettings> {
    if !place.valid() {
        return Err(AppError::invalid("place"));
    }
    {
        let _guard = dir.lock();
        write(dir, Some(place), read(dir).units)?;
        let mut state: State = dir.read_state();
        // A new place: the old weather goes at once, and the 10-minute floor doesn't hold back the first fetch.
        if state.place.as_deref() != Some(&place.key()) {
            state.now = None;
            state.last_try = None;
            state.failures = 0;
        }
        dir.write_state(&state)?;
    }
    let unix = now.unix_timestamp();
    let state: State = dir.read_state();
    if due(&state, place, unix, false) {
        refresh(dir, place, unix);
    }
    Ok(settings(dir, now))
}

/// weather FR-007: no request; the weather is fetched in Celsius and shown in either.
pub fn set_units(dir: &ExtDir, units: Units, now: OffsetDateTime) -> Result<WeatherSettings> {
    {
        let _guard = dir.lock();
        write(dir, read(dir).place.as_ref(), units)?;
    }
    Ok(settings(dir, now))
}

/// The weather to draw, with no request.
pub fn current(dir: &ExtDir, now: OffsetDateTime) -> Option<WeatherView> {
    view(dir, now.unix_timestamp())
}

/// weather FR-006: what the Weather panel shows. `now` is local time, for the update time.
pub fn settings(dir: &ExtDir, now: OffsetDateTime) -> WeatherSettings {
    let file = read(dir);
    let state: State = dir.read_state();
    let updated = file
        .place
        .as_ref()
        .and_then(|p| state.recent(p, now.unix_timestamp()))
        .and_then(|n| OffsetDateTime::from_unix_timestamp(n.fetched).ok())
        .map(|at| {
            let at = at.to_offset(now.offset());
            format!("{:02}:{:02}", at.hour(), at.minute())
        });
    WeatherSettings {
        unreadable: file.unreadable && dir.place().exists(),
        place: file.place,
        units: file.units,
        updated,
        failing: state.failures > 0,
    }
}

/// The weather extension. Remembers what the windows last drew, to say when it changed.
#[derive(Default)]
pub struct Weather {
    shown: Mutex<Option<Option<WeatherView>>>,
}

impl Extension for Weather {
    fn id(&self) -> &'static str {
        "weather"
    }

    fn name(&self) -> &'static str {
        "Weather"
    }

    fn about(&self) -> &'static str {
        "The weather where you are, drawn around your companion and known in conversation. Off until you turn it \
         on; then it asks Open-Meteo for the weather of the city you choose, sending nothing but that city."
    }

    fn skill(&self) -> &'static str {
        include_str!("SKILL.md")
    }

    fn place(&self) -> &'static str {
        "weather.md"
    }

    fn tools(&self) -> Vec<ToolSpec> {
        Vec::new()
    }

    fn run(&self, tool: &str, _: &Value, _: &ExtDir, _: PrimitiveDateTime) -> ToolAnswer {
        ToolAnswer::error(format!("There is no tool named {tool}."))
    }

    fn default_on(&self) -> bool {
        false
    }

    fn hosts(&self) -> &'static [&'static str] {
        &[LOOKUP, FORECAST]
    }

    fn tick(&self, now: OffsetDateTime, dir: &ExtDir) -> Tick {
        let unix = now.unix_timestamp();
        let (place, state) = {
            let _guard = dir.lock();
            let mut state: State = dir.read_state();
            let woke = state.last_tick.is_some_and(|t| unix - t > WOKE);
            state.last_tick = Some(unix);
            let _ = dir.write_state(&state);
            (read(dir).place, (state, woke))
        };
        let (state, woke) = state;
        if let Some(place) = place.filter(|p| due(&state, p, unix, woke)) {
            refresh(dir, &place, unix);
        }
        let shown = view(dir, unix);
        let mut last = self.shown.lock().unwrap_or_else(|e| e.into_inner());
        let changed = last.as_ref() != Some(&shown);
        *last = Some(shown);
        Tick {
            changed,
            ..Tick::default()
        }
    }

    /// Data-model.md section 5: "Weather in Jakarta, Indonesia: rain, 26 °C, daytime."
    fn context_line(&self, dir: &ExtDir, now: OffsetDateTime) -> Option<String> {
        let v = view(dir, now.unix_timestamp())?;
        Some(format!(
            "Weather in {}: {}, {} {}, {}.",
            v.place,
            v.condition.words(),
            v.temperature,
            v.units.symbol(),
            if v.day { "daytime" } else { "night" }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pec_core::ext::Fetch;
    use std::collections::VecDeque;
    use std::sync::Arc;
    use time::macros::datetime;

    /// What was asked, and the answers to give in turn: recorded JSON or a failure.
    #[derive(Default)]
    struct Calls {
        answers: VecDeque<std::result::Result<Value, FetchError>>,
        asked: Vec<String>,
    }

    struct Fake(Arc<Mutex<Calls>>);

    impl Fetch for Fake {
        fn get(
            &self,
            url: &str,
            query: &[(&str, String)],
        ) -> std::result::Result<Value, FetchError> {
            let mut calls = self.0.lock().unwrap();
            let query: Vec<String> = query.iter().map(|(k, v)| format!("{k}={v}")).collect();
            calls.asked.push(format!("{url}?{}", query.join("&")));
            calls.answers.pop_front().unwrap_or(Err(FetchError::Failed))
        }
    }

    fn recorded(name: &str) -> Value {
        let path = format!(
            "{}/src/weather/fixtures/{name}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// A core with Weather on and the fake's calls.
    fn weather_on() -> (tempfile::TempDir, pec_core::Core, Arc<Mutex<Calls>>) {
        let calls = Arc::new(Mutex::new(Calls::default()));
        let (root, core) = crate::testing::core_fetching(Box::new(Fake(Arc::clone(&calls))));
        assert!(
            !core.extension_on("weather"),
            "weather FR-001: off until turned on"
        );
        core.set_extension("weather", true).unwrap();
        (root, core, calls)
    }

    fn answer(calls: &Arc<Mutex<Calls>>, answer: std::result::Result<Value, FetchError>) {
        calls.lock().unwrap().answers.push_back(answer);
    }

    fn asked(calls: &Arc<Mutex<Calls>>) -> Vec<String> {
        std::mem::take(&mut calls.lock().unwrap().asked)
    }

    fn jakarta() -> Place {
        Place {
            name: "Jakarta".into(),
            region: Some("Jakarta".into()),
            country: Some("Indonesia".into()),
            latitude: -6.21,
            longitude: 106.85,
        }
    }

    /// Jakarta time, so the update time reads as the person would see it.
    const T0: OffsetDateTime = datetime!(2026-11-15 14:00 +7);

    #[test]
    fn the_place_file_reads_as_written_and_keeps_the_persons_lines() {
        let (_root, core, _) = weather_on();
        let dir = core.ext_dir("weather").unwrap();
        std::fs::create_dir_all(dir.place().parent().unwrap()).unwrap();
        std::fs::write(
            dir.place(),
            "# Weather\n\nMy home town.\n- LONGITUDE: 106.85\n- place: Jakarta, Indonesia\n- Latitude: -6.21\n",
        )
        .unwrap();
        let file = read(&dir);
        assert_eq!(
            file.place.as_ref().map(Place::label).as_deref(),
            Some("Jakarta, Indonesia")
        );
        assert_eq!(file.units, Units::Celsius);
        set_units(&dir, Units::Fahrenheit, T0).unwrap();
        let text = std::fs::read_to_string(dir.place()).unwrap();
        assert!(
            text.contains("My home town.\n") && text.ends_with("- Units: fahrenheit\n"),
            "{text}"
        );
        assert_eq!(read(&dir).units, Units::Fahrenheit);

        for broken in [
            "- Place: Jakarta\n- Latitude: -6.21\n",
            "- Place: Jakarta\n- Latitude: 200\n- Longitude: 106.85\n",
            "- Place: \n- Latitude: -6.21\n- Longitude: 106.85\n",
            "- Place: Jakarta\n- Latitude: south\n- Longitude: 106.85\n",
        ] {
            std::fs::write(dir.place(), broken).unwrap();
            assert!(read(&dir).place.is_none(), "{broken}");
            assert!(settings(&dir, T0).unreadable, "{broken}");
        }
    }

    // weather FR-005, FR-008.
    #[test]
    fn a_search_sends_only_the_name_and_offers_up_to_five_places() {
        let (_root, core, calls) = weather_on();
        let dir = core.ext_dir("weather").unwrap();
        answer(&calls, Ok(recorded("search-jakarta")));
        let found = search(&dir, "  Jakarta ").unwrap().unwrap();
        assert_eq!(
            asked(&calls),
            ["https://geocoding-api.open-meteo.com/v1/search?name=Jakarta&count=5&format=json"]
        );
        assert_eq!(found.len(), 3);
        assert_eq!(found[0], jakarta());
        assert_eq!(found[1].label(), "Jakarta, Indonesia");
        answer(&calls, Ok(recorded("search-none")));
        assert_eq!(search(&dir, "Nowhereville").unwrap(), Some(Vec::new()));
        answer(&calls, Err(FetchError::Failed));
        assert_eq!(search(&dir, "Jakarta").unwrap(), None, "unreachable");
        let long = "x".repeat(101);
        for bad in ["", "   ", long.as_str(), "a\nb"] {
            assert_eq!(search(&dir, bad), Err(AppError::invalid("name")));
        }
        assert_eq!(asked(&calls).len(), 2, "the empty and the failed search");
        core.set_extension("weather", false).unwrap();
        let off = core.ext_dir("weather").unwrap();
        assert_eq!(search(&off, "Jakarta"), Err(AppError::invalid("weather")));
        assert!(asked(&calls).is_empty(), "off: nothing goes out");
    }

    // weather FR-005, FR-007, FR-010, FR-011, FR-018.
    #[test]
    fn choosing_a_place_fetches_its_weather_at_once() {
        let (_root, core, calls) = weather_on();
        let dir = core.ext_dir("weather").unwrap();
        answer(&calls, Ok(recorded("current-rain")));
        let shown = choose(&dir, &jakarta(), T0).unwrap();
        assert_eq!(
            asked(&calls),
            [
                "https://api.open-meteo.com/v1/forecast?latitude=-6.21&longitude=106.85&current=temperature_2m,weather_code,is_day"
            ]
        );
        assert_eq!(
            (
                shown.updated.as_deref(),
                shown.failing,
                shown.place.map(|p| p.label())
            ),
            (
                Some("14:00"),
                false,
                Some("Jakarta, Jakarta, Indonesia".into())
            )
        );
        let rain = WeatherView {
            place: "Jakarta, Jakarta, Indonesia".into(),
            condition: WeatherCondition::Rain,
            temperature: 26,
            units: Units::Celsius,
            day: true,
        };
        assert_eq!(current(&dir, T0), Some(rain));
        let weather = Weather::default();
        assert_eq!(
            weather.context_line(&dir, T0).as_deref(),
            Some("Weather in Jakarta, Jakarta, Indonesia: rain, 26 °C, daytime.")
        );
        // Changing units makes no request.
        set_units(&dir, Units::Fahrenheit, T0).unwrap();
        assert_eq!(current(&dir, T0).map(|v| v.temperature), Some(79));
        assert!(weather.context_line(&dir, T0).unwrap().contains("79 °F"));
        assert!(asked(&calls).is_empty());
        assert!(
            choose(
                &dir,
                &Place {
                    latitude: 91.0,
                    ..jakarta()
                },
                T0
            )
            .is_err()
        );
    }

    /// Ticks Weather `minutes` after T0: the requests made, and whether the windows must redraw.
    fn tick_at(
        weather: &Weather,
        core: &pec_core::Core,
        calls: &Arc<Mutex<Calls>>,
        minutes: i64,
    ) -> (usize, bool) {
        let dir = core.ext_dir("weather").unwrap();
        let tick = weather.tick(T0 + time::Duration::minutes(minutes), &dir);
        (asked(calls).len(), tick.changed)
    }

    // weather R4, weather FR-010, FR-012, FR-013.
    #[test]
    fn the_weather_refreshes_every_half_hour_waits_after_failures_and_goes_when_old() {
        let (_root, core, calls) = weather_on();
        let weather = Weather::default();
        let dir = core.ext_dir("weather").unwrap();
        // No place: no request.
        assert_eq!(tick_at(&weather, &core, &calls, 0), (0, true));
        write(&dir, Some(&jakarta()), Units::Celsius).unwrap();
        answer(&calls, Ok(recorded("current-rain")));
        // Ticks every minute from here, so none of them looks like waking.
        assert_eq!(
            tick_at(&weather, &core, &calls, 1),
            (1, true),
            "first weather"
        );
        for minute in 2..31 {
            assert_eq!(
                tick_at(&weather, &core, &calls, minute),
                (0, false),
                "{minute}"
            );
        }
        answer(&calls, Ok(recorded("current-rain")));
        assert_eq!(
            tick_at(&weather, &core, &calls, 31),
            (1, false),
            "half an hour on"
        );
        // From here every request fails: the last weather stays, quietly, with waits of 10, 20 and 40 minutes.
        let mut tries = Vec::new();
        for minute in 32..=210 {
            if tick_at(&weather, &core, &calls, minute).0 == 1 {
                tries.push(minute);
            }
            if minute == 120 {
                assert!(
                    current(&dir, T0 + time::Duration::minutes(minute)).is_some(),
                    "kept while recent"
                );
                assert!(settings(&dir, T0 + time::Duration::minutes(minute)).failing);
            }
        }
        assert_eq!(tries[..4], [61, 71, 91, 131], "{tries:?}");
        // Three hours after the last success the weather goes, and the windows are told once.
        let gone = 31 + 180;
        assert!(tick_at(&weather, &core, &calls, gone).1);
        assert_eq!(current(&dir, T0 + time::Duration::minutes(gone)), None);
        assert!(!tick_at(&weather, &core, &calls, gone + 1).1);
    }

    // weather R4: waking from sleep refreshes soon, but never within ten minutes of the last try.
    #[test]
    fn waking_refreshes_the_weather() {
        let (_root, core, calls) = weather_on();
        let weather = Weather::default();
        write(
            &core.ext_dir("weather").unwrap(),
            Some(&jakarta()),
            Units::Celsius,
        )
        .unwrap();
        answer(&calls, Ok(recorded("current-rain")));
        assert_eq!(tick_at(&weather, &core, &calls, 0).0, 1);
        assert_eq!(tick_at(&weather, &core, &calls, 1).0, 0);
        // Asleep from minute 1 to minute 5: too soon after the last try.
        assert_eq!(tick_at(&weather, &core, &calls, 5).0, 0);
        // Asleep again until minute 20: the first tick after waking refreshes.
        answer(&calls, Ok(recorded("current-rain")));
        assert_eq!(tick_at(&weather, &core, &calls, 20).0, 1);
    }

    // weather FR-004, FR-009: off, nothing is asked.
    #[test]
    fn turned_off_it_asks_nothing() {
        let (_root, core, calls) = weather_on();
        write(
            &core.ext_dir("weather").unwrap(),
            Some(&jakarta()),
            Units::Celsius,
        )
        .unwrap();
        core.set_extension("weather", false).unwrap();
        core.ext_tick(T0);
        assert!(asked(&calls).is_empty());
    }
}
