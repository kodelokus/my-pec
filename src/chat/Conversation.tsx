import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import {
  chatHistory,
  chatState,
  compactConversation,
  regenerateReply,
  retryReply,
  sendMessage,
  startContextCarrying,
  startFirstHello,
  startFreshContext,
  stopReply,
} from "../api/commands";
import type { ChatEvent } from "../bindings/ChatEvent";
import type { ChatMessage } from "../bindings/ChatMessage";
import type { ChatState } from "../bindings/ChatState";
import type { MemoryMark } from "../bindings/MemoryMark";
import type { SendRefusal } from "../bindings/SendRefusal";
import { Line } from "../Line";
import { t } from "../strings";
import { CompactNow } from "./CompactNow";
import { Composer } from "./Composer";
import { chime } from "./chime";
import { Marks, Message, Replies } from "./Message";

const PAGE = 50;

/** The turn being written: its text so far, whether the provider is busy, and what it wrote to memory (chat FR-003, FR-025, FR-008). */
type Live = { text: string; waiting: number | null; memory: MemoryMark[] };
type Start = (onEvent: (event: ChatEvent) => void) => Promise<SendRefusal | null>;

/** Older messages stay; the latest page replaces everything from its first message on. */
function merge(previous: ChatMessage[], latest: ChatMessage[]): ChatMessage[] {
  const first = latest[0];
  return first ? [...previous.filter((m) => m.at < first.at), ...latest] : previous;
}

/** memory FR-011, FR-015: buttons that open over-budget files in the memory browser. */
function OpenIn(props: { files: string[]; onOpen?: (path: string, line: number | null) => void }) {
  const { onOpen } = props;
  if (!onOpen) return null;
  return props.files.map((path) => (
    <button key={path} type="button" className="link" onClick={() => onOpen(path, null)}>
      {t.chat.openInMemory(path)}
    </button>
  ));
}

