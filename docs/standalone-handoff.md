# Handoff to the standalone Villow Setup agent

Prepared 2026-09-20 from the local Villow working tree. The maintainer reports that Setup has moved into its own repository. The destination checkout and its hosted CI have not been inspected for this handoff. Verify their actual state before changing anything.

## Your assignment

Finish adapting Villow Setup to run and develop from the root of `nicholasnucifora/Villow-Setup`. Previously its root was a folder named `villow-setup/` inside the web app. Do the necessary path, workflow and documentation adjustments; verify the standalone project; keep unfinished release work visible for the maintainer.

All Setup paths in this document are relative to **your repository root**. Do not create another `villow-setup/` directory. Read `AGENTS.md`, `README.md`, this document and [the release checklist](release-checklist.md) first. Preserve unrelated work and report any differences between this handoff and your checkout.

The hosted web app remains in `nicholasnucifora/Villow`, locally called `disconnect`. Its agent owns web changes. Neither agent should assume it can see the other's checkout or conversation. Source separation is the maintainer's current decision; older instructions to keep Setup nested are superseded. Real cloud and signed Windows qualification can be completed after separation.

## Current boundaries

- Setup is a Tauri 2/Rust and React/TypeScript Windows manager. The web app owns hosted code, authentication, SQL history/baselines and app release generation.
- Setup consumes authenticated release artifacts. Its runtime must never need a web source checkout, parent `.env`, dependencies, Git metadata or raw migration directory.
- `Villow-Setup` is also the selected distribution repository for the Windows installer, web app release archive/manifest and signed channel. The web source repository remains `Villow`. A repository URL is not a trust root.
- The local source has working development packaging and local/native test evidence. It is **not a qualified public release**. Real provider installation, OAuth and signed clean-Windows qualification remain open.
- `src-tauri/trust.json` has empty production keys and no publisher. `docs/qualification.json` records unfulfilled gates. Keep those gates honest; moving files or passing mocks does not satisfy them.
- SignPath is the maintainer's preferred signing route. Enrollment is still owner work, and the checked-in certificate-store release workflow does **not** implement SignPath integration.

## Immediate work in your repository

1. **Inspect the actual checkout.** Check its Git root, remotes, status, root instructions and source tree. If the move is already complete, do not recopy files over newer changes. Keep the package and Cargo lockfiles, hidden configuration, `.github/`, `assets/`, `docs/`, `scripts/`, `src/`, `tests/` and `src-tauri/`. Use [extraction notes](extract-to-repository.md) and `scripts/source-files.mjs` to check the transfer. Exclude generated installers, `dist/`, `node_modules/`, `target/`, caches, logs, local environment files and private credentials. Do not commit test databases or recovery exports containing real resource details.
2. **Audit path assumptions.** Search tracked source/configuration for `villow-setup/`, `../`, `disconnect`, absolute developer-machine paths, workspace dependencies and workflow `working-directory` settings. Review each result. Scripts based on `import.meta.dirname` already resolve their own project root. Keep legitimate package/executable names such as `villow-setup.exe`, distribution URLs and historical evidence. Do not perform a global string replacement. No parent package, compiler cache or branding source folder should be required to build.
3. **Review workflows at the new root.** `.github/workflows/ci.yml` and `release.yml` already assume Setup is the root project; confirm that remains true. Their new location makes them discoverable by GitHub Actions. Check trigger behavior, lockfile paths, artifacts and toolchain installation. Ordinary PR CI must not reach publisher secrets or a privileged signing runner. Branch protections, runner registration, environments and secrets are account settings and do not transfer with source files; record owner actions in the checklist.
4. **Preserve application identity and owned assets.** A folder move is not a reason to change the Tauri identifier, crate/package names, storage namespace or recovery format. The current development identifier is `app.villow.setup.dev`; production identity remains a separate release decision. Keep the actual fixed brand assets in `assets/icon.svg` and `src-tauri/icons/icon.ico`. Do not restore the retired placeholder icon generator, link to the web repo's `villow-icons/` source kit, or silently omit newer owned assets/fonts.
5. **Update contributor and user guidance.** Remove obsolete requirements to stay nested from current instructions while retaining dated historical results. Explain that commands run at this repository's root, `npm run dev` is only the browser preview, and `npm run desktop:dev` exercises the native app. Preserve [release-checklist.md](release-checklist.md) as the maintainer's backlog, link it from README/AGENTS, and record new findings there.
6. **Request the paired web test-harness adjustment below.** This is a development/CI dependency to resolve with the web agent, not a runtime dependency to introduce. Do not mark the split fully verified while this coverage is silently missing.
7. **Verify and report.** Run the relevant standalone checks below after your edits. Fix failures within scope and record remaining external prerequisites accurately. Leave real cloud resources, enrollment, production trust and publication pending their explicit owner inputs.

