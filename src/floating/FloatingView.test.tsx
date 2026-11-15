import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { FloatingView } from "./FloatingView";

const startDragging = vi.fn(async () => {});
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ startDragging }) }));
vi.mock("../api/commands", async (original) => ({ ...(await original<typeof api>()), openChat: vi.fn() }));

beforeEach(() => {
  startDragging.mockClear();
  vi.mocked(api.openChat).mockClear();
});

describe("FloatingView", () => {
  it("drags once the pointer moves 4 pixels, and opens the chat on a click or a key", async () => {
    render(<FloatingView name="Steve" />);
    const view = screen.getByRole("button");
    fireEvent.pointerDown(view, { clientX: 10, clientY: 10 });
    fireEvent.pointerMove(view, { clientX: 12, clientY: 11 });
    fireEvent.pointerMove(view, { clientX: 14, clientY: 10 });
    fireEvent.pointerUp(view);
    expect(startDragging).toHaveBeenCalledTimes(1);
    expect(api.openChat).not.toHaveBeenCalled();

    fireEvent.pointerDown(view, { clientX: 10, clientY: 10 });
    fireEvent.pointerUp(view);
    view.focus();
    await userEvent.keyboard("{Enter} ");
    expect(api.openChat).toHaveBeenCalledTimes(3);
  });

  it("shows only the character, with its label", async () => {
    render(<FloatingView name="Steve" />);
    expect(screen.getByRole("button")).toContainElement(await screen.findByRole("img", { name: "Steve" }));
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });
});
