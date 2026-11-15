import { useCallback, useEffect, useRef, useState } from "react";
import {
  appStatus,
  asAppError,
  chatState,
  dreamTick,
  hideToTray,
  onEvent,
  semanticTick,
  showNote,
} from "./api/commands";
import type { AppError } from "./bindings/AppError";
import type { AppStatus } from "./bindings/AppStatus";
import { DreamJournal } from "./dreams/DreamJournal";
import { AlarmsView } from "./extensions/AlarmsView";
import { NotesView } from "./extensions/NotesView";
import { MainView } from "./main/MainView";
import { type MemoryTarget, MemoryView } from "./memory/MemoryView";
import { Wizard } from "./onboarding/Wizard";
import { SettingsView } from "./settings/SettingsView";
import { errorMessage, t } from "./strings";
import { WorldView } from "./world/WorldView";

/** dreaming R2: how often the app asks the core whether a night is due. */
const TICK_MS = 60_000;

/** onboarding FR-002: the wizard until setup is complete, then the main view or settings. */
export function App() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [settings, setSettings] = useState(false);
  const [memory, setMemory] = useState<MemoryTarget | null>(null);
  const [world, setWorld] = useState(false);
  const [dreams, setDreams] = useState(false);
  const [notesOpen, setNotesOpen] = useState(false);
  const [alarmsOpen, setAlarmsOpen] = useState(false);
  const [dreaming, setDreaming] = useState(false);
  const ticking = useRef(false);
  const [closeNotice, setCloseNotice] = useState(false);
  const [draft, setDraft] = useState("");
  const ok = useRef<HTMLButtonElement>(null);

  const load = useCallback(() => {
    setError(null);
    appStatus()
      .then(setStatus)
      .catch((e) => setError(asAppError(e)));
  }, []);

  useEffect(() => {
    load();
    const subscriptions = [
      onEvent("close_notice", () => setCloseNotice(true)),
      onEvent("open_settings", () => setSettings(true)),
      // extensions FR-022: a notification was clicked.
      onEvent("open_alarms", () => setAlarmsOpen(true)),
    ];
    return () => {
      for (const s of subscriptions) s.then((unlisten) => unlisten());
    };
  }, [load]);

  // dreaming R2: once at start and every minute, ask whether a night is due; the core runs it inside
  // the call. While one runs, the next ticks only read whether the companion is dreaming.
  const ready = status?.setup.state === "Complete" && status.key.status === "Present";
  useEffect(() => {
    if (!ready) return;
    const tick = () => {
      // search R8: memories are prepared for search by meaning a little each minute.
      semanticTick().catch(() => {});
      if (ticking.current) {
        chatState().then(
          (s) => setDreaming(s.dreaming),
          () => {},
        );
        return;
      }
      ticking.current = true;
      const peek = setTimeout(
        () =>
          chatState().then(
            (s) => setDreaming(s.dreaming),
            () => {},
          ),
        1500,
      );
      dreamTick()
        .catch(() => {})
        .finally(() => {
          clearTimeout(peek);
          ticking.current = false;
          setDreaming(false);
        });
    };
    tick();
    const every = setInterval(tick, TICK_MS);
    return () => clearInterval(every);
  }, [ready]);

  // Notice dialogs start on OK, as confirmations start on Cancel (onboarding FR-009).
  useEffect(() => {
    if (closeNotice) ok.current?.focus();
  }, [closeNotice]);

  const notice = closeNotice && (
    <div className="scrim">
      <div role="alertdialog" aria-modal="true" aria-label={t.closeNotice} className="dialog">
        <p>{t.closeNotice}</p>
        <div className="row">
          <button
            type="button"
            className="primary"
            ref={ok}
            onClick={() => {
              setCloseNotice(false);
              void hideToTray();
            }}
          >
            {t.ok}
          </button>
        </div>
      </div>
    </div>
  );

  if (error) {
    return (
      <main className="content">
        <div role="alert" className="column alert error">
          <p>{errorMessage(error)}</p>
          <button type="button" onClick={load}>
            {t.retry}
          </button>
        </div>
      </main>
    );
  }
  if (!status) {
    return (
      <p role="status" className="loading row">
        <span className="spinner" aria-hidden="true" />
        {t.loading}
      </p>
    );
  }

  let view: React.ReactNode;
  if (status.setup.state === "InProgress") {
    view = (
      <Wizard
        draft={status.setup.draft}
        lastFour={status.key.status === "Present" ? status.key.lastFour : null}
        existing={status.existingWorkspace ? status.companion : null}
        onDone={load}
      />
    );
  } else if (settings) {
    view = (
      <SettingsView
        keyStatus={status.key}
        companion={status.companion}
        onChanged={load}
        onClose={() => setSettings(false)}
      />
    );
  } else if (alarmsOpen) {
    view = <AlarmsView companion={status.companion} onClose={() => setAlarmsOpen(false)} />;
  } else if (notesOpen) {
    view = <NotesView companion={status.companion} onClose={() => setNotesOpen(false)} />;
  } else if (dreams) {
    view = <DreamJournal companion={status.companion} onClose={() => setDreams(false)} />;
  } else if (world) {
    view = (
      <WorldView
        companion={status.companion}
        onClose={() => setWorld(false)}
        onOpenNotes={(day) => {
          // journey FR-029: the day's notes in the memory browser.
          setWorld(false);
          setMemory({ path: `memory/${day}.md`, line: null });
        }}
      />
    );
  } else if (memory) {
    view = <MemoryView companion={status.companion} open={memory} onClose={() => setMemory(null)} />;
  } else {
    view = (
      <MainView
        key={status.key.status}
        status={status}
        onOpenSettings={() => setSettings(true)}
        onOpenMemory={(path, line) => {
          // extensions R3: an extension's mark opens its note or the alarm list.
          if (path?.startsWith("notes/"))
            return void showNote(path.slice("notes/".length), true).catch(() => {});
          if (path === "alarms.md") return setAlarmsOpen(true);
          setMemory({ path: path ?? "MEMORY.md", line: line ?? null });
        }}
        onOpenWorld={() => setWorld(true)}
        onOpenDreams={() => setDreams(true)}
        onOpenNotes={() => setNotesOpen(true)}
        onOpenAlarms={() => setAlarmsOpen(true)}
        dreaming={dreaming}
        draft={draft}
        onDraft={setDraft}
      />
    );
  }
  return (
    <>
      {view}
      {notice}
    </>
  );
}
