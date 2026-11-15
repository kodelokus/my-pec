import { emit as send } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AvatarLook } from "../bindings/AvatarLook";
import type { AvatarState } from "../bindings/AvatarState";
import { AvatarView } from "./AvatarView";
import { problemFor, tryAgain } from "./problem";
import type { AvatarRenderer, Live2D, Shown } from "./renderer";

// A fake Live2D renderer behind the interface; each test decides how its load ends.
const fake = vi.hoisted(() => ({
  build: true,
  load: (): Promise<void> => Promise.resolve(),
  made: [] as { setState: Shown[]; pointer: number; lost: () => void; disposed: boolean }[],
}));

vi.mock("virtual:live2d", () => ({
  get loadLive2D(): (() => Promise<Live2D>) | null {
    if (!fake.build) return null;
    return async () => (_canvas, onLost) => {
      const made = { setState: [] as Shown[], pointer: 0, lost: onLost, disposed: false };
      fake.made.push(made);
      const renderer: AvatarRenderer = {
        load: () => fake.load(),
        setState: (shown) => made.setState.push(shown),
        pointer: () => {
          made.pointer += 1;
        },
        dispose: () => {
          made.disposed = true;
        },
      };
      return renderer;
    };
  },
}));

const LIVE: AvatarLook = {
  type: "live2d",
  character: "pip",
  model: "pip/pip.model3.json",
  moods: {},
  activities: {},
};
const STILL: AvatarLook = { type: "static", character: "pip" };
const reason = () => problemFor(JSON.stringify(LIVE));
let manifestReads = 0;
let state: AvatarState = { activity: "idle", mood: "neutral", voice: 0, look: 0 };

function backend(look: AvatarLook) {
  manifestReads = 0;
  mockIPC(
    (cmd) => {
      if (cmd === "avatar_state") return state;
      if (cmd !== "avatar_manifest") throw new Error(cmd);
      manifestReads += 1;
      return { look, problem: null };
    },
    { shouldMockEvents: true },
  );
}

const emit = (patch: Partial<AvatarState>) => {
  state = { ...state, ...patch };
  return act(() => send("avatar_state", state));
};

const media = (matches: boolean) => {
  const listeners: (() => void)[] = [];
  const query = {
    matches,
    addEventListener: (_: string, f: () => void) => listeners.push(f),
    removeEventListener: () => {},
  };
  window.matchMedia = vi.fn(() => query) as never;
  return (next: boolean) =>
    act(() => {
      query.matches = next;
      for (const f of listeners) f();
    });
};

