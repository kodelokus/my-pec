import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { MemoryDoc } from "../bindings/MemoryDoc";
import { chatIdle } from "../testing";
import { MemoryView } from "./MemoryView";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  memoryFiles: vi.fn(),
  memoryRead: vi.fn(),
  memorySave: vi.fn(),
  memorySearch: vi.fn(),
  memoryUndo: vi.fn(),
  openWorkspaceFolder: vi.fn(async () => {}),
  memoryForget: vi.fn(),
  startFreshContext: vi.fn(async () => chatIdle()),
}));

const doc = (path: string, text: string, patch: Partial<MemoryDoc> = {}): MemoryDoc => ({
  path,
  text,
  hash: `hash-${text.length}`,
  entries: [],
  canUndo: false,
  ...patch,
});

const memory = doc("MEMORY.md", "# Memory\n\n## Health\n- Allergic to peanuts.\n", {
  entries: [{ line: 4, text: "Allergic to peanuts.", id: "m_8f3a", at: "2026-11-14", from: "user" }],
  canUndo: true,
});

beforeEach(() => {
  vi.mocked(api.memoryFiles).mockResolvedValue([
    { path: "SOUL.md", kind: "soul", chars: 20, budget: 8000, readable: true },
    { path: "USER.md", kind: "profile", chars: 40, budget: 4000, readable: true },
    { path: "MEMORY.md", kind: "memory", chars: 45, budget: 12000, readable: true },
    { path: "memory/2026-11-14.md", kind: "daily", chars: 30, budget: null, readable: true },
  ]);
  vi.mocked(api.memoryRead).mockImplementation(async (path) =>
    path === "MEMORY.md" ? memory : doc(path, "- 09:00 Bimo hides.\n"),
  );
});

const view = () =>
  render(<MemoryView companion={null} open={{ path: "MEMORY.md", line: null }} onClose={vi.fn()} />);

describe("MemoryView", () => {
  it("lists the files and reads each one's entries", async () => {
    view();
    expect(await screen.findByText("Allergic to peanuts.")).toBeInTheDocument();
    const files = screen.getByRole("navigation", { name: "Memory files" });
    expect(files).toHaveTextContent("Soul");
    expect(files).toHaveTextContent("Your profile");
    await userEvent.click(screen.getByRole("button", { name: "Soul" }));
    expect(api.memoryRead).toHaveBeenCalledWith("SOUL.md");
    await userEvent.click(screen.getByRole("button", { name: "Open folder" }));
    expect(api.openWorkspaceFolder).toHaveBeenCalled();
  });

  // memory FR-029: a save never lands on a change made meanwhile without the user choosing.
  it("asks to reload or keep the user's version when the file changed", async () => {
    vi.mocked(api.memorySave)
      .mockResolvedValueOnce({ kind: "Conflict", text: "changed elsewhere", hash: "new" })
      .mockResolvedValueOnce({ kind: "Saved", hash: "mine" });
    view();
    await userEvent.click(await screen.findByRole("button", { name: "Edit" }));
    const editor = screen.getByRole("textbox", { name: "Long-term memory" });
    await userEvent.type(editor, "- Likes tea.");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("changed since you opened it");
    await userEvent.click(screen.getByRole("button", { name: "Keep my version" }));
    expect(api.memorySave).toHaveBeenLastCalledWith("MEMORY.md", `${memory.text}- Likes tea.`, "new");
    expect(await screen.findByRole("status")).toHaveTextContent("Saved.");
  });

  // memory FR-028: undoing over the user's own later edit is confirmed first.
  it("asks before undoing a change the user has edited since", async () => {
    vi.mocked(api.memoryUndo)
      .mockResolvedValueOnce({ kind: "Edited" })
      .mockResolvedValueOnce({ kind: "Undone" });
    view();
    await userEvent.click(await screen.findByRole("button", { name: "Undo the companion's last change" }));
    const dialog = await screen.findByRole("alertdialog");
    expect(dialog).toHaveTextContent("You've changed this file");
    await userEvent.click(
      screen.getAllByRole("button", { name: "Undo the companion's last change" }).at(-1) as HTMLElement,
    );
    expect(api.memoryUndo).toHaveBeenLastCalledWith("MEMORY.md", true);
    expect(await screen.findByRole("status")).toHaveTextContent("undone");
  });

  // memory FR-026: a hit opens its file at its line.
  it("opens a search result in its file", async () => {
    vi.mocked(api.memorySearch).mockResolvedValue([
      {
        path: "memory/2026-11-14.md",
        day: "2026-11-14",
        firstLine: 1,
        lastLine: 1,
        text: "- 09:00 Bimo hides.",
      },
    ]);
    view();
    await userEvent.type(await screen.findByRole("searchbox", { name: "Search memory" }), "Bimo{Enter}");
    await userEvent.click(await screen.findByRole("button", { name: /Bimo hides/ }));
    await waitFor(() => expect(api.memoryRead).toHaveBeenLastCalledWith("memory/2026-11-14.md"));
  });
});

describe("MemoryView forget", () => {
  // memory FR-027: forgetting asks first, showing what goes.
  it("confirms, forgets the entry, and offers a fresh context while the companion may still see it", async () => {
    vi.mocked(api.memoryForget).mockResolvedValue({ inCurrentContext: true });
    view();
    await userEvent.click(await screen.findByRole("button", { name: "Forget: Allergic to peanuts." }));
    expect(screen.getByRole("alertdialog")).toHaveTextContent(
      "“Allergic to peanuts.” is removed from memory",
    );
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(api.memoryForget).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: "Forget: Allergic to peanuts." }));
    await userEvent.click(screen.getByRole("button", { name: "Forget" }));
    expect(api.memoryForget).toHaveBeenCalledWith("MEMORY.md", { kind: "Entry", id: "m_8f3a" }, memory.hash);
    expect(await screen.findByText(/may still see this/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Start a fresh context" }));
    expect(api.startFreshContext).toHaveBeenCalled();
  });

  // dreaming FR-021: the dream diary is listed, read as text and saved, with nothing to forget or undo.
  it("lists the dream diary and opens it as text", async () => {
    vi.mocked(api.memoryFiles).mockResolvedValue([
      { path: "MEMORY.md", kind: "memory", chars: 45, budget: 12000, readable: true },
      { path: "DREAMS.md", kind: "diary", chars: 40, budget: null, readable: true },
    ]);
    vi.mocked(api.memoryRead).mockImplementation(async (path) =>
      path === "MEMORY.md" ? memory : doc(path, "# Dream diary\n\n## 2026-11-15\nA long road.\n"),
    );
    view();
    await userEvent.click(await screen.findByRole("button", { name: /Dream diary/ }));
    expect(await screen.findByText(/A long road/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^Forget/ })).not.toBeInTheDocument();
  });
});
