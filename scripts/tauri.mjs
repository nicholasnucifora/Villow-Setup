import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { readFileSync } from "node:fs";
import { buildEnv } from "./build-env.mjs";
import { desktopProfile } from "./desktop-profile.mjs";
const root = resolve(import.meta.dirname, "..");
const args = process.argv.slice(2);
let profile;
try {
  profile = desktopProfile(
    args,
    buildEnv(root),
    args.includes("--unsigned-alpha")
      ? JSON.parse(readFileSync(resolve(root, "src-tauri/trust.json"), "utf8"))
      : undefined,
  );
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
const r = spawnSync(
  process.execPath,
  [resolve(root, "node_modules/@tauri-apps/cli/tauri.js"), ...profile.args],
  { cwd: root, env: profile.env, stdio: "inherit" },
);
process.exit(r.status ?? 1);
