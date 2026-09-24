#![allow(dead_code)]
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signer, SigningKey};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    io::{Cursor, Write},
    path::PathBuf,
};
use villow_setup::{
    engine::Providers,
    error::{Error, Result},
    model::*,
    release::*,
    vault::Vault,
};
use zeroize::Zeroizing;
pub fn update_fixture() -> (VerifiedRelease, VerifiedRelease, Installation) {
    let (old, _, mut s) = repair_fixture();
    let mut new = old.clone();
    new.manifest.app_version = "1.0.1".into();
    new.manifest.minimum_manager = "0.2.0".into();
    new.manifest.sequence = old.manifest.sequence + 1;
    new.manifest.schema.compatible_apps = "=1.0.1".into();
    let plan = villow_setup::update_contract::Plan {
        id: "test-update".into(),
        from_manifest_sha256: old.digest.clone(),
        from_schema_revision: old.manifest.schema.revision.clone(),
        to_schema_revision: old.manifest.schema.revision.clone(),
        kind: "code_only".into(),
        source_backup: "updates/test-update/source-backup.json".into(),
        precondition: "updates/test-update/source.sql".into(),
        migrations: vec![],
        postcondition: old.manifest.schema.migrations[0].postcondition.clone(),
        backup_required: true,
        previous_app_compatible: true,
    };
    let descriptor = serde_json::json!({"format":1,"from_manifest_sha256":old.digest,"from_schema_revision":old.manifest.schema.revision,"restore_baseline_manifest_sha256":old.digest,"restore_baseline":old.manifest.schema.migrations[0].file,"restore_postcondition":old.manifest.schema.migrations[0].postcondition,"native_ledger_contract":1,"tables":[{"schema":"public","name":"synthetic","columns":[{"name":"id","type":"integer","identity":"","generated":""}],"rls":false,"force_rls":false}],"triggers":[]});
    new.files.insert(
        plan.source_backup.clone(),
        serde_json::to_vec(&descriptor).unwrap(),
    );
    new.files.insert(
        plan.precondition.clone(),
        old.files[&old.manifest.schema.migrations[0].postcondition].clone(),
    );
    for (p, role) in [
        (&plan.source_backup, "backup_descriptor"),
        (&plan.precondition, "update_precondition"),
    ] {
        new.manifest.files.insert(
            p.clone(),
            FileSpec {
                sha256: String::new(),
                size: 0,
                role: role.into(),
            },
        );
    }
    new.manifest.upgrade_from = vec![old.digest.clone()];
    new.manifest.app_updates = vec![plan];
    new = reseal(new);
    s.step = Step::Complete;
    s.update_lineage = Some(villow_setup::update_database::initial(&s, &old, None).unwrap());
    (old, new, s)
}
pub fn pending_update(s: &mut Installation, new: &VerifiedRelease) {
    s.app_update = Some(RepairIntent {
        from: s.release_digest.clone(),
        to: new.digest.clone(),
        repair_id: new.manifest.app_updates[0].id.clone(),
        operation_id: uuid::Uuid::new_v4().to_string(),
        backup_confirmed_at: now(),
        backup: None,
        previous_operation_id: s.operation_id.clone(),
        previous_deployment_id: s.deployment_id.clone().unwrap(),
        phase: RepairPhase::Database,
        deployment_id: None,
        deployment_status: None,
    });
}
pub fn fixtures() -> (Trust, Vec<u8>, Vec<u8>, Vec<u8>) {
    let signing = SigningKey::from_bytes(&[42; 32]); // Synthetic test key, never shipped as a trust root.
    let trust = Trust {
        format: 1,
        repository: Some("test-owner/test-releases".into()),
        channel: "stable".into(),
        manifest_url: Some(
            "https://github.com/test-owner/test-releases/releases/download/channel/stable.json"
                .into(),
        ),
        public_keys: BTreeMap::from([(
            "test".into(),
            STANDARD.encode(signing.verifying_key().to_bytes()),
        )]),
        minimum_sequence: 1,
        publisher: Some("Synthetic test publisher".into()),
    };
    let files = BTreeMap::from([
        ("package.json","{\"scripts\":{\"build\":\"vite build\"}}"), ("package-lock.json","{\"lockfileVersion\":3}"),
        ("vercel.json","{\"crons\":[{\"path\":\"/api/sync\",\"schedule\":\"0 17 * * *\"}]}"), ("public/sw.js","const APP_SHELL_VERSION = 'test';"),
        ("src/main.ts","document.body.textContent = 'synthetic app';"),
        ("migrations/baseline.sql","CREATE TABLE public.synthetic (id integer PRIMARY KEY, value text NOT NULL); ALTER TABLE public.synthetic ENABLE ROW LEVEL SECURITY; CREATE POLICY synthetic_private ON public.synthetic FOR SELECT TO authenticated USING (value = 'visible'); REVOKE ALL ON public.synthetic FROM PUBLIC, anon; GRANT USAGE ON SCHEMA public TO authenticated; GRANT SELECT ON public.synthetic TO authenticated; CREATE FUNCTION public.synthetic_label(s text) RETURNS text LANGUAGE sql IMMUTABLE AS $$ SELECT upper(s); $$; REVOKE ALL ON FUNCTION public.synthetic_label(text) FROM PUBLIC; GRANT EXECUTE ON FUNCTION public.synthetic_label(text) TO authenticated; INSERT INTO public.synthetic VALUES (0, 'reference');"),
        ("migrations/verify.sql","SELECT EXISTS (SELECT FROM pg_class WHERE relname='synthetic' AND relrowsecurity) AND EXISTS (SELECT FROM pg_policies WHERE schemaname='public' AND tablename='synthetic' AND policyname='synthetic_private') AND has_table_privilege('authenticated','public.synthetic','SELECT') AND NOT has_table_privilege('anon','public.synthetic','SELECT') AND to_regprocedure('public.synthetic_label(text)') IS NOT NULL AND EXISTS (SELECT FROM public.synthetic WHERE id=0 AND value='reference');"),
    ]);
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut specs = BTreeMap::new();
    for (path, bytes) in files {
        zip.start_file(
            path,
            zip::write::SimpleFileOptions::default().unix_permissions(0o644),
        )
        .unwrap();
        zip.write_all(bytes.as_bytes()).unwrap();
        specs.insert(
            path.into(),
            FileSpec {
                sha256: hash(bytes.as_bytes()),
                size: bytes.len() as u64,
                role: if path.ends_with("baseline.sql") {
                    "migration"
                } else if path.ends_with("verify.sql") {
                    "postcondition"
                } else {
                    "deploy"
                }
                .into(),
            },
        );
    }
    let archive = zip.finish().unwrap().into_inner();
    let m = Manifest {
        format: 1,
        app_version: "1.0.0".into(),
        commit: "1".repeat(40),
        released_at: "2026-09-09T00:00:00Z".into(),
        sequence: 1,
        channel: "stable".into(),
        minimum_manager: "0.1.0".into(),
        upgrade_from: vec![],
        fresh_retry_from: vec![],
        installed_repairs: vec![],
        app_updates: vec![],
        archive_url: "https://github.com/test-owner/test-releases/releases/download/v1.0.0/app.zip"
            .into(),
        archive_sha256: hash(&archive),
        archive_size: archive.len() as u64,
        files: specs,
        schema: Schema {
            revision: "baseline-1".into(),
            kind: "fresh_baseline".into(),
            migrations: vec![Migration {
                id: "baseline-1".into(),
                file: "migrations/baseline.sql".into(),
                postcondition: "migrations/verify.sql".into(),
                prerequisite: None,
                transactional: true,
            }],
            compatible_apps: "=1.0.0".into(),
        },
        configuration: CONFIG
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        google_scopes: vec![
            "https://www.googleapis.com/auth/youtube.force-ssl".into(),
            "https://www.googleapis.com/auth/userinfo.email".into(),
            "https://www.googleapis.com/auth/userinfo.profile".into(),
        ],
        bootstrap_contract: 1,
        health_contract: 1,
        build_command: "npm run build".into(),
        install_command: "npm ci".into(),
        output_directory: "dist".into(),
        backup_required: false,
        downtime: "fresh install".into(),
        notes: "Synthetic fixture; no app data".into(),
    };
    let manifest = serde_json::to_vec(&m).unwrap();
    let channel = Channel {
        format: 1,
        channel: "stable".into(),
        sequence: 4,
        generated_at: "2026-09-09T00:00:00Z".into(),
        expires_at: "2026-09-12T00:00:00Z".into(),
        releases: vec![ReleasePointer {
            version: "1.0.0".into(),
            sha256: hash(&manifest),
            url:
                "https://github.com/test-owner/test-releases/releases/download/v1.0.0/manifest.json"
                    .into(),
        }],
        revoked: vec![],
    };
    (trust, sign_channel(&channel), manifest, archive)
}
pub fn sign_channel(c: &Channel) -> Vec<u8> {
    let payload = serde_json::to_vec(c).unwrap();
    let key = SigningKey::from_bytes(&[42; 32]);
    serde_json::to_vec(&Envelope {
        key_id: "test".into(),
        signature: STANDARD.encode(key.sign(&payload).to_bytes()),
        payload: STANDARD.encode(payload),
    })
    .unwrap()
}
pub fn verified() -> VerifiedRelease {
    let (trust, index, manifest, archive) = fixtures();
    let (channel, _) =
        verify_channel(&index, &trust, "2026-09-10T00:00:00Z".parse().unwrap()).unwrap();
    verify_bundle(&manifest, &archive, &channel.releases[0], &channel, &trust).unwrap()
}
pub fn installation(r: &VerifiedRelease) -> Installation {
    let mut s = Installation::new("test-villow".into(), "owner@example.test".into(), r).unwrap();
    s.selection = Some(Selection {
        vercel_user: "u1".into(),
        supabase_user: "s1".into(),
        vercel_account: "team_1".into(),
        supabase_organization: "org1".into(),
        supabase_slug: "org1".into(),
        region: "ap-southeast-2".into(),
        costs_acknowledged: true,
    });
    s
}
pub fn fresh_retry_fixture() -> (VerifiedRelease, VerifiedRelease, Installation) {
    let old = verified();
    let (trust, _, _, archive) = fixtures();
    let mut manifest = old.manifest.clone();
    manifest.app_version = "1.0.1".into();
    manifest.schema.compatible_apps = "=1.0.1".into();
    manifest.minimum_manager = "0.1.1".into();
    manifest.sequence = 2;
    manifest.fresh_retry_from = vec![old.digest.clone()];
    let bytes = serde_json::to_vec(&manifest).unwrap();
    let channel = Channel {
        format: 1,
        channel: "stable".into(),
        sequence: 5,
        generated_at: "2026-09-09T00:00:00Z".into(),
        expires_at: "2026-09-12T00:00:00Z".into(),
        releases: vec![ReleasePointer {
            version: "1.0.1".into(),
            sha256: hash(&bytes),
            url:
                "https://github.com/test-owner/test-releases/releases/download/v1.0.1/manifest.json"
                    .into(),
        }],
        revoked: vec![],
    };
    let (channel, _) = verify_channel(
        &sign_channel(&channel),
        &trust,
        "2026-09-10T00:00:00Z".parse().unwrap(),
    )
    .unwrap();
    let new = verify_bundle(&bytes, &archive, &channel.releases[0], &channel, &trust).unwrap();
    let mut s = installation(&old);
    s.step = Step::Database;
    s.vercel = Some(Resource {
        id: "prj_test".into(),
        account_id: "team_1".into(),
        name: s.name.clone(),
        operation_id: s.operation_id.clone(),
        evidence: "synthetic".into(),
    });
    s.database = Some(Resource {
        id: "abcdefghijklmnopqrst".into(),
        account_id: "org1".into(),
        name: s.name.clone(),
        operation_id: s.operation_id.clone(),
        evidence: "synthetic".into(),
    });
    s.origin = Some("https://test-villow.vercel.app".into());
    s.google = Some(Google {
        project_id: "synthetic-google".into(),
        client_id: "synthetic.apps.googleusercontent.com".into(),
        api_enabled_confirmed: true,
        audience: "external_testing".into(),
        consent_published_confirmed: false,
        testing_access_confirmed: true,
    });
    s.effects.insert(
        "migrate".into(),
        Effect {
            status: EffectStatus::NeedsReview,
            started_at: now(),
            verified_at: None,
        },
    );
    (old, new, s)
}
#[derive(Default)]
pub struct MemoryVault {
    pub values: RefCell<BTreeMap<String, String>>,
    pub fail: Cell<bool>,
}
impl Vault for MemoryVault {
    fn get(&self, id: &str, name: &str) -> Result<Option<Zeroizing<String>>> {
        if self.fail.get() {
            return Err(Error::Vault);
        };
        Ok(self
            .values
            .borrow()
            .get(&format!("{id}/{name}"))
            .cloned()
            .map(Zeroizing::new))
    }
    fn put(&self, id: &str, name: &str, secret: &str) -> Result<()> {
        if self.fail.get() {
            return Err(Error::Vault);
        };
        self.values
            .borrow_mut()
            .insert(format!("{id}/{name}"), secret.into());
        Ok(())
    }
    fn delete(&self, id: &str, name: &str) -> Result<()> {
        if self.fail.get() {
            return Err(Error::Vault);
        };
        self.values.borrow_mut().remove(&format!("{id}/{name}"));
        Ok(())
    }
}
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Cloud {
    pub vercel: Option<Resource>,
    pub database: Option<Resource>,
    pub creates: u32,
    pub migrations: u32,
    pub deployment: Option<String>,
    #[serde(default)]
    pub deploys: u32,
}
pub struct Fake {
    pub path: PathBuf,
    pub error: RefCell<Option<Error>>,
    pub crash: Cell<bool>,
    pub before: Cell<bool>,
    pub health_ok: Cell<bool>,
    pub deployment_status: Cell<DeploymentStatus>,
    pub deployment_lost_response: Cell<bool>,
}
impl Fake {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            error: RefCell::new(None),
            crash: Cell::new(false),
            before: Cell::new(false),
            health_ok: Cell::new(true),
            deployment_status: Cell::new(DeploymentStatus::Ready),
            deployment_lost_response: Cell::new(false),
        }
    }
    pub fn read(&self) -> Cloud {
        std::fs::read(&self.path)
            .ok()
            .and_then(|x| serde_json::from_slice(&x).ok())
            .unwrap_or_default()
    }
    fn save(&self, c: &Cloud) {
        std::fs::write(&self.path, serde_json::to_vec(c).unwrap()).unwrap()
    }
    fn create(&self, s: &Installation, db: bool) -> Result<Resource> {
        assert!(
            !self.before.replace(false),
            "injected crash before remote effect"
        );
        let r = Resource {
            id: if db { "abcdefghijklmnopqrst" } else { "prj_1" }.into(),
            account_id: if db { "org1" } else { "team_1" }.into(),
            name: s.name.clone(),
            operation_id: s.operation_id.clone(),
            evidence: "fake_remote_operation_id".into(),
        };
        let mut c = self.read();
        c.creates += 1;
        if db {
            c.database = Some(r.clone())
        } else {
            c.vercel = Some(r.clone())
        }
        self.save(&c);
        assert!(
            !self.crash.replace(false),
            "injected crash after remote effect"
        );
        if let Some(error) = self.error.borrow_mut().take() {
            return Err(error);
        };
        Ok(r)
    }
}
impl Providers for Fake {
    fn verify_targets(&self, s: &Installation) -> Result<()> {
        if s.selection()?.vercel_account != "team_1" {
            return Err(Error::WrongTarget);
        }
        if let Some(e) = self.error.borrow().as_ref() {
            if *e != Error::Uncertain {
                return Err(e.clone());
            }
        }
        Ok(())
    }
    fn create_vercel(&self, s: &Installation) -> Result<Resource> {
        self.create(s, false)
    }
    fn create_database(&self, s: &Installation) -> Result<Resource> {
        self.create(s, true)
    }
    fn reconcile_vercel(&self, s: &Installation) -> Result<Option<Resource>> {
        Ok(self
            .read()
            .vercel
            .filter(|r| r.operation_id == s.operation_id))
    }
    fn reconcile_database(&self, s: &Installation) -> Result<Option<Resource>> {
        Ok(self
            .read()
            .database
            .filter(|r| r.operation_id == s.operation_id))
    }
    fn reserve_origin(&self, s: &Installation) -> Result<String> {
        Ok(format!("https://{}.vercel.app", s.name))
    }
    fn migrate(&self, _s: &Installation, _r: &VerifiedRelease) -> Result<()> {
        let mut c = self.read();
        if c.migrations == 0 {
            c.migrations += 1;
            self.save(&c)
        }
        if self.crash.replace(false) {
            return Err(Error::Uncertain);
        }
        Ok(())
    }
    fn configure(&self, _s: &Installation, _r: &VerifiedRelease) -> Result<()> {
        Ok(())
    }
    fn upload(&self, _s: &Installation, _r: &VerifiedRelease) -> Result<()> {
        if self.crash.replace(false) {
            Err(Error::Offline)
        } else {
            Ok(())
        }
    }
    fn deploy(&self, _s: &Installation, _r: &VerifiedRelease, reconcile: bool) -> Result<String> {
        let mut c = self.read();
        if reconcile {
            return c.deployment.ok_or(Error::Uncertain);
        }
        c.deployment = Some("dpl_1".into());
        c.deploys += 1;
        self.save(&c);
        if self.deployment_lost_response.replace(false) {
            return Err(Error::Uncertain);
        }
        Ok("dpl_1".into())
    }
    fn health(&self, _s: &Installation, _r: &VerifiedRelease) -> Result<()> {
        if self.health_ok.get() {
            Ok(())
        } else {
            Err(Error::Health)
        }
    }
    fn deployment_status(
        &self,
        _s: &Installation,
        _r: &VerifiedRelease,
    ) -> Result<DeploymentStatus> {
        Ok(self.deployment_status.get())
    }
}

