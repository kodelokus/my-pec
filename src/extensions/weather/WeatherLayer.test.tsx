import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { WeatherCondition } from "../../bindings/WeatherCondition";
import { WeatherLayer } from "./WeatherLayer";

const parts = (condition: WeatherCondition, day: boolean, still = false) => {
  const { container } = render(
    <WeatherLayer
      weather={{ place: "Jakarta", condition, temperature: 26, units: "celsius", day }}
      still={still}
    />,
  );
  const layer = container.firstElementChild as HTMLElement;
  return {
    layer,
    drawn: [
      ...new Set([...layer.querySelectorAll("[data-part]")].map((p) => p.getAttribute("data-part"))),
    ].sort(),
  };
};

// weather FR-014, FR-017.
describe("WeatherLayer", () => {
  it("draws each condition by day and by night", () => {
    for (const [condition, day, drawn] of [
      ["clear", true, ["sun"]],
      ["clear", false, ["moon", "stars"]],
      ["partlyCloudy", true, ["cloud", "sun"]],
      ["partlyCloudy", false, ["cloud", "moon", "stars"]],
      ["cloudy", true, ["cloud"]],
      ["fog", true, ["haze"]],
      ["drizzle", true, ["cloud", "drops"]],
      ["rain", false, ["cloud", "drops"]],
      ["heavyRain", true, ["cloud", "drops"]],
      ["thunderstorm", true, ["cloud", "drops", "flash"]],
      ["snow", true, ["cloud", "flakes"]],
    ] as [WeatherCondition, boolean, string[]][]) {
      expect(parts(condition, day).drawn, `${condition} ${day}`).toEqual(drawn);
    }
  });

  it("is decorative, lets clicks through, and holds still when asked", () => {
    const { layer } = parts("rain", true, true);
    expect(layer).toHaveAttribute("aria-hidden", "true");
    expect(layer).toHaveClass("weather-layer");
    expect(layer).toHaveAttribute("data-still", "true");
    expect(layer.querySelectorAll("[data-part='drops'] line").length).toBeLessThanOrEqual(24);
  });
});
