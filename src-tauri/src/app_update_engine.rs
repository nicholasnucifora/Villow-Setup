//! One bounded step of a completed-installation update. No configure operation:
//! original encryption and bootstrap secrets and provider resources stay stable.
use crate::{
    engine::Providers,
    error::{Error, Result},
    model::*,
    release::VerifiedRelease,
    store::Store,
    update_contract, update_database,
    vault::Vault,
};
pub fn validate(s: &Installation, old: &VerifiedRelease, new: &VerifiedRelease) -> Result<()> {
    s.assert_writable()?;
    let p = update_contract::transition(old, new)?;
    if s.step != Step::Complete
        || s.repair_pending()
        || s.fresh_retry.is_some()
        || s.release_digest != old.digest
        || s.app_version != old.manifest.app_version
        || s.commit != old.manifest.commit
        || s.schema_revision != old.manifest.schema.revision
        || s.google_scopes != old.manifest.google_scopes
        || s.release_sequence != old.manifest.sequence
        || !s.google.as_ref().is_some_and(Google::ready_for_setup)
        || !s.selection()?.costs_acknowledged
        || s.deployment_status != Some(DeploymentStatus::Ready)
        || s.effects.len() != 7
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
        .any(|n| {
            s.effects
                .get(*n)
                .is_none_or(|e| e.status != EffectStatus::Verified)
        })
    {
        return Err(Error::UpdateRefused);
    }
    s.project()?;
    s.database()?;
    s.origin()?;
    let deployment = s.deployment_id.as_ref().ok_or(Error::UpdateRefused)?;
    if let Some(i) = &s.app_update {
        if i.phase == RepairPhase::Complete
            || i.from != old.digest
            || i.to != new.digest
            || i.repair_id != p.id
            || i.previous_operation_id != s.operation_id
            || &i.previous_deployment_id != deployment
            || i.operation_id == s.operation_id
            || uuid::Uuid::parse_str(&i.operation_id).is_err()
            || i.backup.is_none()
            || chrono::DateTime::parse_from_rfc3339(&i.backup_confirmed_at).is_err()
            || (i.phase == RepairPhase::Verify) != i.deployment_id.is_some()
        {
            return Err(Error::UpdateRefused);
        }
    }
    Ok(())
}
pub fn destination(s: &Installation, new: &VerifiedRelease) -> Result<Installation> {
    let p = s.app_update.as_ref().ok_or(Error::UpdatePending)?;
    let mut target = s.clone();
    target.operation_id = p.operation_id.clone();
    target.release_digest = new.digest.clone();
    target.app_version = new.manifest.app_version.clone();
    target.commit = new.manifest.commit.clone();
    target.release_sequence = new.manifest.sequence;
    target.schema_revision = new.manifest.schema.revision.clone();
    target.deployment_id = p.deployment_id.clone();
    target.deployment_status = p.deployment_status;
    Ok(target)
}
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
    validate(&s, old, new)?;
    crate::installed_repair::require_credentials(&s, vault)?;
    providers.verify_targets(&s)?;
    if s.app_update.is_none() {
        let receipt = backup.ok_or(Error::RepairBackupRequired)?;
        if !receipt.managed {
            return Err(Error::BackupInvalid);
        }
        if s.update_lineage.is_none() {
            return Err(Error::UpdateDatabase);
        }
        crate::repair_backup::check_receipt(&receipt, &s, old, new)?;
        crate::managed_update_backup::check(store, vault, &s, &receipt)?;
        if providers.deployment_status(&s, old)? != DeploymentStatus::Ready {
            return Err(Error::DeploymentNotReady);
        }
        database(&s, false)?;
        s.app_update = Some(RepairIntent {
            from: old.digest.clone(),
            to: new.digest.clone(),
            repair_id: update_contract::transition(old, new)?.id.clone(),
            operation_id: receipt.operation_id.clone(),
            backup_confirmed_at: receipt.captured_at.clone(),
            backup: Some(receipt),
            previous_operation_id: s.operation_id.clone(),
            previous_deployment_id: s.deployment_id.clone().ok_or(Error::UpdateRefused)?,
            phase: RepairPhase::Database,
            deployment_id: None,
            deployment_status: None,
        });
        save(store, &mut s)?;
    } else if backup.is_some() {
        return Err(Error::UpdatePending);
    }
    let p = s.app_update.as_ref().unwrap();
    let r = p.backup.as_ref().ok_or(Error::RepairBackupRequired)?;
    if r.operation_id != p.operation_id {
        return Err(Error::BackupInvalid);
    }
    crate::repair_backup::check_receipt(r, &s, old, new)?;
    crate::managed_update_backup::check(store, vault, &s, r)?;
    match p.phase {
        RepairPhase::Database => {
            database(&s, true)?;
            s.app_update.as_mut().unwrap().phase = RepairPhase::Upload;
            save(store, &mut s)?;
        }
        RepairPhase::Upload => {
            database(&s, false)?;
            let target = destination(&s, new)?;
            providers.upload(&target, new)?;
            s.app_update.as_mut().unwrap().phase = RepairPhase::Deploy;
            save(store, &mut s)?;
            let id = providers.deploy(&target, new, false)?;
            save_deployment(store, &mut s, id)?;
        }
        RepairPhase::Deploy => {
            database(&s, false)?;
            let id = providers.deploy(&destination(&s, new)?, new, true)?;
            save_deployment(store, &mut s, id)?;
        }
        RepairPhase::Verify => {
            let status = providers.deployment_status(&destination(&s, new)?, new)?;
            s.app_update.as_mut().unwrap().deployment_status = Some(status);
            save(store, &mut s)?;
            if status == DeploymentStatus::Ready {
                database(&s, false)?;
                let target = destination(&s, new)?;
                providers.health(&target, new)?;
                let receipt = update_database::receipt(&s, old, new)?;
                s = target;
                s.update_lineage
                    .as_mut()
                    .ok_or(Error::UpdateDatabase)?
                    .receipts
                    .push(receipt);
                s.app_update.as_mut().unwrap().phase = RepairPhase::Complete;
                s.check(
                    "app",
                    "App update, database history and authenticated owner health verified",
                );
                save(store, &mut s)?;
                let _ = crate::managed_update_backup::cleanup(store, vault, &mut s);
            }
        }
        RepairPhase::Complete => return Err(Error::UpdateRefused),
    }
    Ok(s)
}
fn save(store: &Store, s: &mut Installation) -> Result<()> {
    s.updated_at = now();
    store.save(s)
}
fn save_deployment(store: &Store, s: &mut Installation, id: String) -> Result<()> {
    identifier(&id)?;
    let p = s.app_update.as_mut().ok_or(Error::UpdatePending)?;
    if p.previous_deployment_id == id {
        return Err(Error::WrongTarget);
    }
    p.deployment_id = Some(id);
    p.deployment_status = Some(DeploymentStatus::Queued);
    p.phase = RepairPhase::Verify;
    save(store, s)
}
