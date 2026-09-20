# Maintenance, recovery and release responsibilities

## What exists now

Setup versions, app release versions and schema identities are separate. New app feature releases can use the existing manager while they satisfy the same environment/bootstrap/health/archive/schema contracts. The first manager version automates **fresh installations only**. It does not apply app updates, adopt existing databases, delete cloud resources, register protocol handlers or install its own updates.

The GUI provides recovery export, read-only import, local credential removal, local forgetting and provider-dashboard links. A checkpoint is not a backup of application data. Restoring it does not recover a missing encryption key. The hosted instance survives desktop uninstall and computer shutdown; provider charges and renewals remain active.

## Recovery behavior

- Pausing between effects is safe. A running effect may finish remotely even if the process is closed. On reopening, resume reads actual account/resource state before further writes.
- Expired access requires reauthentication to the recorded accounts. A changed principal or team is refused. Offline/rate-limited reads preserve progress and use bounded backoff; writes are not blindly retried.
- If creation succeeded but no resource ID was received, inspect the planned name, account, timestamp and provider dashboard. A matching name alone does not confer automatic ownership. The live adapter stops for review; the recovery form accepts an explicit project ID and typed planned name, verifies the authenticated account and returned ID/name/creation time, then records your confirmation as provenance. An uncertain creation without that evidence remains stopped. The fake-provider test separately proves the stronger operation-ID reconciliation case.
- Missing encryption/bootstrap/database credentials after any effect stop setup. Restore the original value from its original secured source; do not click through by generating a new key. Keeping the generated encryption key in the owner’s Vercel environment is essential to the hosted app.
- Removing saved credentials is a local vault operation. Revoke the management grants in each provider separately. Google client-secret copies needed by the app remain in the owner's hosting project.
- Forgetting removes the checkpoint only after vault deletions succeed. The durable release anti-replay state is retained. Uninstalling does not perform provider revocation or cloud deletion.

## Future operation contracts (not exposed as buttons)

**Repair/adoption:** start read-only. Match authenticated account and resource IDs, explicit ownership/provenance, the app descriptor, real schema and checksummed history. Treat recovery files as hints. Empty new, known manager-created and manually maintained databases are distinct. Ambiguous or drifted history needs reviewed reconciliation, not a reset or fabricated ledger.

**Hosted app update:** pin an authenticated release before effects; compare upgrade origins and compatibility windows. Verify backup availability on the owner's actual plan, export any state required to recover, and test restore in a disposable environment. Build before promotion when possible. Expand schema compatibly, preserve old code compatibility, promote deliberately, defer contraction. Code rollback does not undo SQL; never expose an unconditional “undo update”. Preserve the app encryption secret and correctly bump installed PWA shell identity in the app release. New interruption, data-preservation and partial-nontransactional tests are mandatory.

**Manager update:** initially an official, publisher-signed manual installer. In-app updating is deferred until Tauri updater signatures, pinned endpoints/key rotation, version progression, replay protection, interrupted install and clean install/upgrade tests have been qualified. Checkpoints and the Windows vault service namespace must remain compatible. A new app release must not force a new manager unless its contract changes.

**Custom protocol:** only a user-clicked, nonsecret identifier opening a known installation's review page. Unknown origins, payload scripts, credentials and resource mutations are never allowed. Mobile users get download/instructions, not a management grant.

**Cloud removal:** exact stable IDs, names and accounts, proven ownership and current exclusivity, backup/export with a tested restore plan, explicit destructive confirmation. Shared/reused resources default to retained. Persist partial results and reconcile retries. Revoke grants only after they are no longer needed. Do not delete accounts/organizations, infer ownership from a name, or imply project deletion cancels unrelated subscriptions, domains or billing. Add partial-teardown and malicious-target tests before shipping a teardown adapter. Currently there are **no cloud DELETE methods or teardown IPC commands**.

## Practical maintenance assessment

| Area                 | Work and exposure                                                                                                                                                                                                           |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Initial engineering  | Qualify the app release/baseline/bootstrap, run actual disposable provider journeys, establish publisher identity, sign/install/reopen the EXE, review native UI boundaries and resolve findings                            |
| Each manager release | Review dependency/advisory changes, run native/UI/SQL/mutation/portability and Windows install/reopen tests, check signing/timestamp/publisher identity, publish immutable artifacts and inspect release notes              |
| Each app release     | Test app build and schema compatibility, produce authenticated source/archive/checksums/notes, validate required manager contract, refresh the signed channel and revocation list                                           |
| Routine maintenance  | Refresh channel validity before its seven-day maximum; monitor provider docs and token/key-type changes through maintainer workflows, not an owner-device background grant keeper                                           |
| Vendor charges       | Code-signing provider, CI runners/artifact storage and domain/distribution costs belong to the maintainer. Vercel/Supabase/Google usage and any backups/addons belong to each instance owner                                |
| Unexpected support   | Identity verification, enterprise policies, IPv4/IPv6/firewalls, WebView2/SmartScreen, provider outages/quotas, expired OAuth access, lost computers, project transfers, old schema drift and consent-screen policy changes |

No precise maintenance-hour estimate is supportable from this implementation. Plan a small recurring release/security responsibility plus unpredictable onboarding support. There is no background service monitoring everyone’s deployments or keeping management grants alive.
