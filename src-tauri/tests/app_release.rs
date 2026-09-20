mod common;
use std::{collections::BTreeMap, fs, path::PathBuf};
use villow_setup::error::Error;
use villow_setup::release::{verify_bundle, verify_channel, Trust};

// Explicit local qualification input. Production trust remains unconfigured;
// this test-only key is supplied by the app's disposable release harness.
#[test]
#[ignore = "requires an app-owned disposable release fixture"]
fn authenticates_the_app_release_and_transactional_baseline() {
    assert_eq!(
        std::env::var("VILLOW_LOCAL_DB_TESTS").as_deref(),
        Ok("disposable-local-cluster"),
        "App release qualification requires the web harness's disposable local database; database checks cannot be skipped"
    );
    let dir = PathBuf::from(std::env::var("VILLOW_APP_RELEASE_TEST_DIR").unwrap());
    let public_key = fs::read_to_string(dir.join("test-public-key.txt")).unwrap();
    let trust = Trust {
        format: 1,
        repository: Some("test-owner/test-releases".into()),
        channel: "stable".into(),
        manifest_url: Some(
            "https://github.com/test-owner/test-releases/releases/download/channel/stable.json"
                .into(),
        ),
        public_keys: BTreeMap::from([("test-only".into(), public_key.trim().into())]),
        minimum_sequence: 1,
        publisher: Some("Synthetic local test publisher".into()),
    };
    let (channel, _) = verify_channel(
        &fs::read(dir.join("channel.json")).unwrap(),
        &trust,
        chrono::Utc::now(),
    )
    .unwrap();
    let release = verify_bundle(
        &fs::read(dir.join("manifest.json")).unwrap(),
        &fs::read(dir.join("villow-source.zip")).unwrap(),
        &channel.releases[0],
        &channel,
        &trust,
    )
    .unwrap();
    {
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
                .dbname("manager_check")
                .connect(postgres::NoTls)
                .unwrap()
        };
        let mut client = connect();
        let state = common::installation(&release);
        villow_setup::migration::apply(&mut client, &state, &release).unwrap();
        drop(client);
        // A new session models reopening after a completed SQL commit. It does
        // not claim interruption inside a transaction or a lost COMMIT reply.
        let mut client = connect();
        villow_setup::migration::apply(&mut client, &state, &release).unwrap();
        assert!(client
            .query_one("SELECT villow_setup_probe()", &[])
            .unwrap()
            .get::<_, bool>(0));
        assert_eq!(
            client
                .query_one("SELECT count(*) FROM villow_setup.migrations", &[])
                .unwrap()
                .get::<_, i64>(0),
            1
        );
        assert_eq!(
            client
                .query_one("SELECT count(*) FROM users", &[])
                .unwrap()
                .get::<_, i64>(0),
            0
        );
        let owner = client.query_one("SELECT installation_id,release_digest FROM villow_setup.instance WHERE singleton=TRUE", &[]).unwrap();
        assert_eq!(owner.get::<_, String>(0), state.id);
        assert_eq!(owner.get::<_, String>(1), release.digest);
        let rows = client.query("SELECT id,checksum,postcondition_checksum FROM villow_setup.migrations ORDER BY applied_at,id", &[]).unwrap();
        assert_eq!(rows.len(), release.manifest.schema.migrations.len());
        for (row, migration) in rows.iter().zip(&release.manifest.schema.migrations) {
            assert_eq!(row.get::<_, String>(0), migration.id);
            assert_eq!(
                row.get::<_, String>(1),
                release.manifest.files[&migration.file].sha256
            );
            assert_eq!(
                row.get::<_, String>(2),
                release.manifest.files[&migration.postcondition].sha256
            );
        }
        client
            .batch_execute("BEGIN; UPDATE villow_setup.migrations SET checksum='tampered'")
            .unwrap();
        assert_eq!(
            villow_setup::migration::apply(&mut client, &state, &release),
            Err(Error::SchemaDrift)
        );
        client.batch_execute("ROLLBACK").unwrap();
        client
            .batch_execute("ALTER TABLE users DROP COLUMN session_token")
            .unwrap();
        assert_eq!(
            villow_setup::migration::apply(&mut client, &state, &release),
            Err(Error::SchemaDrift)
        );
    }
    assert_eq!(release.manifest.schema.revision, "villow-fresh-158");
    assert!(release.manifest.configuration.contains_key("CRON_SECRET"));
    assert!(release.files.contains_key("lib/api/setup.ts"));
    assert!(release
        .files
        .keys()
        .all(|p| !p.starts_with("villow-setup/") && !p.starts_with("node_modules/")));
    for migration in &release.manifest.schema.migrations {
        villow_setup::sql_guard::validate_transactional(
            std::str::from_utf8(&release.files[&migration.file]).unwrap(),
        )
        .unwrap();
    }
}
