import { useEffect, useState } from "react";
import { onExtChanged, weatherNow } from "../../api/commands";
import type { WeatherView } from "../../bindings/WeatherView";

/** The weather to draw, read again whenever it changes (weather interfaces: `ext_changed` with "weather"). */
export function useWeather(): WeatherView | null {
  const [weather, setWeather] = useState<WeatherView | null>(null);
  useEffect(() => {
    const read = () => weatherNow().then(setWeather, () => setWeather(null));
    read();
    const off = onExtChanged((ids) => {
      if (ids.includes("weather")) read();
    });
    return () => {
      void off.then((stop) => stop());
    };
  }, []);
  return weather;
}
