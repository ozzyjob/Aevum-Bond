//! Cryptographic error types for Aevum & Bond

use thiserror::Error;

/// Cryptographic errors that can occur during key operations, signing, or verification
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CryptoError {
    /// Invalid key size for the specified security level
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    /// Invalid signature size for the specified security level
    #[error("Invalid signature size: expected {expected}, got {actual}")]
    InvalidSignatureSize { expected: usize, actual: usize },

    /// Key generation failed
    #[error("Key generation failed: {reason}")]
    KeyGenerationFailed { reason: String },

    /// Signature generation failed
    #[error("Signature generation failed: {reason}")]
    SignatureFailed { reason: String },

    /// Signature verification failed
    #[error("Signature verification failed")]
    InvalidSignature,

    /// Security level mismatch between components
    #[error("Security level mismatch: expected {expected:?}, got {actual:?}")]
    SecurityLevelMismatch {
        expected: crate::SecurityLevel,
        actual: crate::SecurityLevel,
    },

    /// Invalid data format or encoding
    #[error("Invalid data format: {reason}")]
    InvalidFormat { reason: String },

    /// Internal cryptographic library error
    #[error("Internal crypto error: {reason}")]
    InternalError { reason: String },
}

/// Result type alias for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;
