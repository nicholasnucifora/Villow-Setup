mod common;
use common::*;
use villow_setup::{
    app_updates::{self, Status},
    error::Error,
    model::Step,
    release::{self, Channel, ReleasePointer},
};

fn candidate(
    minimum: &str,
) -> (
    villow_setup::model::Installation,
    Vec<u8>,
    ReleasePointer,
    Channel,
    release::Trust,
) {
    let old = verified();
    let mut s = installation(&old);
    s.step = Step::Complete;
    let (trust, _, _, _) = fixtures();
    let mut m = old.manifest.clone();
    m.app_version = "1.1.0".into();
    m.minimum_manager = minimum.into();
    let bytes = serde_json::to_vec(&m).unwrap();
    let pointer = ReleasePointer {
        version: m.app_version,
        sha256: release::hash(&bytes),
        url: "https://github.com/test-owner/test-releases/releases/download/v1.1.0/manifest.json"
            .into(),
    };
    let channel = Channel {
        format: 1,
        channel: "stable".into(),
        sequence: 5,
        generated_at: "2026-09-24T00:00:00Z".into(),
        expires_at: "2026-09-30T00:00:00Z".into(),
        releases: vec![pointer.clone()],
        revoked: vec![],
    };
    (s, bytes, pointer, channel, trust)
}

#[test]
fn public_availability_is_not_update_authority_and_needs_no_provider_tokens() {
    let (mut s, bytes, p, c, t) = candidate("0.1.0");
    s.credentials_removed = true;
    let result = app_updates::inspect(&s, &bytes, &p, &c, &t).unwrap();
    assert_eq!(result.status, Status::Unsupported);
    assert_eq!(s.app_version, "1.0.0");
    let (s, bytes, p, c, t) = candidate("99.0.0");
    assert_eq!(
        app_updates::inspect(&s, &bytes, &p, &c, &t).unwrap().status,
        Status::ManagerRequired
    );
}

#[test]
fn unauthenticated_or_revoked_metadata_and_read_only_imports_are_refused() {
    let (s, bytes, p, c, t) = candidate("0.1.0");
    for kind in 0..6 {
        let (mut s, mut bytes, mut p, mut c) = (s.clone(), bytes.clone(), p.clone(), c.clone());
        match kind {
            0 => bytes.push(b' '),
            1 => c.revoked.push(p.sha256.clone()),
            2 => c.releases.clear(),
            3 => s.read_only = true,
            4 => s.step = Step::Health,
            _ => p.version = "9.0.0".into(),
        }
        assert!(app_updates::inspect(&s, &bytes, &p, &c, &t).is_err());
    }
}

#[test]
fn unknown_future_contract_still_explains_required_manager_without_accepting_installation() {
    let (s, bytes, mut p, mut c, t) = candidate("99.0.0");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["future_contract"] = serde_json::json!({"not_executed": true});
    let bytes = serde_json::to_vec(&value).unwrap();
    p.sha256 = release::hash(&bytes);
    c.releases = vec![p.clone()];
    assert!(serde_json::from_slice::<release::Manifest>(&bytes).is_err());
    assert_eq!(
        app_updates::inspect(&s, &bytes, &p, &c, &t).unwrap().status,
        Status::ManagerRequired
    );
    value["minimum_manager"] = "not a version".into();
    let bytes = serde_json::to_vec(&value).unwrap();
    p.sha256 = release::hash(&bytes);
    c.releases = vec![p.clone()];
    assert_eq!(
        app_updates::inspect(&s, &bytes, &p, &c, &t).err(),
        Some(Error::Release)
    );
}

#[test]
fn an_older_recommendation_never_becomes_a_downgrade_offer() {
    let (mut s, bytes, p, c, t) = candidate("0.1.0");
    s.app_version = "2.0.0".into();
    assert_eq!(
        app_updates::inspect(&s, &bytes, &p, &c, &t).unwrap().status,
        Status::Unsupported
    );
    s.app_version = p.version.clone();
    s.release_digest = p.sha256.clone();
    assert_eq!(
        app_updates::inspect(&s, &bytes, &p, &c, &t).unwrap().status,
        Status::Current
    );
}
