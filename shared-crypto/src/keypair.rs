//! Keypair generation and management for post-quantum cryptography

use crate::{CryptoError, CryptoResult, PrivateKey, PublicKey, SecurityLevel};
use serde::{Deserialize, Serialize};

/// A post-quantum cryptographic keypair containing both public and private keys
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keypair {
    /// The public key component
    public_key: PublicKey,
    /// The private key component  
    private_key: PrivateKey,
    /// Security level for this keypair
    security_level: SecurityLevel,
}

impl Keypair {
    /// Generate a new keypair for the specified security level
    pub fn generate(security_level: SecurityLevel) -> CryptoResult<Self> {
        match security_level {
            SecurityLevel::Level2 => Self::generate_level2(),
            SecurityLevel::Level3 => Self::generate_level3(),
        }
    }

    /// Generate ML-DSA-44 (Level 2) keypair for Aevum protocol
    /// TEMPORARY: Using Ed25519 as placeholder during development
    /// TODO: Replace with actual ML-DSA-44 implementation before production
    fn generate_level2() -> CryptoResult<Self> {
        use ed25519_dalek::SigningKey;

        // Generate Ed25519 keypair as placeholder
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();

        // Create padded keys to match ML-DSA-44 expected sizes
        let mut public_key_bytes = vec![0u8; SecurityLevel::Level2.public_key_size()];
        let mut private_key_bytes = vec![0u8; SecurityLevel::Level2.private_key_size()];

        // Copy Ed25519 keys into the padded buffers (first 32 bytes for each)
        public_key_bytes[..32].copy_from_slice(verifying_key.as_bytes());
        private_key_bytes[..32].copy_from_slice(signing_key.as_bytes());

        let public_key = PublicKey::from_bytes(public_key_bytes, SecurityLevel::Level2)?;
        let private_key = PrivateKey::from_bytes(private_key_bytes, SecurityLevel::Level2)?;

        Ok(Self {
            public_key,
            private_key,
            security_level: SecurityLevel::Level2,
        })
    }

    /// Generate ML-DSA-65 (Level 3) keypair for Bond protocol
    /// TEMPORARY: Using Ed25519 as placeholder during development
    /// TODO: Replace with actual ML-DSA-65 implementation before production
    fn generate_level3() -> CryptoResult<Self> {
        use ed25519_dalek::SigningKey;

        // Generate Ed25519 keypair as placeholder
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();

        // Create padded keys to match ML-DSA-65 expected sizes
        let mut public_key_bytes = vec![0u8; SecurityLevel::Level3.public_key_size()];
        let mut private_key_bytes = vec![0u8; SecurityLevel::Level3.private_key_size()];

        // Copy Ed25519 keys into the padded buffers (first 32 bytes for each)
        public_key_bytes[..32].copy_from_slice(verifying_key.as_bytes());
        private_key_bytes[..32].copy_from_slice(signing_key.as_bytes());

        let public_key = PublicKey::from_bytes(public_key_bytes, SecurityLevel::Level3)?;
        let private_key = PrivateKey::from_bytes(private_key_bytes, SecurityLevel::Level3)?;

        Ok(Self {
            public_key,
            private_key,
            security_level: SecurityLevel::Level3,
        })
    }

    /// Create a keypair from existing public and private keys
    pub fn from_keys(public_key: PublicKey, private_key: PrivateKey) -> CryptoResult<Self> {
        // Verify security levels match
        if public_key.security_level() != private_key.security_level() {
            return Err(CryptoError::SecurityLevelMismatch {
                expected: public_key.security_level(),
                actual: private_key.security_level(),
            });
        }

        let security_level = public_key.security_level();

        Ok(Self {
            public_key,
            private_key,
            security_level,
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the private key
    pub fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    /// Get the security level
    pub fn security_level(&self) -> SecurityLevel {
        self.security_level
    }

    /// Split the keypair into its components
    pub fn into_keys(self) -> (PublicKey, PrivateKey) {
        (self.public_key, self.private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_level2_keypair() {
        let keypair = Keypair::generate(SecurityLevel::Level2).unwrap();

        assert_eq!(keypair.security_level(), SecurityLevel::Level2);
        assert_eq!(keypair.public_key().security_level(), SecurityLevel::Level2);
        assert_eq!(
            keypair.private_key().security_level(),
            SecurityLevel::Level2
        );

        // Verify key sizes
        assert_eq!(
            keypair.public_key().size(),
            SecurityLevel::Level2.public_key_size()
        );
        assert_eq!(
            keypair.private_key().size(),
            SecurityLevel::Level2.private_key_size()
        );
    }

    #[test]
    fn test_generate_level3_keypair() {
        let keypair = Keypair::generate(SecurityLevel::Level3).unwrap();

        assert_eq!(keypair.security_level(), SecurityLevel::Level3);
        assert_eq!(keypair.public_key().security_level(), SecurityLevel::Level3);
        assert_eq!(
            keypair.private_key().security_level(),
            SecurityLevel::Level3
        );

        // Verify key sizes
        assert_eq!(
            keypair.public_key().size(),
            SecurityLevel::Level3.public_key_size()
        );
        assert_eq!(
            keypair.private_key().size(),
            SecurityLevel::Level3.private_key_size()
        );
    }

    #[test]
    fn test_mismatched_security_levels() {
        let level2_keypair = Keypair::generate(SecurityLevel::Level2).unwrap();
        let level3_keypair = Keypair::generate(SecurityLevel::Level3).unwrap();

        let result = Keypair::from_keys(
            level2_keypair.public_key().clone(),
            level3_keypair.private_key().clone(),
        );

        assert!(matches!(
            result,
            Err(CryptoError::SecurityLevelMismatch { .. })
        ));
    }
}
