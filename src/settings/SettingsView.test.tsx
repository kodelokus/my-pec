import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { chatIdle } from "../testing";
import { SettingsView } from "./SettingsView";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  modelList: vi.fn(),
  selectModel: vi.fn(),
  removeKey: vi.fn(async () => ({ status: "Absent" })),
  chatState: vi.fn(async () => chatIdle({ modelChangePending: true })),
  startContextCarrying: vi.fn(async () => chatIdle()),
  setAutoRecall: vi.fn(async (enabled: boolean) => chatIdle({ autoRecall: enabled })),
  compactConversation: vi.fn(),
  compactionSettings: vi.fn(),
  setCompactionSettings: vi.fn(),
  dreamingSettings: vi.fn(),
  setDreamingSettings: vi.fn(),
}));

const dreamingDefaults = (patch = {}) => ({
  enabled: true,
  time: "03:00",
  model: null,
  modelMissing: false,
  world: true,
  ...patch,
});

const compaction = (patch = {}) => ({
  budgetTokens: 48_000,
  minBudget: 32_000,
  maxBudget: 200_000,
  budgetStep: 1_000,
  model: null,
  modelMissing: false,
  ...patch,
});

const models = [
  { id: "claude-opus-5-5", displayName: "Claude Opus 5.5", supportsEffort: true },
  { id: "claude-sonnet-5-5", displayName: "Claude Sonnet 5.5", supportsEffort: true },
];

beforeEach(() => {
  vi.mocked(api.compactionSettings).mockResolvedValue(compaction());
  vi.mocked(api.dreamingSettings).mockResolvedValue(dreamingDefaults());
  vi.mocked(api.compactConversation).mockResolvedValue({ outcome: { kind: "Compacted" }, state: chatIdle() });
  vi.mocked(api.modelList).mockResolvedValue({
    models,
    selected: "claude-opus-5-5",
    stale: false,
    problem: null,
    fallbackFrom: null,
  });
});

const present = { status: "Present", lastFour: "WXYZ" } as const;

describe("SettingsView", () => {
  it("shows the key's last four characters and asks before removing it", async () => {
    const onChanged = vi.fn();
    render(<SettingsView keyStatus={present} companion={null} onChanged={onChanged} onClose={vi.fn()} />);

    expect(screen.getByText("A key is stored, ending in WXYZ.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Remove key" }));
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(api.removeKey).not.toHaveBeenCalled();

    await userEvent.click(screen.getAllByRole("button", { name: "Remove key" })[1]);
    expect(api.removeKey).toHaveBeenCalled();
    expect(onChanged).toHaveBeenCalled();
  });

  it("chooses a model from the provider's list", async () => {
    vi.mocked(api.selectModel).mockResolvedValue({
      models,
      selected: "claude-sonnet-5-5",
      stale: true,
      problem: null,
      fallbackFrom: null,
    });
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);

    const picker = await screen.findByRole("combobox", { name: "Model for conversation" });
    expect(picker).toHaveValue("claude-opus-5-5");
    expect(screen.queryByText(/out of date/)).not.toBeInTheDocument();
    await userEvent.selectOptions(picker, "claude-sonnet-5-5");

    expect(api.selectModel).toHaveBeenCalledWith("claude-sonnet-5-5");
    expect(picker).toHaveValue("claude-sonnet-5-5");

    // compaction FR-028: switch now compacts onto the new model.
    expect(
      await screen.findByText(/takes effect when our conversation is next tidied up/),
    ).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Switch now" }));
    expect(api.compactConversation).toHaveBeenCalledWith(null, true);
    expect(api.startContextCarrying).not.toHaveBeenCalled();
    expect(screen.queryByText(/next tidied up/)).not.toBeInTheDocument();
  });

  it("notes a stale list and names a model that was replaced", async () => {
    vi.mocked(api.modelList).mockResolvedValue({
      models,
      selected: "claude-opus-5-5",
      stale: true,
      problem: "Unreachable",
      fallbackFrom: "claude-retired-4",
    });
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);

    expect(await screen.findByText("This list may be out of date.")).toBeInTheDocument();
    expect(screen.getByText(/claude-retired-4 is no longer offered, so Claude Opus 5.5/)).toBeInTheDocument();
  });

  it("offers a key field when no key is stored", () => {
    render(
      <SettingsView
        keyStatus={{ status: "Absent" }}
        companion={null}
        onChanged={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    expect(screen.getByText("No key is stored.")).toBeInTheDocument();
    expect(screen.getByLabelText("API key", { selector: "input" })).toHaveAccessibleDescription(
      /platform.claude.com/,
    );
    expect(screen.queryByRole("button", { name: "Remove key" })).not.toBeInTheDocument();
  });
});

