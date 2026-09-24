//! Portable encrypted pre-repair package. Runtime paths are chosen by a native
//! save dialog, never by renderer IPC. No cloud or vault writes occur here.
use crate::{
    backup_database::{self, DatabaseSnapshot},
    backup_file,
    error::{Error, Result},
    model::{now, BackupReceipt, Installation},
    release::{self, Trust, VerifiedRelease},
    vault::Vault,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub const SECRETS: [&str; 8] = [
    "vercel_token",
    "supabase_token",
    "db_password",
    "google_secret",
    "encryption_key",
    "bootstrap_token",
    "anon_key",
    "service_key",
];
#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct Credential {
    pub name: String,
    pub value: String,
}
#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub format: u32,
    pub captured_at: String,
    pub repair_operation_id: String,
    pub destination_digest: String,
    pub checkpoint_json: String,
    pub credentials: Vec<Credential>,
    pub configured_environment: Vec<Credential>,
    pub channel_base64: String,
    pub manifest_base64: String,
    pub archive_base64: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub update_manifest_base64: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub update_archive_base64: String,
    pub database: DatabaseSnapshot,
}

struct BoundedJson(Zeroizing<Vec<u8>>);
impl Write for BoundedJson {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > backup_file::MAX_PAYLOAD_BYTES - self.0.len() {
            return Err(std::io::Error::other("Backup size limit"));
        }
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Package {
    pub fn encode(&self) -> Result<Zeroizing<Vec<u8>>> {
        let mut writer = BoundedJson(Zeroizing::new(Vec::new()));
        serde_json::to_writer(&mut writer, self).map_err(|_| Error::BackupTooLarge)?;
        Ok(writer.0)
    }
}

/// Authentication at capture time is recorded as provenance, not future write
/// authorization. Restore must check current trust/revocations independently.
pub fn save(
    path: &Path,
    password: &str,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    trust: &Trust,
    channel_bytes: &[u8],
    manifest: &[u8],
    archive: &[u8],
    vault: &dyn Vault,
    managed: bool,
    capture: impl FnOnce() -> Result<DatabaseSnapshot>,
) -> Result<BackupReceipt> {
    backup_file::validate_password(password)?;
    crate::installed_repair::validate(s, old, new)?;
    if s.installed_repair.is_some()
        || old.digest != backup_database::SOURCE
        || new.digest != backup_database::DESTINATION
    {
        return Err(Error::BackupDatabase);
    }
    save_package(
        path,
        password,
        s,
        old,
        new,
        trust,
        channel_bytes,
        manifest,
        archive,
        vault,
        managed,
        None,
        capture,
    )
}
pub fn save_update(
    path: &Path,
    password: &str,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    trust: &Trust,
    channel_bytes: &[u8],
    manifest: &[u8],
    archive: &[u8],
    update_manifest: &[u8],
    update_archive: &[u8],
    vault: &dyn Vault,
    capture: impl FnOnce() -> Result<DatabaseSnapshot>,
) -> Result<BackupReceipt> {
    crate::app_update_engine::validate(s, old, new)?;
    if s.app_update.is_some() || s.update_lineage.is_none() {
        return Err(Error::UpdateRefused);
    }
    save_package(
        path,
        password,
        s,
        old,
        new,
        trust,
        channel_bytes,
        manifest,
        archive,
        vault,
        true,
        Some((update_manifest, update_archive)),
        capture,
    )
}
fn save_package(
    path: &Path,
    password: &str,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    trust: &Trust,
    channel_bytes: &[u8],
    manifest: &[u8],
    archive: &[u8],
    vault: &dyn Vault,
    managed: bool,
    update: Option<(&[u8], &[u8])>,
    capture: impl FnOnce() -> Result<DatabaseSnapshot>,
) -> Result<BackupReceipt> {
    backup_file::validate_password(password)?;
    let (channel, _) = release::verify_channel(channel_bytes, trust, chrono::Utc::now())?;
    let pointer = channel
        .releases
        .iter()
        .find(|r| r.sha256 == old.digest)
        .ok_or(Error::Release)?;
    let authenticated = release::verify_bundle(manifest, archive, pointer, &channel, trust)?;
    if authenticated.digest != old.digest
        || manifest.len() > 4_000_000
        || archive.len() > 32 * 1024 * 1024
    {
        return Err(Error::BackupTooLarge);
    }
    if let Some((m, a)) = update {
        let pointer = channel
            .releases
            .iter()
            .find(|r| r.sha256 == new.digest)
            .ok_or(Error::Release)?;
        let verified = release::verify_bundle(m, a, pointer, &channel, trust)?;
        crate::update_contract::transition(&authenticated, &verified)?;
        if m.len() > 4_000_000 || a.len() > 32 * 1024 * 1024 {
            return Err(Error::BackupTooLarge);
        }
    }
    let mut credentials = Vec::new();
    for key in SECRETS {
        let value = vault.require(&s.id, key)?;
        if value.len() > 16_384 {
            return Err(Error::BackupTooLarge);
        }
        credentials.push(Credential {
            name: key.to_owned(),
            value: value.to_string(),
        });
    }
    let captured_at = now();
    let operation = uuid::Uuid::new_v4().to_string();
    // Preserve the exact values Setup configured, including its existing cron
    // derivation. This does not claim to export manual provider-console edits.
    let secret = |key: &str| -> Result<String> {
        credentials
            .iter()
            .find(|c| c.name == key)
            .map(|c| c.value.clone())
            .ok_or(Error::MissingCredential)
    };
    let encryption = Zeroizing::new(secret("encryption_key")?);
    let google = s.google.as_ref().ok_or(Error::Precondition)?;
    let configured_environment = [
        ("VITE_YOUTUBE_CLIENT_ID", google.client_id.clone()),
        ("YOUTUBE_CLIENT_SECRET", secret("google_secret")?),
        (
            "VITE_SUPABASE_URL",
            format!("https://{}.supabase.co", s.database()?.id),
        ),
        ("VITE_SUPABASE_ANON_KEY", secret("anon_key")?),
        ("SUPABASE_SERVICE_KEY", secret("service_key")?),
        ("VITE_APP_URL", s.origin()?.to_string()),
        ("ENCRYPTION_KEY", encryption.to_string()),
        (
            "CRON_SECRET",
            release::hash(format!("villow-cron:{}", encryption.as_str()).as_bytes()),
        ),
        ("VILLOW_EXPECTED_OWNER_EMAIL", s.owner_email.clone()),
        (
            "VILLOW_BOOTSTRAP_TOKEN_HASH",
            release::hash(Zeroizing::new(secret("bootstrap_token")?).as_bytes()),
        ),
        ("VILLOW_INSTALLATION_ID", s.id.clone()),
    ]
    .into_iter()
    .map(|(name, value)| Credential {
        name: name.into(),
        value,
    })
    .collect();
    let package = Package {
        format: if update.is_some() { 2 } else { 1 },
        captured_at: captured_at.clone(),
        repair_operation_id: operation.clone(),
        destination_digest: new.digest.clone(),
        checkpoint_json: serde_json::to_string(s).map_err(|_| Error::BackupInvalid)?,
        credentials,
        configured_environment,
        channel_base64: STANDARD.encode(channel_bytes),
        manifest_base64: STANDARD.encode(manifest),
        archive_base64: STANDARD.encode(archive),
        update_manifest_base64: update.map(|(m, _)| STANDARD.encode(m)).unwrap_or_default(),
        update_archive_base64: update.map(|(_, a)| STANDARD.encode(a)).unwrap_or_default(),
        database: capture()?,
    };
    let saved = if managed {
        backup_file::write_verified_managed(path, package.encode()?, password)?
    } else {
        backup_file::write_verified(path, package.encode()?, password)?
    };
    // write_verified durably closes, reopens, decrypts and compares ALL plaintext
    // bytes, including COPY counts, columns, every row and the original vault.
    Ok(BackupReceipt {
        managed,
        removed_at: None,
        path: saved.path,
        sha256: saved.sha256,
        bytes: saved.bytes,
        captured_at,
        installation_id: s.id.clone(),
        operation_id: operation,
        from: old.digest.clone(),
        to: new.digest.clone(),
    })
}
pub fn check_receipt(
    receipt: &BackupReceipt,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
) -> Result<()> {
    if receipt.installation_id != s.id
        || receipt.removed_at.is_some()
        || receipt.from != old.digest
        || receipt.to != new.digest
        || uuid::Uuid::parse_str(&receipt.operation_id).is_err()
        || chrono::DateTime::parse_from_rfc3339(&receipt.captured_at).is_err()
    {
        return Err(Error::RepairBackupRequired);
    }
    backup_file::matches_saved(Path::new(&receipt.path), &receipt.sha256, receipt.bytes)
}
