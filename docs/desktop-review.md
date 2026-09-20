# Desktop and sandbox review — 2026-09-12–14

The earlier Windows helper startup failure did not recur. Local commands, workspace edits, frontend builds, native compilation, the in-app localhost browser and native Windows UI inspection worked during this review. This made rendered interaction testing possible after the earlier build-only checks.

This was development testing on the maintainer's existing Windows computer. Codex's execution sandbox is not a clean disposable Windows installation and does not establish SmartScreen reputation, signed distribution or behavior without WebView2.

## Defects corrected

- Operation errors could appear above the visible viewport. Errors and status notices now receive focus and scroll into view with space above them.
- Moving to another wizard step could retain the previous step's scroll position and focus. Navigation now focuses the new heading and returns to the top. Non-interactive headings do not acquire a large default focus outline; interactive controls retain their keyboard focus indicators.
- Opening recovery controls now reveals and focuses the panel instead of leaving it below the fold.
- The demo's one-operation failure selector now resets after the selected interruption is consumed, matching its actual behavior. Its label is now “Next operation”.
- Demo database setup no longer exposes password or database-connection inputs. It explicitly uses fictional data.
- Upcoming sidebar steps have readable text instead of reducing the entire row's opacity.
- Switching between demo and real mode clears the local-removal checkbox, typed confirmation and failure-selection UI state.
- Reopening Google configuration now restores the saved audience and both acknowledgments. Continue stays disabled for unsaved edits, including a failed replacement-secret save. Saved and pending configuration have explicit status, and Save/Continue have space between them.
- Reconnecting reports that account access was loaded without claiming ownership checks have already passed. Failed reloads clear that feedback.
- Successful local credential removal now brings a completion notice into view and resets its acknowledgment. Reconnecting does not silently authorize another removal.

## Rendered interaction checks

The browser review covered the complete simulated journey: start, account selection, both project creations, permanent address, Google configuration, database, configuration, deployment and successful health completion. External Google Testing configuration stayed blocked; a failed health check could not produce completion. The exact callback and its copy control were checked. The simulated recovery-export notice, reload/resume and keyboard focus were inspected.

Offline, expired-access and rate-limit failures were retryable. A lost creation response required recovery and did not create a second simulated resource. The failure selector returned to None after consumption. The previously offscreen error was verified in view after the fix. Browser widths of 760, 950, 1120 and 1600 pixels had no horizontal document overflow; the temporary viewport override was reset. The inspected browser console contained no warnings or errors.

The extended native checks below supersede the initial native launch/inspection pass. No real Google sign-in or management credentials were entered. Cloud operations used the isolated demo; clipboard, recovery dialogs, import/export and recovered-checkpoint persistence used the actual Windows application boundary.

### Extended native pass — September 13–14

A temporary Tauri configuration used a separate application identifier and Windows data directory for the review. It changed the identifier and window title only; release trust, permissions and runtime behavior stayed intact. The review used fictional accounts and an untrusted synthetic recovery fixture. Generated review configuration and fixtures stay under ignored `artifacts/native-review/`.

- Walked the native demo from required owner-email validation and account/cost acknowledgments through project creation, permanent address, Google configuration, database, service configuration, deployment and successful health completion.
- Exercised offline, expired-token, rate-limit and lost-response failures. Normal reopen retained account selection. After an abrupt stop during the simulated lost database response, reopening reconciled the same fake resource and continued. This does not qualify live-provider reconciliation.
- Verified actual Windows clipboard output for both Google copy buttons. Testing audience blocked Save even with both acknowledgments checked; internal audience saved. Reopening the fixed build restored it, and unsaved changes disabled Continue.
- Failed intended-owner verification prevented completion; retry completed. App-opening in the demo produced the simulated notice. Expanded evidence entries remained explicitly marked as simulated.
- Inspected native minimum client size **760 × 620**, default **1120 × 790**, and maximized layout. Recovery text wrapped, diagnostics stayed in their own scroll region, and keyboard focus/scrolling exposed below-fold controls. Empty and wrong instance names kept Forget disabled; the exact name enabled it.
- Removing fictional demo access retained progress through reopening. Reconnection restored access. The final removal notice appeared in view, its acknowledgment reset, and the exact-name demo reset returned to Welcome.
- Canceled the actual Windows import dialog with Escape without creating an installation. Malformed JSON and a **100,001-byte** file were rejected with a visible error and no checkpoint. A valid synthetic file claiming completion and verified effects imported only as a read-only inventory: `step` became `health`, `effects` and `checks` were empty, and cloud setup/reconnection controls were unavailable.
- Canceled the actual save dialog and verified “Export cancelled.” Then saved a new recovery file, checked its fixture identity, read-only/credentials-removed flags, empty effects/checks and absence of credential fields. The imported native checkpoint reopened as the same read-only inventory without another import.
- Closed the review application after completion. Its synthetic profile is separate from the normal development identifier; the normal installer was rebuilt without the review override.
- Native page-zoom hotkeys are disabled by the current Tauri configuration. This pass does not claim page-zoom, OS high-DPI, screen-reader or clean-Windows qualification.

