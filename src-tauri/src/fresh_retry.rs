//! Replacement of an authenticated release before ANY app unit has committed.
//! Caller holds the local operation lock; the database adapter holds the SQL lock.
use crate::{
    error::{Error, Result},
    model::*,
    release::VerifiedRelease,
    store::Store,
    vault::Vault,
};

pub fn validate(s: &Installation, old: &VerifiedRelease, new: &VerifiedRelease) -> Result<()> {
    s.assert_writable()?;
    if s.step != Step::Database
        || s.installed_repair.is_some()
        || s.deployment_id.is_some()
        || s.effects.keys().any(|key| {
            ![
                "create_vercel",
                "create_database",
                "reserve_origin",
                "migrate",
            ]
            .contains(&key.as_str())
        })
        || !s
            .effects
            .get("migrate")
            .is_some_and(|e| e.status != EffectStatus::Verified)
        || s.release_digest != old.digest
        || s.commit != old.manifest.commit
        || s.app_version != old.manifest.app_version
        || old.digest == new.digest
        || old.manifest.schema.kind != "fresh_baseline"
        || new.manifest.schema.kind != "fresh_baseline"
        || !new.manifest.upgrade_from.is_empty()
        || !new.manifest.fresh_retry_from.contains(&old.digest)
        || new.manifest.sequence < old.manifest.sequence
        || new.manifest.configuration != old.manifest.configuration
        || new.manifest.google_scopes != old.manifest.google_scopes
        || new.manifest.bootstrap_contract != old.manifest.bootstrap_contract
        || new.manifest.health_contract != old.manifest.health_contract
        || s.google_scopes != old.manifest.google_scopes
        || s.schema_revision != old.manifest.schema.revision
        || !s.google.as_ref().is_some_and(Google::ready_for_setup)
        || !s.selection()?.costs_acknowledged
    {
        return Err(Error::FreshRetryRefused);
    }
    s.project()?;
    s.database()?;
    s.origin()?;
    if let Some(intent) = &s.fresh_retry {
        if intent.from != old.digest || intent.to != new.digest {
            return Err(Error::FreshRetryRefused);
        }
    }
    Ok(())
}

pub fn execute(
    store: &Store,
    vault: &dyn Vault,
    mut s: Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    retarget: impl FnOnce(&Installation) -> Result<()>,
) -> Result<()> {
    validate(&s, old, new)?;
    for name in [
        "encryption_key",
        "bootstrap_token",
        "db_password",
        "google_secret",
        "vercel_token",
        "supabase_token",
    ] {
        vault.require(&s.id, name)?;
    }
    s.fresh_retry = Some(FreshRetryIntent {
        from: old.digest.clone(),
        to: new.digest.clone(),
    });
    s.updated_at = now();
    store.save(&s)?; // Durable intent precedes the database compare-and-swap.
    retarget(&s)?;
    s.release_digest = new.digest.clone();
    s.app_version = new.manifest.app_version.clone();
    s.commit = new.manifest.commit.clone();
    s.release_sequence = new.manifest.sequence;
    s.schema_revision = new.manifest.schema.revision.clone();
    s.google_scopes = new.manifest.google_scopes.clone();
    s.fresh_retry = None;
    s.effects
        .get_mut("migrate")
        .ok_or(Error::FreshRetryRefused)?
        .status = EffectStatus::Planned;
    s.updated_at = now();
    s.check(
        "release",
        "Corrected authenticated release selected before any app unit was installed",
    );
    // A failed local save retains the old pin plus intent; retry observes the
    // database's new pin and completes only after repeating every SQL guard.
    store.save(&s)
}
