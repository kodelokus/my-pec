import { useCallback, useEffect, useState } from "react";
import { asAppError, dreamJournal, dreamNow, undoLastDream } from "../api/commands";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { DreamJournal as Journal } from "../bindings/DreamJournal";
import type { DreamRun } from "../bindings/DreamRun";
import { Confirm } from "../Confirm";
import { Reply } from "../chat/Message";
import { Line } from "../Line";
import { errorMessage, t } from "../strings";

function RunLine(props: { run: DreamRun }) {
  const { run } = props;
  const notes = [
    !run.completed && t.dreams.unfinished,
    run.fellBack && t.dreams.fellBack,
    run.undone && t.dreams.undone,
    run.tokens > 0 && t.dreams.tokens(run.tokens),
  ].filter(Boolean);
  return (
    <li>
      <strong>{t.memory.day(run.night)}</strong> · {t.dreams.run(run.added, run.merged, run.superseded)}
      {notes.length > 0 && <span className="muted"> · {notes.join(" · ")}</span>}
    </li>
  );
}

/** dreaming FR-020 to FR-022, FR-027, FR-018: the diary, the nights, dream now and undo last dream. */
export function DreamJournal(props: { companion: CompanionSummary | null; onClose: () => void }) {
  const [journal, setJournal] = useState<Journal | null>(null);
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const [changed, setChanged] = useState<string[] | null>(null);
  const name = props.companion?.name ?? "";
  const face = props.companion?.characterId ?? "";

  const load = useCallback(() => {
    dreamJournal().then(setJournal, (e) => setNote(errorMessage(asAppError(e))));
  }, []);
  useEffect(load, [load]);

  const act = (work: () => Promise<unknown>) => {
    setBusy(true);
    setNote(null);
    work()
      .catch((e) => setNote(errorMessage(asAppError(e))))
      .finally(() => {
        setBusy(false);
        load();
      });
  };

  const undo = (force: boolean) =>
    act(async () => {
      const outcome = await undoLastDream(force);
      if (outcome.kind === "Changed") return setChanged(outcome.files);
      setNote(outcome.kind === "Undone" ? t.dreams.undoneNote : t.dreams.nothingToUndo);
    });

  return (
    <>
      <header className="app-bar">
        <button type="button" onClick={props.onClose}>
          {t.back}
        </button>
        <h1>{t.dreams.title(name)}</h1>
      </header>
      <main className="content">
        <div className="column">
          <Line characterId={face} asleep={busy}>
            <p>{busy ? t.dreams.dreaming(name) : t.dreams.line(name)}</p>
          </Line>
          <div className="row">
            <button type="button" className="primary" disabled={busy} onClick={() => act(dreamNow)}>
              {t.dreams.dreamNow}
            </button>
            <button type="button" disabled={busy || !journal?.canUndo} onClick={() => undo(false)}>
              {t.dreams.undo}
            </button>
          </div>
          {note && (
            <p role="status" className="hint">
              {note}
            </p>
          )}
          <section aria-labelledby="dream-diary" className="card">
            <h2 id="dream-diary">{t.dreams.diaryLabel}</h2>
            {journal?.diary.trim() ? (
              <div className="dream-diary">
                <Reply text={journal.diary} />
              </div>
            ) : (
              <p className="muted">{t.dreams.empty}</p>
            )}
          </section>
          {journal && journal.runs.length > 0 && (
            <section aria-labelledby="dream-runs" className="card">
              <h2 id="dream-runs">{t.dreams.runsLabel}</h2>
              <ul className="dream-runs">
                {journal.runs.map((run) => (
                  <RunLine key={run.id} run={run} />
                ))}
              </ul>
            </section>
          )}
        </div>
      </main>
      {changed && (
        <Confirm
          message={t.dreams.changedConfirm(changed)}
          confirm={t.dreams.undoAnyway}
          danger
          onCancel={() => setChanged(null)}
          onConfirm={() => {
            setChanged(null);
            undo(true);
          }}
        />
      )}
    </>
  );
}
