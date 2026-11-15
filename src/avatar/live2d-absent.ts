// The Live2D switch's target when this build has no Cubism SDK (avatar R3, FR-018).

import type { Live2D } from "./renderer";

export const loadLive2D: (() => Promise<Live2D>) | null = null;
