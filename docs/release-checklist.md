# Villow Setup: maintainer release checklist

Started 2026-09-20. This is the ongoing backlog for the developer and both repository agents. It is separate from the machine-enforced [qualification record](qualification.json). Leave items open until there is dated evidence for the relevant candidate; local mocks, historical passes and moving repositories do not complete a real release gate.

**Owners:** “You” means the maintainer; “Setup agent” works in `nicholasnucifora/Villow-Setup`; “web agent” works in `nicholasnucifora/Villow` (the local `disconnect` project). Both agents can continue implementation in separate repos. You need not keep the projects nested or translate this checklist into code yourself.

## Published app / configured Alpha follow-up — 2026-09-21

**2026-09-22 installed Alpha repair candidate:** Manager 0.1.3 implements the agreed separate repair flow and app-owned fixture verifier. Local tests pass for native ownership/history/SQL atomicity, saved intent, uncertain deployment resume and existing data preservation; the actual old release to early corrected app fixture also passed. The counterpart is fixing the JSON import, two absent watch-time fields and an obsolete schedule write in app 0.1.2. Final paired qualification, signed corrected release/publication, owner backup and live execution remain pending. See [repair evidence](security-evidence.md) and [the contract](villow-integration-contract.md).

**2026-09-22 hosted onboarding failure:** The maintainer confirms the Vercel site became available and adding their Google account to Test users allowed them to reach `/setup`. Their Vercel logs then show a settings-function JSON import failure and a watch-time query for a missing database column. Inspection of the exact published 0.1.1 archive confirms the invalid bare JSON import and the baseline's omission of `daily_watch_time_seconds` and `daily_watch_reset_at`. A local isolated Node 22 import probe reproduces the import error. These require app-owned fixes and a reviewed continuation plan for an already installed database; the pre-installation correction flow cannot apply. See [the concrete web-app handoff](hosted-setup-failure-handoff.md). No runtime, cloud, database or published release was changed, and full onboarding/health qualification remains open.

**2026-09-22 published correction and deployment follow-up:** The genuine 0.1.1 release and channel sequence 2 were published with owner approval; anonymous canonical downloads and Setup native authentication passed. The original 0.1.0 pointer/assets remain supported. The maintainer subsequently reports database preparation succeeded. Full installation remains unfinished: the next reported failure is Vercel DEPLOYMENT_NOT_FOUND. Setup 0.1.2 now gates sign-in on actual build readiness and exact address assignment, polls the saved deployment without recreating it, and exposes clear progress/failure states. See [current evidence](security-evidence.md). The actual build log and real sign-in/health results remain required. Channel renewal is due **28 September 2026 at 8:20 pm Brisbane**; automatic renewal remains unconfigured. Older dated records below describe earlier candidates.

**2026-09-22 postcondition investigation:** The maintainer's latest retry reports release unit 1's false result check. Exact published SQL was reproduced in disposable local PostgreSQL: automatic API-role grants alone change the access fingerprints and fail both the signed check and hosted probe; the same SQL passes with plain defaults. The web-app agent is preparing the app-owned fix after its pending read-only comparison was approved. A new release alone cannot repin the existing unfinished installation; authenticated continuation remains joint work. See [the investigation and evidence](database-release-investigation.md). No user database or published artifact was changed.

**2026-09-22 database-review follow-up:** The maintainer's later SchemaDrift message occurred after connection and could also mean release SQL failed. This candidate adds bounded phase/unit/SQLSTATE diagnostics and distinguishes pre-installation object counts and incomplete history without weakening any write gate. The supplied Session pooler screenshot and explanation of Setup's generated/saved database password are inline. Thirty UI tests, 43 ordinary native tests and five disposable local PostgreSQL tests passed. The actual provider failure is not yet diagnosed; maintainer retry and new paired qualification remain open. Exact candidate/build identity belongs in the build receipt.

