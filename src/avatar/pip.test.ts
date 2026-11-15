import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { PipRenderer } from "./pip";
import type { Shown } from "./renderer";

const shown = (patch: Partial<Shown>): Shown => ({
  activity: "idle",
  mood: "neutral",
  voice: 0,
  still: false,
  ...patch,
});

let host: HTMLElement;
let now = 0;

beforeEach(async () => {
  vi.useFakeTimers();
  vi.spyOn(performance, "now").mockImplementation(() => now);
  host = document.createElement("div");
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

async function pip() {
  const renderer = new PipRenderer(host);
  await renderer.load({ type: "static", character: "pip" });
  return { renderer, svg: host.querySelector("svg") as SVGSVGElement };
}

describe("PipRenderer", () => {
  // avatar FR-005: the mouth moves with the arriving text and closes when it stops.
  it("talks while text arrives and closes its mouth when it stops", async () => {
    const { renderer, svg } = await pip();
    const faces: string[] = [];
    for (const voice of [3, 6, 9, 12]) {
      now += 120;
      renderer.setState(shown({ activity: "speaking", mood: "happy", voice }));
      faces.push(svg.dataset.face as string);
    }
    expect(faces).toEqual(["open", "happy", "open", "happy"]);
    now += 120;
    renderer.setState(shown({ activity: "speaking", mood: "happy", voice: 15 }));
    expect(svg.dataset.face).toBe("open");
    vi.advanceTimersByTime(300);
    expect(svg.dataset.face).toBe("happy");
  });

  it("shows each mood, has a face of its own for sleeping and thinking, and holds still when asked", async () => {
    const { renderer, svg } = await pip();
    const face = () => [svg.dataset.expr, svg.dataset.face];
    renderer.setState(shown({ mood: "sad" }));
    expect(face()).toEqual(["sad", "sad"]);
    renderer.setState(shown({ activity: "sleeping", mood: "sad" }));
    expect(face()).toEqual(["asleep", "asleep"]);
    renderer.setState(shown({ activity: "thinking", mood: "sad" }));
    expect(face()).toEqual(["thinking", "thinking"]);
    renderer.setState(shown({ still: true }));
    expect(svg.dataset.still).toBe("true");
  });

  // AVA-8: the face turns toward the pointer, but not while holding still or thinking.
  it("follows the pointer only while it may", async () => {
    const { renderer, svg } = await pip();
    const look = () => ["--look-x", "--look-y"].map((name) => svg.style.getPropertyValue(name));
    renderer.setState(shown({}));
    renderer.pointer(1, -1);
    expect(look()).toEqual(["1.00", "-1.00"]);
    renderer.setState(shown({ activity: "thinking" }));
    expect(look()).toEqual(["0.00", "0.00"]);
    renderer.setState(shown({ still: true }));
    renderer.pointer(1, 1);
    expect(look()).toEqual(["0.00", "0.00"]);
  });

  // Two characters on one page must not share gradient or clip ids: a hidden one would unpaint the other.
  it("gives each copy its own ids", async () => {
    const first = (await pip()).svg.querySelector("clipPath")?.id;
    const second = (await pip()).svg.querySelector("clipPath")?.id;
    expect(second).not.toBe(first);
    expect(host.innerHTML).not.toContain("pip0-");
  });

  // weather FR-015: the weather is felt awake only.
  it("shows how it feels the weather, and not while asleep", async () => {
    const { renderer, svg } = await pip();
    renderer.setState(shown({ weather: ["shelter", "shiver"] }));
    expect(svg.dataset.weather).toBe("shelter shiver");
    renderer.setState(shown({ activity: "sleeping", weather: ["shelter"] }));
    expect(svg.dataset.weather).toBe("");
    renderer.setState(shown({}));
    expect(svg.dataset.weather).toBe("");
  });
});
