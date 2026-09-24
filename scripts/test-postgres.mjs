// Starts only a new, disposable loopback cluster. Never reads app credentials.
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  existsSync,
  openSync,
  closeSync,
  readFileSync,
} from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { createServer } from "node:net";
import { buildEnv } from "./build-env.mjs";
const root = resolve(import.meta.dirname, "..");
const updateFixture =
  process.argv[2] === "--app-update-fixture"
    ? resolve(process.argv[3] ?? "")
    : null;
if (
  process.argv.length > 2 &&
  (!updateFixture || !process.argv[3] || process.argv.length !== 4)
)
  throw new Error(
    "Use --app-update-fixture <explicit synthetic fixture directory>.",
  );
if (
  updateFixture &&
  JSON.parse(readFileSync(resolve(updateFixture, "fixture.json"), "utf8"))
    .test_only !== true
)
  throw new Error("Only an explicitly synthetic update fixture is allowed.");
const bin =
  process.env.VILLOW_POSTGRES_BIN ?? "C:/Program Files/PostgreSQL/17/bin";
if (!existsSync(resolve(bin, "initdb.exe")))
  throw new Error(
    "Set VILLOW_POSTGRES_BIN to an installed PostgreSQL bin directory.",
  );
mkdirSync(resolve(root, "artifacts"), { recursive: true });
const dir = mkdtempSync(resolve(root, "artifacts/postgres-")),
  data = resolve(dir, "data");
const password = "SENTINEL-DISPOSABLE-LOCAL-CLUSTER";
writeFileSync(resolve(dir, "password.txt"), password);
const port = await new Promise((ok, fail) => {
  const s = createServer();
  s.once("error", fail);
  s.listen(0, "127.0.0.1", () => {
    const p = s.address().port;
    s.close(() => ok(p));
  });
});
const env = {
  ...buildEnv(root),
  VILLOW_LOCAL_DB_TESTS: "disposable-local-cluster",
  VILLOW_LOCAL_DB_PORT: String(port),
  VILLOW_LOCAL_DB_PASSWORD: password,
  ...(updateFixture ? { VILLOW_APP_UPDATE_TEST_DIR: updateFixture } : {}),
};
const evidence = [];
function run(exe, args, required = true) {
  const log = resolve(dir, "command-" + evidence.length + ".log"),
    fd = openSync(log, "w");
  // A spawned PostgreSQL daemon can inherit pipe handles on Windows. Use
  // files so waiting for pg_ctl does not wait for the daemon's open pipe.
  let r;
  try {
    r = spawnSync(exe, args, {
      cwd: root,
      env,
      stdio: ["ignore", fd, fd],
      windowsHide: true,
      timeout: 180000,
    });
  } finally {
    closeSync(fd);
  }
  const output = readFileSync(log, "utf8");
  evidence.push({ command: exe, args, exit: r.status, output });
  console.log(output);
  if (required && r.status !== 0)
    throw new Error("Disposable PostgreSQL verification failed.");
  return r;
}
try {
  run(resolve(bin, "initdb.exe"), [
    "-D",
    data,
    "-U",
    "postgres",
    "--auth=scram-sha-256",
    "--pwfile",
    resolve(dir, "password.txt"),
    "--encoding=UTF8",
    "--locale=C",
  ]);
  run(resolve(bin, "pg_ctl.exe"), [
    "start",
    "-D",
    data,
    "-l",
    resolve(dir, "server.log"),
    "-o",
    `-h 127.0.0.1 -p ${port}`,
    "-w",
  ]);
  run(process.execPath, [
    "scripts/native.mjs",
    "test",
    "--locked",
    "--no-default-features",
    "--test",
    updateFixture ? "app_update_protocol" : "postgres_protocol",
    "--",
    "--ignored",
    "--show-output",
    "--test-threads=1",
  ]);
} finally {
  if (existsSync(resolve(data, "postmaster.pid")))
    run(
      resolve(bin, "pg_ctl.exe"),
      ["stop", "-D", data, "-m", "fast", "-w"],
      false,
    );
  writeFileSync(
    resolve(
      root,
      updateFixture
        ? "artifacts/app-update-postgres-tests.json"
        : "artifacts/postgres-tests.json",
    ),
    JSON.stringify(
      { disposable_loopback_only: true, directory: dir, checks: evidence },
      null,
      2,
    ),
  );
}
