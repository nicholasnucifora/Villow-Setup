# Villow integration contract

## Temporary protection before repair — manager 0.1.5

The published app manifests and SQL contracts remain unchanged. New installed
repairs use `backup_and_repair {digest}` with a native-owned temporary file and
separate Windows Credential Manager key; there is no renderer password or path.
`apply_installed_repair {digest}` only resumes saved intent. The renderer can no
longer start a repair by asserting a backup boolean. The native checkpoint adds
an optional `RepairIntent.backup` receipt binding an encrypted, read-back-verified
file to installation, repair operation, source and destination. Receipt/file
integrity and key decryptability are rechecked before resumed work. No existing
app key is regenerated. Receipts add `managed` (default false) and optional
`removed_at`. Only managed copies are automatically removed after durable
authenticated completion; failed cleanup is retried locally on reopen. Existing
0.1.4 portable files are retained. No signed app contract or SQL has changed.

The [backup contract](backup-contract.md) pins the exact 0.1.1→0.1.2 release pair,
data scope, file format, native-only secret export and empty-target restore
qualification. There is no restore/adoption IPC. Existing 0.1.3 manual-confirmation
intent remains resumable and is labelled unverified; new work requires the native
file receipt. Nonsecret recovery still strips all repair/backup authority.
Use 0.1.5 or newer once its new checkpoint has been written.

## Installed Alpha correction — manager 0.1.3

An optional format-1 `installed_repairs` array (maximum one) describes a separate
correction for an already-installed, unfinished Alpha. This is implemented in
source; publication and live qualification remain separate. Its single entry is:

```json
{
  "id": "villow-installed-159",
  "from_manifest_sha256": "37072f3ef4efc387b57f6edcccdfd0e71b09fc61abcaee4649f9aba76fa332ac",
  "from_schema_revision": "villow-fresh-158",
  "file": "migrations/repair-159.sql",
  "postcondition": "migrations/postcondition.sql",
  "transactional": true,
  "backup_required": true
}
```

The source digest must be the exact authenticated source manifest; the corrected
0.1.1 source digest is recorded in [the investigation](hosted-setup-failure-handoff.md).
The destination is `villow-fresh-159`, requires manager >=0.1.3 and keeps both
`upgrade_from` and `fresh_retry_from` empty. The SQL file uses role `repair`, is
hash-authenticated in the archive and passes the transactional guard. Its final
check uses role `postcondition` and must also be the destination fresh baseline's
final postcondition. Unknown fields, extra entries, missing backup/transactional
requirements, unsupported revisions/IDs/paths, orphan repair files and self-source
digests are rejected. Fresh installations still apply only `schema.migrations`.

Only the Alpha desktop identifier exposes `check_installed_repair` and
`apply_installed_repair`. Initial eligibility requires the original writable
Health checkpoint, all original effects verified, ready original deployment,
full credentials and unchanged configuration/scopes/contracts. Provider account,
project, original deployment metadata, complete SQL ledger, old schema and the
already-signed-in intended owner are rechecked. Imported recovery, completed
installs, missing secrets, another operation and drift are refused.

Before SQL, apply requires a truthful owner backup/encryption-key confirmation
and durably saves the source/destination/repair ID, backup acknowledgment time,
new operation UUID and old deployment. The acknowledgment is not independent
verification of a backup or restore. SQL uses native and owner advisory locks
plus instance/history/owner/settings table locks. App-owned additive SQL, final
schema check, a native `villow_setup.repairs` receipt and the old-to-new native
instance digest compare-and-swap commit together. The original migration rows
and checksums are retained. Resume accepts only a pristine old state without a
receipt or the corrected state with an exact matching receipt and final schema.

While repair is pending, normal advance and local credential removal/forgetting
are blocked. Existing credentials may be reconnected to the same accounts.
The top-level source pin remains old while the repair intent separately tracks
the destination. Uploads and replacement deployment use a projected destination
state and separate saved operation ID. An uncertain POST is reconciled by its
operation/release metadata, never blindly repeated. Only after new deployment
readiness, exact alias and authenticated owner/app/schema/Google/cron health pass
does Setup promote the local destination pin and Complete state. The original
deployment/effects remain recorded. No account, project or secret is recreated.

