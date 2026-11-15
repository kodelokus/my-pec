import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { chooseLanguage, t } from "../strings";
import { LanguageSection } from "./LanguageSection";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  setLanguage: vi.fn(async (language: string) => language),
}));

afterEach(() => chooseLanguage("en"));

// NFR-I18N-1.
describe("LanguageSection", () => {
  it("saves the language chosen, each named in itself", async () => {
    render(<LanguageSection />);
    const picker = screen.getByRole("combobox", { name: "Language of the app" });
    expect(picker).toHaveValue("en");
    await userEvent.selectOptions(picker, "Bahasa Indonesia");
    expect(api.setLanguage).toHaveBeenCalledWith("id");
  });

  it("switches every string at once, and falls back to English", () => {
    chooseLanguage("id");
    expect(t.settingsTitle).toBe("Pengaturan");
    expect(document.documentElement.lang).toBe("id");
    chooseLanguage("xx");
    expect(t.settingsTitle).toBe("Settings");
  });
});
