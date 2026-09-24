mod common;
use villow_setup::{app_update_engine, error::Error, model::Step, release, update_contract};
#[test]
fn only_exact_authenticated_sources_and_unchanged_requirements_can_update() {
    let (old, new, s) = common::update_fixture();
    let mut altered = new.clone();
    altered.files.insert(
        altered.manifest.app_updates[0].precondition.clone(),
        b"SELECT true".to_vec(),
    );
    assert!(update_contract::transition(&old, &altered).is_err());
    assert!(app_update_engine::validate(&s, &old, &new).is_ok());
    for i in 0..8 {
        let mut bad = new.clone();
        match i {
            0 => bad.manifest.app_updates[0].from_manifest_sha256 = "0".repeat(64),
            1 => bad.manifest.app_version = old.manifest.app_version.clone(),
            2 => bad.manifest.sequence = old.manifest.sequence,
            3 => bad.manifest.google_scopes.push("more-access".into()),
            4 => bad
                .manifest
                .configuration
                .insert("EXTRA_SECRET".into(), "secret".into())
                .map(|_| ())
                .unwrap_or(()),
            5 => bad.manifest.health_contract += 1,
            6 => bad.manifest.build_command = "other build".into(),
            _ => bad
                .files
                .get_mut(&old.manifest.schema.migrations[0].file)
                .unwrap()
                .push(b' '),
        };
        assert!(update_contract::transition(&old, &bad).is_err(), "case {i}");
    }
    let mut imported = s.clone();
    imported.read_only = true;
    assert_eq!(
        app_update_engine::validate(&imported, &old, &new),
        Err(Error::RecoveryReadOnly)
    );
    imported = s.clone();
    imported.step = Step::Health;
    assert!(app_update_engine::validate(&imported, &old, &new).is_err());
}
#[test]
fn malformed_or_unbounded_update_declarations_and_files_are_refused() {
    let (_, r, _) = common::update_fixture();
    let (trust, _, _, _) = common::fixtures();
    for i in 0..11 {
        let mut m = r.manifest.clone();
        match i {
            0 => m.upgrade_from.push(m.upgrade_from[0].clone()),
            1 => m.app_updates[0].backup_required = false,
            2 => m.app_updates[0].previous_app_compatible = false,
            3 => m.minimum_manager = "0.1.5".into(),
            4 => m.app_updates[0].kind = "transactional".into(),
            5 => m.app_updates[0].source_backup = "../source.json".into(),
            6 => m.app_updates[0].to_schema_revision = "other".into(),
            7 => m.app_updates[0].precondition = "public/data.sql".into(),
            8 => {
                m.files
                    .get_mut("updates/test-update/source.sql")
                    .unwrap()
                    .role = "deploy".into()
            }
            9 => m.app_updates.clear(),
            _ => m.backup_required = true,
        };
        assert!(release::validate_manifest(&m, &trust).is_err(), "case {i}");
    }
    let mut files = r.files.clone();
    let path = &r.manifest.app_updates[0].source_backup;
    let mut descriptor: serde_json::Value = serde_json::from_slice(&files[path]).unwrap();
    descriptor["native_ledger_contract"] = serde_json::json!(2);
    files.insert(path.clone(), serde_json::to_vec(&descriptor).unwrap());
    assert!(update_contract::validate_files(&r.manifest, &files).is_err());
    files = r.files.clone();
    files.insert(
        r.manifest.app_updates[0].precondition.clone(),
        b"COMMIT; SELECT true".to_vec(),
    );
    assert!(update_contract::validate_files(&r.manifest, &files).is_err());
    let mut value = serde_json::to_value(&r.manifest).unwrap();
    value["app_updates"][0]["unchecked_sql"] = serde_json::json!("bad");
    assert!(serde_json::from_value::<release::Manifest>(value).is_err());
}
