# Standalone extraction and portability

**Current state:** Villow Setup now lives at the root of its own `nicholasnucifora/Villow-Setup` checkout. The runtime and build remain independent of the Villow web checkout. The source move does not satisfy public release qualification.

The standalone root must keep `package.json`, `package-lock.json`, TypeScript/Vite/Vitest configuration, `index.html`, README, AGENTS.md, `.gitignore`, `.npmrc`, `.github/`, `assets/`, `docs/`, `scripts/`, `src/`, `tests/`, and `src-tauri/` including Cargo manifests/lockfile, toolchain, Tauri config, trust file, capabilities, permissions and owned icon.

Exclude `node_modules/`, `dist/`, `.tools/`, `.cache/`, `artifacts/`, `src-tauri/target/`, `src-tauri/gen/`, local environment files, logs, installers, private keys and signing certificates. `scripts/source-files.mjs` implements this allowlist, rejects symlinks and excludes secrets/caches. No nested Git repository is required or created.

```powershell
npm run portability
```

The portability verifier copies only intended source to an unrelated temporary directory, strips provider and parent environment variables, creates fresh npm/Cargo dependency stores, installs from lockfiles and runs frontend build/tests, source/security checks, native tests and desktop compilation. It can copy an installed local compiler distribution as a developer prerequisite; it does not borrow parent or setup dependency caches. Results and the exact temporary path are recorded in ignored `artifacts/portability.json`. This is dependency isolation, not an OS permission sandbox denying every read elsewhere on the workstation.

After a transfer or checkout run `npm ci`, the verification commands in [development](development.md), and the Windows installer checks. The CI workflows assume this directory is the repository root.

Executed on 2026-09-10: all six standalone verification stages passed for 95 transferred source files in `C:\Users\Nebula PC\AppData\Local\Temp\villow-extracted-7auhnU\project`. The first run found Windows 8.3 versus long-path handling in Vite; configuration and the verifier now canonicalize filesystem paths. The successful run installed fresh dependencies and did not reuse any parent package. Packaging/install/reopen was separately checked from the original setup folder; native visual inspection was blocked by the computer-use sandbox. See [security evidence](security-evidence.md) for exact boundaries and results.

## External settings do not move with files

Configure branch protections, required CI checks, immutable GitHub Releases, Actions permissions and a protected `production-release` environment with independent approval. Restrict the Windows signing runner to reviewed tags and that environment; never use it for pull requests. Provision the signing identity through your eligible signing provider and certificate store or isolated signing service. Set the documented publisher, certificate thumbprint, production application identifier and timestamp URL. Private keys must not be committed or copied with the source.

The user selected `nicholasnucifora/Villow-Setup` as this project's eventual repository and the distribution host for app artifacts/channel. Those coordinates are recorded in `src-tauri/trust.json`. Configure the verified publisher and pinned public signing keys through reviewed code; they remain unset: repository remotes, website notes and renderer inputs must not establish executable trust. Populate `docs/qualification.json` only when its corresponding evidence exists. See [signing](signing-and-distribution.md).

The app release pipeline remains in the video-app repository. It owns the production baseline, manifests, authenticated channel and app build identity. Do not copy the parent's SQL history into this project. Only synthetic fixtures live here.

## Cross-repository integration

The web repository keeps its own `.vercelignore`, release archive allowlist and app integration. Its qualification harness must explicitly obtain a reviewed, pinned Setup checkout to preserve synthetic release/native-database verification. This cross-repository test dependency is separate from the manager runtime, which continues consuming authenticated release artifacts only. See the [standalone handoff](standalone-handoff.md) and [release checklist](release-checklist.md).

Verify `git status --short` before and after any source transfer and preserve unrelated changes. Do not infer a publisher, signing origin or release URL from Git metadata.
