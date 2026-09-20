import { readFileSync, readdirSync, statSync } from "node:fs";
import { resolve, relative } from "node:path";
const root = resolve(import.meta.dirname, "..");
const problems = [];
const ignored = new Set([
  "node_modules",
  "target",
  "gen",
  ".tools",
  ".cache",
  "artifacts",
  "test-results",
  "playwright-report",
]);
function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    ignored.has(e.name)
      ? []
      : e.isDirectory()
        ? walk(resolve(dir, e.name))
        : [resolve(dir, e.name)],
  );
}
for (const f of walk(root)) {
  const rel = relative(root, f).replaceAll("\\", "/");
  if (
    statSync(f).size > 4_000_000 ||
    !/[.](ts|tsx|rs|json|mjs|toml|yml|yaml|html|css)$/.test(f)
  )
    continue;
  const text = readFileSync(f, "utf8");
  if (/-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----/.test(text))
    problems.push(`${rel}: private key`);
  if (/(?:sbp_[a-f0-9]{40}|gh[pousr]_[A-Za-z0-9]{36})/.test(text))
    problems.push(`${rel}: possible provider token`);
  if (
    rel.startsWith("src/") &&
    /dangerouslySetInnerHTML|eval\(|new Function\(/.test(text)
  )
    problems.push(`${rel}: executable text boundary`);
  if (rel.startsWith("dist/") && /SENTINEL[-+]/.test(text))
    problems.push(`${rel}: synthetic credential leaked into frontend bundle`);
}
const conf = JSON.parse(
  readFileSync(resolve(root, "src-tauri/tauri.conf.json"), "utf8"),
);
const cap = JSON.parse(
  readFileSync(resolve(root, "src-tauri/capabilities/main.json"), "utf8"),
);
if (
  cap.remote ||
  cap.windows.join(",") !== "main" ||
  cap.permissions.join(",") !== "allow-setup-commands"
)
  problems.push("Unexpected native capability scope");
if (
  /unsafe-eval|unsafe-inline|https?:/.test(
    conf.app.security.csp.replace("http://ipc.localhost", ""),
  )
)
  problems.push("CSP broadened beyond local IPC");
if (conf.app.windows.some((w) => w.url || w.devtools))
  problems.push("Remote or debug privileged window enabled");
if (problems.length) {
  console.error(problems.join("\n"));
  process.exit(1);
}
console.log(
  "Setup source, frontend assets, CSP and capability checks passed. Pattern scan is not a complete secret audit.",
);
