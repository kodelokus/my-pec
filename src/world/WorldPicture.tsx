import {
  type CSSProperties,
  type KeyboardEvent,
  type PointerEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { DrawnItem } from "../bindings/DrawnItem";
import type { WorldPalette } from "../bindings/WorldPalette";
import { drawing } from "./drawings";
import { TOOLTIP_ID, Tooltip } from "./Tooltip";

/** Grid cells are this many SVG units; each drawing is 32 × 32 (world R9). */
export const CELL = 32;

export type Box = { x: number; y: number; width: number; height: number };

/** The view box that fits every item, with a cell of margin. */
export function fitAll(items: DrawnItem[]): Box {
  if (items.length === 0) return { x: -2 * CELL, y: -2 * CELL, width: 4 * CELL, height: 4 * CELL };
  const xs = items.map((i) => i.x);
  const ys = items.map((i) => i.y);
  const [minX, maxX, minY, maxY] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  const side = Math.max(maxX - minX, maxY - minY) + 3;
  const cx = (minX + maxX + 1) / 2;
  const cy = (minY + maxY + 1) / 2;
  return { x: (cx - side / 2) * CELL, y: (cy - side / 2) * CELL, width: side * CELL, height: side * CELL };
}

/** The box scaled by `factor` around a point, kept between four cells and twice the whole world. */
export function zoom(box: Box, factor: number, whole: Box, cx?: number, cy?: number): Box {
  const [px, py] = [cx ?? box.x + box.width / 2, cy ?? box.y + box.height / 2];
  const width = Math.min(Math.max(box.width * factor, 4 * CELL), Math.max(whole.width * 2, 4 * CELL));
  const scale = width / box.width;
  return { x: px - (px - box.x) * scale, y: py - (py - box.y) * scale, width, height: box.height * scale };
}

const pan = (box: Box, dx: number, dy: number): Box => ({ ...box, x: box.x + dx, y: box.y + dy });

const centredOn = (box: Box, item: DrawnItem): Box => ({
  ...box,
  x: (item.x + 0.5) * CELL - box.width / 2,
  y: (item.y + 0.5) * CELL - box.height / 2,
});

/** What a key does to the view: `+` and `-` zoom, the arrows pan, `0` shows the whole world. */
function keyMove(key: string, box: Box, whole: Box): Box | null {
  const step = box.width / 10;
  switch (key) {
    case "+":
    case "=":
      return zoom(box, 0.8, whole);
    case "-":
      return zoom(box, 1.25, whole);
    case "0":
      return whole;
    case "ArrowLeft":
      return pan(box, -step, 0);
    case "ArrowRight":
      return pan(box, step, 0);
    case "ArrowUp":
      return pan(box, 0, -step);
    case "ArrowDown":
      return pan(box, 0, step);
    default:
      return null;
  }
}

const symbolId = (item: string) => `w-${item.replace(/[^a-z0-9]/g, "-")}`;
const shades = ["", "brightness(1.12)", "brightness(0.86)"];

/** The world as one SVG: a symbol per kind of item, a use per drawn item, coloured by the palette. It opens on the
 * whole world and zooms and pans with mouse, touchpad and keyboard (world FR-027). Nothing animates (world FR-032, FR-034). */
export function WorldPicture(props: {
  items: DrawnItem[];
  palette: WorldPalette;
  label: string;
  /** An item to bring to the middle of the view, for example one chosen in the list. */
  centre: number | null;
  selected: number | null;
  /** journey FR-030: items drawn after this day are hidden; none shows today's world. */
  until: string | null;
  onSelect: (index: number) => void;
  itemLabel: (item: DrawnItem) => string;
  /** The tooltip's name and date for an item; its keywords come with the item. */
  itemTip: (item: DrawnItem) => { name: string; date: string };
  controls: { label: string; zoomIn: string; zoomOut: string; whole: string };
}) {
  const { palette } = props;
  const style = {
    "--w-ground": palette.ground,
    "--w-water": palette.water,
    "--w-leaf": palette.leaf,
    "--w-bark": palette.bark,
    "--w-stone": palette.stone,
    "--w-accent": palette.accent,
    // journey R4: fixed neutrals for things that are white or black in life.
    "--w-light": "#f7f4ec",
    "--w-dark": "#4a4540",
  } as CSSProperties;
  const whole = useMemo(() => fitAll(props.items), [props.items]);
  const [box, setBox] = useState(whole);
  const drag = useRef<{ x: number; y: number } | null>(null);
  const svg = useRef<SVGSVGElement>(null);
  useEffect(() => setBox(whole), [whole]);
  useEffect(() => {
    const item = props.centre === null ? undefined : props.items[props.centre];
    if (item) setBox((b) => centredOn(b, item));
  }, [props.centre, props.items]);

  // journey FR-024, FR-025: the item whose tooltip shows, from hover, focus or a tap.
  const [tip, setTip] = useState<number | null>(null);
  useEffect(() => {
    if (tip === null) return;
    const close = (e: globalThis.KeyboardEvent) => {
      if (e.key === "Escape") setTip(null);
    };
    document.addEventListener("keydown", close);
    return () => document.removeEventListener("keydown", close);
  }, [tip]);
  const leave = (e: PointerEvent) => {
    const to = e.relatedTarget;
    const stays = to instanceof Element && (to.closest(`#${TOOLTIP_ID}`) || to.closest(".world-item"));
    if (!stays) setTip(null);
  };
  const unitsPerPixel = () => box.width / (svg.current?.clientWidth || box.width);
  const onKey = (e: KeyboardEvent) => {
    const next = keyMove(e.key, box, whole);
    if (next) {
      e.preventDefault();
      setBox(next);
    }
  };
  const kinds = [...new Set(props.items.map((i) => i.item))];
  const { x, y, width, height } = box;
  return (
    <div className="world-viewport">
      <div className="row world-controls">
        <button type="button" onClick={() => setBox((b) => zoom(b, 0.8, whole))}>
          {props.controls.zoomIn}
        </button>
        <button type="button" onClick={() => setBox((b) => zoom(b, 1.25, whole))}>
          {props.controls.zoomOut}
        </button>
        <button type="button" onClick={() => setBox(whole)}>
          {props.controls.whole}
        </button>
      </div>
      {/* The map is one keyboard stop for zooming and panning; its items are the buttons inside it. */}
      <div
        className="world-map"
        role="application"
        aria-label={props.controls.label}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: the map takes keys to zoom and pan (world FR-027)
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.target === e.currentTarget) onKey(e);
        }}
      >
        <svg
          ref={svg}
          className="world-picture"
          viewBox={`${x} ${y} ${width} ${height}`}
          style={style}
          onWheel={(e) => {
            const rect = e.currentTarget.getBoundingClientRect();
            const cx = box.x + (e.clientX - rect.left) * unitsPerPixel();
            const cy = box.y + (e.clientY - rect.top) * unitsPerPixel();
            setBox(zoom(box, e.deltaY < 0 ? 0.9 : 1.1, whole, cx, cy));
          }}
          onPointerDown={(e) => {
            drag.current = { x: e.clientX, y: e.clientY };
          }}
          onPointerMove={(e) => {
            if (!drag.current || e.buttons === 0) return;
            const k = unitsPerPixel();
            setBox(pan(box, (drag.current.x - e.clientX) * k, (drag.current.y - e.clientY) * k));
            drag.current = { x: e.clientX, y: e.clientY };
          }}
          onPointerUp={() => {
            drag.current = null;
          }}
        >
          <title>{props.label}</title>
          <defs>
            {kinds.map((kind) => (
              <symbol
                key={kind}
                id={symbolId(kind)}
                viewBox="0 0 32 32"
                // biome-ignore lint/security/noDangerouslySetInnerHtml: bundled drawings, never model output (world R9)
                dangerouslySetInnerHTML={{ __html: drawing(kind) }}
              />
            ))}
          </defs>
          <rect x={x} y={y} width={width} height={height} fill="var(--w-ground)" />
          {props.items.map((item, i) => {
            if (props.until !== null && item.day > props.until) return null;
            const size = CELL * item.size;
            const offset = (CELL - size) / 2;
            // Mirror around the cell's centre line.
            const flip = item.flip ? `translate(${2 * item.x * CELL + CELL} 0) scale(-1 1)` : "";
            return (
              // biome-ignore lint/suspicious/noArrayIndexKey: the file's order is the item's identity
              <g key={i} transform={flip || undefined}>
                {/* biome-ignore lint/a11y/useSemanticElements: SVG has no button element; the use is the item */}
                <use
                  href={`#${symbolId(item.item)}`}
                  x={item.x * CELL + offset}
                  y={item.y * CELL + offset}
                  width={size}
                  height={size}
                  style={{ filter: shades[item.shade] || undefined }}
                  className="world-item"
                  tabIndex={0}
                  role="button"
                  aria-label={props.itemLabel(item)}
                  aria-pressed={props.selected === i}
                  aria-describedby={tip === i ? TOOLTIP_ID : undefined}
                  onClick={() => props.onSelect(i)}
                  onPointerEnter={(e) => {
                    if (e.pointerType !== "touch") setTip(i);
                  }}
                  onPointerLeave={leave}
                  onPointerDown={(e) => {
                    if (e.pointerType === "touch") setTip((t) => (t === i ? null : i));
                  }}
                  onFocus={() => setTip(i)}
                  onBlur={() => setTip((t) => (t === i ? null : t))}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      props.onSelect(i);
                    }
                  }}
                />
              </g>
            );
          })}
          {props.selected !== null && props.items[props.selected] && (
            <rect
              className="world-selection"
              x={props.items[props.selected].x * CELL + 1}
              y={props.items[props.selected].y * CELL + 1}
              width={CELL - 2}
              height={CELL - 2}
              rx={6}
            />
          )}
        </svg>
        {tip !== null && props.items[tip] && (
          <Tooltip
            {...props.itemTip(props.items[tip])}
            keywords={props.items[tip].keywords}
            left={(props.items[tip].x * CELL + CELL / 2 - x) / unitsPerPixel()}
            top={(props.items[tip].y * CELL - y) / unitsPerPixel()}
            onLeave={leave}
          />
        )}
      </div>
    </div>
  );
}
