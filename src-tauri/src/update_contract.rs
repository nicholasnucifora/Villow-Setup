//! Authenticated, bounded transitions. These declarations never authorize a
//! fresh baseline on an existing database or a change to provider permissions.
use crate::{
    backup_database::TableSchema,
    error::{Error, Result},
    release::{self, Manifest, VerifiedRelease},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: String,
    pub file: String,
    pub postcondition: String,
    pub transactional: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub id: String,
    pub from_manifest_sha256: String,
    pub from_schema_revision: String,
    pub to_schema_revision: String,
    pub kind: String,
    pub source_backup: String,
    pub precondition: String,
    pub migrations: Vec<Unit>,
    pub postcondition: String,
    pub backup_required: bool,
    pub previous_app_compatible: bool,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    pub table: String,
    pub name: String,
    pub enabled: String,
    pub definition: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceBackup {
    pub format: u32,
    pub from_manifest_sha256: String,
    pub from_schema_revision: String,
    pub restore_baseline_manifest_sha256: String,
    pub restore_baseline: String,
    pub restore_postcondition: String,
    pub native_ledger_contract: u32,
    pub tables: Vec<TableSchema>,
    pub triggers: Vec<Trigger>,
}
fn role(m: &Manifest, path: &str, expected: &str) -> Result<()> {
    if m.files.get(path).is_none_or(|s| s.role != expected) {
        return Err(Error::Release);
    }
    Ok(())
}
pub fn validate_manifest(m: &Manifest) -> Result<()> {
    let sources: BTreeSet<_> = m.upgrade_from.iter().collect();
    if sources.len() != m.upgrade_from.len()
        || sources.len() > 8
        || sources.iter().any(|s| !release::is_hash(s))
        || sources
            != m.app_updates
                .iter()
                .map(|p| &p.from_manifest_sha256)
                .collect()
        || m.app_updates.len() != sources.len()
    {
        return Err(Error::Release);
    }
    if !m.app_updates.is_empty()
        && (!m.installed_repairs.is_empty()
            || !m.fresh_retry_from.is_empty()
            || semver::Version::parse(&m.minimum_manager).map_err(|_| Error::Release)?
                < semver::Version::new(0, 2, 0))
    {
        return Err(Error::Release);
    }
    let mut ids = BTreeSet::new();
    for p in &m.app_updates {
        let prefix = format!("updates/{}/", p.id);
        if crate::model::identifier(&p.id).is_err()
            || !ids.insert(&p.id)
            || crate::model::identifier(&p.from_schema_revision).is_err()
            || p.to_schema_revision != m.schema.revision
            || !p.backup_required
            || !p.previous_app_compatible
            || p.migrations.len() > 32
            || p.postcondition
                != m.schema
                    .migrations
                    .last()
                    .ok_or(Error::Release)?
                    .postcondition
            || !p.source_backup.starts_with(&prefix)
            || !p.source_backup.ends_with(".json")
            || !p.precondition.starts_with(&prefix)
            || !p.precondition.ends_with(".sql")
        {
            return Err(Error::Release);
        }
        match p.kind.as_str() {
            "code_only"
                if p.migrations.is_empty() && p.from_schema_revision == p.to_schema_revision => {}
            "transactional" if !p.migrations.is_empty() => {}
            _ => return Err(Error::Release),
        }
        role(m, &p.source_backup, "backup_descriptor")?;
        role(m, &p.precondition, "update_precondition")?;
        role(m, &p.postcondition, "postcondition")?;
        let mut units = BTreeSet::new();
        let mut paths = BTreeSet::from([&p.precondition, &p.source_backup]);
        for u in &p.migrations {
            if crate::model::identifier(&u.id).is_err()
                || !units.insert(&u.id)
                || !paths.insert(&u.file)
                || !paths.insert(&u.postcondition)
                || !u.transactional
                || !u.file.starts_with(&prefix)
                || !u.file.ends_with(".sql")
                || !u.postcondition.starts_with(&prefix)
                || !u.postcondition.ends_with(".sql")
            {
                return Err(Error::Release);
            }
            role(m, &u.file, "update")?;
            role(m, &u.postcondition, "postcondition")?;
        }
    }
    Ok(())
}
pub fn referenced_file(m: &Manifest, path: &str, role: &str) -> bool {
    m.app_updates.iter().any(|p| match role {
        "backup_descriptor" => p.source_backup == path,
        "update_precondition" => p.precondition == path,
        "update" => p.migrations.iter().any(|u| u.file == path),
        "postcondition" => p.migrations.iter().any(|u| u.postcondition == path),
        _ => false,
    })
}
pub fn validate_files(m: &Manifest, files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    for p in &m.app_updates {
        descriptor(p, files)?;
        for path in std::iter::once(&p.precondition)
            .chain(std::iter::once(&p.postcondition))
            .chain(
                p.migrations
                    .iter()
                    .flat_map(|u| [&u.file, &u.postcondition]),
            )
        {
            let sql = std::str::from_utf8(files.get(path).ok_or(Error::Release)?)
                .map_err(|_| Error::Release)?;
            crate::sql_guard::validate_transactional(sql)?;
        }
    }
    Ok(())
}
fn sql_ident(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 63
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
pub fn descriptor(p: &Plan, files: &BTreeMap<String, Vec<u8>>) -> Result<SourceBackup> {
    let bytes = files.get(&p.source_backup).ok_or(Error::Release)?;
    if bytes.len() > 2_000_000 {
        return Err(Error::Release);
    }
    let d: SourceBackup = serde_json::from_slice(bytes).map_err(|_| Error::Release)?;
    if d.format != 1
        || d.native_ledger_contract != 1
        || d.from_manifest_sha256 != p.from_manifest_sha256
        || d.restore_baseline_manifest_sha256 != p.from_manifest_sha256
        || d.from_schema_revision != p.from_schema_revision
        || d.tables.is_empty()
        || d.tables.len() > 256
        || d.triggers.len() > 1024
    {
        return Err(Error::Release);
    }
    release::validate_path(&d.restore_baseline)?;
    release::validate_path(&d.restore_postcondition)?;
    let mut previous = "";
    for t in &d.tables {
        if t.schema != "public"
            || !sql_ident(&t.name)
            || t.name.as_str() <= previous
            || t.force_rls
            || t.columns.is_empty()
            || t.columns.len() > 256
        {
            return Err(Error::Release);
        }
        previous = &t.name;
        let mut columns = BTreeSet::new();
        for c in &t.columns {
            if !sql_ident(&c.name)
                || !columns.insert(&c.name)
                || c.sql_type.is_empty()
                || c.sql_type.len() > 128
                || !c.identity.is_empty()
                || !c.generated.is_empty()
            {
                return Err(Error::Release);
            }
        }
    }
    let mut previous = ("", "");
    for t in &d.triggers {
        if !sql_ident(&t.table)
            || !sql_ident(&t.name)
            || (t.table.as_str(), t.name.as_str()) <= previous
            || t.enabled != "O"
            || t.definition.len() > 16000
            || !d.tables.iter().any(|v| v.name == t.table)
        {
            return Err(Error::Release);
        }
        previous = (&t.table, &t.name);
    }
    Ok(d)
}
pub fn transition<'a>(old: &VerifiedRelease, new: &'a VerifiedRelease) -> Result<&'a Plan> {
    let p = new
        .manifest
        .app_updates
        .iter()
        .find(|p| p.from_manifest_sha256 == old.digest)
        .ok_or(Error::UpdateRefused)?;
    let a = &old.manifest;
    let b = &new.manifest;
    if p.from_schema_revision != a.schema.revision
        || p.to_schema_revision != b.schema.revision
        || a.configuration != b.configuration
        || a.google_scopes != b.google_scopes
        || a.bootstrap_contract != b.bootstrap_contract
        || a.health_contract != b.health_contract
        || a.install_command != b.install_command
        || a.build_command != b.build_command
        || a.output_directory != b.output_directory
        || b.sequence <= a.sequence
        || semver::Version::parse(&b.app_version).map_err(|_| Error::Release)?
            <= semver::Version::parse(&a.app_version).map_err(|_| Error::Release)?
    {
        return Err(Error::UpdateRefused);
    }
    let d = descriptor(p, &new.files)?;
    if new.files.get(&p.precondition)
        != old.files.get(
            &a.schema
                .migrations
                .last()
                .ok_or(Error::UpdateRefused)?
                .postcondition,
        )
    {
        return Err(Error::UpdateRefused);
    }
    // This recovery contract reconstructs exactly one authenticated fresh
    // baseline, then restores data and the historical native ledger separately.
    if a.schema.migrations.len() != 1
        || d.restore_baseline != a.schema.migrations[0].file
        || d.restore_postcondition != a.schema.migrations[0].postcondition
    {
        return Err(Error::UpdateRefused);
    }
    if p.kind == "code_only" {
        if a.schema.revision != b.schema.revision
            || a.schema.migrations.len() != b.schema.migrations.len()
        {
            return Err(Error::UpdateRefused);
        }
        for (x, y) in a.schema.migrations.iter().zip(&b.schema.migrations) {
            if x.id != y.id
                || x.prerequisite != y.prerequisite
                || old.files.get(&x.file) != new.files.get(&y.file)
                || old.files.get(&x.postcondition) != new.files.get(&y.postcondition)
            {
                return Err(Error::UpdateRefused);
            }
        }
    }
    Ok(p)
}
/// Canonical JSON: recursive object-key sort; arrays retain their signed order.
pub fn canonical_hash<T: Serialize>(value: &T) -> Result<String> {
    fn sorted(v: serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::Object(m) => serde_json::Value::Object(
                m.into_iter()
                    .map(|(k, v)| (k, sorted(v)))
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .collect(),
            ),
            serde_json::Value::Array(a) => {
                serde_json::Value::Array(a.into_iter().map(sorted).collect())
            }
            v => v,
        }
    }
    let v = sorted(serde_json::to_value(value).map_err(|_| Error::Release)?);
    Ok(release::hash(
        &serde_json::to_vec(&v).map_err(|_| Error::Release)?,
    ))
}
