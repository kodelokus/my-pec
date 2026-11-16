// Pip drawn in parts so it can breathe, blink, talk, show its mood, follow the pointer and sleep without Live2D.
// The parts are in pip.svg and the look of each state in pip.css; this renderer only sets which state is shown.

import type { AvatarLook } from "../bindings/AvatarLook";
import type { Mood } from "../bindings/Mood";
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

/** The whole-body moves and how long each plays (pip.css). */
const MOVES = { dance: 2400, roll: 2000, twirl: 1400, bounce: 1500 } as const;
export type Move = keyof typeof MOVES;
/** A reply bringing one of these moods sets Pip moving. */
const MOOD_MOVES: Partial<Record<Mood, Move>> = {
  happy: "bounce",
  playful: "roll",
  excited: "dance",
  proud: "twirl",
};
/** Idle and awake, Pip moves on its own once in a while, somewhere in this range. */
const FIDGET_MS = [45_000, 120_000] as const;

/** The animated drawing: the state goes to data attributes the styles act on; the pointer turns the face. */
export class PipRenderer implements AvatarRenderer {
  private svg: SVGSVGElement | null = null;
  private voice = 0;
  private open = false;
  private flipped = 0;
  private quiet = 0;
  private moving = 0;
  private fidget = 0;
  private shown: Shown | null = null;

  constructor(private host: HTMLElement) {}

  async load(look: AvatarLook) {
    this.host.innerHTML = (DRAWINGS[look.character] ?? "").replaceAll("pip0-", `pip${++copies}-`);
    this.svg = this.host.querySelector("svg");
    this.scheduleFidget();
  }

  setState(shown: Shown) {
    const before = this.shown;
    this.shown = shown;
    const svg = this.svg;
    if (!svg) return;
    const move = MOOD_MOVES[shown.mood];
    if (move && shown.mood !== before?.mood && shown.activity !== "sleeping") this.play(move);
    if (shown.still || shown.activity === "sleeping") this.stop();
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

  /** Plays a move once, from the start even when the same one is playing; never while holding still. */
  play(move: Move) {
    const svg = this.svg;
    if (!svg || this.shown?.still) return;
    this.stop();
    // Reading the layout lets the animation start again when the same move follows itself.
    void svg.getBoundingClientRect();
    svg.dataset.move = move;
    this.moving = window.setTimeout(() => this.stop(), MOVES[move]);
  }

  private stop() {
    window.clearTimeout(this.moving);
    if (this.svg) delete this.svg.dataset.move;
  }

  /** Idle, awake and on screen, a random move now and then; never in a hidden window, so it stays cheap. */
  private scheduleFidget() {
    window.clearTimeout(this.fidget);
    const [least, most] = FIDGET_MS;
    this.fidget = window.setTimeout(
      () => {
        const s = this.shown;
        if (s && s.activity === "idle" && !s.still && !document.hidden && !this.svg?.dataset.move) {
          const moves = Object.keys(MOVES) as Move[];
          this.play(moves[Math.floor(Math.random() * moves.length)]);
        }
        this.scheduleFidget();
      },
      least + Math.random() * (most - least),
    );
  }

  dispose() {
    window.clearTimeout(this.quiet);
    window.clearTimeout(this.fidget);
    this.stop();
    this.host.innerHTML = "";
  }
}
