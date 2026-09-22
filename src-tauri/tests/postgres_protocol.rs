//! Opt-in local disposable PostgreSQL tests. Never accepts a cloud connection URL.
mod common;
use common::*;
use postgres::{Client, NoTls};
use villow_setup::{
    error::{Error, MigrationStage},
    migration,
    release::hash,
};

struct Database {
    admin: Client,
    name: String,
    port: u16,
    password: String,
}
impl Database {
    fn new() -> Self {
        assert_eq!(
            std::env::var("VILLOW_LOCAL_DB_TESTS").as_deref(),
            Ok("disposable-local-cluster"),
            "Requires an explicitly authorized disposable LOCAL PostgreSQL cluster"
        );
        let port = std::env::var("VILLOW_LOCAL_DB_PORT")
            .unwrap_or_else(|_| "5432".into())
            .parse()
            .unwrap();
        let password =
            std::env::var("VILLOW_LOCAL_DB_PASSWORD").expect("Set synthetic test-cluster password");
        let mut admin = connect(port, &password);
        admin.batch_execute("DO $$ BEGIN IF NOT EXISTS(SELECT FROM pg_roles WHERE rolname='service_role') THEN CREATE ROLE service_role; END IF; END $$;").unwrap();
        admin.batch_execute("DO $$ BEGIN IF NOT EXISTS(SELECT FROM pg_roles WHERE rolname='anon') THEN CREATE ROLE anon; END IF; IF NOT EXISTS(SELECT FROM pg_roles WHERE rolname='authenticated') THEN CREATE ROLE authenticated; END IF; END $$;").expect("Test roles");
        let name = format!("villow_test_{}", uuid::Uuid::new_v4().simple());
        admin
            .batch_execute(&format!("CREATE DATABASE {name}"))
            .expect("Disposable database");
        Self {
            admin,
            name,
            port,
            password,
        }
    }
    fn client(&self) -> Client {
        postgres::Config::new()
            .host("127.0.0.1")
            .port(self.port)
            .user("postgres")
            .password(&self.password)
            .dbname(&self.name)
            .connect(NoTls)
            .expect("Local test database")
    }
}
fn connect(port: u16, password: &str) -> Client {
    postgres::Config::new()
        .host("127.0.0.1")
        .port(port)
        .user("postgres")
        .password(password)
        .dbname("postgres")
        .connect(NoTls)
        .expect("Local test server; no remote URLs accepted")
}

fn seed_repair_owner(c: &mut Client, s: &villow_setup::model::Installation) {
    c.execute("INSERT INTO users(id,email,google_id,is_system_owner,encrypted_token) VALUES('00000000-0000-4000-8000-000000000001',$1,'synthetic-google-id',TRUE,'SENTINEL-encrypted-preserve')", &[&s.owner_email]).unwrap();
    c.batch_execute(
        "INSERT INTO user_settings(user_id) VALUES('00000000-0000-4000-8000-000000000001')",
    )
    .unwrap();
    c.execute("INSERT INTO villow_installation VALUES(TRUE,$1::text::uuid,$2,'00000000-0000-4000-8000-000000000001',$2,'synthetic-google-id',now(),now())", &[&s.id,&s.owner_email]).unwrap();
}

#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn installed_repair_keeps_populated_data_and_reconciles_a_committed_receipt() {
    use villow_setup::{model::RepairPhase, repair_database};
    let db = Database::new();
    let mut c = db.client();
    let (old, new, mut s) = repair_fixture();
    migration::apply(&mut c, &s, &old).unwrap();
    assert_eq!(
        repair_database::check_or_apply(&mut c, &s, &old, &new, false),
        Err(Error::RepairDatabase)
    );
    seed_repair_owner(&mut c, &s);
    c.execute("INSERT INTO synthetic VALUES(1,'keep-existing-data')", &[])
        .unwrap();
    let ledger = c
        .query_one(
            "SELECT row_to_json(m)::text FROM villow_setup.migrations m",
            &[],
        )
        .unwrap()
        .get::<_, String>(0);
    repair_database::check_or_apply(&mut c, &s, &old, &new, false).unwrap();
    let mut second = db.client();
    c.query_one("SELECT pg_advisory_lock($1)", &[&0x56494c4c4f57i64])
        .unwrap();
    assert_eq!(
        repair_database::check_or_apply(&mut second, &s, &old, &new, false),
        Err(Error::Busy)
    );
    c.query_one("SELECT pg_advisory_unlock($1)", &[&0x56494c4c4f57i64])
        .unwrap();
    pending_repair(&mut s, &new);
    repair_database::check_or_apply(&mut c, &s, &old, &new, true).unwrap();
    drop(c);
    let mut c = db.client();
    // The local checkpoint still says Database, modelling a lost COMMIT reply.
    repair_database::check_or_apply(&mut c, &s, &old, &new, true).unwrap();
    s.installed_repair.as_mut().unwrap().phase = RepairPhase::Upload;
    repair_database::check_or_apply(&mut c, &s, &old, &new, false).unwrap();
    assert_eq!(
        c.query_one("SELECT count(*) FROM villow_setup.repairs", &[])
            .unwrap()
            .get::<_, i64>(0),
        1
    );
    assert_eq!(
        c.query_one(
            "SELECT row_to_json(m)::text FROM villow_setup.migrations m",
            &[]
        )
        .unwrap()
        .get::<_, String>(0),
        ledger
    );
    assert_eq!(
        c.query_one("SELECT encrypted_token FROM users", &[])
            .unwrap()
            .get::<_, String>(0),
        "SENTINEL-encrypted-preserve"
    );
    assert_eq!(
        c.query_one("SELECT value FROM synthetic WHERE id=1", &[])
            .unwrap()
            .get::<_, String>(0),
        "keep-existing-data"
    );
    assert_eq!(
        c.query_one("SELECT daily_watch_time_seconds FROM user_settings", &[])
            .unwrap()
            .get::<_, i32>(0),
        0
    );
    assert_eq!(
        c.query_one("SELECT release_digest FROM villow_setup.instance", &[])
            .unwrap()
            .get::<_, String>(0),
        new.digest
    );
    c.batch_execute("UPDATE villow_setup.repairs SET sql_checksum='tampered'")
        .unwrap();
    assert_eq!(
        repair_database::check_or_apply(&mut c, &s, &old, &new, false),
        Err(Error::RepairDatabase)
    );
}

