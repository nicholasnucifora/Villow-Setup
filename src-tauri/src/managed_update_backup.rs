//! Temporary recovery protection for the authenticated app update. Paths
//! and unlock keys are native-owned; no renderer/file import can choose them.
use crate::{
    backup_file,
    error::{Error, Result},
    model::{now, BackupReceipt, DeploymentStatus, Installation, RepairPhase, Step},
    repair_backup::Package,
    store::Store,
    vault::Vault,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

pub const KEY: &str = "app_update_backup_key";

fn metadata(path: &Path) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(m) => {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if m.file_attributes() & 0x400 != 0 {
                    return Err(Error::BackupStorage);
                }
            }
            if m.file_type().is_symlink() {
                return Err(Error::BackupStorage);
            }
            Ok(Some(m))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(Error::BackupStorage),
    }
}

/// Exact native-derived file only. No recursive deletion, user path, junction
/// traversal, or scanning/removing unrelated backup files.
pub fn path(store: &Store, id: &str) -> Result<PathBuf> {
    if uuid::Uuid::parse_str(id)
        .map_err(|_| Error::BackupStorage)?
        .to_string()
        != id
    {
        return Err(Error::BackupStorage);
    }
    metadata(store.root())?.ok_or(Error::BackupStorage)?;
    let root = store
        .root()
        .canonicalize()
        .map_err(|_| Error::BackupStorage)?;
    let directory = root.join("app-update-backups");
    if let Some(m) = metadata(&directory)? {
        if !m.is_dir() {
            return Err(Error::BackupStorage);
        }
    }
    let path = directory.join(format!("{id}.villowbackup"));
    if let Some(m) = metadata(&path)? {
        if !m.is_file() {
            return Err(Error::BackupStorage);
        }
    }
    Ok(path)
}

fn staging(path: &Path) -> Result<PathBuf> {
    let staging = backup_file::staging_path(path)?;
    if let Some(m) = metadata(&staging)? {
        if !m.is_file() {
            return Err(Error::BackupStorage);
        }
    }
    Ok(staging)
}

/// Local removal must not strand a key/file before intent or while successful
/// repair still has pending cleanup. The caller already owns the store lock.
pub fn require_removable(store: &Store, vault: &dyn Vault, s: &Installation) -> Result<()> {
    let path = path(store, &s.id)?;
    let staging = staging(&path)?;
    if s.app_update
        .as_ref()
        .and_then(|p| p.backup.as_ref())
        .is_some_and(|r| r.managed && r.removed_at.is_none())
        || vault.get(&s.id, KEY)?.is_some()
        || metadata(&path)?.is_some()
        || metadata(&staging)?.is_some()
    {
        return Err(Error::BackupRetained);
    }
    Ok(())
}

fn package(path: &Path, vault: &dyn Vault, s: &Installation) -> Result<Package> {
    let key = vault.require(&s.id, KEY)?;
    let bytes = backup_file::read(path, &key)?;
    serde_json::from_slice(&bytes).map_err(|_| Error::BackupInvalid)
}

/// Called under the operation lock, before any repair intent/effects. A file
/// left by a crash before intent can be replaced only after authenticating its
/// encryption and original installation. Never reuse a stale data snapshot.
pub fn prepare(
    store: &Store,
    vault: &dyn Vault,
    s: &Installation,
    to: &str,
) -> Result<(PathBuf, Zeroizing<String>)> {
    s.assert_writable()?;
    if s.step != Step::Complete || s.app_update.is_some() {
        return Err(Error::UpdatePending);
    }
    let saved = store.load()?.ok_or(Error::UpdateRefused)?;
    if saved.id != s.id
        || saved.operation_id != s.operation_id
        || saved.release_digest != s.release_digest
        || saved.app_update.is_some()
        || saved.step != Step::Complete
        || saved.read_only
        || saved.credentials_removed
    {
        return Err(Error::UpdateRefused);
    }
    let path = path(store, &s.id)?;
    fs::create_dir_all(path.parent().ok_or(Error::BackupStorage)?)
        .map_err(|_| Error::BackupStorage)?;
    let staging = staging(&path)?;
    if metadata(&staging)?.is_some() {
        // The sole native staging name is always pre-SQL here. A killed write
        // may be incomplete, so discard it and capture afresh instead of
        // claiming it is a verified backup. Never regenerate a missing key.
        vault.require(&s.id, KEY)?;
        fs::remove_file(staging).map_err(|_| Error::BackupStorage)?;
    }
    if metadata(&path)?.is_some() {
        let previous = package(&path, vault, s)?;
        let checkpoint: Installation =
            serde_json::from_str(&previous.checkpoint_json).map_err(|_| Error::BackupInvalid)?;
        if previous.format != 2
            || previous.destination_digest != to
            || checkpoint.id != s.id
            || checkpoint.operation_id != s.operation_id
            || checkpoint.release_digest != s.release_digest
            || checkpoint.app_update.is_some()
        {
            return Err(Error::BackupInvalid);
        }
        // No repair intent was saved, so no repair SQL could have started.
        fs::remove_file(&path).map_err(|_| Error::BackupStorage)?;
    }
    let key = vault.ensure_random(&s.id, KEY)?;
    let reread = vault.require(&s.id, KEY)?;
    if key.as_str() != reread.as_str()
        || key.len() != 64
        || !key.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(Error::Vault);
    }
    Ok((path, reread))
}

