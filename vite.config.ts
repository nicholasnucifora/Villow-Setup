import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";

export default defineConfig({
  root: realpathSync.native(fileURLToPath(new URL(".", import.meta.url))),
  plugins: [react()],
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    fs: { strict: true },
  },
  envDir: "./.no-environment-files",
  css: { postcss: { plugins: [] } },
  build: { target: "es2022", sourcemap: false },
});
