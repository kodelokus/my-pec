import { useEffect, useState } from "react";
import { asAppError, autostart, extensions, setAutostart, setExtension } from "../api/commands";
import type { AppError } from "../bindings/AppError";
import type { Autostart } from "../bindings/Autostart";
import type { ExtensionInfo } from "../bindings/ExtensionInfo";
import { errorMessage, t } from "../strings";
import { WeatherPanel } from "./weather/WeatherPanel";

/** extensions FR-004, FR-026: each extension on or off, what it does and where its files are; launch at login. */
export function ExtensionsSection() {
  const [list, setList] = useState<ExtensionInfo[] | null>(null);
  const [start, setStart] = useState<Autostart | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const fail = (e: unknown) => setError(asAppError(e));

  useEffect(() => {
    const failed = (e: unknown) => setError(asAppError(e));
    extensions().then(setList, failed);
    autostart().then(setStart, failed);
  }, []);

  return (
    <section aria-labelledby="extensions-section" className="card">
      <h2 id="extensions-section">{t.extensions.section}</h2>
      <p className="hint">{t.extensions.hint}</p>
      {list?.map((ext) => (
        <div key={ext.id} className="extension">
          <label className="check">
            <input
              type="checkbox"
              checked={ext.on}
              aria-describedby={`ext-${ext.id}`}
              onChange={(e) => setExtension(ext.id, e.target.checked).then(setList, fail)}
            />
            {ext.name}
          </label>
          <p id={`ext-${ext.id}`} className="hint">
            {ext.about} {t.extensions.files(ext.folder)}
          </p>
          {ext.id === "weather" && ext.on && <WeatherPanel folder={ext.folder} />}
        </div>
      ))}
      {start && (
        <>
          <label className="check">
            <input
              type="checkbox"
              checked={start.on}
              aria-describedby="autostart-hint"
              onChange={(e) => setAutostart(e.target.checked).then(setStart, fail)}
            />
            {t.extensions.autostart}
          </label>
          <p id="autostart-hint" className="hint">
            {t.extensions.autostartHint}
          </p>
        </>
      )}
      {error && (
        <div role="alert" className="alert error">
          <p>{errorMessage(error)}</p>
        </div>
      )}
    </section>
  );
}
