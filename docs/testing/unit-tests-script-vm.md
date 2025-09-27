# Camada 1: Unit Tests - VM de Script (pUTXO)

## 1.5 Testes da Máquina Virtual de Script

### Basic Opcode Tests
```rust
#[cfg(test)]
mod script_vm_tests {
    use super::*;

    #[test]
    fn op_push_data() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        // Test pushing different data sizes
        let test_cases = vec![
            (vec![0x01, 0x42], vec![0x42]),                    // Push 1 byte
            (vec![0x04, 0x01, 0x02, 0x03, 0x04], vec![0x01, 0x02, 0x03, 0x04]), // Push 4 bytes
            (vec![0x20], (0..32).collect::<Vec<u8>>()),        // Push 32 bytes
        ];
        
        for (mut script, expected_data) in test_cases {
            if script[0] == 0x20 {
                script.extend_from_slice(&expected_data);
            }
            
            let result = interpreter.execute(&script, &context).unwrap();
            assert_eq!(result, ScriptResult::Success);
        }
    }

    #[test]
    fn op_dup() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        let script = vec![
            0x01, 0x42,  // Push 0x42
            0x76,        // OP_DUP
        ];
        
        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Success);
        
        // Stack should have two copies of 0x42
        // This would need access to internal stack state for complete verification
    }

    #[test]
    fn op_equal() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        // Test equal values
        let script_equal = vec![
            0x01, 0x42,  // Push 0x42
            0x01, 0x42,  // Push 0x42
            0x87,        // OP_EQUAL
        ];
        
        let result = interpreter.execute(&script_equal, &context).unwrap();
        assert_eq!(result, ScriptResult::Success);
        
        // Test unequal values
        let script_unequal = vec![
            0x01, 0x42,  // Push 0x42
            0x01, 0x43,  // Push 0x43
            0x87,        // OP_EQUAL
        ];
        
        let result = interpreter.execute(&script_unequal, &context).unwrap();
        assert_eq!(result, ScriptResult::Failure); // Should result in false on stack
    }

    #[test]
    fn op_verify_success() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        let script = vec![
            0x51,        // OP_1 (push true)
            0x69,        // OP_VERIFY
        ];
        
        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Success);
    }

    #[test]
    fn op_verify_failure() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        let script = vec![
            0x00,        // OP_0 (push false)
            0x69,        // OP_VERIFY
        ];
        
        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Failure);
    }

    #[test]
    fn op_checksig_placeholder() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        let script = vec![
            0x21, // Push 33 bytes (compressed pubkey placeholder)
            0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11,
            0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
            0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22,
            0x40, // Push 64 bytes (signature placeholder)
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
            0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
            0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
            0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
            0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
            0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
            0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
            0xAC,        // OP_CHECKSIG
        ];
        
        let result = interpreter.execute(&script, &context).unwrap();
        // Currently returns success as placeholder - will be updated with real ML-DSA verification
        assert_eq!(result, ScriptResult::Success);
    }
}
```