/** chat FR-001 to FR-033: the one conversation with the companion. */
export function Conversation(props: {
  name: string;
  characterId: string;
  hasKey: boolean;
  /** Opens the memory browser at a file and line, from a memory mark (memory FR-008). */
  onOpenMemory?: (path: string, line: number | null) => void;
  /** The main view's lines about the key and the connection, said after the latest message (PER-6). */
  notices?: ReactNode;
  draft: string;
  onDraft: (text: string) => void;
  onOpenSettings: () => void;
  /** An extension's change, as it happens. */
  onMark?: (mark: MemoryMark) => void;
}) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [chat, setChat] = useState<ChatState | null>(null);
  const [live, setLive] = useState<Live | null>(null);
  const [refusal, setRefusal] = useState<SendRefusal | null>(null);
  const [announcement, setAnnouncement] = useState("");
  const [loadingEarlier, setLoadingEarlier] = useState(false);
  const log = useRef<HTMLDivElement>(null);
  const pinned = useRef(true);
  const greeted = useRef(false);
  const onMark = useRef(props.onMark);
  onMark.current = props.onMark;
  // compaction R2: the core says when a compaction is due; the view asks for it, once at a time.
  const [compacting, setCompacting] = useState(false);
  const compactingRef = useRef(false);
  const [compactOpen, setCompactOpen] = useState(false);

  const refresh = useCallback(async () => {
    const [page, state] = await Promise.all([chatHistory(null, PAGE), chatState()]);
    setMessages((previous) => merge(previous, page.messages));
    setHasMore((more) => more || page.hasMore);
    setChat(state);
    return state;
  }, []);

  const compactIfDue = useCallback(
    async (state: ChatState) => {
      if (!state.compactionDue || compactingRef.current) return;
      compactingRef.current = true;
      setCompacting(true);
      try {
        const result = await compactConversation(null, false);
        if (result.outcome.kind === "Compacted") await refresh();
        else setChat((c) => (c && !c.replying ? result.state : c));
      } catch {
        // Busy or unavailable: the next turn boundary tries again (compaction FR-023).
      } finally {
        compactingRef.current = false;
        setCompacting(false);
      }
    },
    [refresh],
  );

  // Newer rules, and extensions turned on after this context began, arrive with the next one, kept as a summary when
  // there is something to compact (Principle III).
  const catchUp = async () => {
    compactingRef.current = true;
    setCompacting(true);
    try {
      const result = await compactConversation(null, true);
      if (result.outcome.kind === "NothingToCompact") await startContextCarrying();
      await refresh();
    } catch {
      // Busy or unavailable: the note stays, so the person can try again.
    } finally {
      compactingRef.current = false;
      setCompacting(false);
    }
  };

  const run = useCallback(
    async (start: Start) => {
      setRefusal(null);
      const onEvent = (event: ChatEvent) => {
        switch (event.kind) {
          case "Started": {
            const sent = event.userMessage;
            if (sent) setMessages((m) => [...m, sent]);
            setLive({ text: "", waiting: null, memory: [] });
            setChat((c) => c && { ...c, replying: true, pendingRetry: null, canRegenerate: false });
            break;
          }
          case "Delta":
            setLive((l) => ({ text: (l?.text ?? "") + event.text, waiting: null, memory: l?.memory ?? [] }));
            break;
          case "Waiting":
            // The text of the failed call was replaced; earlier calls of the turn stay.
            setLive((l) => ({ text: event.text, waiting: event.seconds, memory: l?.memory ?? [] }));
            break;
          case "Compacting":
            setCompacting(true);
            break;
          case "MemoryWritten":
            setLive((l) => l && { ...l, memory: [...l.memory, event.mark] });
            onMark.current?.(event.mark);
            break;
          case "Finished":
          case "Failed":
            setLive(null);
            if (!compactingRef.current) setCompacting(false);
            setChat(event.state);
            if (event.kind === "Finished" && event.message) {
              setAnnouncement(event.message.text);
              if (event.state.sound) chime();
            }
            void refresh();
            if (event.kind === "Finished") void compactIfDue(event.state);
            break;
        }
      };
      const refused = await start(onEvent);
      if (refused) setRefusal(refused);
    },
    [refresh, compactIfDue],
  );

  useEffect(() => {
    void refresh().then((state) => {
      if (props.hasKey && state.firstHelloPending && !greeted.current) {
        greeted.current = true;
        void run(startFirstHello);
      } else if (props.hasKey) {
        void compactIfDue(state);
      }
    });
  }, [refresh, run, compactIfDue, props.hasKey]);

  // Follow the conversation while the user is at the bottom; leave them be when they scroll up.
  useEffect(() => {
    const el = log.current;
    if (el && pinned.current) el.scrollTop = el.scrollHeight;
  });

  async function earlier() {
    const first = messages[0];
    if (!first || !hasMore || loadingEarlier) return;
    setLoadingEarlier(true);
    try {
      const page = await chatHistory(first.id, PAGE);
      setMessages((m) => [...page.messages, ...m]);
      setHasMore(page.hasMore);
    } finally {
      setLoadingEarlier(false);
    }
  }

  function send() {
    if (chat?.replying || live) return setRefusal("Busy");
    const text = props.draft;
    if (!text.trim()) return setRefusal("Empty");
    // The box empties at once and the message shows when the core takes it; the reply can take a while.
    // A refused or failed send puts the text back.
    props.onDraft("");
    void run((onEvent) =>
      sendMessage(text, onEvent).then(
        (refused) => {
          if (refused) props.onDraft(text);
          return refused;
        },
        (error) => {
          props.onDraft(text);
          throw error;
        },
      ),
    );
  }

  const replying = Boolean(chat?.replying || live);
  const pending = !replying ? chat?.pendingRetry : null;

  return (
    <section aria-label={t.conversationArea} className="card chat">
      <div
        ref={log}
        className="log"
        role="log"
        onScroll={(e) => {
          const el = e.currentTarget;
          pinned.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
          if (el.scrollTop < 80) void earlier();
        }}
      >
        {loadingEarlier && <p className="chat-note">{t.chat.earlier}</p>}
        {messages.length === 0 && !live && props.hasKey && !props.notices && (
          <p className="chat-note">{t.chat.empty(props.name)}</p>
        )}
        {messages.map((m, i) => (
          <div key={m.id} className="entry">
            <Message message={m} characterId={props.characterId} onOpenMemory={props.onOpenMemory} />
            {chat?.canRegenerate && !live && i === messages.length - 1 && m.kind === "companion" && (
              <button type="button" className="subtle" onClick={() => void run(regenerateReply)}>
                {t.chat.regenerate}
              </button>
            )}
          </div>
        ))}
        {live && (
          <div aria-busy="true" className="entry">
            {live.waiting === null && live.text ? (
              <Replies
                text={live.text}
                characterId={props.characterId}
                after={<Marks marks={live.memory} onOpen={props.onOpenMemory} />}
              />
            ) : (
              <Line small characterId={props.characterId}>
                {live.waiting !== null ? (
                  <span className="muted">{t.chat.waiting(live.waiting)}</span>
                ) : (
                  <span className="muted thinking">
                    <span className="spinner" aria-hidden="true" />
                    {t.chat.thinking(props.name)}
                  </span>
                )}
                <Marks marks={live.memory} onOpen={props.onOpenMemory} />
              </Line>
            )}
          </div>
        )}

        {props.notices}
        {pending && (
          <>
            <Line small characterId={props.characterId} role="alert">
              <p>{pending.kind === "Failed" ? t.chat.failures[pending.error] : t.chat.interrupted}</p>
            </Line>
            <div className="row under small">
              <button type="button" onClick={() => void run(retryReply)}>
                {t.retry}
              </button>
              {chat?.contextFull && (
                <button type="button" onClick={() => void startFreshContext().then(() => refresh())}>
                  {t.chat.freshContext}
                </button>
              )}
              {pending.kind === "Failed" &&
                (pending.error === "KeyRejected" || pending.error === "NoCredit") && (
                  <button type="button" onClick={props.onOpenSettings}>
                    {t.enterAnotherKey}
                  </button>
                )}
            </div>
          </>
        )}
        {chat?.contextOutdated && !replying && !pending && (
          <>
            <Line small characterId={props.characterId} role="status">
              <p>{t.chat.outdated}</p>
            </Line>
            <div className="row under small">
              <button type="button" onClick={() => void startFreshContext().then(() => refresh())}>
                {t.chat.freshContext}
              </button>
            </div>
          </>
        )}
        {chat?.updatePending && !chat.contextOutdated && !replying && !pending && !compacting && (
          <>
            <Line small characterId={props.characterId} role="status">
              <p>{t.chat.updatePending}</p>
            </Line>
            <div className="row under small">
              <button type="button" onClick={() => void catchUp()}>
                {t.chat.catchUp}
              </button>
            </div>
          </>
        )}
        {compacting && (
          <p className="chat-note" role="status">
            {t.chat.compacting}
          </p>
        )}
      </div>

      {chat && chat.shortened.length > 0 && (
        <p className="hint">
          {t.chat.shortened(chat.shortened)} <OpenIn files={chat.shortened} onOpen={props.onOpenMemory} />
        </p>
      )}
      {chat?.memoryFull && (
        <p className="hint">
          {t.chat.memoryFull} <OpenIn files={["MEMORY.md"]} onOpen={props.onOpenMemory} />
        </p>
      )}
      {chat && chat.skippedLines > 0 && <p className="hint">{t.chat.skipped(chat.skippedLines)}</p>}

      <div className="row small">
        <button
          type="button"
          className="subtle"
          disabled={replying || compacting || !props.hasKey}
          onClick={() => setCompactOpen(true)}
        >
          {t.chat.compactNow}
        </button>
      </div>
      {compactOpen && (
        <CompactNow
          onCompacted={() => {
            setCompactOpen(false);
            void refresh();
          }}
          onClose={() => setCompactOpen(false)}
        />
      )}
      <Composer
        name={props.name}
        value={props.draft}
        onChange={(text) => {
          props.onDraft(text);
          setRefusal(null);
        }}
        onSend={send}
        onStop={() => void stopReply()}
        replying={replying}
        refusal={refusal}
        hasKey={props.hasKey}
      />
      <div className="visually-hidden" aria-live="polite">
        {announcement}
      </div>
    </section>
  );
}
