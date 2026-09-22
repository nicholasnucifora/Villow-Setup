# Provider capability record

Official documentation checked 2026-09-10. Documentation establishes an available API, **not a successful real deployment**. No production resources were used to test these operations.

## Vercel

| Operation                       | API / behavior                                                     | Authorization and constraints                                                                                                 |
| ------------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| Account creation                | Official signup supports email                                     | Human email/identity checks; no GitHub required                                                                               |
| Principal and account selection | `GET /v2/user`, `GET /v2/teams`                                    | Bearer token; select the returned team ID explicitly                                                                          |
| Dedicated project               | `POST /v10/projects`; inspect `GET /v9/projects/{id}`              | Verify `accountId`, stable ID and planned name                                                                                |
| Production origin               | `GET /v9/projects/{id}/domains`, `POST /v10/projects/{id}/domains` | Confirm an exact verified `.vercel.app` domain without a branch or redirect; never use the preview deployment URL as callback |
| Environment configuration       | `POST /v10/projects/{id}/env?upsert=true`                          | Encrypt values, production target only; repeated upsert preserves the same generated encryption key                           |
| Source deployment               | `POST /v2/files`, `POST /v13/deployments`                          | Upload bytes with SHA-1 addressing required by Vercel; SHA-256 and Ed25519 separately authenticate the release                |
| Verification/reconciliation     | deployment detail and project-filtered list                        | Match operation metadata, project and release; READY alone is insufficient app health                                         |

All management requests go to `api.vercel.com`. The team ID is added explicitly. Use the least broad available token and a short expiry; token signup/login is not automatically management consent. Never select a paid upgrade silently. New projects in an already paid account can have billable consequences; the UI requires plan acknowledgement. Large account lists beyond the bounded implementation are reported unsupported instead of silently selecting a different account. [REST authorization](https://vercel.com/docs/rest-api), [accounts](https://vercel.com/docs/accounts).

Source deployment is documented without a Git source; the verified bundle contains package/lockfiles and the real app's build inputs. The server build uses `npm ci`, `npm run build` and `dist`. Existing application functions still need to fit the selected plan. The current app has 12 discovered `api/*.ts` functions; new app-side setup operations must share an existing router. [Source deployments](https://vercel.com/docs/rest-api/deployments/create-a-new-deployment), [file upload](https://vercel.com/docs/rest-api/deployments/upload-deployment-files), [limits](https://vercel.com/docs/limits).

## Supabase

The native adapter also calls `GET /v1/profile` and pins its `gotrue_id` alongside the selected organization. Replacing a token with another user's access to the same organization is refused before further writes. [Profile endpoint](https://supabase.com/docs/reference/api/v1-get-profile).

Email signup and organization/plan selection are human steps. Setup accepts a management token from the official account page. **Checked 2026-09-22:** Supabase's new scoped token form allows organization/project limits and individual permissions, defaults to No access, and coexists with full-account legacy tokens. The exact Setup mapping is in [account-guide.md](account-guide.md#supabase-scoped-management-token). Organization scope covers the newly created project; its six permissions follow the current specification. Real scoped-token installation remains unqualified. App API keys and the database password are separate credentials. [Scoped PAT permissions](https://supabase.com/docs/guides/platform/personal-access-tokens), [Management authorization](https://supabase.com/docs/reference/api/introduction).

| Operation              | API / behavior                          | Scope / fallback                                                                                               |
| ---------------------- | --------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Organization selection | `GET /v1/organizations`                 | Returned organization ID and slug; verify access on resume                                                     |
| Dedicated project      | `POST /v1/projects`                     | `projects:write` / documented `organization_projects_create`; requires database password and organization slug |
| Resource read          | `GET /v1/projects/{ref}`, list projects | Verify organization and stable reference; wait for ACTIVE_HEALTHY                                              |
| Runtime keys           | `GET /v1/projects/{ref}/api-keys`       | Key-read authority; current adapter requires returned legacy `anon` and `service_role` keys                    |
| Migrations             | Native Postgres protocol with TLS       | Generated database password, not a service-role key; direct host or session pooler                             |

The create adapter currently sends the still-documented but deprecated `region` field, with a selected supported region. Move to `region_selection` after provider contract qualification; do not guess its body shape. It does not request paid compute, a plan change, templates or addons. [Create project](https://supabase.com/docs/reference/api/v1-create-a-project), [API keys](https://supabase.com/docs/reference/api/v1-get-project-api-keys).

The integration OAuth code exchange requires client ID **and client secret**, even when PKCE is used. A confidential broker is outside the architecture, so the local token step is intentional. No setup-tool OAuth registration is required for this fallback. [Supabase integration OAuth](https://supabase.com/docs/guides/integrations/build-a-supabase-oauth-integration).

The Management API migrations endpoint is restricted to selected customers. The general installer therefore uses a TLS Postgres session. Direct connections are normally IPv6; the shared session pooler is the IPv4 alternative. Transaction-pooler port 6543 is rejected, preserving session-lock behavior. The selected host/user must match the recorded project. No CLI sidecar, Docker or dashboard SQL pasting is required by end users. [Platform endpoint restriction](https://supabase.com/docs/guides/integrations/supabase-for-platforms), [connection modes](https://supabase.com/docs/guides/database/connecting-to-postgres).

## Google Cloud and YouTube

This is the **owner's hosted-app project**, not an OAuth integration belonging to Setup. Guided steps: create/select project; enable YouTube Data API v3; configure branding/audience and required scopes; create a **Web application** OAuth client; register production origin and `${origin}/api/auth`; securely provide its secret; sign in to the deployed app. An API key cannot grant private subscriptions. No Google Cloud management token or shared client secret is embedded in Setup. [YouTube credentials](https://developers.google.com/youtube/registering_an_application).

Current release scope contract: `youtube.force-ssl`, `userinfo.email`, `userinfo.profile`. YouTube access is broad and includes writes. Optional Google Tasks is later; Gemini is not required. External Testing projects generally issue refresh tokens expiring after seven days for these scopes. Publishing the audience differs from verification; personal/internal exceptions may apply but do not imply warning-free public approval. These screen settings are marked user-confirmed until real app evidence exists. [OAuth token behavior](https://developers.google.com/identity/protocols/oauth2), [scope verification](https://developers.google.com/identity/protocols/oauth2/production-readiness/sensitive-scope-verification).

## No-GitHub-account qualification

The documented journey is email signup → management tokens → owner cloud projects → authenticated archive download → Vercel source deployment → hosted Google sign-in. Reading a public release does not require an end-user GitHub account. Maintainer release infrastructure can use GitHub independently.

**Not yet proved against real providers:** account/signup verification friction, creation ambiguity, default-domain reservation, current API-key availability, source deployment of this app, safe app baseline and intended-owner sign-in. The opt-in acceptance record must cover these before release. No unsupported provider operation is represented as green or routed through a hosted broker.
