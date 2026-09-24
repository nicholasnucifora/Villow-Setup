# Privacy and network access

Villow Setup is a local Windows application. This notice describes its current implementation, including the production operations that remain disabled pending release qualification. It does not describe every feature of the separately hosted Villow web app.

## What stays on this computer

Setup saves progress, your chosen owner email, account/resource identifiers and release/check results in a local checkpoint. Management tokens, database passwords, the Google client secret and generated instance secrets are stored separately in Windows Credential Manager. The interface briefly holds secrets while you enter them; it clears those fields after submission. A Windows account or computer compromise can expose locally available data.

The demo stores fictional progress in its own local browser storage. It does not contact cloud providers or collect management credentials. Use a fictional owner email when exploring it.

## Connections you request

- Checking an official app release downloads signed metadata and source files from the configured GitHub release repository and its release-asset service. These requests carry no management credentials.
- Connecting Vercel or Supabase sends the relevant management token directly to that provider's API. Setup reads your available accounts and performs the individual setup operations you request in the selected accounts.
- Preparing the database connects to the selected Supabase project's database over TLS. It uses that database's credentials to install and verify the authenticated schema plan.
- Choosing **Repair my app** reads your Villow app data, including viewing history and account connections, over the existing secure database connection. It saves that snapshot, setup metadata and original credentials in an encrypted temporary file in Setup's local data directory. A separate unlock key is kept in Windows Credential Manager. The copy is retained if repair fails and removed with its temporary key after verified success. It is not uploaded to the maintainer.
- Configuring your hosted instance sends its required Google client secret, database service key, encryption key and other settings to your own Vercel project. Public settings and server secrets have different purposes.
- Verifying the installed app sends a limited bootstrap credential and fresh challenge to that instance's setup endpoint. Setup does not download your viewing history.
- Account, Google configuration and app sign-in buttons open the selected service in your system browser. Browser cookies and account sign-in are handled there.

The service operators receive connection information such as the requesting IP address. Their handling of account and request data is governed by their own policies: [Vercel](https://vercel.com/legal/privacy-policy), [Supabase](https://supabase.com/privacy), [Google](https://policies.google.com/privacy), and [GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).

## Installer and Windows components

The installer reuses Microsoft WebView2 or downloads its bootstrapper when needed. WebView2 and Windows are separately maintained Microsoft components with their own updates, diagnostic behavior and controls; see [Microsoft's privacy statement](https://www.microsoft.com/en-us/privacy/privacystatement). This notice makes no blanket claim that those components never communicate. Missing-runtime installation and applicable disclosure/control requirements still need review before public distribution.

Villow Setup itself contains no analytics uploader, crash-report uploader, advertising component or background deployment monitor. Management credentials are not sent to a Villow marketing website or maintainer backend.

## Export, removal and questions

Recovery export saves a file only where you choose. It includes your owner email, resource IDs and version details, but no passwords or tokens. Diagnostic information is available for your review; Setup does not submit it automatically. Review exports before sharing them.

Temporary recovery copies contain sensitive app data and credentials. Setup
manages their encryption and cleanup; they do not use a developer signing key.
Keep Setup's local data and Windows account while a repair is unfinished.
Portable backups created in 0.1.4 remain yours to retain/remove, together with
their password and any synced copies. Neither kind is a support attachment to
post in a public issue. See [backup scope and recovery limits](backup-contract.md).

Removing saved credentials removes this computer's setup access. Forgetting an instance removes its local checkpoint after credential removal succeeds. Neither action revokes provider tokens or deletes the running cloud instance. Uninstalling Setup also does not cancel cloud billing; it is not a substitute for explicit credential removal. See [maintenance and removal](maintenance.md).

Use the [project repository](https://github.com/nicholasnucifora/Villow-Setup) for public questions without personal data. A private security-reporting contact and the hosted app's own privacy documentation must be established before a public release; do not post credentials in an issue.
