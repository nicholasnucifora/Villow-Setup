mod common;
use common::*;
use villow_setup::{
    backup_database::DatabaseSnapshot,
    backup_file,
    error::{Error, Result},
    installed_repair as repair, managed_backup as managed,
    model::*,
    repair_backup::Package,
    store::Store,
    vault::Vault,
};
use zeroize::Zeroizing;

// Synthetic package exercises lifecycle only. app_backup separately captures
// and restores all real release tables through this same managed-file path.
fn backup(store: &Store, vault: &MemoryVault, s: &Installation, to: &str) -> BackupReceipt {
    let (path, key) = managed::prepare(store, vault, s, to).unwrap();
    let op = uuid::Uuid::new_v4().to_string();
    let at = now();
    let package = Package {
        format: 1,
        captured_at: at.clone(),
        repair_operation_id: op.clone(),
        destination_digest: to.into(),
        checkpoint_json: serde_json::to_string(s).unwrap(),
        credentials: vec![],
        configured_environment: vec![],
        channel_base64: String::new(),
        manifest_base64: String::new(),
        archive_base64: String::new(),
        database: DatabaseSnapshot {
            format: 1,
            server_major: 17,
            tables: vec![],
        },
    };
    let saved = backup_file::write_verified(&path, package.encode().unwrap(), &key).unwrap();
    BackupReceipt {
        managed: true,
        removed_at: None,
        path: saved.path,
        sha256: saved.sha256,
        bytes: saved.bytes,
        captured_at: at,
        installation_id: s.id.clone(),
        operation_id: op,
        from: s.release_digest.clone(),
        to: to.into(),
    }
}

#[test]
fn failed_or_interrupted_repair_retains_protection_until_saved_authenticated_success() {
    let (old, new, s) = repair_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    let vault = repair_vault(&s);
    let original = vault.values.borrow().clone();
    let fake = Fake::new(dir.path().join("cloud.json"));
    let receipt = backup(&store, &vault, &s, &new.digest);
    let file = std::path::Path::new(&receipt.path);
    let result = repair::advance(
        &store,
        &vault,
        &fake,
        s.clone(),
        &old,
        &new,
        Some(receipt.clone()),
        |_, apply| if apply { Err(Error::Uncertain) } else { Ok(()) },
    );
    assert_eq!(result.err(), Some(Error::Uncertain));
    let mut pending = store.load().unwrap().unwrap();
    managed::cleanup(&store, &vault, &mut pending).unwrap();
    assert!(file.exists());
    assert!(vault.require(&s.id, managed::KEY).is_ok());
    assert!(managed::prepare(&store, &vault, &pending, &new.digest).is_err());
    let exported =
        villow_setup::recovery::import(&villow_setup::recovery::export(&pending).unwrap()).unwrap();
    assert!(exported.read_only && exported.installed_repair.is_none());
    // Resume after the uncertain SQL response, then submit only one deployment.
    for _ in 0..2 {
        pending = repair::advance(&store, &vault, &fake, pending, &old, &new, None, |_, _| {
            Ok(())
        })
        .unwrap();
    }
    fake.health_ok.set(false);
    assert_eq!(
        repair::advance(&store, &vault, &fake, pending, &old, &new, None, |_, _| Ok(
            ()
        ))
        .err(),
        Some(Error::Health)
    );
    assert!(file.exists());
    pending = store.load().unwrap().unwrap();
    fake.health_ok.set(true);
    let completed = repair::advance(&store, &vault, &fake, pending, &old, &new, None, |_, _| {
        Ok(())
    })
    .unwrap();
    assert_eq!(completed.step, Step::Complete);
    assert!(completed
        .installed_repair
        .as_ref()
        .unwrap()
        .backup
        .as_ref()
        .unwrap()
        .removed_at
        .is_some());
    assert!(!file.exists());
    assert!(vault.get(&s.id, managed::KEY).unwrap().is_none());
    assert_eq!(*vault.values.borrow(), original);
    assert_eq!(fake.read().deploys, 1);
    assert_eq!(fake.read().creates, 0);
    let mut reopened = store.load().unwrap().unwrap();
    managed::cleanup(&store, &vault, &mut reopened).unwrap();
}

#[test]
fn missing_wrong_key_and_tampered_file_block_sql_and_never_regenerate_pending_protection() {
    let (old, new, s) = repair_fixture();
    for kind in 0..4 {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        store.save(&s).unwrap();
        let vault = repair_vault(&s);
        let fake = Fake::new(dir.path().join("cloud.json"));
        let r = backup(&store, &vault, &s, &new.digest);
        let _ = repair::advance(
            &store,
            &vault,
            &fake,
            s.clone(),
            &old,
            &new,
            Some(r.clone()),
            |_, apply| if apply { Err(Error::Uncertain) } else { Ok(()) },
        );
        match kind {
            0 => vault.delete(&s.id, managed::KEY).unwrap(),
            1 => vault.put(&s.id, managed::KEY, &"a".repeat(64)).unwrap(),
            2 => std::fs::write(&r.path, b"damaged").unwrap(),
            _ => std::fs::remove_file(&r.path).unwrap(),
        }
        let pending = store.load().unwrap().unwrap();
        assert!(repair::advance(
            &store,
            &vault,
            &fake,
            pending.clone(),
            &old,
            &new,
            None,
            |_, _| panic!("no SQL with lost protection")
        )
        .is_err());
        assert!(managed::prepare(&store, &vault, &pending, &new.digest).is_err());
        if kind == 0 {
            assert!(vault.get(&s.id, managed::KEY).unwrap().is_none());
        }
        assert_eq!(fake.read().deploys, 0);
    }
}

