// How the webview draws the companion: one interface, the still portrait that always works, and the Live2D
// renderer when the build has it (architecture 9.2, avatar R3).

import type { AvatarLook } from "../bindings/AvatarLook";
import type { AvatarState } from "../bindings/AvatarState";
import type { Reaction } from "../extensions/weather/reaction";
import { portrait } from "../portraits";

/**
 * What a renderer shows: the state, with an interface moment's mood, whether to hold still (avatar FR-013, FR-021), and
 * how it feels the weather (weather R7), which a renderer may ignore.
 */
export type Shown = Pick<AvatarState, "activity" | "mood" | "voice"> & {
  still: boolean;
  weather?: Reaction[];
};

/** Why a window shows the portrait instead of its character (avatar R4). */
export type RendererProblem = "missing" | "damaged" | "noGraphics" | "notInBuild" | "timedOut" | "stopped";

/** Reads a file of the look by its path in the manifest. */
export type ReadFile = (path: string) => Promise<Blob>;

export interface AvatarRenderer {
  /** Resolves once the look can be shown; rejects with a `RendererProblem`. */
  load(look: AvatarLook, read: ReadFile): Promise<void>;
  setState(shown: Shown): void;
  /** The pointer, from -1 to 1 across the character's box (AVA-8). */
  pointer(x: number, y: number): void;
  dispose(): void;
}

/** Makes a Live2D renderer on a canvas; `onLost` is called if it stops while running. */
export type Live2D = (canvas: HTMLCanvasElement, onLost: () => void) => AvatarRenderer;

/** The character's still portrait, asleep while it sleeps, or the person's own picture (avatar FR-016, FR-030). */
export class PortraitRenderer implements AvatarRenderer {
  private character = "";
  private picture: string | null = null;

  constructor(private img: HTMLImageElement) {}

  async load(look: AvatarLook, read: ReadFile) {
    this.character = look.character;
    if (look.type === "picture") {
      try {
        this.picture = URL.createObjectURL(await read(look.image));
      } catch {
        // The picture is gone or unreadable: the character's portrait stands in.
      }
    }
    this.img.src = this.picture ?? portrait(this.character);
  }

  setState(shown: Shown) {
    this.img.src = this.picture ?? portrait(this.character, { sleeping: shown.activity === "sleeping" });
  }

  pointer() {}

  dispose() {
    if (this.picture) URL.revokeObjectURL(this.picture);
  }
}
