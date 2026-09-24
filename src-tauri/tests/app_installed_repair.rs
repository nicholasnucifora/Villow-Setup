//! Opt-in app-owned fixture: real published source bytes, synthetic channel key,
//! and disposable loopback PostgreSQL. No production trust or provider access.
mod common;
use std::{collections::BTreeMap, fs, path::PathBuf};
use villow_setup::{error::Error, migration, model::*, release::*, repair_database};

#[test]
#[ignore = "requires the web-owned installed repair fixture and disposable local database"]
fn authenticates_and_repairs_the_installed_app_release() {
    assert_eq!(
        std::env::var("VILLOW_LOCAL_DB_TESTS").as_deref(),
        Ok("disposable-local-cluster")
    );
    let dir = PathBuf::from(std::env::var("VILLOW_INSTALLED_REPAIR_TEST_DIR").unwrap());
    let trust=Trust { format:1, repository:Some("nicholasnucifora/Villow-Setup".into()), channel:"stable".into(),
        manifest_url:Some("https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json".into()),
        public_keys:BTreeMap::from([("test-only".into(),fs::read_to_string(dir.join("test-public-key.txt")).unwrap().trim().into())]), minimum_sequence:1,
        publisher:Some("Synthetic installed repair qualification only".into()) };
    let (channel, _) = verify_channel(
        &fs::read(dir.join("channel.json")).unwrap(),
        &trust,
        chrono::Utc::now(),
    )
    .unwrap();
    let load = |name: &str| {
        let bytes = fs::read(dir.join(name).join("manifest.json")).unwrap();
        let pointer = channel
            .releases
            .iter()
            .find(|p| p.sha256 == hash(&bytes))
            .unwrap();
        verify_bundle(
            &bytes,
            &fs::read(dir.join(name).join("villow-source.zip")).unwrap(),
            pointer,
            &channel,
            &trust,
        )
        .unwrap()
    };
    let old = load("source");
    let new = load("target");
    assert_eq!(
        old.digest,
        "37072f3ef4efc387b57f6edcccdfd0e71b09fc61abcaee4649f9aba76fa332ac"
    );
    assert_eq!(new.manifest.schema.revision, "villow-fresh-159");
    assert_eq!(new.manifest.installed_repairs.len(), 1);
    assert_eq!(
        new.manifest.installed_repairs[0].from_manifest_sha256,
        old.digest
    );
    let connect = || {
        postgres::Config::new()
            .host("127.0.0.1")
            .port(
                std::env::var("VILLOW_LOCAL_DB_PORT")
                    .unwrap()
                    .parse()
                    .unwrap(),
            )
            .user("postgres")
            .password(std::env::var("VILLOW_LOCAL_DB_PASSWORD").unwrap())
            .dbname("manager_repair_check")
            .connect(postgres::NoTls)
            .unwrap()
    };
    let (_, _, mut s) = common::repair_fixture();
    s.release_digest = old.digest.clone();
    s.app_version = old.manifest.app_version.clone();
    s.commit = old.manifest.commit.clone();
    s.schema_revision = old.manifest.schema.revision.clone();
    s.release_sequence = old.manifest.sequence;
    s.google_scopes = old.manifest.google_scopes.clone();
    let mut c = connect();
    migration::apply(&mut c, &s, &old).unwrap();
    // Exercise the actual old owner/bootstrap function, not synthetic native history.
    c.query_one("SELECT public.villow_complete_signin($1::text::uuid,$2,'synthetic-google-id',$2,'synthetic-encrypted-access','synthetic-encrypted-refresh','synthetic-session',NULL,TRUE)",&[&s.id,&s.owner_email]).unwrap();
    let public_rows = |c: &mut postgres::Client| -> String {
        c.query_one("SELECT md5((SELECT string_agg(to_jsonb(u)::text,'' ORDER BY id) FROM public.users u) || (SELECT string_agg((to_jsonb(x)-'daily_watch_time_seconds'-'daily_watch_reset_at')::text,'' ORDER BY user_id) FROM public.user_settings x) || (SELECT row_to_json(i)::text FROM public.villow_installation i))",&[]).unwrap().get(0)
    };
    let before = public_rows(&mut c);
    let history=c.query_one("SELECT md5(string_agg(row_to_json(m)::text,'' ORDER BY id)) FROM villow_setup.migrations m",&[]).unwrap().get::<_,String>(0);
    repair_database::check_or_apply(&mut c, &s, &old, &new, false).unwrap();
    common::pending_repair(&mut s, &new);
    // Force failure after app SQL/result check, at the native CAS. All app DDL
    // and the repair receipt must roll back together. The fixture trigger lives
    // outside the public schema and never enters a signed release.
    c.batch_execute("CREATE FUNCTION villow_setup.test_refuse_update() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'synthetic CAS failure'; END $$; CREATE TRIGGER test_refuse_update BEFORE UPDATE ON villow_setup.instance FOR EACH ROW EXECUTE FUNCTION villow_setup.test_refuse_update()").unwrap();
    assert_eq!(
        repair_database::check_or_apply(&mut c, &s, &old, &new, true),
        Err(Error::RepairDatabase)
    );
    assert!(c
        .query_one("SELECT public.villow_setup_probe()", &[])
        .unwrap()
        .get::<_, bool>(0));
    assert!(c
        .query_one("SELECT to_regclass('villow_setup.repairs')::text", &[])
        .unwrap()
        .get::<_, Option<String>>(0)
        .is_none());
    assert_eq!(public_rows(&mut c), before);
    c.batch_execute("DROP TRIGGER test_refuse_update ON villow_setup.instance; DROP FUNCTION villow_setup.test_refuse_update()").unwrap();
    repair_database::check_or_apply(&mut c, &s, &old, &new, true).unwrap();
    drop(c);
    let mut c = connect();
    // Still at Database locally: reconcile a committed SQL result without replay.
    repair_database::check_or_apply(&mut c, &s, &old, &new, true).unwrap();
    s.installed_repair.as_mut().unwrap().phase = RepairPhase::Upload;
    repair_database::check_or_apply(&mut c, &s, &old, &new, false).unwrap();
    assert_eq!(public_rows(&mut c), before);
    assert_eq!(c.query_one("SELECT md5(string_agg(row_to_json(m)::text,'' ORDER BY id)) FROM villow_setup.migrations m",&[]).unwrap().get::<_,String>(0),history);
    assert_eq!(
        c.query_one("SELECT count(*) FROM villow_setup.repairs", &[])
            .unwrap()
            .get::<_, i64>(0),
        1
    );
    assert_eq!(
        c.query_one("SELECT release_digest FROM villow_setup.instance", &[])
            .unwrap()
            .get::<_, String>(0),
        new.digest
    );
    assert!(c
        .query_one("SELECT public.villow_setup_probe()", &[])
        .unwrap()
        .get::<_, bool>(0));
    assert_eq!(
        c.query_one("SELECT daily_watch_time_seconds FROM user_settings", &[])
            .unwrap()
            .get::<_, i32>(0),
        0
    );
    c.batch_execute("UPDATE villow_setup.repairs SET postcondition_checksum='tampered'")
        .unwrap();
    assert_eq!(
        repair_database::check_or_apply(&mut c, &s, &old, &new, false),
        Err(Error::RepairDatabase)
    );
}
