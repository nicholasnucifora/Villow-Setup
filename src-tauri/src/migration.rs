use crate::{
    error::{Error, Result},
    model::{DbConnection, Installation},
    release::VerifiedRelease,
    vault::Vault,
};
use postgres::{config::SslMode, Client};
use postgres_native_tls::MakeTlsConnector;
use std::time::Duration;

const LOCK_ID: i64 = 0x56494c4c4f57;
pub fn validate_connection(reference: &str, connection: &DbConnection) -> Result<()> {
    if reference.len() != 20 || !reference.bytes().all(|b| b.is_ascii_lowercase()) {
        return Err(Error::WrongTarget);
    }
    let direct =
        connection.host == format!("db.{reference}.supabase.co") && connection.user == "postgres";
    let session = connection.host.ends_with(".pooler.supabase.com")
        && connection.host.starts_with("aws-")
        && connection.host.split('.').count() == 4
        && connection.user == format!("postgres.{reference}");
    if (!direct && !session)
        || !connection
            .host
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b".-".contains(&b))
    {
        return Err(Error::WrongTarget);
    }
    Ok(())
}
pub fn connect(s: &Installation, vault: &dyn Vault) -> Result<Client> {
    let reference = &s.database()?.id;
    let fallback = DbConnection {
        host: format!("db.{reference}.supabase.co"),
        user: "postgres".into(),
    };
    let c = s.db_connection.as_ref().unwrap_or(&fallback);
    validate_connection(reference, c)?;
    let password = vault.require(&s.id, "db_password")?;
    let tls = native_tls::TlsConnector::builder()
        .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
        .build()
        .map_err(|_| Error::Database)?;
    let mut config = postgres::Config::new();
    config
        .host(&c.host)
        .port(5432)
        .user(&c.user)
        .password(password.as_str())
        .dbname("postgres")
        .ssl_mode(SslMode::Require)
        .connect_timeout(Duration::from_secs(15))
        .application_name("VillowSetup");
    config
        .connect(MakeTlsConnector::new(tls))
        .map_err(|_| Error::Database)
}

