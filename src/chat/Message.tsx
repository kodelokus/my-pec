import { type ReactNode, useState } from "react";
import Markdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { openLink } from "../api/commands";
import type { ChatMessage } from "../bindings/ChatMessage";
import type { MemoryMark } from "../bindings/MemoryMark";
import { Line } from "../Line";
import { t } from "../strings";

// chat R13: no raw HTML (react-markdown escapes it), links open only on click, remote images never load.
const components: Components = {
  a: ({ href, children }) => (
    <a
      href={href}
      onClick={(e) => {
        e.preventDefault();
        if (href) void openLink(href);
      }}
    >
      {children}
    </a>
  ),
  img: ({ src, alt }) =>
    typeof src === "string" && src ? (
      <a
        href={src}
        onClick={(e) => {
          e.preventDefault();
          void openLink(src);
        }}
      >
        {t.chat.image(alt ?? "")}
      </a>
    ) : (
      <span>{t.chat.image(alt ?? "")}</span>
    ),
};

/** A reply's text as sanitized Markdown. Also used for the reply while it streams. */
export function Reply(props: { text: string }) {
  return (
    <Markdown remarkPlugins={[remarkGfm]} components={components}>
      {props.text}
    </Markdown>
  );
}

/** Markdown that needs one bubble: lists, headings, quotes, tables and code. */
const STRUCTURED = /^\s*([-*+]\s|\d+[.)]\s|#|>|\||```|~~~)/m;
/** More parts than this read as one long answer, not as a few short messages. */
const MAX_PARTS = 4;

/** A chat reply's parts, one bubble each: plain paragraphs, the way people send a few short messages in a row
 * (the harness's "How you talk"). */
export function parts(text: string): string[] {
  const split = text
    .split(/\n\s*\n/)
    .map((p) => p.trim())
    .filter(Boolean);
  return STRUCTURED.test(text) || split.length < 2 || split.length > MAX_PARTS ? [text] : split;
}

/** A companion reply as one bubble per part; `after` goes in the last one. */
export function Replies(props: { text: string; characterId: string; after?: ReactNode }) {
  const all = parts(props.text);
  return all.map((part, i) => (
    // A reply's parts come from its text alone, so their position is a stable key.
    // biome-ignore lint/suspicious/noArrayIndexKey: see above
    <Line key={i} small follow={i > 0} characterId={props.characterId}>
      <Reply text={part} />
      {i === all.length - 1 && props.after}
    </Line>
  ));
}

type OpenMemory = (path: string, line: number | null) => void;

/** What the companion wrote to memory during a turn, each opening the memory browser there (memory FR-008). */
export function Marks(props: { marks: MemoryMark[]; onOpen?: OpenMemory }) {
  if (props.marks.length === 0) return null;
  return (
    <ul className="marks" aria-label={t.chat.marksLabel}>
      {props.marks.map((mark, i) => (
        // A turn's marks never reorder, so their position is a stable key.
        // biome-ignore lint/suspicious/noArrayIndexKey: see above
        <li key={i} className={mark.ok ? undefined : "failed"}>
          {props.onOpen && mark.ok ? (
            <button type="button" className="link" onClick={() => props.onOpen?.(mark.path, mark.line)}>
              {t.chat.mark(mark)}
            </button>
          ) : (
            <span>{t.chat.mark(mark)}</span>
          )}
        </li>
      ))}
    </ul>
  );
}

/** Where older turns were summarized; opens to show the summary the companion continued with (compaction FR-025). */
function Compacted(props: { summary: string }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="compacted">
      <div className="divider">
        <button type="button" className="link" aria-expanded={open} onClick={() => setOpen((o) => !o)}>
          {open ? t.chat.hideSummary : t.chat.compacted}
        </button>
      </div>
      {open && (
        <div className="compacted-summary">
          <Reply text={props.summary} />
        </div>
      )}
    </div>
  );
}

/** One entry of the conversation (chat FR-004). */
export function Message(props: { message: ChatMessage; characterId: string; onOpenMemory?: OpenMemory }) {
  const { message } = props;
  switch (message.kind) {
    case "divider":
      return (
        <div className="divider">
          <span>{t.chat.divider}</span>
        </div>
      );
    case "compacted":
      return <Compacted summary={message.text} />;
    case "note":
      return <p className="chat-note">{t.chat.note(message.text)}</p>;
    case "user":
      return <div className="bubble user">{message.text}</div>;
    case "companion":
      return (
        <Replies
          text={message.text}
          characterId={props.characterId}
          after={
            <>
              {message.state === "stopped" && <span className="badge">{t.chat.stopped}</span>}
              {message.state === "incomplete" && <span className="badge">{t.chat.incomplete}</span>}
              <Marks marks={message.memory} onOpen={props.onOpenMemory} />
            </>
          }
        />
      );
  }
}
