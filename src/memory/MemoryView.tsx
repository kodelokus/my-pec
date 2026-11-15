import { useCallback, useEffect, useState } from "react";
import {
  asAppError,
  memoryFiles,
  memoryForget,
  memoryRead,
  memorySearch,
  memoryUndo,
  openWorkspaceFolder,
  startFreshContext,
} from "../api/commands";
import { AvatarView } from "../avatar/AvatarView";
import type { CompanionSummary } from "../bindings/CompanionSummary";
import type { ForgetTarget } from "../bindings/ForgetTarget";
import type { MemoryDoc } from "../bindings/MemoryDoc";
import type { MemoryFile } from "../bindings/MemoryFile";
import type { SearchHit } from "../bindings/SearchHit";
import { Confirm } from "../Confirm";
import { errorMessage, t } from "../strings";
import { MemoryEditor } from "./MemoryEditor";
import { SemanticPanel } from "./SemanticPanel";

/** Where to open the browser: a file, and a line to bring into view. */
export type MemoryTarget = { path: string; line: number | null };

const label = (file: Pick<MemoryFile, "path" | "kind">) =>
  file.kind === "daily" ? t.memory.day(file.path.slice(7, 17)) : t.memory.kinds[file.kind];

/** memory FR-025 to FR-030: the companion's memory, to read, search, edit and undo. Plain, like settings. */
export function MemoryView(props: {
  companion: CompanionSummary | null;
  open: MemoryTarget;
  onClose: () => void;
}) {
  const [files, setFiles] = useState<MemoryFile[]>([]);
  const [target, setTarget] = useState<MemoryTarget>(props.open);
  const [doc, setDoc] = useState<MemoryDoc | null>(null);
  const [editing, setEditing] = useState(false);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [askUndo, setAskUndo] = useState(false);
  const [forgetting, setForgetting] = useState<{ target: ForgetTarget; text: string } | null>(null);
  const [stillSeen, setStillSeen] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fail = (e: unknown) => setError(errorMessage(asAppError(e)));

  const load = useCallback((path: string) => {
    setError(null);
    memoryRead(path).then(setDoc, (e) => {
      setDoc(null);
      setError(errorMessage(asAppError(e)));
    });
    memoryFiles().then(setFiles, (e) => setError(errorMessage(asAppError(e))));
  }, []);

  useEffect(() => load(target.path), [load, target.path]);

  // A memory mark or search result brings its line into view.
  useEffect(() => {
    if (doc && target.line !== null)
      document.getElementById(`line-${target.line}`)?.scrollIntoView?.({ block: "center" });
  }, [doc, target.line]);

  const file = files.find((f) => f.path === target.path) ?? {
    path: target.path,
    kind: "memory" as const,
    budget: null,
  };
  const open = (path: string, line: number | null = null) => {
    setEditing(false);
    setStatus(null);
    setTarget({ path, line });
  };

  // memory FR-027: confirmed first, then gone from the file, search and recall. An entry is only offered in long-term
  // memory, so the open file is always the one to change.
  const forget = (target: ForgetTarget) => {
    if (!doc) return;
    const path = doc.path;
    memoryForget(path, target, doc.hash).then((outcome) => {
      setStatus(t.memory.forgotten);
      setStillSeen(outcome.inCurrentContext);
      if (target.kind === "File") setTarget({ path: "MEMORY.md", line: null });
      else load(path);
    }, fail);
  };

  const undo = (force: boolean) =>
    memoryUndo(target.path, force).then((outcome) => {
      if (outcome.kind === "Edited") return setAskUndo(true);
      setStatus(outcome.kind === "Undone" ? t.memory.undone : t.memory.nothingToUndo);
      load(target.path);
    }, fail);

  return (
    <>
      <header className="app-bar">
        <button type="button" onClick={props.onClose}>
          {t.back}
        </button>
        <h1>{t.memory.title}</h1>
        {props.companion && (
          <>
            <span className="muted">{props.companion.name}</span>
            <AvatarView size={32} />
          </>
        )}
      </header>
      <main className="content">
        <div className="column wide memory">
          <nav aria-label={t.memory.filesLabel} className="card memory-files">
            <ul className="file-list">
              {files.map((f) => (
                <li key={f.path}>
                  <button
                    type="button"
                    className={f.path === target.path ? "file current" : "file"}
                    aria-current={f.path === target.path ? "page" : undefined}
                    disabled={!f.readable}
                    onClick={() => open(f.path)}
                  >
                    {label(f)}
                    {!f.readable && <span className="muted"> · {t.memory.unreadable}</span>}
                  </button>
                </li>
              ))}
            </ul>
            <button type="button" onClick={() => void openWorkspaceFolder().catch(fail)}>
              {t.memory.openFolder}
            </button>
          </nav>

          <section aria-labelledby="memory-doc" className="card memory-doc">
            <SemanticPanel variant="offer" />
            <search>
              <form
                className="row"
                onSubmit={(e) => {
                  e.preventDefault();
                  if (query.trim())
                    memorySearch(query).then(setHits, () => setError(t.memory.searchUnavailable));
                }}
              >
                <input
                  type="search"
                  aria-label={t.memory.searchLabel}
                  value={query}
                  onChange={(e) => {
                    setQuery(e.target.value);
                    if (!e.target.value) setHits(null);
                  }}
                />
                <button type="submit">{t.memory.search}</button>
              </form>
            </search>
            {hits && (
              <div aria-live="polite">
                {hits.length === 0 ? (
                  <p className="muted">{t.memory.noHits}</p>
                ) : (
                  <ul className="hits">
                    {hits.map((h) => (
                      <li key={`${h.path}:${h.firstLine}`}>
                        <button
                          type="button"
                          className="link"
                          onClick={() => {
                            setHits(null);
                            open(h.path, h.firstLine);
                          }}
                        >
                          <strong>{h.day ? t.memory.day(h.day) : t.memory.kinds.memory}</strong> {h.text}
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}

            <h2 id="memory-doc">{label(file)}</h2>
            {doc && !editing && (
              <>
                {doc.entries.length > 0 ? (
                  <ul className="entries">
                    {doc.entries.map((e) => (
                      <li
                        key={e.line}
                        id={`line-${e.line}`}
                        className={e.line === target.line ? "current" : undefined}
                      >
                        <span>{e.text}</span>
                        {e.at && <span className="muted"> · {e.at}</span>}
                        <button
                          type="button"
                          className="subtle"
                          aria-label={`${t.memory.forget}: ${e.text}`}
                          onClick={() =>
                            setForgetting({
                              target:
                                e.id && doc.path === "MEMORY.md"
                                  ? { kind: "Entry", id: e.id }
                                  : { kind: "Line", line: e.line },
                              text: e.text,
                            })
                          }
                        >
                          {t.memory.forget}
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <pre className="doc-text">{doc.text || t.memory.empty}</pre>
                )}
                <div className="row">
                  <button type="button" onClick={() => setEditing(true)}>
                    {t.memory.edit}
                  </button>
                  {doc.canUndo && (
                    <button type="button" onClick={() => void undo(false)}>
                      {t.memory.undo}
                    </button>
                  )}
                  {file.kind === "daily" && (
                    <button
                      type="button"
                      className="danger"
                      onClick={() => setForgetting({ target: { kind: "File" }, text: label(file) })}
                    >
                      {t.memory.forgetDay}
                    </button>
                  )}
                </div>
              </>
            )}
            {doc && editing && (
              <MemoryEditor
                key={doc.hash}
                doc={doc}
                label={label(file)}
                budget={file.budget}
                onCancel={() => setEditing(false)}
                onSaved={() => {
                  setEditing(false);
                  setStatus(t.memory.saved);
                  load(target.path);
                }}
              />
            )}
            {status && (
              <p role="status" className="muted">
                {status}
              </p>
            )}
            {stillSeen && (
              <div role="status" className="alert info">
                <p>{t.memory.stillSeen}</p>
                <button
                  type="button"
                  onClick={() => void startFreshContext().then(() => setStillSeen(false), fail)}
                >
                  {t.chat.freshContext}
                </button>
              </div>
            )}
            {error && (
              <div role="alert" className="alert error">
                <p>{error}</p>
              </div>
            )}
          </section>
        </div>
      </main>
      {forgetting && (
        <Confirm
          message={t.memory.forgetConfirm(forgetting.text)}
          confirm={t.memory.forget}
          danger
          onCancel={() => setForgetting(null)}
          onConfirm={() => {
            setForgetting(null);
            forget(forgetting.target);
          }}
        />
      )}
      {askUndo && (
        <Confirm
          message={t.memory.undoEdited}
          confirm={t.memory.undo}
          danger
          onCancel={() => setAskUndo(false)}
          onConfirm={() => {
            setAskUndo(false);
            void undo(true);
          }}
        />
      )}
    </>
  );
}
