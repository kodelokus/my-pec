import { useState } from "react";
import { compactConversation } from "../api/commands";
import { Confirm } from "../Confirm";
import { t } from "../strings";

/** "Compact now" with an optional focus note the summary gives weight to (compaction FR-024). */
export function CompactNow(props: { onCompacted: () => void; onClose: () => void }) {
  const [focus, setFocus] = useState("");
  const [note, setNote] = useState<string | null>(null);

  async function compact() {
    setNote(null);
    try {
      const result = await compactConversation(focus.trim() || null, true);
      switch (result.outcome.kind) {
        case "Compacted":
          return props.onCompacted();
        case "NothingToCompact":
          return setNote(t.chat.nothingToCompact);
        case "Stopped":
          return props.onClose();
        case "Failed":
          return setNote(t.chat.compactFailed);
      }
    } catch {
      setNote(t.chat.compactFailed);
    }
  }

  return (
    <Confirm
      message={t.chat.compactNowMessage}
      confirm={t.chat.compactNow}
      onConfirm={() => void compact()}
      onCancel={props.onClose}
    >
      <label className="field">
        {t.chat.focusLabel}
        <input value={focus} maxLength={500} onChange={(e) => setFocus(e.target.value)} />
      </label>
      {note && <p role="status">{note}</p>}
    </Confirm>
  );
}
