//! Digital signature operations for post-quantum cryptography

use crate::{CryptoError, CryptoResult, PrivateKey, PublicKey, SecurityLevel, Signature};

/// Sign a message using a private key
pub fn sign(message: &[u8], private_key: &PrivateKey) -> CryptoResult<Signature> {
    match private_key.security_level() {
        SecurityLevel::Level2 => sign_level2(message, private_key),
        SecurityLevel::Level3 => sign_level3(message, private_key),
    }
}

/// Verify a signature against a message using a public key
pub fn verify(signature: &Signature, message: &[u8], public_key: &PublicKey) -> CryptoResult<bool> {
    // Ensure signature and public key have matching security levels
    if signature.security_level() != public_key.security_level() {
        return Err(CryptoError::SecurityLevelMismatch {
            expected: public_key.security_level(),
            actual: signature.security_level(),
        });
    }

    match public_key.security_level() {
        SecurityLevel::Level2 => verify_level2(signature, message, public_key),
        SecurityLevel::Level3 => verify_level3(signature, message, public_key),
    }
}

/// Sign using ML-DSA-44 (Level 2) - TEMPORARY Ed25519 implementation
/// TODO: Replace with actual ML-DSA-44 implementation before production
fn sign_level2(message: &[u8], private_key: &PrivateKey) -> CryptoResult<Signature> {
    use ed25519_dalek::{Signer, SigningKey};

    // Extract Ed25519 private key from the first 32 bytes
    let mut ed25519_bytes = [0u8; 32];
    ed25519_bytes.copy_from_slice(&private_key.as_bytes()[..32]);

    let signing_key = SigningKey::from_bytes(&ed25519_bytes);
    let ed25519_signature = signing_key.sign(message);

    // Create padded signature to match ML-DSA-44 expected size
    let mut signature_bytes = vec![0u8; SecurityLevel::Level2.signature_size()];
    signature_bytes[..64].copy_from_slice(&ed25519_signature.to_bytes());

    Signature::from_bytes(signature_bytes, SecurityLevel::Level2)
}

/// Sign using ML-DSA-65 (Level 3) - TEMPORARY Ed25519 implementation
/// TODO: Replace with actual ML-DSA-65 implementation before production
fn sign_level3(message: &[u8], private_key: &PrivateKey) -> CryptoResult<Signature> {
    use ed25519_dalek::{Signer, SigningKey};

    // Extract Ed25519 private key from the first 32 bytes
    let mut ed25519_bytes = [0u8; 32];
    ed25519_bytes.copy_from_slice(&private_key.as_bytes()[..32]);

    let signing_key = SigningKey::from_bytes(&ed25519_bytes);
    let ed25519_signature = signing_key.sign(message);

    // Create padded signature to match ML-DSA-65 expected size
    let mut signature_bytes = vec![0u8; SecurityLevel::Level3.signature_size()];
    signature_bytes[..64].copy_from_slice(&ed25519_signature.to_bytes());

    Signature::from_bytes(signature_bytes, SecurityLevel::Level3)
}

/// Verify ML-DSA-44 (Level 2) signature - TEMPORARY Ed25519 implementation
/// TODO: Replace with actual ML-DSA-44 implementation before production
fn verify_level2(
    signature: &Signature,
    message: &[u8],
    public_key: &PublicKey,
) -> CryptoResult<bool> {
    use ed25519_dalek::{Verifier, VerifyingKey};

    // Extract Ed25519 public key from the first 32 bytes
    let mut ed25519_pub_bytes = [0u8; 32];
    ed25519_pub_bytes.copy_from_slice(&public_key.as_bytes()[..32]);

    // Extract Ed25519 signature from the first 64 bytes
    let mut ed25519_sig_bytes = [0u8; 64];
    ed25519_sig_bytes.copy_from_slice(&signature.as_bytes()[..64]);

    let verifying_key =
        VerifyingKey::from_bytes(&ed25519_pub_bytes).map_err(|e| CryptoError::InvalidFormat {
            reason: format!("Invalid Ed25519 public key: {}", e),
        })?;

    let ed25519_signature = ed25519_dalek::Signature::from_bytes(&ed25519_sig_bytes);

    match verifying_key.verify(message, &ed25519_signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Verify ML-DSA-65 (Level 3) signature - TEMPORARY Ed25519 implementation
/// TODO: Replace with actual ML-DSA-65 implementation before production
fn verify_level3(
    signature: &Signature,
    message: &[u8],
    public_key: &PublicKey,
) -> CryptoResult<bool> {
    use ed25519_dalek::{Verifier, VerifyingKey};

    // Extract Ed25519 public key from the first 32 bytes
    let mut ed25519_pub_bytes = [0u8; 32];
    ed25519_pub_bytes.copy_from_slice(&public_key.as_bytes()[..32]);

    // Extract Ed25519 signature from the first 64 bytes
    let mut ed25519_sig_bytes = [0u8; 64];
    ed25519_sig_bytes.copy_from_slice(&signature.as_bytes()[..64]);

    let verifying_key =
        VerifyingKey::from_bytes(&ed25519_pub_bytes).map_err(|e| CryptoError::InvalidFormat {
            reason: format!("Invalid Ed25519 public key: {}", e),
        })?;

    let ed25519_signature = ed25519_dalek::Signature::from_bytes(&ed25519_sig_bytes);

    match verifying_key.verify(message, &ed25519_signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keypair;

    #[test]
    fn test_sign_and_verify_level2() {
        let keypair = Keypair::generate(SecurityLevel::Level2).unwrap();
        let message = b"Hello, Aevum & Bond!";

        let signature = sign(message, keypair.private_key()).unwrap();
        let is_valid = verify(&signature, message, keypair.public_key()).unwrap();

        assert!(is_valid);
        assert_eq!(signature.security_level(), SecurityLevel::Level2);
        assert_eq!(signature.size(), SecurityLevel::Level2.signature_size());
    }

    #[test]
    fn test_sign_and_verify_level3() {
        let keypair = Keypair::generate(SecurityLevel::Level3).unwrap();
        let message = b"Hello, Aevum & Bond!";

        let signature = sign(message, keypair.private_key()).unwrap();
        let is_valid = verify(&signature, message, keypair.public_key()).unwrap();

        assert!(is_valid);
        assert_eq!(signature.security_level(), SecurityLevel::Level3);
        assert_eq!(signature.size(), SecurityLevel::Level3.signature_size());
    }

    #[test]
    fn test_invalid_signature() {
        let keypair = Keypair::generate(SecurityLevel::Level2).unwrap();
        let message = b"Hello, Aevum & Bond!";
        let wrong_message = b"Wrong message";

        let signature = sign(message, keypair.private_key()).unwrap();
        let is_valid = verify(&signature, wrong_message, keypair.public_key()).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_security_level_mismatch() {
        let keypair_l2 = Keypair::generate(SecurityLevel::Level2).unwrap();
        let keypair_l3 = Keypair::generate(SecurityLevel::Level3).unwrap();
        let message = b"Hello, Aevum & Bond!";

        let signature_l2 = sign(message, keypair_l2.private_key()).unwrap();

        // Try to verify Level 2 signature with Level 3 public key
        let result = verify(&signature_l2, message, keypair_l3.public_key());

        assert!(matches!(
            result,
            Err(CryptoError::SecurityLevelMismatch { .. })
        ));
    }
}
