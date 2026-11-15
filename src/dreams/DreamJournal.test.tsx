import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { DreamRun } from "../bindings/DreamRun";
import { DreamJournal } from "./DreamJournal";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  dreamJournal: vi.fn(),
  dreamNow: vi.fn(),
  undoLastDream: vi.fn(),
}));

const run = (patch: Partial<DreamRun> = {}): DreamRun => ({
  id: 1,
  night: "2026-11-15",
  startedAt: 1,
  endedAt: 2,
  completed: true,
  added: 2,
  merged: 1,
  superseded: 0,
  fellBack: false,
  tokens: 4200,
  undone: false,
  ...patch,
});

beforeEach(() => {
  vi.mocked(api.dreamJournal).mockResolvedValue({
    diary: "# Dream diary\n\n## 2026-11-15\nI dreamt of a **long road**.\n",
    runs: [
      run(),
      run({ id: 0, night: "2026-11-14", completed: false, fellBack: true, tokens: 0, added: 1, merged: 0 }),
    ],
    canUndo: true,
  });
});

const journal = () =>
  render(<DreamJournal companion={{ name: "Steve", characterId: "pip" }} onClose={vi.fn()} />);

// dreaming FR-020 to FR-022, FR-027, FR-018.
describe("DreamJournal", () => {
  it("shows the diary as Markdown and each night with its counts", async () => {
    journal();
    expect(await screen.findByText("long road")).toContainHTML("strong");
    expect(screen.getByText(/2 kept, 1 merged, 0 replaced/)).toHaveTextContent("4,200 tokens");
    expect(screen.getByText(/not finished · kept plainly/)).toBeInTheDocument();
  });

  it("dreams on request and refreshes, asleep while it runs", async () => {
    let finish: () => void = () => {};
    vi.mocked(api.dreamNow).mockImplementation(
      () => new Promise((resolve) => (finish = () => resolve({ state: "Idle", lastRun: run() }))),
    );
    journal();
    await userEvent.click(await screen.findByRole("button", { name: "Dream now" }));
    expect(screen.getByText("Steve is dreaming…").closest(".line")).toHaveClass("asleep");
    expect(screen.getByRole("button", { name: "Dream now" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Undo last dream" })).toBeDisabled();
    finish();
    await screen.findByText(/goes over the notes/);
    expect(api.dreamJournal).toHaveBeenCalledTimes(2);
  });

  it("asks before undoing a dream whose files changed since", async () => {
    vi.mocked(api.undoLastDream)
      .mockResolvedValueOnce({ kind: "Changed", files: ["MEMORY.md"] })
      .mockResolvedValueOnce({ kind: "Undone" });
    journal();
    await userEvent.click(await screen.findByRole("button", { name: "Undo last dream" }));
    expect(api.undoLastDream).toHaveBeenLastCalledWith(false);
    expect(await screen.findByRole("alertdialog")).toHaveTextContent("MEMORY.md changed after the dream");
    await userEvent.click(screen.getByRole("button", { name: "Undo anyway" }));
    expect(api.undoLastDream).toHaveBeenLastCalledWith(true);
    expect(await screen.findByRole("status")).toHaveTextContent("The last dream was undone");
  });

  it("offers no undo when there is nothing to undo, and says when there are no dreams", async () => {
    vi.mocked(api.dreamJournal).mockResolvedValue({ diary: "", runs: [], canUndo: false });
    journal();
    expect(await screen.findByText(/No dreams yet/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Undo last dream" })).toBeDisabled();
  });
});
