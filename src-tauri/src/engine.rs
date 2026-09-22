use crate::{
    error::{Error, Result},
    model::*,
    release::VerifiedRelease,
    store::Store,
    vault::Vault,
};

pub trait Providers {
    fn verify_targets(&self, s: &Installation) -> Result<()>;
    fn create_vercel(&self, s: &Installation) -> Result<Resource>;
    fn create_database(&self, s: &Installation) -> Result<Resource>;
    fn reconcile_vercel(&self, s: &Installation) -> Result<Option<Resource>>;
    fn reconcile_database(&self, s: &Installation) -> Result<Option<Resource>>;
    fn reserve_origin(&self, s: &Installation) -> Result<String>;
    fn migrate(&self, s: &Installation, r: &VerifiedRelease) -> Result<()>;
    fn configure(&self, s: &Installation, r: &VerifiedRelease) -> Result<()>;
    fn upload(&self, s: &Installation, r: &VerifiedRelease) -> Result<()>;
    fn deploy(&self, s: &Installation, r: &VerifiedRelease, reconcile: bool) -> Result<String>;
    fn health(&self, s: &Installation, r: &VerifiedRelease) -> Result<()>;
}
pub struct Engine<'a> {
    pub store: &'a Store,
    pub vault: &'a dyn Vault,
    pub providers: &'a dyn Providers,
}
impl Engine<'_> {
    pub fn advance(&self, release: &VerifiedRelease) -> Result<Installation> {
        let _lock = self.store.lock()?;
        let mut s = self.store.load()?.ok_or(Error::Precondition)?;
        s.assert_writable()?;
        if s.fresh_retry.is_some() {
            return Err(Error::FreshRetryPending);
        }
        if s.release_digest != release.digest
            || s.commit != release.manifest.commit
            || s.app_version != release.manifest.app_version
        {
            return Err(Error::Release);
        }
        if s.step == Step::Complete {
            return Ok(s);
        }
        if !s.selection()?.costs_acknowledged {
            return Err(Error::Precondition);
        }
        self.providers.verify_targets(&s)?;
        if s.effects.is_empty() {
            for name in ["encryption_key", "bootstrap_token", "db_password"] {
                self.vault.ensure_random(&s.id, name)?;
            }
        } else {
            // Never silently rotate a key after any remote effect may have used it.
            for name in ["encryption_key", "bootstrap_token", "db_password"] {
                self.vault.require(&s.id, name)?;
            }
        }
        match s.step {
            Step::Projects => {
                if s.vercel.is_none() {
                    let uncertain = self.begin(&mut s, "create_vercel")?;
                    let result = if uncertain {
                        self.providers
                            .reconcile_vercel(&s)
                            .and_then(|r| r.ok_or(Error::Uncertain))
                    } else {
                        self.providers.create_vercel(&s)
                    };
                    match result {
                        Ok(r) => s.vercel = Some(r),
                        Err(e) => return self.failed(&mut s, "create_vercel", e),
                    }
                    self.finish(&mut s, "create_vercel")?;
                    s.check(
                        "provider",
                        "Hosting project belongs to the selected account",
                    );
                } else if s.database.is_none() {
                    let uncertain = self.begin(&mut s, "create_database")?;
                    let result = if uncertain {
                        self.providers
                            .reconcile_database(&s)
                            .and_then(|r| r.ok_or(Error::Uncertain))
                    } else {
                        self.providers.create_database(&s)
                    };
                    match result {
                        Ok(r) => s.database = Some(r),
                        Err(e) => return self.failed(&mut s, "create_database", e),
                    }
                    self.finish(&mut s, "create_database")?;
                    s.check(
                        "provider",
                        "Database project belongs to the selected organization",
                    );
                }
                if s.vercel.is_some() && s.database.is_some() {
                    s.step = Step::Origin;
                }
            }
            Step::Origin => {
                self.begin(&mut s, "reserve_origin")?;
                match self.providers.reserve_origin(&s) {
                    Ok(origin) => s.origin = Some(origin),
                    Err(e) => return self.failed(&mut s, "reserve_origin", e),
                }
                self.finish(&mut s, "reserve_origin")?;
                s.check(
                    "provider",
                    "Permanent production address is assigned to your project",
                );
                s.step = Step::Google;
            }
            Step::Google => {
                let g = s.google.as_ref().ok_or(Error::Precondition)?;
                if !g.ready_for_setup() {
                    return Err(Error::Precondition);
                }
                let google_check = if g.audience == "external_testing" {
                    "Google External Testing confirmed: test users and seven-day access limit acknowledged"
                } else {
                    "Google API, audience and publishing settings confirmed by you"
                };
                self.vault.require(&s.id, "google_secret")?;
                s.check("user", google_check);
                s.step = Step::Database;
            }
            Step::Database => {
                self.begin(&mut s, "migrate")?;
                if let Err(e) = self.providers.migrate(&s, release) {
                    return self.failed(&mut s, "migrate", e);
                }
                self.finish(&mut s, "migrate")?;
                s.check(
                    "database",
                    "Signed schema and postconditions verified under a database lock",
                );
                s.step = Step::Configuration;
            }
            Step::Configuration => {
                self.begin(&mut s, "configure")?;
                if let Err(e) = self.providers.configure(&s, release) {
                    return self.failed(&mut s, "configure", e);
                }
                self.finish(&mut s, "configure")?;
                s.check(
                    "provider",
                    "Production configuration accepted by your hosting project",
                );
                s.step = Step::Deployment;
            }
            Step::Deployment => {
                // Content-addressed uploads can be repeated. Keep their
                // boundary separate from an uncertain deployment POST.
                if !s
                    .effects
                    .get("deploy")
                    .is_some_and(|e| e.status != EffectStatus::Planned)
                {
                    self.begin(&mut s, "upload")?;
                    if let Err(e) = self.providers.upload(&s, release) {
                        return self.failed(&mut s, "upload", e);
                    }
                    self.finish(&mut s, "upload")?;
                }
                let uncertain = self.begin(&mut s, "deploy")?;
                match self.providers.deploy(&s, release, uncertain) {
                    Ok(id) => s.deployment_id = Some(id),
                    Err(e) => return self.failed(&mut s, "deploy", e),
                }
                self.finish(&mut s, "deploy")?;
                s.step = Step::Health;
            }
            Step::Health => {
                if let Err(e) = self.providers.health(&s, release) {
                    return Err(e);
                }
                s.check(
                    "provider",
                    "Deployment reached READY on your production address",
                );
                s.check("app","App release, schema, intended owner, Google access and cron configuration verified");
                s.step = Step::Complete;
            }
            Step::Complete => {}
        }
        s.updated_at = now();
        self.store.save(&s)?;
        Ok(s)
    }
    fn begin(&self, s: &mut Installation, name: &str) -> Result<bool> {
        let uncertain = s
            .effects
            .get(name)
            .is_some_and(|e| e.status != EffectStatus::Planned);
        s.effects.entry(name.into()).or_insert(Effect {
            status: EffectStatus::Planned,
            started_at: now(),
            verified_at: None,
        });
        // Save intent and then execution as separate durable boundaries.
        self.store.save(s)?;
        s.effects.get_mut(name).unwrap().status = EffectStatus::Executing;
        self.store.save(s)?;
        Ok(uncertain)
    }
    fn finish(&self, s: &mut Installation, name: &str) -> Result<()> {
        let e = s.effects.get_mut(name).ok_or(Error::Storage)?;
        e.status = EffectStatus::Verified;
        e.verified_at = Some(now());
        // Resource ID and completion evidence commit in the same checkpoint.
        self.store.save(s)
    }
    fn failed<T>(&self, s: &mut Installation, name: &str, error: Error) -> Result<T> {
        let status = match error {
            Error::Authentication | Error::RateLimited => EffectStatus::Planned,
            _ => EffectStatus::NeedsReview,
        };
        s.effects.get_mut(name).ok_or(Error::Storage)?.status = status;
        s.updated_at = now();
        self.store.save(s)?;
        Err(error)
    }
}

pub fn remove_credentials(store: &Store, vault: &dyn Vault) -> Result<()> {
    let _lock = store.lock()?;
    let mut s = store.load()?.ok_or(Error::Precondition)?;
    vault.remove_all(&s.id)?;
    s.credentials_removed = true;
    store.save(&s)
}
pub fn forget(store: &Store, vault: &dyn Vault, confirmation: &str) -> Result<()> {
    let _lock = store.lock()?;
    let s = store.load()?.ok_or(Error::Precondition)?;
    if confirmation != s.name {
        return Err(Error::Invalid);
    }
    vault.remove_all(&s.id)?;
    store.forget()
}
