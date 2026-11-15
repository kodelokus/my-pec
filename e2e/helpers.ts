import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

export const APP = join(import.meta.dirname, "../target/debug/my-pec.exe");
export const REPLAY_PORT = 4545;

/** What the app under test reads; only builds with the `e2e` feature honor it (src-tauri/src/lib.rs). */
export const APP_ENV = {
  MYPEC_E2E_ROOT: join(tmpdir(), "my-pec-e2e"),
  MYPEC_E2E_API_URL: `http://127.0.0.1:${REPLAY_PORT}`,
};

export const VALID_KEY = "sk-ant-e2e-valid-0123456789abcdef";
export const REJECTED_KEY = "sk-ant-e2e-rejected-0123456789abcdef";

export const workspaceFile = (name: string) =>
  readFileSync(join(APP_ENV.MYPEC_E2E_ROOT, "workspace", name), "utf8");

/** The files in a workspace folder, or none when it doesn't exist. */
export function workspaceFiles(folder: string): string[] {
  try {
    return readdirSync(join(APP_ENV.MYPEC_E2E_ROOT, "workspace", folder));
  } catch {
    return [];
  }
}

/** Writes a workspace file as an outside editor would. */
export function writeWorkspaceFile(name: string, text: string) {
  const path = join(APP_ENV.MYPEC_E2E_ROOT, "workspace", name);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, text);
}

export const button = (name: string) => $(`button=${name}`);

export async function submitKey(key: string) {
  await $("aria/Claude API key").setValue(key);
  await button("Check key").click();
}

/** Waits for Next to be enabled first: it stays disabled until the key check has finished. */
export async function next(times = 1) {
  for (let i = 0; i < times; i++) {
    await button("Next").waitForEnabled();
    await button("Next").click();
  }
}
