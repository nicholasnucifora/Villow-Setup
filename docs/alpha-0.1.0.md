# Villow Setup Alpha 0.1.0 — real fresh-account test

Prepared 2026-09-21. This Alpha uses the genuine published Villow release and the maintainer-supplied public key. Windows Authenticode remains unsigned, and `docs/qualification.json` still records unfinished public and real-provider qualification. No private release key or passphrase is required or accepted by Setup.

## Exact app and trust identity

| Field | Value |
| --- | --- |
| Distribution repository | `nicholasnucifora/Villow-Setup` (public, owner-authorized) |
| Web source repository | `nicholasnucifora/Villow` (private) |
| Frozen web commit | `1c6f949a3c0bc1b33f289b9684fa6ae84be766f4` |
| Reviewed paired Setup commit | `d01c07d012a961e008a733b9b22058e4421360b3` |
| App / minimum manager | `0.1.0` / `0.1.0` |
| Publisher label | `Fix The Web` — release ownership metadata, not a Windows certificate |
| Key ID | `villow-app-2026-01` |
| Raw Ed25519 public key (base64) | `0krJkLcRpwiVW/ylclRklRX/YFNXE5CkcC6S1CK3+XU=` |
| Signed channel / minimum sequence | `stable` / `1` |
| Initial channel expiry | `2026-09-27T05:45:08.422Z` — 27 September 2026, 3:45 pm Brisbane |
| Schema | `villow-fresh-158`, fresh-only; 159 historical migration identities, including both distinct `110_` identities |
| Formats and contracts | Manifest/channel/schema 1; bootstrap/health 1; no existing-install upgrades |

The web agent reports 1,234 app tests, 10 release-tool tests, server checking, production build and full paired native/database qualification passed for the exact frozen pair. These are handoff results, distinct from the independent public-download and native release checks on the configured Setup candidate. Runtime code and contracts were not changed when embedding this trust; later Setup source identity is in the returned build receipt.

Anonymous downloads were independently verified on 2026-09-21 at 10:09:47 UTC: all three returned HTTP 200 without a GitHub token or cookie. The Ed25519 signature verified against the exact supplied public key; the channel was unexpired and referenced the expected non-revoked manifest.

| Artifact | SHA-256 |
| --- | --- |
| [Channel](https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json), initial 637 bytes | `fbf46c028c7aa31100ff9e1a3f097bbfd2f1521d15988a250dd7bbe69fbc8e46` |
| [Manifest](https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-app-v0.1.0/manifest.json), 71,178 bytes | `5ea4a95f784c0038f7e7cc8483b8ef34d23084f086280fc26bb1a486eb69290c` |
| [App archive](https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-app-v0.1.0/villow-source.zip), 13,127,685 bytes | `945c7fbb7ad6cc01d93a6aa9ee63c9f91c3d26699969b81239e4464c156a00bc` |

The channel is intentionally renewable; its digest, sequence and expiry change on an authenticated renewal. Automatic renewal is not configured. The web-release maintainer must renew before expiry using the existing key and original immutable app artifacts. Expiry is never bypassed and does not require a replacement trust root. Channel expiry can block subsequent setup checks/resume, but it does not shut down an already hosted app.

## Which EXE to use

Use the newly built installer **`Villow Setup Alpha_0.1.0_x64-setup.exe`**, then open **Villow Setup Alpha**. Its expected build location is `src-tauri/target/release/bundle/nsis/`; the returned build receipt records the actual full path, installer SHA-256, final Setup commit and verified Authenticode status.

Do not use `Villow Setup_0.1.0_x64-setup.exe`, `Villow Setup Testing_0.1.0_x64-setup.exe`, an older installed preview or the shared build-directory `villow-setup.exe` as the test target. Alpha has its own name and local checkpoint directory, `app.villow.setup.alpha`. Testing controls are compiled out. It installs for the current Windows user; running as administrator is not part of this walkthrough.

## First walkthrough

1. Install and launch the updated Alpha. Confirm the unsigned-alpha notice is visible. Choose **Begin setup**, then **Check the official release**. It must show **Villow 0.1.0 — Authenticated release**. Choose an instance name and Google owner email, read the dedicated-resource/cost confirmation and select **Start my setup**. This revision moves account visits after release verification; an existing checkpoint resumes at its saved stage.
2. At **Connect Vercel**, sign up/sign in and stop before creating a project. Use **Open Vercel personal tokens** (the personal account page, not team Settings). Name the token Villow Setup, select the intended hosting-team scope and an expiry long enough to test, such as seven days. Paste it and choose **Save Vercel token & continue**. This stores the token; account access is checked after the Supabase step.
3. At **Connect Supabase**, sign in to your dedicated test account and choose an organization, stopping before manual project creation. Open access tokens → main **Generate new token** button → small **Create legacy token** link under **Resource access**. Name it Villow Setup Alpha, choose a short expiry and paste the complete token. This grants full account access; revoke after testing. The maintainer reported a legacy token passed the identity check that refused their scoped token. Select **Save Supabase token & read accounts**, then review accounts, database region and costs before confirming. See [the detailed walkthrough](account-guide.md).
4. Choose the intended Vercel account/team, Supabase organization and region. Review the account/cost confirmation. Continue with the separate create-project buttons. Use fresh dedicated resources; this Alpha cannot adopt your existing database. The database password is generated and saved by Setup.
5. After Setup reserves your website address, follow **Google Cloud**: create/select one dedicated project, enable YouTube Data API v3, choose **External** and keep **Testing** for this prototype. Add your Google owner email under **Audience → Test users** and acknowledge the seven-day Google access limit. In Data Access, use **Copy scope list** and the two supplied screenshot references. Create a **Web application OAuth client** and use the exact origin and `/api/auth` callback shown. Enter Project ID, client ID and secret, confirm the configuration and save. Testing is recorded honestly and can continue to database/deployment. You can publish later when real homepage/policy pages are available; Windows signing is separate.
6. Continue through database preparation, configuration and deployment. At **Make it yours**, open the hosted app, sign in with the intended Google owner and return to check the installation.

Enter account secrets only in the application/provider UI, never in a handoff or chat. If a step fails, keep its nonsecret error and step name. Reopen the same Alpha to resume saved progress instead of creating a duplicate setup. Do not apply the fresh baseline manually to another database.

If a management token expires, the hosted app keeps running. To continue unfinished setup, open **Recovery & settings → Reconnect expired provider access**, replace that provider's token with the same identity/scope and leave the other field blank. Before account selection, the provider pages also offer reuse of saved tokens. Do not recreate resources.

For the first test, close/reopen between completed steps. Deliberate interruption during writes should be a separate supervised disposable-resource test: an uncertain creation must reconcile rather than repeat blindly. Real provider setup, owner consent and cloud interruption recovery are not certified by a successful local build. Closing/uninstalling Alpha does not delete cloud resources or stop their charges.
