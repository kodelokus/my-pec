import type { ReactNode } from "react";
import { AvatarView } from "./avatar/AvatarView";
import { portrait } from "./portraits";

/** The companion with a speech bubble (PER-1, PER-2). The character is decorative; the line is real text. */
export function Line(props: {
  characterId: string;
  /** Problems are alerts and other lines that change are status; conversation messages take neither. */
  role?: "alert" | "status";
  id?: string;
  /** The conversation's size: a small still portrait and the companion's message bubble. */
  small?: boolean;
  /** dreaming FR-025: the portrait asleep while the companion dreams (PER-11). */
  asleep?: boolean;
  /** avatar FR-013: the character's mood while this line is shown. */
  moment?: "happy" | "concerned";
  /** A further bubble of the same message: no portrait or tail, set under the first. */
  follow?: boolean;
  children: ReactNode;
}) {
  const classes = ["line", props.small && "small", props.asleep && "asleep", props.follow && "follow"]
    .filter(Boolean)
    .join(" ");
  return (
    <div className={classes}>
      {props.follow ? (
        <span className="portrait-space" aria-hidden="true" />
      ) : props.small ? (
        <img
          src={portrait(props.characterId, { sleeping: props.asleep })}
          alt=""
          width={40}
          height={40}
          className="portrait"
        />
      ) : (
        <AvatarView size={88} character={props.characterId} moment={props.moment} />
      )}
      {props.asleep && (
        <span className="zzz" aria-hidden="true">
          z
        </span>
      )}
      <div id={props.id} role={props.role} className={props.small ? "bubble companion" : "bubble voice"}>
        {props.children}
      </div>
    </div>
  );
}
