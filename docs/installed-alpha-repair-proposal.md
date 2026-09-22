# Installed Alpha correction: proposed Setup constraints

Historical design proposal; the agreed source implementation is documented in [the integration contract](villow-integration-contract.md). Live execution remains pending.

2026-09-22. Design review for the web agent's `codex/hosted-setup-repair`
candidate. **Not agreed, implemented, signed, published or authorized for live
execution.** Existing no-upgrade/no-adoption behavior remains in force.

## Narrow eligibility

Support only a known manager-created Alpha installation whose database migration
completed and whose first hosted setup remains unfinished at Health. Require
the original writable checkpoint, installation ID, full vault credentials,
selected provider identities, exact project IDs and origin. Verify the saved
deployment's project, production target and original operation/release metadata.
An older Health checkpoint can first acquire readiness evidence through 0.1.2's
existing read-only deployment check.

Reject imported recovery, missing secrets, another pending operation, Complete,
arbitrary databases, unknown release digests, unexpected history or schema,
resource/account changes and ambiguous deployment provenance. No new resource,
owner, password, key or Google client is created. Additional installed states
need a separately reviewed use case, not silent fallback.

The initial exact source is published app 0.1.1 manifest
`37072f3ef4efc387b57f6edcccdfd0e71b09fc61abcaee4649f9aba76fa332ac`.
Do not infer permission from a version string, schema revision, project name or
new channel recommendation alone.

## Authenticated correction description

Propose a separate optional manifest field, provisionally `installed_repairs`,
with one entry for this source manifest, a unique repair ID, one transactional
SQL file and a result-check file. Authenticate these through the destination
manifest/archive file hashes and the existing signed channel. This is separate
from `fresh_retry_from` and general `upgrade_from`. Agree exact names, roles and
limits before implementation; require a new minimum manager version so older
managers cannot consume the extension as a supported release.

The destination still contains its ordinary fresh baseline for new resources.
The installed-repair entry MUST NOT invoke that baseline. Its SQL only expands
the existing schema and deliberately updates the app-owned probe/attestation as
needed. The old and corrected schema must support the old code during the
deployment interval; no destructive contraction or secret rotation.

Configuration classifications, Google scopes, origin, owner and bootstrap/health
contracts stay unchanged for this bounded case. App release packaging should
pin and test its supported Node major; Setup should not rewrite authenticated
source or invent an independent runtime version.

Both releases must remain available and unrevoked in the same freshly verified
channel for initial execution AND interrupted resume. Ordinary expiry/replay
rules still apply. Keep original assets immutable.

## Database transaction and lost-response recovery

1. Record explicit owner-selected intent locally before any mutation, including
   source/destination manifest digests, repair ID and a fresh operation UUID.
   Ordinary advance and competing correction operations are blocked while this
   intent exists. Revalidate providers and required vault entries first.
2. Use the existing certificate-verified SQL connection, advisory lock and bounded
   lock/statement timeouts. In one transaction, lock native instance/history
   tables and the app's owner-registration state. Prevent concurrent schema
   changes to the affected objects during verification/application. Locking and
   old-app concurrency behavior must be included in qualification.
3. Verify exactly one matching `villow_setup.instance`, the old release digest,
   the COMPLETE old migration ledger (IDs and SQL/postcondition hashes), and the
   authenticated old schema/postconditions. The old API cannot supply this
   evidence because its function currently cannot load. Verify
   `public.villow_installation` and the intended owner against the checkpoint;
   preserve the owner ID, Google ID, closed bootstrap and all existing users/data.
   Initially support the already-signed-in owner case reported here. An empty or
   partially claimed bootstrap state needs an explicit separate predicate/test.
4. Run only the signed additive repair and its authenticated final result check.
   Preserve every original migration row and checksum. Insert a separate native
   repair receipt binding installation, source, destination, repair ID and file
   hashes. Compare-and-swap `villow_setup.instance.release_digest` from old to new
   in the SAME transaction. App SQL must not fabricate/delete native history.
5. The destination result check must prove the corrected catalog/ACLs and
   app-owned probe agree with the destination's fresh schema and health identity.
   Preserve public bootstrap/owner data. Historical old fingerprint predicates
   generally become false after an additive change: retain their hashes as
   history, but do not rerun ordinary fresh `apply()` against the repaired schema.
6. On COMMIT-response loss or local-save failure, reconnect under the same locks.
   Accept exactly either old digest + unchanged complete old ledger/schema + no
   repair receipt (safe to execute), or new digest + exact repair receipt + old
   ledger unchanged + destination result check (already committed; do not execute
   again). Any mixed state stops for review. Resume reauthenticates the original
   pending destination rather than following a newer channel recommendation.

Require disposable populated-database evidence that rows, owners, encrypted
tokens, RLS/grants and native history survive. Do not present a local checkpoint
as a database backup. The web proposal must state the actual backup/recovery
requirement before owner approval; Setup cannot claim a provider-plan backup or
restore was checked when it was not.

## Deployment and local state

After confirming the database commit, record the new local release/schema pin
while retaining the repair source, old deployment and operation history. Keep
the existing provider resources, origin, OAuth configuration and vault namespace.
Reuse existing environment values unless the agreed contract explicitly needs
an update; do not generate replacement encryption/bootstrap/database secrets.

Upload the authenticated destination's deploy files and submit one replacement
production deployment to the SAME Vercel project. Persist a separate repair
deployment intent/operation ID before POST. Never clear/reuse the original
deployment effect as if it had not happened. A lost response requires exactly
one matching deployment with the new operation ID AND destination digest,
followed by project/production identity verification; absence or ambiguity stops
instead of causing another POST. Keep the old deployment as provenance and for
review; do not automatically roll back code or SQL.

Poll the saved new deployment and exact production alias with the existing
readiness discipline. Failed build, pending alias, HTTP error and elapsed wait
remain visible and resumable. Complete only after the original authenticated
health contract passes for the new app commit/version/schema, same installation
and owner, same origin, real Google authorization and cron configuration.

The typed UI should expose one review/apply-or-resume action with visible repair,
build and health progress. No generic SQL, URL, filesystem or shell IPC. Recovery
exports remain nonsecret, imported state remains read-only, and credential
removal/forgetting must not turn unresolved repair intent into write authority.

## Required agreement and evidence before implementation

- Exact additive SQL, final schema identity, result-check semantics and supported
  source digest; whether final schema is equivalent for fresh and repaired installs.
- Proposed manifest fields/roles, minimum manager, native repair ledger shape,
  permitted public owner state and backward-read compatibility for checkpoints.
- Native tests for wrong target, drift, changed old ledger, absent secrets,
  imported state, competing locks, rollback, lost COMMIT response, reopened local
  state, repeated resume and no duplicate Vercel POST.
- App tests cold-loading emitted handlers on the pinned runtime and exercising
  settings/onboarding/watch-time against both actual fresh and repaired schemas.
- Paired qualification on exact frozen commits; reviewable owner rollout and
  actual backup/recovery requirements before signing/publication/live execution.

No manager implementation or additional release signing is requested by this
design document. It identifies the work required after both agents agree the
contract and the maintainer authorizes the concrete rollout.
