# Account preparation and screenshots

Updated 2026-09-22. The configured wizard now starts with Welcome → Choose your release → Vercel account and token → Supabase organization and token → Confirm accounts/create projects → Reserve address → Google Cloud project and OAuth. The user no longer visits all three providers before returning to each for credentials. Google starts after the exact production origin and callback are known. The native operation order, vault, checkpoints, account ownership checks and hosted app contract are unchanged. Unconfigured builds retain a read-only three-provider guide without token entry.

Setup creates its own Vercel and Supabase projects; users should not create either project manually. The native engine generates and stores the database password in Windows Credential Manager. The Supabase manual project form and its security switches are explained in an expandable section because they are not required for this path. Existing projects are not adopted. Google Cloud is different: the user creates/selects a dedicated project alongside the OAuth instructions on the Google step.

Each token is saved with the existing typed native command, leaving the other token untouched. Inputs clear after success or failure. Saving Vercel access is storage only; both accounts are read after the Supabase token is saved. The user then selects and confirms targets/costs before creation. Reopening before account selection offers reuse of saved vault tokens without exposing their values to the renderer; missing or refused access still fails closed. A successful account list does not prove later project-creation/key permissions. No computer-use testing was performed for this revision, as requested by the maintainer.

## Vercel token

Open [personal account tokens](https://vercel.com/account/settings/tokens) directly. Team/project Settings can show Billing, Members and Key Management without a Tokens entry. Vercel's [official instructions](https://vercel.com/kb/guide/how-do-i-use-a-vercel-api-access-token) distinguish personal account settings from team settings. Name the token **Villow Setup**, scope it to the intended hosting account/team, and choose an expiry long enough for setup and testing (for example seven days). Paste and save it on the Vercel page before moving to Supabase.

## Supabase scoped management token

Open [access tokens](https://supabase.com/dashboard/account/tokens), name it **Villow Setup**, and choose a suitable expiry (for example seven days). Under **Resource access**, choose **Organization** (all projects in selected organizations) and select only the dedicated Villow organization. The default existing-project selection cannot cover the project Setup will create. This scope includes the other projects in that organization too; use a dedicated organization for this test.

Starting with **No access**, set these entries and leave all others at **None**:

| Group | Permission | Access | Setup use |
| --- | --- | --- | --- |
| Project | Project Settings | Read | Verify the created project's identity and health |
| Application services | API Keys | Read | Retrieve the app's API keys |
| Application services | API Key Secrets | Read | Read the server key value for the hosted app |
| Account and organization | Organizations | Read | List and recheck the organization |
| Account and organization | Projects (account-wide) | Read | List projects during reconciliation |
| Account and organization | Organization Projects | Read-write | Create the dedicated project |

Database and Infrastructure and delivery stay at None. Setup applies SQL through a PostgreSQL connection using its generated database password, not the Management API SQL/migrations endpoints. Tokens cannot exceed the user's own role; the account must be able to create projects in the selected organization.

This mapping follows Supabase's [permission table](https://supabase.com/docs/guides/platform/personal-access-tokens), [API specification](https://api.supabase.com/api/v1-json) and [resource selector source](https://github.com/supabase/supabase/blob/master/apps/studio/components/interfaces/Account/AccessTokens/Scoped/Form/ResourceAccessStep.tsx), checked 2026-09-22 against `src-tauri/src/providers.rs`:

- `GET /v1/organizations`: `organizations_read`.
- `GET /v1/projects`: `projects_read`.
- `POST /v1/projects`: `organization_projects_create`.
- `GET /v1/projects/{ref}`: `project_admin_read`.
- `GET /v1/projects/{ref}/api-keys`: `api_gateway_keys_read`, plus `api_gateway_keys_secret_read` for secret values.
- `GET /v1/profile`: authenticated identity read, no additional named FGA permission in the current public specification. Setup pins `gotrue_id`; scoped tokens do not bypass this check.

Scoped tokens are a Supabase alpha rollout; this is documented guidance, not completed live qualification. If access is refused, check expiry, resource selection and role/permissions before retrying; do not bypass identity checks or silently broaden grants. If only classic/legacy tokens are offered, they grant all the account's access, including other organizations. Use an account dedicated to testing, a short expiry and revoke after completion. The scoped instructions do not require a legacy token.

## Expiry and reconnecting

Vercel and Supabase management tokens stay with the desktop manager and are not copied into the hosted website's environment. Expiry/revocation prevents subsequent manager API calls; the deployed app keeps running with separate runtime credentials. Under **Recovery & settings → Reconnect expired provider access**, replace the expired token with one for the same identity/scope and leave the other field blank. Saved resource IDs, app secrets and progress remain intact. After installation checks finish, management tokens can be revoked at their providers. Google OAuth credentials, database password and app keys are separate and must not be revoked as part of that cleanup. Removing all local credentials is a different operation that also removes the local encryption-key copy.

## Screenshot slots

The reusable `src/GuideImage.tsx` registry contains six named slots. Each currently shows an intentional illustrated placeholder and a caption describing the screenshot to supply. Capture clean examples with synthetic names and hide tokens, client secrets, emails and personal account IDs. The user's supplied screenshots are reference material; they are not copied into the shipped app.

| Slot | Capture | Placement |
| --- | --- | --- |
| `vercel-account` | Dashboard account menu and stopping point before project creation | Vercel account/token page |
| `supabase-organization` | Organization selector and route back from Create a new project | Supabase organization/token page |
| `google-project` | Project selector and Project ID in Project info | Google Cloud, alongside OAuth setup |
| `vercel-token` | Personal token URL, team scope and expiry; hide value | Vercel → Screenshot guide |
| `supabase-token` | Organization resource scope and six required permissions; hide value | Supabase → Screenshot guide |
| `google-oauth` | OAuth Web application origin and redirect URI fields; hide secret | Connect Google |

Save approved screenshots under `src/assets/account-guide/`, import them into `src/GuideImage.tsx`, and replace the relevant `src: null` with the imported URL. Update each caption to describe the finished image. Bundling local images keeps the desktop CSP and offline guidance intact. Recheck 760×620 and 1120×790 layouts after replacement.

## Provider references checked 2026-09-20

- [Vercel access tokens](https://vercel.com/kb/guide/how-do-i-use-a-vercel-api-access-token): personal account token creation, account/team scope and copying the one-time displayed token.
- [Supabase Management API](https://supabase.com/docs/reference/api/introduction): personal access tokens and management authorization. These are distinct from project API keys and database passwords.
- [Google Cloud project creation](https://docs.cloud.google.com/resource-manager/docs/creating-managing-projects) and [project identifiers](https://docs.cloud.google.com/resource-manager/docs/cloud-platform-resource-hierarchy): project name, project ID and project number are different values.

Setup-specific claims about creation, password storage and later migration/OAuth steps follow the current native engine and provider adapters. This content update does not qualify provider behavior or enable production trust. Recheck live screens and release-specific database permissions during the separate disposable-provider qualification.
