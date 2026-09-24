# Hosted setup failures: web-app handoff

This records the initial diagnosis. Later 0.1.3 repair implementation is described in [the integration contract](villow-integration-contract.md).

Prepared 2026-09-22. Diagnosis only; no web source, cloud resource, user database,
credential, published asset or Setup runtime was changed.

The maintainer has successfully prepared their dedicated Alpha database and
reached the deployed website. After adding the intended Google account as a test
user, sign-in reaches `/setup`, which reports “Setup could not load”. Supplied
Vercel logs show two independent server failures. This is now an installed
database, so the earlier correction-before-installation flow is inapplicable.

## Reviewed identities

- Published app: `0.1.1`, web commit
  `b17cc0cfb4a2928f81aae5f42f08797e0cb7d124`.
- Published manifest SHA-256:
  `37072f3ef4efc387b57f6edcccdfd0e71b09fc61abcaee4649f9aba76fa332ac`.
- Published archive SHA-256:
  `714e95039455664e38e3810eecce5582a53f822cbd1cd8cb18d0e818cb055a9a`.
- Current built Setup Alpha: `0.1.2`, source
  `e4be04bd440d5ad7a664af37f1e987374dd0dc12`.

The cached public archive hash was rechecked before inspecting its actual
`lib/api/setup.ts`, `api/watch-time.ts` and `migrations/fresh-baseline.sql`.
Source findings also match the immutable web checkout used for paired
qualification. No access to the maintainer's deployed function files or database
was performed; runtime observations come from their supplied logs.

## 1. Settings and Setup function cannot cold-load

Vercel reports `ERR_IMPORT_ATTRIBUTE_MISSING` for `/var/task/vercel.json` while
loading `/api/settings`. The released `api/settings.ts` statically imports
`../lib/api/setup.js`. At line 6 of the corresponding released source:

```ts
import vercelConfig from '../../vercel.json'
```

The package uses `type: module`. A separate local Node 22.17.0 probe reproduces
the exact error for this JSON-import form; the same JSON loads with the required
import attribute. The probe executes only an isolated import, not the app's
handler, and does not prove how the full Vercel compiler emits that handler.
The provider logs supply the actual cold-load failure evidence.

Fix the app-owned runtime import and test the emitted server entrypoint under
the supported deployment runtime. Adding an attribute is a possible solution,
but validate compiler/bundler support and JSON packaging; an alternative must
retain genuine cron/configuration validation in Setup health. Do not substitute
a hard-coded successful health result. `/api/setup` shares the settings function,
so this dependency also prevents its build and authenticated health handlers
from loading normally.

Neither the released package nor Setup's project/deployment request pins a Node
major. Establish a supported runtime in the app release and qualify it. Merely
downgrading Vercel to Node 22 does not solve this import: the local Node 22 probe
fails too. The user's actual deployed Node version has not been inspected.

References: [Node JSON import attributes](https://nodejs.org/api/esm.html#import-attributes),
[Vercel Node version configuration](https://vercel.com/docs/functions/runtimes/node-js/node-js-versions).

## 2. Released database and watch-time code disagree

The user's `/api/watch-time` logs report stage `settings_read`, SQLSTATE `42703`,
and missing `user_settings.daily_watch_time_seconds`.

The published baseline's `CREATE TABLE public.user_settings` omits both
`daily_watch_time_seconds` and `daily_watch_reset_at`. The released handler
explicitly selects both (including its main settings query around source line
870). Functions in the baseline also reference those fields. This is a defect
in the released schema/code pair, consistent with the supplied live error, not
evidence that the user entered the wrong database password.

Audit the historical schema inputs and all runtime-required fields. Do not
assume adding just the first missing column finishes the repair. Review suitable
types, defaults, existing rows, constraints, RLS, grants and function behavior.

## Requested web-app work and rollout contract

1. Fix the server module-loading error and schema/code mismatch in the web repo.
   Add regression coverage that cold-loads emitted API handlers outside the
   test bundler and exercises settings/onboarding plus watch-time against the
   actual generated fresh schema. Retain the native baseline/history/ACL checks.
   Previous fingerprint equivalence proves faithful replay, not that every
   runtime query is compatible with that schema.
2. Prepare a reviewed, data-preserving correction plan for this already
   installed Alpha instance. Preserve its owner, accounts, OAuth settings,
   encryption/bootstrap secrets, data, project IDs and website address. The plan
   must explicitly handle the pinned release digest, schema attestation and
   migration history, as well as redeploying corrected app code.
3. Coordinate that plan with Setup before changing its contract. Setup currently
   has no installed-app repair/upgrade path. `fresh_retry_from` permits a switch
   only before any app unit commits; it rejects a verified migration or saved
   deployment. Do not erase history, rerun the fresh baseline, pretend this is an
   uninstalled database, or weaken checks to reuse that path.
4. Return the exact web commit, supported runtime, tests, proposed rollout steps
   and any narrowly defined Setup work. A new immutable app release, genuine
   signature and owner-approved publication are separate steps after review.
   Preserve existing published assets and false qualification flags. Do not
   request private signing keys or passphrases.

Until that plan is implemented, rebuilding the current Setup EXE or repeating
the current Vercel deployment will install the same defective app release. The
maintainer should retain the current resources and avoid repeating database
preparation. Windows Authenticode signing remains unrelated and deferred.

Local evidence is in ignored `artifacts/hosted-setup-failure/inspection.json`
and its isolated `inspect.mjs` probe. No computer-use or live provider mutation
was used. Hosted onboarding and final authenticated health remain unqualified.
