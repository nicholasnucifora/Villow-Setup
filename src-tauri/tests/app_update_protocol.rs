//! Opt-in paired fixture, isolated PostgreSQL cluster. No cloud credentials.
mod common;
use std::{collections::BTreeMap, fs, path::PathBuf};
use villow_setup::{
    error::Error, migration, model::*, release::*, update_backup_database, update_contract,
    update_database,
};
#[test]
#[ignore = "requires explicit app-owned synthetic release fixture and disposable cluster"]
fn authenticated_updates_preserve_fresh_and_repaired_history_and_restore_data() {
    assert_eq!(
        std::env::var("VILLOW_LOCAL_DB_TESTS").as_deref(),
        Ok("disposable-local-cluster")
    );
    let dir = PathBuf::from(std::env::var("VILLOW_APP_UPDATE_TEST_DIR").unwrap());
    let info: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("fixture.json")).unwrap()).unwrap();
    assert_eq!(info["test_only"], true);
    let trust=Trust{format:1,repository:Some("nicholasnucifora/Villow-Setup".into()),channel:"stable".into(),manifest_url:Some("https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json".into()),public_keys:BTreeMap::from([("test-only".into(),fs::read_to_string(dir.join("test-public-key.txt")).unwrap().trim().into())]),minimum_sequence:1,publisher:Some("Synthetic updater qualification".into())};
    let channel_bytes = fs::read(dir.join("channel.json")).unwrap();
    let (channel, _) = verify_channel(&channel_bytes, &trust, chrono::Utc::now()).unwrap();
    let load = |folder: &str| {
        let m = fs::read(dir.join(folder).join("manifest.json")).unwrap();
        let a = fs::read(dir.join(folder).join("villow-source.zip")).unwrap();
        let p = channel
            .releases
            .iter()
            .find(|p| p.sha256 == hash(&m))
            .unwrap();
        let r = verify_bundle(&m, &a, p, &channel, &trust)
            .unwrap_or_else(|e| panic!("{folder}: {e:?}"));
        for plan in &r.manifest.app_updates {
            let vector = info["releases"][folder]["plan_hashes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["id"] == plan.id)
                .unwrap();
            assert_eq!(
                update_contract::canonical_hash(plan).unwrap(),
                vector["sha256"].as_str().unwrap()
            );
        }
        r
    };
    let a = load("source");
    let original = load("repaired_source");
    let b = load("b");
    let c = load("c");
    let d = load("d");
    assert!(update_contract::transition(&a, &c).is_err());
    let mut changed = b.clone();
    changed.manifest.google_scopes.push("unexpected".into());
    assert!(update_contract::transition(&a, &changed).is_err());
    changed = b.clone();
    changed
        .files
        .get_mut(&changed.manifest.schema.migrations[0].file)
        .unwrap()
        .push(b' ');
    assert!(update_contract::transition(&a, &changed).is_err());
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
    assert!(password
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    admin.batch_execute(&format!("CREATE ROLE anon; CREATE ROLE authenticated; CREATE ROLE service_role; CREATE ROLE unexpected_update_grantee; CREATE ROLE update_owner LOGIN PASSWORD '{password}' NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOREPLICATION")).unwrap();
    for db in [
        "update_fresh",
        "update_repaired",
        "restore_fresh",
        "restore_repaired",
        "restore_history",
        "update_skip",
    ] {
        admin
            .batch_execute(&format!("CREATE DATABASE {db}"))
            .unwrap();
        connect(db,"postgres").batch_execute(&format!("ALTER SCHEMA public OWNER TO update_owner; GRANT CREATE ON DATABASE {db} TO update_owner; CREATE SCHEMA auth; CREATE FUNCTION auth.uid() RETURNS uuid LANGUAGE sql STABLE AS $$SELECT NULL::uuid$$; GRANT USAGE ON SCHEMA auth TO update_owner,anon,authenticated,service_role; CREATE SCHEMA storage; CREATE TABLE storage.provider_sentinel(value text); INSERT INTO storage.provider_sentinel VALUES('untouched');")).unwrap();
    }
    for repaired in [false, true] {
        let db = if repaired {
            "update_repaired"
        } else {
            "update_fresh"
        };
        let mut source = connect(db, "update_owner");
        let (_, _, mut s) = common::repair_fixture();
        pin(&mut s, if repaired { &original } else { &a });
        s.step = Step::Health;
        migration::apply(&mut source, &s, if repaired { &original } else { &a }).unwrap();
        source.query_one("SELECT public.villow_complete_signin($1::text::uuid,$2,'synthetic-owner',$2,'sealed:access','sealed:refresh','synthetic-session',NULL,TRUE)",&[&s.id,&s.owner_email]).unwrap();
        source.batch_execute("UPDATE public.user_settings SET daily_open_count=3,social_display_name=E'Unicode 🌿 tab\tnewline\nquote\" slash\\ end'; INSERT INTO public.villow_oauth_attempts VALUES('state','browser','sealed:verifier','2026-09-29T00:00:00Z')").unwrap();
        if repaired {
            let p = &a.manifest.installed_repairs[0];
            s.installed_repair = Some(intent(&s, &original, &a, &p.id));
            villow_setup::repair_database::check_or_apply(&mut source, &s, &original, &a, true)
                .unwrap();
            let op = s.installed_repair.as_ref().unwrap().operation_id.clone();
            s.installed_repair.as_mut().unwrap().phase = RepairPhase::Complete;
            s.operation_id = op;
            pin(&mut s, &a);
        }
        s.step = Step::Complete;
        s.update_lineage = Some(
            update_database::initial(&s, &a, if repaired { Some(&original) } else { None })
                .unwrap(),
        );
        if !repaired {
            for (mutate,undo) in [
                ("GRANT SELECT ON public.users TO unexpected_update_grantee","REVOKE SELECT ON public.users FROM unexpected_update_grantee"),
                ("GRANT EXECUTE ON FUNCTION public.villow_setup_probe() TO unexpected_update_grantee","REVOKE EXECUTE ON FUNCTION public.villow_setup_probe() FROM unexpected_update_grantee"),
                ("GRANT SELECT(email) ON public.users TO anon","REVOKE SELECT(email) ON public.users FROM anon"),
                ("GRANT SELECT ON public.users TO service_role WITH GRANT OPTION","REVOKE GRANT OPTION FOR SELECT ON public.users FROM service_role"),
                ("REVOKE MAINTAIN ON public.users FROM service_role","GRANT MAINTAIN ON public.users TO service_role"),
                ("GRANT MAINTAIN ON public.users TO authenticated","REVOKE MAINTAIN ON public.users FROM authenticated"),
                ("GRANT SELECT ON villow_setup.instance TO anon","REVOKE SELECT ON villow_setup.instance FROM anon"),
                ("REVOKE UPDATE ON public.users FROM update_owner","GRANT UPDATE ON public.users TO update_owner"),
                ("REVOKE EXECUTE ON FUNCTION public.villow_setup_probe() FROM update_owner","GRANT EXECUTE ON FUNCTION public.villow_setup_probe() TO update_owner"),
                ("ALTER TABLE public.user_settings ALTER COLUMN social_display_name TYPE text COLLATE \"C\"","ALTER TABLE public.user_settings ALTER COLUMN social_display_name TYPE text COLLATE \"default\""),
                ("ALTER TABLE public.user_settings ALTER COLUMN social_display_name SET (n_distinct=10)","ALTER TABLE public.user_settings ALTER COLUMN social_display_name RESET (n_distinct)"),
                ("CREATE SCHEMA app_update_external; CREATE DOMAIN app_update_external.label AS text; ALTER TABLE public.user_settings ALTER COLUMN social_display_name TYPE app_update_external.label","ALTER TABLE public.user_settings ALTER COLUMN social_display_name TYPE text; DROP DOMAIN app_update_external.label; DROP SCHEMA app_update_external"),
            ] {
                source.batch_execute(mutate).unwrap();
                assert!(source.query_one(include_str!("../src/update_backup_scope.sql"),&[]).unwrap().get::<_,bool>(0),"scope: {mutate}");
                assert!(update_backup_database::capture(&mut source,&s,&a,&b).is_err(),"capture: {mutate}");
                assert!(update_database::check_or_apply(&mut source,&s,&a,&b,false).is_err(),"apply: {mutate}");
                source.batch_execute(undo).unwrap();
            }
        }
        let snapshot = update_backup_database::capture(&mut source, &s, &a, &b).unwrap();
        assert_eq!(snapshot.tables.len(), if repaired { 63 } else { 62 });
        let restore_db = if repaired {
            "restore_repaired"
        } else {
            "restore_fresh"
        };
        let mut restore = connect(restore_db, "update_owner");
        update_backup_database::restore_empty(&mut restore, &s, &a, &b, &snapshot).unwrap();
        let roundtrip = update_backup_database::capture(&mut restore, &s, &a, &b).unwrap();
        assert_eq!(
            serde_json::to_vec(&snapshot).unwrap(),
            serde_json::to_vec(&roundtrip).unwrap()
        );
        assert!(
            update_backup_database::restore_empty(&mut restore, &s, &a, &b, &snapshot).is_err()
        );
        if !repaired {
            lifecycle(&mut restore, &s, &a, &b, &dir, &trust, &channel_bytes);
        }
        let original_rows: String = source
            .query_one(
                "SELECT row_to_json(u)::text FROM public.user_settings u",
                &[],
            )
            .unwrap()
            .get(0);
        let mut skip_state = s.clone();
        if !repaired {
            let mut skip = connect("update_skip", "update_owner");
            update_backup_database::restore_empty(&mut skip, &s, &a, &d, &snapshot).unwrap();
            apply(&mut skip, &mut skip_state, &a, &d);
            assert_eq!(
                skip_state.update_lineage.as_ref().unwrap().receipts.len(),
                1
            );
        }
        apply(&mut source, &mut s, &a, &b);
        // A failed final check rolls back the entire additive unit and receipt.
        let mut broken = c.clone();
        let final_path = broken.manifest.schema.migrations[0].postcondition.clone();
        broken.files.insert(final_path, b"SELECT false".to_vec());
        s.app_update = Some(intent(&s, &b, &broken, &broken.manifest.app_updates[0].id));
        assert_eq!(
            update_database::check_or_apply(&mut source, &s, &b, &broken, true),
            Err(Error::UpdateDatabase)
        );
        let present:bool=source.query_one("SELECT EXISTS(SELECT FROM information_schema.columns WHERE table_schema='public' AND table_name='user_settings' AND column_name='update_fixture_marker')",&[]).unwrap().get(0);
        assert!(!present);
        s.app_update = None;
        let history_snapshot = update_backup_database::capture(&mut source, &s, &b, &c).unwrap();
        assert_eq!(
            history_snapshot.tables.len(),
            if repaired { 64 } else { 63 }
        );
        if !repaired {
            let mut restore = connect("restore_history", "update_owner");
            update_backup_database::restore_empty(&mut restore, &s, &b, &c, &history_snapshot)
                .unwrap();
        }
        apply(&mut source, &mut s, &b, &c);
        apply(&mut source, &mut s, &c, &d);
        assert_eq!(s.update_lineage.as_ref().unwrap().receipts.len(), 3);
        let values: String = source
            .query_one(
                "SELECT (to_jsonb(u)-'update_fixture_marker')::text FROM public.user_settings u",
                &[],
            )
            .unwrap()
            .get(0);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&original_rows).unwrap(),
            serde_json::from_str::<serde_json::Value>(&values).unwrap()
        );
        assert_eq!(
            connect(db, "postgres")
                .query_one("SELECT value FROM storage.provider_sentinel", &[])
                .unwrap()
                .get::<_, String>(0),
            "untouched"
        );
        source
            .batch_execute(
                "UPDATE villow_setup.app_updates SET receipt_hash=repeat('0',64) WHERE position=1",
            )
            .unwrap();
        let mut tx = source.transaction().unwrap();
        assert!(update_database::source(&mut tx, &s, &d).is_err());
        tx.rollback().unwrap();
    }
}
fn lifecycle(
    client: &mut postgres::Client,
    s: &Installation,
    a: &VerifiedRelease,
    b: &VerifiedRelease,
    dir: &std::path::Path,
    trust: &Trust,
    channel: &[u8],
) {
    use villow_setup::{
        app_update_engine as engine, backup_file, managed_update_backup as managed, repair_backup,
        store::Store, vault::Vault,
    };
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(temp.path()).unwrap();
    store.save(s).unwrap();
    let vault = common::repair_vault(s);
    let providers = common::Fake::new(temp.path().join("cloud.json"));
    let (path, key) = managed::prepare(&store, &vault, s, &b.digest).unwrap();
    assert_eq!(
        villow_setup::engine::remove_credentials(&store, &vault),
        Err(Error::BackupRetained)
    );
    let receipt = repair_backup::save_update(
        &path,
        &key,
        s,
        a,
        b,
        trust,
        channel,
        &fs::read(dir.join("source/manifest.json")).unwrap(),
        &fs::read(dir.join("source/villow-source.zip")).unwrap(),
        &fs::read(dir.join("b/manifest.json")).unwrap(),
        &fs::read(dir.join("b/villow-source.zip")).unwrap(),
        &vault,
        || update_backup_database::capture(client, s, a, b),
    )
    .unwrap();
    assert_eq!(
        villow_setup::engine::forget(&store, &vault, &s.name),
        Err(Error::BackupRetained)
    );
    let bytes = backup_file::read(&path, &key).unwrap();
    let package: repair_backup::Package = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(package.format, 2);
    assert_eq!(package.credentials.len(), 8);
    assert!(!package.update_archive_base64.is_empty());
    for c in &package.credentials {
        assert_eq!(c.value, *vault.require(&s.id, &c.name).unwrap());
    }
    assert!(backup_file::read(&path, "wrong password").is_err());
    assert_eq!(
        engine::advance(
            &store,
            &vault,
            &providers,
            s.clone(),
            a,
            b,
            Some(receipt),
            |s, apply| {
                update_database::check_or_apply(client, s, a, b, apply)?;
                if apply {
                    Err(Error::Uncertain)
                } else {
                    Ok(())
                }
            }
        )
        .err(),
        Some(Error::Uncertain)
    );
    let pending = store.load().unwrap().unwrap();
    assert_eq!(
        pending.app_update.as_ref().unwrap().phase,
        RepairPhase::Database
    );
    assert!(path.exists());
    assert_eq!(
        villow_setup::engine::remove_credentials(&store, &vault),
        Err(Error::UpdatePending)
    );
    let imported =
        villow_setup::recovery::import(&villow_setup::recovery::export(&pending).unwrap()).unwrap();
    assert!(
        imported.update_lineage.is_none() && imported.app_update.is_none() && imported.read_only
    );
    vault.put(&s.id, managed::KEY, &"0".repeat(64)).unwrap();
    assert!(engine::advance(
        &store,
        &vault,
        &providers,
        pending.clone(),
        a,
        b,
        None,
        |_, _| panic!("wrong key must stop before SQL")
    )
    .is_err());
    vault.put(&s.id, managed::KEY, &key).unwrap();
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    let resumed = store.load().unwrap().unwrap();
    engine::advance(
        &store,
        &vault,
        &providers,
        resumed,
        a,
        b,
        None,
        |s, apply| update_database::check_or_apply(client, s, a, b, apply),
    )
    .unwrap();
    assert_eq!(
        client
            .query_one("SELECT count(*) FROM villow_setup.app_updates", &[])
            .unwrap()
            .get::<_, i64>(0),
        1
    );
    providers.deployment_lost_response.set(true);
    assert_eq!(
        engine::advance(
            &store,
            &vault,
            &providers,
            store.load().unwrap().unwrap(),
            a,
            b,
            None,
            |s, apply| update_database::check_or_apply(client, s, a, b, apply)
        )
        .err(),
        Some(Error::Uncertain)
    );
    assert_eq!(
        store
            .load()
            .unwrap()
            .unwrap()
            .app_update
            .as_ref()
            .unwrap()
            .phase,
        RepairPhase::Deploy
    );
    engine::advance(
        &store,
        &vault,
        &providers,
        store.load().unwrap().unwrap(),
        a,
        b,
        None,
        |s, apply| update_database::check_or_apply(client, s, a, b, apply),
    )
    .unwrap();
    assert_eq!(providers.read().deploys, 1);
    providers.health_ok.set(false);
    assert_eq!(
        engine::advance(
            &store,
            &vault,
            &providers,
            store.load().unwrap().unwrap(),
            a,
            b,
            None,
            |s, apply| update_database::check_or_apply(client, s, a, b, apply)
        )
        .err(),
        Some(Error::Health)
    );
    assert!(path.exists());
    assert!(vault.get(&s.id, managed::KEY).unwrap().is_some());
    let mut pending = store.load().unwrap().unwrap();
    managed::cleanup(&store, &vault, &mut pending).unwrap();
    assert!(path.exists());
    providers.health_ok.set(true);
    let done = engine::advance(
        &store,
        &vault,
        &providers,
        pending,
        a,
        b,
        None,
        |s, apply| update_database::check_or_apply(client, s, a, b, apply),
    )
    .unwrap();
    assert_eq!(
        done.app_update.as_ref().unwrap().phase,
        RepairPhase::Complete
    );
    assert_eq!(done.update_lineage.as_ref().unwrap().receipts.len(), 1);
    assert!(!path.exists());
    assert!(done
        .app_update
        .as_ref()
        .unwrap()
        .backup
        .as_ref()
        .unwrap()
        .removed_at
        .is_some());
    assert!(vault.get(&s.id, managed::KEY).unwrap().is_none());
    for c in &package.credentials {
        assert_eq!(c.value, *vault.require(&s.id, &c.name).unwrap());
    }
    assert_eq!(providers.read().deploys, 1);
    println!("APP_UPDATE_LIFECYCLE_PASSED: encrypted read-back, original credentials, lost SQL response, restart, lost deployment response, failed health retention, recovery authority stripping, cleanup");
}
fn pin(s: &mut Installation, r: &VerifiedRelease) {
    s.release_digest = r.digest.clone();
    s.commit = r.manifest.commit.clone();
    s.app_version = r.manifest.app_version.clone();
    s.schema_revision = r.manifest.schema.revision.clone();
    s.google_scopes = r.manifest.google_scopes.clone();
    s.release_sequence = r.manifest.sequence;
}
fn intent(s: &Installation, a: &VerifiedRelease, b: &VerifiedRelease, id: &str) -> RepairIntent {
    RepairIntent {
        from: a.digest.clone(),
        to: b.digest.clone(),
        repair_id: id.into(),
        operation_id: uuid::Uuid::new_v4().to_string(),
        backup_confirmed_at: now(),
        backup: None,
        previous_operation_id: s.operation_id.clone(),
        previous_deployment_id: s.deployment_id.clone().unwrap(),
        phase: RepairPhase::Database,
        deployment_id: None,
        deployment_status: None,
    }
}
fn apply(
    client: &mut postgres::Client,
    s: &mut Installation,
    a: &VerifiedRelease,
    b: &VerifiedRelease,
) {
    let plan = update_contract::transition(a, b).unwrap();
    s.app_update = Some(intent(s, a, b, &plan.id));
    update_database::check_or_apply(client, s, a, b, true).unwrap();
    let receipt = update_database::receipt(s, a, b).unwrap();
    update_database::check_or_apply(client, s, a, b, true).unwrap(); // lost successful response reconciles, never repeats SQL
    s.update_lineage.as_mut().unwrap().receipts.push(receipt);
    s.operation_id = s.app_update.as_ref().unwrap().operation_id.clone();
    s.app_update = None;
    pin(s, b);
}
