import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { DrawnItem } from "../bindings/DrawnItem";
import type { JourneyDay } from "../bindings/JourneyDay";
import type { WorldView as World } from "../bindings/WorldView";
import { fitAll } from "./WorldPicture";
import { WorldView } from "./WorldView";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  worldView: vi.fn(),
  forgetTopic: vi.fn(),
  startFreshContext: vi.fn(),
}));

const item = (id: string, x: number, y: number, day: string, extra: Partial<DrawnItem> = {}): DrawnItem => ({
  item: id,
  known: true,
  x,
  y,
  size: 1,
  flip: false,
  shade: 0,
  day,
  topic: null,
  keywords: [],
  ...extra,
});

const world = (items: DrawnItem[], journey: JourneyDay[] = [], makeup: World["makeup"] = []): World => ({
  born: "2026-11-14",
  palette: {
    ground: "#dfe8c8",
    water: "#8cc3d8",
    leaf: "#6aa84f",
    bark: "#8b5e3c",
    stone: "#a7a39a",
    accent: "#e8a33d",
  },
  items,
  journey,
  makeup,
  skippedLines: 0,
  journeySkipped: 0,
  journeyHash: "h1",
});

const companion = { name: "Steve", characterId: "pip" };
const view = (onOpenNotes = vi.fn()) =>
  render(<WorldView companion={companion} onClose={vi.fn()} onOpenNotes={onOpenNotes} />);

/** A world whose 15 November was drawn with running shoes and a cake beside a pebble. */
const diary = () =>
  world(
    [
      item("home.cottage", 0, 0, "2026-11-14"),
      item("flower.daisy", 1, 0, "2026-11-14"),
      item("shoes.running", 2, 0, "2026-11-15", {
        topic: { day: "2026-11-15", n: 0 },
        keywords: ["10 km run", "sister's birthday"],
      }),
      item("cake.birthday", 2, 1, "2026-11-15", {
        topic: { day: "2026-11-15", n: 1 },
        keywords: ["sister's birthday", "10 km run"],
      }),
      item("pebble.round", 3, 1, "2026-11-15", { keywords: ["10 km run", "sister's birthday"] }),
    ],
    [
      {
        day: "2026-11-15",
        topics: [
          {
            n: 0,
            drawn: "shoes.running",
            item: 2,
            story: {
              keyword: "10 km run",
              line: "You ran ten kilometres before work.",
              subjects: ["running"],
              related: false,
              odds: [
                { item: "shoes.running", chance: 0.312 },
                { item: "bottle.water", chance: 0.204 },
              ],
              rest: 0.484,
            },
          },
          {
            n: 1,
            drawn: "cake.birthday",
            item: 3,
            story: {
              keyword: "sister's birthday",
              line: "You baked a cake for your sister's birthday.",
              subjects: ["celebration"],
              related: true,
              odds: [{ item: "cake.birthday", chance: 0.5 }],
              rest: 0.5,
            },
          },
        ],
        scenery: [4],
        hasNotes: true,
      },
    ],
    [
      { subject: "running", share: 0.5 },
      { subject: "celebration", share: 0.5 },
    ],
  );

beforeEach(() => {
  vi.mocked(api.worldView).mockResolvedValue(world([item("home.cottage", 0, 0, "2026-11-14")]));
});

