import { act, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import type { ChatEvent } from "../bindings/ChatEvent";
import type { ChatMessage } from "../bindings/ChatMessage";
import { chatIdle } from "../testing";
import { Conversation } from "./Conversation";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  chatState: vi.fn(),
  chatHistory: vi.fn(),
  compactConversation: vi.fn(),
  sendMessage: vi.fn(),
  retryReply: vi.fn(),
  regenerateReply: vi.fn(),
  startFirstHello: vi.fn(),
  startContextCarrying: vi.fn(),
  startFreshContext: vi.fn(),
  stopReply: vi.fn(),
}));

const message = (id: string, kind: ChatMessage["kind"], text: string): ChatMessage => ({
  id,
  kind,
  text,
  state: kind === "companion" ? "complete" : null,
  at: Number(id),
  memory: [],
});

/** Captures the event callback of the reply a command starts, so a test can play its events. */
function replying(mock: typeof api.sendMessage | typeof api.retryReply) {
  const events: { send?: (e: ChatEvent) => void } = {};
  vi.mocked(mock).mockImplementation(async (...args: unknown[]) => {
    events.send = args[args.length - 1] as (e: ChatEvent) => void;
    return null;
  });
  return (event: ChatEvent) => act(() => events.send?.(event));
}

function Harness(props: {
  onOpenSettings?: () => void;
  onOpenMemory?: (path: string, line: number | null) => void;
}) {
  const [draft, setDraft] = useState("");
  return (
    <Conversation
      name="Steve"
      characterId="pip"
      hasKey
      draft={draft}
      onDraft={setDraft}
      onOpenSettings={props.onOpenSettings ?? vi.fn()}
      onOpenMemory={props.onOpenMemory}
    />
  );
}

beforeEach(() => {
  vi.mocked(api.chatState).mockResolvedValue(chatIdle());
  vi.mocked(api.chatHistory).mockResolvedValue({ messages: [], hasMore: false });
});

