use thiserror::Error;

#[derive(Error, Debug)]
pub enum MulchError {
    #[error("No .mulch/ directory found. Run `mulch init` first.")]
    NotInitialized,

    #[error("Domain \"{domain}\" not found in config. Available domains: {available}")]
    DomainNotFound { domain: String, available: String },

    #[error(
        "Invalid domain name: \"{0}\". Only alphanumeric characters, hyphens, and underscores are allowed."
    )]
    InvalidDomainName(String),

    #[error("Domain \"{0}\" already exists.")]
    DomainAlreadyExists(String),

    #[error("Record \"{0}\" not found. Run `mulch query` to see record IDs.")]
    RecordNotFound(String),

    #[error(
        "Ambiguous identifier \"{id}\" matches {count} records: {ids}. Use more characters to disambiguate."
    )]
    AmbiguousId {
        id: String,
        count: usize,
        ids: String,
    },

    #[error(
        "Timed out waiting for lock on {0}. If no other mulch process is running, delete the lock file manually."
    )]
    LockTimeout(String),

    #[error("Schema validation failed: {0}")]
    ValidationError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, MulchError>;
