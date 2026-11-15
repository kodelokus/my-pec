import { useEffect, useState } from "react";
import {
  asAppError,
  chatState,
  compactConversation,
  compactionSettings,
  dreamingSettings,
  modelList,
  removeKey,
  selectModel,
  setAutoRecall,
  setCompactionSettings,
  setDreamingSettings,
  setSound,
  startContextCarrying,
} from "../api/commands";
import { AvatarView } from "../avatar/AvatarView";
import type { AppError } from "../bindings/AppError";
import type { CompactionSettings } from "../bindings/CompactionSettings";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { DreamingSettings } from "../bindings/DreamingSettings";
import type { KeyStatus } from "../bindings/KeyStatus";
import type { ModelList } from "../bindings/ModelList";
import { Confirm } from "../Confirm";
import { ExtensionsSection } from "../extensions/ExtensionsSection";
import { SemanticPanel } from "../memory/SemanticPanel";
import { KeyStep } from "../onboarding/KeyStep";
import { errorMessage, t } from "../strings";
import { Appearance } from "./Appearance";
import { LanguageSection } from "./LanguageSection";

/** onboarding FR-028 to FR-031: the key and the model. Plain wording, with the companion in the app bar (PER-1, PER-2). */
export function SettingsView(props: {
  keyStatus: KeyStatus;
  companion: CompanionSummary | null;
  onChanged: () => void;
  onClose: () => void;
}) {
  const [list, setList] = useState<ModelList | null>(null);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const [replaced, setReplaced] = useState(false);
  const [modelChange, setModelChange] = useState(false);
  const [autoRecall, setAutoRecallShown] = useState<boolean | null>(null);
  const [sound, setSoundShown] = useState<boolean | null>(null);
  const [compaction, setCompaction] = useState<CompactionSettings | null>(null);
  const [dreaming, setDreaming] = useState<DreamingSettings | null>(null);
  const [budget, setBudget] = useState("");
  const [error, setError] = useState<AppError | null>(null);
  const lastFour = props.keyStatus.status === "Present" ? props.keyStatus.lastFour : null;

  useEffect(() => {
    modelList(true)
      .then(setList)
      .catch((e) => setError(asAppError(e)));
    chatState()
      .then((s) => {
        setAutoRecallShown(s.autoRecall);
        setSoundShown(s.sound);
      })
      .catch((e) => setError(asAppError(e)));
    compactionSettings()
      .then((next) => {
        setCompaction(next);
        setBudget(String(next.budgetTokens));
      })
      .catch((e) => setError(asAppError(e)));
    dreamingSettings()
      .then(setDreaming)
      .catch((e) => setError(asAppError(e)));
  }, []);

  // dreaming FR-026: saved as a whole.
  const saveDreaming = (current: DreamingSettings, change: Partial<DreamingSettings>) => {
    const next = { ...current, ...change };
    return setDreamingSettings(next.enabled, next.time, next.model, next.world).then(setDreaming, (e) =>
      setError(asAppError(e)),
    );
  };

  function showCompaction(next: CompactionSettings) {
    setCompaction(next);
    setBudget(String(next.budgetTokens));
  }

  // compaction FR-003, FR-027: saved as a whole; an out-of-range size is put back as it was.
  const saveCompaction = (budgetTokens: number, model: string | null) =>
    setCompactionSettings(budgetTokens, model).then(showCompaction, (e) => {
      if (compaction) setBudget(String(compaction.budgetTokens));
      fail(e);
    });

  // compaction FR-028: switch now compacts onto the new model, or starts fresh with nothing to compact yet.
  const switchNow = () =>
    compactConversation(null, true)
      .then((r) => (r.outcome.kind === "NothingToCompact" ? startContextCarrying() : r.state))
      .then((s) => setModelChange(s.modelChangePending), fail);

  // chat FR-032: a new model takes effect in a fresh context.
  const checkModelChange = () =>
    chatState()
      .then((s) => setModelChange(s.modelChangePending))
      .catch(() => setModelChange(false));

  const fail = (e: unknown) => setError(asAppError(e));
  const name = (id: string) => list?.models.find((m) => m.id === id)?.displayName ?? id;

  return (
    <>
      <header className="app-bar">
        <button type="button" onClick={props.onClose}>
          {t.back}
        </button>
        <h1>{t.settingsTitle}</h1>
        {props.companion && (
          <>
            <span className="muted">{props.companion.name}</span>
            <AvatarView size={32} moment={error ? "concerned" : undefined} />
          </>
        )}
      </header>
      <main className="content">
        <div className="column">
          <LanguageSection />
          <section aria-labelledby="key-section" className="card">
            <h2 id="key-section">{t.keySection}</h2>
            <p>{lastFour ? t.keyPresent(lastFour) : t.keyAbsent}</p>
            {replaced && (
              <div role="status" className="alert success">
                <p>{t.keyReplaced}</p>
              </div>
            )}
            <KeyStep
              lastFour={lastFour}
              label={t.replaceKey}
              actions={
                lastFour && (
                  <button type="button" onClick={() => setConfirmRemove(true)}>
                    {t.removeKey}
                  </button>
                )
              }
              onAccepted={() => {
                setReplaced(lastFour !== null);
                props.onChanged();
              }}
            />
            {confirmRemove && (
              <Confirm
                message={t.removeConfirm}
                confirm={t.removeKey}
                danger
                onCancel={() => setConfirmRemove(false)}
                onConfirm={() => {
                  setConfirmRemove(false);
                  setReplaced(false);
                  removeKey().then(props.onChanged, fail);
                }}
              />
            )}
          </section>
          <section aria-labelledby="model-section" className="card">
            <h2 id="model-section">{t.modelSection}</h2>
            {list && (
              <>
                {/* A wrapping label would add the selected option to the picker's accessible name. */}
                <label htmlFor="model-picker">{t.modelLabel}</label>
                <select
                  id="model-picker"
                  value={list.selected}
                  onChange={(e) =>
                    selectModel(e.target.value).then((next) => {
                      setList({ ...list, selected: next.selected });
                      void checkModelChange();
                    }, fail)
                  }
                >
                  {list.models.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.displayName}
                    </option>
                  ))}
                </select>
                {modelChange && (
                  <div role="status" className="alert info">
                    <p>{t.chat.modelNote}</p>
                    <button type="button" onClick={() => void switchNow()}>
                      {t.chat.switchNow}
                    </button>
                  </div>
                )}
                {list.stale && (
                  <div role="status" className="alert info">
                    <p>{t.modelStale}</p>
                  </div>
                )}
                {list.fallbackFrom && (
                  <div role="status" className="alert info">
                    <p>{t.modelFallback(list.fallbackFrom, name(list.selected))}</p>
                  </div>
                )}
              </>
            )}
          </section>
          {compaction && list && (
            <section aria-labelledby="conversation-section" className="card">
              <h2 id="conversation-section">{t.conversationSection}</h2>
              {sound !== null && (
                <label className="check">
                  <input
                    type="checkbox"
                    checked={sound}
                    onChange={(e) =>
                      setSound(e.target.checked).then(
                        (s) => setSoundShown(s.sound),
                        (err) => setError(asAppError(err)),
                      )
                    }
                  />
                  {t.soundLabel}
                </label>
              )}
              <label htmlFor="budget">{t.budgetLabel}</label>
              <input
                id="budget"
                type="number"
                min={compaction.minBudget}
                max={compaction.maxBudget}
                step={compaction.budgetStep}
                value={budget}
                aria-describedby="budget-hint"
                onChange={(e) => setBudget(e.target.value)}
                onBlur={() => {
                  if (Number(budget) !== compaction.budgetTokens)
                    void saveCompaction(Number(budget), compaction.model);
                }}
              />
              <p id="budget-hint" className="hint">
                {t.budgetHint(compaction.minBudget, compaction.maxBudget)}
              </p>
              <label htmlFor="compaction-model">{t.compactionModelLabel}</label>
              <select
                id="compaction-model"
                value={compaction.model ?? ""}
                onChange={(e) => void saveCompaction(compaction.budgetTokens, e.target.value || null)}
              >
                <option value="">{t.sameAsChat}</option>
                {list.models.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.displayName}
                  </option>
                ))}
              </select>
              {compaction.modelMissing && (
                <div role="status" className="alert info">
                  <p>{t.compactionModelMissing}</p>
                </div>
              )}
            </section>
          )}
          {dreaming && list && (
            <section aria-labelledby="dreaming-section" className="card">
              <h2 id="dreaming-section">{t.dreamingSection}</h2>
              <label className="check">
                <input
                  type="checkbox"
                  checked={dreaming.enabled}
                  aria-describedby="dreaming-hint"
                  onChange={(e) => void saveDreaming(dreaming, { enabled: e.target.checked })}
                />
                {t.dreamingLabel}
              </label>
              <p id="dreaming-hint" className="hint">
                {t.dreamingHint}
              </p>
              {autoRecall === false && <p className="hint">{t.recallOffDreams}</p>}
              <label htmlFor="dream-time">{t.dreamTimeLabel}</label>
              <input
                id="dream-time"
                type="time"
                value={dreaming.time}
                onChange={(e) => {
                  if (e.target.value) void saveDreaming(dreaming, { time: e.target.value });
                }}
              />
              <label htmlFor="dream-model">{t.dreamModelLabel}</label>
              <select
                id="dream-model"
                value={dreaming.model ?? ""}
                onChange={(e) => void saveDreaming(dreaming, { model: e.target.value || null })}
              >
                <option value="">{t.sameAsChat}</option>
                {list.models.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.displayName}
                  </option>
                ))}
              </select>
              {dreaming.modelMissing && (
                <div role="status" className="alert info">
                  <p>{t.compactionModelMissing}</p>
                </div>
              )}
              <label className="check">
                <input
                  type="checkbox"
                  checked={dreaming.world}
                  aria-describedby="dream-world-hint"
                  onChange={(e) => void saveDreaming(dreaming, { world: e.target.checked })}
                />
                {t.dreamWorldLabel}
              </label>
              <p id="dream-world-hint" className="hint">
                {t.dreamWorldHint}
              </p>
            </section>
          )}
          <Appearance name={props.companion?.name ?? ""} />
          <ExtensionsSection />
          <SemanticPanel variant="settings" />
          <section aria-labelledby="memory-section" className="card">
            <h2 id="memory-section">{t.memorySection}</h2>
            {autoRecall !== null && (
              <label className="check">
                <input
                  type="checkbox"
                  checked={autoRecall}
                  aria-describedby="auto-recall-hint"
                  onChange={(e) =>
                    setAutoRecall(e.target.checked).then((s) => setAutoRecallShown(s.autoRecall), fail)
                  }
                />
                {t.autoRecallLabel}
              </label>
            )}
            <p id="auto-recall-hint" className="hint">
              {t.autoRecallHint}
            </p>
          </section>
          {error && (
            <div role="alert" className="alert error">
              <p>{errorMessage(error)}</p>
            </div>
          )}
          <p className="hint">{t.version(__APP_VERSION__)}</p>
        </div>
      </main>
    </>
  );
}
