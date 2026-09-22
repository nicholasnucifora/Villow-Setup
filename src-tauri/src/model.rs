use crate::error::{Error, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub fn now() -> String {
    Utc::now().to_rfc3339()
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Projects,
    Origin,
    Google,
    Database,
    Configuration,
    Deployment,
    Health,
    Complete,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Queued,
    Building,
    AssigningAddress,
    Ready,
    Failed,
    Canceled,
    AddressFailed,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectStatus {
    Planned,
    Executing,
    Verified,
    NeedsReview,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub status: EffectStatus,
    pub started_at: String,
    pub verified_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub operation_id: String,
    pub evidence: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub slug: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Accounts {
    pub vercel_user: String,
    pub supabase_user: String,
    pub vercel: Vec<Account>,
    pub supabase: Vec<Account>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub vercel_user: String,
    pub supabase_user: String,
    pub vercel_account: String,
    pub supabase_organization: String,
    pub supabase_slug: String,
    pub region: String,
    pub costs_acknowledged: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Google {
    pub project_id: String,
    pub client_id: String,
    pub api_enabled_confirmed: bool,
    pub audience: String,
    pub consent_published_confirmed: bool,
    #[serde(default)]
    pub testing_access_confirmed: bool,
}
impl Google {
    pub fn ready_for_setup(&self) -> bool {
        self.api_enabled_confirmed
            && match self.audience.as_str() {
                "external_testing" => {
                    self.testing_access_confirmed && !self.consent_published_confirmed
                }
                "external_production" | "internal" => self.consent_published_confirmed,
                _ => false,
            }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DbConnection {
    pub host: String,
    pub user: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub kind: String,
    pub title: String,
    pub at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FreshRetryIntent {
    pub from: String,
    pub to: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installation {
    pub format: u32,
    pub id: String,
    pub operation_id: String,
    pub name: String,
    pub owner_email: String,
    pub release_digest: String,
    pub app_version: String,
    pub commit: String,
    pub release_sequence: u64,
    pub schema_revision: String,
    pub google_scopes: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub step: Step,
    pub selection: Option<Selection>,
    pub vercel: Option<Resource>,
    pub database: Option<Resource>,
    pub origin: Option<String>,
    pub google: Option<Google>,
    pub db_connection: Option<DbConnection>,
    pub deployment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_status: Option<DeploymentStatus>,
    pub effects: BTreeMap<String, Effect>,
    pub checks: Vec<Check>,
    pub read_only: bool,
    pub credentials_removed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fresh_retry: Option<FreshRetryIntent>,
}
impl Installation {
    pub fn new(
        name: String,
        owner_email: String,
        release: &crate::release::VerifiedRelease,
    ) -> Result<Self> {
        if name.is_empty()
            || name.len() > 32
            || !name
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || !name.as_bytes()[0].is_ascii_alphanumeric()
            || !valid_email(&owner_email)
        {
            return Err(Error::Invalid);
        }
        let id = Uuid::new_v4().to_string();
        Ok(Self {
            format: 1,
            id: id.clone(),
            operation_id: Uuid::new_v4().to_string(),
            name: format!("{}-{}", name, &id[..8]),
            owner_email: owner_email.to_lowercase(),
            release_digest: release.digest.clone(),
            app_version: release.manifest.app_version.clone(),
            commit: release.manifest.commit.clone(),
            release_sequence: release.manifest.sequence,
            google_scopes: release.manifest.google_scopes.clone(),
            schema_revision: release.manifest.schema.revision.clone(),
            created_at: now(),
            updated_at: now(),
            step: Step::Projects,
            selection: None,
            vercel: None,
            database: None,
            origin: None,
            google: None,
            db_connection: None,
            deployment_id: None,
            deployment_status: None,
            effects: BTreeMap::new(),
            checks: vec![],
            read_only: false,
            credentials_removed: false,
            fresh_retry: None,
        })
    }
    pub fn selection(&self) -> Result<&Selection> {
        self.selection.as_ref().ok_or(Error::Precondition)
    }
    pub fn project(&self) -> Result<&Resource> {
        self.vercel.as_ref().ok_or(Error::Precondition)
    }
    pub fn database(&self) -> Result<&Resource> {
        self.database.as_ref().ok_or(Error::Precondition)
    }
    pub fn origin(&self) -> Result<&str> {
        self.origin.as_deref().ok_or(Error::Precondition)
    }
    pub fn check(&mut self, kind: &str, title: &str) {
        self.checks.retain(|c| c.title != title);
        self.checks.push(Check {
            kind: kind.into(),
            title: title.into(),
            at: now(),
        });
    }
    pub fn assert_writable(&self) -> Result<()> {
        if self.read_only {
            return Err(Error::RecoveryReadOnly);
        }
        if self.credentials_removed {
            return Err(Error::MissingCredential);
        }
        Ok(())
    }
}
pub fn valid_email(s: &str) -> bool {
    s.len() <= 254
        && s.split('@').count() == 2
        && s.contains('.')
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"@._+-".contains(&c))
}
pub fn identifier(s: &str) -> Result<&str> {
    if s.is_empty()
        || s.len() > 128
        || !s
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
    {
        Err(Error::Invalid)
    } else {
        Ok(s)
    }
}
