# Encrypted pre-repair backup, format 1

Manager 0.1.4, 2026-09-23. This extends the bounded installed Alpha repair;
it does not add general upgrades, provider-wide backup or writable adoption.

## Supported identity and contents

The native contract is pinned to original manifest
`37072f3ef4efc387b57f6edcccdfd0e71b09fc61abcaee4649f9aba76fa332ac`
and repair destination
`46da117bce94a5af20f6680a1188a804866aec4ca302f9862686c2e869e5c78a`.
Production trust is unchanged. Both pointers must be present and unrevoked in a
fresh authenticated channel. A new app schema/pair requires contract review.

`backup-tables-0.1.1.json` and `backup-triggers-0.1.1.json` record the reviewed
app-owned inventory: 60 public tables, two native tables, 71 foreign keys, 16
ordinary user triggers, no identity/generated columns or sequences. They were
derived from the exact original release in a disposable PostgreSQL 17 database,
then verified by the native fixture. They are schema metadata, not SQL baselines
or production trust. Runtime/builds never read the web checkout.

The signed app postcondition, pinned inventory, native owner/history checks and
supplemental catalog checks refuse schema/ACL/RLS drift and unsupported objects,
types, rules, sequences, publications, large objects, inheritance, forced RLS,
changed triggers and catalog-recorded cross-schema dependencies. Dependencies
inside arbitrary dynamic SQL are not globally discoverable through PostgreSQL's
catalog; this is an exact app contract, not a generic completeness guarantee.

Capture holds the local operation lock and the native session advisory lock.
One read-only repeatable-read transaction locks the fixed table set in ACCESS
SHARE mode and verifies source/owner/ledger before COPY. UTF8, UTC, ISO YMD,
postgres intervals, exact float output and hex bytea are explicit. Columns are
explicit and rows have deterministic text ordering. No SQL data passes through
JavaScript number conversion. Concurrent later writes are outside the snapshot.

The package contains capture time, a new repair operation ID, destination digest,
original checkpoint, all eight original vault credentials, the values Setup
configured in Vercel (including its existing cron derivation), original raw
manifest/archive, signed channel provenance and all table COPY bytes/counts.
Optional checkpoint fields retain their original null/absence. It excludes
provider Auth/Storage, external service mutations, manual provider changes and
browser storage. The reader must never promote imported checkpoint effects,
credentials or provenance into current write authorization.

## File encoding and persistence

An AES-256-GCM authenticated file uses the maintained `ring` implementation.
The fixed 52-byte little-endian header is authenticated as AAD:

| Bytes | Value                                         |
| ----- | --------------------------------------------- |
| 0–7   | ASCII `VILLOWBK`                              |
| 8–11  | format 1                                      |
| 12–15 | PBKDF2-HMAC-SHA256 iterations, exactly 600000 |
| 16–31 | fresh random 16-byte salt                     |
| 32–43 | fresh random 12-byte nonce                    |
| 44–51 | plaintext length                              |

Ciphertext follows, with a 16-byte GCM tag. The entire UTF8 JSON package is one
authenticated message; altered ordering, header, truncation and appended bytes
are rejected. Passwords are used as entered (no normalization), must have at
least 12 non-padding characters, at most 1024 UTF8 bytes and no NUL. Only the
owner retains the unlock password; it is not saved in the vault being backed up.
The renderer supplies that newly entered password and a digest, never paths,
SQL, existing secrets or a claimed backup-complete flag.

COPY data is capped at 128 MiB cumulatively before base64 expansion. A bounded
JSON writer caps plaintext at 256 MiB. Input sizes and fixed KDF parameters are
checked before allocation/decryption. This implementation uses bounded memory,
not unbounded streaming. Primary secret buffers are zeroized; it does not claim
to erase all library/OS/renderer copies from memory.

An encrypted temporary file is written and synchronized in the chosen directory,
then persisted without replacing any existing filename. The final file is
closed, reopened, decrypted and its entire plaintext hash compared with capture.
Only then is the native receipt bound to installation, source, destination,
repair operation, capture time, file path, length and ciphertext SHA-256 saved
before SQL. Interrupted writes cannot create a receipt. Resume checks those exact
bytes; imported recovery JSON strips the containing repair record. A successfully
saved but unrecorded file after a crash may remain; it cannot authorize a repair.

## Restore boundary and qualification

`backup_database::restore_empty` is an unexposed native primitive. It requires
independent caller authorization, original verified release and empty app/native
namespaces. It executes only that authenticated baseline and fixed native DDL,
disables USER triggers, copies all rows in FK order, reenables them and verifies
row bytes plus original catalog/ACL/RLS/owner/history before one commit. Internal
FK triggers remain active. There is no TRUNCATE, DELETE, schema cleanup, superuser
requirement, server filesystem COPY or `session_replication_role` bypass.
Failure rolls back schema/data/triggers. This is not an in-place rollback or a
live Supabase restore qualification. No runtime command exposes it.

The opt-in native `app_backup` test consumes the app-owned fixture interface
documented for `app_installed_repair`, using a newly created loopback cluster.
It checks the exact published manifests/archives against a synthetic test-only
channel, native capture/encryption/read-back, preservation of all eight vault
entries, exact 62-table restore under an ordinary owning role, bigint/JSON/Unicode,
changed/nonempty target refusal, unsupported catalog objects and transactional
restore failure. Crypto/file tests cover randomization, wrong passwords, changed
headers/ciphertext, truncation, no overwrite and simulated disk-full writes.
Repair tests ensure invalid/missing/changed/other-installation receipts stop both
initial work and resumed SQL/deployment. UI tests cover password matching,
secret clearing, explicit resume and failure states. These use synthetic data;
no real user vault, provider database or installer UI is used during development.
