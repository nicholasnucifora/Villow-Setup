use crate::{
    engine::Providers,
    error::{Error, Result},
    http::{Api, Provider},
    model::*,
    providers::LiveProviders,
    vault::Vault,
};
use reqwest::Method;
use serde_json::Value;

pub fn confirm(
    s: &mut Installation,
    api: &dyn Api,
    vault: &dyn Vault,
    provider: &str,
    id: &str,
    confirmation: &str,
) -> Result<()> {
    s.assert_writable()?;
    if confirmation != s.name || s.step != Step::Projects {
        return Err(Error::Precondition);
    }
    identifier(id)?;
    let (kind, effect, token_name, path) = match provider {
        "vercel" if s.vercel.is_none() => (
            Provider::Vercel,
            "create_vercel",
            "vercel_token",
            format!("/v9/projects/{id}"),
        ),
        "supabase" if s.database.is_none() => (
            Provider::Supabase,
            "create_database",
            "supabase_token",
            format!("/v1/projects/{id}"),
        ),
        _ => return Err(Error::Invalid),
    };
    let e = s.effects.get(effect).ok_or(Error::Precondition)?;
    if !matches!(
        e.status,
        EffectStatus::Executing | EffectStatus::NeedsReview
    ) {
        return Err(Error::Precondition);
    }
    LiveProviders { http: api, vault }.verify_targets(s)?;
    let token = vault.require(&s.id, token_name)?;
    let query = if kind == Provider::Vercel {
        vec![("teamId", s.selection()?.vercel_account.as_str())]
    } else {
        vec![]
    };
    let v = api.provider(kind, Method::GET, &path, &query, &token, None)?;
    let resource = validate_candidate(s, kind, id, &v, &e.started_at)?;
    if kind == Provider::Vercel {
        s.vercel = Some(resource)
    } else {
        s.database = Some(resource)
    }
    let e = s.effects.get_mut(effect).unwrap();
    e.status = EffectStatus::Verified;
    e.verified_at = Some(now());
    s.check("user","You identified an uncertain created project; its account, ID, name and creation time were checked");
    if s.vercel.is_some() && s.database.is_some() {
        s.step = Step::Origin
    }
    s.updated_at = now();
    Ok(())
}
pub fn validate_candidate(
    s: &Installation,
    kind: Provider,
    id: &str,
    v: &Value,
    started_at: &str,
) -> Result<Resource> {
    let (returned_id, account, created) = if kind == Provider::Vercel {
        (
            v["id"].as_str(),
            v["accountId"].as_str(),
            v["createdAt"]
                .as_i64()
                .and_then(chrono::DateTime::from_timestamp_millis),
        )
    } else {
        (
            v["ref"].as_str().or(v["id"].as_str()),
            v["organization_id"].as_str(),
            v["created_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&chrono::Utc)),
        )
    };
    let expected = if kind == Provider::Vercel {
        &s.selection()?.vercel_account
    } else {
        &s.selection()?.supabase_organization
    };
    let start = chrono::DateTime::parse_from_rfc3339(started_at).map_err(|_| Error::Invalid)?;
    let created = created.ok_or(Error::WrongTarget)?;
    if returned_id != Some(id)
        || account != Some(expected.as_str())
        || v["name"] != s.name
        || created < start - chrono::Duration::minutes(2)
        || created > chrono::Utc::now() + chrono::Duration::minutes(5)
    {
        return Err(Error::WrongTarget);
    }
    Ok(Resource {
        id: id.into(),
        account_id: expected.clone(),
        name: s.name.clone(),
        operation_id: s.operation_id.clone(),
        evidence: "owner_confirmed_after_uncertain_creation".into(),
    })
}
