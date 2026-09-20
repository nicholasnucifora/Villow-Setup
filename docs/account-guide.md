# Account preparation and screenshots

The normal wizard starts with Welcome → Prepare your accounts (Vercel, Supabase, Google Cloud) → Connect your accounts. Preparation explains what the user must do in each provider dashboard, what Setup does later and which values they need to keep. Returning to an earlier provider preserves readiness marks for the current session. These marks do not authorize native operations. In an unconfigured build, the guide explicitly supports reading without creating accounts and stops before token entry.

Setup creates its own Vercel and Supabase projects; users should not create either project manually. The native engine generates and stores the database password in Windows Credential Manager. The Supabase manual project form and its security switches are explained in an expandable section because they are not required for this path. Existing projects are not adopted. Google Cloud is different: the user prepares a dedicated project, then later returns to it to configure OAuth after the exact Villow callback is known.

## Screenshot slots

The reusable `src/GuideImage.tsx` registry contains six named slots. Each currently shows an intentional illustrated placeholder and a caption describing the screenshot to supply. Capture clean examples with synthetic names and hide tokens, client secrets, emails and personal account IDs. The user's supplied screenshots are reference material; they are not copied into the shipped app.

| Slot | Capture | Placement |
| --- | --- | --- |
| `vercel-account` | Dashboard account menu and stopping point before project creation | Vercel preparation page |
| `supabase-organization` | Organization selector and route back from Create a new project | Supabase preparation page |
| `google-project` | Project selector and Project ID in Project info | Google Cloud preparation page |
| `vercel-token` | Token form name, scope and expiry, with the generated value hidden | Connect your accounts → Show me where to create the tokens |
| `supabase-token` | Personal access token creation, with the value hidden | Same expandable token guide |
| `google-oauth` | OAuth Web application origin and redirect URI fields; hide secret | Connect Google |

Save approved screenshots under `src/assets/account-guide/`, import them into `src/GuideImage.tsx`, and replace the relevant `src: null` with the imported URL. Update each caption to describe the finished image. Bundling local images keeps the desktop CSP and offline guidance intact. Recheck 760×620 and 1120×790 layouts after replacement.

## Provider references checked 2026-09-20

- [Vercel access tokens](https://vercel.com/kb/guide/how-do-i-use-a-vercel-api-access-token): personal account token creation, account/team scope and copying the one-time displayed token.
- [Supabase Management API](https://supabase.com/docs/reference/api/introduction): personal access tokens and management authorization. These are distinct from project API keys and database passwords.
- [Google Cloud project creation](https://docs.cloud.google.com/resource-manager/docs/creating-managing-projects) and [project identifiers](https://docs.cloud.google.com/resource-manager/docs/cloud-platform-resource-hierarchy): project name, project ID and project number are different values.

Setup-specific claims about creation, password storage and later migration/OAuth steps follow the current native engine and provider adapters. This content update does not qualify provider behavior or enable production trust. Recheck live screens and release-specific database permissions during the separate disposable-provider qualification.
