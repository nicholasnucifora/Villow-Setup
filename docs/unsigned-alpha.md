# Unsigned Windows alpha

The maintainer has explicitly deferred Windows Authenticode/SignPath while testing a real fresh installation. `npm run desktop:build:alpha` prepares that separate candidate once genuine app-release public trust is configured. It does not call the signed-public-release gate, change `docs/qualification.json`, replace release authentication, or establish public-release qualification.

**2026-09-21:** Genuine public trust has been supplied and the published channel/manifest/archive verified anonymously. See [the 0.1.0 Alpha record](alpha-0.1.0.md) for the exact release and instructions. The field requirements below remain the maintainer contract for subsequent releases.

## What the build does

- Uses `Villow Setup Alpha`, identifier `app.villow.setup.alpha`, a window titled “Villow Setup — unsigned alpha”, and a persistent unsigned-alpha notice. Its local checkpoint directory is separate from development, testing and eventual production. Credentials remain in Windows Credential Manager under each installation's UUID; this does not change existing vault entries.
- Compiles testing tools and the demo engine out, even if testing was enabled in the caller's environment. Refuses testing flags, custom Tauri configuration and extra build arguments. Windows packaging explicitly has no certificate or signing command and installs for the current user.
- Validates the reviewed public trust configuration before invoking Tauri. No cloud tokens, certificate, private release key or completed Windows qualification is required to build this candidate. Shape/key-ID checks cannot prove who owns a key; maintainer review of the genuine public material remains required.
- Uses the existing Rust channel signature, expiry, sequence, revocation, manifest/archive checksum and migration verification. A missing, expired or invalid signed release still prevents installation. No special alpha trust bypass exists.

The expected installer after a successful configured build is:

```text
src-tauri/target/release/bundle/nsis/Villow Setup Alpha_0.1.2_x64-setup.exe
```

That path is an expected output, not evidence an alpha has been built. The ordinary `Villow Setup_0.1.0_x64-setup.exe` and optional `Villow Setup Testing_0.1.0_x64-setup.exe` are different files. Rebuilding source does not change an already installed EXE. Do not use the shared build-directory `villow-setup.exe` to distinguish profiles; install the exact recorded installer and verify its hash. Keep alpha as a separate test installation; recovery import cannot migrate it into production or adopt its database.

## Public inputs from the web-release maintainer

Return the following through the reviewed handoff. Never send the private key, provider tokens or account passwords.

| Trust field | Required value |
| --- | --- |
| `format` | `1` |
| `repository` | `nicholasnucifora/Villow-Setup` |
| `channel` | `stable` (the existing signed protocol name, not a statement of qualification) |
| `manifest_url` | `https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json` |
| `publisher` | Maintainer-approved nonempty publisher label. For alpha this is release ownership/installer metadata, not proof of Windows certificate verification. |
| `public_keys` | Mapping from the exact channel envelope `key_id` to canonical base64 of **32 raw Ed25519 public-key bytes**. Not PEM, a certificate or a private key. Use the genuine durable release key, never a test fixture key. |
| `minimum_sequence` | `1` for the first channel initialized at sequence 1; agree a reviewed minimum if the published channel is already further along. |

Also return the exact reviewed web commit, app version, paired Setup commit, manifest SHA-256, archive SHA-256, immutable manifest/archive URLs, signed channel sequence and expiry, supported manager/contract versions, and paired test results. Confirm the public key matches the signing environment's separately pinned public key. Setup does not generate a competing key pair.

## Sequence to the first real test

1. Commit the reviewed Setup candidate and give its full SHA to the web agent. The web agent pins that SHA, runs the required paired harness and freezes its own reviewed web commit. Use `npm run release:check -- --setup-root <clean Setup checkout> --setup-commit <full SHA>` in the web repository. This is an opt-in maintainer test dependency only.
2. The web-release maintainer retains the durable private key and approves the prepared authentic release for publication. The web agent owns packaging, signing, immutable artifacts and channel renewal; the six-day validity must be renewed before expiry without replacing immutable release artifacts.
3. After the genuine public material and published artifacts exist, review and commit matching `src-tauri/trust.json` here. Record this later Setup SHA separately. If Setup code/contracts change, repeat paired qualification against the new candidate; do not silently reuse evidence for changed code.
4. Run the Setup checks, build with `npm run desktop:build:alpha`, inspect both package and application Authenticode status, and record installer SHA-256, source SHA and unsigned profile. Verify the actual installed alpha on Windows. Do not fill signed-public-release flags from alpha evidence.
5. Before asking for tokens, run the installed alpha's release check. Then guide the owner through dedicated Vercel/Supabase resources and the prepared Google project, account/cost confirmation, exact callback, intended-owner consent and hosted-app checks. Account sign-in, consent and key custody remain owner actions. The first real account journey may uncover provider issues that local tests cannot establish.
6. Exercise close/reopen, uncertain create results, expired-token reconnect and local credential removal with the authorized disposable resources. Record actual results; cloud deletion and existing-production tests are outside this workflow.

Windows certificate enrollment can follow later. Ordinary app commits do not update deployed instances: v1 supports fresh installs only. Follow [maintenance](maintenance.md) for the separate upgrade requirements.

## App-release test evidence and limits

`src-tauri/tests/app_release.rs` is opt-in and requires both the web-owned synthetic signed fixture and `VILLOW_LOCAL_DB_TESTS=disposable-local-cluster`. It cannot pass by skipping SQL. The web harness creates/cleans its own loopback cluster, prepares Supabase-compatible test roles and supplies `VILLOW_LOCAL_DB_PORT`/`VILLOW_LOCAL_DB_PASSWORD`; the native test only connects to `127.0.0.1`, database `manager_check`.

The test authenticates the signed channel and full archive/manifest; installs the actual app baseline; reconnects and resumes without reapplying it; checks installation/release identity and every ledger ID, SQL checksum and postcondition checksum; rejects altered ledger checksums and actual schema drift. Separate native security tests reject forged/tampered artifacts. `postgres_protocol` tests exercise real local SQL/postcondition rollback, locks, data preservation, permissions and existing-database refusal with synthetic SQL.

Reconnecting after a completed commit does **not** prove every crash scenario. Remaining qualification includes process/connection termination during the full app baseline, a lost COMMIT response with subsequent reconciliation, and real Supabase/network interruption. Fake-provider lost-response tests and synthetic SQL rollback do not substitute for those observations. No genuine published release, remote database or Google account is exercised by these local tests.