// Sign every mutated fixture again: no production trust or release bytes change.
pub fn reseal(mut r: VerifiedRelease) -> VerifiedRelease {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in &r.files {
        zip.start_file(path, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
        let spec = r.manifest.files.get_mut(path).unwrap();
        spec.sha256 = hash(bytes);
        spec.size = bytes.len() as u64;
    }
    let archive = zip.finish().unwrap().into_inner();
    r.manifest.archive_size = archive.len() as u64;
    r.manifest.archive_sha256 = hash(&archive);
    let manifest = serde_json::to_vec(&r.manifest).unwrap();
    let (trust, _, _, _) = fixtures();
    let channel = Channel { format: 1, channel: "stable".into(), sequence: 6,
        generated_at: "2026-09-09T00:00:00Z".into(), expires_at: "2026-09-12T00:00:00Z".into(),
        releases: vec![ReleasePointer { version: r.manifest.app_version.clone(), sha256: hash(&manifest),
            url: "https://github.com/test-owner/test-releases/releases/download/correction/manifest.json".into() }], revoked: vec![] };
    let (channel, _) = verify_channel(
        &sign_channel(&channel),
        &trust,
        "2026-09-10T00:00:00Z".parse().unwrap(),
    )
    .unwrap();
    verify_bundle(&manifest, &archive, &channel.releases[0], &channel, &trust).unwrap()
}
pub fn repair_fixture() -> (VerifiedRelease, VerifiedRelease, Installation) {
    let (mut old, _, mut s) = fresh_retry_fixture();
    old.manifest.schema.revision = "villow-fresh-158".into();
    old.manifest.schema.migrations[0].id = "villow-fresh-158".into();
    old.files.get_mut("migrations/baseline.sql").unwrap().extend_from_slice(b"CREATE TABLE public.users(id uuid PRIMARY KEY, email text NOT NULL, google_id text NOT NULL, is_system_owner boolean NOT NULL, access_revoked_at timestamptz, encrypted_token text NOT NULL); CREATE TABLE public.user_settings(user_id uuid PRIMARY KEY REFERENCES public.users(id)); CREATE TABLE public.villow_installation(singleton boolean PRIMARY KEY, installation_id uuid NOT NULL, expected_owner_email text NOT NULL, owner_id uuid NOT NULL REFERENCES public.users(id), owner_email text NOT NULL, owner_google_id text NOT NULL, bootstrap_closed_at timestamptz, youtube_verified_at timestamptz);");
    old.files.insert("migrations/verify.sql".into(), b"SELECT to_regclass('public.synthetic') IS NOT NULL AND NOT EXISTS(SELECT FROM information_schema.columns WHERE table_schema='public' AND table_name='user_settings' AND column_name='daily_watch_time_seconds');".to_vec());
    old = reseal(old);
    s.release_digest = old.digest.clone();
    s.schema_revision = old.manifest.schema.revision.clone();
    s.step = Step::Health;
    s.deployment_id = Some("dpl_original".into());
    s.deployment_status = Some(DeploymentStatus::Ready);
    for key in [
        "create_vercel",
        "create_database",
        "reserve_origin",
        "migrate",
        "configure",
        "upload",
        "deploy",
    ] {
        s.effects.insert(
            key.into(),
            Effect {
                status: EffectStatus::Verified,
                started_at: now(),
                verified_at: Some(now()),
            },
        );
    }
    let mut new = old.clone();
    new.manifest.app_version = "1.0.1".into();
    new.manifest.commit = "2".repeat(40);
    new.manifest.minimum_manager = "0.1.3".into();
    new.manifest.sequence = 2;
    new.manifest.schema.compatible_apps = "=1.0.1".into();
    new.manifest.schema.revision = "villow-fresh-159".into();
    new.manifest.schema.migrations[0].id = "villow-fresh-159".into();
    new.manifest.schema.migrations[0].postcondition = "migrations/postcondition.sql".into();
    let patch = b"ALTER TABLE public.user_settings ADD COLUMN daily_watch_time_seconds integer NOT NULL DEFAULT 0; ALTER TABLE public.user_settings ADD COLUMN daily_watch_reset_at timestamptz;".to_vec();
    new.files
        .get_mut("migrations/baseline.sql")
        .unwrap()
        .extend_from_slice(&patch);
    new.files.insert("migrations/repair-159.sql".into(), patch);
    new.files.insert("migrations/postcondition.sql".into(), b"SELECT EXISTS(SELECT FROM information_schema.columns WHERE table_schema='public' AND table_name='user_settings' AND column_name='daily_watch_time_seconds' AND data_type='integer' AND is_nullable='NO') AND EXISTS(SELECT FROM information_schema.columns WHERE table_schema='public' AND table_name='user_settings' AND column_name='daily_watch_reset_at');".to_vec());
    for (path, role) in [
        ("migrations/repair-159.sql", "repair"),
        ("migrations/postcondition.sql", "postcondition"),
    ] {
        new.manifest.files.insert(
            path.into(),
            FileSpec {
                sha256: String::new(),
                size: 0,
                role: role.into(),
            },
        );
    }
    new.manifest.installed_repairs = vec![InstalledRepair {
        id: "villow-installed-159".into(),
        from_manifest_sha256: old.digest.clone(),
        from_schema_revision: "villow-fresh-158".into(),
        file: "migrations/repair-159.sql".into(),
        postcondition: "migrations/postcondition.sql".into(),
        transactional: true,
        backup_required: true,
    }];
    (old, reseal(new), s)
}
pub fn repair_vault(s: &Installation) -> MemoryVault {
    let vault = MemoryVault::default();
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
        vault.put(&s.id, name, &format!("SENTINEL-{name}")).unwrap();
    }
    vault
}
pub fn pending_repair(s: &mut Installation, new: &VerifiedRelease) {
    s.installed_repair = Some(RepairIntent {
        from: s.release_digest.clone(),
        to: new.digest.clone(),
        repair_id: "villow-installed-159".into(),
        operation_id: uuid::Uuid::new_v4().to_string(),
        backup_confirmed_at: now(),
        backup: None,
        previous_operation_id: s.operation_id.clone(),
        previous_deployment_id: s.deployment_id.clone().unwrap(),
        phase: RepairPhase::Database,
        deployment_id: None,
        deployment_status: None,
    });
}

pub fn backup_fixture(
    dir: &std::path::Path,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
) -> BackupReceipt {
    let path = dir.join(format!("synthetic-{}.villowbackup", uuid::Uuid::new_v4()));
    let bytes = b"synthetic file receipt test only; crypto is tested separately";
    std::fs::write(&path, bytes).unwrap();
    BackupReceipt {
        managed: false,
        removed_at: None,
        path: path.to_str().unwrap().into(),
        sha256: hash(bytes),
        bytes: bytes.len() as u64,
        captured_at: now(),
        installation_id: s.id.clone(),
        operation_id: uuid::Uuid::new_v4().to_string(),
        from: old.digest.clone(),
        to: new.digest.clone(),
    }
}
