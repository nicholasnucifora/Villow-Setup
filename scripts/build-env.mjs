import { existsSync } from "node:fs";
import { delimiter, dirname, resolve } from "node:path";
export function buildEnv(root) {
  const env = { ...process.env };
  const original = process.env.PATH ?? process.env.Path ?? "";
  for (const key of Object.keys(env))
    if (key.toLowerCase() === "path") delete env[key];
  const local = resolve(root, ".tools/cargo/bin");
  const hasLocal = existsSync(
    resolve(local, process.platform === "win32" ? "cargo.exe" : "cargo"),
  );
  env[process.platform === "win32" ? "Path" : "PATH"] = [
    ...(hasLocal ? [local] : []),
    dirname(process.execPath),
    original,
  ].join(delimiter);
  if (hasLocal) {
    env.CARGO_HOME = resolve(root, ".tools/cargo");
    env.RUSTUP_HOME = resolve(root, ".tools/rustup");
  }
  return env;
}
