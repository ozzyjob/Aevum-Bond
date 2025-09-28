//! Post-Quantum Cryptography Integration Tests
//!
//! This module tests the integration of ML-DSA (CRYSTALS-Dilithium) cryptography
//! with the Bond and Aevum protocols.

use bond_core::{
    Script, Transaction, TransactionHash, TransactionInput, TransactionOutput, UtxoId,
};
use shared_crypto::{signature, Keypair, SecurityLevel};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_protocol_ml_dsa_65_integration() {
        // Test ML-DSA-65 (Level 3) integration for Bond protocol

        // Generate Bond keypair (Level 3 security)
        let keypair = Keypair::generate(SecurityLevel::Level3).unwrap();
        println!("✅ Generated Bond keypair (ML-DSA-65):");
        println!("   Public key size: {} bytes", keypair.public_key().size());
        println!(
            "   Private key size: {} bytes",
            keypair.private_key().size()
        );
        println!("   Security level: {:?}", keypair.security_level());

        // Create a test transaction
        let message = b"Bond transaction: transfer 1000 Elos";

        // Sign the message
        let signature = signature::sign(message, keypair.private_key()).unwrap();
        println!("✅ Created signature:");
        println!("   Signature size: {} bytes", signature.size());
        println!("   Security level: {:?}", signature.security_level());

        // Verify the signature
        let is_valid = signature::verify(&signature, message, keypair.public_key()).unwrap();
        assert!(is_valid, "Signature should be valid");
        println!("✅ Signature verification: PASSED");

        // Test signature with wrong message
        let wrong_message = b"Wrong message";
        let is_invalid =
            signature::verify(&signature, wrong_message, keypair.public_key()).unwrap();
        assert!(!is_invalid, "Signature should be invalid for wrong message");
        println!("✅ Invalid signature rejection: PASSED");
    }

    #[test]
    fn test_aevum_protocol_ml_dsa_44_integration() {
        // Test ML-DSA-44 (Level 2) integration for Aevum protocol

        // Generate Aevum keypair (Level 2 security)
        let keypair = Keypair::generate(SecurityLevel::Level2).unwrap();
        println!("✅ Generated Aevum keypair (ML-DSA-44):");
        println!("   Public key size: {} bytes", keypair.public_key().size());
        println!(
            "   Private key size: {} bytes",
            keypair.private_key().size()
        );
        println!("   Security level: {:?}", keypair.security_level());

        // Create a test transaction
        let message = b"Aevum transaction: fast payment";

        // Sign the message
        let signature = signature::sign(message, keypair.private_key()).unwrap();
        println!("✅ Created signature:");
        println!("   Signature size: {} bytes", signature.size());
        println!("   Security level: {:?}", signature.security_level());

        // Verify the signature
        let is_valid = signature::verify(&signature, message, keypair.public_key()).unwrap();
        assert!(is_valid, "Signature should be valid");
        println!("✅ Signature verification: PASSED");
    }

    #[test]
    fn test_transaction_signing_workflow() {
        // Test complete transaction signing workflow

        // Generate keypairs for sender and receiver
        let sender_keypair = Keypair::generate(SecurityLevel::Level3).unwrap();
        let receiver_keypair = Keypair::generate(SecurityLevel::Level3).unwrap();

        // Create a mock transaction
        let tx_hash = TransactionHash::from_bytes([1u8; 32]);
        let utxo_id = UtxoId::new(tx_hash, 0);

        // Create transaction input with signature placeholder
        let input = TransactionInput::new(
            utxo_id,
            Script::empty(), // Will be filled with signature
            0,
        );

        // Create transaction output
        let output = TransactionOutput::new(
            1000,            // 1000 Elos
            Script::empty(), // Simplified for test
        );

        // Create transaction
        let mut transaction = Transaction::new(1, vec![input], vec![output], 0);

        // Calculate signature hash
        let message = transaction.signature_hash(&transaction.inputs[0]).unwrap();

        // Sign the transaction
        let signature = signature::sign(&message, sender_keypair.private_key()).unwrap();

        // Create script with signature and public key
        let mut script_data = Vec::new();
        script_data.extend_from_slice(signature.as_bytes());
        script_data.extend_from_slice(sender_keypair.public_key().as_bytes());

        // Update transaction input with signature
        transaction.inputs[0].script_sig = Script::new(script_data);

        println!("✅ Transaction signed successfully:");
        println!("   Transaction size: {} bytes", transaction.size().unwrap());
        println!(
            "   Input script size: {} bytes",
            transaction.inputs[0].script_sig.size()
        );

        // Verify signature extraction works
        let utxo_set = std::collections::HashMap::new();
        let extraction_result =
            transaction.extract_signature_and_key(&transaction.inputs[0], &transaction.outputs[0]);
        assert!(extraction_result.is_ok());

        if let Ok(Some((extracted_sig, extracted_key))) = extraction_result {
            assert_eq!(extracted_sig.as_bytes(), signature.as_bytes());
            assert_eq!(
                extracted_key.as_bytes(),
                sender_keypair.public_key().as_bytes()
            );
            println!("✅ Signature extraction: PASSED");
        }
    }

    #[test]
    fn test_security_level_compatibility() {
        // Test that different security levels are properly handled

        let level2_keypair = Keypair::generate(SecurityLevel::Level2).unwrap();
        let level3_keypair = Keypair::generate(SecurityLevel::Level3).unwrap();

        let message = b"Test message";

        // Sign with Level 2
        let level2_signature = signature::sign(message, level2_keypair.private_key()).unwrap();

        // Try to verify with Level 3 key (should fail)
        let result = signature::verify(&level2_signature, message, level3_keypair.public_key());
        assert!(
            result.is_err(),
            "Should fail due to security level mismatch"
        );

        println!("✅ Security level mismatch detection: PASSED");
    }

    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;

        println!("🚀 Performance Testing:");

        // Test Level 2 (Aevum) performance
        let start = Instant::now();
        let level2_keypair = Keypair::generate(SecurityLevel::Level2).unwrap();
        let level2_keygen_time = start.elapsed();
        println!("   Level 2 key generation: {:?}", level2_keygen_time);

        let message = b"Performance test message";

        let start = Instant::now();
        let level2_signature = signature::sign(message, level2_keypair.private_key()).unwrap();
        let level2_sign_time = start.elapsed();
        println!("   Level 2 signing time: {:?}", level2_sign_time);

        let start = Instant::now();
        let _is_valid =
            signature::verify(&level2_signature, message, level2_keypair.public_key()).unwrap();
        let level2_verify_time = start.elapsed();
        println!("   Level 2 verification time: {:?}", level2_verify_time);

        // Test Level 3 (Bond) performance
        let start = Instant::now();
        let level3_keypair = Keypair::generate(SecurityLevel::Level3).unwrap();
        let level3_keygen_time = start.elapsed();
        println!("   Level 3 key generation: {:?}", level3_keygen_time);

        let start = Instant::now();
        let level3_signature = signature::sign(message, level3_keypair.private_key()).unwrap();
        let level3_sign_time = start.elapsed();
        println!("   Level 3 signing time: {:?}", level3_sign_time);

        let start = Instant::now();
        let _is_valid =
            signature::verify(&level3_signature, message, level3_keypair.public_key()).unwrap();
        let level3_verify_time = start.elapsed();
        println!("   Level 3 verification time: {:?}", level3_verify_time);

        println!("✅ Performance testing completed");
    }
}
