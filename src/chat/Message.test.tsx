import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import * as api from "../api/commands";
import { Message, parts } from "./Message";

vi.mock("../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  openLink: vi.fn(async () => {}),
}));

const reply = (text: string) =>
  render(
    <Message
      message={{ id: "1", kind: "companion", text, state: "complete", at: 0, memory: [] }}
      characterId="pip"
    />,
  );

describe("Message", () => {
  it("renders a reply's Markdown", () => {
    reply("# Plan\n\n- one\n- two\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n`code`");
    expect(screen.getByRole("heading", { name: "Plan" })).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByRole("cell", { name: "2" })).toBeInTheDocument();
    expect(screen.getByText("code").tagName).toBe("CODE");
  });

  // chat R13: model output is untrusted.
  it("shows HTML as text, never loads images, and opens links only through the core", async () => {
    const { container } = reply(
      "<b>bold</b> <script>alert(1)</script>\n\n![a cat](https://example.com/cat.png)\n\n[site](https://example.com)",
    );
    // The portrait beside the bubble is the app's own asset; nothing inside the reply loads.
    expect(container.querySelector(".bubble b, .bubble script, .bubble img")).toBeNull();
    expect(container).toHaveTextContent("<b>bold</b>");

    await userEvent.click(screen.getByRole("link", { name: "Image: a cat" }));
    expect(api.openLink).toHaveBeenCalledWith("https://example.com/cat.png");
    await userEvent.click(screen.getByRole("link", { name: "site" }));
    expect(api.openLink).toHaveBeenCalledWith("https://example.com");
  });
});

// The harness's "How you talk": a few short messages in a row, each in its own bubble.
describe("Replies", () => {
  it("shows plain paragraphs as separate bubbles, the portrait and marks once", () => {
    const { container } = render(
      <Message
        message={{
          id: "1",
          kind: "companion",
          text: "Wkwk, legend.\n\nSakit nggak?",
          state: "stopped",
          at: 0,
          memory: [{ kind: "note", path: "memory/2026-11-14.md", line: 3, excerpt: "Slipped", ok: true }],
        }}
        characterId="pip"
      />,
    );
    const bubbles = [...container.querySelectorAll(".bubble")];
    expect(bubbles.map((b) => b.querySelector("p")?.textContent)).toEqual(["Wkwk, legend.", "Sakit nggak?"]);
    expect(container.querySelectorAll("img.portrait")).toHaveLength(1);
    expect(bubbles[1]).toHaveTextContent("Stopped");
    expect(bubbles[0].querySelector(".marks, .badge")).toBeNull();
  });

  it("keeps structured or long answers in one bubble", () => {
    for (const text of ["Steps:\n\n1. Open it\n2. Save", "a\n\nb\n\nc\n\nd\n\ne", "```\nx\n\ny\n```"]) {
      expect(parts(text)).toEqual([text]);
    }
    expect(parts("One.\n\n  \n\nTwo.")).toEqual(["One.", "Two."]);
  });
});

describe("Marks", () => {
  // memory FR-008: every write shows on the reply that made it, and opens the memory browser there.
  it("shows each write and opens the browser at it", async () => {
    const onOpenMemory = vi.fn();
    render(
      <Message
        message={{
          id: "1",
          kind: "companion",
          text: "Got it.",
          state: "complete",
          at: 0,
          memory: [
            { kind: "remembered", path: "MEMORY.md", line: 4, excerpt: "Allergic to peanuts.", ok: true },
            { kind: "note", path: "memory/2026-11-14.md", line: null, excerpt: "Rina visits.", ok: false },
          ],
        }}
        characterId="pip"
        onOpenMemory={onOpenMemory}
      />,
    );
    const saved = screen.getByRole("list", { name: "Saved to memory" });
    expect(saved).toHaveTextContent("I'll remember: Allergic to peanuts.");
    expect(saved).toHaveTextContent("I couldn't save this: Rina visits.");
    // A failed write has nothing to open.
    expect(screen.getAllByRole("button")).toHaveLength(1);
    await userEvent.click(screen.getByRole("button", { name: /Allergic to peanuts/ }));
    expect(onOpenMemory).toHaveBeenCalledWith("MEMORY.md", 4);
  });
});
