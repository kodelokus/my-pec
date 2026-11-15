import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";
import { asAppError, hideNote, newNote, noteRead, noteSave, noteStyle, onExtChanged } from "../api/commands";
import type { NoteColour } from "../bindings/NoteColour";
import type { NoteDoc } from "../bindings/NoteDoc";
import { errorMessage, t } from "../strings";

/** extensions data 6: saved this long after the last keystroke (extensions FR-014 allows one second). */
export const SAVE_MS = 700;
const MAX = 4_000;
const COLOURS: NoteColour[] = ["yellow", "green", "blue", "pink", "grey"];

/** extensions R7, R9: one sticky note on the desktop. Typing saves; an outside change is read when nothing
 * is unsaved, and never overwritten. */
export function NoteWindow(props: { file: string }) {
  const { file } = props;
  const [doc, setDoc] = useState<NoteDoc | null>(null);
  const [text, setText] = useState("");
  const [note, setNote] = useState<string | null>(null);
  const dirty = useRef(false);
  const base = useRef("");
  const timer = useRef<number>(0);

  const fail = useCallback((e: unknown) => setNote(errorMessage(asAppError(e))), []);
  const read = useCallback(() => {
    if (dirty.current) return;
    noteRead(file).then(
      (d) => {
        if (dirty.current) return;
        setDoc(d);
        setText(d.text);
        base.current = d.hash;
      },
      // A note that can't be read puts itself away; the other windows go on (extensions FR-010).
      () => void hideNote(),
    );
  }, [file]);

  useEffect(() => {
    read();
    const offChanged = onExtChanged((ids) => ids.includes("notes") && read());
    const offFocus = getCurrentWindow().onFocusChanged(({ payload: focused }) => focused && read());
    return () => {
      for (const off of [offChanged, offFocus]) off.then((unlisten) => unlisten());
    };
  }, [read]);

  const save = useCallback(
    (value: string) => {
      noteSave(file, value, base.current).then(async (outcome) => {
        dirty.current = false;
        if (outcome.kind === "Saved") {
          base.current = outcome.hash;
          return;
        }
        // extensions R9: the file changed underneath; the person's text becomes a new note beside it.
        await newNote(value);
        setNote(t.notes.conflict);
        setText(outcome.text);
        base.current = outcome.hash;
      }, fail);
    },
    [file, fail],
  );

  const type = (value: string) => {
    setText(value);
    dirty.current = true;
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => save(value), SAVE_MS);
  };

  const style = (colour: NoteColour | null, onTop: boolean | null) =>
    noteStyle(file, colour, onTop).then(setDoc, fail);

  const colour = doc?.colour ?? "yellow";
  return (
    <div className={`sticky note-${colour}`}>
      <div className="sticky-bar" data-tauri-drag-region>
        {COLOURS.map((c) => (
          <button
            key={c}
            type="button"
            className={`swatch note-${c}`}
            aria-label={t.notes.colour(t.notes.colours[c])}
            aria-pressed={c === colour}
            onClick={() => void style(c, null)}
          />
        ))}
        <label className="check">
          <input
            type="checkbox"
            checked={doc?.onTop ?? false}
            onChange={(e) => void style(null, e.target.checked)}
          />
          {t.notes.onTop}
        </label>
        <button type="button" className="icon" aria-label={t.notes.hideNote} onClick={() => void hideNote()}>
          <span aria-hidden="true">{""}</span>
        </button>
      </div>
      <textarea
        aria-label={doc?.text.split("\n")[0] || t.notes.untitled}
        placeholder={t.notes.placeholder}
        value={text}
        maxLength={MAX}
        onChange={(e) => type(e.target.value)}
      />
      {note && (
        <p role="status" className="sticky-note">
          {note}
        </p>
      )}
    </div>
  );
}
