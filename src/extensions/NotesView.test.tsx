import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { NotesView } from "./NotesView";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  notes: vi.fn(),
  newNote: vi.fn(),
  showNote: vi.fn(),
  deleteNote: vi.fn(),
}));

beforeEach(() => {
  vi.mocked(api.showNote).mockResolvedValue();
  vi.mocked(api.deleteNote).mockResolvedValue();
  vi.mocked(api.notes).mockResolvedValue([
    { file: "packing-list-k7q2mx.md", firstLine: "Packing list", shown: true, colour: "blue" },
    { file: "note-222222.md", firstLine: "", shown: false, colour: "yellow" },
  ]);
});

const view = () => render(<NotesView companion={{ name: "Pec", characterId: "pip" }} onClose={vi.fn()} />);

// extensions FR-011, FR-015.
describe("NotesView", () => {
  it("lists every note, on the desktop or put away, and makes new ones", async () => {
    vi.mocked(api.newNote).mockResolvedValue({
      file: "note-333333.md",
      firstLine: "",
      shown: true,
      colour: "yellow",
    });
    view();
    const items = await screen.findAllByRole("listitem");
    expect(items.map((li) => li.textContent)).toEqual([
      expect.stringContaining("Packing listOn the desktop"),
      expect.stringContaining("Empty notePut away"),
    ]);
    await userEvent.click(screen.getByRole("button", { name: "New note" }));
    expect(api.newNote).toHaveBeenCalled();
  });

  it("shows and puts away a note", async () => {
    view();
    const packing = (await screen.findByText("Packing list")).closest("li") as HTMLElement;
    await userEvent.click(within(packing).getByRole("button", { name: "Put away" }));
    expect(api.showNote).toHaveBeenCalledWith("packing-list-k7q2mx.md", false);
    const empty = screen.getByText("Empty note").closest("li") as HTMLElement;
    await userEvent.click(within(empty).getByRole("button", { name: "Show" }));
    expect(api.showNote).toHaveBeenCalledWith("note-222222.md", true);
  });

  it("asks before deleting, and says where the note goes", async () => {
    view();
    const packing = (await screen.findByText("Packing list")).closest("li") as HTMLElement;
    await userEvent.click(within(packing).getByRole("button", { name: "Delete" }));
    const dialog = screen.getByRole("alertdialog");
    expect(dialog).toHaveTextContent("deleted folder, where you can get it back");
    expect(api.deleteNote).not.toHaveBeenCalled();
    await userEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    expect(api.deleteNote).toHaveBeenCalledWith("packing-list-k7q2mx.md");
  });
});