**2026-09-22 follow-up:** Testing-only Google instructions now show all four supplied guide images inline. Database preparation now discovers the session pooler through the authenticated project API and trusts the official Supabase root certificate only in its native TLS connector, with safe diagnostic categories and visible manual recovery. Local UI/native/profile/packaging checks passed; this does not establish that the reported real database failure is resolved. A new paired rerun is required for this candidate; the web task's earlier rerun is still waiting on approval. See the new build receipt for the exact candidate and current status. Public qualification flags remain unchanged.

- [x] **Web-release maintainer:** Supply the genuine public key/key ID and publisher label, publish the immutable Villow 0.1.0 archive/manifest and signed channel, and report frozen-pair qualification (`1c6f949a3c0bc1b33f289b9684fa6ae84be766f4` web / `d01c07d012a961e008a733b9b22058e4421360b3` Setup). Publication and public repository visibility were owner-approved.
- [x] **Setup agent:** Independently download all three assets without authentication, verify the signature/expiry and expected hashes, embed exactly the supplied public trust, and authenticate the release using the existing native manager twice across reopened local stores. Preserve false public-qualification flags.
- [x] **Setup agent:** Run the ordinary checks and all three frontend profile checks with configured trust. No runtime implementation or contract changes were made; see [alpha-0.1.0.md](alpha-0.1.0.md) and the returned build receipt for the later Setup SHA and exact package verification.
- [ ] **Maintainer + Setup agent:** Complete the first real dedicated-account setup and intended-owner sign-in; qualify uncertain writes, real provider recovery and credential removal. Local package/native release checks do not complete this item.
- [ ] **Web-release maintainer:** Renew the signed channel before **27 September 2026, 3:45 pm Brisbane**, using the existing key and original immutable artifacts. Automatic renewal is not configured. Preserve expiry/replay checks; do not ask Setup to replace trust for an ordinary renewal.

Windows signing remains deferred for this expressly unsigned Alpha. The signed public-release gates and earlier historical task records below remain separate.

## Unsigned-alpha handoff follow-up — 2026-09-20

The maintainer explicitly deferred Windows signing and requested the web agent's follow-up handoff be implemented here. [Unsigned alpha](unsigned-alpha.md) now records the separate build command, exact public trust fields, artifact names and ownership. This supersedes any interpretation below that Windows enrollment must precede local real-account alpha qualification; signed public release gates are unchanged.

- [x] **Setup agent:** Review the pending account walkthrough/testing exclusion work and add an explicit `desktop:build:alpha` profile with its own name/identifier and no testing UI. Validate genuine public-trust configuration before invoking its build; do not generate release keys here.
- [x] **Setup agent:** Require the disposable database in the opt-in app-release test; check new-session resume, full native ledger checksums and installation/release ownership, and reject ledger/schema drift. Document the unproven crash/COMMIT-response scenarios separately.
- [x] **Web agent (handoff received; interface inspected):** Implement explicit clean Setup-root/full-SHA validation and refusal of zero native tests. This is still web working-tree source, not evidence of a final published app candidate or hosted workflow run.
- [ ] **Both agents:** Record the final immutable pair, pin Setup in the web workflow and run its full release qualification against the reviewed app commit. The Setup agent returns its committed candidate plus local results; synthetic app working-tree smoke checks are not an official app release.
- [ ] **Web-release maintainer:** Supply the approved publisher label, genuine raw Ed25519 public key/key ID, signed channel sequence/expiry and published immutable artifacts. Private-key custody remains on that side; no provider tokens are required for this handoff.
- [ ] **Setup agent, after authentic artifacts exist:** Commit the matching public trust, build the actual unsigned alpha, record its installer hash and inspect installation/reopen. Then coordinate the real disposable-account journey and owner consent.

No qualification booleans or production trust values have been filled by this follow-up. Earlier entries and artifact hashes below are dated results, not the final alpha's identity.

## 1. Finish the repository move — do now

