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
}
pub struct Fake {
    pub path: PathBuf,
    pub error: RefCell<Option<Error>>,
    pub crash: Cell<bool>,
    pub before: Cell<bool>,
    pub health_ok: Cell<bool>,
}
impl Fake {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            error: RefCell::new(None),
            crash: Cell::new(false),
            before: Cell::new(false),
            health_ok: Cell::new(true),
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
        self.save(&c);
        Ok("dpl_1".into())
    }
    fn health(&self, _s: &Installation, _r: &VerifiedRelease) -> Result<()> {
        if self.health_ok.get() {
            Ok(())
        } else {
            Err(Error::Health)
        }
    }
}
