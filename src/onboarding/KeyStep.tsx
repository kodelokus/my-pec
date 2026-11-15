import { type ReactNode, useId, useRef, useState } from "react";
import { asAppError, submitKey } from "../api/commands";
import type { AppError } from "../bindings/AppError";
import type { KeyCheckOutcome } from "../bindings/KeyCheckOutcome";
import { Line } from "../Line";
import { errorMessage, KEY_ADDRESS, keyOutcomes, t } from "../strings";
import type { Notice } from "./Wizard";

/**
 * Checks a key with the provider. Also used in settings to replace the key (onboarding FR-010 to FR-015, FR-028).
 * In the wizard the companion says each outcome (PER-6); settings keep plain alert boxes (PER-2).
 */
export function KeyStep(props: {
  lastFour: string | null;
  /** The wizard's portrait and its line, when it has one. Absent in settings. */
  face?: string;
  notice?: Notice;
  label?: string;
  /** Settings' Remove key, shown beside this step's own button. */
  actions?: ReactNode;
  onAccepted: (lastFour: string) => void;
}) {
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<KeyCheckOutcome | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [editing, setEditing] = useState(props.lastFour === null);
  const [copied, setCopied] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const messageId = useId();
  const hintId = useId();

  async function check() {
    setBusy(true);
    setError(null);
    try {
      const result = await submitKey(value);
      setOutcome(result);
      if (result.outcome === "Accepted") {
        // Principle II: the webview keeps no copy once the core has the key.
        setValue("");
        setEditing(false);
        props.onAccepted(result.lastFour);
      }
    } catch (e) {
      setError(asAppError(e));
    } finally {
      setBusy(false);
    }
  }

  function edit() {
    input.current?.focus();
    input.current?.select();
  }

  const problem = outcome && outcome.outcome !== "Accepted" ? keyOutcomes[outcome.outcome] : null;
  const message = problem ? problem.message : error ? errorMessage(error) : null;
  // The same key is worth sending again after an error or a problem that isn't the key's fault.
  const resubmit = error !== null || Boolean(problem?.resubmit);
  const wizard = props.face !== undefined;
  const accepted = props.lastFour !== null && !editing;
  // onboarding US6 AC3: a rejected replacement leaves the stored key in place, and says so.
  const keeps = !wizard && props.lastFour !== null && outcome?.outcome === "Rejected";
  const hint = wizard ? t.keyStored : props.lastFour === null ? t.keyAddressHint : null;

  let line = t.line.key;
  if (busy) line = t.line.checking;
  else if (message) line = message;
  else if (accepted) line = t.line.accepted;
  const notice = props.notice;

  return (
    <>
      {wizard && (
        <Line
          characterId={props.face as string}
          id={messageId}
          role={(notice ? notice.alert : message !== null && !busy) ? "alert" : "status"}
          moment={message !== null && !busy ? "concerned" : accepted ? "happy" : undefined}
        >
          {notice?.text ?? line}
        </Line>
      )}
      <div className={wizard ? "stack under" : "stack"}>
        {wizard && !accepted && (
          <div className="copy-field">
            <code>{KEY_ADDRESS}</code>
            <button
              type="button"
              className="subtle"
              onClick={() => navigator.clipboard?.writeText(KEY_ADDRESS).then(() => setCopied(true))}
            >
              {copied ? t.copied : t.copy}
            </button>
          </div>
        )}
        {accepted ? (
          wizard ? (
            <div className="alert success">
              <p role="status">{t.keyAccepted(props.lastFour as string)}</p>
              <button type="button" className="subtle" onClick={() => setEditing(true)}>
                {t.changeKey}
              </button>
            </div>
          ) : (
            <div className="row">
              <button type="button" onClick={() => setEditing(true)}>
                {props.label}
              </button>
              {props.actions}
            </div>
          )
        ) : (
          <form
            className="stack"
            onSubmit={(e) => {
              e.preventDefault();
              void check();
            }}
          >
            <label>
              {wizard || props.lastFour !== null ? (props.label ?? t.keyLabel) : t.keySection}
              <input
                ref={input}
                type="password"
                autoComplete="off"
                spellCheck={false}
                value={value}
                onChange={(e) => setValue(e.target.value)}
                aria-invalid={problem !== null && !problem.resubmit}
                aria-describedby={message ? messageId : hint ? hintId : undefined}
                disabled={busy}
              />
            </label>
            {hint && (
              <p className="hint" id={hintId}>
                {hint}
              </p>
            )}
            <div className="row">
              <button type="submit" className="primary" disabled={busy}>
                {busy
                  ? t.checking
                  : wizard && message && resubmit
                    ? (problem?.action ?? t.retry)
                    : t.checkKey}
              </button>
              {busy && (
                <>
                  <span className="spinner" aria-hidden="true" />
                  <span className="muted">{t.checkingHint}</span>
                </>
              )}
              {wizard && !busy && problem && !problem.resubmit && (
                <button type="button" onClick={edit}>
                  {problem.action}
                </button>
              )}
              {!busy && props.actions}
            </div>
          </form>
        )}
        {!wizard && message && (
          <div role="alert" id={messageId} className={`alert ${resubmit ? "warning" : "error"}`}>
            <p>
              {message}
              {keeps && <> {t.keyKept}</>}
            </p>
            <button type="button" onClick={resubmit ? () => void check() : edit} disabled={busy}>
              {problem ? problem.action : t.retry}
            </button>
          </div>
        )}
      </div>
    </>
  );
}
