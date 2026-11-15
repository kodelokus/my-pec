import type { WeatherView } from "../../bindings/WeatherView";
import "./weather.css";

/** Drops and flakes per condition, kept few so an idle window stays cheap (weather R6, SC-006). */
const FALLING = { drizzle: 10, rain: 18, heavyRain: 24, thunderstorm: 24, snow: 16 } as const;

/** Spreads `n` items across the box, the same each time, so the picture doesn't jump between renders. */
const spread = (n: number) =>
  Array.from({ length: n }, (_, i) => ({ x: 6 + ((i * 37) % 88), delay: ((i * 0.53) % 1.6).toFixed(2) }));

function Cloud(props: { x: number; y: number; scale?: number; dark?: boolean }) {
  return (
    <g
      className={props.dark ? "weather-cloud dark" : "weather-cloud"}
      transform={`translate(${props.x} ${props.y}) scale(${props.scale ?? 1})`}
    >
      <circle cx="0" cy="6" r="7" />
      <circle cx="9" cy="2" r="9" />
      <circle cx="19" cy="6" r="7" />
      <rect x="0" y="6" width="19" height="7" rx="3.5" />
    </g>
  );
}

/**
 * The weather above and around the character (weather FR-014, FR-017): decorative, never in the way of a
 * click or a drag, and a still picture while reduced motion is on.
 */
export function WeatherLayer(props: { weather: WeatherView; still: boolean }) {
  const { condition, day } = props.weather;
  const clearSky = condition === "clear" || condition === "partlyCloudy";
  const falling = condition in FALLING ? FALLING[condition as keyof typeof FALLING] : 0;
  const dark = condition === "heavyRain" || condition === "thunderstorm";
  return (
    <div
      className="weather-layer"
      data-condition={condition}
      data-day={day}
      data-still={props.still}
      aria-hidden="true"
    >
      <svg viewBox="0 0 100 100" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
        {clearSky && day && (
          <g className="weather-sun" data-part="sun">
            <circle cx="84" cy="14" r="7" />
            <g className="weather-rays">
              {[0, 45, 90, 135, 180, 225, 270, 315].map((a) => (
                <line key={a} x1="84" y1="3" x2="84" y2="0" transform={`rotate(${a} 84 14)`} />
              ))}
            </g>
          </g>
        )}
        {clearSky && !day && (
          <>
            <path className="weather-moon" data-part="moon" d="M86 6a8 8 0 1 0 6 14a7 7 0 1 1 -6 -14z" />
            <g className="weather-stars" data-part="stars">
              <circle cx="14" cy="10" r="1" />
              <circle cx="30" cy="5" r="0.8" />
              <circle cx="66" cy="8" r="0.9" />
              <circle cx="8" cy="26" r="0.7" />
            </g>
          </>
        )}
        {condition === "partlyCloudy" && (
          <g data-part="cloud">
            <Cloud x={62} y={14} scale={0.9} />
          </g>
        )}
        {(condition === "cloudy" || falling > 0) && (
          <g data-part="cloud">
            <Cloud x={8} y={2} dark={dark} />
            <Cloud x={58} y={6} scale={1.1} dark={dark} />
          </g>
        )}
        {falling > 0 && (
          <g
            className={condition === "snow" ? "weather-flakes" : "weather-drops"}
            data-part={condition === "snow" ? "flakes" : "drops"}
          >
            {spread(falling).map(({ x, delay }) =>
              condition === "snow" ? (
                <circle key={x} cx={x} cy="16" r="1.2" style={{ animationDelay: `${delay}s` }} />
              ) : (
                <line key={x} x1={x} y1="16" x2={x - 1} y2="21" style={{ animationDelay: `${delay}s` }} />
              ),
            )}
          </g>
        )}
        {condition === "thunderstorm" && (
          <path className="weather-flash" data-part="flash" d="M52 14l-7 13h6l-4 11l10 -15h-6l4 -9z" />
        )}
        {condition === "fog" && (
          <g className="weather-haze" data-part="haze">
            <rect x="0" y="58" width="100" height="6" rx="3" />
            <rect x="10" y="72" width="80" height="5" rx="2.5" />
            <rect x="0" y="86" width="100" height="6" rx="3" />
          </g>
        )}
      </svg>
    </div>
  );
}
