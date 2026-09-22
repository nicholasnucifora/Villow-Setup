use crate::{
    error::{Error, MigrationStage, Result},
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
pub fn connect(s: &Installation, vault: &dyn Vault, c: &DbConnection) -> Result<Client> {
    let reference = &s.database()?.id;
    validate_connection(reference, c)?;
    let password = vault.require(&s.id, "db_password")?;
    let tls = database_tls()?;
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
        .map_err(|e| connection_error(&e))
}

fn database_tls() -> Result<native_tls::TlsConnector> {
    // Public CA used by Supabase's dashboard, scoped to this connector only.
    // Native TLS still verifies both the chain and the requested hostname.
    let root =
        native_tls::Certificate::from_pem(include_bytes!("../certs/supabase-prod-ca-2021.crt"))
            .map_err(|_| Error::DatabaseTls)?;
    native_tls::TlsConnector::builder()
        .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
        .add_root_certificate(root)
        .build()
        .map_err(|_| Error::DatabaseTls)
}

fn connection_error(error: &(dyn std::error::Error + 'static)) -> Error {
    // Classify typed causes only. Provider messages can contain credentials,
    // SQL and connection strings and must never reach the UI or checkpoint.
    let mut current = Some(error);
    while let Some(cause) = current {
        if let Some(db) = cause.downcast_ref::<postgres::error::DbError>() {
            return match db.code().code() {
                "28P01" | "28000" => Error::DatabaseAccess,
                "53300" | "57P03" => Error::DatabaseUnavailable,
                _ => Error::DatabaseConnect,
            };
        }
        if cause.is::<native_tls::Error>() {
            return Error::DatabaseTls;
        }
        if cause.is::<std::io::Error>() {
            return Error::DatabaseNetwork;
        }
        current = cause.source();
    }
    Error::DatabaseConnect
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn diagnostic_sqlstate_is_bounded_and_never_accepts_error_text() {
        assert_eq!(safe_sqlstate(Some("42501")), "42501");
        for code in [
            None,
            Some("SENTINEL-private-details"),
            Some("42\n01"),
            Some("é425"),
            Some("abcde"),
        ] {
            assert_eq!(safe_sqlstate(code), "unavailable");
        }
    }

    #[test]
    fn tls_required_rejects_a_server_that_only_offers_plaintext() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut ssl_request = [0; 8];
            socket.read_exact(&mut ssl_request).unwrap();
            assert_eq!(ssl_request, [0, 0, 0, 8, 4, 210, 22, 47]);
            socket.write_all(b"N").unwrap();
            let mut next = [0; 1];
            assert_eq!(
                socket.read(&mut next).unwrap(),
                0,
                "must not send a plaintext login after TLS refusal"
            );
        });
        let result = postgres::Config::new()
            .host("127.0.0.1")
            .port(port)
            .user("synthetic")
            .password("SENTINEL-NEVER-SEND")
            .ssl_mode(SslMode::Require)
            .connect(MakeTlsConnector::new(database_tls().unwrap()));
        assert!(result.is_err());
        server.join().unwrap();
    }

    #[test]
    fn database_authentication_errors_are_classified_without_server_text() {
        // Local protocol fixture supplies an auth refusal, without TLS or real credentials.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut length = [0; 4];
            socket.read_exact(&mut length).unwrap();
            let size = u32::from_be_bytes(length) as usize;
            assert!((8..4096).contains(&size));
            socket.read_exact(&mut vec![0; size - 4]).unwrap();
            let fields = b"SFATAL\0C28P01\0MSENTINEL-private-provider-message\0\0";
            socket.write_all(b"E").unwrap();
            socket
                .write_all(&((fields.len() + 4) as u32).to_be_bytes())
                .unwrap();
            socket.write_all(fields).unwrap();
        });
        let error = match postgres::Config::new()
            .host("127.0.0.1")
            .port(port)
            .user("synthetic")
            .ssl_mode(SslMode::Disable)
            .connect(postgres::NoTls)
        {
            Ok(_) => panic!("fixture should reject login"),
            Err(e) => e,
        };
        assert_eq!(connection_error(&error), Error::DatabaseAccess);
        assert!(!connection_error(&error).to_string().contains("SENTINEL"));
        server.join().unwrap();
    }
    #[test]
    fn supabase_root_loads_and_connection_errors_never_echo_details() {
        database_tls().unwrap();
        let certificate =
            native_tls::Certificate::from_pem(include_bytes!("../certs/supabase-prod-ca-2021.crt"))
                .unwrap();
        assert_eq!(
            crate::release::hash(&certificate.to_der().unwrap()),
            "807025ad50d4ed219d2c9c7d299c004f824eb00cf7f65afef607d07b72e6cafa"
        );
        let error = std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "SENTINEL-secret@private-host",
        );
        let classified = connection_error(&error);
        assert_eq!(classified, Error::DatabaseNetwork);
        assert!(!classified.to_string().contains("SENTINEL"));
        let tls_error = match native_tls::Certificate::from_pem(b"not a certificate") {
            Ok(_) => panic!("invalid certificate accepted"),
            Err(e) => e,
        };
        assert_eq!(connection_error(&tls_error), Error::DatabaseTls);
    }
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
        let objects = client.query_one("SELECT (SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p','v','m','S','f')), (SELECT count(*) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND NOT EXISTS (SELECT 1 FROM pg_depend d WHERE d.objid=p.oid AND d.deptype='e'))",&[]).map_err(|_| Error::Database)?;
        let relations: i64 = objects.get(0);
        let routines: i64 = objects.get(1);
        if relations != 0 || routines != 0 {
            return Err(Error::DatabaseNotEmpty {
                relations,
                routines,
            });
        }
        let existing_schema: bool = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname='villow_setup')",
                &[],
            )
            .map_err(|_| Error::Database)?
            .get(0);
        if existing_schema {
            return Err(Error::DatabaseHistoryIncomplete);
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
    let owner = client.query_one("SELECT installation_id, release_digest FROM villow_setup.instance WHERE singleton=TRUE",&[]).map_err(|_| Error::DatabaseHistoryIncomplete)?;
    if owner.get::<_, String>(0) != s.id || owner.get::<_, String>(1) != release.digest {
        return Err(Error::WrongTarget);
    }
    let rows = client.query("SELECT id,checksum,postcondition_checksum FROM villow_setup.migrations ORDER BY applied_at,id",&[]).map_err(|_| Error::DatabaseHistoryIncomplete)?;
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
    for (index, migration) in plan.iter().enumerate().skip(rows.len()) {
        let unit = index + 1;
        if !migration.transactional {
            return Err(Error::Unsupported);
        }
        let mut tx = client.transaction().map_err(|_| Error::Database)?;
        tx.batch_execute(text(release, &migration.file)?)
            .map_err(|e| migration_error(unit, MigrationStage::Sql, &e))?;
        let r = tx
            .query(text(release, &migration.postcondition)?, &[])
            .map_err(|e| migration_error(unit, MigrationStage::Verification, &e))?;
        if r.len() != 1 || r[0].len() != 1 || r[0].try_get::<_, bool>(0).ok() != Some(true) {
            return Err(Error::DatabasePostcondition { unit });
        }
        tx.execute("INSERT INTO villow_setup.migrations(id,checksum,postcondition_checksum) VALUES($1,$2,$3)",
            &[&migration.id,&release.manifest.files[&migration.file].sha256,&release.manifest.files[&migration.postcondition].sha256])
            .map_err(|e| migration_error(unit, MigrationStage::History, &e))?;
        tx.commit().map_err(|_| Error::Uncertain)?;
    }
    Ok(())
}
fn migration_error(unit: usize, stage: MigrationStage, error: &postgres::Error) -> Error {
    // A bounded SQLSTATE and local plan ordinal identify the failure without
    // exposing server messages, SQL, object names, credentials or release text.
    Error::DatabaseMigration {
        unit,
        stage,
        code: safe_sqlstate(error.code().map(|code| code.code())),
    }
}
fn safe_sqlstate(code: Option<&str>) -> String {
    code.filter(|code| {
        code.len() == 5
            && code
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    })
    .unwrap_or("unavailable")
    .to_owned()
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
