import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { problemFor, reportProblem, tryAgain } from "../avatar/problem";
import type { AvatarManifest } from "../bindings/AvatarManifest";
import { Appearance } from "./Appearance";

const live: AvatarManifest = {
  look: { type: "live2d", character: "pip", model: "pip/pip.model3.json", moods: {}, activities: {} },
  problem: null,
};

// The same in every build, with or without Live2D's SDK (avatar FR-018).
vi.mock("virtual:live2d", () => ({ loadLive2D: null }));

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  avatarManifest: vi.fn(async () => live),
  chooseAvatar: vi.fn(async () => live),
  setAvatarPicture: vi.fn(),
  setFloating: vi.fn(async (on: boolean) => ({ on })),
}));

beforeEach(() => {
  tryAgain();
  vi.mocked(api.chooseAvatar).mockClear();
});

describe("Appearance", () => {
  it("chooses a character or its still portrait", async () => {
    render(<Appearance name="Steve" />);
    await userEvent.click(await screen.findByRole("radio", { name: "Still portrait" }));
    expect(api.chooseAvatar).toHaveBeenLastCalledWith("pip", true);
    await userEvent.click(screen.getByRole("radio", { name: "Ozzy" }));
    expect(api.chooseAvatar).toHaveBeenLastCalledWith("ozzy", false);
  });

  it("sends a picked picture and says plainly why one is refused", async () => {
    vi.mocked(api.setAvatarPicture).mockRejectedValue({ kind: "InvalidInput", field: "pictureType" });
    render(<Appearance name="Steve" />);
    const file = new File(["not a picture"], "cat.png", { type: "image/png" });
    await userEvent.upload(await screen.findByLabelText("Your own picture…"), file);
    await waitFor(() => expect(api.setAvatarPicture).toHaveBeenCalled());
    expect(await screen.findByRole("alert")).toHaveTextContent("isn't a PNG, JPEG or WebP picture");
  });

  it("says why the character isn't shown and tries again when asked", async () => {
    render(<Appearance name="Steve" />);
    act(() => reportProblem("noGraphics", JSON.stringify(live.look)));
    expect(await screen.findByText(/graphics can't draw the character/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(screen.queryByText(/graphics can't draw/)).not.toBeInTheDocument();
    // Mounted again: this test build has no Live2D, which is now the reason.
    await waitFor(() => expect(problemFor(JSON.stringify(live.look))).toBe("notInBuild"));
    expect(screen.getByText(/isn't part of this build/)).toBeInTheDocument();
  });

  it("turns the floating character on", async () => {
    render(<Appearance name="Steve" />);
    await userEvent.click(await screen.findByRole("checkbox", { name: "Floating Steve" }));
    expect(api.setFloating).toHaveBeenCalledWith(true);
    expect(screen.getByRole("checkbox", { name: "Floating Steve" })).toBeChecked();
  });
});
