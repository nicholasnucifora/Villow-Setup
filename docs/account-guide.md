# Account preparation and screenshots

Updated 2026-09-22. The configured wizard now starts with Welcome → Choose your release → Vercel account and token → Supabase organization and token → Confirm accounts/create projects → Reserve address → Google Cloud project and OAuth. The user no longer visits all three providers before returning to each for credentials. Google starts after the exact production origin and callback are known. The native operation order, vault, checkpoints, account ownership checks and hosted app contract are unchanged. Unconfigured builds retain a read-only three-provider guide without token entry.

Setup creates its own Vercel and Supabase projects; users should not create either project manually. The native engine generates and stores the database password in Windows Credential Manager. The Supabase manual project form and its security switches are explained in an expandable section because they are not required for this path. Existing projects are not adopted. Google Cloud is different: the user creates/selects a dedicated project alongside the OAuth instructions on the Google step.

Each token is saved with the existing typed native command, leaving the other token untouched. Inputs clear after success or failure. Saving Vercel access is storage only; both accounts are read after the Supabase token is saved. The user then selects and confirms targets/costs before creation. Reopening before account selection offers reuse of saved vault tokens without exposing their values to the renderer; missing or refused access still fails closed. A successful account list does not prove later project-creation/key permissions. No computer-use testing was performed for this revision, as requested by the maintainer.

### Refused access during account discovery

The Supabase page's read-accounts action checks **Vercel identity → Vercel teams → Supabase identity → Supabase organizations** in that order. It can therefore fail on Vercel before making any Supabase request. A future token expiry does not prove that its scope, role, revocation status or copied value permits a request. The previous generic error merged HTTP 401/403 without naming the provider. The diagnostic follow-up now labels these four read-only checks using fixed native messages, including which preceding checks passed or whether Supabase was not yet reached. It retains the original error classification for other failures and never displays provider response bodies or tokens.

The maintainer's 2026-09-22 retry identified Supabase `GET /v1/profile`; the supplied token screen showed a scoped token. The maintainer then reported that a legacy replacement passed account connection. The guide now describes that working Alpha path and its full-account scope. This is evidence for account connection, not completed provider installation or proof that all scoped tokens are incompatible. No profile check is bypassed, and no credentials are changed automatically.

## Vercel token

