import { emit } from "@tauri-apps/api/event";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { Alarm } from "../bindings/Alarm";
import type { AlarmList } from "../bindings/AlarmList";
import { AlarmsView } from "./AlarmsView";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  alarms: vi.fn(),
  saveAlarm: vi.fn(),
  deleteAlarm: vi.fn(),
  alarmAction: vi.fn(),
}));

const alarm = (patch: Partial<Alarm> = {}): Alarm => ({
  key: "3f2a91c0",
  label: "Call Mum",
  time: "19:00",
  repeat: { kind: "once", date: "2026-11-14" },
  state: "waiting",
  next: "2026-11-14 19:00",
  ...patch,
});
const list = (alarms: Alarm[], patch: Partial<AlarmList> = {}): AlarmList => ({
  alarms,
  unreadable: 0,
  notificationsHeld: false,
  ...patch,
});

beforeEach(() => {
  vi.mocked(api.alarms).mockResolvedValue(
    list([
      alarm(),
      alarm({ key: "k2", label: "Run", time: "06:45", repeat: { kind: "weekdays", days: ["Mon", "Fri"] } }),
    ]),
  );
});

const view = () => render(<AlarmsView companion={{ name: "Pec", characterId: "pip" }} onClose={vi.fn()} />);

// extensions FR-017, FR-018, FR-021, FR-025.
describe("AlarmsView", () => {
  it("lists each alarm with its repeat, state and next time", async () => {
    view();
    const call = (await screen.findByText("19:00 Call Mum")).closest("li") as HTMLElement;
    expect(call).toHaveTextContent("2026-11-14 · Waiting · Next: 2026-11-14 19:00");
    expect(screen.getByText("06:45 Run").closest("li")).toHaveTextContent("Mon Fri");
  });

  it("adds an alarm on chosen days, and shows why a time isn't accepted", async () => {
    vi.mocked(api.saveAlarm).mockRejectedValueOnce({ kind: "InvalidInput", field: "time" });
    vi.mocked(api.saveAlarm).mockResolvedValueOnce(list([]));
    view();
    await userEvent.click(await screen.findByRole("button", { name: "Add alarm" }));
    await userEvent.type(screen.getByLabelText("Remind me to"), "Stretch");
    await userEvent.selectOptions(screen.getByLabelText("Repeat"), "On chosen days");
    await userEvent.click(screen.getByRole("checkbox", { name: "Wed" }));
    await userEvent.click(screen.getByRole("button", { name: "Save alarm" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Choose a time.");

    await userEvent.type(screen.getByLabelText("Time"), "07:15");
    await userEvent.click(screen.getByRole("button", { name: "Save alarm" }));
    expect(api.saveAlarm).toHaveBeenLastCalledWith(null, "Stretch", "07:15", {
      kind: "weekdays",
      days: ["Wed"],
    });
    expect(screen.queryByRole("button", { name: "Save alarm" })).not.toBeInTheDocument();
  });

  it("snoozes, finishes, edits and deletes from the list", async () => {
    vi.mocked(api.alarmAction).mockResolvedValue(list([alarm({ state: "snoozed" })]));
    vi.mocked(api.deleteAlarm).mockResolvedValue(list([]));
    view();
    const row = (await screen.findByText("19:00 Call Mum")).closest("li") as HTMLElement;
    await userEvent.click(within(row).getByRole("button", { name: "Snooze" }));
    expect(api.alarmAction).toHaveBeenCalledWith("3f2a91c0", "snooze");
    expect(await screen.findByText(/Snoozed/)).toBeInTheDocument();
    await userEvent.click(within(row).getByRole("button", { name: "Edit" }));
    expect(screen.getByLabelText("Remind me to")).toHaveValue("Call Mum");
    expect(screen.getByLabelText("Day")).toHaveValue("2026-11-14");
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await userEvent.click(within(row).getByRole("button", { name: "Delete" }));
    expect(await screen.findByText("No alarms yet.")).toBeInTheDocument();
  });

  it("says how many lines couldn't be read and when Windows holds notifications back", async () => {
    vi.mocked(api.alarms).mockResolvedValue(list([], { unreadable: 2, notificationsHeld: true }));
    view();
    expect(await screen.findByText(/2 lines in alarms.md couldn't be read/)).toBeInTheDocument();
    expect(screen.getByText(/holding back My Pec's notifications/)).toBeInTheDocument();
  });

  it("reloads when the alarms change outside it", async () => {
    view();
    await screen.findByText("19:00 Call Mum");
    vi.mocked(api.alarms).mockResolvedValue(list([alarm({ label: "Call Dad" })]));
    await emit("ext_changed", ["alarms"]);
    expect(await screen.findByText("19:00 Call Dad")).toBeInTheDocument();
  });
});