// A single TLS session owns the lock. Each signed unit, its postcondition and
// history row commit atomically. SQL is never split on semicolons.
pub fn apply(client: &mut Client, s: &Installation, release: &VerifiedRelease) -> Result<()> {
    client.batch_execute("SET standard_conforming_strings = on; SET lock_timeout = '10s'; SET statement_timeout = '120s';").map_err(|_| Error::Database)?;
    let locked: bool = client
        .query_one("SELECT pg_try_advisory_lock($1)", &[&LOCK_ID])
        .map_err(|_| Error::Database)?
        .get(0);
    if !locked {
        return Err(Error::Busy);
    }
    let result = apply_locked(client, s, release);
    // The connection is dropped by the caller even if unlock fails.
    let _ = client.execute("SELECT pg_advisory_unlock($1)", &[&LOCK_ID]);
    result
}
fn apply_locked(client: &mut Client, s: &Installation, release: &VerifiedRelease) -> Result<()> {
    let ledger: Option<String> = client
        .query_one("SELECT to_regclass('villow_setup.migrations')::text", &[])
        .map_err(|_| Error::Database)?
        .get(0);
    if ledger.is_none() {
        let objects: i64 = client.query_one("SELECT (SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p','v','m','S','f')) + (SELECT count(*) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND NOT EXISTS (SELECT 1 FROM pg_depend d WHERE d.objid=p.oid AND d.deptype='e'))",&[]).map_err(|_| Error::Database)?.get(0);
        if objects != 0 {
            return Err(Error::SchemaDrift);
        }
        let existing_schema: bool = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname='villow_setup')",
                &[],
            )
            .map_err(|_| Error::Database)?
            .get(0);
        if existing_schema {
            return Err(Error::SchemaDrift);
        }
        let mut tx = client.transaction().map_err(|_| Error::Database)?;
        tx.batch_execute("CREATE SCHEMA villow_setup; REVOKE ALL ON SCHEMA villow_setup FROM PUBLIC;
            CREATE TABLE villow_setup.instance (singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK(singleton), installation_id TEXT NOT NULL, release_digest TEXT NOT NULL);
            CREATE TABLE villow_setup.migrations (id TEXT PRIMARY KEY, checksum TEXT NOT NULL, postcondition_checksum TEXT NOT NULL, applied_at TIMESTAMPTZ NOT NULL DEFAULT now());
            REVOKE ALL ON ALL TABLES IN SCHEMA villow_setup FROM PUBLIC, anon, authenticated;")
            .map_err(|_| Error::Database)?;
        tx.execute("INSERT INTO villow_setup.instance(singleton,installation_id,release_digest) VALUES(TRUE,$1,$2)",&[&s.id,&release.digest]).map_err(|_| Error::Database)?;
        tx.commit().map_err(|_| Error::Uncertain)?;
    }
    let owner = client.query_one("SELECT installation_id, release_digest FROM villow_setup.instance WHERE singleton=TRUE",&[]).map_err(|_| Error::SchemaDrift)?;
    if owner.get::<_, String>(0) != s.id || owner.get::<_, String>(1) != release.digest {
        return Err(Error::WrongTarget);
    }
    let rows = client.query("SELECT id,checksum,postcondition_checksum FROM villow_setup.migrations ORDER BY applied_at,id",&[]).map_err(|_| Error::SchemaDrift)?;
    let plan = &release.manifest.schema.migrations;
    if rows.len() > plan.len() {
        return Err(Error::SchemaDrift);
    }
    for (i, row) in rows.iter().enumerate() {
        let expected = &plan[i];
        if row.get::<_, String>(0) != expected.id
            || row.get::<_, String>(1) != release.manifest.files[&expected.file].sha256
            || row.get::<_, String>(2) != release.manifest.files[&expected.postcondition].sha256
        {
            return Err(Error::SchemaDrift);
        }
        check_postcondition(client, text(release, &expected.postcondition)?)?;
    }
    for migration in plan.iter().skip(rows.len()) {
        if !migration.transactional {
            return Err(Error::Unsupported);
        }
        let mut tx = client.transaction().map_err(|_| Error::Database)?;
        tx.batch_execute(text(release, &migration.file)?)
            .map_err(|_| Error::SchemaDrift)?;
        let r = tx
            .query(text(release, &migration.postcondition)?, &[])
            .map_err(|_| Error::SchemaDrift)?;
        if r.len() != 1 || r[0].len() != 1 || r[0].try_get::<_, bool>(0).ok() != Some(true) {
            return Err(Error::SchemaDrift);
        }
        tx.execute("INSERT INTO villow_setup.migrations(id,checksum,postcondition_checksum) VALUES($1,$2,$3)",
            &[&migration.id,&release.manifest.files[&migration.file].sha256,&release.manifest.files[&migration.postcondition].sha256])
            .map_err(|_| Error::SchemaDrift)?;
        tx.commit().map_err(|_| Error::Uncertain)?;
    }
    Ok(())
}
fn text<'a>(release: &'a VerifiedRelease, path: &str) -> Result<&'a str> {
    std::str::from_utf8(release.files.get(path).ok_or(Error::Release)?).map_err(|_| Error::Release)
}
fn check_postcondition(client: &mut Client, sql: &str) -> Result<()> {
    let mut tx = client
        .build_transaction()
        .read_only(true)
        .start()
        .map_err(|_| Error::Database)?;
    let r = tx.query(sql, &[]).map_err(|_| Error::SchemaDrift)?;
    if r.len() != 1 || r[0].len() != 1 || r[0].try_get::<_, bool>(0).ok() != Some(true) {
        return Err(Error::SchemaDrift);
    }
    tx.commit().map_err(|_| Error::Database)
}