describe("Conversation", () => {
  it("shows the companion thinking, then streams the reply and announces it once", async () => {
    const play = replying(api.sendMessage);
    render(<Harness />);
    await userEvent.type(screen.getByLabelText("Message Steve"), "Hi{Enter}");
    expect(api.sendMessage).toHaveBeenCalledWith("Hi", expect.any(Function));

    await play({ kind: "Started", userMessage: message("1", "user", "Hi") });
    expect(screen.getByText("Steve is thinking…")).toBeInTheDocument();
    await play({ kind: "Delta", text: "Hello " });
    await play({ kind: "Delta", text: "Sam" });
    expect(screen.getByText("Hello Sam")).toBeInTheDocument();
    expect(screen.getByLabelText("Message Steve")).toHaveValue("");

    vi.mocked(api.chatHistory).mockResolvedValue({
      messages: [message("1", "user", "Hi"), message("2", "companion", "Hello Sam")],
      hasMore: false,
    });
    await play({ kind: "Finished", message: message("2", "companion", "Hello Sam"), state: chatIdle() });
    expect(await screen.findByText("Hello Sam", { selector: "p" })).toBeInTheDocument();
    expect(document.querySelector("[aria-live]")).toHaveTextContent("Hello Sam");
  });

  it("empties the box and shows the message while the reply is still being written", async () => {
    const events: { send?: (e: ChatEvent) => void; done?: (r: null) => void } = {};
    vi.mocked(api.sendMessage).mockImplementation(
      (_text, onEvent) =>
        new Promise((resolve) => {
          events.send = onEvent;
          events.done = resolve;
        }),
    );
    render(<Harness />);
    await userEvent.type(screen.getByLabelText("Message Steve"), "Hi{Enter}");
    expect(screen.getByLabelText("Message Steve")).toHaveValue("");

    act(() => events.send?.({ kind: "Started", userMessage: message("1", "user", "Hi") }));
    expect(screen.getByText("Hi")).toBeInTheDocument();
    await act(async () => events.done?.(null));
  });

  it("puts the text back when the send is refused", async () => {
    vi.mocked(api.sendMessage).mockResolvedValue("Busy");
    render(<Harness />);
    await userEvent.type(screen.getByLabelText("Message Steve"), "Hi{Enter}");
    expect(await screen.findByLabelText("Message Steve")).toHaveValue("Hi");
  });

  it("does not send whitespace and keeps what was typed", async () => {
    render(<Harness />);
    await userEvent.type(screen.getByLabelText("Message Steve"), "   {Enter}");
    expect(api.sendMessage).not.toHaveBeenCalled();
    expect(screen.getByLabelText("Message Steve")).toHaveValue("   ");
    expect(screen.getByText("Type a message first.")).toBeInTheDocument();
  });

  // chat FR-025: the text of a reply that is retried after a busy provider is discarded from the view.
  it("discards the streamed text while waiting to retry", async () => {
    const play = replying(api.sendMessage);
    render(<Harness />);
    await userEvent.type(screen.getByLabelText("Message Steve"), "Tell me{Enter}");
    await play({ kind: "Started", userMessage: message("1", "user", "Tell me") });
    await play({ kind: "Delta", text: "Once upon" });
    await play({ kind: "Waiting", seconds: 4, text: "" });
    expect(screen.queryByText("Once upon")).not.toBeInTheDocument();
    expect(screen.getByText("Claude is busy. Trying again in 4 s…")).toBeInTheDocument();
  });

  it("offers Retry and another key when the key is no longer accepted", async () => {
    vi.mocked(api.chatState).mockResolvedValue(
      chatIdle({ pendingRetry: { kind: "Failed", error: "KeyRejected" } }),
    );
    vi.mocked(api.chatHistory).mockResolvedValue({ messages: [message("1", "user", "Hi")], hasMore: false });
    const onOpenSettings = vi.fn();
    render(<Harness onOpenSettings={onOpenSettings} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("no longer accepts your API key");
    await userEvent.click(screen.getByRole("button", { name: "Enter another key" }));
    expect(onOpenSettings).toHaveBeenCalled();
    vi.mocked(api.retryReply).mockResolvedValue(null);
    await userEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(api.retryReply).toHaveBeenCalled();
  });

  it("offers Regenerate on the last reply only", async () => {
    vi.mocked(api.chatState).mockResolvedValue(chatIdle({ canRegenerate: true }));
    vi.mocked(api.chatHistory).mockResolvedValue({
      messages: [
        message("1", "companion", "Old"),
        message("2", "user", "Hi"),
        message("3", "companion", "New"),
      ],
      hasMore: false,
    });
    render(<Harness />);
    await screen.findByText("New");
    expect(screen.getAllByRole("button", { name: "Regenerate" })).toHaveLength(1);
    vi.mocked(api.regenerateReply).mockResolvedValue(null);
    await userEvent.click(screen.getByRole("button", { name: "Regenerate" }));
    expect(api.regenerateReply).toHaveBeenCalled();
  });

  it("loads earlier messages when scrolled to the top", async () => {
    vi.mocked(api.chatHistory)
      .mockResolvedValueOnce({ messages: [message("5", "user", "Latest")], hasMore: true })
      .mockResolvedValueOnce({ messages: [message("2", "user", "Earliest")], hasMore: false });
    render(<Harness />);
    await screen.findByText("Latest");
    fireEvent.scroll(screen.getByRole("log"), { target: { scrollTop: 0 } });
    expect(await screen.findByText("Earliest")).toBeInTheDocument();
    expect(api.chatHistory).toHaveBeenLastCalledWith("5", 50);
  });

  it("greets once when the first hello is pending", async () => {
    vi.mocked(api.chatState).mockResolvedValue(chatIdle({ firstHelloPending: true }));
    vi.mocked(api.startFirstHello).mockResolvedValue(null);
    render(<Harness />);
    await vi.waitFor(() => expect(api.startFirstHello).toHaveBeenCalledTimes(1));
  });
});

describe("Conversation memory notes", () => {
  // memory FR-011, FR-015: a full long-term memory and a shortened note each say so and open in the memory browser.
  it("tells the user about full or shortened files and opens them", async () => {
    vi.mocked(api.chatState).mockResolvedValue(
      chatIdle({ memoryFull: true, shortened: ["memory/2026-11-14.md"] }),
    );
    const onOpenMemory = vi.fn();
    render(<Harness onOpenMemory={onOpenMemory} />);
    expect(await screen.findByText(/long-term memory is full/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Open MEMORY.md in memory" }));
    expect(onOpenMemory).toHaveBeenCalledWith("MEMORY.md", null);
    await userEvent.click(screen.getByRole("button", { name: "Open memory/2026-11-14.md in memory" }));
    expect(onOpenMemory).toHaveBeenLastCalledWith("memory/2026-11-14.md", null);
  });
});

describe("Conversation compaction", () => {
  // compaction R2, FR-026: the view asks for a due compaction after a reply and shows a calm status.
  it("compacts after a reply that made it due, without blocking the composer", async () => {
    const play = replying(api.sendMessage);
    let finish: (value: Awaited<ReturnType<typeof api.compactConversation>>) => void = () => {};
    vi.mocked(api.compactConversation).mockImplementation(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    render(<Harness />);
    await userEvent.type(screen.getByLabelText("Message Steve"), "Hi{Enter}");
    await play({ kind: "Started", userMessage: message("1", "user", "Hi") });
    await play({
      kind: "Finished",
      message: message("2", "companion", "Hello"),
      state: chatIdle({ compactionDue: true }),
    });
    expect(api.compactConversation).toHaveBeenCalledTimes(1);
    expect(api.compactConversation).toHaveBeenCalledWith(null, false);
    expect(await screen.findByRole("status", { name: "" })).toHaveTextContent("Tidying up");
    const composer = screen.getByLabelText("Message Steve");
    expect(composer).toBeEnabled();
    await userEvent.type(composer, "Still there?{Enter}");
    expect(api.sendMessage).toHaveBeenLastCalledWith("Still there?", expect.any(Function));

    vi.mocked(api.chatHistory).mockResolvedValue({
      messages: [message("0", "compacted", "## Where we are\nPlanning."), message("1", "user", "Hi")],
      hasMore: false,
    });
    await act(async () => finish({ outcome: { kind: "Compacted" }, state: chatIdle() }));
    expect(screen.queryByText(/Tidying up/)).not.toBeInTheDocument();
    // compaction FR-025: a thin divider that opens to show the summary.
    const divider = await screen.findByRole("button", { name: "Earlier conversation summarized" });
    expect(divider).toHaveAttribute("aria-expanded", "false");
    await userEvent.click(divider);
    expect(screen.getByText("Planning.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Hide the summary" }));
    expect(screen.queryByText("Planning.")).not.toBeInTheDocument();
  });

  it("compacts once on load when due and shows nothing when it fails", async () => {
    vi.mocked(api.chatState).mockResolvedValue(chatIdle({ compactionDue: true }));
    vi.mocked(api.compactConversation).mockResolvedValue({
      outcome: { kind: "Failed", error: "Unreachable" },
      state: chatIdle({ compactionDue: true }),
    });
    render(<Harness />);
    await vi.waitFor(() => expect(api.compactConversation).toHaveBeenCalledTimes(1));
    await vi.waitFor(() => expect(screen.queryByText(/Tidying up/)).not.toBeInTheDocument());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  // compaction FR-022: a message that was too long compacts inline; if that fails, retry or a fresh context.
  it("shows the status while an overflowing message compacts, then offers retry or a fresh context", async () => {
    const play = replying(api.sendMessage);
    vi.mocked(api.retryReply).mockResolvedValue(null);
    vi.mocked(api.startFreshContext).mockResolvedValue(chatIdle());
    render(<Harness />);
    await userEvent.type(screen.getByLabelText("Message Steve"), "huge{Enter}");
    await play({ kind: "Started", userMessage: message("1", "user", "huge") });
    // compaction FR-024: no compact now while a reply is being written.
    expect(screen.getByRole("button", { name: "Compact now" })).toBeDisabled();
    await play({ kind: "Compacting" });
    expect(screen.getByText(/Tidying up/)).toBeInTheDocument();
    const full = chatIdle({
      contextFull: true,
      pendingRetry: { kind: "Failed", error: "ContextTooLong" },
    });
    vi.mocked(api.chatState).mockResolvedValue(full);
    await play({ kind: "Failed", error: "ContextTooLong", partial: null, state: full });
    expect(screen.queryByText(/Tidying up/)).not.toBeInTheDocument();
    expect(await screen.findByRole("alert")).toHaveTextContent("even after tidying it up");
    await userEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(api.retryReply).toHaveBeenCalled();
    await userEvent.click(await screen.findByRole("button", { name: "Start a fresh context" }));
    expect(api.startFreshContext).toHaveBeenCalled();
  });

  // Newer rules, and extensions turned on after the context began, arrive with the next context.
  it("offers to catch up, compacting, or carrying the turns over when there is nothing to compact", async () => {
    vi.mocked(api.chatState).mockResolvedValue(chatIdle({ updatePending: true }));
    vi.mocked(api.compactConversation).mockResolvedValue({
      outcome: { kind: "NothingToCompact" },
      state: chatIdle({ updatePending: true }),
    });
    vi.mocked(api.startContextCarrying).mockImplementation(async () => {
      vi.mocked(api.chatState).mockResolvedValue(chatIdle());
      return chatIdle();
    });
    render(<Harness />);
    expect(await screen.findByText(/learned a few new things/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Tidy up now" }));
    expect(api.compactConversation).toHaveBeenCalledWith(null, true);
    expect(api.startContextCarrying).toHaveBeenCalled();
    await vi.waitFor(() => expect(screen.queryByText(/learned a few new things/)).not.toBeInTheDocument());
  });
});
