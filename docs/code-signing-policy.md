# Code signing policy

**Development status: unsigned.** SignPath Foundation is the preferred signing route under investigation. No application, approval, certificate or sponsorship has been obtained. The current installer is a development package, and cloud setup is disabled until independent app-release trust is configured and qualification is complete.

## Proposed responsibilities

The sole current maintainer is [nicholasnucifora](https://github.com/nicholasnucifora). The proposed arrangement has this maintainer authoring changes, reviewing outside contributions and approving signing requests. SignPath must confirm that this arrangement is acceptable before enrollment. No additional reviewer or organizational identity is implied.

## Release policy

Release source must identify a reviewed immutable commit. Builds, locked dependencies, component notices and checks must be inspectable. Windows signing must cover the application executable inside the installer as well as the installer itself. Product name and version must match the approved configuration. A signed outer installer containing an unsigned application is insufficient.

Signing credentials belong in the approved signing service and protected CI settings. Untrusted pull requests must not obtain signing access. Each release will require deliberate maintainer approval, signature/timestamp verification and clean Windows qualification before publication. The current local-certificate workflow has not yet been replaced by a SignPath integration.

The intended official distribution repository is [Villow-Setup](https://github.com/nicholasnucifora/Villow-Setup). Release pages must link to this policy and the [privacy notice](privacy.md). Until qualification is complete, downloads must retain their development status.

SignPath signs Windows binaries; the hosted Villow source archives and SQL have separate Ed25519 release authentication. Approval of one mechanism does not enable the other.

## Before this becomes a public signing policy

Confirm provider acceptance, the applicable conditions, maintainer roles, actual certificate identity, protected repository access, component notices, and incident/revocation contacts. Only after acceptance add the required provider attribution and the actual expected Windows publisher. Do not present a pending application as free signing already provided.

If signing access or release integrity is compromised, stop distribution, investigate, contact the signing provider and publish verified recovery instructions. The [signing implementation notes](signing-and-distribution.md) track the remaining work and current provider sources.