Open [personal account tokens](https://vercel.com/account/settings/tokens) directly. Team/project Settings can show Billing, Members and Key Management without a Tokens entry. Vercel's [official instructions](https://vercel.com/kb/guide/how-do-i-use-a-vercel-api-access-token) distinguish personal account settings from team settings. Name the token **Villow Setup**, scope it to the intended hosting account/team, and choose an expiry long enough for setup and testing (for example seven days). Paste and save it on the Vercel page before moving to Supabase.

## Supabase management token for this Alpha

Open [access tokens](https://supabase.com/dashboard/account/tokens) and click the main **Generate new token** button. On the Generate token page, directly under **Resource access** on the left, follow the small underlined **Create legacy token** link. The arrow menu's **Generate token for experimental API** is a different option. Name the token **Villow Setup Alpha**, use a short expiry covering testing (for example seven days), copy it once and paste it into Setup.

[Classic/legacy tokens](https://supabase.com/docs/guides/platform/personal-access-tokens) have all the account's permissions across current and future organizations/projects. Use a dedicated test account and revoke the management token after installation testing. The earlier six-permission scoped-token recipe is withdrawn as a confirmed setup path: the public permission mapping did not establish compatibility with the required profile check. The successful legacy retry does not establish the precise reason for the scoped refusal. Replacing only Supabase access preserves the Vercel credential and saved setup.

## Database region

The [region](https://supabase.com/docs/guides/platform/regions) is where the primary database stores Villow data. Choose near most users; Sydney is the straightforward choice for Australia/New Zealand. The Alpha's native allowlist supports Sydney, Northern Virginia, Ireland and Singapore. This is a Setup limitation, not Supabase's full region list. It does not set Vercel's execution region, restrict who can use the app, or allow Setup to move an existing database.

## Google walkthrough

The form follows five numbered steps: project and Project ID; YouTube API; audience/data access; Web application client; copy credentials and save. Project ID appears next to project preparation, and client fields appear next to Google's creation dialog. Each is entered once. Unsaved values live only in the open form; the existing typed save writes the configuration and places the secret in Windows Credential Manager. The guide does not promise draft recovery after closing Setup.

Choose **External** and keep **Testing**, the single walkthrough path. In **Audience**, scroll below **OAuth user cap** to **Test users → + Add users**, enter the intended owner's Google email, save and check it appears in the table. Friends must be added before signing in. The supplied Audience screenshot is shown inline at this point. Acknowledge the seven-day Google access/refresh limit. There are no publishing instructions or collapsed Google guide sections. Previously saved internal/production configurations remain readable without being silently converted; only those saved setups see a compatibility notice and an explicit confirmation if they changed Google to Testing. See [Google's audience guide](https://support.google.com/cloud/answer/15549945) and [refresh expiration rules](https://developers.google.com/identity/protocols/oauth2#expiration).

Setup now accepts External Testing in the native engine only with the new explicit `testing_access_confirmed` acknowledgment, the API confirmation, actual Testing audience and a stored OAuth secret. It records `consent_published_confirmed: false`; Testing is never relabeled as production. The new field defaults to false on older checkpoints. Unknown audiences and unacknowledged Testing still fail. A reminder survives the remaining steps and completion. Google configuration remains a user declaration, not an API-verified Google-console status. The hosted release contract, provider writes, signature verification and fresh-database restrictions are unchanged.

Google's disabled Publish app button and message about completing Branding do not block this Testing walkthrough. The earlier production/Branding troubleshooting sections were removed at the maintainer's request; no future publishing workflow is promised.

Data Access now has a direct fixed dashboard link and a **Copy scope list** action sourced from the authenticated release's `google_scopes`. Paste into **Manually add scopes → Add to table**, then **Update**, and save on Data Access if offered. The two maintainer-supplied screenshots are bundled locally as received; they contain no visible account credentials. Captions explain the identity checkboxes, the final-page YouTube checkbox, and the priority of the release's live list over static references. The frozen 0.1.0 manifest's three scopes match these examples; its SHA-256 was rechecked against the supplied digest.

The client instructions now name **Application type → Web application**, **Name → Villow Web**, **Add URI** for the origin and redirect, and **Create**. The client authorizes the hosted site. Copy buttons report pending/success/failure next to their own address; success changes the button to **Copied!**, and a clipboard failure gives a manual Ctrl+C fallback without claiming success.

The creation dialog supplies Client ID and Client secret. Save both through Setup before closing it. No memorization or JSON download is required; a password-manager backup is optional. Creation date/status do not need to be copied. Client ID can be recovered under Clients; a lost unsaved secret requires a replacement. A test-user warning points back to Audience. See [Google's client guide](https://support.google.com/cloud/answer/15549257). The guide tells users to keep secrets out of screenshots and chats.

## Expiry and reconnecting

### Database connection recovery

The default asks Supabase for its actual PRIMARY pooler host/user via [Get pooler config](https://supabase.com/docs/reference/api/v1-get-pooler-config); the native connector always uses session mode on port 5432 with the saved database password. The API requires `database_pooling_config_read` for scoped tokens; this Alpha's working documented token path remains legacy. Explicit saved connection settings override discovery. The bundled public Supabase CA supplies certificate verification without changing Windows trust or requiring a certificate download from the user.

If the step fails, use the always-visible **Supabase → project → Connect → Session pooler → View parameters** instructions. Copy host/user, keep the password blank unless changed, save, then retry. Do not paste a full connection string or choose transaction mode. The displayed error distinguishes recognized network, TLS, login and availability causes from unknown connection and later SQL-operation failures. The previous generic message does not prove which cause the user encountered. Sources: [connection methods](https://supabase.com/docs/guides/database/connecting-to-postgres), [SSL verification](https://supabase.com/docs/guides/platform/ssl-enforcement).

### Management tokens

Vercel and Supabase management tokens stay with the desktop manager and are not copied into the hosted website's environment. Expiry/revocation prevents subsequent manager API calls; the deployed app keeps running with separate runtime credentials. Under **Recovery & settings → Reconnect expired provider access**, replace the expired token with one for the same identity/scope and leave the other field blank. Saved resource IDs, app secrets and progress remain intact. After installation checks finish, management tokens can be revoked at their providers. Google OAuth credentials, database password and app keys are separate and must not be revoked as part of that cleanup. Removing all local credentials is a different operation that also removes the local encryption-key copy.

## Screenshot slots

The reusable `src/GuideImage.tsx` registry contains ten named slots: six placeholders and four supplied screenshots (two scopes, Audience/Add users, and the client-created dialog with synthetic placeholder values). All four are shown inline. Capture clean future examples with synthetic names and hide tokens, client secrets, emails and personal account IDs. Earlier screenshots containing credentials are not bundled.

| Slot | Capture | Placement |
| --- | --- | --- |
| `vercel-account` | Dashboard account menu and stopping point before project creation | Vercel account/token page |
| `supabase-organization` | Organization selector and route back from Create a new project | Supabase organization/token page |
| `google-project` | Project selector and Project ID in Project info | Google Cloud, alongside OAuth setup |
| `vercel-token` | Personal token URL, team scope and expiry; hide value | Vercel → Screenshot guide |
| `supabase-token` | Small Create legacy token link under Resource access, then name/expiry; hide value | Supabase → Screenshot guide |
| `google-oauth` | Application type, Name, origins and redirect URI fields; hide credentials | Google step 4 |
| `google-audience` | Supplied Audience screenshot highlighting Test users → + Add users | Google step 3 |
| `google-client-created` | Supplied dialog with copy controls highlighted and placeholder values | Google step 5, before the matching fields |
| `google-identity-scopes` | Supplied identity-scope checkboxes (bundled PNG) | Google Data Access reference |
| `google-youtube-scope` | Supplied youtube.force-ssl checkbox on final page (bundled PNG) | Google Data Access reference |

Save approved screenshots under `src/assets/account-guide/`, import them into `src/GuideImage.tsx`, and replace the relevant `src: null` with the imported URL. Update each caption to describe the finished image. Bundling local images keeps the desktop CSP and offline guidance intact. Recheck 760×620 and 1120×790 layouts after replacement.

## Provider references checked 2026-09-20

- [Vercel access tokens](https://vercel.com/kb/guide/how-do-i-use-a-vercel-api-access-token): personal account token creation, account/team scope and copying the one-time displayed token.
- [Supabase Management API](https://supabase.com/docs/reference/api/introduction): personal access tokens and management authorization. These are distinct from project API keys and database passwords.
- [Google Cloud project creation](https://docs.cloud.google.com/resource-manager/docs/creating-managing-projects) and [project identifiers](https://docs.cloud.google.com/resource-manager/docs/cloud-platform-resource-hierarchy): project name, project ID and project number are different values.

Setup-specific claims about creation, password storage and later migration/OAuth steps follow the current native engine and provider adapters. This content update does not qualify provider behavior or enable production trust. Recheck live screens and release-specific database permissions during the separate disposable-provider qualification.
