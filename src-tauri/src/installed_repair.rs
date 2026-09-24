//! One authenticated correction of a manager-created, unfinished Alpha install.
//! Normal fresh installation and general upgrades never enter this state machine.
use crate::{
    engine::Providers,
    error::{Error, Result},
    model::*,
    release::{InstalledRepair, VerifiedRelease},
    store::Store,
    vault::Vault,
};

pub fn validate<'a>(
    s: &Installation,
    old: &VerifiedRelease,
    new: &'a VerifiedRelease,
) -> Result<&'a InstalledRepair> {
    s.assert_writable()?;
    let repair = new
        .manifest
        .installed_repairs
        .first()
        .ok_or(Error::RepairRefused)?;
    if s.step != Step::Health
        || s.effects.len() != 7
        || s.fresh_retry.is_some()
        || s.release_digest != old.digest
        || s.commit != old.manifest.commit
        || s.app_version != old.manifest.app_version
        || s.schema_revision != old.manifest.schema.revision
        || repair.from_manifest_sha256 != old.digest
        || repair.from_schema_revision != old.manifest.schema.revision
        || old.digest == new.digest
        || old.manifest.schema.kind != "fresh_baseline"
        || new.manifest.schema.kind != "fresh_baseline"
        || !old.manifest.installed_repairs.is_empty()
        || !new.manifest.upgrade_from.is_empty()
        || !new.manifest.fresh_retry_from.is_empty()
        || new.manifest.installed_repairs.len() != 1
        || new.manifest.sequence < old.manifest.sequence
        || new.manifest.configuration != old.manifest.configuration
        || new.manifest.google_scopes != old.manifest.google_scopes
        || new.manifest.bootstrap_contract != old.manifest.bootstrap_contract
        || new.manifest.health_contract != old.manifest.health_contract
        || s.google_scopes != old.manifest.google_scopes
        || !s.google.as_ref().is_some_and(Google::ready_for_setup)
        || !s.selection()?.costs_acknowledged
        || s.deployment_status != Some(DeploymentStatus::Ready)
        || [
            "create_vercel",
            "create_database",
            "reserve_origin",
            "migrate",
            "configure",
            "upload",
            "deploy",
        ]
        .iter()
        .any(|name| {
            !s.effects
                .get(*name)
                .is_some_and(|e| e.status == EffectStatus::Verified)
        })
    {
        return Err(Error::RepairRefused);
    }
    s.project()?;
    s.database()?;
    s.origin()?;
    let previous = s.deployment_id.as_ref().ok_or(Error::RepairRefused)?;
    if let Some(p) = &s.installed_repair {
        if p.from != old.digest
            || p.to != new.digest
            || p.repair_id != repair.id
            || p.previous_operation_id != s.operation_id
            || &p.previous_deployment_id != previous
            || p.phase == RepairPhase::Complete
            || p.backup_confirmed_at.is_empty()
            || chrono::DateTime::parse_from_rfc3339(&p.backup_confirmed_at).is_err()
            || uuid::Uuid::parse_str(&p.operation_id).is_err()
            || p.operation_id == s.operation_id
            || (p.phase == RepairPhase::Verify) != p.deployment_id.is_some()
        {
            return Err(Error::RepairRefused);
        }
    }
    Ok(repair)
}

pub fn require_credentials(s: &Installation, vault: &dyn Vault) -> Result<()> {
    for name in [
        "encryption_key",
        "bootstrap_token",
        "db_password",
        "google_secret",
        "vercel_token",
        "supabase_token",
        "service_key",
        "anon_key",
    ] {
        vault.require(&s.id, name)?;
    }
    Ok(())
}

// Project the replacement deployment without erasing the original checkpoint.
pub fn destination(s: &Installation, new: &VerifiedRelease) -> Result<Installation> {
    let p = s.installed_repair.as_ref().ok_or(Error::RepairRefused)?;
    let mut target = s.clone();
    target.operation_id = p.operation_id.clone();
    target.release_digest = new.digest.clone();
    target.commit = new.manifest.commit.clone();
    target.app_version = new.manifest.app_version.clone();
    target.schema_revision = new.manifest.schema.revision.clone();
    target.release_sequence = new.manifest.sequence;
    target.deployment_id = p.deployment_id.clone();
    target.deployment_status = p.deployment_status;
    Ok(target)
}

