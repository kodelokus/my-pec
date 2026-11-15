// The Live2D renderer on the Cubism Web Framework. Imported only by index.ts, after Cubism Core has loaded: the
// framework reads Core's global as its modules load (avatar R3). Its look is checked by hand.

import { CubismDefaultParameterId } from "@cubism/framework/cubismdefaultparameterid";
import { CubismModelSettingJson } from "@cubism/framework/cubismmodelsettingjson";
import { BreathParameterData, CubismBreath } from "@cubism/framework/effect/cubismbreath";
import { CubismEyeBlink } from "@cubism/framework/effect/cubismeyeblink";
import { CubismFramework } from "@cubism/framework/live2dcubismframework";
import { CubismMatrix44 } from "@cubism/framework/math/cubismmatrix44";
import { CubismUserModel } from "@cubism/framework/model/cubismusermodel";
import type { ACubismMotion } from "@cubism/framework/motion/acubismmotion";
import type { AvatarLook } from "../../bindings/AvatarLook";
import type { AvatarRenderer, Live2D, ReadFile, Shown } from "../renderer";

/** Where vite.config.ts serves the framework's shaders, from the page's root. */
const SHADERS = "/live2d-shaders/";
/** avatar FR-020: at most this many frames a second. */
const FRAME_MS = 1000 / 30;

let started = false;

function startFramework() {
  if (started) return;
  CubismFramework.startUp();
  CubismFramework.initialize();
  started = true;
}

const id = (name: string) => CubismFramework.getIdManager().getId(name);
const P = CubismDefaultParameterId;

/** The parent folder of a path in the manifest, for the model's own relative paths. */
const folder = (path: string) => path.slice(0, path.lastIndexOf("/") + 1);

class Model extends CubismUserModel {
  expressions = new Map<string, ACubismMotion>();
  motions = new Map<string, ACubismMotion[]>();

  async read(look: Extract<AvatarLook, { type: "live2d" }>, read: ReadFile, gl: WebGLRenderingContext) {
    const bytes = async (path: string) => (await read(path)).arrayBuffer();
    const base = folder(look.model);
    const settingBytes = await bytes(look.model);
    const setting = new CubismModelSettingJson(settingBytes, settingBytes.byteLength);
    const moc = await bytes(base + setting.getModelFileName());
    this.loadModel(moc);
    if (!this._model) throw "damaged";
    for (let i = 0; i < setting.getExpressionCount(); i++) {
      const buffer = await bytes(base + setting.getExpressionFileName(i));
      const name = setting.getExpressionName(i);
      this.expressions.set(name, this.loadExpression(buffer, buffer.byteLength, name));
    }
    // A motion must know which parameters blinking and lip sync drive, or updating it fails.
    const ids = (count: number, at: (i: number) => ReturnType<typeof id>) =>
      Array.from({ length: count }, (_, i) => at(i));
    const blink = ids(setting.getEyeBlinkParameterCount(), (i) => setting.getEyeBlinkParameterId(i));
    const lipSync = ids(setting.getLipSyncParameterCount(), (i) => setting.getLipSyncParameterId(i));
    // Only the groups the manifest names are loaded.
    for (const group of new Set(Object.values(look.activities).map((m) => m?.motionGroup ?? ""))) {
      const loaded: ACubismMotion[] = [];
      for (let i = 0; i < setting.getMotionCount(group); i++) {
        const buffer = await bytes(base + setting.getMotionFileName(group, i));
        const motion = this.loadMotion(buffer, buffer.byteLength, `${group}_${i}`);
        motion.setEffectIds(blink, lipSync);
        loaded.push(motion);
      }
      if (loaded.length > 0) this.motions.set(group, loaded);
    }
    if (setting.getPhysicsFileName()) {
      const buffer = await bytes(base + setting.getPhysicsFileName());
      this.loadPhysics(buffer, buffer.byteLength);
    }
    if (setting.getPoseFileName()) {
      const buffer = await bytes(base + setting.getPoseFileName());
      this.loadPose(buffer, buffer.byteLength);
    }
    if (setting.getEyeBlinkParameterCount() > 0) this._eyeBlink = CubismEyeBlink.create(setting);
    this._breath = CubismBreath.create();
    this._breath.setParameters([new BreathParameterData(id(P.ParamBreath), 0.5, 0.5, 3.2345, 1)]);

    this.createRenderer(gl.drawingBufferWidth, gl.drawingBufferHeight);
    this.getRenderer().startUp(gl);
    this.getRenderer().setIsPremultipliedAlpha(true);
    for (let i = 0; i < setting.getTextureCount(); i++) {
      const blob = await read(base + setting.getTextureFileName(i));
      this.getRenderer().bindTexture(i, await texture(gl, blob));
    }
  }

  private mood = "";
  private group = "";

  /** The mapped expression, or the neutral face (architecture 9.3). */
  showMood(expression: string) {
    if (expression === this.mood) return;
    this.mood = expression;
    const motion = this.expressions.get(expression);
    if (motion) this._expressionManager.startMotion(motion, false);
    else this._expressionManager.stopAllMotions();
  }

  /** The activity's motion group, looped; none while holding still (avatar FR-021). */
  play(group: string) {
    if (group === this.group) return;
    this.group = group;
    this._motionManager.stopAllMotions();
  }