### Standalone verification

Use Windows with the documented Node 22, Rust 1.90, Visual Studio C++/Windows SDK and WebView2 prerequisites, checking the repository's current pinned versions first. From the Setup root:

```powershell
npm ci
npm run check
npm test
npm run build
npm run native:test
npm run native:check
npm run security:scan
npm run desktop:build
npm run portability
```

Also run the existing CI checks applicable to your changes, including disposable PostgreSQL integration when native/database contracts change. Inspect existing package-smoke and `scripts/verify-dev-package.ps1` usage before running installation checks; use a dedicated test profile/directory and clean up only resources the test owns. A browser preview cannot replace native IPC, Windows Credential Manager, file-dialog or installer checks. Report a permission-blocked vault test separately from a code failure.

`npm run portability` copies an allowlisted source tree into an unrelated temporary directory and tests with isolated dependency stores. It does not prove that a machine without developer tools can install the product. `npm run release:gate` is expected to reject the currently incomplete production qualification; do not weaken it to make extraction appear successful.

Development packaging currently creates `src-tauri/target/release/bundle/nsis/Villow Setup_0.1.0_x64-setup.exe`; use the actual package version if it has changed. Cloning source does not provide a built installer. Future end users download the reviewed installer from GitHub Releases, run it, then use their own provider accounts. Developers clone and build. Current workflows prepare review artifacts; they do not automatically publish a public download.

### Historical evidence, not a new standalone pass

The September 10 portability run and September 12–14 desktop review are documented in [security evidence](security-evidence.md), [extraction notes](extract-to-repository.md) and [desktop review](desktop-review.md). The latter records 15 UI tests, 34 ordinary native tests, native dialogs/recovery checks and an **unsigned** development installer install, two launches and uninstall. Those results do not qualify newer source/branding, the new repository's CI, real cloud/OAuth or a signed clean Windows machine. Record your own exact commit and outcomes rather than copying historical pass counts into a new report.

## Required coordination with the Villow web agent

### First paired change: preserve native release compatibility tests

In the inspected web working tree:

- `scripts/test-app-release.mjs` resolves `root/villow-setup` and runs native verification only if that checkout exists. Without it, the command can succeed while its `native_verifier` and `native_database_install_resume` evidence fields are false.
- `scripts/setup-schema.mjs --check --release <directory>` also hard-codes that nested checkout as the working directory for Setup's ignored `app_release` native test. The fixture and local database are created by the web harness.
- Web `.github/workflows/app-release.yml` calls `npm run release:check`; it must explicitly obtain the reviewed Setup test code after extraction.

Send the web agent this request through the maintainer:

> Setup now lives at the root of `nicholasnucifora/Villow-Setup`. Please adapt `scripts/test-app-release.mjs` and `scripts/setup-schema.mjs` to share an explicit, validated external Setup checkout path, and update release qualification CI to obtain a reviewed immutable Setup commit. Agree the interface with the Setup agent before implementing it. A proposed option is `--setup-root` or `VILLOW_SETUP_TEST_ROOT`; neither exists in the inspected code yet. Release qualification must fail clearly if the required native verifier is missing, rather than silently pass without it. If an app-only local mode may skip native coverage, make that an explicit option with visible incomplete evidence. Keep integration fixtures disposable and production trust separate. Report the web commit, chosen Setup commit, command/interface and verification results back to the Setup agent.