#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn installed_repair_refuses_drift_or_wrong_owner_and_rolls_back_every_write() {
    use villow_setup::repair_database;
    for kind in 0..7 {
        let db = Database::new();
        let mut c = db.client();
        let (old, mut new, mut s) = repair_fixture();
        migration::apply(&mut c, &s, &old).unwrap();
        seed_repair_owner(&mut c, &s);
        match kind {
            0 => {
                c.batch_execute("UPDATE villow_setup.instance SET installation_id='someone-else'")
                    .unwrap();
            }
            1 => {
                c.batch_execute("UPDATE villow_setup.migrations SET checksum='tampered'")
                    .unwrap();
            }
            2 => {
                c.batch_execute("UPDATE villow_installation SET owner_email='other@example.test'")
                    .unwrap();
            }
            3 => {
                c.batch_execute(
                    "ALTER TABLE user_settings ADD COLUMN daily_watch_time_seconds text",
                )
                .unwrap();
            }
            4 => {
                new.files
                    .get_mut("migrations/repair-159.sql")
                    .unwrap()
                    .extend_from_slice(b"SELECT 1/0;");
                new = reseal(new);
            }
            5 => {
                new.files.insert(
                    "migrations/postcondition.sql".into(),
                    b"SELECT false".to_vec(),
                );
                new = reseal(new);
            }
            _ => {
                c.batch_execute("ALTER TABLE villow_setup.instance ALTER COLUMN installation_id TYPE bytea USING convert_to(installation_id,'UTF8')").unwrap();
            }
        }
        pending_repair(&mut s, &new);
        assert_eq!(
            repair_database::check_or_apply(&mut c, &s, &old, &new, true),
            Err(Error::RepairDatabase)
        );
        assert_eq!(
            c.query_one("SELECT release_digest FROM villow_setup.instance", &[])
                .unwrap()
                .get::<_, String>(0),
            old.digest
        );
        assert!(c
            .query_one("SELECT to_regclass('villow_setup.repairs')::text", &[])
            .unwrap()
            .get::<_, Option<String>>(0)
            .is_none());
        if kind != 3 {
            assert!(!c.query_one("SELECT EXISTS(SELECT FROM information_schema.columns WHERE table_name='user_settings' AND column_name='daily_watch_time_seconds')",&[]).unwrap().get::<_,bool>(0));
        }
        assert_eq!(
            c.query_one("SELECT encrypted_token FROM users", &[])
                .unwrap()
                .get::<_, String>(0),
            "SENTINEL-encrypted-preserve"
        );
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        let _ = self
            .admin
            .batch_execute(&format!("DROP DATABASE {} WITH (FORCE)", self.name));
    }
}

