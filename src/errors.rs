use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretsProviderError {
    #[error("Initialization error: {0}")]
    Initialization(String),

    #[error("Incorrect typecast for secret {0}")]
    InvalidType(String),

    #[error("Unknown secret type for secret {0}")]
    UnknownType(String),

    #[error("Backend implementation failed: {0}")]
    ProviderFailed(String),
}
