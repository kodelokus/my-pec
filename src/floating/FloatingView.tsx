import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRef } from "react";
import { openChat } from "../api/commands";
import { AvatarView } from "../avatar/AvatarView";
import { useWeather } from "../extensions/weather/useWeather";

/** A press that moves this far drags the window instead of opening the chat (avatar R8). */
const DRAG_PX = 4;

/** avatar FR-024 to FR-027: the character alone on the desktop. It can be dragged, and a click opens the chat. */
export function FloatingView(props: { name: string }) {
  const press = useRef<{ x: number; y: number } | null>(null);
  const weather = useWeather();
  return (
    // biome-ignore lint/a11y/useSemanticElements: the whole window is the button, and it must also start a drag.
    <div
      className="floating-view"
      role="button"
      tabIndex={0}
      onPointerDown={(e) => {
        press.current = { x: e.clientX, y: e.clientY };
      }}
      onPointerMove={(e) => {
        const from = press.current;
        if (from && Math.hypot(e.clientX - from.x, e.clientY - from.y) >= DRAG_PX) {
          press.current = null;
          void getCurrentWindow().startDragging();
        }
      }}
      onPointerUp={() => {
        if (press.current) void openChat();
        press.current = null;
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          void openChat();
        }
      }}
    >
      <AvatarView size={180} name={props.name} weather={weather} />
    </div>
  );
}
