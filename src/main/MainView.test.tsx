import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { ProviderError } from "../bindings/ProviderError";
import { chatIdle, completeStatus } from "../testing";
import { MainView } from "./MainView";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  modelList: vi.fn(),
  chatState: vi.fn(async () => chatIdle()),
  chatHistory: vi.fn(async () => ({ messages: [], hasMore: false })),
  prepareWorld: vi.fn(async () => ({ recreated: [] })),
  extensions: vi.fn(async () => []),
  autostart: vi.fn(async () => ({ on: false, offered: true })),
  alarms: vi.fn(async () => ({ alarms: [], unreadable: 0, notificationsHeld: false })),
  setAutostart: vi.fn(async () => ({ on: true, offered: true })),
  weatherNow: vi.fn(async () => null),
}));

const listWith = (problem: ProviderError | null) => ({
  models: [],
  selected: "claude-opus-5-5",
  stale: problem !== null,
  problem,
  fallbackFrom: null,
});

// onboarding FR-032, FR-033, FR-036: each degraded state says what is wrong and offers the way out.
// PER-5, PER-6: the presence line and the companion's line say which problem it is.
const cases: [string, boolean, ProviderError | null, RegExp, string, string, "settings" | "retry"][] = [
  ["no key", false, null, /until you add an API key/, "Steve is waiting for a key", "Add a key", "settings"],
  ["unreachable", true, "Unreachable", /can't reach Claude/, "Steve is offline", "Retry", "retry"],
  [
    "key rejected",
    true,
    "KeyRejected",
    /no longer accepts/,
    "Steve can't reply",
    "Enter another key",
    "settings",
  ],
  ["no credit", true, "NoCredit", /no credit/, "Steve can't reply", "Enter another key", "settings"],
];

describe("MainView", () => {
  it("greets the user by name and shows the conversation area", async () => {
    vi.mocked(api.modelList).mockResolvedValue(listWith(null));
    render(<MainView status={completeStatus({ callMe: "Sam" })} onOpenSettings={vi.fn()} />);
    expect(
      screen.getByRole("heading", { name: /^Good (morning|afternoon|evening), Sam$/ }),
    ).toBeInTheDocument();
    expect(screen.getByText("Steve is here")).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Conversation" })).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: "Settings" })).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it.each(cases)("%s", async (_, hasKey, problem, message, presence, action, leadsTo) => {
    vi.mocked(api.modelList).mockReset().mockResolvedValue(listWith(problem));
    const onOpenSettings = vi.fn();
    const key = hasKey ? ({ status: "Present", lastFour: "WXYZ" } as const) : ({ status: "Absent" } as const);
    render(<MainView status={completeStatus({ key })} onOpenSettings={onOpenSettings} />);

    expect(await screen.findByRole("alert")).toHaveTextContent(message);
    expect(screen.getByText(presence)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: action }));
    if (leadsTo === "settings") expect(onOpenSettings).toHaveBeenCalled();
    else expect(api.modelList).toHaveBeenCalledTimes(2);
  });

  it("says which files were recreated", () => {
    vi.mocked(api.modelList).mockResolvedValue(listWith(null));
    render(<MainView status={completeStatus({ recreatedFiles: ["MEMORY.md"] })} onOpenSettings={vi.fn()} />);
    expect(screen.getByRole("status")).toHaveTextContent("MEMORY.md");
  });
});

describe("MainView world", () => {
  // An older companion gets its world when the main view opens; the World button opens it.
  it("prepares the world once, reports a recreated world file, and opens the world", async () => {
    vi.mocked(api.modelList).mockResolvedValue(listWith(null));
    vi.mocked(api.prepareWorld).mockResolvedValue({ recreated: ["world/world.jsonl"] });
    const onOpenWorld = vi.fn();
    render(<MainView status={completeStatus()} onOpenSettings={vi.fn()} onOpenWorld={onOpenWorld} />);
    expect(await screen.findByText(/recreated them from defaults: world\/world.jsonl/)).toBeInTheDocument();
    expect(api.prepareWorld).toHaveBeenCalledTimes(1);
    await userEvent.click(screen.getByRole("button", { name: "World" }));
    expect(onOpenWorld).toHaveBeenCalled();
  });
});

