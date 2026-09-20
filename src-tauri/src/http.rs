use crate::{
    error::{Error, Result},
    release::MAX_BUNDLE,
};
use reqwest::{blocking::Client, Method};
use serde_json::Value;
use std::{io::Read, time::Duration};
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Vercel,
    Supabase,
}
impl Provider {
    pub fn host(self) -> &'static str {
        match self {
            Self::Vercel => "api.vercel.com",
            Self::Supabase => "api.supabase.com",
        }
    }
}
pub struct Http {
    client: Client,
}
pub trait Api {
    fn provider(
        &self,
        provider: Provider,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value>;
    fn upload(&self, token: &str, team: &str, bytes: &[u8]) -> Result<String>;
    fn health(&self, origin: &str, token: &str, nonce: &str) -> Result<Value>;
}
impl Api for Http {
    fn provider(
        &self,
        p: Provider,
        m: Method,
        path: &str,
        q: &[(&str, &str)],
        t: &str,
        b: Option<&Value>,
    ) -> Result<Value> {
        Http::provider(self, p, m, path, q, t, b)
    }
    fn upload(&self, token: &str, team: &str, bytes: &[u8]) -> Result<String> {
        Http::upload(self, token, team, bytes)
    }
    fn health(&self, origin: &str, token: &str, nonce: &str) -> Result<Value> {
        Http::health(self, origin, token, nonce)
    }
}
impl Http {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .no_proxy()
                .connect_timeout(Duration::from_secs(15))
                .timeout(Duration::from_secs(90))
                .user_agent(concat!("VillowSetup/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|_| Error::Offline)?,
        })
    }
    pub fn provider(
        &self,
        provider: Provider,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value> {
        let mut url =
            Url::parse(&format!("https://{}", provider.host())).map_err(|_| Error::Invalid)?;
        if !path.starts_with('/') || path.contains(['?', '#', '\\', '%']) || path.contains("..") {
            return Err(Error::Invalid);
        }
        url.set_path(path);
        url.query_pairs_mut().extend_pairs(query.iter().copied());
        validate_credential_url(&url, provider.host())?;
        let mut attempt = 0;
        loop {
            let mut request = self
                .client
                .request(method.clone(), url.clone())
                .bearer_auth(token);
            if let Some(json) = body {
                request = request.json(json);
            }
            let response = request.send().map_err(|_| {
                if method == Method::GET {
                    Error::Offline
                } else {
                    Error::Uncertain
                }
            })?;
            let status = response.status().as_u16();
            if method == Method::GET && matches!(status, 429 | 502 | 503 | 504) && attempt < 2 {
                let delay = response
                    .headers()
                    .get("retry-after")
                    .and_then(|s| s.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1 << attempt)
                    .clamp(1, 10);
                std::thread::sleep(Duration::from_secs(delay));
                attempt += 1;
                continue;
            }
            check_status(status)?;
            let bytes = read_limited(response, 8 * 1024 * 1024)?;
            if bytes.is_empty() {
                return Ok(Value::Null);
            }
            return serde_json::from_slice(&bytes).map_err(|_| {
                if method == Method::GET {
                    Error::Provider
                } else {
                    Error::Uncertain
                }
            });
        }
    }
    pub fn upload(&self, token: &str, team: &str, bytes: &[u8]) -> Result<String> {
        use sha1::{Digest, Sha1};
        let sha = hex::encode(Sha1::digest(bytes)); // Vercel's content-addressing protocol, not our authentication hash.
        let response = self
            .client
            .post("https://api.vercel.com/v2/files")
            .query(&[("teamId", team)])
            .bearer_auth(token)
            .header("Content-Type", "application/octet-stream")
            .header("x-vercel-digest", &sha)
            .header("x-vercel-size", bytes.len())
            .body(bytes.to_vec())
            .send()
            .map_err(|_| Error::Offline)?;
        check_status(response.status().as_u16())?;
        Ok(sha)
    }
    pub fn release_bytes(&self, mut url: Url, limit: u64) -> Result<Vec<u8>> {
        // Release assets carry no credentials. GitHub uses a signed asset-CDN redirect.
        for _ in 0..4 {
            validate_release_destination(&url)?;
            let response = self
                .client
                .get(url.clone())
                .send()
                .map_err(|_| Error::Offline)?;
            if response.status().is_redirection() {
                let location = response
                    .headers()
                    .get("location")
                    .and_then(|s| s.to_str().ok())
                    .ok_or(Error::Release)?;
                url = url.join(location).map_err(|_| Error::Release)?;
                continue;
            }
            check_status(response.status().as_u16())?;
            return read_limited(response, limit.min(MAX_BUNDLE));
        }
        Err(Error::Release)
    }
    pub fn health(&self, origin: &str, token: &str, nonce: &str) -> Result<Value> {
        let url = validate_origin(origin)?;
        let url = url
            .join("/api/setup?action=health")
            .map_err(|_| Error::Invalid)?;
        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&serde_json::json!({"nonce":nonce}))
            .send()
            .map_err(|_| Error::Offline)?;
        check_status(response.status().as_u16())?;
        serde_json::from_slice(&read_limited(response, 32_768)?).map_err(|_| Error::Health)
    }
}
pub fn validate_credential_url(url: &Url, expected: &str) -> Result<()> {
    if url.scheme() != "https"
        || url.host_str() != Some(expected)
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        Err(Error::WrongTarget)
    } else {
        Ok(())
    }
}
pub fn validate_release_destination(url: &Url) -> Result<()> {
    let allowed = [
        "github.com",
        "raw.githubusercontent.com",
        "release-assets.githubusercontent.com",
        "objects.githubusercontent.com",
    ];
    if url.scheme() != "https"
        || !allowed.contains(&url.host_str().unwrap_or(""))
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        Err(Error::Release)
    } else {
        Ok(())
    }
}
pub fn validate_origin(origin: &str) -> Result<Url> {
    let url = Url::parse(origin).map_err(|_| Error::Invalid)?;
    let host = url.host_str().ok_or(Error::Invalid)?;
    if url.scheme() != "https"
        || !host.ends_with(".vercel.app")
        || host.split('.').count() != 3
        || url.path() != "/"
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(Error::WrongTarget);
    }
    Ok(url)
}
pub fn check_status(status: u16) -> Result<()> {
    match status {
        200..=299 => Ok(()),
        300..=399 => Err(Error::WrongTarget),
        401 | 403 => Err(Error::Authentication),
        429 => Err(Error::RateLimited),
        500..=599 => Err(Error::Uncertain),
        _ => Err(Error::Provider),
    }
}
fn read_limited(response: reqwest::blocking::Response, limit: u64) -> Result<Vec<u8>> {
    if response.content_length().is_some_and(|n| n > limit) {
        return Err(Error::Provider);
    }
    let mut bytes = Vec::new();
    response
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Offline)?;
    if bytes.len() as u64 > limit {
        return Err(Error::Provider);
    }
    Ok(bytes)
}
