import "@testing-library/jest-dom/vitest";
import { mockIPC } from "@tauri-apps/api/mocks";
import { cleanup } from "@testing-library/react";
import { afterEach, beforeEach } from "vitest";
import { options } from "./testing";

// The character appears on most screens. The avatar commands a test doesn't mock itself answer as an
// idle Pip with its still portrait, and events can be listened to. Any other command fails, as with no backend.
const answers: Record<string, unknown> = {
  avatar_state: { activity: "idle", mood: "neutral", voice: 0, look: 0 },
  avatar_manifest: { look: { type: "static", character: "pip" }, problem: null },
  floating_settings: { on: false },
  setup_options: options,
};

beforeEach(() =>
  mockIPC(
    (cmd) => {
      if (cmd in answers) return answers[cmd];
      throw new Error(`no backend for ${cmd}`);
    },
    { shouldMockEvents: true },
  ),
);

afterEach(cleanup);
