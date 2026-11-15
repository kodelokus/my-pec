import { useCallback, useEffect, useState } from "react";
import {
  alarms,
  autostart,
  extensions,
  modelList,
  onExtChanged,
  prepareWorld,
  setAutostart,
  showNote,
} from "../api/commands";
import { AvatarView, useReducedMotion } from "../avatar/AvatarView";
import type { AppStatus } from "../bindings/AppStatus";
import type { MemoryMark } from "../bindings/MemoryMark";
import type { ProviderError } from "../bindings/ProviderError";
import { Conversation } from "../chat/Conversation";
import { useWeather } from "../extensions/weather/useWeather";
import { WeatherSky } from "../extensions/weather/WeatherSky";
import { Line } from "../Line";
import { t } from "../strings";

/** onboarding FR-024, and the degraded states onboarding FR-032, FR-033, FR-036, told by the companion (PER-5, PER-6). */
export function MainView(props: {
  status: AppStatus;
  onOpenSettings: () => void;
  /** memory FR-025: the memory browser, at a file and line when given. */
  onOpenMemory?: (path?: string, line?: number | null) => void;
  /** WLD-8: the companion's world. */
  onOpenWorld?: () => void;
  /** The dream journal. */
  onOpenDreams?: () => void;
  /** The notes list and the alarm list. */
  onOpenNotes?: () => void;
  onOpenAlarms?: () => void;
  /** A dream sweep is running (dreaming FR-025). */
  dreaming?: boolean;
  /** Unsent composer text, kept by the app across switches to settings (chat FR-028). */
  draft?: string;
  onDraft?: (text: string) => void;
}) {
  const { companion, key, callMe } = props.status;
  const [problem, setProblem] = useState<ProviderError | null>(null);
  const [worldRecreated, setWorldRecreated] = useState<string[]>([]);
  const weather = useWeather();
  const still = useReducedMotion();
  const recreatedFiles = [...props.status.recreatedFiles, ...worldRecreated];

  // An older companion gets its id and world when the main view opens, with no prompt.
  useEffect(() => {
    prepareWorld().then(
      (outcome) => setWorldRecreated(outcome.recreated),
      () => {},
    );
  }, []);
  // The extensions that are on show their buttons; the first alarm brings the launch-at-login offer.
  const [on, setOn] = useState<string[]>([]);
  const [offer, setOffer] = useState(false);
  const checkOffer = useCallback(() => {
    Promise.all([autostart(), alarms()]).then(
      ([start, list]) => setOffer(!start.offered && !start.on && list.alarms.length > 0),
      () => {},
    );
  }, []);
  useEffect(() => {
    const load = () =>
      extensions().then(
        (list) => setOn(list.filter((e) => e.on).map((e) => e.id)),
        () => {},
      );
    load();
    checkOffer();
    const off = onExtChanged((ids) => {
      load();
      if (ids.includes("alarms")) checkOffer();
    });
    return () => {
      off.then((unlisten) => unlisten());
    };
  }, [checkOffer]);
  const onMark = (mark: MemoryMark) => {
    if (mark.kind === "noteMade") void showNote(mark.path.replace("notes/", ""), true).catch(() => {});
    if (mark.kind === "alarmSet") checkOffer();
  };
  const answerOffer = (yes: boolean) => {
    setOffer(false);
    void setAutostart(yes).catch(() => {});
  };
  const hasKey = key.status === "Present";
  const name = companion?.name ?? "";
  const face = companion?.characterId ?? "";

  const refresh = useCallback(() => {
    if (!hasKey) return;
    modelList(true)
      .then((list) => setProblem(list.problem))
      .catch(() => setProblem("Unreachable"));
  }, [hasKey]);
  useEffect(refresh, [refresh]);

  const presence = !hasKey
    ? t.presence.noKey(name)
    : problem === "Unreachable"
      ? t.presence.offline(name)
      : problem
        ? t.presence.blocked(name)
        : t.presence.here(name);
  const say = (text: string, action: string, onClick: () => void) => (
    <>
      <Line small characterId={face} role="alert">
        <p>{text}</p>
      </Line>
      <div className="row under small">
        <button type="button" onClick={onClick}>
          {action}
        </button>
      </div>
    </>
  );

  return (
    <main className="content wash">
      <div className="with-avatar">
        {/* AVA-3: the character docked beside the conversation; concerned while a problem is told (avatar FR-013). */}
        <aside className="dock">
          {/* weather FR-014: the weather fills the space above the character. */}
          {weather && <WeatherSky weather={weather} still={still} />}
          <AvatarView
            size={180}
            name={name}
            moment={!hasKey || problem ? "concerned" : undefined}
            weather={weather}
            weatherLayer={false}
          />
          {/* weather FR-016: the weather in words, announced once when it changes. */}
          <p className="weather-line muted" aria-live="polite" aria-atomic="true">
            {weather ? t.weather.line(weather) : ""}
          </p>
        </aside>
        <div className="column main">
          <header className="greeting">
            <div>
              <h1>{t.greeting(callMe, new Date().getHours())}</h1>
              <p className="muted">{presence}</p>
            </div>
            {on.includes("notes") && (
              <button
                type="button"
                className="icon"
                aria-label={t.notes.open}
                title={t.notes.open}
                onClick={() => props.onOpenNotes?.()}
              >
                <span aria-hidden="true">{""}</span>
              </button>
            )}
            {on.includes("alarms") && (
              <button
                type="button"
                className="icon"
                aria-label={t.alarms.open}
                title={t.alarms.open}
                onClick={() => props.onOpenAlarms?.()}
              >
                <span aria-hidden="true">{""}</span>
              </button>
            )}
            <button
              type="button"
              className="icon"
              aria-label={t.world.open}
              title={t.world.open}
              onClick={() => props.onOpenWorld?.()}
            >
              <span aria-hidden="true">{"\ue909"}</span>
            </button>
            <button
              type="button"
              className="icon"
              aria-label={t.dreams.open}
              title={t.dreams.open}
              onClick={() => props.onOpenDreams?.()}
            >
              <span aria-hidden="true">{""}</span>
            </button>
            <button
              type="button"
              className="icon"
              aria-label={t.memory.title}
              title={t.memory.title}
              onClick={() => props.onOpenMemory?.()}
            >
              <span aria-hidden="true">{"\ue82d"}</span>
            </button>
            <button
              type="button"
              className="icon"
              aria-label={t.settings}
              title={t.settings}
              onClick={props.onOpenSettings}
            >
              <span aria-hidden="true">{""}</span>
            </button>
          </header>
          <Conversation
            name={name}
            characterId={face}
            hasKey={hasKey}
            draft={props.draft ?? ""}
            onDraft={props.onDraft ?? (() => {})}
            onOpenSettings={props.onOpenSettings}
            onOpenMemory={props.onOpenMemory}
            onMark={onMark}
            notices={
              (recreatedFiles.length > 0 || !hasKey || problem || props.dreaming || offer) && (
                <>
                  {offer && (
                    <>
                      <Line small characterId={face} role="status">
                        <p>{t.extensions.offer(name)}</p>
                      </Line>
                      <div className="row under small">
                        <button type="button" onClick={() => answerOffer(false)}>
                          {t.extensions.notNow}
                        </button>
                        <button type="button" className="primary" onClick={() => answerOffer(true)}>
                          {t.extensions.yes}
                        </button>
                      </div>
                    </>
                  )}
                  {props.dreaming && (
                    <Line small asleep characterId={face} role="status">
                      <p>{t.dreams.dreaming(name)}</p>
                    </Line>
                  )}
                  {recreatedFiles.length > 0 && (
                    <Line small characterId={face} role="status">
                      <p>{t.recreated(recreatedFiles)}</p>
                    </Line>
                  )}
                  {!hasKey && say(t.noKey, t.addKey, props.onOpenSettings)}
                  {hasKey &&
                    problem &&
                    say(
                      t.problems[problem as keyof typeof t.problems] ?? t.chat.failures[problem],
                      problem === "Unreachable" ? t.retry : t.enterAnotherKey,
                      problem === "Unreachable" ? refresh : props.onOpenSettings,
                    )}
                </>
              )
            }
          />
        </div>
      </div>
    </main>
  );
}
