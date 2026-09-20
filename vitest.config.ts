import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";
export default defineConfig({
  root: realpathSync.native(fileURLToPath(new URL(".", import.meta.url))),
  envDir: "./.no-environment-files",
  css: { postcss: { plugins: [] } },
  plugins: [react()],
  test: {
    environment: "jsdom",
    include: ["tests/**/*.spec.ts", "tests/**/*.spec.tsx"],
    restoreMocks: true,
  },
});
