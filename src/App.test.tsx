import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";
import * as api from "./api/commands";
import { chatIdle, completeStatus } from "./testing";

const handlers: Record<string, () => void> = {};

vi.mock("./api/commands", async (original) => ({
  ...(await original<typeof api>()),
  appStatus: vi.fn(),
  modelList: vi.fn(async () => ({
    models: [],
    selected: "",
    stale: true,
    problem: null,
    fallbackFrom: null,
  })),
  hideToTray: vi.fn(async () => {}),
  chatState: vi.fn(async () => chatIdle()),
  dreamTick: vi.fn(async () => ({ state: "Idle", lastRun: null })),
  chatHistory: vi.fn(async () => ({ messages: [], hasMore: false })),
  onEvent: vi.fn(async (name: string, handler: () => void) => {
    handlers[name] = handler;
    return () => {};
  }),
}));

describe("App", () => {
  it("explains the tray once when the window is first closed", async () => {
    vi.mocked(api.appStatus).mockResolvedValue(completeStatus());
    render(<App />);
    await screen.findByRole("heading", { name: /Good (morning|afternoon|evening)/ });

    act(() => handlers.close_notice());
    expect(screen.getByRole("alertdialog")).toHaveTextContent(/right here in the system tray.*Quit/);

    await userEvent.click(screen.getByRole("button", { name: "OK" }));
    expect(api.hideToTray).toHaveBeenCalled();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("opens settings from the tray", async () => {
    vi.mocked(api.appStatus).mockResolvedValue(completeStatus());
    render(<App />);
    await screen.findByRole("heading", { name: /Good (morning|afternoon|evening)/ });

    act(() => handlers.open_settings());
    expect(await screen.findByRole("heading", { name: "Settings" })).toBeInTheDocument();
  });

  // dreaming R2: the app asks about dreaming at start and every minute, never two at once.
  it("ticks the dream schedule at start and every minute, one at a time", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      vi.mocked(api.appStatus).mockResolvedValue(completeStatus());
      let finish: () => void = () => {};
      vi.mocked(api.dreamTick).mockImplementationOnce(
        () => new Promise((resolve) => (finish = () => resolve({ state: "Idle", lastRun: null }))),
      );
      render(<App />);
      await screen.findByRole("heading", { name: /Good (morning|afternoon|evening)/ });
      await vi.waitFor(() => expect(api.dreamTick).toHaveBeenCalledTimes(1));
      await act(async () => vi.advanceTimersByTime(60_000));
      expect(api.dreamTick).toHaveBeenCalledTimes(1);
      await act(async () => finish());
      await act(async () => vi.advanceTimersByTime(60_000));
      expect(api.dreamTick).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
  });
});
