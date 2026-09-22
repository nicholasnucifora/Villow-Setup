//! Installed repair protocol. Public schema SQL comes only from the signed app.
use crate::{
    error::{Error, Result},
    installed_repair,
    model::Installation,
    release::{InstalledRepair, VerifiedRelease},
};
use postgres::{Client, Transaction};
const LOCK: i64 = 0x56494c4c4f57;

/// Read-only validation inside the backup's repeatable-read snapshot.
pub(crate) fn validate_backup_source(
    tx: &mut Transaction<'_>,
    s: &Installation,
    old: &VerifiedRelease,
) -> Result<()> {
    let rows = tx
        .query(
            "SELECT installation_id,release_digest FROM villow_setup.instance",
            &[],
        )
        .map_err(|_| Error::RepairDatabase)?;
    if rows.len() != 1
        || rows[0].get::<_, String>(0) != s.id
        || rows[0].get::<_, String>(1) != old.digest
    {
        return Err(Error::RepairDatabase);
    }
    ledger(tx, old)?;
    owner(tx, s)?;
    for unit in &old.manifest.schema.migrations {
        check(tx, text(old, &unit.postcondition)?)?;
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
    let repair = installed_repair::validate(s, old, new)?;
    if apply && s.installed_repair.is_none() {
        return Err(Error::RepairRefused);
    }
    client.batch_execute("SET standard_conforming_strings = on; SET lock_timeout = '10s'; SET statement_timeout = '120s'").map_err(|_| Error::Database)?;
    let locked: bool = client
        .query_one("SELECT pg_try_advisory_lock($1)", &[&LOCK])
        .map_err(|_| Error::Database)?
        .get(0);
    if !locked {
        return Err(Error::Busy);
    }
    let result = transaction(client, s, old, new, repair, apply);
    let _ = client.execute("SELECT pg_advisory_unlock($1)", &[&LOCK]);
    result
}
fn transaction(
    client: &mut Client,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    repair: &InstalledRepair,
    apply: bool,
) -> Result<()> {
    let mut tx = client.transaction().map_err(|_| Error::Database)?;
    // Same owner lock used by the app's sign-in function. Preserve rows while checking.
    tx.batch_execute("SELECT pg_advisory_xact_lock(905177311); LOCK TABLE villow_setup.instance, villow_setup.migrations, public.villow_installation, public.users, public.user_settings IN ACCESS EXCLUSIVE MODE").map_err(|_| Error::RepairDatabase)?;
    let owners = tx
        .query(
            "SELECT installation_id,release_digest FROM villow_setup.instance",
            &[],
        )
        .map_err(|_| Error::RepairDatabase)?;
    if owners.len() != 1 || owners[0].try_get::<_, String>(0).ok().as_ref() != Some(&s.id) {
        return Err(Error::RepairDatabase);
    }
    let digest: String = owners[0].try_get(1).map_err(|_| Error::RepairDatabase)?;
    ledger(&mut tx, old)?;
    owner(&mut tx, s)?;
    let receipt_table: Option<String> = tx
        .query_one("SELECT to_regclass('villow_setup.repairs')::text", &[])
        .map_err(|_| Error::RepairDatabase)?
        .get(0);
    if digest == new.digest {
        if s.installed_repair.is_none() || receipt_table.is_none() {
            return Err(Error::RepairDatabase);
        }
        let rows = tx.query("SELECT installation_id,from_digest,to_digest,repair_id,sql_checksum,postcondition_checksum,operation_id FROM villow_setup.repairs", &[]).map_err(|_| Error::RepairDatabase)?;
        let expected = [
            &s.id,
            &old.digest,
            &new.digest,
            &repair.id,
            &new.manifest.files[&repair.file].sha256,
            &new.manifest.files[&repair.postcondition].sha256,
            &s.installed_repair.as_ref().unwrap().operation_id,
        ];
        if rows.len() != 1
            || expected
                .iter()
                .enumerate()
                .any(|(i, v)| rows[0].try_get::<_, String>(i).ok().as_ref() != Some(*v))
        {
            return Err(Error::RepairDatabase);
        }
        check(&mut tx, text(new, &repair.postcondition)?)?;
        return tx.commit().map_err(|_| Error::Uncertain);
    }
    if digest != old.digest
        || receipt_table.is_some()
        || s.installed_repair
            .as_ref()
            .is_some_and(|p| p.phase != crate::model::RepairPhase::Database)
    {
        return Err(Error::RepairDatabase);
    }
    for unit in &old.manifest.schema.migrations {
        check(&mut tx, text(old, &unit.postcondition)?)?;
    }
    if !apply {
        return tx.commit().map_err(|_| Error::Database);
    }
    tx.batch_execute(text(new, &repair.file)?)
        .map_err(|_| Error::RepairDatabase)?;
    check(&mut tx, text(new, &repair.postcondition)?)?;
    ledger(&mut tx, old)?;
    owner(&mut tx, s)?;
    tx.batch_execute("CREATE TABLE villow_setup.repairs (repair_id TEXT PRIMARY KEY, installation_id TEXT NOT NULL, from_digest TEXT NOT NULL, to_digest TEXT NOT NULL, sql_checksum TEXT NOT NULL, postcondition_checksum TEXT NOT NULL, operation_id TEXT NOT NULL, applied_at TIMESTAMPTZ NOT NULL DEFAULT now()); REVOKE ALL ON TABLE villow_setup.repairs FROM PUBLIC, anon, authenticated, service_role").map_err(|_| Error::RepairDatabase)?;
    let p = s.installed_repair.as_ref().ok_or(Error::RepairRefused)?;
    tx.execute("INSERT INTO villow_setup.repairs(repair_id,installation_id,from_digest,to_digest,sql_checksum,postcondition_checksum,operation_id) VALUES($1,$2,$3,$4,$5,$6,$7)", &[&repair.id, &s.id, &old.digest, &new.digest, &new.manifest.files[&repair.file].sha256, &new.manifest.files[&repair.postcondition].sha256, &p.operation_id]).map_err(|_| Error::RepairDatabase)?;
    if tx.execute("UPDATE villow_setup.instance SET release_digest=$1 WHERE installation_id=$2 AND release_digest=$3", &[&new.digest, &s.id, &old.digest]).map_err(|_| Error::RepairDatabase)? != 1 { return Err(Error::RepairDatabase); }
    tx.commit().map_err(|_| Error::Uncertain)
}
fn ledger(tx: &mut Transaction<'_>, old: &VerifiedRelease) -> Result<()> {
    let rows = tx.query("SELECT id,checksum,postcondition_checksum FROM villow_setup.migrations ORDER BY applied_at,id", &[]).map_err(|_| Error::RepairDatabase)?;
    if rows.len() != old.manifest.schema.migrations.len() {
        return Err(Error::RepairDatabase);
    }
    for (row, unit) in rows.iter().zip(&old.manifest.schema.migrations) {
        if row.try_get::<_, String>(0).ok().as_ref() != Some(&unit.id)
            || row.try_get::<_, String>(1).ok().as_ref()
                != Some(&old.manifest.files[&unit.file].sha256)
            || row.try_get::<_, String>(2).ok().as_ref()
                != Some(&old.manifest.files[&unit.postcondition].sha256)
        {
            return Err(Error::RepairDatabase);
        }
    }
    Ok(())
}
fn owner(tx: &mut Transaction<'_>, s: &Installation) -> Result<()> {
    let matches: bool = tx.query_one("SELECT (SELECT count(*)=1 FROM public.villow_installation) AND (SELECT count(*)=1 FROM public.users WHERE is_system_owner) AND (SELECT count(*)=1 FROM public.villow_installation i JOIN public.users u ON u.id=i.owner_id JOIN public.user_settings x ON x.user_id=u.id WHERE i.singleton AND i.installation_id::text=$1 AND i.expected_owner_email=$2 AND i.owner_email=$2 AND lower(trim(u.email))=$2 AND i.owner_google_id=u.google_id AND i.owner_google_id<>'' AND u.is_system_owner AND u.access_revoked_at IS NULL AND i.bootstrap_closed_at IS NOT NULL AND i.youtube_verified_at IS NOT NULL)", &[&s.id, &s.owner_email]).map_err(|_| Error::RepairDatabase)?.get(0);
    if !matches {
        return Err(Error::RepairDatabase);
    }
    Ok(())
}
fn text<'a>(r: &'a VerifiedRelease, path: &str) -> Result<&'a str> {
    std::str::from_utf8(r.files.get(path).ok_or(Error::Release)?).map_err(|_| Error::Release)
}
fn check(tx: &mut Transaction<'_>, sql: &str) -> Result<()> {
    let rows = tx.query(sql, &[]).map_err(|_| Error::RepairDatabase)?;
    if rows.len() != 1 || rows[0].len() != 1 || rows[0].try_get::<_, bool>(0).ok() != Some(true) {
        return Err(Error::RepairDatabase);
    }
    Ok(())
}
