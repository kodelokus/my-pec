import { useState } from "react";
import { asAppError, memorySave } from "../api/commands";
import type { MemoryDoc } from "../bindings/MemoryDoc";
import { errorMessage, t } from "../strings";

/** memory FR-025, FR-029: a memory file as plain text, saved exactly as typed, never over a change made meanwhile. */
export function MemoryEditor(props: {
  doc: MemoryDoc;
  label: string;
  budget: number | null;
  onSaved: () => void;
  onCancel: () => void;
}) {
  const [text, setText] = useState(props.doc.text);
  const [base, setBase] = useState(props.doc.hash);
  const [conflict, setConflict] = useState<{ text: string; hash: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const used = [...text].length;
  const over = props.budget !== null && used > props.budget;

  const save = (baseHash: string) =>
    memorySave(props.doc.path, text, baseHash).then(
      (outcome) => {
        if (outcome.kind === "Saved") return props.onSaved();
        setConflict({ text: outcome.text, hash: outcome.hash });
      },
      (e) => setError(errorMessage(asAppError(e))),
    );

  return (
    <div className="stack">
      <label>
        {props.label}
        <textarea
          rows={18}
          value={text}
          aria-describedby={props.budget !== null ? "memory-budget" : undefined}
          onChange={(e) => setText(e.target.value)}
        />
      </label>
      {props.budget !== null && (
        <>
          <progress
            className={over ? "meter over" : "meter"}
            max={props.budget}
            value={Math.min(used, props.budget)}
          />
          <p id="memory-budget" role="status" className={over ? "count over" : "count"}>
            {t.budget(used, props.budget)}
          </p>
        </>
      )}
      {conflict && (
        <div role="alert" className="alert warning">
          <p>{t.memory.conflict}</p>
          <div className="row">
            <button
              type="button"
              onClick={() => {
                setText(conflict.text);
                setBase(conflict.hash);
                setConflict(null);
              }}
            >
              {t.memory.reload}
            </button>
            <button
              type="button"
              onClick={() => {
                setBase(conflict.hash);
                setConflict(null);
                void save(conflict.hash);
              }}
            >
              {t.memory.keepMine}
            </button>
          </div>
        </div>
      )}
      {error && (
        <div role="alert" className="alert error">
          <p>{error}</p>
        </div>
      )}
      <div className="row">
        <button type="button" onClick={props.onCancel}>
          {t.cancel}
        </button>
        <button
          type="button"
          className="primary"
          disabled={conflict !== null}
          onClick={() => void save(base)}
        >
          {t.memory.save}
        </button>
      </div>
    </div>
  );
}
