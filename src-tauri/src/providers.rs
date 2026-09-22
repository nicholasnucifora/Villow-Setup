use crate::{
    engine::Providers,
    error::{Error, Result},
    http::{Api, Provider},
    migration,
    model::*,
    release::{hash, VerifiedRelease},
    vault::Vault,
};
use reqwest::Method;
use serde_json::{json, Value};

pub struct LiveProviders<'a> {
    pub http: &'a dyn Api,
    pub vault: &'a dyn Vault,
}
fn string(v: &Value, key: &str) -> Result<String> {
    let s = v.get(key).and_then(Value::as_str).ok_or(Error::Provider)?;
    if s.len() > 500 {
        return Err(Error::Provider);
    }
    Ok(s.to_string())
}
// Authentication responses deliberately omit provider bodies. Add only fixed
// context at these read-only checks; never turn a failed identity check into
// a successful account discovery or change write/reconciliation error handling.
fn account_access(error: Error, context: Error) -> Error {
    if error == Error::Authentication {
        context
    } else {
        error
    }
}
impl LiveProviders<'_> {
    fn vercel(
        &self,
        s: &Installation,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value> {
        let token = self.vault.require(&s.id, "vercel_token")?;
        self.http.provider(
            Provider::Vercel,
            method,
            path,
            &[("teamId", &s.selection()?.vercel_account)],
            &token,
            body,
        )
    }
    fn supabase(
        &self,
        s: &Installation,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value> {
        let token = self.vault.require(&s.id, "supabase_token")?;
        self.http
            .provider(Provider::Supabase, method, path, &[], &token, body)
    }
    pub fn accounts(&self, s: &Installation) -> Result<Accounts> {
        let vtoken = self.vault.require(&s.id, "vercel_token")?;
        let stoken = self.vault.require(&s.id, "supabase_token")?;
        let user = self
            .http
            .provider(
                Provider::Vercel,
                Method::GET,
                "/v2/user",
                &[],
                &vtoken,
                None,
            )
            .map_err(|e| account_access(e, Error::VercelIdentityAccess))?;
        let teams = self
            .http
            .provider(
                Provider::Vercel,
                Method::GET,
                "/v2/teams",
                &[("limit", "100")],
                &vtoken,
                None,
            )
            .map_err(|e| account_access(e, Error::VercelTeamsAccess))?;
        // Do not silently present an incomplete account list as complete.
        if !teams["pagination"]["next"].is_null() {
            return Err(Error::Unsupported);
        }
        let profile = self
            .http
            .provider(
                Provider::Supabase,
                Method::GET,
                "/v1/profile",
                &[],
                &stoken,
                None,
            )
            .map_err(|e| account_access(e, Error::SupabaseIdentityAccess))?;
        let organizations = self
            .http
            .provider(
                Provider::Supabase,
                Method::GET,
                "/v1/organizations",
                &[],
                &stoken,
                None,
            )
            .map_err(|e| account_access(e, Error::SupabaseOrganizationsAccess))?;
        let map = |v: &Value| -> Result<Account> {
            Ok(Account {
                id: string(v, "id")?,
                name: string(v, "name")?,
                slug: string(v, "slug")?,
            })
        };
        Ok(Accounts {
            vercel_user: string(&user["user"], "id")?,
            supabase_user: string(&profile, "gotrue_id")?,
            vercel: teams["teams"]
                .as_array()
                .ok_or(Error::Provider)?
                .iter()
                .map(map)
                .collect::<Result<_>>()?,
            supabase: organizations
                .as_array()
                .ok_or(Error::Provider)?
                .iter()
                .map(map)
                .collect::<Result<_>>()?,
        })
    }
    fn verify_vercel_resource(&self, s: &Installation, value: &Value) -> Result<Resource> {
        let id = string(value, "id")?;
        identifier(&id)?;
        let account = string(value, "accountId")?;
        let name = string(value, "name")?;
        if account != s.selection()?.vercel_account
            || name != s.name
            || s.vercel
                .as_ref()
                .is_some_and(|r| r.id != id || r.operation_id != s.operation_id)
        {
            return Err(Error::WrongTarget);
        }
        Ok(Resource {
            id,
            account_id: account,
            name,
            operation_id: s.operation_id.clone(),
            evidence: "creation_response_and_account_read".into(),
        })
    }
    fn verify_database_resource(&self, s: &Installation, value: &Value) -> Result<Resource> {
        let id = value
            .get("ref")
            .and_then(Value::as_str)
            .or_else(|| value.get("id").and_then(Value::as_str))
            .ok_or(Error::Provider)?
            .to_string();
        identifier(&id)?;
        let account = string(value, "organization_id")?;
        let name = string(value, "name")?;
        if account != s.selection()?.supabase_organization
            || name != s.name
            || s.database
                .as_ref()
                .is_some_and(|r| r.id != id || r.operation_id != s.operation_id)
        {
            return Err(Error::WrongTarget);
        }
        Ok(Resource {
            id,
            account_id: account,
            name,
            operation_id: s.operation_id.clone(),
            evidence: "creation_response_and_account_read".into(),
        })
    }
    fn domains(&self, s: &Installation) -> Result<Value> {
        self.vercel(
            s,
            Method::GET,
            &format!("/v9/projects/{}/domains", identifier(&s.project()?.id)?),
            None,
        )
    }
    fn matching_domain(&self, s: &Installation, value: &Value) -> Result<bool> {
        let expected = format!("{}.vercel.app", s.name);
        Ok(value["domains"]
            .as_array()
            .ok_or(Error::Provider)?
            .iter()
            .any(|d| {
                d["name"] == expected
                    && d["verified"] == true
                    && d["gitBranch"].is_null()
                    && d["redirect"].is_null()
            }))
    }
}
impl Providers for LiveProviders<'_> {
    fn verify_targets(&self, s: &Installation) -> Result<()> {
        let accounts = self.accounts(s)?;
        let selected = s.selection()?;
        if accounts.vercel_user != selected.vercel_user
            || accounts.supabase_user != selected.supabase_user
            || !accounts
                .vercel
                .iter()
                .any(|a| a.id == selected.vercel_account)
            || !accounts
                .supabase
                .iter()
                .any(|a| a.id == selected.supabase_organization && a.slug == selected.supabase_slug)
        {
            return Err(Error::WrongTarget);
        }
        if let Some(r) = &s.vercel {
            self.verify_vercel_resource(
                s,
                &self.vercel(
                    s,
                    Method::GET,
                    &format!("/v9/projects/{}", identifier(&r.id)?),
                    None,
                )?,
            )?;
        }
        if let Some(r) = &s.database {
            self.verify_database_resource(
                s,
                &self.supabase(
                    s,
                    Method::GET,
                    &format!("/v1/projects/{}", identifier(&r.id)?),
                    None,
                )?,
            )?;
        }
        if let Some(origin) = &s.origin {
            if origin != &format!("https://{}.vercel.app", s.name)
                || !self.matching_domain(s, &self.domains(s)?)?
            {
                return Err(Error::WrongTarget);
            }
        }
        Ok(())
    }
    fn create_vercel(&self, s: &Installation) -> Result<Resource> {
        let v = self.vercel(s,Method::POST,"/v10/projects",Some(&json!({"name":s.name,"framework":"vite","installCommand":"npm ci","buildCommand":"npm run build","outputDirectory":"dist"})))?;
        let r = self.verify_vercel_resource(s, &v)?;
        self.verify_vercel_resource(
            s,
            &self.vercel(
                s,
                Method::GET,
                &format!("/v9/projects/{}", identifier(&r.id)?),
                None,
            )?,
        )
    }
    fn create_database(&self, s: &Installation) -> Result<Resource> {
        let password = self.vault.require(&s.id, "db_password")?;
        // No paid plan, compute upgrade or add-on is selected here.
        let v = self.supabase(
            s,
            Method::POST,
            "/v1/projects",
            Some(
                &json!({"name":s.name,"organization_slug":s.selection()?.supabase_slug,
            "region":s.selection()?.region,"db_pass":password.as_str()}),
            ),
        )?;
        self.verify_database_resource(s, &v)
    }
    fn reconcile_vercel(&self, s: &Installation) -> Result<Option<Resource>> {
        // A matching name is not ownership provenance. Read, then stop for review.
        let _ = self.vercel(
            s,
            Method::GET,
            &format!("/v9/projects/{}", identifier(&s.name)?),
            None,
        )?;
        Ok(None)
    }
    fn reconcile_database(&self, s: &Installation) -> Result<Option<Resource>> {
        let _ = self.supabase(s, Method::GET, "/v1/projects", None)?;
        Ok(None)
    }
    fn reserve_origin(&self, s: &Installation) -> Result<String> {
        if !self.matching_domain(s, &self.domains(s)?)? {
            self.vercel(
                s,
                Method::POST,
                &format!("/v10/projects/{}/domains", identifier(&s.project()?.id)?),
                Some(&json!({"name":format!("{}.vercel.app",s.name)})),
            )?;
        }
        if !self.matching_domain(s, &self.domains(s)?)? {
            return Err(Error::WrongTarget);
        }
        Ok(format!("https://{}.vercel.app", s.name))
    }
    fn migrate(&self, s: &Installation, r: &VerifiedRelease) -> Result<()> {
        let v = self.supabase(
            s,
            Method::GET,
            &format!("/v1/projects/{}", identifier(&s.database()?.id)?),
            None,
        )?;
        self.verify_database_resource(s, &v)?;
        if v["status"] != "ACTIVE_HEALTHY" {
            return Err(Error::Precondition);
        }
        let mut client = migration::connect(s, self.vault)?;
        migration::apply(&mut client, s, r)
    }
    fn configure(&self, s: &Installation, _r: &VerifiedRelease) -> Result<()> {
        let keys = self.supabase(
            s,
            Method::GET,
            &format!("/v1/projects/{}/api-keys", identifier(&s.database()?.id)?),
            None,
        )?;
        let keys = keys.as_array().ok_or(Error::Provider)?;
        for (provider_name, vault_name) in [("anon", "anon_key"), ("service_role", "service_key")] {
            let key = keys
                .iter()
                .find(|k| k["name"] == provider_name)
                .ok_or(Error::Unsupported)?;
            self.vault
                .put(&s.id, vault_name, &string(key, "api_key")?)?;
        }
        let google_secret = self.vault.require(&s.id, "google_secret")?;
        let encryption = self.vault.require(&s.id, "encryption_key")?;
        let bootstrap = self.vault.require(&s.id, "bootstrap_token")?;
        let service = self.vault.require(&s.id, "service_key")?;
        let anon = self.vault.require(&s.id, "anon_key")?;
        let g = s.google.as_ref().ok_or(Error::Precondition)?;
        let vars = [
            ("VITE_YOUTUBE_CLIENT_ID", g.client_id.clone()),
            ("YOUTUBE_CLIENT_SECRET", google_secret.to_string()),
            (
                "VITE_SUPABASE_URL",
                format!("https://{}.supabase.co", s.database()?.id),
            ),
            ("VITE_SUPABASE_ANON_KEY", anon.to_string()),
            ("SUPABASE_SERVICE_KEY", service.to_string()),
            ("VITE_APP_URL", s.origin()?.to_owned()),
            ("ENCRYPTION_KEY", encryption.to_string()),
            (
                "CRON_SECRET",
                hash(format!("villow-cron:{}", encryption.as_str()).as_bytes()),
            ),
            ("VILLOW_EXPECTED_OWNER_EMAIL", s.owner_email.clone()),
            ("VILLOW_BOOTSTRAP_TOKEN_HASH", hash(bootstrap.as_bytes())),
            ("VILLOW_INSTALLATION_ID", s.id.clone()),
        ];
        let token = self.vault.require(&s.id, "vercel_token")?;
        let env: Vec<_> = vars.iter().map(|(key,value)| json!({"key":key,"value":value,"type":"encrypted","target":["production"]})).collect();
        self.http.provider(
            Provider::Vercel,
            Method::POST,
            &format!("/v10/projects/{}/env", identifier(&s.project()?.id)?),
            &[
                ("teamId", &s.selection()?.vercel_account),
                ("upsert", "true"),
            ],
            &token,
            Some(&json!(env)),
        )?;
        // Values are never read back to the renderer. Authenticated app health
        // separately verifies that its runtime configuration is usable.
        Ok(())
    }
    fn upload(&self, s: &Installation, r: &VerifiedRelease) -> Result<()> {
        let token = self.vault.require(&s.id, "vercel_token")?;
        for (path, spec) in &r.manifest.files {
            if spec.role == "deploy" {
                let actual =
                    self.http
                        .upload(&token, &s.selection()?.vercel_account, &r.files[path])?;
                if actual != upload_digest(&r.files[path]) {
                    return Err(Error::Provider);
                }
            }
        }
        Ok(())
    }
    fn deploy(&self, s: &Installation, r: &VerifiedRelease, reconcile: bool) -> Result<String> {
        if reconcile {
            let token = self.vault.require(&s.id, "vercel_token")?;
            let deployments = self.http.provider(
                Provider::Vercel,
                Method::GET,
                "/v6/deployments",
                &[
                    ("teamId", &s.selection()?.vercel_account),
                    ("projectId", &s.project()?.id),
                    ("limit", "100"),
                ],
                &token,
                None,
            )?;
            let matches: Vec<_> = deployments["deployments"]
                .as_array()
                .ok_or(Error::Provider)?
                .iter()
                .filter(|d| {
                    d["meta"]["villowOperation"] == s.operation_id
                        && d["meta"]["villowRelease"] == r.digest
                })
                .collect();
            if matches.len() != 1 {
                return Err(Error::Uncertain);
            }
            return string(matches[0], "uid");
        }
        let mut files = Vec::new();
        for (path, spec) in &r.manifest.files {
            if spec.role != "deploy" {
                continue;
            }
            let bytes = &r.files[path];
            let sha = upload_digest(bytes);
            files.push(json!({"file":path,"sha":sha,"size":bytes.len()}));
        }
        let v = self.vercel(s,Method::POST,"/v13/deployments",Some(&json!({"name":s.name,"project":s.project()?.id,"target":"production",
            "files":files,"meta":{"villowOperation":s.operation_id,"villowRelease":r.digest},
            "projectSettings":{"framework":"vite","installCommand":"npm ci","buildCommand":"npm run build","outputDirectory":"dist"}})))?;
        let id = string(&v, "id")?;
        identifier(&id)?;
        Ok(id)
    }
    fn health(&self, s: &Installation, r: &VerifiedRelease) -> Result<()> {
        let deployment = s.deployment_id.as_ref().ok_or(Error::Precondition)?;
        let v = self.vercel(
            s,
            Method::GET,
            &format!("/v13/deployments/{}", identifier(deployment)?),
            None,
        )?;
        if v["projectId"] != s.project()?.id
            || v["meta"]["villowOperation"] != s.operation_id
            || v["meta"]["villowRelease"] != r.digest
            || v["readyState"] != "READY"
            || v["target"] != "production"
        {
            return Err(Error::Health);
        }
        let nonce = uuid::Uuid::new_v4().to_string();
        let token = self.vault.require(&s.id, "bootstrap_token")?;
        let h = self.http.health(s.origin()?, &token, &nonce)?;
        validate_health(&h, s, r, &nonce)
    }
}
fn upload_digest(bytes: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    hex::encode(Sha1::digest(bytes))
}
pub fn validate_health(
    h: &Value,
    s: &Installation,
    r: &VerifiedRelease,
    nonce: &str,
) -> Result<()> {
    if h["format"] != 1
        || h["nonce"] != nonce
        || h["installationId"] != s.id
        || h["commit"] != r.manifest.commit
        || h["appVersion"] != r.manifest.app_version
        || h["schemaRevision"] != r.manifest.schema.revision
        || h["ownerEmail"] != s.owner_email
        || h["origin"] != s.origin()?
        || h["intendedOwnerVerified"] != true
        || h["configurationValid"] != true
        || h["databaseProbePassed"] != true
        || h["youtubeAuthorizationVerified"] != true
        || h["cronConfigured"] != true
        || h["bootstrapClosed"] != true
    {
        return Err(Error::Health);
    }
    Ok(())
}
