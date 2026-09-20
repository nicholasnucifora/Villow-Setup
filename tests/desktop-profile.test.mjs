import assert from "node:assert/strict";
import { test } from "node:test";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import {
  desktopProfile,
  validateAlphaTrust,
  distributionRepository,
  channelUrl,
} from "../scripts/desktop-profile.mjs";

// Public RFC 8032 test vector, used in memory for configuration validation only.
// No signing key is generated or written and checked-in trust is never changed.
const publicKey = Buffer.from(
  "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
  "hex",
).toString("base64");
const trust = () => ({
  format: 1,
  repository: distributionRepository,
  channel: "stable",
  manifest_url: channelUrl,
  publisher: "Unit test publisher",
  public_keys: { "release-2026": publicKey },
  minimum_sequence: 1,
});
const alpha = ["build", "--bundles", "nsis", "--unsigned-alpha"];

test("ordinary packaging resets inherited testing and alpha flags", () => {
  const result = desktopProfile(["build", "--bundles", "nsis"], {
    VILLOW_SETUP_TESTING: "1",
    VILLOW_SETUP_ALPHA: "1",
  });
  assert.equal(result.env.VILLOW_SETUP_TESTING, "0");
  assert.equal(result.env.VILLOW_SETUP_ALPHA, "0");
  assert.equal(result.config, null);
});

test("alpha is explicitly unsigned and separate from development/testing", () => {
  const result = desktopProfile(alpha, { VILLOW_SETUP_TESTING: "1" }, trust());
  assert.equal(result.env.VILLOW_SETUP_TESTING, "0");
  assert.equal(result.env.VILLOW_SETUP_ALPHA, "1");
  assert.equal(result.config.productName, "Villow Setup Alpha");
  assert.equal(result.config.identifier, "app.villow.setup.alpha");
  assert.equal(result.config.app.windows[0].devtools, false);
  assert.equal(result.config.bundle.windows.certificateThumbprint, null);
  assert.equal(result.config.bundle.windows.signCommand, null);
  assert.equal(result.config.bundle.windows.nsis.installMode, "currentUser");
  const testing = desktopProfile(["dev", "--testing-tools"], {});
  assert.notEqual(result.config.identifier, testing.config.identifier);
  assert.equal(testing.env.VILLOW_SETUP_ALPHA, "0");
});

test("alpha refuses testing, config overrides and unreviewed build arguments", () => {
  for (const extra of [
    ["--testing-tools"],
    ["--config", "extra.json"],
    ["--config=extra.json"],
    ["-cextra.json"],
    ["-c", "extra.json"],
    ["--debug"],
    ["--no-bundle"],
  ])
    assert.throws(() => desktopProfile([...alpha, ...extra], {}, trust()));
  assert.throws(() => desktopProfile(alpha, { TAURI_CONFIG: "{}" }, trust()));
  assert.throws(() => desktopProfile(["dev", "--unsigned-alpha"], {}, trust()));
});

test("alpha refuses missing or malformed public trust instead of enabling a bypass", () => {
  for (const changes of [
    { publisher: null },
    { publisher: " " },
    { format: 2 },
    { repository: "test-owner/test-releases" },
    { channel: "other" },
    { manifest_url: channelUrl + "?redirect=1" },
    { minimum_sequence: 0 },
    { minimum_sequence: 1.5 },
    { public_keys: {} },
    { public_keys: [] },
    { public_keys: { "test-only": publicKey } },
    { public_keys: { "release-2026": "not-a-public-key" } },
    { public_keys: { "release-2026": publicKey + "\n" } },
    { public_keys: { "release-2026": Buffer.alloc(32).toString("base64") } },
  ])
    assert.throws(() => validateAlphaTrust({ ...trust(), ...changes }));
  assert.throws(() => validateAlphaTrust(undefined));
  assert.doesNotThrow(() => validateAlphaTrust(trust()));
});

test("alpha planning does not alter embedded trust or public qualification", () => {
  const files = ["../src-tauri/trust.json", "../docs/qualification.json"].map(
    (path) => new URL(path, import.meta.url),
  );
  const before = files.map((path) => readFileSync(path, "utf8"));
  desktopProfile(alpha, {}, trust());
  assert.deepEqual(
    files.map((path) => readFileSync(path, "utf8")),
    before,
  );
  const result = spawnSync(process.execPath, ["scripts/release-gate.mjs"], {
    cwd: new URL("..", import.meta.url),
    encoding: "utf8",
    env: { ...process.env, VILLOW_SETUP_ALPHA: "1" },
  });
  assert.equal(result.status, 1);
  assert.match(
    result.stderr,
    /unsigned alpha is not a qualified public release/,
  );
});