describe("SettingsView memory", () => {
  // memory FR-020: auto-recall can be switched off and on.
  it("shows and changes the auto-recall setting", async () => {
    vi.mocked(api.chatState).mockResolvedValueOnce(chatIdle({ autoRecall: true }));
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);
    const toggle = await screen.findByRole("checkbox", { name: "Bring up related memories automatically" });
    expect(toggle).toBeChecked();
    await userEvent.click(toggle);
    expect(api.setAutoRecall).toHaveBeenCalledWith(false);
    expect(toggle).not.toBeChecked();
  });

  it("switches to a new context carrying the turns when there is nothing to compact yet", async () => {
    vi.mocked(api.compactConversation).mockResolvedValue({
      outcome: { kind: "NothingToCompact" },
      state: chatIdle({ modelChangePending: true }),
    });
    vi.mocked(api.selectModel).mockResolvedValue({
      models,
      selected: "claude-sonnet-5-5",
      stale: false,
      problem: null,
      fallbackFrom: null,
    });
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);
    await userEvent.selectOptions(
      await screen.findByRole("combobox", { name: "Model for conversation" }),
      "claude-sonnet-5-5",
    );
    await userEvent.click(await screen.findByRole("button", { name: "Switch now" }));
    expect(api.startContextCarrying).toHaveBeenCalled();
  });

  // compaction FR-003, FR-027: the working size and the model that summarizes.
  it("saves the working size and the compaction model", async () => {
    vi.mocked(api.setCompactionSettings).mockImplementation(async (budgetTokens, model) =>
      compaction({ budgetTokens, model }),
    );
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);
    const size = await screen.findByRole("spinbutton", { name: "Working size (tokens)" });
    expect(size).toHaveValue(48_000);
    expect(size).toHaveAttribute("min", "32000");
    expect(size).toHaveAttribute("max", "200000");
    expect(size).toHaveAttribute("step", "1000");
    await userEvent.clear(size);
    await userEvent.type(size, "32000");
    await userEvent.tab();
    expect(api.setCompactionSettings).toHaveBeenCalledWith(32_000, null);

    const model = screen.getByRole("combobox", { name: "Model for summarizing" });
    expect(model).toHaveValue("");
    await userEvent.selectOptions(model, "claude-sonnet-5-5");
    expect(api.setCompactionSettings).toHaveBeenLastCalledWith(32_000, "claude-sonnet-5-5");
    expect(model).toHaveValue("claude-sonnet-5-5");
  });

  it("says when the chosen compaction model is no longer offered", async () => {
    vi.mocked(api.compactionSettings).mockResolvedValue(
      compaction({ model: "claude-haiku-4-5", modelMissing: true }),
    );
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);
    expect(
      await screen.findByText(/no longer offered, so the conversation model is used/),
    ).toBeInTheDocument();
  });

  // dreaming FR-026: on or off, the dream time and the model for dreaming.
  it("shows and saves the dreaming settings", async () => {
    vi.mocked(api.setDreamingSettings).mockImplementation(async (enabled, time, model, world) =>
      dreamingDefaults({ enabled, time, model, world }),
    );
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);
    const toggle = await screen.findByRole("checkbox", { name: "Dream at night" });
    expect(toggle).toBeChecked();
    await userEvent.click(toggle);
    expect(api.setDreamingSettings).toHaveBeenLastCalledWith(false, "03:00", null, true);
    const time = screen.getByLabelText("Dream time");
    expect(time).toHaveValue("03:00");
    fireEvent.change(time, { target: { value: "14:00" } });
    expect(api.setDreamingSettings).toHaveBeenLastCalledWith(false, "14:00", null, true);
    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: "Model for dreaming" }),
      "claude-sonnet-5-5",
    );
    expect(api.setDreamingSettings).toHaveBeenLastCalledWith(false, "14:00", "claude-sonnet-5-5", true);
    // journey FR-008: drawing the days has its own switch.
    const world = screen.getByRole("checkbox", { name: "Draw my days in the world" });
    expect(world).toBeChecked();
    await userEvent.click(world);
    expect(api.setDreamingSettings).toHaveBeenLastCalledWith(false, "14:00", "claude-sonnet-5-5", false);
    expect(world).not.toBeChecked();
  });

  it("notes a withdrawn dreaming model and that recall off means fewer dreams", async () => {
    vi.mocked(api.dreamingSettings).mockResolvedValue(
      dreamingDefaults({ model: "claude-haiku-4-5", modelMissing: true }),
    );
    vi.mocked(api.chatState).mockResolvedValue(chatIdle({ autoRecall: false }));
    render(<SettingsView keyStatus={present} companion={null} onChanged={vi.fn()} onClose={vi.fn()} />);
    expect(await screen.findByText(/dreams keep less/)).toBeInTheDocument();
    expect(
      screen.getAllByText(/no longer offered, so the conversation model is used/).length,
    ).toBeGreaterThan(0);
  });
});
