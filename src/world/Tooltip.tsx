import type { PointerEvent } from "react";

export const TOOLTIP_ID = "world-tip";

/** journey FR-024, FR-025: an item's name, its date and its day's keywords, its own first. Plain text only. */
export function Tooltip(props: {
  name: string;
  date: string;
  keywords: string[];
  left: number;
  top: number;
  onLeave: (e: PointerEvent) => void;
}) {
  return (
    <div
      id={TOOLTIP_ID}
      role="tooltip"
      className="world-tip"
      style={{ left: props.left, top: props.top }}
      onPointerLeave={props.onLeave}
    >
      <strong>{props.name}</strong>
      <span>{props.date}</span>
      {props.keywords.length > 0 && <span className="world-tip-keywords">{props.keywords.join(" · ")}</span>}
    </div>
  );
}