## Automated and package checks

- Fifteen UI tests passed, including four interruption/retry cases, focus behavior, full demo completion, saved Google restoration/unsaved-edit guards, failed secret-save recovery, reconnect feedback and local removal/forgetting.
- All 34 ordinary native tests passed on the approved host rerun. The OS credential-vault test failed inside the restricted sandbox with `Error::Vault`; it passed with normal Windows-account access. Vault access remains a separate permission boundary. The opt-in PostgreSQL and extracted-app fixture tests were not repeated for these UI/documentation edits; their earlier evidence remains dated in [security evidence](security-evidence.md).
- Frontend TypeScript/Vite checks, native release compilation, NSIS packaging and the source/bundle capability scan passed.
- The unsigned package was installed into a unique workspace test directory, its license/privacy/signing-policy resources checked, and its application kept running through two launches. Its own uninstaller removed the test copy. See `artifacts/integration-package-smoke.json` for the exact package hash and checks. That script records `visual_verification: false` because it does not inspect rendered pixels; native inspection is recorded separately above.
- The package harness initially checked the uninstall registration as soon as the folder disappeared. NSIS removed that entry afterward, producing a timing failure in the harness. It now waits up to the existing 20-second deadline for both cleanup steps. A fresh isolated package run passed with neither the directory nor registration remaining; no manual registry deletion was used.
- The public release gate continued to refuse unconfigured publisher trust and missing real-provider/signed-Windows qualification. No gate was waived.

Final reviewed installer SHA256: `bd449853dc08c7f2aa0b42eab96f597f13119eebac778e43e192c03fba60e4eb`; Authenticode: **NotSigned**. The final package check also verified removal of its test installation directory and current-user uninstall registration.

Latest raw local logs are `artifacts/native-review/ui-tests.log`, `native-tests.log`, `desktop-build.log` and `release-gate.log`. That folder also contains the synthetic recovery fixtures, actual `exported-recovery.json` and running observation notes. Earlier `artifacts/sandbox-*.log` files record the first pass. Generated files are excluded from source extraction and commits. Native/browser observations are recorded here from the inspection session, not represented as an automated screenshot assertion suite.

## SignPath preparation and limits

Setup now preserves the inherited MIT license in source and installer resources, includes a [privacy notice](privacy.md) and explicitly provisional [code signing policy](code-signing-policy.md), and has a repeatable [dependency inventory](dependency-license-review.md). Its 376 package declarations do not establish a completed license audit: 12 package notice locations and final shipped-component obligations still need review.

The [application preparation](signpath-application.md) now follows the actual embedded SignPath form, including required reputation evidence and agreement/personal-data consent, with optional marketing consent identified separately. No form data was entered, terms accepted, message sent or account connected. Approval, suitable public source/release evidence, complete notices and the actual SignPath build integration remain outstanding.

Clean Windows profiles, missing-WebView2 installation, OS high-DPI settings, signed publisher/timestamp verification, real cloud deployment and real intended-owner OAuth remain unqualified. Keep production trust empty until those release checks are complete. The later repository separation does not upgrade this historical evidence.
