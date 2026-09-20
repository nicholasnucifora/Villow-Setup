# Windows signing and distribution

Status: the user selected **`nicholasnucifora/Villow-Setup`** for public distribution on 2026-09-11; GitHub's public API verified the repository. **Individual publishing is provisional**, with no organization established. The maintainer confirmed Australia on 2026-09-11. Legal publisher identity, issuer approval and signing credentials remain unconfirmed. `app.villow.setup.dev` and the unsigned development title make that explicit. Do not call the generated EXE a public release.

## Eligibility checked 2026-09-11

Microsoft Artifact Signing Public Trust currently supports organizations in Australia, alongside the other documented regions; individual developers must be located in the United States or Canada. The maintainer has now confirmed Australia and a provisional individual publisher. Artifact Signing Public Trust therefore does not fit the current plan. Consider an approved OSS signing program or an alternative recognized issuer after verifying its individual/Australian enrollment requirements. Private Trust certificates do not solve public publisher trust. [Microsoft eligibility](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart).

Before choosing a signer, record legal publisher name, jurisdiction, individual/organization type, domain ownership, provider eligibility evidence, account/MFA ownership and key-custody arrangements. Obtain the certificate through the issuer’s verification flow. No release should invent a publisher, claim approval or ask users to disable antivirus.

A valid signature identifies the publisher and detects tampering; it does not certify harmlessness. New signed downloads may still encounter SmartScreen reputation warnings. EV certificates no longer automatically bypass SmartScreen. Document the expected publisher and known official source rather than teaching users to click through every warning. [Microsoft SmartScreen guidance](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

## Australian individual publisher: current decision

The maintainer confirmed living in Australia on 2026-09-11. Individual publishing remains provisional. A GitHub organization is a repository-management choice; it does not supply the legal business identity used for an organization certificate. No company formation is required to continue preparing this project.

| Route | Current assessment | Next requirement |
| --- | --- | --- |
| Microsoft Artifact Signing Public Trust | Unavailable for the current Australian-individual plan; Microsoft currently limits individual enrollment to the US and Canada | Reconsider only if eligibility or the actual legal publisher changes |
| SignPath Foundation | Free signing for approved open-source projects, with the Foundation named as certificate publisher | Project approval, qualifying public source/release, signing policy, MFA and release approval |
| Paid individual certificate | SSL.com and Certum document individual products; Australian enrollment and full key-storage costs still need provider confirmation | Choose a provider/quote, then complete its identity verification directly |

Recommendation: **check SignPath eligibility first if the project will remain open source and the maintainer accepts Foundation-branded certificate publishing**. Approval is discretionary; do not promise acceptance or dates. Its requirements include an OSI-approved license without commercial dual licensing, an existing released/documented application, a verifiable source-to-binary build and manual signing approval. [SignPath conditions](https://signpath.org/terms.html), [application](https://signpath.org/apply.html).

Setup now preserves the parent's MIT notice in its source, installer license page and installed resources. The [dependency review](dependency-license-review.md) identifies the remaining component-notice work. [Privacy](privacy.md) and a [draft Code signing policy](code-signing-policy.md) are prepared and linked from the README. A public repository alone is not evidence that all licensing and release conditions have been met. No public release, application submission, external contact or signing-service subscription has been made.

A paid individual certificate displays the verified person's name. The advertised certificate price may exclude required hardware or cloud signing/storage charges. Confirm Australian enrollment and the complete quote before buying. [SSL.com individual product](https://www.ssl.com/products/software-integrity/code-signing/iv/), [Certum product types](https://www.certum.eu/en/code-signing-certificates/).

The maintainer has now selected **SignPath for preparation and eligibility checking**. See the [application preparation note](signpath-application.md) for project details and a draft enquiry about the project's pre-release status, solo maintainer roles and the conditions page's draft label. Approval remains unconfirmed. Any contact, identity or billing details requested during enrollment belong in the provider's own flow, not this conversation. Provider credentials and certificate material stay out of repository files.

The present signing workflow uses a protected Windows certificate store/hardware provider. SignPath or a cloud signer would require its own reviewed workflow integration after approval; selecting a provider does not silently enable an unimplemented signing backend.

SignPath's [GitHub integration](https://docs.signpath.io/trusted-build-systems/github) consumes uploaded build artifacts and identifies the project, signing policy and artifact configuration. The approved integration must preserve the relationship between immutable source and both the signed application and installer. Its [artifact reference](https://docs.signpath.io/artifact-configuration/reference) documents PE signing but does not list NSIS as a composite format; therefore we must not assume automatic signing of the embedded application. Confirm the build/sign/package/sign arrangement and uninstaller handling during onboarding before replacing the existing workflow. No SignPath tokens or invented configuration IDs are stored here.

## Three independent authentication layers

1. First install: recognized Windows publisher signature plus a known distribution origin. A matching checksum on a compromised website is insufficient by itself.
2. Future manager updates: a signed manual installer for v1. A future Tauri updater must independently enforce its required update signatures and pinned update metadata; no updater is installed now. [Tauri updater](https://v2.tauri.app/plugin/updater/).
3. App releases: Ed25519-authenticated channel → manifest SHA-256 → full archive/files/migrations. Signing the desktop EXE does not authenticate arbitrary downloaded SQL.

## Build and signing configuration

Development: `npm run desktop:build` creates an NSIS current-user installer. It is unsigned. WebView2 download bootstrapper mode is configured; test both present-runtime and missing-runtime environments. Administrator access is not part of the normal app install. Corporate policy or runtime installation may require exceptional approval. [Tauri Windows packaging](https://v2.tauri.app/distribute/windows-installer/).

Production: use `scripts/prepare-release.mjs` only after filling the real trust configuration and passing the release gate. It generates a Tauri config override from a legal publisher name, production application identifier, certificate thumbprint and selected timestamp URL. The signing certificate/private key must already be available through the protected Windows signing host's OS certificate store or hardware provider. The generated configuration contains identifiers, not private key material.

Build using the generated override, then run `scripts/verify-signature.ps1` against **both the main application and the NSIS installer**. Require a Valid Authenticode result, the configured certificate thumbprint and intended subject. Verify timestamping, install/reopen behavior and version information on a clean Windows machine. A public packaging gate deliberately fails while qualification.json has missing evidence.

## CI and custody

The root `.github/workflows` are now discoverable by GitHub Actions when committed and pushed. Ordinary CI receives no signing keys, provider tokens or production credentials. Signing uses an isolated, access-controlled Windows release runner and a protected `production-release` environment. Never run untrusted PR code on it. Keep releases permission separate from signing authority and marketing-site credentials. Protected workflows must check a reviewed tag and immutable source commit, run qualification first, and preserve audit logs without secrets. Branch rules, environments, runner registration, variables and secrets are repository settings and still require maintainer verification.

Back up/recover signing authority through the issuer’s supported procedure; do not export a private key into this repository or marketing hosting. Record key IDs and certificate thumbprints, ownership of MFA/recovery methods, a revocation contact and replacement procedure. If a key is compromised, stop releases, revoke through the issuer, publish authenticated revocation information, and move trust using a separately authenticated manager update. Do not silently replace release trust keys over an unsigned webpage.

The release workflow packages a signed artifact for review; publication is deliberately a separate maintainer action after signature/install checks. Configure immutable releases and protected tags in the extracted GitHub repository. The marketing website can link downloads and notes but has no signing or management authority.
