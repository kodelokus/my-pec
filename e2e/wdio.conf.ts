import { execFileSync } from "node:child_process";
import { rmSync } from "node:fs";
import type { Server } from "node:http";
import { join } from "node:path";
import { APP, APP_ENV, REPLAY_PORT } from "./helpers";
import { startReplayServer } from "./replay-server";

let server: Server;

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./specs/**/*.spec.ts"],
  // Specs share one scratch root and one credential entry, so they run one at a time.
  maxInstances: 1,
  capabilities: [{ browserName: "tauri", "tauri:options": { application: APP } } as WebdriverIO.Capabilities],
  services: [["@wdio/tauri-service", { driverProvider: "external", env: APP_ENV }]],
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: { timeout: 120_000 },
  waitforTimeout: 15_000,
  outputDir: join(import.meta.dirname, "logs"),

  onPrepare: async () => {
    server = await startReplayServer(REPLAY_PORT);
  },
  // Each spec starts from a clean state: an empty data root, no stored key, and a replay server that accepts it.
  beforeSession: async () => {
    await fetch(`http://127.0.0.1:${REPLAY_PORT}/__reset`, { method: "POST" });
    rmSync(APP_ENV.MYPEC_E2E_ROOT, { recursive: true, force: true });
    try {
      execFileSync("cmdkey", ["/delete:claude-api-key.MyPec-e2e"], { stdio: "ignore" });
    } catch {
      // No entry left by an earlier spec.
    }
  },
  onComplete: () => {
    server?.close();
  },
};