- [x] **Setup agent:** Complete [standalone-handoff.md](standalone-handoff.md). Confirm source and hidden configuration live at the new repository root, audit parent-relative paths, preserve application identity, and update current instructions. Keep generated files/secrets out of Git.
- [x] **Setup agent:** Run the standalone build, frontend/native checks, packaging checks and portability verifier against the actual new checkout. Record its commit, environment and results; identify anything not run.
- [ ] **Both agents:** Replace the web release test's hard-coded nested Setup path with an agreed explicit checkout interface. Pin a reviewed Setup SHA in the paired qualification workflow. Prove that a missing verifier fails required qualification instead of silently losing coverage, then run the actual app archive/native baseline check.
- [ ] **You + Setup agent:** Review GitHub Actions, branch rules, protected environments and runner settings in the actual new repo. Source files do not carry these settings with them. Keep publisher access out of ordinary PR CI.
- [ ] **Both agents + you:** Confirm the standalone source and paired check are complete before deleting the old nested copy. The source handoff itself does not remove or publish anything.

Done means the new repo supports independent development and preserves cross-repo test coverage. It does not mean a public installer has been approved.

## 2. SignPath and release preparation — later, with your input

SignPath work has two parts: your enrollment/approval decisions and the agent's build integration after the approved configuration is known.

- [ ] **You:** Review current eligibility, terms, privacy and role requirements directly with SignPath. Use [application preparation](signpath-application.md) as dated notes, not a claim that today's form or terms are unchanged. Confirm how a solo Australian maintainer and a project without a qualified public installer can proceed. Enter identity/contact details in the provider UI; no organization needs to be invented.
- [ ] **You:** Decide whether to apply and accept the applicable terms. Set up account protection/recovery and any required approver roles. Do not put passwords, recovery codes, signing tokens or private keys in Git or agent messages.
- [ ] **You + Setup agent:** Publish accurate project/privacy/license/signing-policy information when ready. Do not claim Foundation approval or sponsorship before it exists. Review the draft policy against what is actually approved.
- [ ] **Setup agent:** Regenerate `npm run license:inventory`, review outstanding notices and the actual distributed components, and run the current dependency/security checks. The previous inventory and its unresolved notice locations are not a completed license review. Include owned fonts/icons and installer/runtime components as applicable; record unresolved findings.
- [ ] **Setup agent, after approved provider settings exist:** Implement the approved SignPath workflow. The current release workflow expects a certificate store/provider and is not already integrated. Preserve provenance and review boundaries, signing the inner application before packaging it unchanged and signing the outer installer; confirm uninstaller/runtime handling with the provider. Verify the actual publisher, signatures and timestamps in the final package.
- [ ] **You + Setup agent:** Finalize production application identity and storage behavior, protected release settings and signing approval process. Keep development and production state appropriately separated; do not change identity merely because the source moved.

## 3. Agree the official app/manager candidate — both repos

- [ ] **Web agent:** Prepare the current fresh database baseline, history map and postcondition from the reviewed web source. Reconcile newer migrations; do not assume the historical 159-file baseline still describes the latest app. Keep both `110_` identities and LF-normalized migration hashes.
- [ ] **Both agents:** Choose exact web and Setup commits and supported contract/manager versions. Run extracted app build, native archive verification and disposable database installation/resume against that pair. Exchange the evidence packet described in the handoff.
- [ ] **You + web agent:** Prepare the real app archive, manifest and channel from an immutable reviewed web commit. The source stays in `Villow`; approved distribution assets belong in `Villow-Setup`. Current workflows prepare artifacts for review and do not publish them automatically.
- [ ] **You + both agents:** Establish reviewed Ed25519 release/channel keys and actual Windows publisher trust. Keep those two signing systems separate. Review `src-tauri/trust.json` through source review; a remote URL or test key cannot establish trust. Record the official release digest and contract revision only when known.
- [ ] **Both agents:** Document a gated candidate-building procedure for real qualification. Production trust configuration alone must not label a candidate publicly qualified. Preserve the final release gate; never set qualification flags to bypass the checks needed to build/test a candidate.

## 4. Real cloud and OAuth qualification — supervised later

“Cloud/OAuth testing” means checking that Setup can actually create a fresh hosted instance and that Google's real browser sign-in initializes and authorizes the intended owner. Local tests cannot prove provider permissions, consent configuration, deployment behavior or token refresh. This work can be done from the standalone repo with the web agent coordinating its side.

