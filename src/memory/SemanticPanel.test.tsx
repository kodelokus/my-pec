import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { SemanticStatus } from "../bindings/SemanticStatus";
import { SemanticPanel } from "./SemanticPanel";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  semanticStatus: vi.fn(),
  startSemanticDownload: vi.fn(),
  cancelSemanticDownload: vi.fn(),
  setSemanticSearch: vi.fn(),
  removeSemanticModel: vi.fn(),
  dismissSemanticOffer: vi.fn(),
  semanticTick: vi.fn(),
}));

const status = (patch: Partial<SemanticStatus> = {}): SemanticStatus => ({
  state: { kind: "NotDownloaded" },
  offer: true,
  downloadBytes: 135_390_915,
  source: "Hugging Face (huggingface.co)",
  diskBytes: 0,
  ...patch,
});

beforeEach(() => {
  vi.mocked(api.semanticStatus).mockResolvedValue(status());
  vi.mocked(api.semanticTick).mockResolvedValue(
    status({ state: { kind: "Ready" }, offer: false, diskBytes: 135_000_000 }),
  );
});

describe("SemanticPanel", () => {
  // search FR-008, FR-009: the offer says how big, from where, and that memories stay on the computer.
  it("offers the download with its size and source, and 'Not now' hides it in the memory browser", async () => {
    vi.mocked(api.dismissSemanticOffer).mockResolvedValue(status({ offer: false }));
    render(<SemanticPanel variant="offer" />);
    const offer = await screen.findByRole("complementary", { name: "Search by meaning" });
    expect(offer).toHaveTextContent("135 MB");
    expect(offer).toHaveTextContent("huggingface.co");
    expect(offer).toHaveTextContent("your memories never leave it");
    await userEvent.click(within(offer).getByRole("button", { name: "Not now" }));
    expect(api.dismissSemanticOffer).toHaveBeenCalled();
    expect(screen.queryByRole("complementary")).toBeNull();
  });

  it("downloads with progress, can cancel, then prepares the memories", async () => {
    let finish: (s: SemanticStatus) => void = () => {};
    vi.mocked(api.startSemanticDownload).mockReturnValue(new Promise((resolve) => (finish = resolve)));
    vi.mocked(api.cancelSemanticDownload).mockResolvedValue(status({ offer: false }));
    render(<SemanticPanel variant="settings" />);
    vi.mocked(api.semanticStatus).mockResolvedValue(
      status({ state: { kind: "Downloading", done: 40_000_000, total: 135_000_000 }, offer: false }),
    );
    await userEvent.click(await screen.findByRole("button", { name: "Download" }));
    expect(api.startSemanticDownload).toHaveBeenCalled();
    expect(await screen.findByText("Downloading the model: 40 MB of 135 MB.")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("value", "40000000");
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(api.cancelSemanticDownload).toHaveBeenCalled();
    await act(async () =>
      finish(status({ state: { kind: "Preparing", done: 0, total: 10 }, offer: false, diskBytes: 1 })),
    );
    expect(api.semanticTick).toHaveBeenCalled();
    expect(await screen.findByText(/^Ready\. The model uses 135 MB/)).toBeInTheDocument();
    // Never a notification, badge or reminder (search FR-009).
    expect(document.body).not.toHaveTextContent(/remind|badge|don't forget/i);
  });

  // search US3: a problem is told plainly, with a way to try again.
  it("shows what needs attention with 'Try again'", async () => {
    vi.mocked(api.semanticStatus).mockResolvedValue(
      status({ state: { kind: "NeedsAttention", reason: "Damaged" }, offer: false }),
    );
    vi.mocked(api.startSemanticDownload).mockResolvedValue(
      status({ state: { kind: "Ready" }, offer: false, diskBytes: 1 }),
    );
    render(<SemanticPanel variant="settings" />);
    expect(await screen.findByText(/The download was damaged and has been removed/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(api.startSemanticDownload).toHaveBeenCalled();
  });

  // search US5: the switch keeps the model; removing asks first.
  it("switches off and removes the model after confirming", async () => {
    vi.mocked(api.semanticStatus).mockResolvedValue(
      status({ state: { kind: "Ready" }, offer: false, diskBytes: 135_000_000 }),
    );
    vi.mocked(api.setSemanticSearch).mockResolvedValue(
      status({ state: { kind: "Off" }, offer: false, diskBytes: 135_000_000 }),
    );
    vi.mocked(api.removeSemanticModel).mockResolvedValue(status());
    render(<SemanticPanel variant="settings" />);
    const toggle = await screen.findByRole("checkbox", { name: "Find memories by meaning" });
    expect(toggle).toBeChecked();
    await userEvent.click(toggle);
    expect(api.setSemanticSearch).toHaveBeenCalledWith(false);
    expect(await screen.findByText("Off. Memories are found by matching words.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Remove the model" }));
    const dialog = screen.getByRole("alertdialog");
    await userEvent.click(within(dialog).getByRole("button", { name: "Remove the model" }));
    expect(api.removeSemanticModel).toHaveBeenCalled();
    expect(await screen.findByRole("button", { name: "Download" })).toBeInTheDocument();
  });
});
