import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import assert from "node:assert/strict";

const root = resolve(import.meta.dirname, "..");
for (const profile of ["normal", "testing", "alpha"]) {
  const testing = profile === "testing";
  const output = `artifacts/ui-${profile}`;
  const result = spawnSync(
    process.execPath,
    ["node_modules/vite/bin/vite.js", "build", "--outDir", output],
    {
      cwd: root,
      env: {
        ...process.env,
        VILLOW_SETUP_TESTING: testing ? "1" : "0",
        VILLOW_SETUP_ALPHA: profile === "alpha" ? "1" : "0",
      },
      encoding: "utf8",
    },
  );
  if (result.status !== 0) throw new Error(result.stdout + result.stderr);
  const files = readdirSync(resolve(root, output, "assets")).filter((file) =>
    file.endsWith(".js"),
  );
  const javascript = files
    .map((file) => readFileSync(resolve(root, output, "assets", file), "utf8"))
    .join("\n");
  for (const marker of [
    "villow-setup-demo-v1",
    "Show testing tools",
    "Demo failure",
    "Load demo accounts",
    "Review diagnostic information",
  ])
    assert.equal(
      javascript.includes(marker),
      testing,
      `${output}: unexpected ${marker} inclusion`,
    );
  assert.equal(
    files.some((file) => file.startsWith("TestingTools-")),
    testing,
  );
  assert.ok(javascript.includes("Vercel access token"));
  assert.ok(javascript.includes("Supabase management token"));
  assert.equal(
    javascript.includes("Unsigned alpha · Fresh test installations"),
    profile === "alpha",
  );
  console.log(
    `${profile} build: account fields present; testing engine and controls ${testing ? "included by opt-in" : "excluded"}; alpha label ${profile === "alpha" ? "present" : "absent"}.`,
  );
}
