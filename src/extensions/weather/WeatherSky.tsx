import type { WeatherCondition } from "../../bindings/WeatherCondition";
import type { WeatherView } from "../../bindings/WeatherView";
import "./sky.css";

/** How much falls, and how many clouds, for each condition (weather FR-014). */
const FALLING: Partial<Record<WeatherCondition, number>> = {
  drizzle: 16,
  rain: 28,
  heavyRain: 44,
  thunderstorm: 44,
  snow: 26,
};
const CLOUDS: Partial<Record<WeatherCondition, number>> = {
  partlyCloudy: 2,
  cloudy: 3,
  fog: 1,
  drizzle: 3,
  rain: 3,
  heavyRain: 4,
  thunderstorm: 4,
  snow: 3,
};

/** A fixed pseudo-random number from 0 to 1 for a pair, so a pattern is scattered but the same every render. */
const scatter = (i: number, salt: number) => {
  const v = Math.sin(i * 12.9898 + salt * 78.233) * 43758.5453;
  return v - Math.floor(v);
};

/** Where each drop, flake or star goes: a position from 0 to 100, a fraction of the cycle, and a size. */
const spread = (n: number, seed: number) =>
  Array.from({ length: n }, (_, i) => ({
    x: Math.round(scatter(i, seed) * 100),
    delay: scatter(i, seed + 1).toFixed(2),
    size: 0.7 + scatter(i, seed + 2) * 0.6,
  }));

const CLOUD_PLACES = [
  { top: 9, left: -8, width: 62 },
  { top: 19, left: 42, width: 70 },
  { top: 4, left: 50, width: 48 },
  { top: 27, left: 2, width: 58 },
];

function Cloud(props: { top: number; left: number; width: number; index: number }) {
  return (
    <svg
      className="sky-cloud"
      viewBox="0 0 120 56"
      style={{
        top: `${props.top}%`,
        left: `${props.left}%`,
        width: `${props.width}%`,
        animationDelay: `${-props.index * 4}s`,
      }}
      aria-hidden="true"
    >
      <path d="M20 54h82a18 18 0 0 0 2-36 26 26 0 0 0-49-6 20 20 0 0 0-33 14A16 16 0 0 0 20 54z" />
    </svg>
  );
}

/**
 * The weather filling the space above the character in the main window (weather FR-014, FR-017): a sky that
 * fits the condition and the time of day, with what falls reaching down to the character. Decorative, behind
 * everything, and a still picture while reduced motion is on.
 */
export function WeatherSky(props: { weather: WeatherView; still: boolean }) {
  const { condition, day } = props.weather;
  const clearSky = condition === "clear" || condition === "partlyCloudy";
  const falling = FALLING[condition] ?? 0;
  const snow = condition === "snow";
  const wet = falling > 0 && !snow;
  return (
    <div
      className="weather-sky"
      data-condition={condition}
      data-day={day}
      data-still={props.still}
      aria-hidden="true"
    >
      <div className="sky-tint" />
      {clearSky && day && (
        <div className="sky-sun" data-part="sun">
          <span className="sky-rays" />
        </div>
      )}
      {!day && (clearSky || condition === "cloudy") && (
        <>
          <svg className="sky-moon" data-part="moon" viewBox="0 0 40 40" aria-hidden="true">
            <path d="M26 3a17 17 0 1 0 11 27A14 14 0 0 1 26 3z" />
          </svg>
          {clearSky && (
            <div className="sky-stars" data-part="stars">
              {spread(16, 3).map(({ x, delay, size }, i) => (
                <i
                  // biome-ignore lint/suspicious/noArrayIndexKey: a fixed pattern; the position is the identity.
                  key={i}
                  style={{
                    left: `${x}%`,
                    top: `${(i * 23) % 46}%`,
                    animationDelay: `${Number(delay) * 3}s`,
                    transform: `scale(${size})`,
                  }}
                />
              ))}
            </div>
          )}
        </>
      )}
      {falling > 0 && (
        <div className={snow ? "sky-snow" : "sky-rain"} data-part={snow ? "flakes" : "drops"}>
          {spread(falling, snow ? 5 : 1).map(({ x, delay, size }, i) => (
            <i
              // biome-ignore lint/suspicious/noArrayIndexKey: a fixed pattern; the position is the identity.
              key={i}
              style={
                {
                  left: `${x}%`,
                  animationDelay: `${Number(delay) * (snow ? 4 : 0.9)}s`,
                  "--size": size,
                  "--at": delay,
                } as React.CSSProperties
              }
            />
          ))}
        </div>
      )}
      {/* Clouds after what falls, so the rain comes out from under them. */}
      {Array.from({ length: CLOUDS[condition] ?? 0 }, (_, i) => (
        <Cloud key={CLOUD_PLACES[i].top} index={i} {...CLOUD_PLACES[i]} />
      ))}
      {wet && (
        <div className="sky-ground" data-part="ripples">
          {spread(5, 2).map(({ x, delay }) => (
            <i key={x} style={{ left: `${10 + x * 0.8}%`, animationDelay: `${Number(delay) * 1.2}s` }} />
          ))}
        </div>
      )}
      {condition === "fog" && (
        <div className="sky-fog" data-part="haze">
          <i />
          <i />
          <i />
        </div>
      )}
      {condition === "thunderstorm" && (
        <>
          <svg className="sky-bolt" data-part="flash" viewBox="0 0 24 60" aria-hidden="true">
            <path d="M14 0L2 32h9L6 60L22 22h-9L18 0z" />
          </svg>
          <div className="sky-flash" />
        </>
      )}
    </div>
  );
}
