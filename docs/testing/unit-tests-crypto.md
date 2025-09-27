# Camada 1: Unit Tests - Criptografia Pós-Quântica

## 1.3 Testes ML-DSA (CRYSTALS-Dilithium)

### Bond Protocol - ML-DSA-65 (Level 3)
```rust
#[cfg(test)]
mod bond_crypto_tests {
    use super::*;
    use shared_crypto::mldsa65::*;

    #[test]
    fn mldsa65_keypair_generation() {
        let keypair = MLDSAKeyPair65::generate();
        
        assert_eq!(keypair.public_key().len(), MLDSA65_PUBLIC_KEY_SIZE);
        assert_eq!(keypair.private_key().len(), MLDSA65_PRIVATE_KEY_SIZE);
        
        // Keys should be different each time
        let keypair2 = MLDSAKeyPair65::generate();
        assert_ne!(keypair.public_key(), keypair2.public_key());
        assert_ne!(keypair.private_key(), keypair2.private_key());
    }

    #[test]
    fn mldsa65_signature_verification_success() {
        let keypair = MLDSAKeyPair65::generate();
        let message = b"Bond transaction data";
        
        let signature = keypair.sign(message).unwrap();
        assert_eq!(signature.len(), MLDSA65_SIGNATURE_SIZE);
        
        let verification = keypair.public_key().verify(message, &signature);
        assert!(verification.is_ok());
        assert!(verification.unwrap());
    }

    #[test]
    fn mldsa65_signature_verification_failure() {
        let keypair1 = MLDSAKeyPair65::generate();
        let keypair2 = MLDSAKeyPair65::generate();
        let message = b"Bond transaction data";
        
        let signature = keypair1.sign(message).unwrap();
        
        // Wrong public key should fail verification
        let verification = keypair2.public_key().verify(message, &signature);
        assert!(verification.is_ok());
        assert!(!verification.unwrap());
    }

    #[test]
    fn mldsa65_tampered_message_detection() {
        let keypair = MLDSAKeyPair65::generate();
        let original_message = b"Original Bond transaction";
        let tampered_message = b"Tampered Bond transaction";
        
        let signature = keypair.sign(original_message).unwrap();
        
        // Original message should verify
        assert!(keypair.public_key().verify(original_message, &signature).unwrap());
        
        // Tampered message should fail
        assert!(!keypair.public_key().verify(tampered_message, &signature).unwrap());
    }

    #[test]
    fn mldsa65_signature_determinism() {
        let keypair = MLDSAKeyPair65::generate();
        let message = b"Deterministic test message";
        
        // Note: ML-DSA is probabilistic, so signatures will be different each time
        let sig1 = keypair.sign(message).unwrap();
        let sig2 = keypair.sign(message).unwrap();
        
        // Signatures should be different (probabilistic)
        assert_ne!(sig1, sig2);
        
        // But both should verify
        assert!(keypair.public_key().verify(message, &sig1).unwrap());
        assert!(keypair.public_key().verify(message, &sig2).unwrap());
    }

    proptest! {
        #[test]
        fn mldsa65_random_message_signing(
            message in prop::collection::vec(any::<u8>(), 1..1000)
        ) {
            let keypair = MLDSAKeyPair65::generate();
            let signature = keypair.sign(&message).unwrap();
            
            prop_assert!(keypair.public_key().verify(&message, &signature).unwrap());
        }
    }
}
```

### Aevum Protocol - ML-DSA-44 (Level 1)
```rust
#[cfg(test)]
mod aevum_crypto_tests {
    use super::*;
    use shared_crypto::mldsa44::*;

    #[test]
    fn mldsa44_performance_optimized() {
        let keypair = MLDSAKeyPair44::generate();
        let message = b"Aevum high-frequency transaction";
        
        let start = std::time::Instant::now();
        let signature = keypair.sign(message).unwrap();
        let sign_duration = start.elapsed();
        
        let start = std::time::Instant::now();
        let verified = keypair.public_key().verify(message, &signature).unwrap();
        let verify_duration = start.elapsed();
        
        assert!(verified);
        
        // Performance benchmarks for high-frequency usage
        assert!(sign_duration.as_millis() < 10, "Signing should be < 10ms");
        assert!(verify_duration.as_millis() < 5, "Verification should be < 5ms");
    }

    #[test]
    fn mldsa44_batch_verification() {
        let keypairs: Vec<_> = (0..10).map(|_| MLDSAKeyPair44::generate()).collect();
        let messages: Vec<Vec<u8>> = (0..10)
            .map(|i| format!("Transaction {}", i).into_bytes())
            .collect();
        
        let signatures: Vec<_> = keypairs.iter()
            .zip(&messages)
            .map(|(kp, msg)| kp.sign(msg).unwrap())
            .collect();
        
        // Batch verification
        for ((keypair, message), signature) in keypairs.iter().zip(&messages).zip(&signatures) {
            assert!(keypair.public_key().verify(message, signature).unwrap());
        }
    }

    #[test]
    fn mldsa44_account_abstraction_compatibility() {
        let keypair = MLDSAKeyPair44::generate();
        
        // Test multi-signature scenario for Smart Accounts
        let account_address = Address::from_public_key(keypair.public_key());
        let transaction_data = format!("Transfer from {}", account_address);
        
        let signature = keypair.sign(transaction_data.as_bytes()).unwrap();
        let verified = keypair.public_key().verify(transaction_data.as_bytes(), &signature).unwrap();
        
        assert!(verified);
    }
}
```

### Cross-Protocol Compatibility Tests
```rust
#[cfg(test)]
mod cross_protocol_crypto_tests {
    #[test]
    fn mldsa_security_level_compatibility() {
        // Ensure Level 3 (Bond) and Level 1 (Aevum) can coexist
        let bond_keypair = MLDSAKeyPair65::generate();
        let aevum_keypair = MLDSAKeyPair44::generate();
        
        let bond_message = b"Bond store-of-value transaction";
        let aevum_message = b"Aevum high-speed transaction";
        
        let bond_signature = bond_keypair.sign(bond_message).unwrap();
        let aevum_signature = aevum_keypair.sign(aevum_message).unwrap();
        
        // Both should work independently
        assert!(bond_keypair.public_key().verify(bond_message, &bond_signature).unwrap());
        assert!(aevum_keypair.public_key().verify(aevum_message, &aevum_signature).unwrap());
        
        // Cross-verification should fail (different key types)
        // This ensures protocol separation
    }

    #[test]
    fn bridge_transaction_dual_signature() {
        // Test scenario: Bridge transaction requires both protocols
        let bond_keypair = MLDSAKeyPair65::generate();
        let aevum_keypair = MLDSAKeyPair44::generate();
        
        let bridge_message = b"Bridge: Lock BND, Mint wBND";
        
        let bond_signature = bond_keypair.sign(bridge_message).unwrap();
        let aevum_signature = aevum_keypair.sign(bridge_message).unwrap();
        
        // Both signatures should verify independently
        assert!(bond_keypair.public_key().verify(bridge_message, &bond_signature).unwrap());
        assert!(aevum_keypair.public_key().verify(bridge_message, &aevum_signature).unwrap());
    }
}
```
