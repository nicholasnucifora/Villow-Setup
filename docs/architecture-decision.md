# Architecture decision: local Windows application

Recorded 2026-09-10. This implements the owner's selected direction; it is not a request to choose an architecture again.

Villow Setup is a Tauri 2 desktop application with bundled React/TypeScript views and a Rust operation engine. Windows is the first platform. Its source now lives in its own repository. No source, dependency, environment file, migration or Git metadata from the separate video-app checkout is needed at runtime or to build Setup.

Tauri was selected because its Windows installer, OS-native boundary and comparatively small bundled application fit this use case. The frontend has no provider networking, arbitrary filesystem access, shell bridge or secret-read command. Rust performs authenticated HTTP, signature verification, archive inspection, SQL migrations and OS-vault access. WebView2 supplies the renderer; NSIS uses current-user installation. Node, Rust, C++ build tools and Git are not end-user prerequisites. An existing WebView2 runtime is reused; its Microsoft bootstrapper may need network access and corporate device approval.

## Trust boundaries

1. **Bundled GUI → Rust:** named typed commands, a main-window capability and local-only CSP. No remote web page receives native privileges. Google and account dashboards open in the system browser. Notes are rendered as text.
2. **Rust → providers:** fixed HTTPS API hosts, redirects disabled on credential requests, certificate checks enabled, no ambient proxy inheritance. Selected Vercel principal/team and Supabase organization are checked before effects. No credential is sent to the marketing website.
3. **Rust → local storage:** nonsecret checkpoints use SQLite with FULL synchronization and a process-independent file lock. Secrets use Windows Credential Manager. Vault failures stop work. A locked/unavailable vault does not trigger a plaintext fallback.
4. **Release publisher → engine:** a signed channel envelope authenticates manifest hashes, which authenticate the complete source archive and every migration/postcondition. Archive entries remain in memory; paths, links, duplicate names and sizes are validated. Source is built on the owner's Vercel account, never executed locally.
5. **Rust → Postgres:** certificate-verified direct/session-pooler connection, port 5432, expected project hostname/user, session advisory lock, transaction per signed migration unit, checksummed history and postconditions. A service-role API key is not SQL authorization.
6. **Recovery file → engine:** an imported file is an untrusted read-only inventory. It cannot confer ownership, verified checks or write access.

## Bounded first release

Implemented paths cover fresh setup, provider account selection, dedicated project creation, permanent address reservation, guided Google configuration, verified transactional database installation, production environment configuration, source-file deployment and intended-owner health verification. The engine saves before/after effects and takes one bounded step per user action. Closing between effects pauses naturally. A pending remote write is not cancelled by pretending it never happened.

Management authorization uses a locally supplied Vercel token and Supabase management token. Supabase's documented integration exchange still needs a client secret; there is no shared desktop secret or maintainer-hosted broker. Google configuration is guided through official screens. These are explicit human operations, not dashboard scraping.

The distribution repository is configured as **nicholasnucifora/Villow-Setup**, following the maintainer's choice. As of 2026-09-21, the maintainer's genuine app-release public key and publisher label are configured for the explicitly unsigned Alpha; its public channel/manifest/archive were independently authenticated. This does not establish a Windows signing identity or completed public qualification. Development/demo operation is visibly labelled; it never becomes a production authorization shortcut. Exact release identity and limits are recorded in [alpha-0.1.0.md](alpha-0.1.0.md).

## Recovery decisions

The checkpoint contains installation/operation IDs, immutable release identity, selected account IDs, returned resource IDs, effect states, checks and timestamps. It stores neither provider bodies nor secret values. Encryption/bootstrap/database secrets are generated once before the first effect; after any effect, missing secrets halt instead of regenerating.

Provider create operations are not blindly retried. An uncertain create without a durable returned ID requires reconciliation: a name match alone is insufficient. Fake providers have durable operation IDs and prove the automated case; the live adapter conservatively stops where its provider cannot prove equivalence. This limitation must remain visible in release qualification.

Credential removal is separate from forgetting progress, uninstalling the application and deleting cloud resources. Deletion failure retains the checkpoint for retry. Removal does not revoke provider grants or erase the configuration required by the hosted application. Full cloud teardown, adoption, repair, app upgrades, custom protocols and an in-app self-updater are deferred; their contracts are in maintenance.md.

## Limits of this implementation

The GUI is trusted application code, not a defense against malware running as the same Windows user. Such malware may read the OS vault or alter checkpoints. Rust zeroizes primary secret holders, but HTTP/JSON/OS/WebView copies can remain transiently in memory. No universal memory-erasure claim is made. No analytics, crash upload, automatic background monitor, payment operation or cloud delete endpoint is included.

The app-side release/bootstrap contract and disposable real-provider qualification are still required before a public installer is suitable for nontechnical users. See app-contract-required.md and security-evidence.md.