- [ ] **You:** Choose disposable Vercel/Supabase projects/accounts and a Google test project/account, authorize the test actions, and agree costs and cleanup scope. You complete interactive consent. Do not use your existing Villow database as a test or send tokens in chat.
- [ ] **Setup agent:** Verify provider identity/organization selection, project creation, canonical domain, environment configuration and deployment with the intended candidate. Prove an end user does not need GitHub access, Git, Node, Rust or a source checkout to install a hosted instance.
- [ ] **You + both agents:** Enable the required YouTube API and configure Google's current audience/test-user/consent requirements. Match the exact authorized origin and `${VITE_APP_URL}/api/auth` redirect. Test intended-owner first sign-in, wrong-account refusal and bootstrap closure. Record Testing-mode restrictions/expiry using current provider documentation at test time.
- [ ] **Web agent:** Verify session authorization, refresh and recovery with real credentials; ensure tokens never appear in redirect URLs and service secrets never enter the browser bundle. Test the actual schema/RLS and grants with browser/service roles, canonical CORS/origin and separate cron authorization. Preserve the 12-function routing limit.
- [ ] **Both agents:** Check signed release identity and bounded setup health against the actual deployed instance, then normal owner operations and the intended small-group access model. Review quota expectations before inviting users; setup health alone does not prove feed/search behavior.
- [ ] **Setup agent + you:** Exercise interruption after relevant provider writes, lost responses, restart/resume and expired-token reconnect. Confirm selected resources/identity and the encryption key remain stable; uncertain creation must stop for reconciliation instead of making duplicates.
- [ ] **Setup agent + you:** Verify real local credential removal, read-only recovery behavior and export redaction. Closing/uninstalling Setup or removing local credentials must not imply cloud deletion or canceled billing. Cloud cleanup is a separate owner action; v1 has no automatic cloud teardown.
- [ ] **Both agents:** Record pass/fail/unrun results and issues with redacted logs and the exact candidate identities. Complete `real_no_github_fresh_install`, `real_interrupted_resume`, `real_intended_owner_sign_in`, `real_schema_and_rls_verification` and `credential_removal_verified` only when their real checks pass.

### Separate concern: updating an existing Villow instance

If deploying the new auth/session code to an existing web instance, the **web agent and you** must follow the web repo's `SETUP_INTEGRATION.md`: back up, preserve the existing `ENCRYPTION_KEY`, apply migration 158 before that code, verify auth, then enable legacy-route session enforcement and configure `CRON_SECRET`. Account for migrations already applied. Never apply the fresh baseline or treat this as Setup adoption. Rollback after encrypted token writes must keep compatible readers. This rollout is not silently authorized by a fresh-install test.

## 5. Clean Windows qualification — later with the signed candidate

“Clean Windows” means a fresh VM or suitable clean user/machine environment without your development tools, credentials, browser state or build caches. The previous developer-machine unsigned installation is useful evidence, but cannot establish these results.

- [ ] **Setup agent + you:** Test the actual signed Windows x64 installer on the supported Windows versions/profile, without Node/Rust/Git/Visual Studio installed. Record OS, architecture, package digest, verified publisher/signatures/timestamps and WebView2 state. Check WebView2 present/missing behavior and standard-user installation.
- [ ] **Setup agent + you:** Open the installed app, close/reopen it, resume its local state and complete the agreed real journey. Check Credential Manager access, native file dialogs, recovery and local credential removal in that environment.
- [ ] **Setup agent + you:** Review minimum window size, high-DPI scaling, keyboard focus/navigation and critical errors. Record accessibility checks actually performed and any limits; do not copy demo/native smoke results as clean-machine evidence.
- [ ] **Setup agent + you:** Verify uninstall/reinstall behavior, registration/shortcuts and intended local-data retention/removal. Confirm it does not delete cloud resources. Cleanup must target only the test installation/profile.
- [ ] **Both agents:** Fix defects and repeat the affected checks against the resulting candidate. Mark `clean_signed_windows_install` and `clean_signed_windows_reopen` only with actual signed-machine evidence.

## 6. Public release and continued maintenance

