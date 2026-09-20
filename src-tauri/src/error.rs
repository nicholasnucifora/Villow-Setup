use serde::Serialize;

// Never surface raw provider bodies, URLs, SQL or third-party error strings.
#[derive(Debug, Clone, Serialize, thiserror::Error, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("Reconnect the provider: access has expired or was refused.")]
    Authentication,
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
    #[error("The database is not reachable over a certificate-verified connection. Check the direct or session-pooler host and password.")]
    Database,
    #[error("The deployed app has not passed its required authenticated checks.")]
    Health,
    #[error("This recovery file provides read-only information. Resource ownership must be re-established before a future repair feature can write.")]
    RecoveryReadOnly,
    #[error("This operation is outside version one's scope.")]
    Unsupported,
}
pub type Result<T> = std::result::Result<T, Error>;
