mod common;
use common::*;
use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
};
use villow_setup::{
    error::Error,
    http::*,
    model::*,
    providers::validate_health,
    recovery,
    release::*,
    store::Store,
    vault::{self, Vault},
};

#[test]
fn accepts_authenticated_fixture() {
    assert_eq!(verified().manifest.app_version, "1.0.0");
}
#[test]
fn signature_guard_rejects_structurally_valid_forgery() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let (t, index, _, _) = fixtures();
    let mut envelope: Envelope = serde_json::from_slice(&index).unwrap();
    let mut channel: Channel =
        serde_json::from_slice(&STANDARD.decode(&envelope.payload).unwrap()).unwrap();
    channel.sequence += 1;
    envelope.payload = STANDARD.encode(serde_json::to_vec(&channel).unwrap());
    assert!(verify_channel(
        &serde_json::to_vec(&envelope).unwrap(),
        &t,
        "2026-09-10T00:00:00Z".parse().unwrap()
    )
    .is_err());
}
#[test]
fn rejects_missing_forged_and_unknown_signatures() {
    let (t, index, _, _) = fixtures();
    for malformed in [
        b"{}".to_vec(),
        index
            .iter()
            .map(|b| if *b == b'A' { b'B' } else { *b })
            .collect(),
        b"{\"key_id\":\"wrong\",\"payload\":\"AA==\",\"signature\":\"AA==\"}".to_vec(),
    ] {
        assert!(verify_channel(&malformed, &t, "2026-09-10T00:00:00Z".parse().unwrap()).is_err())
    }
}
#[test]
fn rejects_expired_future_and_replayed_channel() {
    let (t, index, _, _) = fixtures();
    let (c, _) = verify_channel(&index, &t, "2026-09-10T00:00:00Z".parse().unwrap()).unwrap();
    assert!(verify_channel(&index, &t, "2026-09-13T00:00:00Z".parse().unwrap()).is_err());
    assert!(verify_channel(&index, &t, "2026-09-01T00:00:00Z".parse().unwrap()).is_err());
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    store.accept_release(4, "first").unwrap();
    assert_eq!(store.accept_release(3, "old"), Err(Error::Release));
    assert_eq!(store.accept_release(4, "substitution"), Err(Error::Release));
    store.forget().unwrap();
    assert_eq!(store.accept_release(3, "old"), Err(Error::Release));
    let mut revoked = c.clone();
    revoked.revoked.push(c.releases[0].sha256.clone());
    let (_, _, m, a) = fixtures();
    assert!(verify_bundle(&m, &a, &c.releases[0], &revoked, &t).is_err());
}
#[test]
fn tampering_manifest_archive_or_sql_never_authenticates() {
    let (t, index, mut m, mut a) = fixtures();
    let (c, _) = verify_channel(&index, &t, "2026-09-10T00:00:00Z".parse().unwrap()).unwrap();
    m[10] ^= 1;
    assert!(verify_bundle(&m, &a, &c.releases[0], &c, &t).is_err());
    m[10] ^= 1;
    a[40] ^= 1;
    assert!(verify_bundle(&m, &a, &c.releases[0], &c, &t).is_err());
    let r = verified();
    let mut files = r.manifest.files.clone();
    files.get_mut("migrations/baseline.sql").unwrap().sha256 = "0".repeat(64);
    let (_, _, _, archive) = fixtures();
    assert!(read_archive(&archive, &files).is_err());
}
#[test]
fn paths_and_unexpected_executables_are_rejected() {
    for path in [
        "../outside",
        "/absolute",
        "C:/x",
        "\\\\server\\share",
        "src/../x",
        "src\\x",
        "src/a.ts:stream",
        "src/CON.txt",
        "src/com1.ts",
        ".env.production",
        "villow-setup/src/a.ts",
        "node_modules/x.js",
        "bin/steal.exe",
        "src/x.",
        "src/x ",
        "src/%2e%2e/x",
    ] {
        assert!(validate_path(path).is_err(), "{path}")
    }
}
#[test]
fn archives_reject_symlinks_duplicate_case_and_size_bombs() {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    z.add_symlink(
        "src/a.ts",
        "../../outside",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    let bytes = z.finish().unwrap().into_inner();
    let specs = BTreeMap::from([(
        "src/a.ts".into(),
        FileSpec {
            sha256: hash(b"../../outside"),
            size: 13,
            role: "deploy".into(),
        },
    )]);
    assert!(read_archive(&bytes, &specs).is_err());
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut specs = BTreeMap::new();
    for p in ["src/A.ts", "src/a.ts"] {
        z.start_file(p, zip::write::SimpleFileOptions::default())
            .unwrap();
        z.write_all(b"a").unwrap();
        specs.insert(
            p.into(),
            FileSpec {
                sha256: hash(b"a"),
                size: 1,
                role: "deploy".into(),
            },
        );
    }
    assert!(read_archive(&z.finish().unwrap().into_inner(), &specs).is_err());
    let mut specs = verified().manifest.files;
    specs.get_mut("src/main.ts").unwrap().size = MAX_FILE + 1;
    let (_, _, _, a) = fixtures();
    assert!(read_archive(&a, &specs).is_err());
}
#[test]
fn rejects_nontransactional_duplicate_migrations_and_unsupported_upgrades() {
    let (t, _, _, _) = fixtures();
    let mut m = verified().manifest;
    m.schema.migrations[0].transactional = false;
    assert!(validate_manifest(&m, &t).is_err());
    m = verified().manifest;
    m.schema.migrations.push(m.schema.migrations[0].clone());
    assert!(validate_manifest(&m, &t).is_err());
    m = verified().manifest;
    m.upgrade_from.push("0.9.0".into());
    assert!(validate_manifest(&m, &t).is_err());
    m = verified().manifest;
    m.minimum_manager = "99.0.0".into();
    assert!(validate_manifest(&m, &t).is_err());
}
#[test]
fn credentials_cannot_follow_redirects_or_change_hosts() {
    for url in [
        "http://api.vercel.com/x",
        "https://api.vercel.com.evil.test/x",
        "https://api.vercel.com@evil.test/x",
        "https://api.vercel.com:444/x",
        "https://127.0.0.1/x",
    ] {
        assert!(validate_credential_url(&url.parse().unwrap(), "api.vercel.com").is_err())
    }
    for status in [301, 302, 303, 307, 308] {
        assert_eq!(check_status(status), Err(Error::WrongTarget))
    }
    assert_eq!(check_status(401), Err(Error::Authentication));
    assert_eq!(check_status(429), Err(Error::RateLimited));
    assert!(
        validate_release_destination(&"https://github.com.evil.test/steal".parse().unwrap())
            .is_err()
    );
}
#[test]
fn database_destinations_are_bound_to_project_and_tls_host() {
    let good = DbConnection {
        host: "db.abcdefghijklmnopqrst.supabase.co".into(),
        user: "postgres".into(),
    };
    villow_setup::migration::validate_connection("abcdefghijklmnopqrst", &good).unwrap();
    for c in [
        DbConnection {
            host: "db.unrelatedabcdefghijk.supabase.co".into(),
            ..good.clone()
        },
        DbConnection {
            host: "aws-0-ap-southeast-2.pooler.supabase.com.evil.test".into(),
            user: "postgres.abcdefghijklmnopqrst".into(),
        },
        DbConnection {
            host: "aws-0-ap-southeast-2.pooler.supabase.com".into(),
            user: "postgres.wrong".into(),
        },
    ] {
        assert!(villow_setup::migration::validate_connection("abcdefghijklmnopqrst", &c).is_err())
    }
}
#[test]
fn health_requires_authenticated_owner_and_real_app_details() {
    let r = verified();
    let mut s = installation(&r);
    s.origin = Some("https://test.vercel.app".into());
    assert_eq!(
        validate_health(&serde_json::json!({"status":"ok"}), &s, &r, "nonce"),
        Err(Error::Health)
    );
    let h = serde_json::json!({"format":1,"nonce":"nonce","installationId":s.id,"commit":r.manifest.commit,"appVersion":r.manifest.app_version,"schemaRevision":r.manifest.schema.revision,"ownerEmail":s.owner_email,"origin":s.origin,"intendedOwnerVerified":true,"configurationValid":true,"databaseProbePassed":true,"youtubeAuthorizationVerified":true,"cronConfigured":true,"bootstrapClosed":true});
    validate_health(&h, &s, &r, "nonce").unwrap();
    for k in [
        "intendedOwnerVerified",
        "configurationValid",
        "databaseProbePassed",
        "youtubeAuthorizationVerified",
        "cronConfigured",
        "bootstrapClosed",
    ] {
        let mut broken = h.clone();
        broken[k] = false.into();
        assert!(validate_health(&broken, &s, &r, "nonce").is_err())
    }
    assert!(validate_health(&h, &s, &r, "replayed").is_err());
}
#[test]
fn secret_variants_are_redacted_and_never_in_checkpoints_or_exports() {
    use base64::Engine;
    let secret = "SENTINEL+secret/with=encoding";
    let variants = [
        secret.to_string(),
        base64::engine::general_purpose::STANDARD.encode(secret),
        url::form_urlencoded::byte_serialize(secret.as_bytes()).collect::<String>(),
    ];
    let redacted = vault::redact(&variants.join(" "), &[secret]);
    for v in &variants {
        assert!(!redacted.contains(v))
    }
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let s = installation(&verified());
    let v = MemoryVault::default();
    v.put(&s.id, "vercel_token", secret).unwrap();
    store.save(&s).unwrap();
    let export = recovery::export(&s).unwrap();
    assert!(!export.contains(secret));
    let bytes = std::fs::read(d.path().join("state.sqlite3")).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains(secret));
    let mut forged: serde_json::Value = serde_json::from_str(&export).unwrap();
    forged["installation"]["read_only"] = false.into();
    forged["installation"]["checks"] =
        serde_json::json!([{"kind":"app","title":"Forged green check","at":"today"}]);
    let imported = recovery::import(&forged.to_string()).unwrap();
    assert!(imported.read_only);
    assert!(imported.checks.is_empty());
    assert_eq!(imported.assert_writable(), Err(Error::RecoveryReadOnly));
}
#[test]
fn missing_vault_never_falls_back_to_plaintext() {
    let v = MemoryVault::default();
    v.fail.set(true);
    assert_eq!(
        v.ensure_random("id", "encryption_key").unwrap_err(),
        Error::Vault
    );
    assert!(v.values.borrow().is_empty())
}
#[test]
fn unconfigured_build_is_fail_closed() {
    assert!(!Trust::embedded().unwrap().configured());
}
