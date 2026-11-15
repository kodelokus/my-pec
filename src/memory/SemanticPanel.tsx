import { useCallback, useEffect, useState } from "react";
import {
  asAppError,
  cancelSemanticDownload,
  dismissSemanticOffer,
  removeSemanticModel,
  semanticStatus,
  semanticTick,
  setSemanticSearch,
  startSemanticDownload,
} from "../api/commands";
import type { SemanticStatus } from "../bindings/SemanticStatus";
import { Confirm } from "../Confirm";
import { errorMessage, t } from "../strings";

/** How often the progress is read while the model downloads or memories are prepared (search R12). */
const POLL_MS = 1000;

/** Search by meaning. In settings, the whole section; in the memory browser, only the offer and its
 * progress. Never a notification, badge or reminder (search FR-009). */
export function SemanticPanel(props: { variant: "settings" | "offer" }) {
  const [status, setStatus] = useState<SemanticStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [removing, setRemoving] = useState(false);
  const fail = useCallback((e: unknown) => setError(errorMessage(asAppError(e))), []);

  useEffect(() => {
    semanticStatus().then(setStatus, fail);
  }, [fail]);

  const busy = status?.state.kind === "Downloading" || status?.state.kind === "Preparing";
  useEffect(() => {
    if (!busy) return;
    const poll = setInterval(() => semanticStatus().then(setStatus, () => {}), POLL_MS);
    return () => clearInterval(poll);
  }, [busy]);

  if (!status) return null;
  const download = () => {
    setError(null);
    startSemanticDownload().then((s) => {
      setStatus(s);
      // Prepare the memories right away instead of waiting for the next minute (search R8).
      semanticTick().then(setStatus, () => {});
    }, fail);
    semanticStatus().then(setStatus, () => {});
  };
  const state = status.state;
  const progress =
    state.kind === "Downloading" ? (
      <>
        <progress value={state.done} max={state.total} />
        <p>{t.semantic.downloading(t.semantic.size(state.done), t.semantic.size(state.total))}</p>
        <button type="button" onClick={() => cancelSemanticDownload().then(setStatus, fail)}>
          {t.semantic.cancel}
        </button>
      </>
    ) : state.kind === "Preparing" ? (
      <>
        <progress value={state.done} max={state.total} />
        <p>{t.semantic.preparing(state.done, state.total)}</p>
      </>
    ) : null;
  const offer = (
    <div className="semantic-offer">
      <p>{t.semantic.offer(t.semantic.size(status.downloadBytes))}</p>
      <div className="row">
        <button type="button" className="primary" onClick={download}>
          {t.semantic.download}
        </button>
        {props.variant === "offer" && (
          <button type="button" onClick={() => dismissSemanticOffer().then(setStatus, fail)}>
            {t.semantic.notNow}
          </button>
        )}
      </div>
    </div>
  );
  const alert = error && (
    <div role="alert" className="alert error">
      <p>{error}</p>
    </div>
  );

  if (props.variant === "offer") {
    if (!status.offer && state.kind !== "Downloading") return null;
    return (
      <aside className="card semantic" aria-label={t.semantic.title}>
        {state.kind === "Downloading" ? progress : offer}
        {alert}
      </aside>
    );
  }

  const inPlace = status.diskBytes > 0 && state.kind !== "Downloading";
  return (
    <section aria-labelledby="semantic-section" className="card semantic">
      <h2 id="semantic-section">{t.semantic.title}</h2>
      <p className="hint">{t.semantic.hint}</p>
      {inPlace && (
        <label className="check">
          <input
            type="checkbox"
            checked={state.kind !== "Off"}
            onChange={(e) => setSemanticSearch(e.target.checked).then(setStatus, fail)}
          />
          {t.semantic.switchLabel}
        </label>
      )}
      <div role="status">
        {state.kind === "NotDownloaded" && offer}
        {progress}
        {state.kind === "Ready" && <p>{t.semantic.ready(t.semantic.size(status.diskBytes))}</p>}
        {state.kind === "Off" && <p>{t.semantic.off}</p>}
        {state.kind === "NeedsAttention" && (
          <>
            <p>{t.semantic.problems[state.reason]}</p>
            <button type="button" onClick={download}>
              {t.semantic.tryAgain}
            </button>
          </>
        )}
      </div>
      {inPlace && (
        <button type="button" className="subtle" onClick={() => setRemoving(true)}>
          {t.semantic.remove}
        </button>
      )}
      {alert}
      {removing && (
        <Confirm
          message={t.semantic.removeConfirm}
          confirm={t.semantic.remove}
          danger
          onCancel={() => setRemoving(false)}
          onConfirm={() => {
            setRemoving(false);
            removeSemanticModel().then(setStatus, fail);
          }}
        />
      )}
    </section>
  );
}
