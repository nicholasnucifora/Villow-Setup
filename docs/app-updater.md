# App updates — Setup Alpha 0.2.0

This candidate adds **Check for updates → Update my app** to a completed,
locally owned Alpha installation. Reopening a pending update offers **Resume
update** for the same release. Windows signing remains deferred. Genuine app
0.1.2 remains published; synthetic 0.2.x fixtures are never public releases.
Real-provider updating and recovery remain unqualified.

## Signed contract

Manifest format 1 gains optional `app_updates`. Plans and `upgrade_from` match
one-to-one: at most eight exact source manifest hashes and 32 transactional units
per plan. Each plan binds source/destination schemas, source recovery inventory,
precondition, ordered SQL and final postcondition, and requires
`backup_required:true`, `previous_app_compatible:true`, minimum Setup 0.2.0.
Top-level `backup_required:false` continues to describe fresh installation.
Version and sequence must increase; changed configuration, permissions, build,
bootstrap or health contracts are refused. Code-only plans have no SQL units and
preserve the exact baseline/postcondition bytes. Existing databases never receive
a fresh baseline as an update. New nondeploy roles are `update`,
`update_precondition`, `backup_descriptor`. Every file retains authenticated
hash/size checks. Plan hashes use compact UTF-8 JSON, recursively sorted object
keys and unchanged array order.

Both releases must remain authenticated and unrevoked in a fresh channel at
start/resume. Public availability needs no provider token and never grants writes.
Effects require original credentials, matching provider targets and source pin.
Original encryption and bootstrap secrets are never regenerated.

## Protection and history

The destination-signed source descriptor binds exact public tables, ordered
columns and user triggers to the source baseline and final postcondition.
Native code owns and checks private ledger schema/ACLs/keys/defaults. Unsupported
relations, types or dependencies stop the operation. This first restore contract
requires one fresh baseline with empty tables after reconstruction, no identity
or generated columns, and no forced RLS. Foreign-key cycles are refused.

Encrypted format-2 recovery packages contain source rows, eight original vault
credentials, checkpoint/configuration, signed channel and both release archives.
The destination archive preserves the authenticated recovery descriptor. Native
paths use `app-update-backups/<installation UUID>.villowbackup`; a separate
Windows-vault key unlocks them. Limits remain 128 MiB of decoded COPY data,
256 MiB packaged plaintext and 32 MiB per compressed release. No end-user backup
password or developer signing key is requested.

Original fresh migrations and Alpha repair rows stay intact. The append-only
`villow_setup.app_updates` receipt chain records operation, plan, source,
destination, unit hashes and previous receipt/base anchor. All update SQL,
checks, receipt and instance digest CAS commit together. A lost SQL response
reconciles the exact receipt; a lost deployment response reconciles saved
operation/release metadata rather than submitting a second deployment.

Authenticated health and durable completion precede file deletion, then key
deletion. Failures retain protection; cleanup retries on open. Credential removal
and forgetting cannot strand pending protection. Recovery JSON strips update
intent/history and stays read-only. Empty-target restore is assisted, with no
renderer command or permission to overwrite populated data. Later user activity
is not in the snapshot; automatic rollback after later writes is not promised.

## Verification

UI, strict manifest/transition and `managed_update_backup` tests run normally.
The opt-in app-owned fixture uses disposable PostgreSQL only:

```cmd
node scripts/test-postgres.mjs --app-update-fixture "<synthetic fixture directory>"
```

`app_update_protocol` checks fresh and repaired source histories, code-only,
additive, repeated and explicit skipped transitions, populated data and original
credentials, rollback, lost SQL/deployment responses, reopening, failed-health
retention, recovery authority stripping, cleanup and empty-target restore. It
emits `APP_UPDATE_LIFECYCLE_PASSED` after actual engine coverage. The web paired
runner also exercises old app handlers on the new schema and requires exact clean
commits. Local tests do not certify live providers or Windows installation.

Synthetic keys remain explicit test inputs. Runtime trust, public release assets
and unfinished qualification flags are unchanged.