#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn real_postgres_transactions_drift_and_locks() {
    let db = Database::new();
    let mut c = db.client();
    let r = verified();
    let s = installation(&r);
    migration::apply(&mut c, &s, &r).unwrap();
    c.execute("INSERT INTO public.synthetic VALUES(1,'preserve-me')", &[])
        .unwrap();
    migration::apply(&mut c, &s, &r).unwrap();
    assert_eq!(
        c.query_one("SELECT count(*) FROM villow_setup.migrations", &[])
            .unwrap()
            .get::<_, i64>(0),
        1
    );
    assert_eq!(
        c.query_one("SELECT value FROM public.synthetic WHERE id=1", &[])
            .unwrap()
            .get::<_, String>(0),
        "preserve-me"
    );
    let mut second = db.client();
    c.query_one("SELECT pg_advisory_lock($1)", &[&0x56494c4c4f57i64])
        .unwrap();
    assert_eq!(migration::apply(&mut second, &s, &r), Err(Error::Busy));
    c.query_one("SELECT pg_advisory_unlock($1)", &[&0x56494c4c4f57i64])
        .unwrap();
    c.execute(
        "UPDATE villow_setup.migrations SET checksum='tampered'",
        &[],
    )
    .unwrap();
    assert_eq!(migration::apply(&mut c, &s, &r), Err(Error::SchemaDrift));
}
#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn failed_sql_or_postcondition_rolls_back_entire_unit() {
    for (path, sql, expected) in [
        ("migrations/baseline.sql", "CREATE TABLE public.partial_effect (id int); SELECT 1/0;", Error::DatabaseMigration { unit: 1, stage: MigrationStage::Sql, code: "22012".into() }),
        ("migrations/verify.sql", "SELECT false;", Error::DatabasePostcondition { unit: 1 }),
        ("migrations/verify.sql", "SELECT 1/0;", Error::DatabaseMigration { unit: 1, stage: MigrationStage::Verification, code: "22012".into() }),
        ("migrations/baseline.sql", "CREATE TABLE public.partial_effect (id int); DO $$ BEGIN RAISE EXCEPTION 'SENTINEL-private-database-details' USING ERRCODE='42501'; END $$;", Error::DatabaseMigration { unit: 1, stage: MigrationStage::Sql, code: "42501".into() }),
    ] {
        let db = Database::new();
        let mut c = db.client();
        let mut r = verified();
        let s = installation(&r);
        r.files.insert(path.into(), sql.as_bytes().to_vec());
        r.manifest.files.get_mut(path).unwrap().sha256 = hash(sql.as_bytes());
        let error = migration::apply(&mut c, &s, &r).unwrap_err();
        assert_eq!(error, expected);
        assert!(!error.to_string().contains("SENTINEL"));
        assert!(!serde_json::to_string(&error).unwrap().contains("SENTINEL"));
        let count: i64 = c
            .query_one(
                "SELECT count(*) FROM pg_tables WHERE schemaname='public'",
                &[],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
        let count: i64 = c
            .query_one("SELECT count(*) FROM villow_setup.migrations", &[])
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
    }
}
#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn nonempty_legacy_database_is_never_adopted_or_reset() {
    let db = Database::new();
    let mut c = db.client();
    c.batch_execute(
        "CREATE TABLE public.existing (value text); INSERT INTO public.existing VALUES('retain');",
    )
    .unwrap();
    let r = verified();
    assert_eq!(
        migration::apply(&mut c, &installation(&r), &r),
        Err(Error::DatabaseNotEmpty {
            relations: 1,
            routines: 0
        })
    );
    assert_eq!(
        c.query_one("SELECT value FROM public.existing", &[])
            .unwrap()
            .get::<_, String>(0),
        "retain"
    );
}

#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn incomplete_history_stops_without_adoption() {
    let db = Database::new();
    let mut c = db.client();
    c.batch_execute("CREATE SCHEMA villow_setup;").unwrap();
    let r = verified();
    assert_eq!(
        migration::apply(&mut c, &installation(&r), &r),
        Err(Error::DatabaseHistoryIncomplete)
    );
    let ledger: Option<String> = c
        .query_one("SELECT to_regclass('villow_setup.migrations')::text", &[])
        .unwrap()
        .get(0);
    assert!(ledger.is_none());
}

fn seed_unapplied_history(c: &mut Client, s: &villow_setup::model::Installation) {
    c.batch_execute("CREATE SCHEMA villow_setup; CREATE TABLE villow_setup.instance (singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton), installation_id text NOT NULL, release_digest text NOT NULL); CREATE TABLE villow_setup.migrations (id text PRIMARY KEY, checksum text NOT NULL, postcondition_checksum text NOT NULL, applied_at timestamptz NOT NULL DEFAULT now());").unwrap();
    c.execute(
        "INSERT INTO villow_setup.instance VALUES (true,$1,$2)",
        &[&s.id, &s.release_digest],
    )
    .unwrap();
}