fn complete_projection(
    s: &Installation,
    new: &villow_setup::release::VerifiedRelease,
    r: BackupReceipt,
) -> Installation {
    let mut s = s.clone();
    pending_repair(&mut s, new);
    let p = s.installed_repair.as_mut().unwrap();
    p.operation_id = r.operation_id.clone();
    p.backup = Some(r);
    p.phase = RepairPhase::Verify;
    p.deployment_id = Some("dpl_repair".into());
    p.deployment_status = Some(DeploymentStatus::Ready);
    let mut target = repair::destination(&s, new).unwrap();
    target.installed_repair.as_mut().unwrap().phase = RepairPhase::Complete;
    target.step = Step::Complete;
    target.check("app", "Synthetic successful health");
    target
}

struct DeleteFails<'a>(&'a MemoryVault);
impl Vault for DeleteFails<'_> {
    fn get(&self, id: &str, key: &str) -> Result<Option<Zeroizing<String>>> {
        self.0.get(id, key)
    }
    fn put(&self, id: &str, key: &str, value: &str) -> Result<()> {
        self.0.put(id, key, value)
    }
    fn delete(&self, _: &str, _: &str) -> Result<()> {
        Err(Error::Vault)
    }
}

#[test]
fn cleanup_requires_durable_success_and_retries_after_file_or_key_deletion_failure() {
    let (_, new, s) = repair_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    let vault = repair_vault(&s);
    let r = backup(&store, &vault, &s, &new.digest);
    let file = std::path::Path::new(&r.path);
    let mut done = complete_projection(&s, &new, r.clone());
    assert!(managed::cleanup(&store, &vault, &mut done).is_err()); // Complete save failed/not performed.
    assert!(file.exists());
    store.save(&done).unwrap();
    #[cfg(windows)]
    {
        // Windows denies delete while a handle without FILE_SHARE_DELETE is open.
        use std::os::windows::fs::OpenOptionsExt;
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(file)
            .unwrap();
        assert!(managed::cleanup(&store, &vault, &mut done).is_err());
        assert!(file.exists());
        assert!(vault.require(&s.id, managed::KEY).is_ok());
        drop(held);
    }
    assert_eq!(
        managed::cleanup(&store, &DeleteFails(&vault), &mut done),
        Err(Error::Vault)
    );
    assert!(!file.exists());
    assert!(vault.require(&s.id, managed::KEY).is_ok());
    let mut reopened = store.load().unwrap().unwrap();
    assert!(reopened
        .installed_repair
        .as_ref()
        .unwrap()
        .backup
        .as_ref()
        .unwrap()
        .removed_at
        .is_none());
    managed::cleanup(&store, &vault, &mut reopened).unwrap();
    assert!(vault.get(&s.id, managed::KEY).unwrap().is_none());
    assert!(reopened
        .installed_repair
        .unwrap()
        .backup
        .unwrap()
        .removed_at
        .is_some());
}

#[test]
fn pre_intent_interruption_is_discoverable_and_recaptured_without_changing_original_secrets() {
    let (_, new, s) = repair_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    let vault = repair_vault(&s);
    let (path, key) = managed::prepare(&store, &vault, &s, &new.digest).unwrap(); // Interrupted before writing.
    let (_, reread) = managed::prepare(&store, &vault, &s, &new.digest).unwrap();
    assert_eq!(key.as_str(), reread.as_str());
    let first = backup(&store, &vault, &s, &new.digest); // Interrupted after durable file, before intent.
    let second = backup(&store, &vault, &s, &new.digest);
    assert_ne!(first.operation_id, second.operation_id);
    assert_eq!(first.path, second.path);
    assert!(path.exists());
    managed::check(&store, &vault, &s, &second).unwrap();
    assert!(managed::check(&store, &vault, &s, &first).is_err());
    vault.fail.set(true);
    assert!(managed::prepare(&store, &vault, &s, &new.digest).is_err());
    assert!(path.exists());
}

#[test]
fn portable_readonly_or_outside_path_receipts_never_authorize_cleanup() {
    let (old, new, s) = repair_fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.save(&s).unwrap();
    let vault = repair_vault(&s);
    let portable = backup_fixture(dir.path(), &s, &old, &new);
    let mut done = complete_projection(&s, &new, portable.clone());
    store.save(&done).unwrap();
    managed::cleanup(&store, &vault, &mut done).unwrap();
    assert!(std::path::Path::new(&portable.path).exists());
    done.installed_repair
        .as_mut()
        .unwrap()
        .backup
        .as_mut()
        .unwrap()
        .managed = true;
    store.save(&done).unwrap();
    assert!(managed::cleanup(&store, &vault, &mut done).is_err());
    assert!(std::path::Path::new(&portable.path).exists());
    done.read_only = true;
    store.save(&done).unwrap();
    assert_eq!(
        managed::cleanup(&store, &vault, &mut done),
        Err(Error::RecoveryReadOnly)
    );
    assert!(managed::path(&store, "../outside").is_err());
}
