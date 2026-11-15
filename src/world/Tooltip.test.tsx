import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { DrawnItem } from "../bindings/DrawnItem";
import { t } from "../strings";
import { WorldPicture } from "./WorldPicture";

const item = (id: string, x: number, day: string, keywords: string[]): DrawnItem => ({
  item: id,
  known: true,
  x,
  y: 0,
  size: 1,
  flip: false,
  shade: 0,
  day,
  topic: null,
  keywords,
});

const items = [
  item("shoes.running", 1, "2026-11-15", ["10 km run", "sister's birthday"]),
  item("tree.oak", 3, "2026-11-01", []),
];

const picture = () =>
  render(
    <WorldPicture
      items={items}
      palette={{
        ground: "#dfe8c8",
        water: "#8cc3d8",
        leaf: "#6aa84f",
        bark: "#8b5e3c",
        stone: "#a7a39a",
        accent: "#e8a33d",
      }}
      label="Steve's world"
      centre={null}
      selected={null}
      until={null}
      onSelect={vi.fn()}
      itemLabel={(i) => `${t.world.name(i.item)}, drawn ${i.day}`}
      itemTip={(i) => ({ name: t.world.name(i.item), date: i.day })}
      controls={t.world.controls}
    />,
  );

describe("world tooltips", () => {
  // journey FR-024, FR-025: name, date and the day's keywords, on hover, focus and tap.
  it("opens on hover with the day's keywords and stays while the pointer is on it", async () => {
    picture();
    const shoes = screen.getByRole("button", { name: /^Running shoes/ });
    fireEvent.pointerEnter(shoes, { pointerType: "mouse" });
    const tip = screen.getByRole("tooltip");
    expect(tip).toHaveTextContent("Running shoes2026-11-1510 km run · sister's birthday");
    fireEvent.pointerLeave(shoes, { pointerType: "mouse", relatedTarget: tip });
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
    fireEvent.pointerLeave(tip, { pointerType: "mouse", relatedTarget: document.body });
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  // jsdom moves no focus onto SVG elements, so the focus events are sent directly.
  it("opens on keyboard focus as the item's description and closes on Escape and blur", () => {
    picture();
    const oak = screen.getByRole("button", { name: /^Oak tree/ });
    fireEvent.focus(oak);
    expect(screen.getByRole("tooltip")).toHaveTextContent("Oak tree2026-11-01");
    expect(oak).toHaveAttribute("aria-describedby", screen.getByRole("tooltip").id);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByRole("tooltip")).toBeNull();
    fireEvent.focus(oak);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
    fireEvent.blur(oak);
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("opens and closes on tap", () => {
    picture();
    const shoes = screen.getByRole("button", { name: /^Running shoes/ });
    fireEvent.pointerDown(shoes, { pointerType: "touch" });
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
    fireEvent.pointerDown(shoes, { pointerType: "touch" });
    expect(screen.queryByRole("tooltip")).toBeNull();
  });
});
