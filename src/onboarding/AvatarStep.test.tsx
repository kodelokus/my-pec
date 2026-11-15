import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { options } from "../testing";
import { AvatarStep } from "./AvatarStep";

describe("AvatarStep", () => {
  // avatar FR-028: every bundled character is shown as the character, with the default chosen.
  it("shows each bundled character with the default preselected", () => {
    render(<AvatarStep characterId="pip" characters={options.characters} notice={null} onChange={vi.fn()} />);
    const choices = screen.getByRole("group", { name: "Character" });
    expect(within(choices).getByRole("radio", { name: "Pip" })).toBeChecked();
    expect(choices.querySelectorAll(".avatar")).toHaveLength(options.characters.length);
  });
});
