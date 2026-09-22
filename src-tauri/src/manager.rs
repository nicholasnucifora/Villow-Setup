use crate::{
    engine::{Engine, Providers},
    error::{Error, Result},
    http::Http,
    model::*,
    providers::LiveProviders,
    release::{self, Trust, VerifiedRelease},
    store::Store,
    vault::{OsVault, Vault},
};
use serde::Serialize;
use std::path::Path;

pub struct Manager {
    pub store: Store,
    pub trust: Trust,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub manager_version: String,
    pub trust_configured: bool,
    pub installation: Option<Installation>,
    pub release: Option<release::Manifest>,
    pub release_checked_at: Option<String>,
    pub message: String,
    pub release_digest: Option<String>,
    pub fresh_retry: Option<FreshRetryOffer>,
    pub installed_repair: Option<InstalledRepairOffer>,
}
#[derive(Serialize)]
pub struct InstalledRepairOffer {
    pub digest: String,
    pub app_version: String,
}
#[derive(Serialize)]
pub struct FreshRetryOffer {
    pub digest: String,
    pub app_version: String,
}
impl Manager {
    pub fn open(path: &Path) -> Result<Self> {
        let manager = Self {
            store: Store::open(path)?,
            trust: Trust::embedded()?,
        };
        // A crash after verified completion may leave only local cleanup. It
        // needs no provider calls and must not make a working app look broken.
        if let Ok(_lock) = manager.store.lock() {
            if let Some(mut s) = manager.store.load()? {
                let _ = crate::managed_backup::cleanup(&manager.store, &OsVault, &mut s);
            }
        }
        Ok(manager)
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        Ok(Snapshot { manager_version: env!("CARGO_PKG_VERSION").into(), trust_configured: self.trust.configured(),
            installation: self.store.load()?, release: None, release_checked_at: None, release_digest: None, fresh_retry: None, installed_repair: None,
            message: if self.trust.configured() { "Release status has not been checked this session." } else {
                "The official signed Villow release and publisher are not configured. Cloud setup is unavailable; you can read the account guide without creating accounts or tokens."
            }.into() })
    }
    pub fn release(&self, pinned: Option<&str>) -> Result<VerifiedRelease> {
        let channel = self.channel()?;
        self.release_in_channel(&channel, pinned)
    }
    fn channel(&self) -> Result<release::Channel> {
        self.channel_document().map(|(channel, _)| channel)
    }
    fn channel_document(&self) -> Result<(release::Channel, Vec<u8>)> {
        if !self.trust.configured() {
            return Err(Error::Unconfigured);
        }
        let http = Http::new()?;
        let url = url::Url::parse(
            self.trust
                .manifest_url
                .as_deref()
                .ok_or(Error::Unconfigured)?,
        )
        .map_err(|_| Error::Unconfigured)?;
        let channel_bytes = http.release_bytes(url, 1_000_000)?;
        let (channel, channel_digest) =
            release::verify_channel(&channel_bytes, &self.trust, chrono::Utc::now())?;
        self.store
            .accept_release(channel.sequence, &channel_digest)?;
        Ok((channel, channel_bytes))
    }
    fn release_in_channel(
        &self,
        channel: &release::Channel,
        pinned: Option<&str>,
    ) -> Result<VerifiedRelease> {
        let http = Http::new()?;
        let pointer = if let Some(digest) = pinned {
            channel
                .releases
                .iter()
                .find(|p| p.sha256 == digest)
                .ok_or(Error::Release)?
        } else {
            channel.releases.first().ok_or(Error::Release)?
        };
        if channel.revoked.contains(&pointer.sha256) {
            return Err(Error::Release);
        }
        // Cache is only an availability/performance hint. Authentication is repeated.
        let cache = self.store.root().join("release-cache");
        std::fs::create_dir_all(&cache).map_err(|_| Error::Storage)?;
        let manifest_path = cache.join(format!("{}.json", pointer.sha256));
        let archive_path = cache.join(format!("{}.zip", pointer.sha256));
        if let (Ok(manifest), Ok(archive)) =
            (std::fs::read(&manifest_path), std::fs::read(&archive_path))
        {
            if let Ok(r) =
                release::verify_bundle(&manifest, &archive, pointer, &channel, &self.trust)
            {
                return Ok(r);
            }
        }
        let manifest = http.release_bytes(self.trust.artifact_url(&pointer.url)?, 4_000_000)?;
        // Authenticate the manifest BEFORE using its download URL.
        if release::hash(&manifest) != pointer.sha256 {
            return Err(Error::Release);
        }
        let m: release::Manifest = serde_json::from_slice(&manifest).map_err(|_| Error::Release)?;
        release::validate_manifest(&m, &self.trust)?;
        let archive =
            http.release_bytes(self.trust.artifact_url(&m.archive_url)?, m.archive_size)?;
        let verified = release::verify_bundle(&manifest, &archive, pointer, &channel, &self.trust)?;
        std::fs::write(manifest_path, &manifest).map_err(|_| Error::Storage)?;
        std::fs::write(archive_path, &archive).map_err(|_| Error::Storage)?;
        Ok(verified)
    }
    pub fn check_release(&self) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let s = self.store.load()?;
        let r = self.release(s.as_ref().map(|s| s.release_digest.as_str()))?;
        let mut snapshot = self.snapshot()?;
        snapshot.release_digest = Some(r.digest);
        snapshot.release = Some(r.manifest);
        snapshot.release_checked_at = Some(now());
        snapshot.message = "Release signature, archive and migration checksums verified.".into();
        Ok(snapshot)
    }
    pub fn check_fresh_retry(&self) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        let channel = self.channel()?;
        let old = self.release_in_channel(&channel, Some(&s.release_digest))?;
        let new = self.release_in_channel(
            &channel,
            s.fresh_retry.as_ref().map(|intent| intent.to.as_str()),
        )?;
        if s.fresh_retry.is_none() && !new.manifest.fresh_retry_from.contains(&old.digest) {
            let mut snapshot = self.snapshot()?;
            snapshot.message = "No authenticated correction is available for this unfinished setup yet. Keep your saved setup and wait for the release update.".into();
            return Ok(snapshot);
        }
        crate::fresh_retry::validate(&s, &old, &new)?;
        let http = Http::new()?;
        LiveProviders {
            http: &http,
            vault: &OsVault,
        }
        .verify_targets(&s)?;
        let mut snapshot = self.snapshot()?;
        snapshot.fresh_retry = Some(FreshRetryOffer {
            digest: new.digest,
            app_version: new.manifest.app_version,
        });
        Ok(snapshot)
    }
    pub fn use_fresh_retry(&self, digest: String) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        let channel = self.channel()?;
        let old = self.release_in_channel(&channel, Some(&s.release_digest))?;
        let new = self.release_in_channel(&channel, Some(&digest))?;
        crate::fresh_retry::validate(&s, &old, &new)?;
        let http = Http::new()?;
        let providers = LiveProviders {
            http: &http,
            vault: &OsVault,
        };
        providers.verify_targets(&s)?;
        crate::fresh_retry::execute(&self.store, &OsVault, s, &old, &new, |state| {
            let connection = providers.database_connection(state)?;
            let mut client = crate::migration::connect(state, &OsVault, &connection)?;
            crate::migration::retarget_unapplied(&mut client, state, &old, &new)
        })?;
        let mut snapshot = self.snapshot()?;
        snapshot.message = "Corrected release saved. Your accounts and credentials are retained. Click Prepare my database to continue.".into();
        Ok(snapshot)
    }
    pub fn start(&self, name: String, email: String, digest: String) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        if self.store.load()?.is_some() {
            return Err(Error::Precondition);
        }
        let release = self.release(Some(&digest))?;
        let state = Installation::new(name, email, &release)?;
        self.store.save(&state)?;
        self.snapshot()
    }
    pub fn check_installed_repair(&self) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        if s.step != Step::Health || s.installed_repair.is_some() {
            return Err(Error::RepairRefused);
        }
        let channel = self.channel()?;
        let old = self.release_in_channel(&channel, Some(&s.release_digest))?;
        let new = self.release_in_channel(&channel, None)?;
        if !new
            .manifest
            .installed_repairs
            .iter()
            .any(|r| r.from_manifest_sha256 == old.digest)
        {
            let mut snapshot = self.snapshot()?;
            snapshot.message = "No signed repair is available for this installed Alpha yet. Keep your saved setup and existing database.".into();
            return Ok(snapshot);
        }
        crate::installed_repair::validate(&s, &old, &new)?;
        crate::installed_repair::require_credentials(&s, &OsVault)?;
        let http = Http::new()?;
        let providers = LiveProviders {
            http: &http,
            vault: &OsVault,
        };
        providers.verify_targets(&s)?;
        if providers.deployment_status(&s, &old)? != DeploymentStatus::Ready {
            return Err(Error::DeploymentNotReady);
        }
        let connection = providers.database_connection(&s)?;
        let mut client = crate::migration::connect(&s, &OsVault, &connection)?;
        crate::repair_database::check_or_apply(&mut client, &s, &old, &new, false)?;
        let mut snapshot = self.snapshot()?;
        snapshot.installed_repair = Some(InstalledRepairOffer {
            digest: new.digest,
            app_version: new.manifest.app_version,
        });
        snapshot.message = "Signed repair verified against your installed database and owner. Setup will save and check a backup before applying it.".into();
        Ok(snapshot)
    }
    pub fn apply_installed_repair(&self, digest: String) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        let channel = self.channel()?;
        let old = self.release_in_channel(&channel, Some(&s.release_digest))?;
        // Never follow a new recommended release when resuming saved intent.
        let new = self.release_in_channel(&channel, Some(&digest))?;
        let http = Http::new()?;
        let providers = LiveProviders {
            http: &http,
            vault: &OsVault,
        };
        crate::installed_repair::advance(
            &self.store,
            &OsVault,
            &providers,
            s,
            &old,
            &new,
            None,
            |state, apply| {
                let connection = providers.database_connection(state)?;
                let mut client = crate::migration::connect(state, &OsVault, &connection)?;
                crate::repair_database::check_or_apply(&mut client, state, &old, &new, apply)
            },
        )?;
        self.snapshot()
    }
    pub fn backup_and_repair(&self, digest: String) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        let (channel, channel_bytes) = self.channel_document()?;
        let old = self.release_in_channel(&channel, Some(&s.release_digest))?;
        let new = self.release_in_channel(&channel, Some(&digest))?;
        crate::installed_repair::validate(&s, &old, &new)?;
        if s.installed_repair.is_some() {
            return Err(Error::RepairPending);
        }
        let http = Http::new()?;
        let providers = LiveProviders {
            http: &http,
            vault: &OsVault,
        };
        providers.verify_targets(&s)?;
        if providers.deployment_status(&s, &old)? != DeploymentStatus::Ready {
            return Err(Error::DeploymentNotReady);
        }
        let cache = self.store.root().join("release-cache");
        let manifest = std::fs::read(cache.join(format!("{}.json", old.digest)))
            .map_err(|_| Error::Storage)?;
        let archive =
            std::fs::read(cache.join(format!("{}.zip", old.digest))).map_err(|_| Error::Storage)?;
        let (path, password) =
            crate::managed_backup::prepare(&self.store, &OsVault, &s, &new.digest)?;
        let backup = crate::repair_backup::save(
            &path,
            &password,
            &s,
            &old,
            &new,
            &self.trust,
            &channel_bytes,
            &manifest,
            &archive,
            &OsVault,
            true,
            || {
                let connection = providers.database_connection(&s)?;
                let mut client = crate::migration::connect(&s, &OsVault, &connection)?;
                crate::backup_database::capture(&mut client, &s, &old)
            },
        )?;
        crate::installed_repair::advance(
            &self.store,
            &OsVault,
            &providers,
            s,
            &old,
            &new,
            Some(backup),
            |state, apply| {
                let connection = providers.database_connection(state)?;
                let mut client = crate::migration::connect(state, &OsVault, &connection)?;
                crate::repair_database::check_or_apply(&mut client, state, &old, &new, apply)
            },
        )?;
        self.snapshot()
    }
    pub fn credentials(&self, vercel: String, supabase: String) -> Result<Snapshot> {
        use zeroize::Zeroizing;
        let vercel = Zeroizing::new(vercel);
        let supabase = Zeroizing::new(supabase);
        let _lock = self.store.lock()?;
        let mut s = self.store.load()?.ok_or(Error::Precondition)?;
        if s.read_only {
            return Err(Error::RecoveryReadOnly);
        }
        // Reauthentication cannot change the recorded account selection.
        if !vercel.is_empty() {
            OsVault.put(&s.id, "vercel_token", &vercel)?;
        }
        if !supabase.is_empty() {
            OsVault.put(&s.id, "supabase_token", &supabase)?;
        }
        s.credentials_removed = false;
        self.store.save(&s)?;
        self.snapshot()
    }
    pub fn accounts(&self) -> Result<Accounts> {
        let _lock = self.store.lock()?;
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        LiveProviders {
            http: &Http::new()?,
            vault: &OsVault,
        }
        .accounts(&s)
    }
    pub fn select(&self, selection: Selection) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let mut s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        if !s.effects.is_empty()
            || s.selection.is_some()
            || !selection.costs_acknowledged
            || !["ap-southeast-2", "us-east-1", "eu-west-1", "ap-southeast-1"]
                .contains(&selection.region.as_str())
        {
            return Err(Error::Precondition);
        }
        for value in [
            &selection.vercel_user,
            &selection.supabase_user,
            &selection.vercel_account,
            &selection.supabase_organization,
            &selection.supabase_slug,
        ] {
            identifier(value)?;
        }
        s.selection = Some(selection);
        LiveProviders {
            http: &Http::new()?,
            vault: &OsVault,
        }
        .verify_targets(&s)?;
        self.store.save(&s)?;
        self.snapshot()
    }
    pub fn google(&self, google: Google, secret: String) -> Result<Snapshot> {
        let secret = zeroize::Zeroizing::new(secret);
        let _lock = self.store.lock()?;
        let mut s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        if s.step != Step::Google
            || google.client_id.len() > 200
            || !google.client_id.ends_with(".apps.googleusercontent.com")
            || !google
                .client_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
        {
            return Err(Error::Invalid);
        }
        identifier(&google.project_id)?;
        OsVault.put(&s.id, "google_secret", &secret)?;
        s.google = Some(google);
        self.store.save(&s)?;
        self.snapshot()
    }
    pub fn database_connection(
        &self,
        connection: DbConnection,
        password: String,
    ) -> Result<Snapshot> {
        let password = zeroize::Zeroizing::new(password);
        let _lock = self.store.lock()?;
        let mut s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        if s.step != Step::Database {
            return Err(Error::Precondition);
        }
        crate::migration::validate_connection(&s.database()?.id, &connection)?;
        if !password.is_empty() {
            OsVault.put(&s.id, "db_password", &password)?;
        }
        s.db_connection = Some(connection);
        self.store.save(&s)?;
        self.snapshot()
    }
    pub fn reconcile_created(
        &self,
        provider: String,
        id: String,
        confirmation: String,
    ) -> Result<Snapshot> {
        let _lock = self.store.lock()?;
        let mut s = self.store.load()?.ok_or(Error::Precondition)?;
        crate::reconcile::confirm(
            &mut s,
            &Http::new()?,
            &OsVault,
            &provider,
            &id,
            &confirmation,
        )?;
        self.store.save(&s)?;
        self.snapshot()
    }
    pub fn advance(&self) -> Result<Snapshot> {
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        let r = self.release(Some(&s.release_digest))?;
        Engine {
            store: &self.store,
            vault: &OsVault,
            providers: &LiveProviders {
                http: &Http::new()?,
                vault: &OsVault,
            },
        }
        .advance(&r)?;
        self.snapshot()
    }
    pub fn check_deployment(&self) -> Result<Snapshot> {
        let s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        let release = self.release(Some(&s.release_digest))?;
        Engine {
            store: &self.store,
            vault: &OsVault,
            providers: &LiveProviders {
                http: &Http::new()?,
                vault: &OsVault,
            },
        }
        .check_deployment(&release)?;
        self.snapshot()
    }
    pub fn browser_url(&self, step: &str) -> Result<String> {
        let fixed = match step {
            "vercel_signup" => Some("https://vercel.com/signup"),
            "vercel_token" => Some("https://vercel.com/account/settings/tokens"),
            "supabase_signup" => Some("https://supabase.com/dashboard/sign-up"),
            "supabase_token" => Some("https://supabase.com/dashboard/account/tokens"),
            "google_project" => Some("https://console.cloud.google.com/projectcreate"),
            "google_api" => {
                Some("https://console.cloud.google.com/apis/library/youtube.googleapis.com")
            }
            "google_audience" => Some("https://console.cloud.google.com/auth/audience"),
            "google_branding" => Some("https://console.cloud.google.com/auth/branding"),
            "google_scopes" => Some("https://console.cloud.google.com/auth/scopes"),
            "google_client" => Some("https://console.cloud.google.com/auth/clients"),
            "google_dashboard" => Some("https://console.cloud.google.com/"),
            "vercel_dashboard" => Some("https://vercel.com/dashboard"),
            "supabase_dashboard" => Some("https://supabase.com/dashboard/projects"),
            _ => None,
        };
        if let Some(url) = fixed {
            return Ok(url.into());
        }
        if step == "app" {
            let s = self.store.load()?.ok_or(Error::Precondition)?;
            if s.repair_pending() {
                let p = s.installed_repair.as_ref().ok_or(Error::RepairRefused)?;
                if s.read_only
                    || p.phase != RepairPhase::Verify
                    || p.deployment_status != Some(DeploymentStatus::Ready)
                {
                    return Err(Error::DeploymentNotReady);
                }
                crate::http::validate_origin(s.origin()?)?;
                return Ok(s.origin()?.into());
            }
            if s.read_only
                || !(s.step == Step::Complete
                    || (s.step == Step::Health
                        && s.deployment_status == Some(DeploymentStatus::Ready)))
            {
                return Err(Error::Precondition);
            }
            crate::http::validate_origin(s.origin()?)?;
            return Ok(s.origin()?.into());
        }
        Err(Error::Invalid)
    }
}
