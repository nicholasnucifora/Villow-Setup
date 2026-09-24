# Distribution renewal automation

Owner-authorized on 24 September 2026. This directory runs only in the publisher's
GitHub workflow. It is not shipped in Setup, has no app/cloud credentials and does
not build, migrate or deploy Villow. New app releases and SQL remain web-owned.
The channel/signing helpers were copied from the reviewed web tooling; format 1,
the original public key and the native seven-day maximum remain unchanged.

The public distribution repository supplies a temporary `GITHUB_TOKEN` scoped
to itself. No personal publishing token is stored. The private web repository
does not need to become public or purchase private-branch protection for this job.

Protect `main`; configure the `villow-channel-renewal` environment to accept only
`main`, with `VILLOW_RELEASE_SIGNING_KEY` as its sole signing secret. No PR or
arbitrary-ref trigger receives the key. The initial owner-run provisioning helper
decrypts the existing key locally, verifies its public key, and sends it directly
to GitHub's encrypted secret API through the official CLI. Never log/export it.

`workflow_dispatch` defaults to a signing dry run. Scheduled publication remains
disabled until `VILLOW_CHANNEL_RENEWAL_ENABLED=true`. Enable only after a passing
dry run and a verified publication. Observe the first scheduled run separately.
Daily attempts provide retry time; GitHub schedules can be delayed or disabled.
Retain GitHub Actions failure notifications for the account enabling the schedule.

Every published renewal is downloaded anonymously and verified byte-for-byte.
The public download check requests cache revalidation and retries reads with
170 seconds of total backoff, plus bounded request timeouts, to allow replacement
assets to propagate. A stale hash never passes. Failures include safe HTTP/hash
diagnostics; public candidate, previous listing and plan artifacts are retained
even on failure so an upload can be reconciled before any new publication.
The job then commits the public signed listing and verification receipt to
`codex/channel-renewal-records`, leaving application branches unchanged. This
provides an audit history and repository activity during periods without feature
commits, addressing GitHub's 60-day public-repository inactivity rule. A failed
receipt write fails the job; do not silently rely on the schedule staying active.
GitHub's workflow token does not recursively trigger push workflows for these
receipt commits. The receipt does not override native signature/expiry checks.

All channel writers must serialize publication. Before an out-of-band app release
or revocation, disable the renewal variable, wait for any active renewal job to
finish, then fetch and authenticate the current channel before signing. After
verified publication, restore renewal. The last-moment hash check is not an atomic
lock across manual publishers. A missing/uncertain channel asset requires review;
do not manufacture trust or assume a failed upload was rolled back.

Checks: `node --test release-automation/tests/*.checks.mjs`. These use disposable
keys/data. Hosted dry-run, live publication, owner notifications and scheduled-run
observations are recorded separately; local tests alone do not prove activation.
