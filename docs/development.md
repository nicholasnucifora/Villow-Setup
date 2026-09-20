# Development and verification

Run commands from this standalone repository root. Node 22 and Rust 1.90 are pinned by the supported development configuration and `src-tauri/rust-toolchain.toml`. Windows builds also require the Visual Studio C++ toolchain and Windows SDK. The GUI uses WebView2; NSIS can install its official bootstrapper when absent.

`scripts/bootstrap-rust.ps1` optionally installs the official Rust toolchain under `.tools/` without changing the system PATH. Its installer download is checked against the official SHA256 download. The checksum uses the same origin as the executable; it is transport/integrity evidence, not an independent publisher trust root. Normally maintainers should use their existing supported Rust installation.

```powershell
npm ci
npm run build
npm test
npm run native:test
npm run native:check
npm run security:scan
node scripts/mutation-check.mjs
npm run desktop:build
```

`npm run dev` is only a browser preview on loopback port 1420. Native setup commands are unavailable there. `npm run desktop:dev` starts the actual desktop app. Never supply real tokens to a preview, test fixture or demo.

The native layer keeps state in Tauri's per-user local-data directory for the application identifier, with secrets in the OS credential vault under the installation ID. Nothing reads a parent `.env` or `.vercel` directory. Changing the application identifier deliberately separates development and production data.

## Real local PostgreSQL tests

On this Windows workstation PostgreSQL 17 is installed. The helper below creates its own cluster under ignored `artifacts/`, binds only to `127.0.0.1` on an available port, uses a synthetic password, runs migration tests and stops the cluster in `finally`. It does not discover or connect to an existing database. Set `VILLOW_POSTGRES_BIN` if its binaries are elsewhere.

```powershell
node scripts/test-postgres.mjs
```

For an explicitly disposable CI cluster, set `VILLOW_LOCAL_DB_TESTS=disposable-local-cluster`, `VILLOW_LOCAL_DB_PORT` and a synthetic `VILLOW_LOCAL_DB_PASSWORD`, then run:

```powershell
node scripts/native.mjs test --locked --no-default-features --test postgres_protocol -- --ignored --test-threads=1
```

Those tests hard-code the connection host to loopback and create uniquely named databases. They never accept a cloud database URL. They test synthetic protocol fixtures.

The web app's `npm run release:check` creates a synthetic signed release from app source and calls Setup's opt-in `app_release` native test against its full baseline. That test input belongs to the disposable harness; it never populates production trust. The web harness must obtain an explicitly pinned Setup checkout and pass its validated path; it must not rely on the former nested location. This paired-harness adjustment is still pending.

## Packaged checks

After `npm run desktop:build`, run `.\scripts\verify-dev-package.ps1` on Windows for an isolated unsigned-package smoke test. It refuses to replace an already registered Villow Setup installation or interrupt an existing process, installs into a new artifact directory, launches twice, then uninstalls its own copy. It does not inspect the rendered UI.

Build the NSIS package, install it in a clean Windows test profile, launch and close it, then reopen the demo after an injected lost response. Verify the demo boundary remains prominent, the callback copies correctly, keyboard focus is visible, long errors wrap, and no provider token is requested in demo mode. Verify dark Windows system controls, minimum-size layout, high DPI and a machine without WebView2. Local process smoke checks are not a substitute for these visual checks.

For a production release also verify the signer on both the installed application and installer, timestamps, selected application identifier, known download origin and the signed app channel. Record observations and artifact SHA256 in the qualification evidence. Do not mark the public gate complete using a development build or synthetic provider test.

## Real-provider qualification

First review and publish the implemented app contract and configure reviewed public trust roots. Use dedicated disposable Vercel/Supabase/Google resources only after the owner authorizes the accounts, cost exposure and cleanup scope. Exercise fresh install, interruption after each provider write, expired-token reconnection to the same principal, owner sign-in, bounded YouTube probe, RLS/grants and local credential removal. Inspect provider bills and retained resources explicitly. No test helper creates or deletes real cloud resources automatically.
