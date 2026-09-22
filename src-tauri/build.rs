fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "snapshot",
            "check_release",
            "check_fresh_retry",
            "use_fresh_retry",
            "check_installed_repair",
            "apply_installed_repair",
            "backup_and_repair",
            "start_installation",
            "save_credentials",
            "discover_accounts",
            "select_accounts",
            "set_google",
            "set_database_connection",
            "advance",
            "check_deployment",
            "open_step",
            "export_recovery",
            "import_recovery",
            "remove_credentials",
            "forget_instance",
            "reconcile_created",
        ]),
    ))
    .expect("Tauri capability build failed")
}
