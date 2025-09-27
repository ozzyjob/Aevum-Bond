use thiserror::Error;

/// Bond protocol specific errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BondError {
    #[error("Invalid block hash: expected {expected}, got {actual}")]
    InvalidBlockHash { expected: String, actual: String },
    
    #[error("Invalid proof of work: hash {hash} does not meet difficulty target {target}")]
    InvalidProofOfWork { hash: String, target: String },
    
    #[error("Invalid transaction: {reason}")]
    InvalidTransaction { reason: String },
    
    #[error("Invalid UTXO: {reason}")]
    InvalidUtxo { reason: String },
    
    #[error("Script execution failed: {reason}")]
    ScriptExecutionFailed { reason: String },
    
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    #[error("Double spending detected: UTXO {utxo_id} already spent")]
    DoubleSpending { utxo_id: String },
    
    #[error("Cryptographic error: {reason}")]
    CryptographicError { reason: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("JSON error: {0}")]
    JsonError(String),
    
    #[error("Arithmetic overflow in {operation}")]
    ArithmeticOverflow { operation: String },
}

impl From<bincode::Error> for BondError {
    fn from(err: bincode::Error) -> Self {
        BondError::SerializationError(err.to_string())
    }
}

impl From<serde_json::Error> for BondError {
    fn from(err: serde_json::Error) -> Self {
        BondError::JsonError(err.to_string())
    }
}

pub type BondResult<T> = Result<T, BondError>;