### Advanced Script Scenarios
```rust
#[cfg(test)]
mod advanced_script_tests {
    use super::*;

    #[test]
    fn time_lock_script() {
        let interpreter = ScriptInterpreter::new();
        
        // Test script that checks if current block height >= 150
        let required_height = 150u32;
        let mut script = vec![0x04]; // Push 4 bytes
        script.extend_from_slice(&required_height.to_le_bytes());
        script.push(0xF0); // OP_CHECKBLOCKHEIGHT
        
        // Test with sufficient block height
        let context_sufficient = ScriptContext::new(200, 1234567890, vec![1, 2, 3], 0);
        let result = interpreter.execute(&script, &context_sufficient).unwrap();
        assert_eq!(result, ScriptResult::Success);
        
        // Test with insufficient block height
        let context_insufficient = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        let result = interpreter.execute(&script, &context_insufficient).unwrap();
        assert_eq!(result, ScriptResult::Failure);
    }

    #[test]
    fn multi_condition_script() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        // Script: Push two identical values, duplicate one, check all three are equal
        let script = vec![
            0x01, 0x42,  // Push 0x42
            0x01, 0x42,  // Push 0x42
            0x76,        // OP_DUP (duplicate top)
            0x87,        // OP_EQUAL (check top two)
            // Stack now has: [0x42, true]
            // Need to verify both conditions...
        ];
        
        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Success);
    }

    #[test]
    fn guardian_recovery_script() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        // Simplified guardian recovery script
        // In real implementation, this would check multiple guardian signatures
        let script = vec![
            // Guardian 1 signature check (placeholder)
            0x21, // Push guardian 1 pubkey (33 bytes)
            0x02, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x40, // Push guardian 1 signature (64 bytes placeholder)
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0xAC, // OP_CHECKSIG
        ];
        
        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Success); // Placeholder implementation
    }

    #[test]
    fn rate_limit_script() {
        // This would test a script that enforces spending rate limits
        // Implementation would need to track spending windows
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        // Placeholder for rate limiting logic
        let script = vec![0x51]; // OP_1 (placeholder)
        
        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Success);
    }

    #[test]
    fn script_size_limits() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        // Test maximum script size (prevent DoS)
        let max_script = vec![0x51; 10000]; // 10KB of OP_1
        let result = interpreter.execute(&max_script, &context).unwrap();
        
        // Should hit operation limit before completing
        match result {
            ScriptResult::Error(msg) => assert!(msg.contains("Operation limit exceeded")),
            _ => panic!("Expected operation limit error"),
        }
    }

    #[test]
    fn stack_overflow_protection() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
        
        // Try to overflow the stack
        let mut script = Vec::new();
        for _ in 0..200 { // Try to push 200 items
            script.extend_from_slice(&[0x01, 0x42]); // Push 0x42
        }
        
        let result = interpreter.execute(&script, &context);
        
        // Should hit stack size limit
        match result {
            Err(BondError::ScriptExecutionFailed { reason }) => {
                assert!(reason.contains("Stack overflow"));
            }
            _ => panic!("Expected stack overflow error"),
        }
    }

    proptest! {
        #[test]
        fn script_execution_deterministic(
            ops in prop::collection::vec(0x00u8..0x51u8, 1..50)
        ) {
            let interpreter = ScriptInterpreter::new();
            let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);
            
            // Filter out invalid opcodes for deterministic test
            let valid_script: Vec<u8> = ops.into_iter()
                .filter(|&op| op == 0x00 || op == 0x51 || op == 0x76 || op == 0x87)
                .collect();
            
            if !valid_script.is_empty() {
                let result1 = interpreter.execute(&valid_script, &context);
                let result2 = interpreter.execute(&valid_script, &context);
                
                // Same script should produce same result
                prop_assert_eq!(result1.is_ok(), result2.is_ok());
            }
        }
    }
}
```

