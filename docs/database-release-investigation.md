# Fresh database verification failure — 22 September 2026

Status: a release compatibility defect is reproduced locally. A corrected authenticated app release and a reviewed continuation path are not yet available. Do not reset the maintainer's project, remove the saved installation, bypass verification or reuse its database as a test.

## Report and exact inputs

The maintainer's installed Setup `d9450faacf95a227e9ccc54fec4f0b5aa0385399` reached `DatabasePostcondition { unit: 1 }`. The SQL batch completed without a reported PostgreSQL error, but its result check was not true. The transaction containing the app baseline was not committed. The manager's separately committed installation-history schema can remain.

- Published app version: `0.1.0`, frozen web commit `1c6f949a3c0bc1b33f289b9684fa6ae84be766f4`.
- Manifest SHA-256: `5ea4a95f784c0038f7e7cc8483b8ef34d23084f086280fc26bb1a486eb69290c`.
- Archive SHA-256: `945c7fbb7ad6cc01d93a6aa9ee63c9f91c3d26699969b81239e4464c156a00bc`.
- Unit: `villow-fresh-158`, `migrations/fresh-baseline.sql`; verification: `migrations/postcondition.sql`.
- Expected catalog MD5: `67cceb14b7a0759ae92492a6a3325de9`.

Both downloaded artifact SHA-256 values were rechecked against the previously authenticated release; the exact SQL file hashes were checked against that manifest before local execution. The public archive was used only as diagnostic data in ignored artifacts, not as an implementation dependency or source migration directory.

## Reproduction

A new disposable loopback PostgreSQL 17.5 cluster used synthetic roles `anon`, `authenticated`, `service_role` and a synthetic `auth.uid()` stub. Each experiment ran the exact baseline and verification in a transaction followed by rollback. The cluster was stopped afterwards. No real Supabase connection, credentials or user data were accessed.

| Local case | Result check / hosted probe | Catalog MD5 |
| --- | --- | --- |
| Plain PostgreSQL defaults | true / true | `67cceb14b7a0759ae92492a6a3325de9` |
| Automatic table, sequence and function grants to API roles | false / false | `188cb57bb6f2b0f557db110d5a567ae5` |
| Add `extensions` to the search path, without automatic grants | true / true | `67cceb14b7a0759ae92492a6a3325de9` |
| Automatic grants, then explicitly remove unwanted anon/authenticated access in this local experiment | true / true | `67cceb14b7a0759ae92492a6a3325de9` |

With automatic grants, only the `table-access` and `function-access` category digests differ. The column, constraint, function definition, index, policy, table/RLS and trigger categories match. The release dump grants intended service-role access and revokes PUBLIC execution on restricted functions, but does not remove grants inherited directly by the named API roles from project defaults. Revoking PUBLIC does not revoke a named role's separate grant.

Supabase documents automatic grants and its transition toward opt-in defaults in [Securing your API](https://supabase.com/docs/guides/api/securing-your-api#default-privileges). Its [initial schema](https://github.com/supabase/postgres/blob/develop/migrations/db/init-scripts/00000000000000-initial-schema.sql) also defines default grants. These sources establish a supported environment variation; the maintainer's actual defaults have not been inspected, so this reproduction does not prove it is the only cause on that specific project.

The corrective SQL used in the fourth experiment is not shipped in Setup, is not an instruction to the maintainer and is not a replacement for authenticated app-owned SQL. The result proves that the original expected fingerprint is attainable with the intended effective privileges; accepting the differing fingerprint would hide a real permission difference.

Ignored evidence: `artifacts/postcondition-investigation/results.json`, `commands.json`, `run.log`, and `reproduce.mjs`. Initial helper attempts stopped on an empty PostgreSQL service-file variable and a missing synthetic auth schema; those fixture issues were corrected before the successful results above.

## Ownership and next work

The web-app agent received the exact release identity, reproduction, category evidence and requirements to normalize the intended ACLs in the app-owned release generator and test automatic grants on/off. The maintainer resolved that task's read-only code-comparison approval, and the web-app agent confirmed work on an isolated candidate. Existing published files, channel, signing key and visibility are not to be changed as part of diagnosis.

Setup currently pins the original manifest digest in its local checkpoint and in `villow_setup.instance`. Checking releases keeps that pin. A new published version alone therefore cannot repair this unfinished setup. A continuation design must preserve the same installation identity, vault entries and provider resources; authenticate the old and new release relationship; prove no app unit has committed and no app objects exist; and reconcile interrupted local/remote metadata updates. It must reject completed installations, nonempty migration history, mismatched ownership, imported recovery state and unknown outcomes. No writable adoption or app upgrade is authorized by this work.

The agents must agree and test that design before promising the maintainer can resume with a replacement release. Publication/signing follows the existing release approval procedure. Windows Authenticode remains deferred. Public qualification flags remain unchanged.
