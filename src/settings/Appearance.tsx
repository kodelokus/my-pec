import { useEffect, useState } from "react";
import {
  asAppError,
  avatarManifest,
  chooseAvatar,
  floatingSettings,
  onEvent,
  setAvatarPicture,
  setFloating,
  setupOptions,
} from "../api/commands";
import { AvatarView } from "../avatar/AvatarView";
import { tryAgain, useRendererProblem } from "../avatar/problem";
import type { AvatarManifest } from "../bindings/AvatarManifest";
import type { Character } from "../bindings/Character";
import { errorMessage, t } from "../strings";

/** How the companion looks, whether it floats, and why the character can't be shown (avatar FR-017, FR-023, FR-029, FR-030). */
export function Appearance(props: { name: string }) {
  const [characters, setCharacters] = useState<Character[]>([]);
  const [manifest, setManifest] = useState<AvatarManifest | null>(null);
  const [floating, setFloatingShown] = useState<boolean | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const store = useRendererProblem();

  useEffect(() => {
    const fail = (e: unknown) => setMessage(errorMessage(asAppError(e)));
    setupOptions().then((o) => setCharacters(o.characters), fail);
    avatarManifest().then(setManifest, fail);
    const readFloating = () => floatingSettings().then((f) => setFloatingShown(f.on), fail);
    void readFloating();
    // The tray's item changes it too.
    const off = onEvent("floating_changed", readFloating);
    return () => void off.then((stop) => stop());
  }, []);

  const show = (next: Promise<AvatarManifest>) =>
    next.then(
      (m) => {
        setManifest(m);
        setMessage(null);
      },
      (e) => {
        const error = asAppError(e);
        const refused = error.kind === "InvalidInput" && t.avatar.pictureRefused[error.field];
        setMessage(refused || errorMessage(error));
      },
    );

  const look = manifest?.look;
  const problem = store.look === JSON.stringify(look ?? null) ? store.problem : null;
  const chosen = look?.type === "static" ? "still" : look?.type === "live2d" ? look.character : null;
  return (
    <section aria-labelledby="appearance-section" className="card">
      <h2 id="appearance-section">{t.avatar.section}</h2>
      <fieldset className="options characters">
        <legend className="visually-hidden">{t.avatar.section}</legend>
        {characters.map((c) => (
          <label key={c.id} className="option character">
            <span>
              <input
                type="radio"
                name="look"
                checked={chosen === c.id}
                onChange={() => void show(chooseAvatar(c.id, false))}
              />
              {c.name}
            </span>
            <AvatarView size={72} character={c.id} />
          </label>
        ))}
        {look && (
          <label className="option character">
            <span>
              <input
                type="radio"
                name="look"
                checked={chosen === "still"}
                onChange={() => void show(chooseAvatar(look.character, true))}
              />
              {t.avatar.still}
            </span>
          </label>
        )}
      </fieldset>
      <label htmlFor="own-picture">{t.avatar.picture}</label>
      <input
        id="own-picture"
        type="file"
        accept="image/png,image/jpeg,image/webp"
        aria-describedby="own-picture-hint"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) void file.arrayBuffer().then((bytes) => show(setAvatarPicture(bytes)));
          e.target.value = "";
        }}
      />
      <p id="own-picture-hint" className="hint">
        {t.avatar.pictureHint}
      </p>
      {message && (
        <div role="alert" className="alert error">
          <p>{message}</p>
        </div>
      )}
      {manifest?.problem && (
        <div role="status" className="alert info">
          <p>{t.avatar.manifest[manifest.problem]}</p>
        </div>
      )}
      {problem && (
        <div role="status" className="alert info">
          <p>{t.avatar.problems[problem]}</p>
          <button type="button" onClick={tryAgain}>
            {t.avatar.tryAgain}
          </button>
        </div>
      )}
      {floating !== null && (
        <>
          <label className="check">
            <input
              type="checkbox"
              checked={floating}
              aria-describedby="floating-hint"
              onChange={(e) =>
                setFloating(e.target.checked).then(
                  (f) => setFloatingShown(f.on),
                  (err) => setMessage(errorMessage(asAppError(err))),
                )
              }
            />
            {t.avatar.floating(props.name)}
          </label>
          <p id="floating-hint" className="hint">
            {t.avatar.floatingHint}
          </p>
        </>
      )}
    </section>
  );
}
