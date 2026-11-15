import { existsSync, readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import type { Plugin } from "vite";
import { defineConfig } from "vitest/config";

const here = (path: string) => fileURLToPath(new URL(path, import.meta.url));
// The one version: tauri.conf.json reads it from package.json too.
const { version } = JSON.parse(readFileSync(here("package.json"), "utf8"));
// Feature 009 research R3: Live2D's SDK is compiled in only when the owner has put it in vendor/cubism/.
const withLive2d = existsSync(here("vendor/cubism/Framework/src"));
// The framework fetches its WebGL shaders by file name from one folder; they are served at this fixed path.
const SHADERS = "live2d-shaders";
const shaderDir = here("vendor/cubism/Framework/Shaders/WebGL");

const live2dSwitch = (): Plugin => ({
  name: "live2d-switch",
  resolveId: (id) =>
    id === "virtual:live2d"
      ? here(withLive2d ? "src/avatar/live2d/index.ts" : "src/avatar/live2d-absent.ts")
      : null,
  configureServer(server) {
    if (!withLive2d) return;
    server.middlewares.use(`/${SHADERS}/`, (req, res, next) => {
      const name = (req.url ?? "").split("?")[0].replace(/^\//, "");
      if (!readdirSync(shaderDir).includes(name)) return next();
      res.setHeader("content-type", "text/plain");
      res.end(readFileSync(`${shaderDir}/${name}`));
    });
  },
  generateBundle() {
    if (!withLive2d) return;
    for (const name of readdirSync(shaderDir)) {
      this.emitFile({
        type: "asset",
        fileName: `${SHADERS}/${name}`,
        source: readFileSync(`${shaderDir}/${name}`),
      });
    }
  },
});

// Tauri expects a fixed port and must see Rust errors in the console.
export default defineConfig({
  plugins: [react(), live2dSwitch()],
  define: { __APP_VERSION__: JSON.stringify(version) },
  resolve: { alias: withLive2d ? { "@cubism/framework": here("vendor/cubism/Framework/src") } : {} },
  clearScreen: false,
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**", "**/target/**"] } },
  test: {
    environment: "jsdom",
    setupFiles: ["src/test-setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
