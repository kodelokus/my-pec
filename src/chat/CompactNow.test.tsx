import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { chatIdle } from "../testing";
import { CompactNow } from "./CompactNow";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  compactConversation: vi.fn(),
}));

const result = (outcome: Awaited<ReturnType<typeof api.compactConversation>>["outcome"]) => ({
  outcome,
  state: chatIdle(),
});

beforeEach(() => vi.mocked(api.compactConversation).mockReset());

// compaction FR-024: compact on demand, with an optional focus note.
describe("CompactNow", () => {
  it("sends the trimmed focus note and closes once compacted", async () => {
    vi.mocked(api.compactConversation).mockResolvedValue(result({ kind: "Compacted" }));
    const onCompacted = vi.fn();
    render(<CompactNow onCompacted={onCompacted} onClose={vi.fn()} />);
    await userEvent.type(screen.getByLabelText(/make sure to keep/), "  the formula  ");
    await userEvent.click(screen.getByRole("button", { name: "Compact now" }));
    expect(api.compactConversation).toHaveBeenCalledWith("the formula", true);
    expect(onCompacted).toHaveBeenCalled();
  });

  it("sends no note when none is typed, and says when there is nothing to compact or it failed", async () => {
    vi.mocked(api.compactConversation).mockResolvedValueOnce(result({ kind: "NothingToCompact" }));
    render(<CompactNow onCompacted={vi.fn()} onClose={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "Compact now" }));
    expect(api.compactConversation).toHaveBeenCalledWith(null, true);
    expect(await screen.findByRole("status")).toHaveTextContent("nothing to compact yet");

    vi.mocked(api.compactConversation).mockResolvedValueOnce(
      result({ kind: "Failed", error: "Unreachable" }),
    );
    await userEvent.click(screen.getByRole("button", { name: "Compact now" }));
    expect(await screen.findByRole("status")).toHaveTextContent("couldn't compact");
  });

  it("closes on Cancel without compacting", async () => {
    const onClose = vi.fn();
    render(<CompactNow onCompacted={vi.fn()} onClose={onClose} />);
    await userEvent.keyboard("{Enter}");
    expect(onClose).toHaveBeenCalled();
    expect(api.compactConversation).not.toHaveBeenCalled();
  });
});
