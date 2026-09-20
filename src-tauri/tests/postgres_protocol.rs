//! Opt-in local disposable PostgreSQL tests. Never accepts a cloud connection URL.
mod common;
use common::*;
use postgres::{Client, NoTls};
use villow_setup::{error::Error, migration, release::hash};

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
    for failing_sql in [true, false] {
        let db = Database::new();
        let mut c = db.client();
        let mut r = verified();
        let s = installation(&r);
        let path = if failing_sql {
            "migrations/baseline.sql"
        } else {
            "migrations/verify.sql"
        };
        let sql = if failing_sql {
            "CREATE TABLE public.partial_effect (id int); SELECT 1/0;"
        } else {
            "SELECT false;"
        };
        r.files.insert(path.into(), sql.as_bytes().to_vec());
        r.manifest.files.get_mut(path).unwrap().sha256 = hash(sql.as_bytes());
        assert_eq!(migration::apply(&mut c, &s, &r), Err(Error::SchemaDrift));
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
        Err(Error::SchemaDrift)
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
