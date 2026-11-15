// Pip drawn in parts so it can breathe, blink, talk, show its mood, follow the pointer and sleep without Live2D.
// The parts are in pip.svg and the look of each state in pip.css; this renderer only sets which state is shown.

import type { AvatarLook } from "../bindings/AvatarLook";
import "./pip.css";
import pip from "./pip.svg?raw";
import type { AvatarRenderer, Shown } from "./renderer";

/** One drawing per character that has one; the others keep their still portrait. */
const DRAWINGS: Record<string, string> = { pip };
/** Each copy on a page gets its own gradient and clip ids, so a hidden copy never unpaints another. */
let copies = 0;

export const isAnimated = (look: AvatarLook) => look.type !== "picture" && look.character in DRAWINGS;

/** How long the mouth stays as it is while text streams in, and closes after the text stops (avatar FR-005). */
const MOUTH_MS = 110;
const QUIET_MS = 300;

/** The animated drawing: the state goes to data attributes the styles act on; the pointer turns the face. */
export class PipRenderer implements AvatarRenderer {
  private svg: SVGSVGElement | null = null;
  private voice = 0;
  private open = false;
  private flipped = 0;
  private quiet = 0;
  private shown: Shown | null = null;

  constructor(private host: HTMLElement) {}

  async load(look: AvatarLook) {
    this.host.innerHTML = (DRAWINGS[look.character] ?? "").replaceAll("pip0-", `pip${++copies}-`);
    this.svg = this.host.querySelector("svg");
  }

  setState(shown: Shown) {
    this.shown = shown;
    const svg = this.svg;
    if (!svg) return;
    const speaking = shown.activity === "speaking";
    if (speaking && shown.voice !== this.voice) {
      const now = performance.now();
      if (now - this.flipped > MOUTH_MS) {
        this.open = !this.open;
        this.flipped = now;
      }
      window.clearTimeout(this.quiet);
      this.quiet = window.setTimeout(() => {
        this.open = false;
        if (this.shown) this.setState(this.shown);
      }, QUIET_MS);
    }
    if (!speaking) this.open = false;
    this.voice = shown.voice;
    svg.dataset.activity = shown.activity;
    svg.dataset.still = String(shown.still);
    // Sleeping and thinking have a face of their own; otherwise the mood shows, with the mouth open on a word.
    const expr =
      shown.activity === "sleeping" ? "asleep" : shown.activity === "thinking" ? "thinking" : shown.mood;
    svg.dataset.expr = expr;
    svg.dataset.face = this.open ? "open" : expr;
    // weather FR-015: the weather, felt awake only.
    svg.dataset.weather = shown.activity === "sleeping" ? "" : (shown.weather ?? []).join(" ");
    if (shown.activity !== "idle" && shown.activity !== "listening") this.pointer(0, 0);
  }

  pointer(x: number, y: number) {
    if (this.shown?.still) return;
    this.svg?.style.setProperty("--look-x", x.toFixed(2));
    this.svg?.style.setProperty("--look-y", y.toFixed(2));
  }

  dispose() {
    window.clearTimeout(this.quiet);
    this.host.innerHTML = "";
  }
}
