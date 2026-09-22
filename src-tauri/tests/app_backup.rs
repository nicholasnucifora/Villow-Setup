//! Explicit app-owned release fixture and disposable loopback PostgreSQL only.
mod common;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{collections::BTreeMap, fs, path::PathBuf};
use villow_setup::{
    backup_database, backup_file, migration, model::*, release::*, repair_backup, vault::Vault,
};

#[test]
#[ignore = "requires exact app release fixture and a newly created disposable local cluster"]
fn native_encrypted_backup_restores_exact_rows_and_refuses_existing_or_changed_targets() {
    assert_eq!(
        std::env::var("VILLOW_LOCAL_DB_TESTS").as_deref(),
        Ok("disposable-local-cluster")
    );
    let dir = PathBuf::from(std::env::var("VILLOW_INSTALLED_REPAIR_TEST_DIR").unwrap());
    let channel_bytes = fs::read(dir.join("channel.json")).unwrap();
    let trust=Trust {format:1,repository:Some("nicholasnucifora/Villow-Setup".into()),channel:"stable".into(),manifest_url:Some("https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json".into()),public_keys:BTreeMap::from([("test-only".into(),fs::read_to_string(dir.join("test-public-key.txt")).unwrap().trim().into())]),minimum_sequence:1,publisher:Some("Synthetic backup qualification".into())};
    let (channel, _) = verify_channel(&channel_bytes, &trust, chrono::Utc::now()).unwrap();
    let load = |folder: &str| {
        let manifest = fs::read(dir.join(folder).join("manifest.json")).unwrap();
        let archive = fs::read(dir.join(folder).join("villow-source.zip")).unwrap();
        let pointer = channel
            .releases
            .iter()
            .find(|p| p.sha256 == hash(&manifest))
            .unwrap();
        (
            verify_bundle(&manifest, &archive, pointer, &channel, &trust).unwrap(),
            manifest,
            archive,
        )
    };
    let (old, manifest, archive) = load("source");
    let (new, _, _) = load("target");
    assert_eq!(old.digest, backup_database::SOURCE);
    assert_eq!(new.digest, backup_database::DESTINATION);
    let port: u16 = std::env::var("VILLOW_LOCAL_DB_PORT")
        .unwrap()
        .parse()
        .unwrap();
    let password = std::env::var("VILLOW_LOCAL_DB_PASSWORD").unwrap();
    let connect = |db: &str, user: &str| {
        postgres::Config::new()
            .host("127.0.0.1")
            .port(port)
            .user(user)
            .password(&password)
            .dbname(db)
            .connect(postgres::NoTls)
            .unwrap()
    };
    let mut admin = connect("postgres", "postgres");
    // Role password is assigned as a literal only in this synthetic fixture.
    assert!(password
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    admin.batch_execute(&format!("CREATE ROLE backup_owner LOGIN PASSWORD '{password}' NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOREPLICATION")).unwrap();
    for db in ["backup_source", "backup_target", "backup_partial"] {
        admin
            .batch_execute(&format!("CREATE DATABASE {db}"))
            .unwrap();
        connect(db,"postgres").batch_execute(&format!("ALTER SCHEMA public OWNER TO backup_owner; GRANT CREATE ON DATABASE {db} TO backup_owner; CREATE SCHEMA auth; CREATE FUNCTION auth.uid() RETURNS uuid LANGUAGE sql STABLE AS $$SELECT NULL::uuid$$; GRANT USAGE ON SCHEMA auth TO backup_owner,anon,authenticated,service_role; CREATE SCHEMA storage; CREATE TABLE storage.provider_sentinel(value text); INSERT INTO storage.provider_sentinel VALUES('do not change');")).unwrap();
    }
    let (_, _, mut s) = common::repair_fixture();
    s.release_digest = old.digest.clone();
    s.commit = old.manifest.commit.clone();
    s.app_version = old.manifest.app_version.clone();
    s.schema_revision = old.manifest.schema.revision.clone();
    s.google_scopes = old.manifest.google_scopes.clone();
    s.release_sequence = old.manifest.sequence;
    let mut source = connect("backup_source", "backup_owner");
    migration::apply(&mut source, &s, &old).unwrap();
    source.query_one("SELECT public.villow_complete_signin($1::text::uuid,$2,'synthetic-owner',$2,'sealed:access','sealed:refresh','synthetic-session',NULL,TRUE)",&[&s.id,&s.owner_email]).unwrap();
    source.batch_execute("UPDATE public.user_settings SET daily_open_count=3,social_display_name=E'Unicode 🌿 tab\tnewline\nquote\" slash\\ end'; INSERT INTO public.task_integrations(user_id,provider,access_token_ciphertext,refresh_token_ciphertext,granted_scopes,sync_cursor) SELECT id,'google-tasks','sealed:tasks-access','sealed:tasks-refresh',ARRAY['one','two,three'],'{\"nested\":9007199254740993}'::jsonb FROM public.users; INSERT INTO public.things_to_do(user_id,integration_id,title,details,position,provider_metadata) SELECT user_id,id,'Preserve task',E'tab\tnewline\n\\N',9007199254740993,'{\"null\":null}'::jsonb FROM public.task_integrations; INSERT INTO public.villow_oauth_attempts VALUES('state','browser','sealed:verifier','2026-09-29T00:00:00Z')").unwrap();
    let vault = common::repair_vault(&s);
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("data.villowbackup");
    let unlock = "SYNTHETIC backup password only";
    let receipt = repair_backup::save(
        &file,
        unlock,
        &s,
        &old,
        &new,
        &trust,
        &channel_bytes,
        &manifest,
        &archive,
        &vault,
        false,
        || backup_database::capture(&mut source, &s, &old),
    )
    .unwrap();
    repair_backup::check_receipt(&receipt, &s, &old, &new).unwrap();
    assert!(backup_file::read(&file, "wrong password").is_err());
    let plaintext = backup_file::read(&file, unlock).unwrap();
    let package: repair_backup::Package = serde_json::from_slice(&plaintext).unwrap();
    assert_eq!(package.credentials.len(), 8);
    assert_eq!(package.database.tables.len(), 62);
    for secret in &package.credentials {
        assert_eq!(secret.value, *vault.require(&s.id, &secret.name).unwrap());
    }
    assert_eq!(STANDARD.decode(&package.archive_base64).unwrap(), archive);
    assert_eq!(STANDARD.decode(&package.manifest_base64).unwrap(), manifest);
    let saved: Installation = serde_json::from_str(&package.checkpoint_json).unwrap();
    assert_eq!(saved.id, s.id);
    assert_eq!(saved.origin, s.origin);
    // The runtime's automatic path produces the same complete restorable data,
    // using an OS-vault-owned key rather than a renderer-supplied password.
    let store = villow_setup::store::Store::open(temp.path()).unwrap();
    store.save(&s).unwrap();
    let (managed_path, managed_key) =
        villow_setup::managed_backup::prepare(&store, &vault, &s, &new.digest).unwrap();
    let managed_receipt = repair_backup::save(
        &managed_path,
        &managed_key,
        &s,
        &old,
        &new,
        &trust,
        &channel_bytes,
        &manifest,
        &archive,
        &vault,
        true,
        || backup_database::capture(&mut source, &s, &old),
    )
    .unwrap();
    villow_setup::managed_backup::check(&store, &vault, &s, &managed_receipt).unwrap();
    let managed_plaintext = backup_file::read(&managed_path, &managed_key).unwrap();
    let managed_package: repair_backup::Package =
        serde_json::from_slice(&managed_plaintext).unwrap();
    assert_eq!(
        serde_json::to_vec(&managed_package.database).unwrap(),
        serde_json::to_vec(&package.database).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&managed_package.credentials).unwrap(),
        serde_json::to_vec(&package.credentials).unwrap()
    );
    let mut target = connect("backup_target", "backup_owner");
    backup_database::restore_empty(&mut target, &s, &old, &package.database).unwrap();
    let restored = backup_database::capture(&mut target, &s, &old).unwrap();
    assert_eq!(
        serde_json::to_vec(&restored).unwrap(),
        serde_json::to_vec(&package.database).unwrap()
    );
    assert!(backup_database::restore_empty(&mut target, &s, &old, &package.database).is_err());
    assert_eq!(
        connect("backup_target", "postgres")
            .query_one("SELECT value FROM storage.provider_sentinel", &[])
            .unwrap()
            .get::<_, String>(0),
        "do not change"
    );
    let mut partial = connect("backup_partial", "backup_owner");
    let mut damaged = package.database.clone();
    damaged
        .tables
        .iter_mut()
        .find(|t| t.schema.name == "things_to_do")
        .unwrap()
        .copy_base64 = STANDARD.encode(b"invalid COPY data\n");
    assert!(backup_database::restore_empty(&mut partial, &s, &old, &damaged).is_err());
    assert_eq!(
        partial
            .query_one(
                "SELECT count(*) FROM pg_class WHERE relnamespace='public'::regnamespace",
                &[]
            )
            .unwrap()
            .get::<_, i64>(0),
        0
    );
    assert!(partial
        .query_one("SELECT to_regnamespace('villow_setup')::text", &[])
        .unwrap()
        .get::<_, Option<String>>(0)
        .is_none());
    // A syntactically valid row with a missing parent must still fail its FK.
    let mut invalid_fk = package.database.clone();
    let task = invalid_fk
        .tables
        .iter_mut()
        .find(|t| t.schema.name == "things_to_do")
        .unwrap();
    let user_column = task
        .schema
        .columns
        .iter()
        .position(|c| c.name == "user_id")
        .unwrap();
    let copy = String::from_utf8(STANDARD.decode(&task.copy_base64).unwrap()).unwrap();
    let mut fields = copy
        .trim_end_matches('\n')
        .split('\t')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    fields[user_column] = "99999999-9999-4999-8999-999999999999".into();
    task.copy_base64 = STANDARD.encode(format!("{}\n", fields.join("\t")));
    assert!(backup_database::restore_empty(&mut partial, &s, &old, &invalid_fk).is_err());
    // Extra objects can otherwise evade the release's ordinary schema probe.
    for (add, remove) in [
        ("CREATE SCHEMA outside; CREATE TABLE outside.child_users() INHERITS(public.users)","DROP TABLE outside.child_users; DROP SCHEMA outside"),
        ("ALTER TABLE villow_setup.migrations DROP CONSTRAINT migrations_pkey; ALTER TABLE villow_setup.migrations ADD PRIMARY KEY(checksum)","ALTER TABLE villow_setup.migrations DROP CONSTRAINT migrations_pkey; ALTER TABLE villow_setup.migrations ADD PRIMARY KEY(id)"),
        ("CREATE INDEX unexpected_native ON villow_setup.migrations(checksum)","DROP INDEX villow_setup.unexpected_native"),
        ("GRANT USAGE ON SCHEMA villow_setup TO PUBLIC; GRANT SELECT ON villow_setup.migrations TO PUBLIC","REVOKE ALL ON SCHEMA villow_setup FROM PUBLIC; REVOKE ALL ON villow_setup.migrations FROM PUBLIC"),
        ("GRANT SELECT(checksum) ON villow_setup.migrations TO anon","REVOKE SELECT(checksum) ON villow_setup.migrations FROM anon"),
        ("CREATE SCHEMA outside; CREATE TABLE outside.integration(updated_at timestamptz); CREATE TRIGGER touch BEFORE UPDATE ON outside.integration FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column()","DROP TABLE outside.integration; DROP SCHEMA outside"),
        (
            "CREATE SEQUENCE public.unexpected",
            "DROP SEQUENCE public.unexpected",
        ),
        (
            "CREATE VIEW public.unexpected AS SELECT 1 a",
            "DROP VIEW public.unexpected",
        ),
        (
            "CREATE TYPE public.unexpected AS ENUM ('x')",
            "DROP TYPE public.unexpected",
        ),
        (
            "ALTER TABLE villow_setup.instance ALTER COLUMN installation_id DROP NOT NULL",
            "ALTER TABLE villow_setup.instance ALTER COLUMN installation_id SET NOT NULL",
        ),
        (
            "ALTER TABLE public.users FORCE ROW LEVEL SECURITY",
            "ALTER TABLE public.users NO FORCE ROW LEVEL SECURITY",
        ),
        (
            "ALTER TABLE public.user_settings DISABLE TRIGGER USER",
            "ALTER TABLE public.user_settings ENABLE TRIGGER USER",
        ),
    ] {
        source.batch_execute(add).unwrap();
        assert!(
            backup_database::capture(&mut source, &s, &old).is_err(),
            "unsupported schema accepted: {add}"
        );
        source.batch_execute(remove).unwrap();
    }
    backup_database::capture(&mut source, &s, &old).unwrap();
    println!("Native encrypted file read-back, original 8 vault entries, 62-table exact restore, bigint/JSON/Unicode, non-superuser/FKs, provider preservation, rollback, nonempty and schema drift refusal passed. Disposable loopback only; no live restore authorization.");
}