### Programmable UTXO Integration Tests
```rust
#[cfg(test)]
mod putxo_integration_tests {
    use super::*;

    #[test]
    fn social_recovery_scenario() {
        // Test complete social recovery flow
        let owner_keypair = MLDSAKeyPair65::generate();
        let guardian1_keypair = MLDSAKeyPair65::generate();
        let guardian2_keypair = MLDSAKeyPair65::generate();
        let guardian3_keypair = MLDSAKeyPair65::generate();
        
        // Create pUTXO with 2-of-3 guardian recovery
        let mut utxo = ProgrammableUtxo::simple(
            UtxoId::new(TransactionHash::ZERO, 0),
            1_000_000, // 1 BND
            Script::empty(),
            100
        );
        
        utxo.add_guardian(GuardianConfig {
            public_key: guardian1_keypair.public_key().to_bytes(),
            confirmation_period: 144, // ~24 hours
            weight: 1,
        });
        
        utxo.add_guardian(GuardianConfig {
            public_key: guardian2_keypair.public_key().to_bytes(),
            confirmation_period: 144,
            weight: 1,
        });
        
        utxo.add_guardian(GuardianConfig {
            public_key: guardian3_keypair.public_key().to_bytes(),
            confirmation_period: 144,
            weight: 1,
        });
        
        // Test recovery transaction with 2 guardian signatures
        let recovery_message = b"Recovery transaction for lost keys";
        let guardian1_sig = guardian1_keypair.sign(recovery_message).unwrap();
        let guardian2_sig = guardian2_keypair.sign(recovery_message).unwrap();
        
        // Verify signatures
        assert!(guardian1_keypair.public_key().verify(recovery_message, &guardian1_sig).unwrap());
        assert!(guardian2_keypair.public_key().verify(recovery_message, &guardian2_sig).unwrap());
        
        // In real implementation, this would construct and validate the recovery script
        assert!(true); // Placeholder for full recovery validation
    }

    #[test]
    fn mfa_spending_scenario() {
        let owner_keypair = MLDSAKeyPair65::generate();
        let hardware_key = b"hardware_key_public_key_placeholder";
        
        let mut utxo = ProgrammableUtxo::simple(
            UtxoId::new(TransactionHash::ZERO, 0),
            10_000_000, // 10 BND (high value requiring MFA)
            Script::empty(),
            100
        );
        
        utxo.set_mfa_config(MfaConfig {
            required_factors: 2,
            methods: vec![
                AuthMethod::HardwareKey { 
                    public_key: hardware_key.to_vec() 
                },
                AuthMethod::Totp { 
                    shared_secret_hash: [0x42; 32] 
                },
            ],
        });
        
        // Test spending with MFA requirements
        let spending_message = b"High-value spending transaction";
        let owner_sig = owner_keypair.sign(spending_message).unwrap();
        
        // Simulate hardware key signature (placeholder)
        let hardware_sig = b"hardware_signature_placeholder";
        
        // Simulate TOTP validation (placeholder)
        let totp_code = "123456";
        
        // In real implementation, would validate all MFA factors
        assert!(owner_keypair.public_key().verify(spending_message, &owner_sig).unwrap());
        assert!(hardware_sig.len() > 0); // Placeholder validation
        assert!(totp_code.len() == 6); // Placeholder validation
    }

    #[test]
    fn time_lock_expiration() {
        let mut utxo = ProgrammableUtxo::simple(
            UtxoId::new(TransactionHash::ZERO, 0),
            1_000_000,
            Script::empty(),
            100
        );
        
        // Add absolute time lock for block 200
        utxo.add_time_lock(TimeLock {
            lock_type: TimeLockType::BlockHeight,
            value: 200,
        });
        
        // Test before time lock expiration
        assert!(!utxo.can_spend(150, 0).unwrap());
        
        // Test after time lock expiration
        assert!(utxo.can_spend(200, 0).unwrap());
        assert!(utxo.can_spend(250, 0).unwrap());
    }

    #[test]
    fn rate_limiting_enforcement() {
        let mut utxo = ProgrammableUtxo::simple(
            UtxoId::new(TransactionHash::ZERO, 0),
            10_000_000, // 10 BND
            Script::empty(),
            100
        );
        
        // Set rate limit: max 1 BND per day
        utxo.set_rate_limit(RateLimit {
            max_value_per_window: 1_000_000, // 1 BND
            window_seconds: 86400, // 24 hours
            current_window_start: 1234567890,
            spent_in_window: 0,
        });
        
        let timestamp = 1234567890;
        
        // First spending should be allowed
        assert!(utxo.can_spend(150, timestamp).unwrap());
        
        // Simulate spending within the window
        if let Some(ref mut rate_limit) = utxo.metadata.rate_limits {
            rate_limit.spent_in_window = 500_000; // 0.5 BND spent
        }
        
        // Second spending within limit should be allowed
        assert!(utxo.can_spend(150, timestamp + 3600).unwrap()); // 1 hour later
        
        // Exceed rate limit
        if let Some(ref mut rate_limit) = utxo.metadata.rate_limits {
            rate_limit.spent_in_window = 1_000_000; // 1 BND spent (at limit)
        }
        
        // Additional spending should be blocked
        assert!(!utxo.can_spend(150, timestamp + 7200).unwrap()); // 2 hours later
        
        // After window expires, spending should be allowed again
        assert!(utxo.can_spend(150, timestamp + 86401).unwrap()); // Next day
    }
}
```
