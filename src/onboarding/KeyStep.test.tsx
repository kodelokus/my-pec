import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { KeyCheckOutcome } from "../bindings/KeyCheckOutcome";
import { KeyStep } from "./KeyStep";

vi.mock("../api/commands", async (original) => ({ ...(await original<typeof api>()), submitKey: vi.fn() }));

const KEY = "sk-ant-api03-0123456789abcdefWXYZ";

// onboarding FR-012: each outcome says what happened and offers its own way forward; none of them is accepted.
const cases: [KeyCheckOutcome["outcome"], RegExp, string, "resubmits" | "edits"][] = [
  ["Incomplete", /complete key/, "Fix the key", "edits"],
  ["Rejected", /didn't accept/, "Enter another key", "edits"],
  ["NoCredit", /genuine.*no credit/, "Try again", "resubmits"],
  ["Unreachable", /couldn't reach Claude.*hasn't been checked/, "Retry", "resubmits"],
  ["StorageUnavailable", /Credential Manager.*won't keep it anywhere else/, "Try again", "resubmits"],
];

describe("KeyStep", () => {
  it.each(cases)("%s gets its own message and recovery", async (outcome, message, action, recovery) => {
    vi.mocked(api.submitKey)
      .mockReset()
      .mockResolvedValue({ outcome } as KeyCheckOutcome);
    const onAccepted = vi.fn();
    render(<KeyStep lastFour={null} face="pip" onAccepted={onAccepted} />);

    await userEvent.type(screen.getByLabelText("Claude API key"), KEY);
    await userEvent.click(screen.getByRole("button", { name: "Check key" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(message);
    // avatar FR-013: the character is concerned while the problem is told.
    expect(document.querySelector(".avatar")).toHaveAttribute("data-mood", "concerned");
    expect(onAccepted).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: action }));
    if (recovery === "resubmits") {
      expect(api.submitKey).toHaveBeenCalledTimes(2);
      expect(api.submitKey).toHaveBeenLastCalledWith(KEY);
    } else {
      expect(screen.getByLabelText("Claude API key")).toHaveFocus();
    }
  });

  it("forgets the key once it is accepted and shows only its last four characters", async () => {
    vi.mocked(api.submitKey).mockReset().mockResolvedValue({ outcome: "Accepted", lastFour: "WXYZ" });
    const onAccepted = vi.fn();
    const { container, rerender } = render(<KeyStep lastFour={null} face="pip" onAccepted={onAccepted} />);

    await userEvent.type(screen.getByLabelText("Claude API key"), `  ${KEY}  `);
    await userEvent.click(screen.getByRole("button", { name: "Check key" }));
    expect(onAccepted).toHaveBeenCalledWith("WXYZ");
    rerender(<KeyStep lastFour="WXYZ" face="pip" onAccepted={onAccepted} />);

    expect(screen.getByText("Key accepted (ends in WXYZ).")).toBeInTheDocument();
    expect(container.querySelector(".avatar")).toHaveAttribute("data-mood", "happy");
    expect(container.innerHTML).not.toContain("0123456789abcdef");
    await userEvent.click(screen.getByRole("button", { name: "Use a different key" }));
    expect(screen.getByLabelText("Claude API key")).toHaveValue("");
  });

  it("shows where a key can be created as copyable text", () => {
    render(<KeyStep lastFour={null} face="pip" onAccepted={vi.fn()} />);
    expect(screen.getByText("https://platform.claude.com/settings/keys")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy" })).toBeInTheDocument();
  });
});
