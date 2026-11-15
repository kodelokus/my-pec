import { useState } from "react";
import type { SetupOptions } from "../bindings/SetupOptions";
import type { SoulDraft } from "../bindings/SoulDraft";
import { Confirm } from "../Confirm";
import { Line } from "../Line";
import { samples, t } from "../strings";
import type { Notice } from "./Wizard";

/** onboarding FR-016 to FR-019. The companion quotes the chosen template's sample line (PER-9). */
export function SoulStep(props: {
  soul: SoulDraft;
  options: SetupOptions;
  face: string;
  notice: Notice;
  onChange: (soul: SoulDraft) => void;
}) {
  const { soul, options, onChange } = props;
  const [pending, setPending] = useState<string | null>(null);
  const current = options.templates.find((tpl) => tpl.id === soul.templateId);
  const edited = soul.text !== (current?.text ?? "");
  // Count characters, not UTF-16 units, to match the core's budget.
  const used = [...soul.text].length;
  const budget = options.soulBudgetChars;
  const over = used > budget;
  const sample = samples[soul.templateId] ?? samples.blank;
  const name = soul.name.trim() || options.defaultName;

  function pick(id: string) {
    const text = options.templates.find((tpl) => tpl.id === id)?.text ?? "";
    onChange({ ...soul, templateId: id, text });
    setPending(null);
  }

  return (
    <>
      <Line
        characterId={props.face}
        role={props.notice?.alert || (!props.notice && over) ? "alert" : "status"}
      >
        {props.notice?.text ?? (over ? t.line.overBudget(budget) : t.line.soul(sample.as, sample.line(name)))}
      </Line>
      <div className="soul">
        <fieldset className="options">
          <legend>{t.templatesLabel}</legend>
          {options.templates.map((tpl) => (
            <label key={tpl.id} className="option">
              <input
                type="radio"
                name="template"
                value={tpl.id}
                checked={tpl.id === soul.templateId}
                onChange={() => (edited ? setPending(tpl.id) : pick(tpl.id))}
              />
              <span>
                {tpl.title}
                <small>{tpl.description}</small>
              </span>
            </label>
          ))}
        </fieldset>
        <div className="editor">
          <label>
            {t.nameLabel}
            <input
              value={soul.name}
              maxLength={options.nameMaxChars}
              placeholder={options.defaultName}
              aria-describedby={soul.name.trim() ? undefined : "name-hint"}
              onChange={(e) => onChange({ ...soul, name: e.target.value })}
            />
          </label>
          {!soul.name.trim() && (
            <p id="name-hint" className="hint">
              {t.nameHint(options.defaultName)}
            </p>
          )}
          <label>
            {t.soulTextLabel}
            <textarea
              rows={12}
              value={soul.text}
              aria-describedby="soul-budget"
              onChange={(e) => onChange({ ...soul, text: e.target.value })}
            />
          </label>
          <progress className={over ? "meter over" : "meter"} max={budget} value={Math.min(used, budget)} />
          <p id="soul-budget" role="status" className={over ? "count over" : "count"}>
            {t.budget(used, budget)}
          </p>
        </div>
      </div>
      {pending && (
        <Confirm
          message={t.discardConfirm}
          confirm={t.discard}
          danger
          onConfirm={() => pick(pending)}
          onCancel={() => setPending(null)}
        />
      )}
    </>
  );
}
