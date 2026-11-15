import { type ReactNode, useEffect, useRef } from "react";
import { t } from "./strings";

/** An in-window confirmation, reachable by keyboard and announced by screen readers (onboarding FR-009). */
export function Confirm(props: {
  message: string;
  confirm: string;
  /** Danger for a loss the user can't undo; Primary when nothing is deleted. */
  danger?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  /** Fields between the message and the buttons, such as compact-now's focus note. */
  children?: ReactNode;
}) {
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), []);
  return (
    <div className="scrim">
      <div role="alertdialog" aria-modal="true" aria-label={props.message} className="dialog">
        <p>{props.message}</p>
        {props.children}
        <div className="row">
          <button type="button" ref={cancel} onClick={props.onCancel}>
            {t.cancel}
          </button>
          <button type="button" className={props.danger ? "danger" : "primary"} onClick={props.onConfirm}>
            {props.confirm}
          </button>
        </div>
      </div>
    </div>
  );
}
