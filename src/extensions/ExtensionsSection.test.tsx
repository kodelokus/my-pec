import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { ExtensionInfo } from "../bindings/ExtensionInfo";
import { ExtensionsSection } from "./ExtensionsSection";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  extensions: vi.fn(),
  setExtension: vi.fn(),
  autostart: vi.fn(),
  setAutostart: vi.fn(),
}));

const info = (id: string, name: string, on: boolean): ExtensionInfo => ({
  id,
  name,
  about: `${name} do things.`,
  folder: `C:\\data\\workspace\\${id}`,
  on,
});

beforeEach(() => {
  vi.mocked(api.extensions).mockResolvedValue([
    info("notes", "Sticky notes", true),
    info("alarms", "Alarms", true),
  ]);
  vi.mocked(api.autostart).mockResolvedValue({ on: false, offered: false });
});

// extensions FR-004, FR-026.
describe("ExtensionsSection", () => {
  it("lists each extension with what it does and where its files are, and turns it off", async () => {
    vi.mocked(api.setExtension).mockResolvedValue([
      info("notes", "Sticky notes", false),
      info("alarms", "Alarms", true),
    ]);
    render(<ExtensionsSection />);
    const notes = await screen.findByRole("checkbox", { name: "Sticky notes" });
    expect(notes).toBeChecked();
    expect(notes).toHaveAccessibleDescription("Sticky notes do things. Files: C:\\data\\workspace\\notes");
    await userEvent.click(notes);
    expect(api.setExtension).toHaveBeenCalledWith("notes", false);
    expect(await screen.findByRole("checkbox", { name: "Sticky notes" })).not.toBeChecked();
    expect(screen.getByRole("checkbox", { name: "Alarms" })).toBeChecked();
  });

  it("turns launch at login on", async () => {
    vi.mocked(api.setAutostart).mockResolvedValue({ on: true, offered: true });
    render(<ExtensionsSection />);
    const start = await screen.findByRole("checkbox", { name: "Start My Pec when I sign in to Windows" });
    expect(start).not.toBeChecked();
    await userEvent.click(start);
    expect(api.setAutostart).toHaveBeenCalledWith(true);
    expect(start).toBeChecked();
  });
});