- [ ] **You + both agents:** Review the final evidence record: actual web/Setup commits, contract version, app manifest/archive/baseline digests, installer hash and signature evidence, platform versions, results and known limits. Complete remaining fields in `docs/qualification.json` truthfully and run `npm run release:gate`. Its success does not replace owner release review.
- [ ] **You, with agent preparation:** Approve and publish the reviewed immutable app artifacts/channel and signed installer to the intended GitHub Releases locations, with clear version, checksums and release notes. Verify that a clean download contains the exact reviewed assets. A Git clone or workflow artifact is not the end-user release page.
- [ ] **Setup agent:** Keep README's end-user download/run instructions distinct from developer build commands. Explain that users supply their own cloud accounts and the web app keeps running after Setup closes. Do not promise automatic app/manager upgrades, repair, adoption or teardown in v1.
- [ ] **You + web agent:** Establish an owner and procedure for channel renewal, revocation and key recovery. The current publisher emits a channel valid for six days and the verifier caps validity at seven days: renew before expiry even without a feature release, using original immutable artifacts, a higher sequence and the previously authenticated channel. Do not recreate an existing app tag to extend its life.
- [ ] **You + both agents:** Maintain signing/account access, dependencies/security findings, provider API/OAuth changes, supported Windows/WebView2 behavior, compatibility pins and release evidence. Coordinate changes to manifest/schema/health/auth/environment contracts before shipping either side. Treat automatic updates/upgrades as future design work, not existing functionality.

## Progress and coordination record

### 2026-09-22 — continue prototype setup in actual Google External Testing

The maintainer confirmed External Testing is their existing prototype workflow after Google blocked publication on production Branding configuration. Setup now records explicit test-user/seven-day acknowledgment and accepts actual Testing at the native Google gate without claiming publication. Older checkpoints default the new acknowledgment to false. New-user UI follows External; saved Internal configurations remain compatible. The two supplied scope screenshots and exact-scope clipboard shortcut are included. Production branding needs real instance pages and remains a later, separate action.

Passed: dependency install/check/build, 29 UI tests, 38 native tests, all frontend profiles, five packaging tests and security scan. Paired qualification is being coordinated with the web-release task; record exact source pairing/results in the receipt. Real Google sign-in, refresh behavior and full provider/recovery/public qualification remain pending. No provider tokens, real database or computer-use tools were used for automated checks.

### 2026-09-22 — legacy token confirmed and Google walkthrough clarified

**Owner:** Setup agent. The maintainer reported Supabase profile refusal with the scoped token, then successful account connection using a legacy replacement. Token guidance now documents that exact path and its broad account scope. Database-region meaning, five Google steps, Web application/client name, audience/status distinctions, one-time credential capture and adjacent copy feedback are implemented. Source/runtime schemas, cloud requests, trust and app contracts are unchanged; the native refusal message was updated to match the guidance.

**Passed:** dependency install/check/build, 26 UI tests, 36 ordinary native tests with normal Windows vault access, three frontend profiles, five packaging tests and security scan. **Pending:** manual review of the rebuilt Alpha and the remaining real installation/Google sign-in/recovery checks. No computer use was performed. This UI revision does not inherit full paired qualification from the original candidate; public flags stay unchanged. The returned receipt identifies the new source and unsigned installer.

### 2026-09-22 — diagnose account refusal without guessing token scope

