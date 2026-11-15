import { useCallback, useEffect, useState } from "react";
import { asAppError, forgetTopic, startFreshContext, worldView } from "../api/commands";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { DrawnItem } from "../bindings/DrawnItem";
import type { JourneyDay } from "../bindings/JourneyDay";
import type { WorldView as World } from "../bindings/WorldView";
import { Confirm } from "../Confirm";
import { Line } from "../Line";
import { errorMessage, t } from "../strings";
import { WorldPicture } from "./WorldPicture";

export const itemName = (item: DrawnItem) =>
  item.known ? t.world.name(item.item) : t.world.unknown(item.item);

/** How long each day shows while the time control plays (journey FR-030). */
const STEP_MS = 600;

/** WLD-8: the companion's world as a map of the user's days, led by the companion (world FR-029, journey FR-032). */
export function WorldView(props: {
  companion: CompanionSummary | null;
  onClose: () => void;
  onOpenNotes: (day: string) => void;
}) {
  const [world, setWorld] = useState<World | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [centre, setCentre] = useState<number | null>(null);
  // The time control: an index into the days, or none for today (journey FR-030).
  const [until, setUntil] = useState<number | null>(null);
  const [playing, setPlaying] = useState(false);
  const [forgetting, setForgetting] = useState<{
    day: string;
    n: number;
    keyword: string;
    line: string;
  } | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [stillSeen, setStillSeen] = useState(false);
  const fail = useCallback((e: unknown) => setError(errorMessage(asAppError(e))), []);
  const load = useCallback(() => worldView().then(setWorld, fail), [fail]);

  useEffect(() => {
    void load();
  }, [load]);

  const days = [...new Set(world?.items.map((i) => i.day) ?? [])].sort();
  useEffect(() => {
    if (!playing) return;
    const timer = setInterval(() => {
      setUntil((u) => {
        const next = (u ?? -1) + 1;
        if (next >= days.length - 1) {
          setPlaying(false);
          return null;
        }
        return next;
      });
    }, STEP_MS);
    return () => clearInterval(timer);
  }, [playing, days.length]);

  const name = props.companion?.name ?? "";
  const face = props.companion?.characterId ?? "";
  const items = world?.items ?? [];
  const journey = world?.journey ?? [];
  const drawn = items.filter((i) => !i.item.startsWith("home."));
  const lastDay = drawn.at(-1)?.day;
  const latest = drawn.filter((i) => i.day === lastDay).map(itemName);
  const label = (item: DrawnItem) => t.world.itemLabel(itemName(item), t.memory.day(item.day));
  const storyOf = (item: DrawnItem) =>
    journey.find((d) => d.day === item.topic?.day)?.topics.find((topic) => topic.n === item.topic?.n)
      ?.story ?? null;
  // journey FR-032: the newest day with a story, told only from the record.
  const told = journey.find((d) => d.topics.some((topic) => topic.story && topic.item !== null));
  const picks = (told?.topics ?? []).flatMap((topic) =>
    topic.story && topic.item !== null
      ? [{ name: itemName(items[topic.item]), keyword: topic.story.keyword }]
      : [],
  );
  const line = told ? t.world.journeyLine(t.memory.day(told.day), picks) : t.world.line(drawn.length, latest);
  // journey FR-026: one entry per day, newest first; days the journey never considered list their items as before.
  const listed = [...days].reverse();
  const choose = (i: number) => {
    setSelected(i);
    setCentre(i);
  };
  const itemButton = (i: number) => (
    <button type="button" className="link" aria-pressed={selected === i} onClick={() => choose(i)}>
      {itemName(items[i])}
    </button>
  );
  const chosen = selected === null ? undefined : items[selected];
  const chosenStory = chosen ? storyOf(chosen) : null;

  const diaryDay = (day: string, entry: JourneyDay | undefined) => {
    const indexes = items.flatMap((item, i) => (item.day === day ? [i] : []));
    if (!entry) {
      return (
        <ul className="world-day">
          {indexes.map((i) => (
            <li key={i}>{itemButton(i)}</li>
          ))}
        </ul>
      );
    }
    return (
      <>
        <ul>
          {entry.topics.map((topic) => (
            <li key={topic.n}>
              {topic.item === null ? t.world.name(topic.drawn) : itemButton(topic.item)}
              {topic.story ? (
                <>
                  {` — “${topic.story.keyword}”`}
                  <p className="world-story">{topic.story.line}</p>
                  <details>
                    <summary>{t.world.whyThisOne}</summary>
                    <table className="world-odds">
                      <caption className="visually-hidden">
                        {t.world.oddsCaption(topic.story.keyword)}
                      </caption>
                      <thead>
                        <tr>
                          <th scope="col">{t.world.oddsItem}</th>
                          <th scope="col">{t.world.oddsChance}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {topic.story.odds.map((o) => (
                          <tr key={o.item} className={o.item === topic.drawn ? "picked" : undefined}>
                            <td>
                              {t.world.name(o.item)}
                              {o.item === topic.drawn && ` (${t.world.picked})`}
                            </td>
                            <td>{t.world.percent(o.chance)}</td>
                          </tr>
                        ))}
                        {topic.story.rest > 0 && (
                          <tr>
                            <td>{t.world.rest}</td>
                            <td>{t.world.percent(topic.story.rest)}</td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                    {topic.story.related && <p className="muted">{t.world.related}</p>}
                  </details>
                  <button
                    type="button"
                    className="subtle"
                    onClick={() => {
                      const story = topic.story;
                      if (story) setForgetting({ day, n: topic.n, keyword: story.keyword, line: story.line });
                    }}
                  >
                    {t.world.forgetStory}
                  </button>
                </>
              ) : (
                <p className="world-story">{t.world.forgottenStory}</p>
              )}
            </li>
          ))}
        </ul>
        {entry.scenery.length > 0 && (
          <>
            <p className="muted">{t.world.sceneryLabel}</p>
            <ul className="world-day">
              {entry.scenery.map((i) => (
                <li key={i}>{itemButton(i)}</li>
              ))}
            </ul>
          </>
        )}
        {entry.hasNotes && (
          <button type="button" className="link" onClick={() => props.onOpenNotes(day)}>
            {t.world.notes}
          </button>
        )}
      </>
    );
  };

  return (
    <>
      <header className="app-bar">
        <button type="button" onClick={props.onClose}>
          {t.back}
        </button>
        <h1>{t.world.title(name)}</h1>
      </header>
      <main className="content wash">
        <div className="column wide world">
          <Line characterId={face} role="status">
            {world ? line : t.loading}
          </Line>
          {world && (
            <>
              <section className="card world-stage under" aria-label={t.world.pictureLabel(name)}>
                <WorldPicture
                  items={world.items}
                  palette={world.palette}
                  label={t.world.summary(name, world.items.length, latest)}
                  centre={centre}
                  selected={selected}
                  until={until === null ? null : days[until]}
                  onSelect={setSelected}
                  itemLabel={label}
                  itemTip={(item) => ({ name: itemName(item), date: t.memory.day(item.day) })}
                  controls={t.world.controls}
                />
                {days.length > 1 && (
                  <div className="row world-time">
                    <label>
                      {t.world.timeLabel}{" "}
                      <input
                        type="range"
                        min={0}
                        max={days.length - 1}
                        value={until ?? days.length - 1}
                        aria-valuetext={t.memory.day(days[until ?? days.length - 1])}
                        onChange={(e) => {
                          setPlaying(false);
                          const value = Number(e.target.value);
                          setUntil(value >= days.length - 1 ? null : value);
                        }}
                      />
                    </label>
                    <button
                      type="button"
                      onClick={() => {
                        if (!playing && until === null) setUntil(0);
                        setPlaying(!playing);
                      }}
                    >
                      {playing ? t.world.pause : t.world.play}
                    </button>
                  </div>
                )}
              </section>
              {chosen && (
                <p role="status" className="under">
                  {chosenStory
                    ? t.world.story(
                        itemName(chosen),
                        t.memory.day(chosen.day),
                        chosenStory.keyword,
                        chosenStory.line,
                      )
                    : label(chosen)}
                </p>
              )}
              {status && (
                <p role="status" className="under muted">
                  {status}
                </p>
              )}
              {stillSeen && (
                <div role="status" className="alert info under">
                  <p>{t.memory.stillSeen}</p>
                  <button
                    type="button"
                    onClick={() => void startFreshContext().then(() => setStillSeen(false), fail)}
                  >
                    {t.chat.freshContext}
                  </button>
                </div>
              )}
              <section aria-labelledby="world-makeup" className="card under">
                <h2 id="world-makeup">{t.world.makeupTitle}</h2>
                {world.makeup.length === 0 ? (
                  <p className="muted">{t.world.makeupEmpty}</p>
                ) : (
                  <>
                    <div className="world-makeup-bar" aria-hidden="true">
                      {world.makeup.map((s) => (
                        <span
                          key={s.subject}
                          style={{ flexGrow: s.share }}
                          title={t.world.subject(s.subject)}
                        >
                          {s.share >= 0.1 ? t.world.subject(s.subject) : ""}
                        </span>
                      ))}
                    </div>
                    <ul className="world-makeup">
                      {world.makeup.map((s) => (
                        <li key={s.subject}>
                          {t.world.subject(s.subject)} {t.world.percent(s.share)}
                        </li>
                      ))}
                    </ul>
                  </>
                )}
              </section>
              <section aria-labelledby="world-list" className="card under world-list">
                <h2 id="world-list">{t.world.listTitle}</h2>
                {world.skippedLines > 0 && <p className="hint">{t.world.skipped(world.skippedLines)}</p>}
                {world.journeySkipped > 0 && <p className="hint">{t.world.skipped(world.journeySkipped)}</p>}
                {listed.map((day) => (
                  <section key={day} aria-label={t.memory.day(day)} className="world-diary-day">
                    <h3>{t.memory.day(day)}</h3>
                    {diaryDay(
                      day,
                      journey.find((d) => d.day === day),
                    )}
                  </section>
                ))}
              </section>
              <section aria-labelledby="world-about" className="card under">
                <h2 id="world-about">{t.world.about}</h2>
                <dl className="world-facts">
                  <dt>{t.world.born}</dt>
                  <dd>{world.born ? t.memory.day(world.born) : t.world.unknownBirth}</dd>
                </dl>
                <p className="muted">{t.world.howItGrows}</p>
              </section>
            </>
          )}
          {error && (
            <div role="alert" className="alert error">
              <p>{error}</p>
            </div>
          )}
        </div>
      </main>
      {forgetting && (
        <Confirm
          message={t.world.forgetStoryConfirm(forgetting.keyword, forgetting.line)}
          confirm={t.memory.forget}
          danger
          onCancel={() => setForgetting(null)}
          onConfirm={() => {
            const target = forgetting;
            setForgetting(null);
            forgetTopic(target.day, target.n, world?.journeyHash ?? "").then((outcome) => {
              setStatus(t.world.storyForgotten);
              setStillSeen(outcome.inCurrentContext);
              void load();
            }, fail);
          }}
        />
      )}
    </>
  );
}
