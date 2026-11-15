import { useEffect, useState } from "react";
import { asAppError, weatherChoose, weatherSearch, weatherSettings, weatherUnits } from "../../api/commands";
import type { AppError } from "../../bindings/AppError";
import type { Place } from "../../bindings/Place";
import type { Units } from "../../bindings/Units";
import type { WeatherSettings } from "../../bindings/WeatherSettings";
import { errorMessage, t } from "../../strings";

const label = (p: Place) => [p.name, p.region, p.country].filter(Boolean).join(", ");

/**
 * weather FR-005 to FR-007: find and choose the city, pick the units, and see plainly what is sent, when it
 * was last updated and the data credit. A failed update is a quiet line here, never an alert elsewhere (weather FR-013).
 */
export function WeatherPanel(props: { folder: string }) {
  const [shown, setShown] = useState<WeatherSettings | null>(null);
  const [name, setName] = useState("");
  const [found, setFound] = useState<Place[] | null | undefined>(undefined);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<AppError | null>(null);
  const fail = (e: unknown) => setError(asAppError(e));

  useEffect(() => {
    weatherSettings().then(setShown, (e) => setError(asAppError(e)));
  }, []);

  const search = async () => {
    setBusy(true);
    setError(null);
    try {
      setFound(await weatherSearch(name));
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const choose = async (place: Place) => {
    setBusy(true);
    try {
      setShown(await weatherChoose(place));
      setFound(undefined);
      setName("");
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const units = (next: Units) => weatherUnits(next).then(setShown, fail);

  return (
    <div className="weather-panel">
      <p className="hint">
        {shown?.place ? t.weather.chosen(label(shown.place)) : t.weather.noneChosen}
        {shown?.updated && ` ${t.weather.updated(shown.updated)}`}
      </p>
      {shown?.failing && <p className="hint">{t.weather.failing}</p>}
      {shown?.unreadable && <p className="hint">{t.weather.unreadable}</p>}
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          void search();
        }}
      >
        <label htmlFor="weather-city">{t.weather.cityLabel}</label>
        <input id="weather-city" value={name} onChange={(e) => setName(e.target.value)} maxLength={100} />
        <button type="submit" disabled={busy || name.trim() === ""}>
          {busy ? t.weather.searching : t.weather.search}
        </button>
      </form>
      {found === null && <p role="status">{t.weather.unreachable}</p>}
      {found?.length === 0 && <p role="status">{t.weather.noPlace}</p>}
      {found && found.length > 0 && (
        <ul className="weather-places" aria-label={t.weather.results}>
          {found.map((place) => (
            <li key={`${place.latitude},${place.longitude},${place.name}`}>
              <button type="button" className="link" disabled={busy} onClick={() => void choose(place)}>
                {label(place)}
              </button>
            </li>
          ))}
        </ul>
      )}
      {shown && (
        <fieldset className="row">
          <legend>{t.weather.units}</legend>
          {(["celsius", "fahrenheit"] as const).map((u) => (
            <label key={u} className="check">
              <input
                type="radio"
                name="weather-units"
                checked={shown.units === u}
                onChange={() => void units(u)}
              />
              {u === "celsius" ? t.weather.celsius : t.weather.fahrenheit}
            </label>
          ))}
        </fieldset>
      )}
      <p className="hint">
        {t.weather.sent} {t.extensions.files(props.folder)}
      </p>
      <p className="hint">{t.weather.credit}</p>
      {error && (
        <div role="alert" className="alert error">
          <p>{errorMessage(error)}</p>
        </div>
      )}
    </div>
  );
}
