# App integration: implemented, awaiting external qualification

Updated 2026-09-11. The user authorized changes to the Villow application while Setup was still nested. The app-side code previously requested by this document has now been implemented, and Setup has since moved to its standalone repository. This is no longer a list of coding tasks for the user.

| Contract | App implementation |
| --- | --- |
| Authenticated source release | `scripts/app-release.mjs`, `scripts/sign-app-channel.mjs`, root protected review-only workflow, explicit archive allowlist and `.vercelignore` |
| Fresh database | Data-free `villow-fresh-158` baseline, full-filename history mapping, transactional replay and catalog postcondition |
| Intended owner | Migration 158 and `lib/main-oauth.ts`; verified Google subject/email, expected-owner gate and atomic bootstrap closure |
| Safe OAuth exchange | Expiring browser-bound state, PKCE, encrypted single-use HttpOnly exchange, no credentials in redirect URLs |
| Session/token security | Verified app sessions, encrypted stored Google tokens, compatible plaintext reads during rollout and stable instance key |
| Build/health | `GET /api/setup?action=build`, `POST /api/setup?action=health`, routed through the existing settings function |
| Canonical origin/cron | Explicit configured origin, exact CORS, unchanged 12-function budget and separate `CRON_SECRET` |

See [the versioned wire contract](villow-integration-contract.md) for the manager interfaces. The parent app's `SETUP_INTEGRATION.md` records implementation, rollout order, release operation and actual verification. This reference is documentation only; Setup has no runtime dependency on that file or checkout.

## Existing deployed apps

Apply **only migration 158**, after the existing history, before deploying the changed authentication. Preserve the existing `ENCRYPTION_KEY`. Do not apply the fresh baseline to an existing database or add Setup bootstrap variables to adopt one. Existing deployments can enable complete API session enforcement with `VILLOW_REQUIRE_APP_SESSION=true` after the authentication rollout; that also requires a separate cron secret.

Old plaintext-only app code cannot read tokens written by the new encryption wrapper. A rollback needs compatible readers and the same key, or a separately planned data recovery. No live migration or deployment was performed in this work.

## What remains before public use

The implementation and synthetic local checks do not establish a production publisher or prove provider behavior. Distribution coordinates now point to the user-selected `nicholasnucifora/Villow-Setup` repository. Publisher identity and public keys remain unset, so Setup creation stays disabled until verified signing and qualification evidence exist.

1. Review and publish the actual immutable app artifacts and signed channel; configure the verified publisher and public keys for the chosen `nicholasnucifora/Villow-Setup` distribution repository.
2. Authorize disposable provider accounts, cost and cleanup scope, then qualify the no-GitHub end-user install, intended-owner Google consent, authenticated health, interruption/resume and credential removal.
3. Qualify a properly signed package on clean Windows profiles and inspect the actual UI, keyboard/high-DPI behavior and WebView2 paths.

Automated app upgrades, existing-database adoption, cloud teardown and writable recovery after local-state loss remain outside version one. Do not mark public qualification fields true based on mock success.
