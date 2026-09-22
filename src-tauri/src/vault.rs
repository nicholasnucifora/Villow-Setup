use crate::error::{Error, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;
pub const SECRET_NAMES: &[&str] = &[
    "vercel_token",
    "supabase_token",
    "db_password",
    "google_secret",
    "encryption_key",
    "bootstrap_token",
    "anon_key",
    "service_key",
    "repair_backup_key",
];
pub trait Vault {
    fn get(&self, instance: &str, name: &str) -> Result<Option<Zeroizing<String>>>;
    fn put(&self, instance: &str, name: &str, secret: &str) -> Result<()>;
    fn delete(&self, instance: &str, name: &str) -> Result<()>;
    fn require(&self, instance: &str, name: &str) -> Result<Zeroizing<String>> {
        self.get(instance, name)?.ok_or(Error::MissingCredential)
    }
    fn ensure_random(&self, instance: &str, name: &str) -> Result<Zeroizing<String>> {
        if let Some(s) = self.get(instance, name)? {
            return Ok(s);
        }
        let mut bytes = Zeroizing::new([0u8; 32]);
        OsRng.fill_bytes(bytes.as_mut());
        let s = Zeroizing::new(hex::encode(bytes.as_ref()));
        self.put(instance, name, &s)?;
        self.require(instance, name)
    }
    fn remove_all(&self, instance: &str) -> Result<()> {
        for name in SECRET_NAMES {
            self.delete(instance, name)?;
        }
        for name in SECRET_NAMES {
            if self.get(instance, name)?.is_some() {
                return Err(Error::Vault);
            }
        }
        Ok(())
    }
}
pub struct OsVault;
impl OsVault {
    fn entry(instance: &str, name: &str) -> Result<keyring::Entry> {
        if !SECRET_NAMES.contains(&name) || uuid::Uuid::parse_str(instance).is_err() {
            return Err(Error::Invalid);
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        return Err(Error::Vault);
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        keyring::Entry::new("VillowSetup", &format!("{instance}/{name}")).map_err(|_| Error::Vault)
    }
}
impl Vault for OsVault {
    fn get(&self, instance: &str, name: &str) -> Result<Option<Zeroizing<String>>> {
        match Self::entry(instance, name)?.get_password() {
            Ok(s) => Ok(Some(Zeroizing::new(s))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(Error::Vault),
        }
    }
    fn put(&self, instance: &str, name: &str, secret: &str) -> Result<()> {
        if secret.is_empty() || secret.len() > 2400 || secret.contains(['\r', '\n', '\0']) {
            return Err(Error::Invalid);
        }
        Self::entry(instance, name)?
            .set_password(secret)
            .map_err(|_| Error::Vault)
    }
    fn delete(&self, instance: &str, name: &str) -> Result<()> {
        match Self::entry(instance, name)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(Error::Vault),
        }
    }
}
pub fn redact(input: &str, secrets: &[&str]) -> String {
    let mut result = input.to_owned();
    let mut variants = Vec::new();
    for secret in secrets.iter().filter(|s| !s.is_empty()) {
        variants.extend([
            secret.to_string(),
            STANDARD.encode(secret.as_bytes()),
            url::form_urlencoded::byte_serialize(secret.as_bytes()).collect::<String>(),
            serde_json::to_string(secret)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
        ]);
    }
    variants.sort_by_key(|s| std::cmp::Reverse(s.len()));
    for v in variants {
        result = result.replace(&v, "[redacted]");
    }
    result
}
