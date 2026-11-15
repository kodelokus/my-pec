import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import type * as api from "../api/commands";
import { draft, options } from "../testing";
import { SoulStep } from "./SoulStep";
import { Wizard } from "./Wizard";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  setupOptions: vi.fn(async () => ({ ...options, soulBudgetChars: 20 })),
  saveSetupDraft: vi.fn(async () => {}),
}));

function Harness() {
  const [soul, setSoul] = useState(draft().soul);
  return <SoulStep soul={soul} options={options} face="pip" notice={null} onChange={setSoul} />;
}

describe("SoulStep", () => {
  it("asks before a template switch discards edits", async () => {
    render(<Harness />);
    const text = screen.getByLabelText("Personality text");

    await userEvent.click(screen.getByRole("radio", { name: /Calm mentor/ }));
    expect(text).toHaveValue("# Soul\n\nBe calm.");
    // PER-9: the companion quotes the template's fixed sample line at once.
    expect(screen.getAllByRole("status")[0]).toHaveTextContent(
      /As a calm mentor I'd sound like this: “Hello. I'm Steve\./,
    );
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();

    await userEvent.type(text, " Mine.");
    await userEvent.click(screen.getByRole("radio", { name: /Warm friend/ }));
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(text).toHaveValue("# Soul\n\nBe calm. Mine.");

    await userEvent.click(screen.getByRole("radio", { name: /Warm friend/ }));
    await userEvent.click(screen.getByRole("button", { name: "Switch and discard edits" }));
    expect(text).toHaveValue("# Soul\n\nBe warm.");
  });

  it("warns when the text is over budget and still allows Next", async () => {
    render(<Wizard draft={draft({ step: "Soul" })} lastFour="WXYZ" existing={null} onDone={vi.fn()} />);
    const text = await screen.findByLabelText("Personality text");

    await userEvent.type(text, " This is now well over twenty characters.");

    expect(screen.getByRole("alert")).toHaveTextContent(/more than I can use.*won't be used in conversation/);
    expect(screen.getByRole("button", { name: "Next" })).toBeEnabled();
  });

  it("shows the default name when the name is cleared", async () => {
    render(<Harness />);
    const name = screen.getByLabelText("Name");
    await userEvent.clear(name);
    expect(name).toHaveAttribute("placeholder", "Pec");
    expect(screen.getByText("Leave empty to use Pec.")).toBeInTheDocument();
  });
});
