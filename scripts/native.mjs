import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { buildEnv } from "./build-env.mjs";
const root = resolve(import.meta.dirname, "..");
const local = resolve(root, ".tools/cargo/bin/cargo.exe");
const env = buildEnv(root);
const r = spawnSync(
  existsSync(local) ? local : "cargo",
  process.argv.slice(2),
  { cwd: resolve(root, "src-tauri"), env, stdio: "inherit" },
);
if (r.error) {
  console.error("Rust toolchain unavailable. See docs/development.md.");
}
process.exit(r.status ?? 1);
