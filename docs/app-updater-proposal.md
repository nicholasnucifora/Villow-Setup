# Completed-installation app updates — contract proposal

Requested by the maintainer on 24 September 2026. This is the next capability
after the qualified, bounded Alpha repair. It is not an implemented update path
or permission to test against the maintainer's working installation. Windows
signing remains deferred; genuine app authentication remains mandatory.

## Owner experience

On a completed installation, Setup shows the installed Villow version and
**Check for updates**. A supported newer release shows its version, release notes,
expected interruption and **Update my app**. One click verifies prerequisites,
saves and verifies temporary recovery protection, performs only the authenticated
upgrade plan, deploys into the same Vercel project and waits for authenticated
health. Progress stays visible. Closing/reopening offers **Resume update** for
the same saved release. Successful checks permit temporary-copy cleanup.

Checking availability needs the signed public release listing and saved local
installation identity, not a fresh provider login or any database write. Starting
or resuming requires the saved credentials and verified provider targets. Expired
provider tokens lead to reconnecting the same accounts, never recreating projects.
No developer signing passphrase belongs in this user flow.

The release check distinguishes current, newer-but-incompatible, manager-too-old,
unavailable/offline and unauthenticated results. A newer version number by itself
never enables Update. Read-only recovery imports remain unable to update.

## Proposed division of work

The web repository owns immutable release artifacts, SQL and the source-to-target
upgrade declaration. Setup owns validation, encrypted recovery protection,
checkpointing, database execution, provider deployment and the interface. The
runtime never reads the web checkout. Agree the signed format and fixture before
enabling a mutating updater IPC.

Each destination release must explicitly authorize exact source manifest hashes.
The declaration must identify the source schema, ordered transactional upgrade
units and their checksums/postconditions, a destination postcondition, and the
backup inventory needed to recover the source. A fresh baseline is never an
upgrade. Code-only updates explicitly declare no app SQL and preserve the source
schema. New required configuration or Google permissions require a separately
supported Setup flow; they cannot silently broaden access during an update.

The native installation ledger must preserve historical fresh-install and Alpha
repair evidence. Repeated updates need an append-only receipt chain and exact
old-to-new instance digest change in the same transaction as upgrade SQL.
Completed Alpha repairs must be supported as sources as well as fresh 0.1.2
installs; their original ledgers are different. A missing/skipped version is
supported only where the destination declares and qualifies that exact source.

## Protection and recovery requirements

Both releases are authenticated, unrevoked and present in a fresh channel before
effects and resume. Save durable intent before SQL or deployment. After a lost
database response reconcile the exact operation receipt; after a lost deployment
response reconcile provider metadata, never submit a duplicate deployment.

Temporary backup needs a new verified source inventory for each supported schema
and native receipt history. The current backup is pinned to the 0.1.1 repair
source and must not be labelled a general updater backup by removing its digest
guard. Preserve original app encryption credentials. Encrypted backup read-back,
interrupted capture, wrong/missing key and cleanup retry remain required.

Transactional SQL failure rolls back that SQL transaction. Later deployment or
health failure retains backup and intent. A backup alone does not implement a
safe automatic rollback after users have written new data; recovery claims must
match tested behavior. Never replay a fresh baseline on the existing database.

## Qualification before enabling writes

Use disposable synthetic releases/databases for unit and protocol testing, then
the app-owned paired fixture for the exact candidate. Cover fresh versus repaired
source ledgers, repeated updates, skipped versions, schema/owner/receipt drift,
revocation, insufficient manager versions, changed permissions, populated data,
encrypted credential preservation, transactional rollback, lost SQL response,
lost deployment response, failed health, restart/resume and backup cleanup.

A future signed candidate and an authorized real-provider update are separate
release qualification. Do not publish a placeholder version to manufacture an
update or alter the already working 0.1.2 artifacts.