pub fn check(store: &Store, vault: &dyn Vault, s: &Installation, r: &BackupReceipt) -> Result<()> {
    if !r.managed {
        return Ok(());
    } // Older portable backup: never managed/deleted.
    if r.removed_at.is_some() || r.installation_id != s.id {
        return Err(Error::RepairBackupRequired);
    }
    let path = path(store, &s.id)?;
    if path.to_str() != Some(r.path.as_str()) {
        return Err(Error::BackupStorage);
    }
    backup_file::matches_saved(&path, &r.sha256, r.bytes)?;
    let package = package(&path, vault, s)?;
    let original: Installation =
        serde_json::from_str(&package.checkpoint_json).map_err(|_| Error::BackupInvalid)?;
    if package.format != 2
        || package.repair_operation_id != r.operation_id
        || package.destination_digest != r.to
        || package.captured_at != r.captured_at
        || original.id != r.installation_id
        || original.release_digest != r.from
        || original.operation_id
            != s.app_update
                .as_ref()
                .map(|p| p.previous_operation_id.as_str())
                .unwrap_or(&s.operation_id)
        || original.owner_email != s.owner_email
    {
        return Err(Error::BackupInvalid);
    }
    Ok(())
}

/// The caller holds the local lock. Completion must already be durable. An
/// interrupted cleanup retries on next open; any failure keeps its pending
/// receipt visible. Never delete a failed/in-progress or portable backup.
pub fn cleanup(store: &Store, vault: &dyn Vault, s: &mut Installation) -> Result<()> {
    let Some(p) = &s.app_update else {
        return Ok(());
    };
    let Some(r) = &p.backup else {
        return Ok(());
    };
    if !r.managed || r.removed_at.is_some() || p.phase != RepairPhase::Complete {
        return Ok(());
    }
    s.assert_writable()?;
    if s.step != Step::Complete
        || s.release_digest != p.to
        || s.operation_id != p.operation_id
        || r.installation_id != s.id
        || r.operation_id != p.operation_id
        || r.from != p.from
        || r.to != p.to
        || s.deployment_id != p.deployment_id
        || p.deployment_id.is_none()
        || p.deployment_status != Some(DeploymentStatus::Ready)
        || s.deployment_status != Some(DeploymentStatus::Ready)
        || !s.checks.iter().any(|c| c.kind == "app")
    {
        return Err(Error::UpdateRefused);
    }
    // Do not accept an in-memory Complete projection before it is persisted.
    let saved = store.load()?.ok_or(Error::UpdateRefused)?;
    if saved.read_only
        || saved.step != Step::Complete
        || saved.id != s.id
        || saved.release_digest != s.release_digest
        || saved.app_update != s.app_update
    {
        return Err(Error::UpdateRefused);
    }
    let path = path(store, &s.id)?;
    if path.to_str() != Some(r.path.as_str()) {
        return Err(Error::BackupStorage);
    }
    if metadata(&path)?.is_some() {
        check(store, vault, s, r)?;
        fs::remove_file(&path).map_err(|_| Error::BackupStorage)?;
    }
    let staging = staging(&path)?;
    if metadata(&staging)?.is_some() {
        // persist_noclobber may have linked the final file before unlinking
        // staging when interrupted. Only the exact completed ciphertext may
        // be removed by post-success cleanup.
        backup_file::matches_saved(&staging, &r.sha256, r.bytes)?;
        fs::remove_file(staging).map_err(|_| Error::BackupStorage)?;
    }
    // File first, key second: a failed file removal never destroys its key.
    vault.delete(&s.id, KEY)?;
    if vault.get(&s.id, KEY)?.is_some() {
        return Err(Error::Vault);
    }
    let mut cleaned = s.clone();
    cleaned
        .app_update
        .as_mut()
        .unwrap()
        .backup
        .as_mut()
        .unwrap()
        .removed_at = Some(now());
    cleaned.updated_at = now();
    store.save(&cleaned)?;
    *s = cleaned;
    Ok(())
}
