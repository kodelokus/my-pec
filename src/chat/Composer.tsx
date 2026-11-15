import { useEffect, useRef } from "react";
import { setListening } from "../api/commands";
import type { SendRefusal } from "../bindings/SendRefusal";
import { t } from "../strings";

/** chat FR-002, FR-005, FR-022: the message field, with Send and Stop. Without a key the main view's alert takes its place (chat FR-029). */
export function Composer(props: {
  name: string;
  value: string;
  onChange: (value: string) => void;
  onSend: () => void;
  onStop: () => void;
  replying: boolean;
  refusal: SendRefusal | null;
  hasKey: boolean;
}) {
  // avatar FR-004: the character listens while the box has text and focus; the core hears only changes.
  const listening = useRef(false);
  const focused = useRef(false);
  const listen = (on: boolean) => {
    if (on === listening.current) return;
    listening.current = on;
    void setListening(on).catch(() => {});
  };
  // biome-ignore lint/correctness/useExhaustiveDependencies: only the text matters; sending empties it.
  useEffect(() => listen(focused.current && props.value !== ""), [props.value]);
  if (!props.hasKey) return null;
  return (
    <form
      className="composer"
      onSubmit={(e) => {
        e.preventDefault();
        props.onSend();
      }}
    >
      <label className="visually-hidden" htmlFor="composer">
        {t.chat.composerLabel(props.name)}
      </label>
      <textarea
        id="composer"
        rows={2}
        value={props.value}
        placeholder={t.chat.composerLabel(props.name)}
        aria-describedby={props.refusal ? "composer-refusal" : undefined}
        onFocus={(e) => {
          focused.current = true;
          listen(e.target.value !== "");
        }}
        onBlur={() => {
          focused.current = false;
          listen(false);
        }}
        onChange={(e) => props.onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
            e.preventDefault();
            props.onSend();
          } else if (e.key === "Escape" && props.replying) {
            e.preventDefault();
            props.onStop();
          }
        }}
      />
      {props.replying ? (
        <button type="button" onClick={props.onStop}>
          {t.chat.stop}
        </button>
      ) : (
        <button type="submit" className="primary">
          {t.chat.send}
        </button>
      )}
      {props.refusal && (
        <p id="composer-refusal" className="hint composer-refusal">
          {t.chat.refusals[props.refusal]}
        </p>
      )}
    </form>
  );
}
