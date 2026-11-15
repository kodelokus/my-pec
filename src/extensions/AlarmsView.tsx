import { useCallback, useEffect, useState } from "react";
import { alarmAction, alarms, asAppError, deleteAlarm, onExtChanged, saveAlarm } from "../api/commands";
import type { Alarm } from "../bindings/Alarm";
import type { AlarmList } from "../bindings/AlarmList";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { Day } from "../bindings/Day";
import type { Repeat } from "../bindings/Repeat";
import { Line } from "../Line";
import { errorMessage, t } from "../strings";

const DAYS: Day[] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
type Kind = Repeat["kind"];
type Draft = { key: string | null; label: string; time: string; kind: Kind; date: string; days: Day[] };

const empty: Draft = { key: null, label: "", time: "", kind: "once", date: "", days: [] };

function draftOf(alarm: Alarm): Draft {
  const r = alarm.repeat;
  return {
    key: alarm.key,
    label: alarm.label,
    time: alarm.time,
    kind: r.kind,
    date: r.kind === "once" ? r.date : "",
    days: r.kind === "weekdays" ? r.days : r.kind === "weekly" ? [r.day] : [],
  };
}

function repeatOf(d: Draft): Repeat {
  switch (d.kind) {
    case "once":
      return { kind: "once", date: d.date };
    case "daily":
      return { kind: "daily" };
    case "weekdays":
      return { kind: "weekdays", days: d.days };
    case "weekly":
      return { kind: "weekly", day: d.days[0] ?? "Mon" };
  }
}

function repeatText(r: Repeat): string {
  switch (r.kind) {
    case "once":
      return r.date;
    case "daily":
      return t.alarms.repeats.daily;
    case "weekdays":
      return r.days.map((d) => t.alarms.days[d]).join(" ");
    case "weekly":
      return `${t.alarms.repeats.weekly}: ${t.alarms.days[r.day]}`;
  }
}

/** extensions FR-017 to FR-025: the alarm list and its editor. */
export function AlarmsView(props: {
  companion: CompanionSummary | null;
  onClose: () => void;
  onSaved?: () => void;
}) {
  const [list, setList] = useState<AlarmList | null>(null);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const name = props.companion?.name ?? "";

  const fail = useCallback((e: unknown) => {
    const error = asAppError(e);
    setNote(
      error.kind === "InvalidInput"
        ? (t.alarms.invalid[error.field] ?? errorMessage(error))
        : errorMessage(error),
    );
  }, []);
  const load = useCallback(() => {
    alarms().then(setList, fail);
  }, [fail]);
  useEffect(() => {
    load();
    const off = onExtChanged((ids) => ids.includes("alarms") && load());
    return () => {
      off.then((unlisten) => unlisten());
    };
  }, [load]);

  const done = (next: Promise<AlarmList>, saved = false) =>
    next.then((l) => {
      setList(l);
      setNote(null);
      setDraft(null);
      if (saved) props.onSaved?.();
    }, fail);

  const editor = draft && (
    <form
      className="card editor"
      onSubmit={(e) => {
        e.preventDefault();
        void done(saveAlarm(draft.key, draft.label, draft.time, repeatOf(draft)), true);
      }}
    >
      <label htmlFor="alarm-label">{t.alarms.labelLabel}</label>
      <input
        id="alarm-label"
        value={draft.label}
        maxLength={200}
        onChange={(e) => setDraft({ ...draft, label: e.target.value })}
      />
      <label htmlFor="alarm-time">{t.alarms.timeLabel}</label>
      <input
        id="alarm-time"
        type="time"
        value={draft.time}
        onChange={(e) => setDraft({ ...draft, time: e.target.value })}
      />
      <label htmlFor="alarm-repeat">{t.alarms.repeatLabel}</label>
      <select
        id="alarm-repeat"
        value={draft.kind}
        onChange={(e) => setDraft({ ...draft, kind: e.target.value as Kind })}
      >
        {(Object.keys(t.alarms.repeats) as Kind[]).map((k) => (
          <option key={k} value={k}>
            {t.alarms.repeats[k]}
          </option>
        ))}
      </select>
      {draft.kind === "once" && (
        <>
          <label htmlFor="alarm-date">{t.alarms.dateLabel}</label>
          <input
            id="alarm-date"
            type="date"
            value={draft.date}
            onChange={(e) => setDraft({ ...draft, date: e.target.value })}
          />
        </>
      )}
      {(draft.kind === "weekdays" || draft.kind === "weekly") && (
        <fieldset className="days">
          <legend>{t.alarms.daysLabel}</legend>
          {DAYS.map((day) => (
            <label key={day} className="check">
              <input
                type={draft.kind === "weekly" ? "radio" : "checkbox"}
                name="alarm-days"
                checked={draft.days.includes(day)}
                onChange={(e) =>
                  setDraft({
                    ...draft,
                    days:
                      draft.kind === "weekly"
                        ? [day]
                        : e.target.checked
                          ? [...draft.days, day]
                          : draft.days.filter((d) => d !== day),
                  })
                }
              />
              {t.alarms.days[day]}
            </label>
          ))}
        </fieldset>
      )}
      <div className="row">
        <button type="button" onClick={() => setDraft(null)}>
          {t.alarms.cancel}
        </button>
        <button type="submit" className="primary">
          {t.alarms.save}
        </button>
      </div>
    </form>
  );

  return (
    <>
      <header className="app-bar">
        <button type="button" onClick={props.onClose}>
          {t.back}
        </button>
        <h1>{t.alarms.title}</h1>
      </header>
      <main className="content">
        <div className="column">
          <Line characterId={props.companion?.characterId ?? ""}>
            <p>{t.alarms.line(name)}</p>
          </Line>
          {list && list.unreadable > 0 && (
            <p role="status" className="hint">
              {t.alarms.unreadable(list.unreadable)}
            </p>
          )}
          {list?.notificationsHeld && (
            <p role="status" className="hint">
              {t.alarms.held}
            </p>
          )}
          {note && (
            <div role="alert" className="alert error">
              <p>{note}</p>
            </div>
          )}
          {editor ?? (
            <div className="row">
              <button type="button" className="primary" onClick={() => setDraft(empty)}>
                {t.alarms.add}
              </button>
            </div>
          )}
          {list && list.alarms.length === 0 && <p className="muted meta">{t.alarms.empty}</p>}
          <ul className="ext-list" aria-label={t.alarms.title}>
            {list?.alarms.map((alarm) => (
              <li key={alarm.key} className={`card ${alarm.state}`}>
                <div>
                  <strong>
                    {alarm.time} {alarm.label}
                  </strong>
                  <p className="muted meta">
                    {repeatText(alarm.repeat)} · {t.alarms.states[alarm.state]}
                    {alarm.next && ` · ${t.alarms.next(alarm.next)}`}
                  </p>
                </div>
                <div className="row">
                  {(alarm.state === "waiting" || alarm.state === "snoozed") && (
                    <>
                      <button type="button" onClick={() => void done(alarmAction(alarm.key, "snooze"))}>
                        {t.alarms.snooze}
                      </button>
                      <button type="button" onClick={() => void done(alarmAction(alarm.key, "done"))}>
                        {t.alarms.done}
                      </button>
                    </>
                  )}
                  <button type="button" onClick={() => setDraft(draftOf(alarm))}>
                    {t.alarms.edit}
                  </button>
                  <button type="button" onClick={() => void done(deleteAlarm(alarm.key))}>
                    {t.alarms.delete}
                  </button>
                </div>
              </li>
            ))}
          </ul>
        </div>
      </main>
    </>
  );
}
