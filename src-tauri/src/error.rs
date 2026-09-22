use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, thiserror::Error, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStage {
    #[error("running the release SQL")]
    Sql,
    #[error("checking the result")]
    Verification,
    #[error("recording completion")]
    History,
}

// Never surface raw provider bodies, URLs, SQL or third-party error strings.
#[derive(Debug, Clone, Serialize, thiserror::Error, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("Reconnect the provider: access has expired or was refused.")]
    Authentication,
    #[error("Vercel refused access while checking your account identity (GET /v2/user). Supabase has not been checked yet. Check that your Vercel token was copied in full, is still valid, and permits account access. Replace only the Vercel token on its page or in Recovery & settings; an expiry date in the future does not rule out a scope or access restriction.")]
    VercelIdentityAccess,
    #[error("Vercel refused access while listing your teams (GET /v2/teams). Supabase has not been checked yet. Check that the saved Vercel token permits access to your intended hosting team. Replace only the Vercel token on its page or in Recovery & settings. This refusal does not establish that the token has expired.")]
    VercelTeamsAccess,
    #[error("Supabase refused access while checking your account identity (GET /v1/profile). The Vercel account checks passed. An unexpired token can still be refused. Check that you copied the management token in full. This Alpha's Supabase instructions explain the Create legacy token link and its account-wide access; scoped tokens have failed this check in account testing. Replace only the Supabase token and keep your existing setup and projects. If a legacy token also fails, report this exact check.")]
    SupabaseIdentityAccess,
    #[error("Supabase refused access while listing your organizations (GET /v1/organizations). The Vercel and Supabase identity checks passed. Check Organizations: Read, the selected organization and your account's membership. An expiry date in the future does not guarantee permission. Replace only the Supabase token if its access settings need changing.")]
    SupabaseOrganizationsAccess,
    #[error("The selected account or resource does not match this installation. Further writes have stopped; review the saved resources.")]
    WrongTarget,
    #[error("The provider could not be reached. Progress is saved; try again when connected.")]
    Offline,
    #[error("The provider is busy. Progress is saved; wait before trying again.")]
    RateLimited,
    #[error(
        "The provider refused the operation. Inspect its dashboard; no raw response is displayed."
    )]
    Provider,
    #[error("A remote effect may have completed. Review the named resource in its provider dashboard before reconciliation; creation will not be repeated.")]
    Uncertain,
    #[error(
        "Another setup operation is running. Wait for it to finish or close the other application."
    )]
    Busy,
    #[error("Local progress could not be saved. No further remote effects will run.")]
    Storage,
    #[error(
        "Windows Credential Manager is unavailable. Secrets have not been saved as plaintext."
    )]
    Vault,
    #[error("A required credential is missing. Reconnect without starting a new installation.")]
    MissingCredential,
    #[error("The release could not be authenticated, is expired/revoked, or is not compatible. No cloud changes were made.")]
    Release,
    #[error("No official release source or signing identity has been configured in this development build.")]
    Unconfigured,
    #[error("The input is not valid for this setup step.")]
    Invalid,
    #[error("Finish the prerequisite step before continuing.")]
    Precondition,
    #[error("Database history, schema or ownership differs from the verified plan. Manual review is required; nothing will be reset.")]
    SchemaDrift,
    #[error("The database connection worked, but the fresh-database check found {relations} existing tables/views/sequences and {routines} non-extension functions in the public schema before Villow's installation history exists. This does not establish who created them. Setup stopped before installing; nothing will be reset. Keep this project and report this message for review. Changing the password will not fix this check.")]
    DatabaseNotEmpty { relations: i64, routines: i64 },
    #[error("The database connection worked, but Villow's installation history is incomplete or unreadable. Setup cannot safely continue. Keep the project and report this message; do not delete tables, reset the database or change its password.")]
    DatabaseHistoryIncomplete,
    #[error("The database connection worked, but database preparation stopped at release unit {unit} while {stage} (PostgreSQL code: {code}). This is not a password failure and does not by itself mean existing data was found. The unfinished unit was not committed. Keep this project and report this entire message so the release can be checked; do not reset the database.")]
    DatabaseMigration {
        unit: usize,
        stage: MigrationStage,
        code: String,
    },
    #[error("The database connection worked, but the result check for release unit {unit} did not pass. The unfinished unit was not committed. Keep this project and report this message for review; do not reset the database or change its password.")]
    DatabasePostcondition { unit: usize },
    #[error("A corrected release cannot be used at this stage. Setup requires the same owned, unfinished installation with no committed app tables or migration units. Your saved setup has not been reset.")]
    FreshRetryRefused,
    #[error("Finish switching to the saved corrected release before preparing the database. Use Resume release change below; your accounts and credentials are retained.")]
    FreshRetryPending,
    #[error("This repair cannot be applied to the saved installation. It requires the original unfinished Alpha setup, matching installed release, unchanged database history and the same signed-in owner. Keep your resources and report this message; nothing will be reset.")]
    RepairRefused,
    #[error("Finish the saved app repair using Resume repair. Normal setup actions are paused so the installed database and release stay consistent.")]
    RepairPending,
    #[error("Setup could not verify the recovery copy for this repair. Keep your saved setup and use Repair my app to start, or Resume repair if already started. If this continues, report this message; do not reset the database.")]
    RepairBackupRequired,
    #[error("Choose a backup password of at least 12 characters and save it in your password manager. This protects your Villow data backup; it is separate from the developer's signing passphrase.")]
    BackupPassword,
    #[error("Setup could not read, save or verify its recovery copy. Check that this PC has free disk space and reopen Setup. Keep its saved files and database; no further repair work was performed.")]
    BackupStorage,
    #[error("The recovery copy could not be unlocked or verified. Keep this PC's saved Setup data and report this message. Setup stopped this operation; saved repair progress is retained.")]
    BackupInvalid,
    #[error("This backup exceeds this Alpha's limit of 128 MiB of app database data or 256 MiB for the recovery package. Keep your database and ask for help with a larger backup; this operation stopped.")]
    BackupTooLarge,
    #[error("Setup could not make a complete, restorable copy of this database. The database layout, access or dependencies need review. No repair has started; keep the existing database.")]
    BackupDatabase,
    #[error("The repair's database check failed. The schema, installation history or owner differs from the authenticated repair plan. Keep this database and report this message; do not reset it or repeat fresh preparation.")]
    RepairDatabase,
    #[error("A database operation failed after connecting. Progress is saved. Retry Prepare my database; if it persists, report this message. Do not recreate the project or reset its tables.")]
    Database,
    #[error("Setup could not reach the database host on port 5432. In the connection settings below, use the Session pooler host and user from Supabase → your project → Connect. Direct connections usually need IPv6. Check your network, firewall and Supabase network restrictions; keep the saved password unless you changed it.")]
    DatabaseNetwork,
    #[error("Setup could not verify the database's secure connection. The Supabase root certificate is bundled, and certificate and hostname checks remain enabled. Check your PC's date/time and the host copied from Supabase Connect. If this continues, report this exact message; do not turn off SSL or certificate checks.")]
    DatabaseTls,
    #[error("The database refused its saved login or access rules. Check the host and user against Supabase Connect. Only enter a replacement database password if you changed it in Supabase; the management token is not the database password. Also check the project's network restrictions.")]
    DatabaseAccess,
    #[error("The database is starting up or has no free connections. Wait until the Supabase project is healthy, then retry Prepare my database. Keep your existing project and saved settings.")]
    DatabaseUnavailable,
    #[error("The database connection could not finish. Check Supabase → your project → Connect → Session pooler, and use its host and user in the settings below. Port 5432 is required. Keep the saved password unless you changed it. If it still fails, report this exact message.")]
    DatabaseConnect,
    #[error("Setup could not read usable Session pooler details from Supabase. Open your project → Connect → Session pooler and copy its host and user into the connection settings below. Keep the saved password; do not select Transaction pooler.")]
    DatabasePooler,
    #[error("The deployed app has not passed its required authenticated checks.")]
    Health,
    #[error("Your Vercel build or website address is not ready yet. Return to the build step to check its status before signing in.")]
    DeploymentNotReady,
    #[error("This recovery file provides read-only information. Resource ownership must be re-established before a future repair feature can write.")]
    RecoveryReadOnly,
    #[error("This operation is outside version one's scope.")]
    Unsupported,
}
pub type Result<T> = std::result::Result<T, Error>;
