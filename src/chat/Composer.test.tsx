import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import type { SendRefusal } from "../bindings/SendRefusal";
import { Composer } from "./Composer";

function Harness(props: {
  replying?: boolean;
  refusal?: SendRefusal;
  onSend?: () => void;
  onStop?: () => void;
}) {
  const [value, setValue] = useState("");
  return (
    <Composer
      name="Steve"
      value={value}
      onChange={setValue}
      onSend={props.onSend ?? vi.fn()}
      onStop={props.onStop ?? vi.fn()}
      replying={props.replying ?? false}
      refusal={props.refusal ?? null}
      hasKey
    />
  );
}

describe("Composer", () => {
  it("sends with Enter and adds a line with Shift+Enter", async () => {
    const onSend = vi.fn();
    render(<Harness onSend={onSend} />);
    const field = screen.getByLabelText("Message Steve");

    await userEvent.type(field, "one{Shift>}{Enter}{/Shift}two");
    expect(field).toHaveValue("one\ntwo");
    expect(onSend).not.toHaveBeenCalled();
    await userEvent.type(field, "{Enter}");
    expect(onSend).toHaveBeenCalledTimes(1);
  });

  it("explains a refusal without clearing the text", async () => {
    render(<Harness refusal="TooLong" />);
    const field = screen.getByLabelText("Message Steve");
    await userEvent.type(field, "kept");
    expect(field).toHaveValue("kept");
    expect(field).toHaveAccessibleDescription(/longer than 32,000 characters/);
  });

  // chat FR-022: Stop and Escape end a reply.
  it("stops a reply with the Stop button and with Escape", async () => {
    const onStop = vi.fn();
    render(<Harness replying onStop={onStop} />);
    await userEvent.click(screen.getByRole("button", { name: "Stop" }));
    await userEvent.type(screen.getByLabelText("Message Steve"), "{Escape}");
    expect(onStop).toHaveBeenCalledTimes(2);
  });

  // avatar FR-004: the character listens while the box has text and focus.
  it("tells the core when it starts and stops listening", async () => {
    const heard: unknown[] = [];
    mockIPC((cmd, args) => {
      if (cmd === "set_listening") heard.push((args as { listening: boolean }).listening);
    });
    render(<Harness />);
    const field = screen.getByLabelText("Message Steve");
    await userEvent.click(field);
    await userEvent.type(field, "hel");
    expect(heard).toEqual([true]);
    await userEvent.clear(field);
    await userEvent.type(field, "x");
    await userEvent.tab();
    expect(heard).toEqual([true, false, true, false]);
  });
});
