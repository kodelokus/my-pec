import type { AboutDraft } from "../bindings/AboutDraft";
import { Line } from "../Line";
import { t } from "../strings";
import type { Notice } from "./Wizard";

/** onboarding FR-021: both values are optional; Skip leaves them unset. */
export function AboutYouStep(props: {
  about: AboutDraft;
  name: string;
  face: string;
  notice: Notice;
  onChange: (about: AboutDraft) => void;
}) {
  const { about, onChange } = props;
  return (
    <>
      <Line characterId={props.face} role={props.notice?.alert ? "alert" : "status"}>
        {props.notice?.text ?? t.line.about}
      </Line>
      <div className="stack under">
        <label>
          {t.callMeLabel(props.name)}
          <input
            value={about.callMe ?? ""}
            onChange={(e) => onChange({ ...about, callMe: e.target.value })}
          />
        </label>
        <label>
          {t.languageLabel}
          <input
            value={about.language ?? ""}
            onChange={(e) => onChange({ ...about, language: e.target.value })}
          />
        </label>
      </div>
    </>
  );
}
