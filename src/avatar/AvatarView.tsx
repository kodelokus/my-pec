import { loadLive2D } from "virtual:live2d";
import { useEffect, useRef, useState } from "react";
import { avatarFile, avatarManifest, avatarState, onAvatarState } from "../api/commands";
import type { AvatarLook } from "../bindings/AvatarLook";
import type { AvatarManifest } from "../bindings/AvatarManifest";
import type { AvatarSource } from "../bindings/AvatarSource";
import type { AvatarState } from "../bindings/AvatarState";
import type { WeatherView } from "../bindings/WeatherView";
import { reactions } from "../extensions/weather/reaction";
import { WeatherLayer } from "../extensions/weather/WeatherLayer";
import { t } from "../strings";
import { isAnimated, PipRenderer } from "./pip";
import { problemFor, reportProblem, useRendererProblem } from "./problem";
import { type AvatarRenderer, PortraitRenderer, type RendererProblem } from "./renderer";

/** avatar FR-014: a character that isn't ready by then is given up until the window opens again. */
const LOAD_LIMIT_MS = 5_000;
const PROBLEMS: RendererProblem[] = ["missing", "damaged", "noGraphics", "notInBuild", "timedOut", "stopped"];
const WORKSPACE: AvatarSource = { kind: "Workspace" };

/** avatar FR-021: the system's reduced-motion setting, followed while it changes. */
export function useReducedMotion() {
  const query =
    typeof window.matchMedia === "function" ? window.matchMedia("(prefers-reduced-motion: reduce)") : null;
  const [reduced, setReduced] = useState(query?.matches ?? false);
  useEffect(() => {
    if (!query) return;
    const changed = () => setReduced(query.matches);
    query.addEventListener("change", changed);
    return () => query.removeEventListener("change", changed);
  }, [query]);
  return reduced;
}

const hasWebGl = () => {
  try {
    return document.createElement("canvas").getContext("webgl") !== null;
  } catch {
    return false;
  }
};

/**
 * The companion's character wherever it is shown (avatar FR-002, PER-1): the still portrait first, swapped for the
 * animated character once it is ready, and back on any failure (avatar R4).
 */
export function AvatarView(props: {
  size: number;
  /** Names the character for screen readers (avatar FR-022); without it the character is decorative, as beside a line. */
  name?: string;
  /** A character to show instead of the companion's own look, as each choice in the wizard. */
  character?: string;
  source?: AvatarSource;
  /** avatar FR-013: an interface moment's mood, only while it is shown. */
  moment?: "happy" | "concerned";
  /** The weather drawn around the character, and felt by it; only the main and floating windows. */
  weather?: WeatherView | null;
  /** Draw the weather in the character's own box; the main window draws a whole sky instead and passes false. */
  weatherLayer?: boolean;
}) {
  const [state, setState] = useState<AvatarState>({ activity: "idle", mood: "neutral", voice: 0, look: 0 });
  const [manifest, setManifest] = useState<AvatarManifest | null>(null);
  const [live, setLive] = useState(false);
  const { attempt } = useRendererProblem();
  const reduced = useReducedMotion();
  const box = useRef<HTMLDivElement>(null);
  const img = useRef<HTMLImageElement>(null);
  const figure = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const renderer = useRef<AvatarRenderer | null>(null);
  const source = props.source ?? WORKSPACE;
  const mood = props.moment ?? state.mood;
  const weather = props.weather ?? null;
  const shown = {
    activity: state.activity,
    mood,
    voice: state.voice,
    still: reduced,
    weather: reactions(weather),
  };

  useEffect(() => {
    let open = true;
    avatarState().then(
      (s) => open && setState(s),
      () => {},
    );
    const off = onAvatarState(setState);
    return () => {
      open = false;
      void off.then((stop) => stop());
    };
  }, []);

  // avatar FR-029: a new look in any window shows here too.
  // biome-ignore lint/correctness/useExhaustiveDependencies: the revision says when to read the manifest again.
  useEffect(() => {
    avatarManifest().then(setManifest, () => {});
  }, [state.look]);

  const own = manifest?.look;
  const look: AvatarLook | null =
    own && props.character && own.character !== props.character
      ? { type: "static", character: props.character }
      : (own ?? null);
  const lookKey = JSON.stringify(look);

  // biome-ignore lint/correctness/useExhaustiveDependencies: the look is reloaded when its content or the attempt changes.
  useEffect(() => {
    if (!look || !img.current || !figure.current) return;
    const read = (path: string) => avatarFile(source, path).then((bytes) => new Blob([bytes]));
    // The animated drawing when the character has one, else its still portrait or the own picture.
    const portrait = isAnimated(look) ? new PipRenderer(figure.current) : new PortraitRenderer(img.current);
    renderer.current = portrait;
    void portrait.load(look, read);
    setLive(false);
    let animated: AvatarRenderer | null = null;
    let done = false;
    const fail = (problem: RendererProblem) => {
      if (done) return;
      done = true;
      reportProblem(problem, lookKey);
      animated?.dispose();
      renderer.current = portrait;
      setLive(false);
    };
    let timer = 0;
    if (look.type === "live2d" && problemFor(lookKey) === null) {
      const into = canvas.current;
      if (!loadLive2D) fail("notInBuild");
      else if (!into || !hasWebGl()) fail("noGraphics");
      else {
        timer = window.setTimeout(() => fail("timedOut"), LOAD_LIMIT_MS);
        loadLive2D()
          .then((make) => {
            if (done) return;
            const made = make(into, () => fail("stopped"));
            animated = made;
            return made.load(look, read);
          })
          .then(
            () => {
              window.clearTimeout(timer);
              if (done) return;
              renderer.current = animated;
              setLive(true);
            },
            (e) => {
              window.clearTimeout(timer);
              fail(PROBLEMS.includes(e) ? e : "damaged");
            },
          );
      }
    }
    return () => {
      done = true;
      window.clearTimeout(timer);
      animated?.dispose();
      portrait.dispose();
    };
  }, [lookKey, attempt]);

  useEffect(() => {
    renderer.current?.setState(shown);
  });

  // AVA-8: the eyes follow the pointer while the character is idle or listening.
  const follows = state.activity === "idle" || state.activity === "listening";
  useEffect(() => {
    if (!follows) return;
    const move = (e: PointerEvent) => {
      const rect = box.current?.getBoundingClientRect();
      if (!rect || rect.width === 0) return;
      const x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      const y = ((e.clientY - rect.top) / rect.height) * 2 - 1;
      renderer.current?.pointer(Math.max(-1, Math.min(1, x)), Math.max(-1, Math.min(1, y)));
    };
    document.addEventListener("pointermove", move);
    return () => document.removeEventListener("pointermove", move);
  }, [follows]);

  const label = props.name
    ? { role: "img", "aria-label": t.avatar.label(props.name, state.activity, mood) }
    : {};
  return (
    <div
      ref={box}
      className="avatar"
      data-mood={mood}
      data-activity={state.activity}
      aria-hidden={props.name ? undefined : true}
      style={{ "--avatar-size": `${props.size}px` } as React.CSSProperties}
      {...label}
    >
      <img
        ref={img}
        alt=""
        width={props.size}
        height={props.size}
        className="portrait"
        hidden={live || (look !== null && isAnimated(look))}
      />
      <div ref={figure} className="figure" hidden={live || look === null || !isAnimated(look)} />
      <canvas ref={canvas} width={props.size * 2} height={props.size * 2} hidden={!live} />
      {weather && props.weatherLayer !== false && <WeatherLayer weather={weather} still={reduced} />}
    </div>
  );
}