// The caller holds the local operation lock and authenticates BOTH releases.
// database(false) is a read/check; database(true) reconciles or applies one SQL transaction.
pub fn advance(
    store: &Store,
    vault: &dyn Vault,
    providers: &dyn Providers,
    mut s: Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    backup: Option<BackupReceipt>,
    mut database: impl FnMut(&Installation, bool) -> Result<()>,
) -> Result<Installation> {
    let repair = validate(&s, old, new)?;
    require_credentials(&s, vault)?;
    providers.verify_targets(&s)?;
    if s.installed_repair.is_none() {
        let backup = backup.ok_or(Error::RepairBackupRequired)?;
        crate::repair_backup::check_receipt(&backup, &s, old, new)?;
        crate::managed_backup::check(store, vault, &s, &backup)?;
        if providers.deployment_status(&s, old)? != DeploymentStatus::Ready {
            return Err(Error::DeploymentNotReady);
        }
        database(&s, false)?;
        s.installed_repair = Some(RepairIntent {
            from: old.digest.clone(),
            to: new.digest.clone(),
            repair_id: repair.id.clone(),
            operation_id: backup.operation_id.clone(),
            backup_confirmed_at: backup.captured_at.clone(),
            backup: Some(backup),
            previous_operation_id: s.operation_id.clone(),
            previous_deployment_id: s.deployment_id.clone().unwrap(),
            phase: RepairPhase::Database,
            deployment_id: None,
            deployment_status: None,
        });
        save(store, &mut s)?; // Before SQL or any replacement deployment.
    }
    if let Some(receipt) = &s.installed_repair.as_ref().unwrap().backup {
        if receipt.operation_id != s.installed_repair.as_ref().unwrap().operation_id {
            return Err(Error::RepairBackupRequired);
        }
        crate::repair_backup::check_receipt(receipt, &s, old, new)?;
        crate::managed_backup::check(store, vault, &s, receipt)?;
    }
    // Existing 0.1.3 intent may have an explicitly owner-confirmed manual backup.
    // Resume that recorded operation; never label its backup as native-verified.
    match s.installed_repair.as_ref().unwrap().phase {
        RepairPhase::Database => {
            database(&s, true)?;
            s.installed_repair.as_mut().unwrap().phase = RepairPhase::Upload;
            save(store, &mut s)?;
        }
        RepairPhase::Upload => {
            database(&s, false)?; // Require the committed receipt and corrected schema.
            let target = destination(&s, new)?;
            providers.upload(&target, new)?; // Content-addressed; safe to repeat.
            s.installed_repair.as_mut().unwrap().phase = RepairPhase::Deploy;
            save(store, &mut s)?; // Any later failure requires reconciliation, never a second POST.
            let id = providers.deploy(&target, new, false)?;
            save_deployment(store, &mut s, id)?;
        }
        RepairPhase::Deploy => {
            database(&s, false)?;
            let id = providers.deploy(&destination(&s, new)?, new, true)?;
            save_deployment(store, &mut s, id)?;
        }
        RepairPhase::Verify => {
            let target = destination(&s, new)?;
            let status = providers.deployment_status(&target, new)?;
            s.installed_repair.as_mut().unwrap().deployment_status = Some(status);
            save(store, &mut s)?;
            if status == DeploymentStatus::Ready {
                database(&s, false)?;
                let target = destination(&s, new)?;
                providers.health(&target, new)?;
                s = target;
                s.installed_repair.as_mut().unwrap().phase = RepairPhase::Complete;
                s.step = Step::Complete;
                s.check("app", "Installed Alpha repair, replacement deployment and authenticated owner health verified");
                save(store, &mut s)?;
                // Success is durable before removing any temporary copy. A
                // cleanup failure is retried on open and remains visible.
                let _ = crate::managed_backup::cleanup(store, vault, &mut s);
            }
        }
        RepairPhase::Complete => return Err(Error::RepairRefused),
    }
    Ok(s)
}
fn save(store: &Store, s: &mut Installation) -> Result<()> {
    s.updated_at = now();
    store.save(s)
}
fn save_deployment(store: &Store, s: &mut Installation, id: String) -> Result<()> {
    identifier(&id)?;
    let p = s.installed_repair.as_mut().ok_or(Error::RepairRefused)?;
    if id == p.previous_deployment_id {
        return Err(Error::WrongTarget);
    }
    p.deployment_id = Some(id);
    p.deployment_status = Some(DeploymentStatus::Queued);
    p.phase = RepairPhase::Verify;
    save(store, s)
}