Both releases must remain unrevoked in a fresh authenticated channel throughout
resume. Recovery exports strip repair authority; importing remains read-only.
Manager 0.1.3 reads older checkpoints, but older managers do not understand a
checkpoint after this repair begins; continue with 0.1.3 or newer.

The opt-in `app_installed_repair` test takes `VILLOW_INSTALLED_REPAIR_TEST_DIR`
with `source/` and `target/` manifest/archive pairs, a channel listing both, and
`test-public-key.txt`. Its test-only key uses the actual repository URL solely
to authenticate unchanged source artifact URLs. It requires the existing
disposable-loopback environment and `manager_repair_check` database. Production
trust is never changed. The test verifies actual old baseline/owner bootstrap,
rollback at native CAS, populated-data/history preservation, committed-result
resume, destination probe and forged-receipt refusal.

## Authenticated retry before installation — manager 0.1.1

The optional format-1 manifest field `fresh_retry_from` is an array of at most eight distinct lowercase SHA-256 manifest digests. It defaults to empty and is omitted when empty. Nonempty values require `minimum_manager >= 0.1.1`, `fresh_baseline` and empty `upgrade_from`; a manifest cannot name its own digest. This authorizes replacement of a specific release before any app unit commits, not upgrades or adoption. The app-owned ACL correction is tracked in [the database investigation](database-release-investigation.md).

`check_fresh_retry` offers the channel's recommended authenticated correction, or the exact pending target after interruption. `use_fresh_retry` accepts only a digest, not URLs or SQL. Both old and new releases are authenticated against the same freshly fetched channel, with expiry, replay and revocation checks. The old pointer must remain available and unrevoked during the supported transition. Configuration classifications, Google scopes and bootstrap/health contracts must match.

Only a writable local Database-step installation with an attempted but unverified migration is eligible. Later effects, removed credentials, read-only imports, missing prerequisite/resource state and changed pending targets are refused. Live provider identity/resource checks precede any transition. Existing vault secrets are required; none are generated or changed.

Setup saves local `{from,to}` intent before the SQL operation. Under its session advisory lock and a transaction locking the installation-history tables, it requires one matching installation identity, the old or pending-new digest, zero migration rows and the original fresh-public-object predicate. It changes only the owned `villow_setup.instance.release_digest` with a conditional update, then saves the new local release identity, retaining resource IDs, owner, origin, Google settings and credentials. No baseline runs during this action. Ordinary advance is blocked while intent remains. A lost SQL response or failed final local save reconciles against the same old/new digest and all the same guards. Recovery import/export clears transition intent.

A new immutable signed app release and channel update must follow paired qualification and publisher approval. Implementing this protocol does not make that release publicly available or qualify real-provider recovery. Windows signing remains deferred.

## Current app integration

The app-side integration was implemented on 2026-09-11 with explicit user authorization. It remains a local candidate, not an authenticated public release.

- React 18 / TypeScript / Vite, package version `0.1.0`. The parent owns immutable app packaging and channel signing; Setup has independent build roots and dependencies.
- There are now 159 append-only SQL files through migration 158, including both distinct `110_` files. The app compiles a data-free fresh baseline and maps every full history identity without renaming old migrations.
- Google redirects to `${VITE_APP_URL}/api/auth`, not a Supabase Auth callback. Main sign-in requests `youtube.force-ssl`, email and profile scopes.
- Main OAuth uses browser-bound single-use state, PKCE and an HttpOnly exchange. Google credentials are absent from redirect URLs. Stored main Google tokens are encrypted; rollout readers also accept old plaintext values.
- Migration 158 transactionally pins the authenticated intended owner, rejects first-owner initialization from another account and refuses adoption of a database with users.
- Setup health shares the existing settings function. There are still 12 API entry files. Canonical origin checks replace the placeholder deployment CORS origin.
- `APP_SHELL_VERSION` is bumped locally and stamped with the authenticated app release version/commit during packaging.
- Google Tasks and Todoist remain optional. Country recommendations do not require Gemini.
- Real provider creation, Google consent and signed Windows qualification remain outstanding. See [integration status](app-contract-required.md) and [security evidence](security-evidence.md).

## Configuration classification

