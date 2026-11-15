import { AvatarView } from "../avatar/AvatarView";
import type { Character } from "../bindings/Character";
import { Line } from "../Line";
import { t } from "../strings";
import type { Notice } from "./Wizard";

/** onboarding FR-020, avatar FR-028: the bundled characters, moving where they can, one preselected. */
export function AvatarStep(props: {
  characterId: string;
  characters: Character[];
  notice: Notice;
  onChange: (id: string) => void;
}) {
  const only = props.characters.length === 1 ? props.characters[0] : null;
  return (
    <>
      <Line characterId={props.characterId} role={props.notice?.alert ? "alert" : "status"}>
        {props.notice?.text ?? (only ? t.line.avatarOnly(only.name) : t.line.avatar)}
      </Line>
      <fieldset className="options characters under">
        <legend>{t.charactersLabel}</legend>
        {props.characters.map((c) => (
          <label key={c.id} className="option character">
            <span>
              <input
                type="radio"
                name="character"
                value={c.id}
                checked={c.id === props.characterId}
                onChange={() => props.onChange(c.id)}
              />
              {c.name}
            </span>
            <AvatarView size={112} character={c.id} source={{ kind: "Bundled", character: c.id }} />
          </label>
        ))}
      </fieldset>
    </>
  );
}
