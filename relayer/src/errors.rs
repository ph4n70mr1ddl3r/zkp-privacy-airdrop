use thiserror::Error;

#[derive(Error, Debug)]
pub enum RelayerError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Proof validation failed: {0}")]
    ProofValidation(String),

    #[error(" cryptographic error: {0}")]
    Crypto(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type RelayerResult<T> = Result<T, RelayerError>;
