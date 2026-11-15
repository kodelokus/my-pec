import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { draft, options } from "../testing";
import { Wizard } from "./Wizard";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  setupOptions: vi.fn(),
  saveSetupDraft: vi.fn(),
  submitKey: vi.fn(),
  completeSetup: vi.fn(),
}));

beforeEach(() => {
  vi.mocked(api.setupOptions).mockResolvedValue(options);
  vi.mocked(api.saveSetupDraft).mockResolvedValue();
  vi.mocked(api.submitKey).mockResolvedValue({ outcome: "Accepted", lastFour: "WXYZ" });
  vi.mocked(api.completeSetup).mockResolvedValue({ name: "Steve", characterId: "pip" });
});

const next = () => userEvent.click(screen.getByRole("button", { name: "Next" }));

describe("Wizard", () => {
  it("finishes on the defaults once a key is accepted", async () => {
    const onDone = vi.fn();
    render(<Wizard draft={draft()} lastFour={null} existing={null} onDone={onDone} />);

    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
    // People paste their key; typing it a key at a time re-renders the wizard 35 times for nothing.
    await userEvent.click(screen.getByLabelText("Claude API key"));
    await userEvent.paste("sk-ant-api03-0123456789abcdefWXYZ");
    await userEvent.click(screen.getByRole("button", { name: "Check key" }));
    expect(await screen.findByText(/Key accepted/)).toHaveTextContent("WXYZ");

    await next();
    expect(screen.getByRole("group", { name: "Personality" })).toBeInTheDocument();
    await next();
    expect(screen.getByRole("radio", { name: "Pip" })).toBeChecked();
    await next();
    await userEvent.click(screen.getByRole("button", { name: "Meet Steve" }));

    expect(api.completeSetup).toHaveBeenCalledWith("New");
    expect(onDone).toHaveBeenCalled();
  });

  it("explains a workspace that cannot be written and stays usable", async () => {
    vi.mocked(api.completeSetup).mockRejectedValue({
      kind: "WorkspaceNotWritable",
      path: "C:\\data\\workspace",
    });
    const onDone = vi.fn();
    render(<Wizard draft={draft({ step: "AboutYou" })} lastFour="WXYZ" existing={null} onDone={onDone} />);

    await userEvent.click(screen.getByRole("button", { name: "Meet Steve" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("C:\\data\\workspace");
    expect(onDone).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(screen.getByRole("radio", { name: "Pip" })).toBeInTheDocument();
  });

  it("keeps every entry when going back", async () => {
    render(<Wizard draft={draft({ step: "Soul" })} lastFour="WXYZ" existing={null} onDone={vi.fn()} />);

    const name = await screen.findByLabelText("Name");
    await userEvent.clear(name);
    await userEvent.type(name, "Ada");
    await next();
    await next();
    await userEvent.type(screen.getByLabelText("What Ada calls you"), "Sam");
    await userEvent.click(screen.getByRole("button", { name: "Back" }));
    await userEvent.click(screen.getByRole("button", { name: "Back" }));

    expect(screen.getByLabelText("Name")).toHaveValue("Ada");
    await next();
    await next();
    expect(screen.getByLabelText("What Ada calls you")).toHaveValue("Sam");
    expect(api.saveSetupDraft).toHaveBeenLastCalledWith(
      expect.objectContaining({ step: "AboutYou", about: { callMe: "Sam", language: null } }),
    );
  });

  it("offers to keep an existing companion or start fresh after the key", async () => {
    const onDone = vi.fn();
    const existing = { name: "Old Friend", characterId: "pip" };
    const { unmount } = render(
      <Wizard draft={draft()} lastFour="WXYZ" existing={existing} onDone={onDone} />,
    );

    await userEvent.click(screen.getByRole("button", { name: "Keep this companion" }));
    expect(api.completeSetup).toHaveBeenCalledWith("AdoptExisting");
    expect(onDone).toHaveBeenCalled();
    unmount();

    render(<Wizard draft={draft()} lastFour="WXYZ" existing={existing} onDone={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "Start fresh" }));
    expect(screen.getByRole("alertdialog")).toHaveTextContent(/backup folder/);
    await userEvent.click(screen.getAllByRole("button", { name: "Start fresh" })[1]);
    expect(await screen.findByRole("group", { name: "Personality" })).toBeInTheDocument();
    await next();
    await next();
    await userEvent.click(screen.getByRole("button", { name: "Meet Steve" }));
    expect(api.completeSetup).toHaveBeenLastCalledWith("ReplaceExisting");
  });
});
