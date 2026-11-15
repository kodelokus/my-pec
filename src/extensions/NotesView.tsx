import { useCallback, useEffect, useState } from "react";
import { asAppError, deleteNote, newNote, notes, onExtChanged, showNote } from "../api/commands";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { NoteSummary } from "../bindings/NoteSummary";
import { Confirm } from "../Confirm";
import { Line } from "../Line";
import { errorMessage, t } from "../strings";

/** extensions FR-015: every note, on the desktop or put away, newest first. */
export function NotesView(props: { companion: CompanionSummary | null; onClose: () => void }) {
  const [list, setList] = useState<NoteSummary[] | null>(null);
  const [deleting, setDeleting] = useState<NoteSummary | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const fail = useCallback((e: unknown) => setNote(errorMessage(asAppError(e))), []);

  const load = useCallback(() => {
    notes().then(setList, fail);
  }, [fail]);
  useEffect(() => {
    load();
    const off = onExtChanged((ids) => ids.includes("notes") && load());
    return () => {
      off.then((unlisten) => unlisten());
    };
  }, [load]);

  const act = (work: Promise<unknown>) => work.then(load, fail);

  return (
    <>
      <header className="app-bar">
        <button type="button" onClick={props.onClose}>
          {t.back}
        </button>
        <h1>{t.notes.title}</h1>
      </header>
      <main className="content">
        <div className="column">
          <Line characterId={props.companion?.characterId ?? ""}>
            <p>{t.notes.line(props.companion?.name ?? "")}</p>
          </Line>
          {note && (
            <div role="alert" className="alert error">
              <p>{note}</p>
            </div>
          )}
          <div className="row">
            <button type="button" className="primary" onClick={() => void act(newNote())}>
              {t.notes.newNote}
            </button>
          </div>
          {list && list.length === 0 && <p className="muted meta">{t.notes.empty}</p>}
          <ul className="ext-list" aria-label={t.notes.title}>
            {list?.map((n) => (
              <li key={n.file} className={`card note-${n.colour}`}>
                <div>
                  <strong>{n.firstLine || t.notes.untitled}</strong>
                  <p className="muted meta">{n.shown ? t.notes.shown : t.notes.hidden}</p>
                </div>
                <div className="row">
                  <button type="button" onClick={() => void act(showNote(n.file, !n.shown))}>
                    {n.shown ? t.notes.hide : t.notes.show}
                  </button>
                  <button type="button" onClick={() => setDeleting(n)}>
                    {t.notes.delete}
                  </button>
                </div>
              </li>
            ))}
          </ul>
        </div>
      </main>
      {deleting && (
        <Confirm
          message={t.notes.deleteConfirm(deleting.firstLine || t.notes.untitled)}
          confirm={t.notes.delete}
          danger
          onCancel={() => setDeleting(null)}
          onConfirm={() => {
            setDeleting(null);
            void act(deleteNote(deleting.file));
          }}
        />
      )}
    </>
  );
}
