//! Shared Post-Quantum Cryptography
//!
//! This crate provides post-quantum cryptographic primitives for both
//! Bond and Aevum protocols, implementing ML-DSA (CRYSTALS-Dilithium).
//!
//! # Security Levels
//!
//! - **Bond Protocol**: ML-DSA-65 (Level 3) - ~192-bit quantum security
//! - **Aevum Protocol**: ML-DSA-44 (Level 2) - ~128-bit quantum security

pub mod error;
pub mod keypair;
pub mod signature;
pub mod types;

pub use error::*;
pub use keypair::*;
pub use signature::*;
pub use types::*;

use serde::{Deserialize, Serialize};

/// Security levels for ML-DSA implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// ML-DSA-44 - Level 2 security (~128-bit quantum resistance)
    /// Used by Aevum protocol for high-performance transactions
    Level2,
    /// ML-DSA-65 - Level 3 security (~192-bit quantum resistance)  
    /// Used by Bond protocol for maximum security store of value
    Level3,
}

impl SecurityLevel {
    /// Get the public key size in bytes for this security level
    pub fn public_key_size(&self) -> usize {
        match self {
            SecurityLevel::Level2 => 1312, // ML-DSA-44
            SecurityLevel::Level3 => 1952, // ML-DSA-65
        }
    }

    /// Get the private key size in bytes for this security level
    pub fn private_key_size(&self) -> usize {
        match self {
            SecurityLevel::Level2 => 2560, // ML-DSA-44
            SecurityLevel::Level3 => 4032, // ML-DSA-65
        }
    }

    /// Get the signature size in bytes for this security level
    pub fn signature_size(&self) -> usize {
        match self {
            SecurityLevel::Level2 => 2420, // ML-DSA-44
            SecurityLevel::Level3 => 3309, // ML-DSA-65
        }
    }
}
