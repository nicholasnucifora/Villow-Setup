use crate::{
    error::{Error, Result},
    manager::{Manager, Snapshot},
    model::{Accounts, DbConnection, Google, Selection},
    vault::OsVault,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager as TauriManager, State};

struct Gate(Arc<Mutex<()>>);
async fn work<T: Send + 'static>(
    app: AppHandle,
    gate: &Gate,
    f: impl FnOnce(Manager) -> Result<T> + Send + 'static,
) -> std::result::Result<T, String> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|_| Error::Storage.to_string())?;
    let gate = gate.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _busy = gate.try_lock().map_err(|_| Error::Busy)?;
        f(Manager::open(&root)?)
    })
    .await
    .map_err(|_| {
        "The operation stopped unexpectedly. Reopen to reconcile saved progress.".to_string()
    })?
    .map_err(|e| e.to_string())
}
#[tauri::command]
async fn snapshot(app: AppHandle, gate: State<'_, Gate>) -> std::result::Result<Snapshot, String> {
    work(app, &gate, |m| m.snapshot()).await
}
#[tauri::command]
async fn check_release(
    app: AppHandle,
    gate: State<'_, Gate>,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, |m| m.check_release()).await
}
#[tauri::command]
async fn check_fresh_retry(
    app: AppHandle,
    gate: State<'_, Gate>,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, |m| m.check_fresh_retry()).await
}
#[tauri::command]
async fn use_fresh_retry(
    app: AppHandle,
    gate: State<'_, Gate>,
    digest: String,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| m.use_fresh_retry(digest)).await
}
#[tauri::command]
async fn start_installation(
    app: AppHandle,
    gate: State<'_, Gate>,
    name: String,
    email: String,
    digest: String,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| m.start(name, email, digest)).await
}
#[tauri::command]
async fn save_credentials(
    app: AppHandle,
    gate: State<'_, Gate>,
    vercel: String,
    supabase: String,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| m.credentials(vercel, supabase)).await
}
#[tauri::command]
async fn discover_accounts(
    app: AppHandle,
    gate: State<'_, Gate>,
) -> std::result::Result<Accounts, String> {
    work(app, &gate, |m| m.accounts()).await
}
#[tauri::command]
async fn select_accounts(
    app: AppHandle,
    gate: State<'_, Gate>,
    selection: Selection,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| m.select(selection)).await
}
#[tauri::command]
async fn set_google(
    app: AppHandle,
    gate: State<'_, Gate>,
    google: Google,
    secret: String,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| m.google(google, secret)).await
}
#[tauri::command]
async fn set_database_connection(
    app: AppHandle,
    gate: State<'_, Gate>,
    connection: DbConnection,
    password: String,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| {
        m.database_connection(connection, password)
    })
    .await
}
#[tauri::command]
async fn advance(app: AppHandle, gate: State<'_, Gate>) -> std::result::Result<Snapshot, String> {
    work(app, &gate, |m| m.advance()).await
}
#[tauri::command]
async fn reconcile_created(
    app: AppHandle,
    gate: State<'_, Gate>,
    provider: String,
    id: String,
    confirmation: String,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| {
        m.reconcile_created(provider, id, confirmation)
    })
    .await
}
#[tauri::command]
async fn open_step(
    app: AppHandle,
    gate: State<'_, Gate>,
    step: String,
) -> std::result::Result<(), String> {
    work(app, &gate, move |m| {
        let url = m.browser_url(&step)?;
        open::that_detached(url).map_err(|_| Error::Invalid)
    })
    .await
}
#[tauri::command]
async fn export_recovery(
    app: AppHandle,
    gate: State<'_, Gate>,
) -> std::result::Result<bool, String> {
    work(app, &gate, |m| {
        let _lock = m.store.lock()?;
        let s = m.store.load()?.ok_or(Error::Precondition)?;
        let text = crate::recovery::export(&s)?;
        let path = rfd::FileDialog::new()
            .set_title("Save nonsecret recovery information")
            .set_file_name("villow-recovery.json")
            .add_filter("JSON", &["json"])
            .save_file();
        if let Some(path) = path {
            std::fs::write(path, text).map_err(|_| Error::Storage)?;
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .await
}
#[tauri::command]
async fn import_recovery(
    app: AppHandle,
    gate: State<'_, Gate>,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, |m| {
        let _lock = m.store.lock()?;
        if m.store.load()?.is_some() {
            return Err(Error::Precondition);
        }
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open nonsecret recovery information")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            if std::fs::metadata(&path).map_err(|_| Error::Storage)?.len() > 100_000 {
                return Err(Error::Invalid);
            }
            let text = std::fs::read_to_string(path).map_err(|_| Error::Storage)?;
            m.store.save(&crate::recovery::import(&text)?)?;
        }
        m.snapshot()
    })
    .await
}
#[tauri::command]
async fn remove_credentials(
    app: AppHandle,
    gate: State<'_, Gate>,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, |m| {
        crate::engine::remove_credentials(&m.store, &OsVault)?;
        m.snapshot()
    })
    .await
}
#[tauri::command]
async fn forget_instance(
    app: AppHandle,
    gate: State<'_, Gate>,
    confirmation: String,
) -> std::result::Result<Snapshot, String> {
    work(app, &gate, move |m| {
        crate::engine::forget(&m.store, &OsVault, &confirmation)?;
        m.snapshot()
    })
    .await
}
pub fn run() {
    tauri::Builder::default()
        .manage(Gate(Arc::new(Mutex::new(()))))
        .invoke_handler(tauri::generate_handler![
            snapshot,
            check_release,
            check_fresh_retry,
            use_fresh_retry,
            start_installation,
            save_credentials,
            discover_accounts,
            select_accounts,
            set_google,
            set_database_connection,
            advance,
            reconcile_created,
            open_step,
            export_recovery,
            import_recovery,
            remove_credentials,
            forget_instance
        ])
        .run(tauri::generate_context!())
        .expect("Villow Setup could not start its local window");
}
