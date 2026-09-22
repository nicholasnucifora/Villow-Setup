# Maintenance, recovery and release responsibilities

## What exists now

Setup versions, app release versions and schema identities are separate. New app feature releases can use the existing manager while they satisfy the same environment/bootstrap/health/archive/schema contracts. The first manager version automates fresh installations; Alpha 0.1.3 additionally supports the narrow authenticated correction for the known unfinished installed Alpha. It does not implement general app updates, adoption, cloud deletion, protocol handlers or its own updates.

The GUI provides recovery export, read-only import, local credential removal, local forgetting and provider-dashboard links. A checkpoint is not a backup of application data. Restoring it does not recover a missing encryption key. The hosted instance survives desktop uninstall and computer shutdown; provider charges and renewals remain active.

## Temporary protection before an installed Alpha repair — manager 0.1.5

**Repair my app** creates and checks an encrypted recovery copy automatically.
Setup includes the existing app key and keeps a separate random unlock key in
Windows Credential Manager. No password, backup file selection, provider export
or developer signing passphrase is needed.

The temporary copy is stored in Setup's local data directory. Setup reads
a consistent snapshot of all 60 app tables and its two installation-history
tables over the existing certificate-verified connection. It includes the
original eight vault credentials, configured environment, account/resource IDs,
checkpoint, original manifest/archive and signed channel provenance. No provider
CLI, Docker, Vercel key retrieval or database-password change is needed.

Only encrypted bytes are written. Setup finishes, closes, reopens, decrypts and
compares the complete file before saving a receipt and starting repair. Vault
failures, partial writes, changed files and unsupported layouts stop the operation.
The same receipt/file/key is rechecked on resume. The copy remains after failure
or interruption, and is automatically removed only after the repaired website
passes authenticated checks and success is saved. Cleanup failures retry on open.
Portable backups created in 0.1.4 are retained. A repair started in 0.1.3 keeps its
explicit manual confirmation, clearly labelled as unverified; it is not upgraded
to a fabricated automatic backup. New repairs require a native backup receipt.
The original encryption key is retained throughout; restoring it is essential to
reading encrypted account connections.

This is a snapshot, not continuous protection: activity after capture, remote
Google/YouTube changes, revoked tokens, manually edited provider configuration,
Supabase Storage/Auth, browser storage and other cloud services are not backed
up. The supported app version does not use Supabase Auth/Storage for its data.
Unknown objects or dependencies are refused rather than silently omitted. The
Alpha supports at most 128 MiB of decoded app COPY data and a 256 MiB recovery
package. It does not promise zero possible data loss.

**Restore is still assisted.** Native restore is qualified against an empty
local database using the exact original app release. It preserves every row,
original ID, ciphertext and native ledger while keeping foreign keys active;
any failure rolls back the transaction. It cannot overwrite a populated app.
There is no restore/adoption button or generic SQL/file IPC in this Alpha. If
recovery is needed, keep this Windows account and Setup's saved local data and
request maintainer assistance; do not post backup files or keys in a support chat. Independent account
and target authorization and current release trust are required before any real
restore. A successful file check is not a live-provider restore drill.

The nonsecret recovery JSON remains a separate, read-only resource inventory.
It contains neither app data nor backup/repair authority and does not replace
this encrypted file. See [the backup contract](backup-contract.md) for the
format, supported release pair and test procedure.

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