**Owner:** Setup agent. The maintainer encountered the generic authentication error while entering the Supabase token. The actual rejected provider/check remains unknown. Account discovery now reports one of four fixed Vercel/Supabase identity/list checks, preserving rejection and omitting raw provider bodies. No permission was broadened and no identity check was bypassed. The pre-creation error footer now describes the read-only account connection stage. See [security evidence](security-evidence.md#account-access-refusal-diagnostics--2026-09-22).

**Passed:** dependency install/check/build, 23 UI and 36 native tests, all frontend profiles, five packaging tests and security scan. **Pending:** maintainer retry with saved tokens in the rebuilt Alpha and the exact provider/check from the resulting message, followed by any evidence-based adapter or token-guidance correction. Full paired qualification was not rerun; signing/public qualification flags remain unchanged. No computer use or real account access was performed. New installer identity is in the returned diagnostic build receipt.

### 2026-09-22 — account/token sequence and scoped-token guidance

**Owner:** Setup agent, following the maintainer's real Alpha feedback. Configured setup now authenticates the release first, combines Vercel account/token on one page and Supabase organization/token on the next, and combines Google project/OAuth at the stage where the reserved callback is available. Each provider token is saved separately through existing native IPC and cleared from its input; saved tokens can be reused after reopening. Account discovery and explicit target/cost confirmation still precede project creation. Recovery can replace only an expired token without altering saved resources or app secrets.

Vercel guidance now links directly to personal tokens and distinguishes team settings. Supabase guidance maps the current scoped-token form to six documented permissions and Organization resource access; Database and Infrastructure permissions stay None. Expiry guidance distinguishes desktop management access from hosted runtime credentials. See [account-guide.md](account-guide.md) for the endpoint mapping and sources.

**Passed:** `npm ci`, TypeScript check, 23 UI tests, production frontend build, normal/testing/alpha build checks, five packaging-profile tests, source/CSP/capability scan, and 34 ordinary native tests (the sandbox blocked the vault test; the normal Windows-account rerun passed). New UI coverage includes vault save failure, rejected discovery after replacement, reuse of saved tokens after reopening, single-provider token replacement and no project creation before confirmation. The native engine, provider requests, trust, hosted-app contracts and qualification flags are unchanged; full web-owned paired qualification was not rerun for this frontend revision. The frozen native/web evidence remains associated with its original pair.

**Still to test:** the maintainer's live scoped-token account/project/key journey and revised desktop layout. No computer-use skill, real credentials, real cloud writes, installer launch or live provider test was used for this revision. The returned build receipt records the updated unsigned Alpha installer and source identity. Windows signing and unfinished public qualification stay deferred.

### 2026-09-20 — account walkthrough revision

**Owner:** Setup agent. Implemented separate Vercel, Supabase and Google Cloud preparation pages with progress, back navigation, concise stopping points and six screenshot placeholders. Clarified that Setup creates Vercel/Supabase projects, generates the database password and later configures the database; Google project ID and OAuth details are requested at their respective steps. Improved real token-entry instructions, require both tokens for initial connection, clear inputs on success/failure and require successful discovery plus explicit account confirmation before creation. These UI checks supplement the existing Rust authorization boundary.

Ordinary builds exclude testing UI and the demo engine. `desktop:dev:testing` / `desktop:build:testing` opt in and isolate application identity. The testing switch starts off; switching it off exits a running demo. Added build-variant assertions to CI and documented replacement of screenshot slots in [account-guide.md](account-guide.md). This work does not implement the older standalone-handoff assignments or change the separate web repository.

**Passed:** `npm ci`; `npm run check`; `npm test` (20); `npm run build`; `npm run test:build-profiles`; `npm run native:test` (34 ordinary tests with normal Windows-account access); `node scripts/native.mjs fmt --all -- --check`; `npm run security:scan`; `npm run desktop:build`; `git diff --check`. The restricted native suite first hit the expected Credential Manager sandbox failure; the complete normal-access rerun passed. Existing five opt-in app/PostgreSQL tests were not run. The ordinary native welcome/Vercel/Supabase screens and ordinary browser preparation journey were visually inspected. The release gate rejected all unresolved production prerequisites as expected.

**Artifact:** `src-tauri/target/release/bundle/nsis/Villow Setup_0.1.0_x64-setup.exe`, SHA-256 `21DE76A46E76880FA6E7379C77320DA8E01D78D908DE13EE9305BB71C36AE740`, Authenticode `NotSigned`. The packaged frontend excludes testing controls. The native executable was launched directly; a new installer install/uninstall smoke cycle, clean-machine/high-DPI/minimum-size qualification and the separate testing NSIS build were not performed. These changes are local working-tree changes, not a new published release.

**Still blocked:** A complete real-account walkthrough needs the authenticated app release and publisher configuration plus the established qualification process. The new build explains this before collecting tokens or asking the user to prepare accounts. Production trust, qualification flags and all cloud resources remain unchanged.

Add a dated entry for each completed stage or concrete blocker:

```text
Date / owner:
Item and outcome (passed / failed / not run):
Setup commit and web commit, where relevant:
Contract / app release digest / installer digest, where relevant:
Commands, environment and redacted evidence location:
Known limits or prerequisite:
Request to the other agent or maintainer, and its confirmed response:
```

Initial entry, 2026-09-20: this handoff/checklist was prepared in the web workspace's local Setup copy for transfer to the standalone repository. No destination CI run, paired-harness fix, new native test, real cloud action, signing enrollment or public release is claimed by this documentation change. Historical evidence remains in [desktop review](desktop-review.md) and [security evidence](security-evidence.md).

### 2026-09-20 — standalone Setup working tree

**Owner:** Setup agent.

**Completed:** The Git root is `C:\Users\Nebula PC\Development\Villow-Setup`, branch `main`, with `origin` set to `https://github.com/nicholasnucifora/Villow-Setup.git`. The package/Cargo lockfiles, hidden configuration, workflows, fixed brand assets, docs, scripts, frontend, tests and native source are present at that root. The allowlist contains 105 source files. The path audit found no build or runtime dependency on the former nested checkout; retained `villow-setup/` strings are archive-exclusion security fixtures or historical handoff evidence. The development identifier remains `app.villow.setup.dev`; crate/package names and production trust gates remain unchanged.

Current guidance now treats this checkout as standalone and links the handoff and this checklist. The source move was not used to populate `src-tauri/trust.json` or `docs/qualification.json`.

The checked-in workflows are root-relative and use the root npm/Cargo lockfiles. Ordinary push/PR CI declares read-only contents permission and has no publisher variables or privileged signing runner. The manual release workflow requires a version tag, runs `release:gate` before signing, and confines signing to the `production-release` environment on the dedicated `villow-signing` runner; it uploads a review artifact but has no release-write permission. This is source review only: the maintainer must still verify actual branch rules, environment approvals, runner registration/isolation, variables and secrets in GitHub.

**Dependency finding and remediation:** The first current `cargo audit` found medium-severity RUSTSEC-2026-0285 in `rustls 0.23.44`. `src-tauri/Cargo.lock` now pins the compatible patched `rustls 0.23.45`. The rerun reports no known vulnerabilities and seven allowed transitive warnings: unmaintained `proc-macro-error` and `unic-*` packages plus the `glib` iterator unsoundness advisory in Tauri's cross-platform GUI dependency graph. These warnings remain review backlog; they are not represented as resolved or as clean signed-Windows evidence.

**Passed on Windows 10.0.26200.9457 AMD64, Node 22.17.0, npm 10.9.2, Rust/Cargo 1.90.0:** `npm ci`; `npm run check`; `npm test` (15 tests); `npm run build`; `npm run native:test` with normal Windows-account access (34 ordinary tests, including Credential Manager); `npm run native:check`; Rust formatting check; `npm run security:scan`; `node scripts/mutation-check.mjs`; `npm audit --audit-level=moderate` (0 vulnerabilities); patched `cargo audit` (0 vulnerabilities, 7 allowed warnings); `npm run license:inventory`; `npm run desktop:build`; `scripts/verify-dev-package.ps1`; and `npm run portability`. The refreshed inventory still has 370 Cargo packages, six non-development npm packages, no missing license declarations and 12 notice locations requiring review; regeneration is not completion of that review. The final portability run passed all six stages for 105 files in `C:\Users\Nebula PC\AppData\Local\Temp\villow-extracted-muEgbA\project`, with isolated npm/Cargo stores and `parent_dependencies_used: false`. Redacted machine evidence is in ignored `artifacts/portability.json`, `artifacts/integration-package-smoke.json` and `artifacts/dependency-license-inventory.json`.

The restricted-sandbox native run first failed only the Windows Credential Manager round-trip with `Error::Vault`; the same complete suite passed with normal Windows-account access. The first portability attempt stopped in isolated `npm ci` with npm's own “Exit handler never called” error; two normal-access reruns passed, including the final patched lockfile run.

**Development package:** `src-tauri/target/release/bundle/nsis/Villow Setup_0.1.0_x64-setup.exe`, SHA-256 `425E1D21861FF244DAEFCCD7BE4A961C7E6759369CCD4788DF36A9F77968A853`, Authenticode `NotSigned`. Its MIT license, privacy notice and development signing policy were installed; the application stayed running on two launches; its own uninstaller removed the isolated directory and current-user registration. `visual_verification` is false.

**Expected rejection:** `npm run release:gate` rejected empty production publisher/key configuration and every unfulfilled real-provider, owner, signed-Windows and evidence-record gate. It was not weakened.

**Not run / still pending:** the app-owned `app_release` fixture test and four explicitly disposable PostgreSQL tests were not rerun because no app/database contract changed and the separated web harness is not yet wired. Hosted GitHub Actions, branch protection, protected environment, signing-runner settings and repository secrets were not inspected through the GitHub account. No clean-machine visual/accessibility check, real provider/OAuth operation, signed package, SignPath enrollment, public artifact, cloud change or publication occurred. HEAD was still the initial commit `5a34417b95f704b2b05085557e221bcfbd514836` during verification; the standalone source and this evidence were working-tree changes, so the maintainer must review, commit and push them before hosted CI or another clone can rely on them.

**Paired web status:** Not implemented or verified. No web checkout/commit was available in this task. The old nested copy must not be deleted solely on the strength of this local pass; wait for the paired harness result and maintainer confirmation.

### Pasteable request to the Villow web agent

> Setup is now a standalone checkout at `nicholasnucifora/Villow-Setup`. Please replace the implicit `root/villow-setup` lookup in the Villow web release harness with this concrete qualification interface:
>
> `npm run release:check -- --setup-root <path> --setup-commit <40-character-SHA>`
>
> `scripts/test-app-release.mjs` should parse both required options in qualification mode, resolve `--setup-root` to a real directory, require `package.json` with `name: "villow-setup"` plus `src-tauri/Cargo.toml`, require a clean Git checkout, and verify `git rev-parse HEAD` exactly equals `--setup-commit`. Pass the same validated root to `scripts/setup-schema.mjs --check --release <fixture-directory> --setup-root <path> --setup-commit <SHA>`; do not rediscover a sibling directory there. Missing options, a nonexistent or wrong project, a dirty checkout, SHA mismatch, missing verifier test, or a failed native/database verification must make required release qualification exit nonzero. If an app-only developer command is retained, give it a separate explicit name, report `native_verifier: false` and `native_database_install_resume: false`, and never let it satisfy `release:check` or release qualification.
>
> In `.github/workflows/app-release.yml`, add a second `actions/checkout` for `nicholasnucifora/Villow-Setup` at one reviewed full SHA (never moving `main`), with `persist-credentials: false`, into an explicit qualification path. Install that checkout's own locked dependencies/toolchain, pass its real path and pinned SHA through the interface above, and record the web SHA, Setup SHA and synthetic fixture/app source identity separately in evidence.
>
> Required tests: (1) required qualification fails when `--setup-root` is omitted; (2) missing path, wrong project shape, dirty checkout and SHA mismatch each fail; (3) an explicit app-only command, if provided, succeeds only with visibly incomplete native evidence; (4) the real pinned checkout authenticates the generated archive/manifest and runs Setup's ignored `app_release` verifier; (5) disposable PostgreSQL baseline install, interruption/resume, history and postcondition checks pass; and (6) both native evidence fields are true only after those real checks pass. Keep fixtures and databases disposable, preserve production trust separation, and do not copy web SQL into Setup.
>
> Please return the web repository branch and exact commit, changed files/PR, pinned Setup commit, supported contract/manager versions, rollout order, exact commands and redacted results. The current verified Setup working tree is based on `5a34417b95f704b2b05085557e221bcfbd514836`, but its final commit does not exist yet; pin the reviewed commit produced after these changes, not that initial placeholder commit.
