import { useEffect, useState } from "react";
import { asAppError, completeSetup, saveSetupDraft, setupOptions } from "../api/commands";
import type { AppError } from "../bindings/AppError";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { SetupCompletion } from "../bindings/SetupCompletion";
import type { SetupDraft } from "../bindings/SetupDraft";
import type { SetupOptions } from "../bindings/SetupOptions";
import type { SetupStep } from "../bindings/SetupStep";
import { Confirm } from "../Confirm";
import { Line } from "../Line";
import { errorMessage, steps, t } from "../strings";
import { AboutYouStep } from "./AboutYouStep";
import { AvatarStep } from "./AvatarStep";
import { KeyStep } from "./KeyStep";
import { SoulStep } from "./SoulStep";

const ORDER: SetupStep[] = ["Key", "Soul", "Avatar", "AboutYou"];

/** A line the wizard says in place of the step's own: progress, a problem, or the existing-companion question. */
export type Notice = { text: string; alert: boolean } | null;

type Props = {
  draft: SetupDraft;
  lastFour: string | null;
  /** The companion found in an existing workspace (onboarding FR-008). */
  existing: CompanionSummary | null;
  onDone: () => void;
};

/** onboarding FR-004 to FR-009: four steps, saved as they change, with only the key required. Each opens with the companion's line (PER-1, PER-2). */
export function Wizard(props: Props) {
  const [draft, setDraft] = useState(props.draft);
  const [lastFour, setLastFour] = useState(props.lastFour);
  const [options, setOptions] = useState<SetupOptions | null>(null);
  const [error, setError] = useState<{ error: AppError; retry: () => void } | null>(null);
  // Keeping an existing companion finishes at once, so a resumed draft past the key step chose to start fresh.
  const [mode, setMode] = useState<SetupCompletion | null>(
    props.existing === null ? "New" : props.draft.step === "Key" ? null : "ReplaceExisting",
  );
  const [confirmFresh, setConfirmFresh] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const load = () =>
      setupOptions()
        .then(setOptions)
        .catch((e) => setError({ error: asAppError(e), retry: load }));
    load();
  }, []);

  function update(next: SetupDraft) {
    setDraft(next);
    saveSetupDraft(next).catch((e) => setError({ error: asAppError(e), retry: () => update(next) }));
  }

  async function finish(last: SetupDraft, how: SetupCompletion) {
    setBusy(true);
    setError(null);
    try {
      setDraft(last);
      await saveSetupDraft(last);
      await completeSetup(how);
      props.onDone();
    } catch (e) {
      setError({ error: asAppError(e), retry: () => void finish(last, how) });
    } finally {
      setBusy(false);
    }
  }

  const index = ORDER.indexOf(draft.step);
  const go = (step: SetupStep) => update({ ...draft, step });
  const choosing = draft.step === "Key" && lastFour !== null && mode === null && props.existing !== null;
  const name = draft.soul.name.trim() || (options?.defaultName ?? "");
  const notice: Notice = busy
    ? { text: t.line.finishing, alert: false }
    : error
      ? { text: errorMessage(error.error), alert: true }
      : choosing && props.existing
        ? { text: t.line.existing(props.existing.name), alert: false }
        : null;
  const face = draft.characterId;

  return (
    <>
      <main className="content wash">
        <div className={draft.step === "Soul" ? "column wide" : "column"}>
          <div className="dots">
            <ol aria-label={t.stepsLabel}>
              {ORDER.map((step, i) => (
                <li
                  key={step}
                  className={i < index ? "done" : undefined}
                  aria-current={step === draft.step ? "step" : undefined}
                >
                  <span className="visually-hidden">{steps[step]}</span>
                </li>
              ))}
            </ol>
            <span className="caption">{t.stepCaption(index + 1, ORDER.length, steps[draft.step])}</span>
          </div>

          <fieldset disabled={busy} className="stack">
            {draft.step === "Key" && (
              <KeyStep lastFour={lastFour} face={face} notice={notice} onAccepted={setLastFour} />
            )}
            {options && draft.step === "Soul" && (
              <SoulStep
                soul={draft.soul}
                options={options}
                face={face}
                notice={notice}
                onChange={(soul) => update({ ...draft, soul })}
              />
            )}
            {options && draft.step === "Avatar" && (
              <AvatarStep
                characterId={draft.characterId}
                characters={options.characters}
                notice={notice}
                onChange={(characterId) => update({ ...draft, characterId })}
              />
            )}
            {draft.step === "AboutYou" && (
              <AboutYouStep
                about={draft.about}
                name={name}
                face={face}
                notice={notice}
                onChange={(about) => update({ ...draft, about })}
              />
            )}
            {/* Without the options, a failure to load them is still said. */}
            {!options && (draft.step === "Soul" || draft.step === "Avatar") && notice && (
              <Line characterId={face} role="alert">
                {notice.text}
              </Line>
            )}

            {choosing && (
              <div className="row under">
                <button type="button" className="primary" onClick={() => void finish(draft, "AdoptExisting")}>
                  {t.keepCompanion}
                </button>
                <button type="button" onClick={() => setConfirmFresh(true)}>
                  {t.startFresh}
                </button>
              </div>
            )}
            {confirmFresh && (
              <Confirm
                message={t.startFreshConfirm}
                confirm={t.startFresh}
                onCancel={() => setConfirmFresh(false)}
                onConfirm={() => {
                  setConfirmFresh(false);
                  setMode("ReplaceExisting");
                  go("Soul");
                }}
              />
            )}

            {busy && (
              <p className="row under" aria-hidden="true">
                <span className="spinner" />
              </p>
            )}
            {error && (
              <div className="row under">
                <button type="button" onClick={error.retry}>
                  {t.retry}
                </button>
              </div>
            )}
          </fieldset>
        </div>
      </main>

      <nav className="footer">
        {index > 0 && (
          <button type="button" onClick={() => go(ORDER[index - 1])} disabled={busy}>
            {t.back}
          </button>
        )}
        <div className="end">
          {draft.step === "AboutYou" && (
            <button
              type="button"
              className="subtle"
              disabled={busy || mode === null}
              onClick={() => mode && void finish({ ...draft, about: { callMe: null, language: null } }, mode)}
            >
              {t.skip}
            </button>
          )}
          {!choosing && (
            <button
              type="button"
              className="primary"
              disabled={busy || lastFour === null || mode === null}
              onClick={() =>
                index < ORDER.length - 1 ? go(ORDER[index + 1]) : mode && void finish(draft, mode)
              }
            >
              {index < ORDER.length - 1 ? t.next : t.meet(name)}
            </button>
          )}
        </div>
      </nav>
    </>
  );
}
