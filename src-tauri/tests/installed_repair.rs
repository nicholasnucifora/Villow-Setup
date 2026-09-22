mod common;
use common::*;
use std::cell::Cell;
use villow_setup::{
    engine::{self, Engine},
    error::Error,
    installed_repair as repair,
    model::*,
    recovery,
    release::validate_manifest,
    store::Store,
};

#[test]
fn signed_repair_metadata_is_bounded_and_separate_from_fresh_retry() {
    let (old, new, s) = repair_fixture();
    repair::validate(&s, &old, &new).unwrap();
    let (trust, _, _, _) = fixtures();
    for kind in 0..8 {
        let mut m = new.manifest.clone();
        match kind {
            0 => m.installed_repairs.push(m.installed_repairs[0].clone()),
            1 => m.installed_repairs[0].from_manifest_sha256 = "invalid".into(),
            2 => m.installed_repairs[0].backup_required = false,
            3 => m.installed_repairs[0].transactional = false,
            4 => m.minimum_manager = "0.1.2".into(),
            5 => m.fresh_retry_from = vec![old.digest.clone()],
            6 => m.files.get_mut("migrations/repair-159.sql").unwrap().role = "deploy".into(),
            _ => m.installed_repairs.clear(),
        }
        assert_eq!(validate_manifest(&m, &trust), Err(Error::Release));
    }
    let mut raw = serde_json::to_value(&new.manifest).unwrap();
    raw["installed_repairs"][0]["arbitrary_sql"] = "SELECT true".into();
    assert!(serde_json::from_value::<villow_setup::release::Manifest>(raw).is_err());
}

#[test]
fn repair_requires_backup_original_checkpoint_and_credentials_before_mutation() {
    let (old, new, s) = repair_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    let fake = Fake::new(dir.path().join("cloud.json"));
    let vault = repair_vault(&s);
    assert_eq!(
        repair::advance(
            &store,
            &vault,
            &fake,
            s.clone(),
            &old,
            &new,
            false,
            |_, _| panic!("no SQL before backup confirmation")
        )
        .err(),
        Some(Error::RepairBackupRequired)
    );
    assert_eq!(
        repair::advance(
            &store,
            &MemoryVault::default(),
            &fake,
            s.clone(),
            &old,
            &new,
            true,
            |_, _| panic!("no SQL without vault")
        )
        .err(),
        Some(Error::MissingCredential)
    );
    assert!(store.load().unwrap().unwrap().installed_repair.is_none());
    for kind in 0..8 {
        let mut bad = s.clone();
        match kind {
            0 => bad.read_only = true,
            1 => bad.credentials_removed = true,
            2 => bad.step = Step::Complete,
            3 => bad.effects.get_mut("migrate").unwrap().status = EffectStatus::Executing,
            4 => bad.deployment_id = None,
            5 => bad.release_digest = "e".repeat(64),
            6 => {
                pending_repair(&mut bad, &new);
                bad.installed_repair.as_mut().unwrap().to = "f".repeat(64);
            }
            _ => {
                bad.effects.insert(
                    "unknown_pending_operation".into(),
                    Effect {
                        status: EffectStatus::Executing,
                        started_at: now(),
                        verified_at: None,
                    },
                );
            }
        }
        assert!(repair::validate(&bad, &old, &new).is_err());
    }
    assert_eq!(fake.read().deploys, 0);
}