  frame(seconds: number, shown: Shown, mouth: number) {
    const model = this._model;
    const motions = this.motions.get(this.group) ?? [];
    if (motions.length > 0 && this._motionManager.isFinished()) {
      this._motionManager.startMotionPriority(motions[Math.floor(Math.random() * motions.length)], false, 2);
    }
    this._dragManager.update(seconds);
    model.loadParameters();
    const moved = !shown.still && this._motionManager.updateMotion(model, seconds);
    model.saveParameters();
    if (shown.activity === "sleeping") {
      // Eyes closed while asleep, also when holding still (avatar FR-021).
      model.setParameterValueById(id(P.ParamEyeLOpen), 0);
      model.setParameterValueById(id(P.ParamEyeROpen), 0);
    } else if (!moved && !shown.still) {
      this._eyeBlink?.updateParameters(model, seconds);
    }
    this._expressionManager.updateMotion(model, seconds);
    if (!shown.still) {
      const [x, y] = [this._dragManager.getX(), this._dragManager.getY()];
      model.addParameterValueById(id(P.ParamAngleX), x * 30);
      model.addParameterValueById(id(P.ParamAngleY), y * 30);
      model.addParameterValueById(id(P.ParamEyeBallX), x);
      model.addParameterValueById(id(P.ParamEyeBallY), y);
      if (shown.activity === "thinking") {
        // Architecture 9.3: eyes drift up and to the side.
        model.addParameterValueById(id(P.ParamEyeBallX), 0.6);
        model.addParameterValueById(id(P.ParamEyeBallY), 0.6);
      }
      this._breath.updateParameters(model, shown.activity === "sleeping" ? seconds / 2 : seconds);
      this._physics?.evaluate(model, seconds);
      model.setParameterValueById(id(P.ParamMouthOpenY), mouth);
    }
    this._pose?.updateParameters(model, seconds);
    model.update();
  }
}

async function texture(gl: WebGLRenderingContext, blob: Blob): Promise<WebGLTexture> {
  const image = await createImageBitmap(blob, { premultiplyAlpha: "premultiply" });
  const tex = gl.createTexture();
  if (!tex) throw "noGraphics";
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
  gl.generateMipmap(gl.TEXTURE_2D);
  return tex;
}

export const live2d: Live2D = (canvas, onLost) => {
  const gl = canvas.getContext("webgl", { premultipliedAlpha: true, alpha: true });
  const model = new Model();
  let look: Extract<AvatarLook, { type: "live2d" }> | null = null;
  let shown: Shown = { activity: "idle", mood: "neutral", voice: 0, still: false };
  let [mouth, lastVoice, lastVoiceAt, lastFrame, frame] = [0, 0, 0, 0, 0];
  let ready = false;

  const lost = (e: Event) => {
    e.preventDefault();
    stop();
    onLost();
  };
  canvas.addEventListener("webglcontextlost", lost);

  function stop() {
    cancelAnimationFrame(frame);
    frame = 0;
  }

  // avatar FR-019, FR-020: drawn only while the page is visible, at most 30 frames a second.
  function draw(now: number) {
    frame = requestAnimationFrame(draw);
    if (!gl || now - lastFrame < FRAME_MS) return;
    const seconds = lastFrame ? (now - lastFrame) / 1000 : 0;
    lastFrame = now;
    // avatar FR-005: the mouth follows how fast the reply's text arrives.
    const opening =
      shown.activity === "speaking" && now - lastVoiceAt < 250 ? 0.6 + 0.4 * Math.sin(now / 60) : 0;
    mouth += (opening - mouth) * 0.5;
    model.frame(seconds, shown, mouth);
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    // The model in its own coordinates, fitted to the canvas (as Live2D's samples draw it).
    const projection = new CubismMatrix44();
    projection.scale(1, canvas.width / canvas.height);
    projection.multiplyByMatrix(model.getModelMatrix());
    model.getRenderer().setMvpMatrix(projection);
    model.getRenderer().setRenderState(null as never, [0, 0, canvas.width, canvas.height]);
    model.getRenderer().drawModel(SHADERS);
  }

  const visibility = () => {
    if (document.visibilityState === "hidden") stop();
    else if (ready && !frame) frame = requestAnimationFrame(draw);
  };
  document.addEventListener("visibilitychange", visibility);

  const renderer: AvatarRenderer = {
    async load(next, read) {
      if (!gl) throw "noGraphics";
      if (next.type !== "live2d") throw "damaged";
      look = next;
      startFramework();
      await model.read(next, read, gl);
      ready = true;
      visibility();
    },
    setState(next) {
      if (next.voice !== lastVoice) {
        lastVoice = next.voice;
        lastVoiceAt = performance.now();
      }
      shown = next;
      if (!look) return;
      model.showMood(look.moods[next.mood]?.expression ?? "");
      model.play(next.still ? "" : (look.activities[next.activity]?.motionGroup ?? ""));
    },
    pointer(x, y) {
      if (!shown.still) model.setDragging(x, -y);
    },
    dispose() {
      stop();
      ready = false;
      document.removeEventListener("visibilitychange", visibility);
      canvas.removeEventListener("webglcontextlost", lost);
      model.release();
    },
  };
  return renderer;
};