beforeEach(() => {
  state = { activity: "idle", mood: "neutral", voice: 0, look: 0 };
  fake.build = true;
  fake.load = () => Promise.resolve();
  fake.made = [];
  tryAgain();
  HTMLCanvasElement.prototype.getContext = vi.fn(() => ({})) as never;
  media(false);
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

/** What stands in for the animated character: here, Pip's own drawing. */
const shownImg = () => screen.getByRole("img", { hidden: true }).querySelector(".figure") as HTMLElement;
const drawing = () => shownImg().querySelector("svg") as SVGSVGElement;

describe("AvatarView", () => {
  it("follows the state, names it in its label and is not a live region", async () => {
    backend(LIVE);
    render(<AvatarView size={100} name="Steve" />);
    const avatar = await screen.findByRole("img", { name: "Steve" });
    await waitFor(() => expect(fake.made).toHaveLength(1));
    expect(avatar).not.toHaveAttribute("aria-live");
    await emit({ activity: "thinking" });
    expect(avatar).toHaveAccessibleName("Steve, thinking");
    await emit({ activity: "idle", mood: "happy" });
    expect(avatar).toHaveAccessibleName("Steve, happy");
    await emit({ activity: "sleeping", mood: "neutral" });
    expect(avatar).toHaveAccessibleName("Steve, asleep");
    expect(fake.made[0].setState.map((s) => s.activity)).toContain("thinking");
  });

  it("forwards the pointer only while idle or listening", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue(new DOMRect(0, 0, 100, 100));
    backend(LIVE);
    render(<AvatarView size={100} />);
    await waitFor(() => expect(fake.made).toHaveLength(1));
    await waitFor(() => expect(document.querySelector("canvas")).not.toHaveAttribute("hidden"));
    fireEvent.pointerMove(document, { clientX: 10, clientY: 10 });
    await emit({ activity: "listening" });
    fireEvent.pointerMove(document, { clientX: 20, clientY: 10 });
    await emit({ activity: "speaking" });
    fireEvent.pointerMove(document, { clientX: 30, clientY: 10 });
    expect(fake.made[0].pointer).toBe(2);
  });

  it("shows the portrait first and keeps it, with the reason, whenever the character can't be drawn", async () => {
    const cases: [string, () => void, string][] = [
      ["not in this build", () => (fake.build = false), "notInBuild"],
      ["no WebGL", () => (HTMLCanvasElement.prototype.getContext = vi.fn(() => null) as never), "noGraphics"],
      ["missing files", () => (fake.load = () => Promise.reject("missing")), "missing"],
      ["an unreadable model", () => (fake.load = () => Promise.reject(new Error("bad moc"))), "damaged"],
    ];
    for (const [what, arrange, problem] of cases) {
      tryAgain();
      arrange();
      backend(LIVE);
      const { unmount } = render(<AvatarView size={100} name="Steve" />);
      await waitFor(() => expect(reason()).toBe(problem), { timeout: 1000 });
      expect(shownImg(), what).toBeVisible();
      unmount();
      fake.build = true;
      fake.load = () => Promise.resolve();
      HTMLCanvasElement.prototype.getContext = vi.fn(() => ({})) as never;
    }
  });

  it("gives up after 5 seconds, and swaps back if the character stops while running", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    fake.load = () => new Promise(() => {});
    backend(LIVE);
    const { rerender } = render(<AvatarView size={100} name="Steve" />);
    await waitFor(() => expect(fake.made).toHaveLength(1));
    expect(shownImg()).toBeVisible();
    await act(() => vi.advanceTimersByTimeAsync(5_000));
    expect(reason()).toBe("timedOut");
    rerender(<AvatarView size={100} name="Steve" moment="happy" />);
    expect(fake.made).toHaveLength(1);
    vi.useRealTimers();

    tryAgain();
    fake.load = () => Promise.resolve();
    await waitFor(() => expect(fake.made).toHaveLength(2));
    await waitFor(() => expect(shownImg()).not.toBeVisible());
    act(() => fake.made[1].lost());
    expect(reason()).toBe("stopped");
    expect(shownImg()).toBeVisible();
  });

  it("never tries the character for a still look, and Pip's drawing sleeps and shows the mood", async () => {
    backend(STILL);
    render(<AvatarView size={100} name="Steve" moment="concerned" />);
    const avatar = await screen.findByRole("img", { name: "Steve, concerned" });
    await waitFor(() => expect(drawing()).toHaveAttribute("data-face", "concerned"));
    await emit({ activity: "sleeping" });
    expect(drawing()).toHaveAttribute("data-face", "asleep");
    expect(avatar.querySelector("img")).not.toBeVisible();
    expect(fake.made).toHaveLength(0);
    expect(reason()).toBeNull();
  });

  it("reads the look again when it changes, and holds still while reduced motion is on", async () => {
    const reduce = media(true);
    backend(LIVE);
    render(<AvatarView size={100} />);
    await waitFor(() => expect(fake.made).toHaveLength(1));
    await waitFor(() => expect(fake.made[0].setState.at(-1)?.still).toBe(true));
    reduce(false);
    expect(fake.made[0].setState.at(-1)?.still).toBe(false);
    await emit({ look: 1 });
    await waitFor(() => expect(manifestReads).toBe(2));
  });
});