#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn fresh_retry_database_compare_and_swap_is_locked_and_idempotent() {
    let db = Database::new();
    let mut c = db.client();
    let (old, new, mut s) = fresh_retry_fixture();
    s.fresh_retry = Some(villow_setup::model::FreshRetryIntent {
        from: old.digest.clone(),
        to: new.digest.clone(),
    });
    seed_unapplied_history(&mut c, &s);
    let mut second = db.client();
    c.query_one("SELECT pg_advisory_lock($1)", &[&0x56494c4c4f57i64])
        .unwrap();
    assert_eq!(
        migration::retarget_unapplied(&mut second, &s, &old, &new),
        Err(Error::Busy)
    );
    c.query_one("SELECT pg_advisory_unlock($1)", &[&0x56494c4c4f57i64])
        .unwrap();
    migration::retarget_unapplied(&mut c, &s, &old, &new).unwrap();
    drop(c);
    let mut c = db.client();
    migration::retarget_unapplied(&mut c, &s, &old, &new).unwrap();
    assert_eq!(
        c.query_one("SELECT release_digest FROM villow_setup.instance", &[])
            .unwrap()
            .get::<_, String>(0),
        new.digest
    );
    let mut updated = s.clone();
    updated.release_digest = new.digest.clone();
    migration::apply(&mut c, &updated, &new).unwrap();
    assert_eq!(
        migration::retarget_unapplied(&mut c, &s, &old, &new),
        Err(Error::FreshRetryRefused)
    );
    assert_eq!(
        c.query_one("SELECT count(*) FROM villow_setup.migrations", &[])
            .unwrap()
            .get::<_, i64>(0),
        1
    );
}

#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn fresh_retry_never_rebinds_unowned_nonempty_or_committed_database() {
    for changed in [
        "owner",
        "release",
        "unit",
        "table",
        "function",
        "missing_history",
    ] {
        let db = Database::new();
        let mut c = db.client();
        let (old, new, mut s) = fresh_retry_fixture();
        s.fresh_retry = Some(villow_setup::model::FreshRetryIntent {
            from: old.digest.clone(),
            to: new.digest.clone(),
        });
        seed_unapplied_history(&mut c, &s);
        c.batch_execute(match changed {
            "owner" => "UPDATE villow_setup.instance SET installation_id='another-installation'",
            "release" => "UPDATE villow_setup.instance SET release_digest='another-release'",
            "unit" => "INSERT INTO villow_setup.migrations VALUES ('existing','checksum','postcondition',now())",
            "table" => "CREATE TABLE public.keep_me (value text); INSERT INTO public.keep_me VALUES ('retain')",
            "function" => "CREATE FUNCTION public.keep_me() RETURNS int LANGUAGE sql AS 'SELECT 1'",
            _ => "DROP TABLE villow_setup.migrations",
        }).unwrap();
        let before: String = c
            .query_one("SELECT release_digest FROM villow_setup.instance", &[])
            .unwrap()
            .get(0);
        assert_eq!(
            migration::retarget_unapplied(&mut c, &s, &old, &new),
            Err(Error::FreshRetryRefused)
        );
        assert_eq!(
            c.query_one("SELECT release_digest FROM villow_setup.instance", &[])
                .unwrap()
                .get::<_, String>(0),
            before
        );
        if changed == "table" {
            assert_eq!(
                c.query_one("SELECT value FROM public.keep_me", &[])
                    .unwrap()
                    .get::<_, String>(0),
                "retain"
            );
        }
    }
}

#[test]
#[ignore = "requires explicitly authorized local disposable Postgres"]
fn real_schema_permissions_functions_constraints_and_reference_data() {
    let db = Database::new();
    let mut c = db.client();
    let r = verified();
    let s = installation(&r);
    migration::apply(&mut c, &s, &r).unwrap();
    assert_eq!(
        c.query_one("SELECT value FROM public.synthetic WHERE id=0", &[])
            .unwrap()
            .get::<_, String>(0),
        "reference"
    );
    assert!(c
        .execute("INSERT INTO public.synthetic VALUES(0,'duplicate')", &[])
        .is_err());
    assert!(c
        .execute("INSERT INTO public.synthetic VALUES(5,NULL)", &[])
        .is_err());
    c.execute(
        "INSERT INTO public.synthetic VALUES(1,'private'),(2,'visible')",
        &[],
    )
    .unwrap();
    c.batch_execute("SET ROLE authenticated").unwrap();
    let rows = c
        .query("SELECT id FROM public.synthetic ORDER BY id", &[])
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, i32>(0), 2);
    assert_eq!(
        c.query_one("SELECT public.synthetic_label('works')", &[])
            .unwrap()
            .get::<_, String>(0),
        "WORKS"
    );
    assert!(c
        .execute("INSERT INTO public.synthetic VALUES(3,'forbidden')", &[])
        .is_err());
    c.batch_execute("RESET ROLE; SET ROLE anon").unwrap();
    assert!(c.query("SELECT * FROM public.synthetic", &[]).is_err());
    c.batch_execute("RESET ROLE").unwrap();
    assert_eq!(
        c.query_one("SELECT count(*) FROM public.synthetic", &[])
            .unwrap()
            .get::<_, i64>(0),
        3
    );
}