#[test]
fn lost_database_commit_and_deployment_response_resume_once_with_original_data() {
    let (old, new, s) = repair_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    let fake = Fake::new(dir.path().join("cloud.json"));
    let vault = repair_vault(&s);
    let secrets = vault.values.borrow().clone();
    let committed = Cell::new(false);
    let writes = Cell::new(0);
    assert_eq!(
        repair::advance(
            &store,
            &vault,
            &fake,
            s.clone(),
            &old,
            &new,
            true,
            |state, apply| {
                if apply {
                    assert_eq!(
                        store.load()?.unwrap().installed_repair,
                        state.installed_repair
                    );
                    committed.set(true);
                    writes.set(writes.get() + 1);
                    return Err(Error::Uncertain);
                }
                Ok(())
            }
        )
        .err(),
        Some(Error::Uncertain)
    );
    drop(store);
    let store = Store::open(dir.path()).unwrap();
    let pending = store.load().unwrap().unwrap();
    assert_eq!(pending.release_digest, old.digest);
    assert_eq!(
        Engine {
            store: &store,
            vault: &vault,
            providers: &fake
        }
        .advance(&old)
        .err(),
        Some(Error::RepairPending)
    );
    assert_eq!(
        engine::remove_credentials(&store, &vault),
        Err(Error::RepairPending)
    );
    assert_eq!(
        engine::forget(&store, &vault, &s.name),
        Err(Error::RepairPending)
    );
    let recovered = recovery::import(&recovery::export(&pending).unwrap()).unwrap();
    assert!(recovered.read_only && recovered.installed_repair.is_none());
    let op = pending
        .installed_repair
        .as_ref()
        .unwrap()
        .operation_id
        .clone();
    let mut state = repair::advance(&store, &vault, &fake, pending, &old, &new, false, |_, _| {
        assert!(committed.get());
        Ok(())
    })
    .unwrap();
    assert_eq!(
        state.installed_repair.as_ref().unwrap().phase,
        RepairPhase::Upload
    );
    fake.crash.set(true);
    assert_eq!(
        repair::advance(
            &store,
            &vault,
            &fake,
            state.clone(),
            &old,
            &new,
            false,
            |_, _| Ok(())
        )
        .err(),
        Some(Error::Offline)
    );
    assert_eq!(fake.read().deploys, 0);
    fake.deployment_lost_response.set(true);
    assert_eq!(
        repair::advance(&store, &vault, &fake, state, &old, &new, false, |_, _| Ok(
            ()
        ))
        .err(),
        Some(Error::Uncertain)
    );
    assert_eq!(fake.read().deploys, 1);
    state = store.load().unwrap().unwrap();
    assert_eq!(
        state.installed_repair.as_ref().unwrap().phase,
        RepairPhase::Deploy
    );
    state = repair::advance(&store, &vault, &fake, state, &old, &new, false, |_, _| {
        Ok(())
    })
    .unwrap();
    assert_eq!(
        state.installed_repair.as_ref().unwrap().phase,
        RepairPhase::Verify
    );
    fake.deployment_status.set(DeploymentStatus::Building);
    state = repair::advance(&store, &vault, &fake, state, &old, &new, false, |_, _| {
        Ok(())
    })
    .unwrap();
    fake.deployment_status.set(DeploymentStatus::Ready);
    fake.health_ok.set(false);
    assert_eq!(
        repair::advance(&store, &vault, &fake, state, &old, &new, false, |_, _| Ok(
            ()
        ))
        .err(),
        Some(Error::Health)
    );
    state = store.load().unwrap().unwrap();
    assert_eq!(
        state.installed_repair.as_ref().unwrap().deployment_status,
        Some(DeploymentStatus::Ready)
    );
    fake.health_ok.set(true);
    state = repair::advance(&store, &vault, &fake, state, &old, &new, false, |_, _| {
        Ok(())
    })
    .unwrap();
    assert_eq!(state.step, Step::Complete);
    assert_eq!(state.release_digest, new.digest);
    assert_eq!(state.operation_id, op);
    assert_eq!(state.id, s.id);
    assert_eq!(state.origin, s.origin);
    assert_eq!(state.database.unwrap().id, s.database.unwrap().id);
    assert_eq!(
        serde_json::to_value(state.effects).unwrap(),
        serde_json::to_value(s.effects).unwrap()
    );
    assert_eq!(
        state.installed_repair.unwrap().previous_deployment_id,
        "dpl_original"
    );
    assert_eq!(*vault.values.borrow(), secrets);
    assert_eq!(writes.get(), 1);
    assert_eq!(fake.read().deploys, 1);
    assert_eq!(fake.read().creates, 0);
}
