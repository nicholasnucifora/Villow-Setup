use crate::{
    error::{Error, Result},
    model::{Installation, Step},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recovery {
    pub format: u32,
    pub purpose: String,
    pub installation: Installation,
    pub instructions: Vec<String>,
}
pub fn export(s: &Installation) -> Result<String> {
    let mut installation = s.clone();
    installation.read_only = true;
    // Never carry credentials or inferred write authorization in an export.
    installation.effects.clear();
    installation.fresh_retry = None;
    installation.deployment_status = None;
    installation.credentials_removed = true;
    for r in [&mut installation.vercel, &mut installation.database]
        .into_iter()
        .flatten()
    {
        r.evidence = "untrusted_recovery_hint".into();
    }
    serde_json::to_string_pretty(&Recovery { format: 1, purpose: "Villow Setup nonsecret recovery".into(), installation,
        instructions: vec![
            "This file contains account and resource identifiers and your owner email; review before sharing.".into(),
            "Sign in to Vercel, Supabase and Google Cloud using your own accounts. Never give this file management credentials.".into(),
            "Importing restores a read-only resource inventory. It does not authorize modifying or deleting any resource.".into(),
            "Your cloud application continues when the local manager is removed. Provider billing remains separate.".into(),
            "If local encryption credentials are lost, recover the existing ENCRYPTION_KEY from the original Vercel project. Never generate a replacement for existing data.".into(),
        ] }).map_err(|_| Error::Storage)
}
pub fn import(text: &str) -> Result<Installation> {
    if text.len() > 100_000 {
        return Err(Error::Invalid);
    }
    let mut r: Recovery = serde_json::from_str(text).map_err(|_| Error::Invalid)?;
    if r.format != 1
        || r.installation.format != 1
        || uuid::Uuid::parse_str(&r.installation.id).is_err()
        || !crate::model::valid_email(&r.installation.owner_email)
        || r.installation.name.len() > 64
    {
        return Err(Error::Invalid);
    }
    crate::model::identifier(&r.installation.name)?;
    if let Some(origin) = &r.installation.origin {
        crate::http::validate_origin(origin)?;
    }
    for resource in [&r.installation.vercel, &r.installation.database]
        .into_iter()
        .flatten()
    {
        crate::model::identifier(&resource.id)?;
    }
    r.installation.read_only = true;
    r.installation.credentials_removed = true;
    r.installation.effects.clear();
    r.installation.fresh_retry = None;
    r.installation.deployment_status = None;
    r.installation.checks.clear(); // Imported assertions never render as verified checks.
    if r.installation.step == Step::Complete {
        r.installation.step = Step::Health;
    }
    Ok(r.installation)
}
