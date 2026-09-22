use crate::error::{Error, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};
use url::Url;

pub const MAX_BUNDLE: u64 = 128 * 1024 * 1024;
pub const MAX_FILE: u64 = 32 * 1024 * 1024;
pub const CONFIG: &[(&str, &str)] = &[
    ("VITE_YOUTUBE_CLIENT_ID", "public"),
    ("YOUTUBE_CLIENT_SECRET", "secret"),
    ("VITE_SUPABASE_URL", "public"),
    ("VITE_SUPABASE_ANON_KEY", "public"),
    ("SUPABASE_SERVICE_KEY", "secret"),
    ("VITE_APP_URL", "public"),
    ("ENCRYPTION_KEY", "secret"),
    ("CRON_SECRET", "secret"),
    ("VILLOW_EXPECTED_OWNER_EMAIL", "server"),
    ("VILLOW_BOOTSTRAP_TOKEN_HASH", "server"),
    ("VILLOW_INSTALLATION_ID", "server"),
];

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trust {
    pub format: u32,
    pub repository: Option<String>,
    pub channel: String,
    pub manifest_url: Option<String>,
    pub public_keys: BTreeMap<String, String>,
    pub minimum_sequence: u64,
    pub publisher: Option<String>,
}
impl Trust {
    pub fn embedded() -> Result<Self> {
        serde_json::from_str(include_str!("../trust.json")).map_err(|_| Error::Unconfigured)
    }
    pub fn configured(&self) -> bool {
        self.format == 1
            && self
                .repository
                .as_ref()
                .is_some_and(|s| s.split('/').count() == 2 && !s.contains("example"))
            && self.manifest_url.is_some()
            && !self.public_keys.is_empty()
            && self
                .publisher
                .as_ref()
                .is_some_and(|s| !s.trim().is_empty())
    }
    pub fn artifact_url(&self, raw: &str) -> Result<Url> {
        let repo = self.repository.as_ref().ok_or(Error::Unconfigured)?;
        let url = Url::parse(raw).map_err(|_| Error::Release)?;
        if url.scheme() != "https"
            || url.host_str() != Some("github.com")
            || url.port().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !url
                .path()
                .starts_with(&format!("/{repo}/releases/download/"))
            || raw.contains('%')
            || raw.contains('\\')
            || raw.contains("/latest/")
        {
            return Err(Error::Release);
        }
        Ok(url)
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    pub format: u32,
    pub channel: String,
    pub sequence: u64,
    pub generated_at: String,
    pub expires_at: String,
    pub releases: Vec<ReleasePointer>,
    pub revoked: Vec<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasePointer {
    pub version: String,
    pub sha256: String,
    pub url: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub app_version: String,
    pub commit: String,
    pub released_at: String,
    pub sequence: u64,
    pub channel: String,
    pub minimum_manager: String,
    pub upgrade_from: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fresh_retry_from: Vec<String>,
    pub archive_url: String,
    pub archive_sha256: String,
    pub archive_size: u64,
    pub files: BTreeMap<String, FileSpec>,
    pub schema: Schema,
    pub configuration: BTreeMap<String, String>,
    pub google_scopes: Vec<String>,
    pub bootstrap_contract: u32,
    pub health_contract: u32,
    pub build_command: String,
    pub install_command: String,
    pub output_directory: String,
    pub backup_required: bool,
    pub downtime: String,
    pub notes: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileSpec {
    pub sha256: String,
    pub size: u64,
    pub role: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    pub revision: String,
    pub kind: String,
    pub migrations: Vec<Migration>,
    pub compatible_apps: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Migration {
    pub id: String,
    pub file: String,
    pub postcondition: String,
    pub prerequisite: Option<String>,
    pub transactional: bool,
}
// Only release authentication constructs this capability in production code.
#[derive(Clone)]
pub struct VerifiedRelease {
    pub manifest: Manifest,
    pub digest: String,
    pub files: BTreeMap<String, Vec<u8>>,
}
pub fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn verify_channel(
    bytes: &[u8],
    trust: &Trust,
    now: DateTime<Utc>,
) -> Result<(Channel, String)> {
    if !trust.configured() {
        return Err(Error::Unconfigured);
    }
    if bytes.len() > 1_000_000 {
        return Err(Error::Release);
    }
    let e: Envelope = serde_json::from_slice(bytes).map_err(|_| Error::Release)?;
    let key = STANDARD
        .decode(trust.public_keys.get(&e.key_id).ok_or(Error::Release)?)
        .map_err(|_| Error::Release)?;
    let key = VerifyingKey::from_bytes(&key.try_into().map_err(|_| Error::Release)?)
        .map_err(|_| Error::Release)?;
    let sig = STANDARD.decode(e.signature).map_err(|_| Error::Release)?;
    let signature = Signature::from_slice(&sig).map_err(|_| Error::Release)?;
    let payload = STANDARD.decode(e.payload).map_err(|_| Error::Release)?;
    key.verify_strict(&payload, &signature)
        .map_err(|_| Error::Release)?;
    let c: Channel = serde_json::from_slice(&payload).map_err(|_| Error::Release)?;
    let issued = DateTime::parse_from_rfc3339(&c.generated_at).map_err(|_| Error::Release)?;
    let expires = DateTime::parse_from_rfc3339(&c.expires_at).map_err(|_| Error::Release)?;
    if c.format != 1
        || c.channel != trust.channel
        || c.sequence < trust.minimum_sequence
        || issued > now + chrono::Duration::minutes(5)
        || expires <= now
        || expires <= issued
        || expires - issued > chrono::Duration::days(7)
        || c.releases.is_empty()
        || c.releases.len() > 32
    {
        return Err(Error::Release);
    }
    let mut seen = BTreeSet::new();
    for p in &c.releases {
        trust.artifact_url(&p.url)?;
        if !seen.insert(&p.sha256)
            || !is_hash(&p.sha256)
            || semver::Version::parse(&p.version).is_err()
        {
            return Err(Error::Release);
        }
    }
    Ok((c, hash(&payload)))
}
pub fn verify_bundle(
    manifest_bytes: &[u8],
    archive: &[u8],
    pointer: &ReleasePointer,
    channel: &Channel,
    trust: &Trust,
) -> Result<VerifiedRelease> {
    let digest = hash(manifest_bytes);
    if manifest_bytes.len() > 4_000_000
        || digest != pointer.sha256
        || channel.revoked.contains(&digest)
        || !channel
            .releases
            .iter()
            .any(|p| p.sha256 == digest && p.url == pointer.url)
    {
        return Err(Error::Release);
    }
    let manifest: Manifest = serde_json::from_slice(manifest_bytes).map_err(|_| Error::Release)?;
    validate_manifest(&manifest, trust)?;
    if manifest.app_version != pointer.version
        || manifest.fresh_retry_from.contains(&digest)
        || manifest.channel != channel.channel
        || archive.len() as u64 != manifest.archive_size
        || hash(archive) != manifest.archive_sha256
    {
        return Err(Error::Release);
    }
    let files = read_archive(archive, &manifest.files)?;
    for migration in &manifest.schema.migrations {
        crate::sql_guard::validate_transactional(
            std::str::from_utf8(files.get(&migration.file).ok_or(Error::Release)?)
                .map_err(|_| Error::Release)?,
        )?;
    }
    Ok(VerifiedRelease {
        manifest,
        digest,
        files,
    })
}
pub fn validate_manifest(m: &Manifest, trust: &Trust) -> Result<()> {
    let retry_sources: BTreeSet<_> = m.fresh_retry_from.iter().collect();
    if retry_sources.len() != m.fresh_retry_from.len()
        || retry_sources.len() > 8
        || retry_sources.iter().any(|digest| !is_hash(digest))
        || (!retry_sources.is_empty()
            && (m.schema.kind != "fresh_baseline" || !m.upgrade_from.is_empty()))
        || (!retry_sources.is_empty()
            && semver::Version::parse(&m.minimum_manager).map_err(|_| Error::Release)?
                < semver::Version::new(0, 1, 1))
    {
        return Err(Error::Release);
    }
    trust.artifact_url(&m.archive_url)?;
    if m.format != 1
        || m.bootstrap_contract != 1
        || m.health_contract != 1
        || m.sequence == 0
        || m.commit.len() != 40
        || !m.commit.bytes().all(|c| c.is_ascii_hexdigit())
        || semver::Version::parse(&m.minimum_manager).map_err(|_| Error::Release)?
            > semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
        || semver::Version::parse(&m.app_version).is_err()
        || DateTime::parse_from_rfc3339(&m.released_at).is_err()
        || m.archive_size > MAX_BUNDLE
        || !is_hash(&m.archive_sha256)
        || m.files.len() > 15000
        || m.schema.kind != "fresh_baseline"
        || m.schema.migrations.is_empty()
        || !m.upgrade_from.is_empty()
        || m.backup_required
        || m.install_command != "npm ci"
        || m.build_command != "npm run build"
        || m.output_directory != "dist"
    {
        return Err(Error::Release);
    }
    let config: BTreeMap<_, _> = CONFIG
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    if !semver::VersionReq::parse(&m.schema.compatible_apps)
        .map_err(|_| Error::Release)?
        .matches(&semver::Version::parse(&m.app_version).map_err(|_| Error::Release)?)
    {
        return Err(Error::Release);
    }
    if m.configuration != config {
        return Err(Error::Release);
    }
    let scopes: BTreeSet<_> = m.google_scopes.iter().map(String::as_str).collect();
    if scopes
        != BTreeSet::from([
            "https://www.googleapis.com/auth/youtube.force-ssl",
            "https://www.googleapis.com/auth/userinfo.email",
            "https://www.googleapis.com/auth/userinfo.profile",
        ])
    {
        return Err(Error::Release);
    }
    let mut previous: Option<&str> = None;
    let mut ids = BTreeSet::new();
    for migration in &m.schema.migrations {
        if !migration.transactional
            || !ids.insert(&migration.id)
            || migration.prerequisite.as_deref() != previous
            || crate::model::identifier(&migration.id).is_err()
        {
            return Err(Error::Release);
        }
        for (path, role) in [
            (&migration.file, "migration"),
            (&migration.postcondition, "postcondition"),
        ] {
            if m.files.get(path).is_none_or(|s| s.role != role) {
                return Err(Error::Release);
            }
        }
        previous = Some(&migration.id);
    }
    if previous != Some(m.schema.revision.as_str()) {
        return Err(Error::Release);
    }
    for name in [
        "package.json",
        "package-lock.json",
        "vercel.json",
        "public/sw.js",
    ] {
        if m.files.get(name).is_none_or(|f| f.role != "deploy") {
            return Err(Error::Release);
        }
    }
    for (path, f) in &m.files {
        validate_path(path)?;
        if f.size > MAX_FILE
            || !is_hash(&f.sha256)
            || !["deploy", "migration", "postcondition"].contains(&f.role.as_str())
        {
            return Err(Error::Release);
        }
        if f.role == "deploy" {
            if path.starts_with("migrations/") || path.ends_with(".sql") {
                return Err(Error::Release);
            }
        } else if !path.starts_with("migrations/") || !path.ends_with(".sql") {
            return Err(Error::Release);
        }
    }
    Ok(())
}
fn is_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub fn validate_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 240
        || path.starts_with('/')
        || path.contains(['\\', ':', '\0', '%'])
    {
        return Err(Error::Release);
    }
    for part in path.split('/') {
        let lower = part.to_ascii_lowercase();
        let stem = lower.split('.').next().unwrap_or("");
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || !part
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_.@() ".contains(&c))
            || [
                "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
                "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8",
                "lpt9",
            ]
            .contains(&stem)
            || lower.starts_with(".env")
            || [
                "villow-setup",
                "node_modules",
                ".git",
                ".github",
                ".vercel",
                ".npmrc",
                ".yarnrc",
                "target",
                "dist",
                "logs",
                "secrets",
            ]
            .contains(&lower.as_str())
            || [
                ".exe", ".dll", ".cmd", ".bat", ".ps1", ".sh", ".pfx", ".pem", ".key",
            ]
            .iter()
            .any(|ext| lower.ends_with(ext))
        {
            return Err(Error::Release);
        }
    }
    Ok(())
}
pub fn read_archive(
    bytes: &[u8],
    specs: &BTreeMap<String, FileSpec>,
) -> Result<BTreeMap<String, Vec<u8>>> {
    if bytes.len() as u64 > MAX_BUNDLE {
        return Err(Error::Release);
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| Error::Release)?;
    if zip.len() > 15000 {
        return Err(Error::Release);
    }
    let mut files = BTreeMap::new();
    let mut case_names = BTreeSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|_| Error::Release)?;
        let name = entry.name().to_string();
        // No filesystem extraction. Even directory records are omitted from the contract.
        validate_path(&name)?;
        if entry.is_dir()
            || entry
                .unix_mode()
                .is_some_and(|m| (m & 0o170000 != 0 && m & 0o170000 != 0o100000) || m & 0o111 != 0)
            || !case_names.insert(name.to_ascii_lowercase())
        {
            return Err(Error::Release);
        }
        let spec = specs.get(&name).ok_or(Error::Release)?;
        total = total.checked_add(entry.size()).ok_or(Error::Release)?;
        if entry.size() > MAX_FILE || entry.size() != spec.size || total > MAX_BUNDLE {
            return Err(Error::Release);
        }
        let mut data = Vec::new();
        (&mut entry)
            .take(MAX_FILE + 1)
            .read_to_end(&mut data)
            .map_err(|_| Error::Release)?;
        if data.len() as u64 != spec.size || hash(&data) != spec.sha256 {
            return Err(Error::Release);
        }
        files.insert(name, data);
    }
    if files.len() != specs.len() {
        return Err(Error::Release);
    }
    Ok(files)
}
