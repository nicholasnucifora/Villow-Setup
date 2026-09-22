# Villow Setup

A local Windows application for creating an owner-operated Villow instance in Vercel, Supabase and Google Cloud. The manager is separate from the hosted video app: closing or uninstalling it does not stop that app or cancel cloud billing.

**Status: unsigned Alpha for fresh-account testing, not a qualified public installer.** Genuine app-release public trust is configured for Villow 0.1.2, with 0.1.0 and 0.1.1 retained for authenticated unfinished-installation correction. The maintainer reports the corrected database preparation passed on their fresh Supabase project. Full provider installation, owner sign-in and recovery still need qualification; Windows Authenticode signing remains deferred.

## Try the development application

On Windows with Node 22, Rust 1.90, Visual Studio C++ Build Tools and WebView2:

```powershell
npm ci
npm run desktop:dev
```

Normal development and installer builds exclude testing controls. After verifying the release, work through Vercel's account and token together, then Supabase's organization and token. Setup creates dedicated projects and reserves the website address before guiding Google Cloud's project and OAuth setup in one place. The current build explains its release prerequisite before asking users to create accounts or tokens. [Account and token instructions](docs/account-guide.md) explain this Alpha’s legacy Supabase token path, database regions, Google client creation and how to replace expired access without stopping the hosted app.

To opt in to the isolated demo and interruption scenarios, run `npm run desktop:dev:testing` or build `npm run desktop:build:testing`. Enable **Show testing tools**, then choose **Explore demo**. The testing application has a separate name and data identifier. The demo stores only fictional state in its own browser storage, makes no provider requests and never accepts management tokens. Ordinary builds exclude its engine and controls entirely.

```powershell
npm run desktop:build
```

The development installer is generated at `src-tauri/target/release/bundle/nsis/Villow Setup_0.1.5_x64-setup.exe`. Its development identifier is `app.villow.setup.dev`. It is unsigned and must not be distributed as a trusted public release. See [development](docs/development.md) for prerequisites and [signing and distribution](docs/signing-and-distribution.md) for the protected release path.

For the maintainer's real-account prototype, `npm run desktop:build:alpha` selects a separate **Villow Setup Alpha** installer with testing tools excluded. The maintainer-supplied public trust is configured and the published downloads have been verified without GitHub authentication. Windows signing and public qualification remain separate. See [this Alpha's release and test instructions](docs/alpha-0.1.0.md) and the [unsigned-alpha procedure](docs/unsigned-alpha.md).

Manager 0.1.5 creates and verifies a temporary encrypted recovery copy automatically before the bounded repair of the known unfinished installed Alpha. Choose **Repair my app**; no backup password or file selection is needed. Setup preserves the original app key, applies the authenticated repair and rebuilds the same Vercel project. The copy stays if anything fails and is removed after the repaired app passes authenticated checks. Later-stage recovery remains assisted and cannot overwrite an existing database. The genuine 0.1.2 repair is published. See [backup and recovery](docs/backup-contract.md) and [the installed repair contract](docs/villow-integration-contract.md). General upgrades and adoption remain deferred.

## Implemented

- Native Rust orchestration, React interface and per-user Tauri/NSIS packaging.
- Guided provider tokens stored in Windows Credential Manager; selected provider identities, organizations and resource IDs checked before operations.
- Dedicated project creation, permanent production domain, Google Cloud instructions and exact OAuth callback, signed SQL migration plan, production environment configuration, source upload and deployment adapters.
- Durable SQLite checkpoints, process lock, conservative reconciliation, stable encryption secrets, reconnect after token expiry and resumable content uploads.
- Cryptographic release verification, constrained archives, authenticated health contract, nonsecret recovery export, read-only recovery import and local credential removal.
- Separate deterministic demo and security, provider, recovery, SQL, OS-vault and UI tests.

## Boundaries and remaining gates

Actual cloud creation, no-GitHub deployment behavior, real Google sign-in and real-provider interrupted resume have **not** been qualified. The app-side release, schema, owner, authentication and health contracts are implemented and locally tested. [App integration status](docs/app-contract-required.md) records what changed. Setup develops independently from this repository root. The web maintainer reports the frozen app/Setup pair passed its full qualification; exact commits and the separately verified genuine published artifacts are recorded in [the Alpha release record](docs/alpha-0.1.0.md).

General automated upgrades, writable adoption after loss of local state, general repair, cloud teardown, custom protocol links and automatic manager updates are deferred. Imported resource IDs are hints, never permission to mutate resources. Unknown outcomes stop for review; setup does not guess that repeating a creation is safe.

## Evidence and handoff

- [Security controls, actual results and limitations](docs/security-evidence.md)
- [Desktop and sandbox review](docs/desktop-review.md)
- [Architecture decision](docs/architecture-decision.md)
- [Provider capabilities and primary sources](docs/provider-capabilities.md)
- [Versioned Villow integration contract](docs/villow-integration-contract.md)
- [Standalone repository handoff](docs/standalone-handoff.md)
- [Maintainer release checklist](docs/release-checklist.md)
- [Standalone extraction and portability](docs/extract-to-repository.md)
- [Maintenance and removal](docs/maintenance.md)
- [Code signing policy — development status](docs/code-signing-policy.md)
- [Privacy and network access](docs/privacy.md)
- [SignPath application preparation](docs/signpath-application.md)

## License

The project uses the [MIT license](LICENSE), preserving the parent app's existing notice. Dependency licenses remain separate; [component review](docs/dependency-license-review.md) records the current inventory and outstanding notices before public distribution.

All source, dependencies, tests, docs and build configuration belong to this folder. There is no runtime dependency on the parent checkout, its environment, credentials, Git history or migration directory. Do not commit generated installers, caches, credentials or signing material.