The two agents should agree where the reviewed Setup commit is pinned and how it is intentionally updated. CI must check out that exact SHA, not a moving `main`, and install that project's own prerequisites/dependencies. Validate missing and invalid supplied paths plus the actual archive/native/database test. Record both real checkout SHAs separately from the synthetic app fixture commit. Keep ordinary app-only CI lightweight; add the paired qualification where relevant rather than making every web test require Windows.

Some web references to `villow-setup` are deliberately correct: a synthetic private-file fixture proves that Setup content is excluded from the web release, and deployment/archive exclusions remain useful. Do not erase them indiscriminately. Removing the old nested copy should happen only after the maintainer confirms the new source is complete and the replacement integration check works.

### Ownership and changes that must be coordinated

| Change | Primary owner | Paired work needed |
| --- | --- | --- |
| Desktop UI, native IPC, local vault/checkpoints, NSIS packaging | Setup agent | Inform web agent if request/response or supported install behavior changes |
| Hosted APIs, Google OAuth/session enforcement, origin/CORS, env variables | Web agent | Update Setup instructions/adapters and test exact callback, owner and health behavior |
| SQL history, fresh baseline, postcondition, schema identity | Web agent | Update and test the authenticated release against Setup's native database engine |
| Archive/manifest/channel format, `minimum_manager`, bootstrap/health version | Joint | Agree compatibility first; test both pinned commits and document rollout order |
| Windows signing and publisher configuration | Maintainer + Setup agent | Keep separate from the web app's Ed25519 channel-signing keys |
| Official app artifacts, channel renewal, revocations/trust rotation | Maintainer + web agent, reviewed by Setup agent | Verify Setup accepts the intended release and rejects incompatible/untrusted data |
| Real cloud/OAuth and clean signed Windows qualification | Maintainer + both agents as applicable | Use an agreed candidate pair and disposable accounts/projects; retain redacted evidence |

Use [the versioned integration contract](villow-integration-contract.md) as the shared contract, and the web repository's root `SETUP_INTEGRATION.md` for its rollout instructions. Changes to one copy of the contract must be communicated to the other agent. Do not copy the web SQL source into Setup as a workaround for separation.

### Non-negotiable integration behavior

- The OAuth callback is exactly `${VITE_APP_URL}/api/auth`, not a Supabase Auth callback. The intended owner must be verified by the server; caller-supplied IDs/cookies are not credentials. Never expose Google tokens in redirect URLs.
- Fresh installations use the authenticated app-owned fresh baseline. Existing Villow deployments must apply migration `158_add_setup_bootstrap.sql` before the related OAuth/session code, preserve their existing `ENCRYPTION_KEY`, and use the documented session/cron rollout. **Never run the fresh baseline against an existing database.** Recheck newer migrations before preparing the next app release.
- `/api/setup` dispatches through the existing settings function before shared CORS/auth logic. Preserve the 12-function budget. Bootstrap tokens authorize bounded health attestation, not user sessions or general database access.
- Preserve checkpoint-before-effect ordering, reconciliation of uncertain creation, OS-vault-only secrets and read-only recovery imports. No plaintext secret fallback, cloud teardown, writable adoption or upgrades in v1.
- App releases come from a reviewed immutable web commit and an explicit allowlist. Synthetic signing keys and fixture commits are test-only. Real public trust needs reviewed keys and publisher evidence.

### Handoff packet for each paired change

Give the other agent the problem and proposed contract change, repository/branch and exact commit, changed files or patch/PR, supported versions, rollout order, tests run and their results, and the specific action requested. For release qualification also include app manifest/baseline/archive digests, installer SHA-256/signature evidence, environment versions and explicit unrun checks. Share redacted logs only; never tokens, cookies, `.env`, private keys or personal recovery files.

When the other checkout is unavailable, leave a concrete request in the checklist and give the maintainer a pasteable message. Do not claim a coordinated change landed until the counterpart confirms it with a commit and evidence. The maintainer can bring that packet to the existing Villow web task; agents do not automatically share task history.

## Completion report expected from you

Report the root/path changes made, standalone checks actually run, generated development installer location, status of the paired web harness change, and any concrete owner inputs still required. Update the maintained checklist with dated evidence. Distinguish **ready for independent development** from **qualified for public release**. The immediate handoff authorizes the repository adaptation; enrollment, live provider testing and publication remain later work.
