import {
  cpSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
  mkdirSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { buildEnv } from "./build-env.mjs";
const root = resolve(import.meta.dirname, "..");
const temp = mkdtempSync(resolve(tmpdir(), "villow-guard-mutations-"));
const crate = resolve(temp, "src-tauri");
mkdirSync(crate, { recursive: true });
for (const item of [
  "Cargo.toml",
  "Cargo.lock",
  "build.rs",
  "rust-toolchain.toml",
  "trust.json",
  "certs",
  "backup-tables-0.1.1.json",
  "backup-triggers-0.1.1.json",
  "src",
  "tests",
])
  cpSync(resolve(root, "src-tauri", item), resolve(crate, item), {
    recursive: true,
  });
const env = {
  ...buildEnv(root),
  CARGO_TARGET_DIR: resolve(root, "src-tauri/target"),
};
const cargo = process.platform === "win32" ? "cargo.exe" : "cargo";
const mutants = [
  {
    file: "release.rs",
    from: /key\.verify_strict\(&payload,\s*&signature\)\s*\.map_err\(\|_\|\s*Error::Release\)\?;/,
    to: "let _ = (&key, &signature);",
    test: "signature_guard_rejects_structurally_valid_forgery",
  },
  {
    file: "http.rs",
    from: "300..=399 => Err(Error::WrongTarget)",
    to: "300..=399 => Ok(())",
    test: "credentials_cannot_follow_redirects_or_change_hosts",
  },
  {
    file: "store.rs",
    from: "sequence < seq || (sequence == seq && digest != hash)",
    to: "sequence < 0 && digest == hash && seq == 0",
    test: "rejects_expired_future_and_replayed_channel",
  },
];
const evidence = [];
for (const mutant of mutants) {
  const path = resolve(crate, "src", mutant.file),
    original = readFileSync(path, "utf8");
  if (
    typeof mutant.from === "string"
      ? !original.includes(mutant.from)
      : !mutant.from.test(original)
  )
    throw new Error(`Mutation location moved: ${mutant.file}`);
  writeFileSync(path, original.replace(mutant.from, mutant.to));
  const r = spawnSync(
    cargo,
    [
      "test",
      "--locked",
      "--no-default-features",
      "--test",
      "security",
      mutant.test,
    ],
    { cwd: crate, env, encoding: "utf8" },
  );
  writeFileSync(path, original);
  const output = r.stdout + r.stderr;
  if (
    r.status === 0 ||
    !output.includes("test result: FAILED") ||
    !output.includes(mutant.test)
  )
    throw new Error(
      `Guard mutation was not caught by its assertion: ${mutant.file}\n${output}`,
    );
  evidence.push({
    guard: mutant.file,
    test: mutant.test,
    result: "test failed as expected when guard was bypassed in isolated copy",
  });
}
console.log(
  JSON.stringify(
    {
      mutations: evidence,
      temporary_copy: temp,
      production_source_modified: false,
    },
    null,
    2,
  ),
);
