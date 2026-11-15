// The Live2D switch's target when vendor/cubism/ is present (avatar R3). Cubism Core is a classic
// script that defines a global the framework reads as its modules load, so it is loaded first, once per window,
// and only then is the renderer imported. Nothing of Live2D runs until a character is actually shown.

import coreUrl from "../../../vendor/cubism/Core/live2dcubismcore.min.js?url";
import type { Live2D } from "../renderer";

let ready: Promise<Live2D> | null = null;

export const loadLive2D = (): Promise<Live2D> => {
  ready ??= new Promise<void>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = coreUrl;
    script.onload = () => resolve();
    script.onerror = () => reject("damaged");
    document.head.append(script);
  }).then(() => import("./model").then((m) => m.live2d));
  return ready;
};
