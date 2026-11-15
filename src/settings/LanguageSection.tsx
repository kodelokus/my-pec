import { useState } from "react";
import { asAppError, setLanguage } from "../api/commands";
import type { AppError } from "../bindings/AppError";
import { errorMessage, LANGUAGES, language, t } from "../strings";

/** NFR-I18N-1: the interface language. Every window reloads its text once the choice is saved. */
export function LanguageSection() {
  const [error, setError] = useState<AppError | null>(null);
  return (
    <section aria-labelledby="language-section" className="card">
      <h2 id="language-section">{t.languageSection}</h2>
      {/* A wrapping label would add the selected option to the picker's accessible name. */}
      <label htmlFor="language-picker">{t.appLanguageLabel}</label>
      <select
        id="language-picker"
        value={language}
        aria-describedby="language-hint"
        onChange={(e) => setLanguage(e.target.value).catch((e) => setError(asAppError(e)))}
      >
        {LANGUAGES.map((l) => (
          <option key={l.id} value={l.id} lang={l.id}>
            {l.name}
          </option>
        ))}
      </select>
      <p id="language-hint" className="hint">
        {t.languageHint}
      </p>
      {error && (
        <div role="alert" className="alert error">
          <p>{errorMessage(error)}</p>
        </div>
      )}
    </section>
  );
}
