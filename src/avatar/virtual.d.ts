// Resolved by the Vite plugin in vite.config.ts to src/avatar/live2d/ or to live2d-absent.ts (avatar R3).
declare module "virtual:live2d" {
  export const loadLive2D: (() => Promise<import("./renderer").Live2D>) | null;
}