| Name                          | Classification                                         | Destination                                                                 |
| ----------------------------- | ------------------------------------------------------ | --------------------------------------------------------------------------- |
| `VITE_YOUTUBE_CLIENT_ID`      | Public OAuth identifier                                | Browser build and app server                                                |
| `YOUTUBE_CLIENT_SECRET`       | Secret                                                 | App server only                                                             |
| `VITE_SUPABASE_URL`           | Public project URL                                     | Browser/server; server also supports `SUPABASE_URL` fallback                |
| `VITE_SUPABASE_ANON_KEY`      | Public app API key                                     | Browser/server; security depends on reviewed database grants and RLS        |
| `SUPABASE_SERVICE_KEY`        | Privileged API secret                                  | App server only; not a database password                                    |
| `VITE_APP_URL`                | Public canonical origin                                | Browser/server, exact callback construction                                 |
| `ENCRYPTION_KEY`              | Secret, stable per instance                            | App server and local OS vault; never regenerated on resume                  |
| `CRON_SECRET`                 | Secret, distinct from bootstrap/encryption credentials | App server; deterministically derived from the stable instance key by Setup |
| `VILLOW_EXPECTED_OWNER_EMAIL` | Server-only owner identifier                           | Implemented app contract; not secret, but personal information              |
| `VILLOW_BOOTSTRAP_TOKEN_HASH` | Server-only verifier                                   | Implemented app contract; raw token stays in OS vault                       |
| `VILLOW_INSTALLATION_ID`      | Server-only nonsecret installation ID                  | Implemented app contract                                                    |

Optional quota, telemetry and integration variables are not required by this fresh-install adapter. New required variables, changed classifications or widened Google scopes require a compatible manager release; ordinary app features should not.

## Authenticated release contract, format 1

The app repository owns this contract and schema. Setup's Rust `release` module is the parser, with synthetic examples in native test fixtures. The user-selected distribution repository is `nicholasnucifora/Villow-Setup`, with channel URL `https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json`. These coordinates are configured in `src-tauri/trust.json`; the publisher name and key IDs remain unset until verified. The app source stays in `nicholasnucifora/Villow`. No Git-remote inference establishes trust.

1. A stable, signed **channel envelope** has `key_id`, base64 `payload`, and base64 Ed25519 `signature`. Sign exact payload bytes. The payload has `format`, `channel`, monotonic `sequence`, `generated_at`, `expires_at` (at most seven days), `releases` and `revoked` manifest digests. The first release is recommended. Each pointer has version, SHA-256 and immutable GitHub release-asset URL.
2. The selected plain JSON manifest is authenticated by that signed pointer's SHA-256. It carries `format`, `app_version`, 40-character immutable `commit`, `released_at`, release `sequence`, `channel`, `minimum_manager`, `upgrade_from`, `archive_url`, `archive_sha256`, `archive_size`, exact `files`, `schema`, exact `configuration`, `google_scopes`, `bootstrap_contract`, `health_contract`, fixed build/install/output settings, backup/downtime requirements and plain-text notes.
3. `files` maps every ZIP entry to SHA-256, uncompressed size and role (`deploy`, `migration`, `postcondition`). No archive directory records, links, executable permissions, case collisions, traversal, environment files, dependencies, Setup files or installers. Only `deploy` entries go to Vercel. Every executable build input, lockfile and migration must be covered. There is no runtime download-and-execute setup script.
4. Schema format 1 accepts only `fresh_baseline` with uniquely identified, ordered transactional units. Each names one SQL file, one boolean postcondition query and an exact preceding migration ID. `revision` equals the final ID. Nontransactional units and upgrade origins are refused in version one. Postconditions must check the actual schema, constraints, grants, policies, functions and required nonpersonal reference data.
5. `bootstrap_contract=1` and `health_contract=1` require the app-side behavior in app-contract-required.md. `npm ci`, `npm run build`, `dist` are currently fixed supported build settings. Preserve dependency integrity in the lockfile.

Channel authenticity/freshness and revocations are rechecked before effects. A pinned release that disappears from the supported list is blocked, not silently replaced. Local channel sequence/hash survive forgetting an instance and reject older sequence numbers or equal-sequence substitutions. Loss of all local data also loses local replay history; the compiled minimum and signed expiration still apply. Trust-key rotation and minimum-version changes require a signed manager release; they cannot come from marketing-page content.

An immutable release origin is recommended. The signed channel is mutable _data_; the selected executable source commit and archive are immutable. The channel publisher must refresh validity regularly even when no app feature ships, or new effects intentionally stop. The alternative is a separately authenticated release-status service; none is introduced here. [GitHub immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases).
