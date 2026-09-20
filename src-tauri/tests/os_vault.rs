#[cfg(target_os = "windows")]
#[test]
fn windows_credential_manager_roundtrip_and_verified_removal() {
    use villow_setup::vault::{OsVault, Vault};
    let id = uuid::Uuid::new_v4().to_string();
    struct Cleanup(String);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = OsVault.remove_all(&self.0);
        }
    }
    let _cleanup = Cleanup(id.clone());
    assert!(OsVault.get(&id, "vercel_token").unwrap().is_none());
    OsVault
        .put(&id, "vercel_token", "SENTINEL-WINDOWS-VAULT-ONLY")
        .unwrap();
    assert_eq!(
        OsVault.require(&id, "vercel_token").unwrap().as_str(),
        "SENTINEL-WINDOWS-VAULT-ONLY"
    );
    let first = OsVault.ensure_random(&id, "encryption_key").unwrap();
    let second = OsVault.ensure_random(&id, "encryption_key").unwrap();
    assert_eq!(first.as_str(), second.as_str());
    OsVault.remove_all(&id).unwrap();
    assert!(OsVault.get(&id, "vercel_token").unwrap().is_none());
    assert!(OsVault.get(&id, "encryption_key").unwrap().is_none());
}
