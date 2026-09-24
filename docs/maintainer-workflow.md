# Villow maintainer workflow

Updated 24 September 2026. You can develop and push Villow normally. Setup users
install published release archives; ordinary web commits do not change their apps.

## When you want to release your web changes

Tell the web-app task:

> Prepare these changes as the next Setup-compatible Villow release. Compare
> against the actual published release, identify database and configuration
> changes, prepare fresh-install and exact-source update plans, and run the
> automated app and paired Setup tests on disposable databases. Coordinate any
> required Setup changes. Give me a short result and the exact local signing
> command for Developer Command Prompt (CMD). Do not publish yet.

The agent owns versioning, migration packaging, compatibility review, tests and
release artifacts. You review what changed, enter your existing signing
passphrase into the prepared local masked prompt, and approve publication.
Never paste the passphrase into chat. You do not manually calculate hashes or
repeat account creation for every release. Targeted real-provider testing is
still needed when authentication, cloud configuration or deployment changes.

Signing helpers may internally use PowerShell, but the supplied command must
work from CMD: double-quoted executable/script paths, no leading `&` operator.
Use the exact helper prepared for that release, never a previous release's file.
The publisher must coordinate with renewal before changing the signed channel:
disable renewal, drain active runs, preserve current releases/revocations and
sequence, then restore the prior enabled state after public verification.

## What users do

Setup Alpha 0.2.0 adds **Check for updates → Update my app** for completed local
installations. Supported releases include an explicit transition from their
installed version. Setup handles temporary encrypted protection, database work,
deployment and checks. Interrupted work offers **Resume update**. It preserves
the original app encryption key and does not ask users for your signing passphrase.

The genuine published app is still 0.1.2. The updater is qualified with synthetic
future releases until a genuine new release is prepared, signed and published.
No live update has been performed as a test. Automatic Setup-EXE updates,
web-app notifications and security-advisory notices are separate remaining work.

## The September expiry

The 28 September date was superseded. Daily renewal was enabled on GitHub on
24 September after a successful dry run, real publication and anonymous signature
verification. Run 35968255120 passed. Independently verified channel sequence 5
expires 30 September at 5:11 pm Brisbane. Renewal runs daily around 11:17 am
Brisbane with this PC off. The first scheduled run had not happened at activation.

No weekly signing command, passphrase entry or EXE rebuild is required. GitHub
Actions can still fail; inspect failure notifications and the live workflow if
that happens. The EXE itself does not expire. An expired channel blocks new
install/update actions, but already-running websites keep working.

The existing encrypted signing-key backup remains important. Do not move keys
through chat or enable the obsolete web-repository renewal template.

Engineering details: [updater](app-updater.md), [maintenance](maintenance.md),
[backup](backup-contract.md), and the web repository's `APP_UPDATE_CONTRACT.md`
and `RELEASE_OPERATIONS.md`. These are implementation references for the agents;
you do not need to memorize them before continuing development.
