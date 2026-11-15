import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { WeatherCondition } from "../../bindings/WeatherCondition";
import { WeatherSky } from "./WeatherSky";

const sky = (condition: WeatherCondition, day: boolean, still = false) => {
  const { container } = render(
    <WeatherSky
      weather={{ place: "Jakarta", condition, temperature: 23, units: "celsius", day }}
      still={still}
    />,
  );
  const root = container.firstElementChild as HTMLElement;
  return {
    root,
    drawn: [
      ...new Set([...root.querySelectorAll("[data-part]")].map((p) => p.getAttribute("data-part"))),
    ].sort(),
    clouds: root.querySelectorAll(".sky-cloud").length,
  };
};

// weather FR-014, FR-017: the weather fills the space above the character in the main window.
describe("WeatherSky", () => {
  it("draws each condition by day and by night, with what falls reaching the ground", () => {
    for (const [condition, day, drawn, clouds] of [
      ["clear", true, ["sun"], 0],
      ["clear", false, ["moon", "stars"], 0],
      ["partlyCloudy", true, ["sun"], 2],
      ["cloudy", true, [], 3],
      ["cloudy", false, ["moon"], 3],
      ["fog", true, ["haze"], 1],
      ["drizzle", true, ["drops", "ripples"], 3],
      ["rain", false, ["drops", "ripples"], 3],
      ["heavyRain", true, ["drops", "ripples"], 4],
      ["thunderstorm", true, ["drops", "flash", "ripples"], 4],
      ["snow", true, ["flakes"], 3],
    ] as [WeatherCondition, boolean, string[], number][]) {
      const shown = sky(condition, day);
      expect(shown.drawn, `${condition} ${day}`).toEqual(drawn);
      expect(shown.clouds, `${condition} ${day}`).toBe(clouds);
    }
  });

  it("is decorative, the same on every render, and holds still when asked", () => {
    const first = sky("heavyRain", true, true);
    expect(first.root).toHaveAttribute("aria-hidden", "true");
    expect(first.root).toHaveAttribute("data-still", "true");
    const again = sky("heavyRain", true, true);
    expect(again.root.innerHTML).toBe(first.root.innerHTML);
  });
});