// dreaming FR-025: a calm status with the companion asleep while it dreams; chat stays usable.
describe("MainView while dreaming", () => {
  it("says the companion is dreaming, shows it asleep and keeps the composer", async () => {
    vi.mocked(api.modelList).mockResolvedValue(listWith(null));
    const onOpenDreams = vi.fn();
    render(
      <MainView status={completeStatus()} onOpenSettings={vi.fn()} onOpenDreams={onOpenDreams} dreaming />,
    );
    const status = await screen.findByText("Steve is dreaming…");
    expect(status.closest(".line")).toHaveClass("asleep");
    expect(screen.getByLabelText("Message Steve")).toBeEnabled();
    await userEvent.click(screen.getByRole("button", { name: "Dreams" }));
    expect(onOpenDreams).toHaveBeenCalled();
  });
});

// extensions FR-004, FR-027: the extensions that are on have buttons; the first alarm brings the launch-at-login
// question, once.
describe("MainView with extensions", () => {
  const on = (id: string, name: string) => ({ id, name, about: "", folder: "", on: true });

  it("shows the buttons of the extensions that are on", async () => {
    vi.mocked(api.modelList).mockResolvedValue(listWith(null));
    vi.mocked(api.extensions).mockResolvedValueOnce([on("alarms", "Alarms")]);
    const onOpenAlarms = vi.fn();
    render(<MainView status={completeStatus()} onOpenSettings={vi.fn()} onOpenAlarms={onOpenAlarms} />);
    await userEvent.click(await screen.findByRole("button", { name: "Alarms" }));
    expect(onOpenAlarms).toHaveBeenCalled();
    expect(screen.queryByRole("button", { name: "Sticky notes" })).not.toBeInTheDocument();
  });

  it("asks once about launch at login after the first alarm", async () => {
    vi.mocked(api.modelList).mockResolvedValue(listWith(null));
    vi.mocked(api.autostart).mockResolvedValueOnce({ on: false, offered: false });
    vi.mocked(api.alarms).mockResolvedValueOnce({
      alarms: [
        { key: "k", label: "Tea", time: "07:30", repeat: { kind: "daily" }, state: "waiting", next: null },
      ],
      unreadable: 0,
      notificationsHeld: false,
    });
    render(<MainView status={completeStatus()} onOpenSettings={vi.fn()} />);
    expect(await screen.findByText(/Alarms only ring while My Pec is running/)).toHaveTextContent(
      "It also lets Steve dream overnight.",
    );
    await userEvent.click(screen.getByRole("button", { name: "Yes, start at sign-in" }));
    expect(api.setAutostart).toHaveBeenCalledWith(true);
    expect(screen.queryByText(/Alarms only ring/)).not.toBeInTheDocument();
  });
});

// weather FR-016: the weather in words beside the character, as a polite live region.
describe("MainView with the weather", () => {
  it("says the weather under the character, and nothing when there is none", async () => {
    vi.mocked(api.modelList).mockResolvedValue(listWith(null));
    vi.mocked(api.weatherNow).mockResolvedValueOnce({
      place: "Jakarta, Indonesia",
      condition: "heavyRain",
      temperature: 26,
      units: "celsius",
      day: true,
    });
    const { container } = render(<MainView status={completeStatus()} onOpenSettings={vi.fn()} />);
    expect(await screen.findByText("Heavy rain · 26°")).toHaveAttribute("aria-live", "polite");
    // The main window fills the space above the character with the sky, not the small layer around it.
    expect(container.querySelector(".weather-sky")).not.toBeNull();
    expect(container.querySelector(".weather-layer")).toBeNull();
  });
});
