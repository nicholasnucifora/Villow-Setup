//! Append-only update history. App SQL, checks, receipt and digest CAS share one
//! transaction. Historical fresh migrations and the Alpha repair remain intact.
use crate::repair_database::{check, text};
use crate::{
    error::{Error, Result},
    model::{Installation, RepairPhase},
    release::VerifiedRelease,
    update_contract::{self, Plan},
};
use postgres::{Client, Transaction};
use serde::{Deserialize, Serialize};
pub const LOCK: i64 = 0x56494c4c4f57;
pub const UPDATE_DDL: &str = "CREATE TABLE villow_setup.app_updates (position INTEGER PRIMARY KEY, receipt JSONB NOT NULL, receipt_hash TEXT NOT NULL, applied_at TIMESTAMPTZ NOT NULL DEFAULT now()); REVOKE ALL ON TABLE villow_setup.app_updates FROM PUBLIC, anon, authenticated, service_role";
pub const REPAIR_DDL: &str = "CREATE TABLE villow_setup.repairs (repair_id TEXT PRIMARY KEY, installation_id TEXT NOT NULL, from_digest TEXT NOT NULL, to_digest TEXT NOT NULL, sql_checksum TEXT NOT NULL, postcondition_checksum TEXT NOT NULL, operation_id TEXT NOT NULL, applied_at TIMESTAMPTZ NOT NULL DEFAULT now()); REVOKE ALL ON TABLE villow_setup.repairs FROM PUBLIC, anon, authenticated, service_role";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnitEvidence {
    pub id: String,
    pub sql_checksum: String,
    pub postcondition_checksum: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RepairEvidence {
    pub repair_id: String,
    pub installation_id: String,
    pub from_digest: String,
    pub to_digest: String,
    pub sql_checksum: String,
    pub postcondition_checksum: String,
    pub operation_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Base {
    pub manifest_sha256: String,
    pub migrations: Vec<UnitEvidence>,
    pub repair: Option<RepairEvidence>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub format: u32,
    pub installation_id: String,
    pub operation_id: String,
    pub from_digest: String,
    pub to_digest: String,
    pub from_schema_revision: String,
    pub to_schema_revision: String,
    pub plan_id: String,
    pub plan_sha256: String,
    pub previous_receipt_sha256: String,
    pub units: Vec<UnitEvidence>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Lineage {
    pub base: Base,
    pub receipts: Vec<Receipt>,
}
/// Only call with authenticated releases. Repaired 0.1.2 retains 0.1.1 history.
pub fn initial(
    s: &Installation,
    old: &VerifiedRelease,
    original: Option<&VerifiedRelease>,
) -> Result<Lineage> {
    if s.update_lineage.is_some() || s.release_digest != old.digest || s.update_pending() {
        return Err(Error::UpdateRefused);
    }
    let (baseline, repair) = if let Some(p) = &s.installed_repair {
        let original = original.ok_or(Error::UpdateRefused)?;
        let r = old
            .manifest
            .installed_repairs
            .first()
            .ok_or(Error::UpdateRefused)?;
        if p.phase != RepairPhase::Complete
            || p.to != old.digest
            || p.from != original.digest
            || r.from_manifest_sha256 != original.digest
            || p.repair_id != r.id
            || p.operation_id != s.operation_id
        {
            return Err(Error::UpdateRefused);
        }
        (
            original,
            Some(RepairEvidence {
                repair_id: r.id.clone(),
                installation_id: s.id.clone(),
                from_digest: original.digest.clone(),
                to_digest: old.digest.clone(),
                sql_checksum: old.manifest.files[&r.file].sha256.clone(),
                postcondition_checksum: old.manifest.files[&r.postcondition].sha256.clone(),
                operation_id: p.operation_id.clone(),
            }),
        )
    } else {
        if original.is_some() {
            return Err(Error::UpdateRefused);
        }
        (old, None)
    };
    Ok(Lineage {
        base: Base {
            manifest_sha256: baseline.digest.clone(),
            migrations: baseline
                .manifest
                .schema
                .migrations
                .iter()
                .map(|m| UnitEvidence {
                    id: m.id.clone(),
                    sql_checksum: baseline.manifest.files[&m.file].sha256.clone(),
                    postcondition_checksum: baseline.manifest.files[&m.postcondition]
                        .sha256
                        .clone(),
                })
                .collect(),
            repair,
        },
        receipts: vec![],
    })
}
pub fn receipt(s: &Installation, old: &VerifiedRelease, new: &VerifiedRelease) -> Result<Receipt> {
    let plan = update_contract::transition(old, new)?;
    let p = s.app_update.as_ref().ok_or(Error::UpdatePending)?;
    let history = s.update_lineage.as_ref().ok_or(Error::UpdateDatabase)?;
    if history.receipts.len() >= 128 {
        return Err(Error::UpdateRefused);
    }
    Ok(Receipt {
        format: 1,
        installation_id: s.id.clone(),
        operation_id: p.operation_id.clone(),
        from_digest: old.digest.clone(),
        to_digest: new.digest.clone(),
        from_schema_revision: plan.from_schema_revision.clone(),
        to_schema_revision: plan.to_schema_revision.clone(),
        plan_id: plan.id.clone(),
        plan_sha256: update_contract::canonical_hash(plan)?,
        previous_receipt_sha256: match history.receipts.last() {
            Some(r) => update_contract::canonical_hash(r)?,
            None => update_contract::canonical_hash(&history.base)?,
        },
        units: plan
            .migrations
            .iter()
            .map(|u| UnitEvidence {
                id: u.id.clone(),
                sql_checksum: new.manifest.files[&u.file].sha256.clone(),
                postcondition_checksum: new.manifest.files[&u.postcondition].sha256.clone(),
            })
            .collect(),
    })
}
pub fn validate_history(
    tx: &mut Transaction<'_>,
    s: &Installation,
    extra: Option<&Receipt>,
) -> Result<()> {
    let lineage = s.update_lineage.as_ref().ok_or(Error::UpdateDatabase)?;
    if lineage.receipts.len() > 128 {
        return Err(Error::UpdateDatabase);
    }
    let rows = tx.query("SELECT id,checksum,postcondition_checksum FROM villow_setup.migrations ORDER BY applied_at,id",&[]).map_err(|_|Error::UpdateDatabase)?;
    if rows.len() != lineage.base.migrations.len() {
        return Err(Error::UpdateDatabase);
    }
    for (row, e) in rows.iter().zip(&lineage.base.migrations) {
        if row.get::<_, String>(0) != e.id
            || row.get::<_, String>(1) != e.sql_checksum
            || row.get::<_, String>(2) != e.postcondition_checksum
        {
            return Err(Error::UpdateDatabase);
        }
    }
    let has_repair: bool = tx
        .query_one(
            "SELECT to_regclass('villow_setup.repairs') IS NOT NULL",
            &[],
        )
        .map_err(|_| Error::UpdateDatabase)?
        .get(0);
    if has_repair != lineage.base.repair.is_some() {
        return Err(Error::UpdateDatabase);
    }
    if let Some(r) = &lineage.base.repair {
        let rows=tx.query("SELECT repair_id,installation_id,from_digest,to_digest,sql_checksum,postcondition_checksum,operation_id FROM villow_setup.repairs",&[]).map_err(|_|Error::UpdateDatabase)?;
        let values = [
            &r.repair_id,
            &r.installation_id,
            &r.from_digest,
            &r.to_digest,
            &r.sql_checksum,
            &r.postcondition_checksum,
            &r.operation_id,
        ];
        if rows.len() != 1
            || values
                .iter()
                .enumerate()
                .any(|(i, v)| rows[0].get::<_, String>(i) != **v)
            || r.from_digest != lineage.base.manifest_sha256
            || r.installation_id != s.id
        {
            return Err(Error::UpdateDatabase);
        }
    }
    let expected = lineage.receipts.iter().chain(extra).collect::<Vec<_>>();
    let has_updates: bool = tx
        .query_one(
            "SELECT to_regclass('villow_setup.app_updates') IS NOT NULL",
            &[],
        )
        .map_err(|_| Error::UpdateDatabase)?
        .get(0);
    if has_updates != !expected.is_empty() {
        return Err(Error::UpdateDatabase);
    }
    let mut previous = update_contract::canonical_hash(&lineage.base)?;
    let mut digest = lineage
        .base
        .repair
        .as_ref()
        .map(|r| &r.to_digest)
        .unwrap_or(&lineage.base.manifest_sha256)
        .clone();
    if has_updates {
        let rows=tx.query("SELECT position,receipt::text,receipt_hash FROM villow_setup.app_updates ORDER BY position",&[]).map_err(|_|Error::UpdateDatabase)?;
        if rows.len() != expected.len() {
            return Err(Error::UpdateDatabase);
        }
        for (i, (row, r)) in rows.iter().zip(expected).enumerate() {
            let actual: Receipt = serde_json::from_str(&row.get::<_, String>(1))
                .map_err(|_| Error::UpdateDatabase)?;
            let checksum = update_contract::canonical_hash(r)?;
            if row.get::<_, i32>(0) != i as i32 + 1
                || &actual != r
                || row.get::<_, String>(2) != checksum
                || r.previous_receipt_sha256 != previous
                || r.from_digest != digest
                || r.installation_id != s.id
                || r.format != 1
            {
                return Err(Error::UpdateDatabase);
            }
            previous = checksum;
            digest = r.to_digest.clone();
        }
    }
    let target = extra
        .map(|r| r.to_digest.as_str())
        .unwrap_or(&s.release_digest);
    if digest != target {
        return Err(Error::UpdateDatabase);
    }
    Ok(())
}
pub fn source(tx: &mut Transaction<'_>, s: &Installation, old: &VerifiedRelease) -> Result<()> {
    instance(tx, s, &old.digest)?;
    validate_history(tx, s, None)?;
    crate::repair_database::owner(tx, s).map_err(|_| Error::UpdateDatabase)?;
    check(
        tx,
        text(
            old,
            &old.manifest
                .schema
                .migrations
                .last()
                .ok_or(Error::Release)?
                .postcondition,
        )?,
    )
    .map_err(|_| Error::UpdateDatabase)
}
fn instance(tx: &mut Transaction<'_>, s: &Installation, digest: &str) -> Result<()> {
    let rows = tx
        .query(
            "SELECT installation_id,release_digest FROM villow_setup.instance",
            &[],
        )
        .map_err(|_| Error::UpdateDatabase)?;
    if rows.len() != 1
        || rows[0].get::<_, String>(0) != s.id
        || rows[0].get::<_, String>(1) != digest
    {
        return Err(Error::UpdateDatabase);
    }
    Ok(())
}
pub fn check_or_apply(
    client: &mut Client,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    apply: bool,
) -> Result<()> {
    let plan = update_contract::transition(old, new)?;
    if apply && !s.update_pending() {
        return Err(Error::UpdatePending);
    }
    client.batch_execute("SET standard_conforming_strings=on; SET lock_timeout='10s'; SET statement_timeout='120s'").map_err(|_|Error::UpdateDatabase)?;
    let locked: bool = client
        .query_one("SELECT pg_try_advisory_lock($1)", &[&LOCK])
        .map_err(|_| Error::UpdateDatabase)?
        .get(0);
    if !locked {
        return Err(Error::Busy);
    }
    let result = transaction(client, s, old, new, plan, apply);
    let _ = client.execute("SELECT pg_advisory_unlock($1)", &[&LOCK]);
    result
}
fn transaction(
    client: &mut Client,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    p: &Plan,
    apply: bool,
) -> Result<()> {
    let mut tx = client.transaction().map_err(|_| Error::UpdateDatabase)?;
    tx.batch_execute("SELECT pg_advisory_xact_lock(905177311); LOCK TABLE villow_setup.instance,villow_setup.migrations,public.villow_installation,public.users,public.user_settings IN ACCESS EXCLUSIVE MODE").map_err(|_|Error::UpdateDatabase)?;
    let digest: String = tx
        .query_one("SELECT release_digest FROM villow_setup.instance", &[])
        .map_err(|_| Error::UpdateDatabase)?
        .get(0);
    if digest == new.digest {
        let r = receipt(s, old, new)?;
        instance(&mut tx, s, &new.digest)?;
        validate_history(&mut tx, s, Some(&r))?;
        let mut committed = s.clone();
        committed
            .update_lineage
            .as_mut()
            .ok_or(Error::UpdateDatabase)?
            .receipts
            .push(r);
        crate::update_backup_database::native_scope(&mut tx, &committed)?;
        crate::repair_database::owner(&mut tx, s).map_err(|_| Error::UpdateDatabase)?;
        check(&mut tx, text(new, &p.postcondition)?).map_err(|_| Error::UpdateDatabase)?;
        return tx.commit().map_err(|_| Error::Uncertain);
    }
    if digest != old.digest
        || s.app_update
            .as_ref()
            .is_some_and(|p| p.phase != RepairPhase::Database)
    {
        return Err(Error::UpdateDatabase);
    }
    source(&mut tx, s, old)?;
    let descriptor = update_contract::descriptor(p, &new.files)?;
    crate::update_backup_database::scope(&mut tx, s, &descriptor)?;
    check(&mut tx, text(new, &p.precondition)?).map_err(|_| Error::UpdateDatabase)?;
    if !apply {
        return tx.commit().map_err(|_| Error::UpdateDatabase);
    }
    for u in &p.migrations {
        tx.batch_execute(text(new, &u.file)?)
            .map_err(|_| Error::UpdateDatabase)?;
        check(&mut tx, text(new, &u.postcondition)?).map_err(|_| Error::UpdateDatabase)?;
    }
    check(&mut tx, text(new, &p.postcondition)?).map_err(|_| Error::UpdateDatabase)?;
    validate_history(&mut tx, s, None)?;
    crate::repair_database::owner(&mut tx, s).map_err(|_| Error::UpdateDatabase)?;
    if s.update_lineage
        .as_ref()
        .ok_or(Error::UpdateDatabase)?
        .receipts
        .is_empty()
    {
        tx.batch_execute(UPDATE_DDL)
            .map_err(|_| Error::UpdateDatabase)?;
    }
    let r = receipt(s, old, new)?;
    let json = serde_json::to_string(&r).map_err(|_| Error::UpdateDatabase)?;
    let checksum = update_contract::canonical_hash(&r)?;
    let position = s.update_lineage.as_ref().unwrap().receipts.len() as i32 + 1;
    tx.execute("INSERT INTO villow_setup.app_updates(position,receipt,receipt_hash) VALUES($1,$2::text::jsonb,$3)",&[&position,&json,&checksum]).map_err(|_|Error::UpdateDatabase)?;
    if tx.execute("UPDATE villow_setup.instance SET release_digest=$1 WHERE installation_id=$2 AND release_digest=$3",&[&new.digest,&s.id,&old.digest]).map_err(|_|Error::UpdateDatabase)?!=1 {return Err(Error::UpdateDatabase);}
    validate_history(&mut tx, s, Some(&r))?;
    let mut committed = s.clone();
    committed
        .update_lineage
        .as_mut()
        .ok_or(Error::UpdateDatabase)?
        .receipts
        .push(r);
    crate::update_backup_database::native_scope(&mut tx, &committed)?;
    tx.commit().map_err(|_| Error::Uncertain)
}
