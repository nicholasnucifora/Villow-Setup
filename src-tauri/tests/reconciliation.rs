mod common;
use common::*;
use serde_json::json;
use villow_setup::{error::Error, http::Provider, reconcile::validate_candidate};

#[test]
fn uncertain_creation_needs_exact_id_account_name_and_recent_timestamp() {
    let s = installation(&verified());
    let started = chrono::Utc::now();
    let v = json!({"id":"prj_expected","accountId":"team_1","name":s.name,"createdAt":started.timestamp_millis()});
    let validate = |candidate: &serde_json::Value| {
        validate_candidate(
            &s,
            Provider::Vercel,
            "prj_expected",
            candidate,
            &started.to_rfc3339(),
        )
    };
    assert_eq!(
        validate(&v).unwrap().evidence,
        "owner_confirmed_after_uncertain_creation"
    );
    for (field, value) in [
        ("id", json!("different")),
        ("accountId", json!("other-team")),
        ("name", json!("shared-project")),
        (
            "createdAt",
            json!((started - chrono::Duration::days(1)).timestamp_millis()),
        ),
        (
            "createdAt",
            json!((started + chrono::Duration::days(1)).timestamp_millis()),
        ),
    ] {
        let mut changed = v.clone();
        changed[field] = value;
        assert!(matches!(validate(&changed), Err(Error::WrongTarget)));
    }
}

#[test]
fn backup_and_incompatible_schema_contracts_are_refused_for_fresh_install() {
    let (trust, _, _, _) = fixtures();
    let mut r = verified();
    r.manifest.backup_required = true;
    assert!(villow_setup::release::validate_manifest(&r.manifest, &trust).is_err());
    r.manifest.backup_required = false;
    r.manifest.schema.compatible_apps = "<1.0.0".into();
    assert!(villow_setup::release::validate_manifest(&r.manifest, &trust).is_err());
}

#[test]
fn failed_upload_is_retryable_without_an_uncertain_deployment() {
    use villow_setup::{engine::Engine, model::Step, store::Store};
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let r = verified();
    let mut s = installation(&r);
    s.step = Step::Deployment;
    store.save(&s).unwrap();
    let vault = MemoryVault::default();
    let fake = Fake::new(d.path().join("remote.json"));
    fake.crash.set(true);
    let e = Engine {
        store: &store,
        vault: &vault,
        providers: &fake,
    };
    assert!(matches!(e.advance(&r), Err(Error::Offline)));
    let saved = store.load().unwrap().unwrap();
    assert!(!saved.effects.contains_key("deploy"));
    assert!(fake.read().deployment.is_none());
    assert_eq!(e.advance(&r).unwrap().step, Step::Deployment);
    assert_eq!(e.check_deployment(&r).unwrap().step, Step::Health);
}