describe("WorldView", () => {
  // world FR-029, FR-035, FR-036, journey FR-033: led by the companion, facts only, no pressure, no id.
  it("shows the home and birth date, and no id, streaks or stages", async () => {
    view();
    expect(
      await screen.findByText("This is my home. Everything I draw grows out from here."),
    ).toBeInTheDocument();
    expect(screen.queryByText("K7Q2-M9TX-3HPA")).toBeNull();
    expect(screen.queryByRole("button", { name: "Copy" })).toBeNull();
    expect(
      screen.getByText(/During the night's dream I draw something for what happened in your day/),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^Cottage, drawn/ })).toBeInTheDocument();
    expect(screen.getByText("Nothing drawn for your days yet.")).toBeInTheDocument();
    expect(document.body).not.toHaveTextContent(/streak|in a row|missed|stage|next level|days since|redraw/i);
  });

  it("counts what was drawn and names the latest day's items when no day has a story", async () => {
    vi.mocked(api.worldView).mockResolvedValue(
      world([
        item("home.cottage", 0, 0, "2026-11-14"),
        item("flower.daisy", 1, 0, "2026-11-14"),
        item("flower.daisy", 0, 1, "2026-11-15"),
        item("thing.unknown", 1, 1, "2026-11-15", { known: false }),
      ]),
    );
    view();
    const line = await screen.findByText(/^I've drawn 3 things around my home\. Most recently: /);
    expect(line).toHaveTextContent("Unknown item (thing.unknown)");
    await userEvent.click(screen.getByRole("button", { name: /^Unknown item \(thing.unknown\), drawn/ }));
    expect(screen.getAllByRole("status").at(-1)).toHaveTextContent(
      /^Unknown item \(thing.unknown\), drawn .*2026/,
    );
  });
});

describe("WorldView as a diary", () => {
  // journey FR-026, FR-027, FR-032: the newest day's story leads, and the list reads by day.
  it("tells the newest day and lists each topic beside its object, then the scenery", async () => {
    vi.mocked(api.worldView).mockResolvedValue(diary());
    view();
    expect(
      await screen.findByText(
        /^For .*15.* I drew running shoes for “10 km run” and birthday cake for “sister's birthday”\.$/,
      ),
    ).toBeInTheDocument();
    const list = screen.getByRole("region", { name: "The diary" });
    const day = within(list).getByRole("region", { name: /15/ });
    expect(day).toHaveTextContent(/Running shoes — “10 km run”You ran ten kilometres before work\./);
    expect(within(day).getByText("Also drawn that day")).toBeInTheDocument();
    expect(within(day).getByRole("button", { name: "Round pebble" })).toBeInTheDocument();
    // Selecting a topic object tells its story.
    await userEvent.click(within(day).getByRole("button", { name: "Birthday cake" }));
    expect(screen.getAllByRole("status").at(-1)).toHaveTextContent(
      /^Birthday cake, .*15.*: “sister's birthday”\. You baked a cake for your sister's birthday\.$/,
    );
    // Keywords are never standing labels on the picture.
    expect(document.querySelector(".world-picture")?.textContent).not.toContain("10 km run");
  });

  // journey FR-028, FR-029: why this one, and the day's notes.
  it("shows the odds of a pick and opens the day's notes", async () => {
    vi.mocked(api.worldView).mockResolvedValue(diary());
    const onOpenNotes = vi.fn();
    view(onOpenNotes);
    const list = await screen.findByRole("region", { name: "The diary" });
    const why = within(list).getAllByText("Why this one");
    await userEvent.click(why[0]);
    const table = within(list).getAllByRole("table")[0];
    expect(within(table).getByRole("row", { name: /Running shoes \(drawn\) 31%/ })).toBeInTheDocument();
    expect(within(table).getByRole("row", { name: /Water bottle 20%/ })).toBeInTheDocument();
    expect(within(table).getByRole("row", { name: /Everything else 48%/ })).toBeInTheDocument();
    await userEvent.click(why[1]);
    expect(within(list).getByText("It came from a related subject.")).toBeInTheDocument();
    await userEvent.click(within(list).getByRole("button", { name: "Notes from that day" }));
    expect(onOpenNotes).toHaveBeenCalledWith("2026-11-15");
  });

  // journey FR-041, FR-043: forgetting a story asks first, and offers a fresh context when it may still be seen.
  it("forgets a story after confirming and offers a fresh context", async () => {
    vi.mocked(api.worldView).mockResolvedValue(diary());
    vi.mocked(api.forgetTopic).mockResolvedValue({ inCurrentContext: true });
    view();
    const list = await screen.findByRole("region", { name: "The diary" });
    await userEvent.click(within(list).getAllByRole("button", { name: "Forget this story" })[0]);
    const dialog = screen.getByRole("alertdialog");
    expect(dialog).toHaveTextContent("“10 km run”");
    expect(dialog).toHaveTextContent("“You ran ten kilometres before work.”");
    await userEvent.click(within(dialog).getByRole("button", { name: "Forget" }));
    expect(api.forgetTopic).toHaveBeenCalledWith("2026-11-15", 0, "h1");
    expect(await screen.findByText("The story was forgotten. The drawing stays.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start a fresh context" })).toBeInTheDocument();
  });

  // journey FR-031: what the days are made of, as text and a bar.
  it("shows the makeup as text", async () => {
    vi.mocked(api.worldView).mockResolvedValue(diary());
    view();
    const makeup = await screen.findByRole("region", { name: "What your days are made of" });
    expect(within(makeup).getByText("A run 50%")).toBeInTheDocument();
    expect(within(makeup).getByText("A celebration 50%")).toBeInTheDocument();
  });

  // journey FR-030: the world as it was on a past day, by keyboard, and played forward.
  it("shows the world as it was and plays it forward", async () => {
    vi.mocked(api.worldView).mockResolvedValue(diary());
    view();
    const slider = await screen.findByRole("slider", { name: /Show the world as it was on/ });
    const drawnCount = () => document.querySelectorAll(".world-picture use").length;
    expect(drawnCount()).toBe(5);
    fireEvent.change(slider, { target: { value: "0" } });
    expect(drawnCount()).toBe(2);
    expect(slider).toHaveAttribute("aria-valuetext", expect.stringMatching(/14/));
    vi.useFakeTimers();
    try {
      await act(async () => {
        screen.getByRole("button", { name: "Play" }).click();
      });
      await act(async () => {
        vi.advanceTimersByTime(2_000);
      });
      expect(drawnCount()).toBe(5);
      expect(screen.getByRole("button", { name: "Play" })).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });
});

describe("WorldView looking around", () => {
  const grown = () =>
    world([
      item("home.cottage", 0, 0, "2026-11-14"),
      item("flower.daisy", 1, 0, "2026-11-14"),
      item("pebble.round", -2, 1, "2026-11-15"),
      item("thing.unknown", 3, -3, "2026-11-16", { known: false }),
    ]);
  const viewBox = () =>
    (document.querySelector(".world-picture")?.getAttribute("viewBox") ?? "").split(" ").map(Number);

  // world FR-027: opens on the whole world; keys zoom, pan and fit again.
  it("opens fitted to every item and zooms and pans by keyboard", async () => {
    vi.mocked(api.worldView).mockResolvedValue(grown());
    view();
    await screen.findByText(/^I've drawn 3 things/);
    const fitted = viewBox();
    expect(fitted).toEqual(Object.values(fitAll(grown().items)));
    const map = screen.getByRole("application");
    map.focus();
    await userEvent.keyboard("+");
    expect(viewBox()[2]).toBeLessThan(fitted[2]);
    await userEvent.keyboard("{ArrowRight}");
    const panned = viewBox();
    await userEvent.keyboard("0");
    expect(viewBox()).toEqual(fitted);
    expect(panned[0]).not.toEqual(fitted[0]);
    await userEvent.click(screen.getByRole("button", { name: "Zoom out" }));
    expect(viewBox()[2]).toBeGreaterThan(fitted[2]);
    await userEvent.click(screen.getByRole("button", { name: "Whole world" }));
    expect(viewBox()).toEqual(fitted);
  });

  // world FR-031, FR-032: the list holds everything, by day, newest first; choosing an entry selects it with an outline.
  it("lists every item by day and selects one from the list", async () => {
    vi.mocked(api.worldView).mockResolvedValue(grown());
    view();
    const list = await screen.findByRole("region", { name: "The diary" });
    const days = within(list)
      .getAllByRole("heading", { level: 3 })
      .map((h) => h.textContent);
    expect(days).toHaveLength(3);
    expect(days[0]).toMatch(/16/);
    expect(within(list).getAllByRole("button")).toHaveLength(4);
    await userEvent.click(within(list).getByRole("button", { name: "Unknown item (thing.unknown)" }));
    expect(screen.getAllByRole("status").at(-1)).toHaveTextContent(/^Unknown item \(thing.unknown\), drawn/);
    expect(document.querySelector(".world-selection")).not.toBeNull();
    expect(screen.getByRole("button", { name: /^Unknown item \(thing.unknown\), drawn/ })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });
});
