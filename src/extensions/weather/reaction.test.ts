import { describe, expect, it } from "vitest";
import type { WeatherView } from "../../bindings/WeatherView";
import { reactions } from "./reaction";

const weather = (patch: Partial<WeatherView>): WeatherView => ({
  place: "Jakarta, Indonesia",
  condition: "clear",
  temperature: 26,
  units: "celsius",
  day: true,
  ...patch,
});

// weather FR-015, R7.
describe("reactions", () => {
  it("squints in the sun, shelters in the wet, flinches at thunder and shivers in the cold", () => {
    for (const [patch, felt] of [
      [{}, ["squint"]],
      [{ day: false }, []],
      [{ condition: "partlyCloudy" }, []],
      [{ condition: "drizzle" }, ["shelter"]],
      [{ condition: "heavyRain" }, ["shelter"]],
      [{ condition: "thunderstorm" }, ["shelter", "flinch"]],
      [{ condition: "snow", temperature: -2 }, ["shiver"]],
      [{ condition: "rain", temperature: 8 }, ["shelter", "shiver"]],
      [{ condition: "cloudy", temperature: 49, units: "fahrenheit" }, ["shiver"]],
      [{ condition: "cloudy", temperature: 51, units: "fahrenheit" }, []],
    ] as [Partial<WeatherView>, string[]][]) {
      expect(reactions(weather(patch)), JSON.stringify(patch)).toEqual(felt);
    }
    expect(reactions(null)).toEqual([]);
  });
});
