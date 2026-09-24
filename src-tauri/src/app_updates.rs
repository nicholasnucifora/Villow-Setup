//! Read-only availability. Signed metadata is information, not update authority.
use crate::{
    error::{Error, Result},
    model::{Installation, Step},
    release::{self, Channel, ReleasePointer, Trust},
};
use semver::Version;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Available,
    Current,
    ManagerRequired,
    Unsupported,
}

#[derive(Debug, Serialize)]
pub struct Availability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    pub downtime: String,
    pub status: Status,
    pub app_version: String,
    pub minimum_manager: String,
    pub message: String,
    pub notes: String,
}

pub fn eligible(s: &Installation) -> Result<()> {
    if s.read_only {
        return Err(Error::RecoveryReadOnly);
    }
    if s.step != Step::Complete
        || s.repair_pending()
        || s.update_pending()
        || s.fresh_retry.is_some()
    {
        return Err(Error::Precondition);
    }
    // Checking a public release needs no vault/provider access. Missing or expired
    // credentials will be handled before a future authenticated update starts.
    Ok(())
}

pub fn current(s: &Installation) -> Availability {
    Availability {
        digest: None,
        downtime: String::new(),
        status: Status::Current,
        app_version: s.app_version.clone(),
        minimum_manager: String::new(),
        message: format!("Villow {} is the latest approved release.", s.app_version),
        notes: String::new(),
    }
}

pub fn inspect(
    s: &Installation,
    bytes: &[u8],
    pointer: &ReleasePointer,
    channel: &Channel,
    trust: &Trust,
) -> Result<Availability> {
    eligible(s)?;
    trust.artifact_url(&pointer.url)?;
    if bytes.len() > 4_000_000
        || release::hash(bytes) != pointer.sha256
        || channel.revoked.contains(&pointer.sha256)
        || !channel.releases.iter().any(|p| {
            p.sha256 == pointer.sha256 && p.url == pointer.url && p.version == pointer.version
        })
    {
        return Err(Error::Release);
    }
    // Read minimal authenticated metadata before requiring a supported manifest
    // shape, so a future manager contract can give an actionable message.
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| Error::Release)?;
    let string = |name: &str, max| -> Result<String> {
        let text = value[name].as_str().ok_or(Error::Release)?;
        if text.len() > max {
            return Err(Error::Release);
        }
        Ok(text.to_string())
    };
    let app_version = string("app_version", 100)?;
    let minimum_manager = string("minimum_manager", 100)?;
    let notes = string("notes", 16000)?;
    let proposed = Version::parse(&app_version).map_err(|_| Error::Release)?;
    let installed = Version::parse(&s.app_version).map_err(|_| Error::Release)?;
    let manager = Version::parse(&minimum_manager).map_err(|_| Error::Release)?;
    if pointer.version != app_version || value["channel"].as_str() != Some(channel.channel.as_str())
    {
        return Err(Error::Release);
    }
    if pointer.sha256 == s.release_digest {
        return Ok(current(s));
    }
    if proposed <= installed {
        return Ok(Availability {
            digest: None,
            downtime: String::new(),
            status: Status::Unsupported,
            app_version,
            minimum_manager,
            message: "The release list does not offer a newer version for this installation."
                .into(),
            notes: String::new(),
        });
    }
    let status = if manager > Version::parse(env!("CARGO_PKG_VERSION")).unwrap() {
        Status::ManagerRequired
    } else {
        Status::Unsupported
    };
    let message = if status == Status::ManagerRequired {
        format!("Villow {app_version} needs a newer Setup application.")
    } else {
        format!("Villow {app_version} is published, but a supported update must be verified before installation.")
    };
    Ok(Availability {
        digest: None,
        downtime: String::new(),
        status,
        app_version,
        minimum_manager,
        message,
        notes,
    })
}
