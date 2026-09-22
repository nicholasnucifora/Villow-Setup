mod common;
use common::*;
use villow_setup::{
    engine::Engine, error::Error, fresh_retry, model::*, release::validate_manifest, store::Store,
    vault::Vault,
};

fn vault_for(s: &Installation) -> MemoryVault {
    let vault = MemoryVault::default();
    for name in [
        "encryption_key",
        "bootstrap_token",
        "db_password",
        "google_secret",
        "vercel_token",
        "supabase_token",
    ] {
        vault.put(&s.id, name, &format!("SENTINEL-{name}")).unwrap();
    }
    vault
}

#[test]
fn uncertain_database_commit_keeps_intent_and_reopens_with_same_credentials() {
    let (old, new, s) = fresh_retry_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    let vault = vault_for(&s);
    let before = vault.values.borrow().clone();
    let remote = std::cell::RefCell::new(old.digest.clone());
    let result = fresh_retry::execute(&store, &vault, s.clone(), &old, &new, |pending| {
        assert_eq!(store.load()?.unwrap().fresh_retry, pending.fresh_retry);
        *remote.borrow_mut() = new.digest.clone();
        Err(Error::Uncertain)
    });
    assert_eq!(result, Err(Error::Uncertain));
    let pending = store.load().unwrap().unwrap();
    assert_eq!(pending.release_digest, old.digest);
    assert_eq!(pending.fresh_retry.as_ref().unwrap().to, new.digest);
    let fake = Fake::new(dir.path().join("cloud.json"));
    assert_eq!(
        Engine {
            store: &store,
            vault: &vault,
            providers: &fake
        }
        .advance(&old)
        .err(),
        Some(Error::FreshRetryPending)
    );
    drop(store);
    let store = Store::open(dir.path()).unwrap();
    fresh_retry::execute(&store, &vault, pending, &old, &new, |_| {
        assert_eq!(*remote.borrow(), new.digest);
        Ok(())
    })
    .unwrap();
    let saved = store.load().unwrap().unwrap();
    assert_eq!(saved.release_digest, new.digest);
    assert!(saved.fresh_retry.is_none());
    assert_eq!(saved.id, s.id);
    assert_eq!(saved.operation_id, s.operation_id);
    assert_eq!(saved.database.unwrap().id, s.database.unwrap().id);
    assert_eq!(saved.origin, s.origin);
    assert_eq!(*vault.values.borrow(), before);
}

#[test]
fn fresh_retry_rejects_unrelated_releases_finished_steps_and_changed_setup_contracts() {
    let (old, new, s) = fresh_retry_fixture();
    fresh_retry::validate(&s, &old, &new).unwrap();
    let mut variants = vec![];
    let mut x = s.clone();
    x.read_only = true;
    variants.push(x);
    let mut x = s.clone();
    x.credentials_removed = true;
    variants.push(x);
    let mut x = s.clone();
    x.step = Step::Complete;
    variants.push(x);
    let mut x = s.clone();
    x.effects.get_mut("migrate").unwrap().status = EffectStatus::Verified;
    variants.push(x);
    let mut x = s.clone();
    x.effects
        .insert("configure".into(), x.effects["migrate"].clone());
    variants.push(x);
    let mut x = s.clone();
    x.fresh_retry = Some(FreshRetryIntent {
        from: old.digest.clone(),
        to: "f".repeat(64),
    });
    variants.push(x);
    for x in variants {
        assert!(fresh_retry::validate(&x, &old, &new).is_err());
    }
    let mut x = new.clone();
    x.manifest.fresh_retry_from.clear();
    assert!(fresh_retry::validate(&s, &old, &x).is_err());
    let mut x = new.clone();
    x.manifest.google_scopes.push("new-scope".into());
    assert!(fresh_retry::validate(&s, &old, &x).is_err());
    let mut x = new.clone();
    x.manifest.bootstrap_contract = 2;
    assert!(fresh_retry::validate(&s, &old, &x).is_err());
    assert!(fresh_retry::validate(&s, &old, &old).is_err());
}

#[test]
fn missing_vault_stops_before_intent_or_remote_effect_and_metadata_is_bounded() {
    let (old, new, s) = fresh_retry_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    assert_eq!(
        fresh_retry::execute(&store, &MemoryVault::default(), s, &old, &new, |_| panic!(
            "no remote effect"
        )),
        Err(Error::MissingCredential)
    );
    assert!(store.load().unwrap().unwrap().fresh_retry.is_none());
    let (trust, _, _, _) = fixtures();
    for bad in [
        vec!["invalid".into()],
        vec![old.digest.clone(); 2],
        (0..9).map(|i| format!("{i:064x}")).collect(),
    ] {
        let mut m = new.manifest.clone();
        m.fresh_retry_from = bad;
        assert!(validate_manifest(&m, &trust).is_err());
    }
    let legacy = serde_json::to_value(old.manifest).unwrap();
    assert!(legacy.get("fresh_retry_from").is_none());
}
