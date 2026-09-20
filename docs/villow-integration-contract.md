# Villow integration contract

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

| Name                          | Classification                        | Destination                                                          |
| ----------------------------- | ------------------------------------- | -------------------------------------------------------------------- |
| `VITE_YOUTUBE_CLIENT_ID`      | Public OAuth identifier               | Browser build and app server                                         |
| `YOUTUBE_CLIENT_SECRET`       | Secret                                | App server only                                                      |
| `VITE_SUPABASE_URL`           | Public project URL                    | Browser/server; server also supports `SUPABASE_URL` fallback         |
| `VITE_SUPABASE_ANON_KEY`      | Public app API key                    | Browser/server; security depends on reviewed database grants and RLS |
| `SUPABASE_SERVICE_KEY`        | Privileged API secret                 | App server only; not a database password                             |
| `VITE_APP_URL`                | Public canonical origin               | Browser/server, exact callback construction                          |
| `ENCRYPTION_KEY`              | Secret, stable per instance           | App server and local OS vault; never regenerated on resume           |
| `CRON_SECRET` | Secret, distinct from bootstrap/encryption credentials | App server; deterministically derived from the stable instance key by Setup |
| `VILLOW_EXPECTED_OWNER_EMAIL` | Server-only owner identifier          | Implemented app contract; not secret, but personal information          |
| `VILLOW_BOOTSTRAP_TOKEN_HASH` | Server-only verifier                  | Implemented app contract; raw token stays in OS vault                   |
| `VILLOW_INSTALLATION_ID`      | Server-only nonsecret installation ID | Implemented app contract                                                |

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
