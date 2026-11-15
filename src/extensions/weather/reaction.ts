// How the character feels the weather (weather FR-015, R7): chosen here once, shown by Pip's renderer.

import type { WeatherCondition } from "../../bindings/WeatherCondition";
import type { WeatherView } from "../../bindings/WeatherView";

export type Reaction = "squint" | "shelter" | "flinch" | "shiver";

const WET: WeatherCondition[] = ["drizzle", "rain", "heavyRain", "thunderstorm"];
/** Below this, in Celsius, the character shivers (spec Assumptions). */
const COLD = 10;

export function reactions(weather: WeatherView | null): Reaction[] {
  if (!weather) return [];
  const out: Reaction[] = [];
  if (weather.condition === "clear" && weather.day) out.push("squint");
  if (WET.includes(weather.condition)) out.push("shelter");
  if (weather.condition === "thunderstorm") out.push("flinch");
  const celsius = weather.units === "celsius" ? weather.temperature : ((weather.temperature - 32) * 5) / 9;
  if (celsius < COLD) out.push("shiver");
  return out;
}
