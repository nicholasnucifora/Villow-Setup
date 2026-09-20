import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { buildEnv } from "./build-env.mjs";
const root = resolve(import.meta.dirname, "..");
const env = buildEnv(root);
const r = spawnSync(
  process.execPath,
  [
    resolve(root, "node_modules/@tauri-apps/cli/tauri.js"),
    ...process.argv.slice(2),
  ],
  { cwd: root, env, stdio: "inherit" },
);
process.exit(r.status ?? 1);
