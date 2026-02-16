use thiserror::Error;

/// Unified error type for the ZKP Privacy Airdrop project.
///
/// This enum provides consistent error handling across all Rust modules
/// including CLI, relayer, shared utilities, and tree-builder.
///
/// # Usage
/// ```
/// use zkp_airdrop_utils::{ZkpError, ZkpResult};
///
/// fn example() -> ZkpResult<()> {
///     Err(ZkpError::InvalidInput {
///         field: "proof".to_string(),
///         reason: "invalid format".to_string(),
///     })
/// }
/// ```
#[derive(Error, Debug)]
pub enum ZkpError {
    /// Configuration-related errors
    #[error("Configuration error in {field}: {reason}")]
    Config { field: String, reason: String },

    /// Invalid input errors with field context
    #[error("Invalid input for {field}: {reason}")]
    InvalidInput { field: String, reason: String },

    /// Blockchain interaction errors
    #[error("Blockchain error: {0}")]
    Blockchain(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(String),

    /// Redis/cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Proof validation errors
    #[error("Proof validation failed: {0}")]
    ProofValidation(String),

    /// Cryptographic operation errors
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),

    /// File I/O errors
    #[error("File I/O error on {path}: {reason}")]
    Io { path: String, reason: String },

    /// Merkle tree operation errors
    #[error("Merkle tree error: {0}")]
    MerkleTree(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Internal/unexpected errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Not found errors
    #[error("{resource} not found: {identifier}")]
    NotFound {
        resource: String,
        identifier: String,
    },

    /// Unauthorized access errors
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Conflict errors (e.g., double-claim attempts)
    #[error("Conflict: {0}")]
    Conflict(String),
}

/// Result type alias using ZkpError
pub type ZkpResult<T> = Result<T, ZkpError>;

/// Convenience methods for creating common error types
impl ZkpError {
    /// Create a configuration error
    pub fn config(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Config {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Create an I/O error
    pub fn io(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Io {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
            identifier: identifier.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(reason: impl Into<String>) -> Self {
        Self::Conflict(reason.into())
    }

    /// Create an internal error
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::Internal(reason.into())
    }
}

impl From<std::io::Error> for ZkpError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            path: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for ZkpError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let err = ZkpError::config("database", "connection failed");
        assert!(matches!(err, ZkpError::Config { .. }));
        assert!(err.to_string().contains("database"));
    }

    #[test]
    fn test_invalid_input_error() {
        let err = ZkpError::invalid_input("proof", "invalid format");
        assert!(matches!(err, ZkpError::InvalidInput { .. }));
        assert!(err.to_string().contains("proof"));
    }

    #[test]
    fn test_io_error() {
        let err = ZkpError::io("/path/to/file", "permission denied");
        assert!(matches!(err, ZkpError::Io { .. }));
    }

    #[test]
    fn test_not_found_error() {
        let err = ZkpError::not_found("tree", "0x1234");
        assert!(matches!(err, ZkpError::NotFound { .. }));
        assert!(err.to_string().contains("tree"));
    }

    #[test]
    fn test_conflict_error() {
        let err = ZkpError::conflict("double claim detected");
        assert!(matches!(err, ZkpError::Conflict { .. }));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let zkp_err: ZkpError = io_err.into();
        assert!(matches!(zkp_err, ZkpError::Io { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = ZkpError::config("test", "reason");
        let display = err.to_string();
        assert!(display.contains("Configuration error"));
        assert!(display.contains("test"));
        assert!(display.contains("reason"));
    }
}
