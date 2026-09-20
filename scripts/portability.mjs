import {
  cpSync,
  copyFileSync,
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  existsSync,
  realpathSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve, dirname, delimiter } from "node:path";
import { spawnSync } from "node:child_process";
import { sourceFiles } from "./source-files.mjs";
const root = resolve(import.meta.dirname, ".."),
  temp = realpathSync.native(mkdtempSync(resolve(tmpdir(), "villow-extracted-"))),
  project = resolve(temp, "project");
mkdirSync(project, { recursive: true });
const files = sourceFiles(root);
for (const file of files) {
  mkdirSync(dirname(resolve(project, file)), { recursive: true });
  copyFileSync(resolve(root, file), resolve(project, file));
}
// An installed compiler is a developer prerequisite, not a source dependency.
// Copy only the standalone compiler distribution, not Cargo's registry/cache or
// this project's node_modules. The extracted build has fresh dependency stores.
let compiler = resolve(
  root,
  ".tools/rustup/toolchains/1.90.0-x86_64-pc-windows-msvc",
);
if (!existsSync(compiler)) {
  const discovered = spawnSync(
    "rustup",
    ["which", "--toolchain", "1.90.0", "rustc"],
    { encoding: "utf8" },
  );
  if (discovered.status !== 0)
    throw new Error(
      "Install the pinned Rust 1.90.0 developer toolchain before portability verification.",
    );
  compiler = resolve(dirname(discovered.stdout.trim()), "..");
}
const isolatedCompiler = resolve(temp, "compiler");
const env = {};
for (const k of [
  "SystemRoot",
  "WINDIR",
  "ComSpec",
  "TEMP",
  "TMP",
  "USERPROFILE",
  "LOCALAPPDATA",
  "APPDATA",
  "ProgramFiles",
  "ProgramFiles(x86)",
  "PROGRAMDATA",
  "NUMBER_OF_PROCESSORS",
  "PROCESSOR_ARCHITECTURE",
])
  if (process.env[k]) env[k] = process.env[k];
const basePath = (process.env.PATH ?? process.env.Path ?? "")
  .split(delimiter)
  .filter(
    (p) =>
      !p.toLowerCase().includes(root.toLowerCase()) &&
      !p.includes("node_modules"),
  );
if (existsSync(compiler))
  cpSync(compiler, isolatedCompiler, { recursive: true });
env[process.platform === "win32" ? "Path" : "PATH"] = [
  resolve(isolatedCompiler, "bin"),
  dirname(process.execPath),
  ...basePath,
].join(delimiter);
env.CARGO_HOME = resolve(temp, "cargo-home");
env.RUSTUP_HOME = resolve(temp, "rustup-unused");
env.npm_config_cache = resolve(temp, "npm-cache");
const evidence = [];
function run(exe, args, cwd = project) {
  const r = spawnSync(exe, args, {
    cwd,
    env,
    encoding: "utf8",
    maxBuffer: 20 * 1024 * 1024,
  });
  const label = [exe, ...args].join(" ");
  const output = (r.stdout ?? "") + (r.stderr ?? "");
  evidence.push({ command: label, exit: r.status, output });
  console.log(output.slice(-5500));
  if (r.status !== 0) {
    save();
    throw new Error(`Independent build failed: ${label}`);
  }
}
function save() {
  mkdirSync(resolve(root, "artifacts"), { recursive: true });
  writeFileSync(
    resolve(root, "artifacts/portability.json"),
    JSON.stringify(
      {
        directory: project,
        files: files.length,
        parent_dependencies_used: false,
        checks: evidence,
      },
      null,
      2,
    ),
  );
}
run(process.execPath, [
  resolve(dirname(process.execPath), "node_modules/npm/bin/npm-cli.js"),
  "ci",
]);
run(process.execPath, [
  resolve(dirname(process.execPath), "node_modules/npm/bin/npm-cli.js"),
  "run",
  "build",
]);
run(process.execPath, [
  resolve(dirname(process.execPath), "node_modules/npm/bin/npm-cli.js"),
  "test",
]);
run(process.execPath, ["scripts/security-scan.mjs"]);
const cargo = resolve(
  isolatedCompiler,
  "bin",
  process.platform === "win32" ? "cargo.exe" : "cargo",
);
run(
  cargo,
  ["test", "--locked", "--no-default-features"],
  resolve(project, "src-tauri"),
);
run(cargo, ["check", "--locked"], resolve(project, "src-tauri"));
save();
console.log(`Independent-directory verification passed: ${project}`);
