import { emit } from "@tauri-apps/api/event";
import { mockWindows } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { NoteWindow, SAVE_MS } from "./NoteWindow";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  noteRead: vi.fn(),
  noteSave: vi.fn(),
  noteStyle: vi.fn(),
  hideNote: vi.fn(),
  newNote: vi.fn(),
}));

const FILE = "packing-list-k7q2mx.md";
const doc = (text: string, hash = "h1") => ({
  file: FILE,
  text,
  hash,
  colour: "yellow" as const,
  onTop: false,
});

beforeEach(() => {
  mockWindows("note-packing-list-k7q2mx");
  vi.mocked(api.noteRead).mockResolvedValue(doc("Packing list"));
  vi.mocked(api.noteSave).mockResolvedValue({ kind: "Saved", hash: "h2" });
});
afterEach(() => vi.useRealTimers());

const box = () => screen.getByRole("textbox");
const flush = () => act(async () => {});

// extensions FR-014, FR-008, extensions R9.
describe("NoteWindow", () => {
  it("saves 700 ms after the last keystroke, against the text it read", async () => {
    render(<NoteWindow file={FILE} />);
    expect(await screen.findByDisplayValue("Packing list")).toBeInTheDocument();
    vi.useFakeTimers();
    fireEvent.change(box(), { target: { value: "Packing list\n- hat" } });
    act(() => vi.advanceTimersByTime(SAVE_MS - 100));
    fireEvent.change(box(), { target: { value: "Packing list\n- hats" } });
    act(() => vi.advanceTimersByTime(SAVE_MS - 1));
    expect(api.noteSave).not.toHaveBeenCalled();
    act(() => vi.advanceTimersByTime(1));
    expect(api.noteSave).toHaveBeenCalledTimes(1);
    expect(api.noteSave).toHaveBeenCalledWith(FILE, "Packing list\n- hats", "h1");
  });

  it("keeps the person's text as a new note when the file changed underneath", async () => {
    vi.mocked(api.noteSave).mockResolvedValue({ kind: "Conflict", text: "Edited elsewhere", hash: "h9" });
    render(<NoteWindow file={FILE} />);
    await screen.findByDisplayValue("Packing list");
    vi.useFakeTimers();
    fireEvent.change(box(), { target: { value: "Mine" } });
    act(() => vi.advanceTimersByTime(SAVE_MS));
    vi.useRealTimers();
    expect(await screen.findByRole("status")).toHaveTextContent("your text was kept in a new note");
    expect(api.newNote).toHaveBeenCalledWith("Mine");
    expect(box()).toHaveValue("Edited elsewhere");
  });

  it("reads an outside change, but never over unsaved typing", async () => {
    render(<NoteWindow file={FILE} />);
    await screen.findByDisplayValue("Packing list");
    vi.mocked(api.noteRead).mockResolvedValue(doc("Changed in an editor", "h3"));
    await emit("ext_changed", ["notes"]);
    expect(await screen.findByDisplayValue("Changed in an editor")).toBeInTheDocument();

    fireEvent.change(box(), { target: { value: "Typing…" } });
    vi.mocked(api.noteRead).mockResolvedValue(doc("Changed again", "h4"));
    await emit("ext_changed", ["notes"]);
    await flush();
    expect(box()).toHaveValue("Typing…");
  });

  it("changes colour, keeps itself on top and puts itself away", async () => {
    vi.mocked(api.noteStyle).mockResolvedValue({ ...doc("Packing list"), colour: "blue", onTop: true });
    render(<NoteWindow file={FILE} />);
    await screen.findByDisplayValue("Packing list");
    fireEvent.click(screen.getByRole("button", { name: "Make it blue" }));
    expect(api.noteStyle).toHaveBeenCalledWith(FILE, "blue", null);
    expect(await screen.findByRole("button", { name: "Make it blue" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Keep on top" }));
    expect(api.noteStyle).toHaveBeenLastCalledWith(FILE, null, false);
    fireEvent.click(screen.getByRole("button", { name: "Put the note away" }));
    expect(api.hideNote).toHaveBeenCalled();
  });
});
