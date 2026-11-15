// Writes a long conversation file for performance checks (chat checks, chat SC-005).
// Usage: node scripts/make-transcript.mjs <workspace folder> <message count>
// The conversation file must not exist yet; this script never overwrites one.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [workspace, countArg] = process.argv.slice(2);
const count = Number(countArg);
if (!workspace || !Number.isInteger(count) || count < 1) {
  console.error("Usage: node scripts/make-transcript.mjs <workspace folder> <message count>");
  process.exit(1);
}
const file = join(workspace, "sessions", "main.jsonl");
if (existsSync(file)) {
  console.error(`${file} already exists. Move it aside first.`);
  process.exit(1);
}

const read = (name) => {
  try {
    return readFileSync(join(workspace, name), "utf8");
  } catch {
    return "";
  }
};
const start = Date.now() - count * 60_000;
const files = Object.fromEntries(["IDENTITY.md", "MEMORY.md", "SOUL.md", "USER.md"].map((n) => [n, read(n)]));
const lines = [
  { id: `${start}-0`, at: start, type: "context", model: "claude-opus-5-5", files, harness: "chat-1" },
];
for (let i = 0; i < count; i++) {
  const at = start + (i + 1) * 60_000;
  const text =
    i % 2 === 0
      ? `Message ${i} from the user.`
      : `Reply ${i}, with **some** Markdown and a list:\n\n- one\n- two`;
  const common = { id: `${at}-0`, at, type: "message", content: [{ type: "text", text }] };
  lines.push(
    i % 2 === 0
      ? { ...common, role: "user", provenance: "user", text, zone: "UTC" }
      : {
          ...common,
          role: "assistant",
          provenance: "companion",
          state: "complete",
          model: "claude-opus-5-5",
        },
  );
}
mkdirSync(join(workspace, "sessions"), { recursive: true });
writeFileSync(file, `${lines.map((l) => JSON.stringify(l)).join("\n")}\n`);
console.log(`Wrote ${count} messages to ${file}`);
