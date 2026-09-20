# Villow Setup

Keep all manager implementation and dependencies in this standalone repository.
Run project commands from this repository root. The separate Villow web
repository owns hosted app source, SQL baselines and release publication.
Never make the manager runtime depend on that checkout's dependencies,
environment, Git metadata or migration files.
An explicitly opt-in local test may consume an app-owned synthetic release
fixture to verify this public contract; production trust must stay separate.

Windows desktop application: Tauri 2, Rust native engine, React/TypeScript UI.
Read [README](README.md), the [architecture decision](docs/architecture-decision.md),
[security evidence](docs/security-evidence.md), the [standalone handoff](docs/standalone-handoff.md)
and the maintained [release checklist](docs/release-checklist.md).

Security invariants:

- Distribution coordinates are user-selected: nicholasnucifora/Villow-Setup.
  Production remains disabled until verified publisher identity, public keys,
  a signed Villow release and qualification evidence exist. Never enable production using demo keys.
- All privileged effects belong to Rust. No generic request, shell or filesystem IPC.
- Checkpoint intent before effects. Uncertain creates require reconciliation.
- Secrets belong to Windows Credential Manager; vault errors are fatal.
- Nonsecret recovery files are untrusted hints, never write authorization.
- Never touch existing production as a test. Opt-in integration uses disposable resources.
- No cloud deletion, legacy adoption or upgrades in version one.

Run npm ci, npm run check, npm test, npm run build, then npm run native:test.
`npm run dev` is only the browser preview; use `npm run desktop:dev` for the
native application boundary.
Use npm run desktop:build for an unsigned development installer. Public packaging
requires the release gate and actual publisher signing. Do not claim a mock
deployment or unsigned installer is ready for public use.

The maintainer explicitly authorized a separate unsigned alpha for fresh-account
qualification while Windows signing is deferred. Use `npm run desktop:build:alpha`
and follow `docs/unsigned-alpha.md`. Genuine app-release authentication is still
mandatory; never promote fixture keys or mark signed/public qualification passed.
