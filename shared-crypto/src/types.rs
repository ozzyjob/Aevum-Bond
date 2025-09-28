//! Type definitions for post-quantum cryptographic primitives

use serde::{Deserialize, Serialize};
use std::fmt;

/// A post-quantum public key that can be used for signature verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    /// The raw key bytes
    pub(crate) bytes: Vec<u8>,
    /// Security level of this key
    pub(crate) security_level: crate::SecurityLevel,
}

impl PublicKey {
    /// Create a new public key from raw bytes
    pub fn from_bytes(
        bytes: Vec<u8>,
        security_level: crate::SecurityLevel,
    ) -> crate::CryptoResult<Self> {
        let expected_size = security_level.public_key_size();
        if bytes.len() != expected_size {
            return Err(crate::CryptoError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }

        Ok(Self {
            bytes,
            security_level,
        })
    }

    /// Get the raw bytes of this public key
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the security level of this public key
    pub fn security_level(&self) -> crate::SecurityLevel {
        self.security_level
    }

    /// Get the size of this public key in bytes
    pub fn size(&self) -> usize {
        self.bytes.len()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PublicKey({:?}, {} bytes)",
            self.security_level,
            self.bytes.len()
        )
    }
}

/// A post-quantum private key used for signing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateKey {
    /// The raw key bytes (kept private)
    pub(crate) bytes: Vec<u8>,
    /// Security level of this key
    pub(crate) security_level: crate::SecurityLevel,
}

impl PrivateKey {
    /// Create a new private key from raw bytes
    pub fn from_bytes(
        bytes: Vec<u8>,
        security_level: crate::SecurityLevel,
    ) -> crate::CryptoResult<Self> {
        let expected_size = security_level.private_key_size();
        if bytes.len() != expected_size {
            return Err(crate::CryptoError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }

        Ok(Self {
            bytes,
            security_level,
        })
    }

    /// Get the raw bytes of this private key
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the security level of this private key
    pub fn security_level(&self) -> crate::SecurityLevel {
        self.security_level
    }

    /// Get the size of this private key in bytes
    pub fn size(&self) -> usize {
        self.bytes.len()
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PrivateKey({:?}, {} bytes, REDACTED)",
            self.security_level,
            self.bytes.len()
        )
    }
}

/// A post-quantum digital signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// The raw signature bytes
    pub(crate) bytes: Vec<u8>,
    /// Security level used to create this signature
    pub(crate) security_level: crate::SecurityLevel,
}

impl Signature {
    /// Create a new signature from raw bytes
    pub fn from_bytes(
        bytes: Vec<u8>,
        security_level: crate::SecurityLevel,
    ) -> crate::CryptoResult<Self> {
        let expected_size = security_level.signature_size();
        if bytes.len() != expected_size {
            return Err(crate::CryptoError::InvalidSignatureSize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }

        Ok(Self {
            bytes,
            security_level,
        })
    }

    /// Get the raw bytes of this signature
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the security level of this signature
    pub fn security_level(&self) -> crate::SecurityLevel {
        self.security_level
    }

    /// Get the size of this signature in bytes
    pub fn size(&self) -> usize {
        self.bytes.len()
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Signature({:?}, {} bytes)",
            self.security_level,
            self.bytes.len()
        )
    }
}
