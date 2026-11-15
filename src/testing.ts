// Shared data for component tests, shaped like what the core returns.

import type { AppStatus } from "./bindings/AppStatus";
import type { ChatState } from "./bindings/ChatState";
import type { SetupDraft } from "./bindings/SetupDraft";
import type { SetupOptions } from "./bindings/SetupOptions";

export const options: SetupOptions = {
  templates: [
    { id: "warm-friend", title: "Warm friend", description: "Kind.", text: "# Soul\n\nBe warm." },
    { id: "calm-mentor", title: "Calm mentor", description: "Steady.", text: "# Soul\n\nBe calm." },
    { id: "blank", title: "Blank", description: "Empty.", text: "" },
  ],
  characters: [
    { id: "pip", name: "Pip" },
    { id: "ozzy", name: "Ozzy" },
  ],
  soulBudgetChars: 8000,
  defaultName: "Pec",
  nameMaxChars: 40,
};

export const draft = (patch: Partial<SetupDraft> = {}): SetupDraft => ({
  step: "Key",
  soul: { templateId: "warm-friend", text: "# Soul\n\nBe warm.", name: "Steve" },
  characterId: "pip",
  about: { callMe: null, language: null },
  ...patch,
});

export const completeStatus = (patch: Partial<AppStatus> = {}): AppStatus => ({
  setup: { state: "Complete" },
  key: { status: "Present", lastFour: "WXYZ" },
  companion: { name: "Steve", characterId: "pip" },
  existingWorkspace: false,
  recreatedFiles: [],
  callMe: null,
  ...patch,
});

export const chatIdle = (patch: Partial<ChatState> = {}): ChatState => ({
  replying: false,
  canRegenerate: false,
  pendingRetry: null,
  firstHelloPending: false,
  compactionDue: false,
  contextFull: false,
  modelChangePending: false,
  shortened: [],
  skippedLines: 0,
  contextOutdated: false,
  updatePending: false,
  memoryFull: false,
  autoRecall: true,
  dreaming: false,
  sound: true,
  ...patch,
});
