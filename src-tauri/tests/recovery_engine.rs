mod common;
use common::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use villow_setup::{
    engine::{forget, remove_credentials, Engine},
    error::Error,
    model::*,
    store::Store,
    vault::Vault,
};

#[test]
fn crash_after_remote_effect_reopens_without_duplicate_or_new_encryption_key() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(&d.path().join("local")).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    let s = installation(&r);
    store.save(&s).unwrap();
    let fake = Fake::new(d.path().join("cloud.json"));
    fake.crash.set(true);
    assert!(catch_unwind(AssertUnwindSafe(|| Engine {
        store: &store,
        vault: &vault,
        providers: &fake
    }
    .advance(&r)))
    .is_err());
    assert!(store.load().unwrap().unwrap().vercel.is_none());
    assert_eq!(fake.read().creates, 1);
    let key = vault.require(&s.id, "encryption_key").unwrap().to_string();
    drop(store);
    let reopened = Store::open(&d.path().join("local")).unwrap();
    let cloud = Fake::new(d.path().join("cloud.json"));
    let resumed = Engine {
        store: &reopened,
        vault: &vault,
        providers: &cloud,
    }
    .advance(&r)
    .unwrap();
    assert!(resumed.vercel.is_some());
    assert_eq!(cloud.read().creates, 1);
    assert_eq!(
        vault.require(&s.id, "encryption_key").unwrap().as_str(),
        key
    );
}
#[test]
fn crash_before_effect_does_not_guess_a_safe_retry() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    store.save(&installation(&r)).unwrap();
    let fake = Fake::new(d.path().join("remote.json"));
    fake.before.set(true);
    let e = Engine {
        store: &store,
        vault: &vault,
        providers: &fake,
    };
    assert!(catch_unwind(AssertUnwindSafe(|| e.advance(&r))).is_err());
    assert!(matches!(e.advance(&r), Err(Error::Uncertain)));
    assert_eq!(fake.read().creates, 0)
}
#[test]
fn lost_response_reconciles_persisted_operation_identity() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    store.save(&installation(&r)).unwrap();
    let fake = Fake::new(d.path().join("remote.json"));
    *fake.error.borrow_mut() = Some(Error::Uncertain);
    let e = Engine {
        store: &store,
        vault: &vault,
        providers: &fake,
    };
    assert!(matches!(e.advance(&r), Err(Error::Uncertain)));
    assert_eq!(
        store.load().unwrap().unwrap().effects["create_vercel"].status,
        EffectStatus::NeedsReview
    );
    e.advance(&r).unwrap();
    assert_eq!(fake.read().creates, 1)
}
#[test]
fn double_invocation_is_locked_before_effects() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    store.save(&installation(&r)).unwrap();
    let fake = Fake::new(d.path().join("remote.json"));
    let lock = store.lock().unwrap();
    assert!(matches!(
        Engine {
            store: &store,
            vault: &vault,
            providers: &fake
        }
        .advance(&r),
        Err(Error::Busy)
    ));
    assert_eq!(fake.read().creates, 0);
    drop(lock);
    assert!(store.lock().is_ok())
}
#[test]
fn expired_rate_limited_offline_wrong_account_and_missing_vault_make_no_changes() {
    for err in [
        Error::Authentication,
        Error::RateLimited,
        Error::Offline,
        Error::WrongTarget,
    ] {
        let d = tempfile::tempdir().unwrap();
        let store = Store::open(d.path()).unwrap();
        let vault = MemoryVault::default();
        let r = verified();
        store.save(&installation(&r)).unwrap();
        let fake = Fake::new(d.path().join("remote.json"));
        *fake.error.borrow_mut() = Some(err.clone());
        assert!(
            matches!(Engine{store:&store,vault:&vault,providers:&fake}.advance(&r),Err(x) if x==err)
        );
        assert_eq!(fake.read().creates, 0);
        assert!(store.load().unwrap().unwrap().effects.is_empty())
    }
}
#[test]
fn cannot_retarget_account_or_replay_another_release() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    let mut s = installation(&r);
    s.selection.as_mut().unwrap().vercel_account = "other_team".into();
    store.save(&s).unwrap();
    let fake = Fake::new(d.path().join("remote.json"));
    let e = Engine {
        store: &store,
        vault: &vault,
        providers: &fake,
    };
    assert!(matches!(e.advance(&r), Err(Error::WrongTarget)));
    s.release_digest = "wrong".into();
    store.save(&s).unwrap();
    assert!(matches!(e.advance(&r), Err(Error::Release)));
    assert_eq!(fake.read().creates, 0)
}
#[test]
fn missing_encryption_key_after_creation_is_not_regenerated() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    let s = installation(&r);
    store.save(&s).unwrap();
    let fake = Fake::new(d.path().join("remote.json"));
    let e = Engine {
        store: &store,
        vault: &vault,
        providers: &fake,
    };
    e.advance(&r).unwrap();
    vault.delete(&s.id, "encryption_key").unwrap();
    assert!(matches!(e.advance(&r), Err(Error::MissingCredential)));
    assert_eq!(fake.read().creates, 1)
}
#[test]
fn completed_migration_with_lost_response_is_reconciled_and_health_can_fail() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    let s = installation(&r);
    store.save(&s).unwrap();
    let fake = Fake::new(d.path().join("remote.json"));
    let e = Engine {
        store: &store,
        vault: &vault,
        providers: &fake,
    };
    for _ in 0..3 {
        e.advance(&r).unwrap();
    }
    let mut s = store.load().unwrap().unwrap();
    s.google = Some(Google {
        project_id: "project".into(),
        client_id: "id.apps.googleusercontent.com".into(),
        api_enabled_confirmed: true,
        audience: "external_production".into(),
        consent_published_confirmed: true,
    });
    store.save(&s).unwrap();
    vault.put(&s.id, "google_secret", "synthetic").unwrap();
    e.advance(&r).unwrap();
    fake.crash.set(true);
    assert!(matches!(e.advance(&r), Err(Error::Uncertain)));
    assert_eq!(fake.read().migrations, 1);
    e.advance(&r).unwrap();
    assert_eq!(fake.read().migrations, 1);
    e.advance(&r).unwrap();
    e.advance(&r).unwrap();
    fake.health_ok.set(false);
    assert!(matches!(e.advance(&r), Err(Error::Health)));
    assert_eq!(store.load().unwrap().unwrap().step, Step::Health);
    fake.health_ok.set(true);
    assert_eq!(e.advance(&r).unwrap().step, Step::Complete);
}
#[test]
fn removal_retains_progress_on_vault_failure_and_forget_cannot_delete_cloud() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let vault = MemoryVault::default();
    let r = verified();
    let s = installation(&r);
    store.save(&s).unwrap();
    vault
        .put(&s.id, "vercel_token", "SENTINEL-only-test")
        .unwrap();
    vault.fail.set(true);
    assert_eq!(remove_credentials(&store, &vault), Err(Error::Vault));
    assert!(!store.load().unwrap().unwrap().credentials_removed);
    vault.fail.set(false);
    assert_eq!(forget(&store, &vault, "wrong"), Err(Error::Invalid));
    remove_credentials(&store, &vault).unwrap();
    assert!(vault.values.borrow().is_empty());
    forget(&store, &vault, &s.name).unwrap();
    assert!(store.load().unwrap().is_none());
}
