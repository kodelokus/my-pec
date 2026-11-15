// Why this window shows the still portrait, for settings (avatar R4). It stays in the window: the
// core can't know a draw failed, and the conversation never hears of it.

import { useSyncExternalStore } from "react";
import type { RendererProblem } from "./renderer";

/** The reason belongs to the look it was about; another look starts with none. */
type Store = { problem: RendererProblem | null; look: string | null; attempt: number };

let current: Store = { problem: null, look: null, attempt: 0 };
const listeners = new Set<() => void>();

function set(next: Store) {
  current = next;
  for (const listener of listeners) listener();
}

/** The reason this look couldn't be drawn in this window, if it couldn't. */
export const problemFor = (look: string) => (current.look === look ? current.problem : null);

export const reportProblem = (problem: RendererProblem, look: string) => set({ ...current, problem, look });

/** avatar FR-015, FR-017: a failed character is tried again only when the person asks. */
export const tryAgain = () => set({ problem: null, look: null, attempt: current.attempt + 1 });

export const useRendererProblem = () =>
  useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => current,
  );
