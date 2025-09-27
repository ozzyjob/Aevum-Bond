# Camada 1: Unit Tests - Transações

## 1.6 Testes de Transações e Inputs/Outputs

### Transaction Structure Tests
```rust
#[cfg(test)]
mod transaction_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn transaction_creation() {
        let inputs = vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: Script::from_bytes(vec![0x51]), // OP_1
            }
        ];
        
        let outputs = vec![
            TransactionOutput {
                value: 1_000_000, // 1 BND
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]), // OP_DUP OP_CHECKSIG
            }
        ];
        
        let transaction = Transaction::new(inputs, outputs).unwrap();
        
        assert_eq!(transaction.inputs.len(), 1);
        assert_eq!(transaction.outputs.len(), 1);
        assert_eq!(transaction.outputs[0].value, 1_000_000);
        assert!(transaction.hash() != TransactionHash::ZERO);
    }

    #[test]
    fn transaction_serialization() {
        let inputs = vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }
        ];
        
        let outputs = vec![
            TransactionOutput {
                value: 5_000_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }
        ];
        
        let original = Transaction::new(inputs, outputs).unwrap();
        
        // Test bincode serialization
        let serialized = bincode::serialize(&original).unwrap();
        let deserialized: Transaction = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(original.hash(), deserialized.hash());
        assert_eq!(original.inputs.len(), deserialized.inputs.len());
        assert_eq!(original.outputs.len(), deserialized.outputs.len());
        assert_eq!(original.outputs[0].value, deserialized.outputs[0].value);
    }

    #[test]
    fn transaction_hash_deterministic() {
        let inputs = vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }
        ];
        
        let outputs = vec![
            TransactionOutput {
                value: 2_500_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }
        ];
        
        let tx1 = Transaction::new(inputs.clone(), outputs.clone()).unwrap();
        let tx2 = Transaction::new(inputs, outputs).unwrap();
        
        // Same transaction data should produce same hash
        assert_eq!(tx1.hash(), tx2.hash());
    }

    #[test]
    fn transaction_validation() {
        // Valid transaction
        let valid_inputs = vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }
        ];
        
        let valid_outputs = vec![
            TransactionOutput {
                value: 1_000_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }
        ];
        
        let valid_tx = Transaction::new(valid_inputs, valid_outputs);
        assert!(valid_tx.is_ok());
        
        // Invalid: no inputs
        let invalid_tx = Transaction::new(vec![], vec![
            TransactionOutput {
                value: 1_000_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }
        ]);
        assert!(invalid_tx.is_err());
        
        // Invalid: no outputs
        let invalid_tx2 = Transaction::new(vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }
        ], vec![]);
        assert!(invalid_tx2.is_err());
        
        // Invalid: zero value output
        let invalid_tx3 = Transaction::new(vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }
        ], vec![
            TransactionOutput {
                value: 0,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }
        ]);
        assert!(invalid_tx3.is_err());
    }

    #[test]
    fn transaction_fee_calculation() {
        let inputs = vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }
        ];
        
        let outputs = vec![
            TransactionOutput {
                value: 999_000, // 0.999 BND (leaving 0.001 BND as fee)
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }
        ];
        
        let transaction = Transaction::new(inputs, outputs).unwrap();
        
        // Mock UTXO set with input value
        let mut utxo_set = HashMap::new();
        utxo_set.insert(UtxoId::new(TransactionHash::ZERO, 0), Utxo {
            value: 1_000_000, // 1 BND input
            script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            block_height: 100,
        });
        
        let fee = transaction.calculate_fee(&utxo_set).unwrap();
        assert_eq!(fee, 1_000); // 0.001 BND fee
    }

    proptest! {
        #[test]
        fn transaction_serialization_roundtrip(
            num_inputs in 1usize..10,
            num_outputs in 1usize..10,
            values in prop::collection::vec(1u64..1_000_000_000, 1..10)
        ) {
            let inputs: Vec<TransactionInput> = (0..num_inputs).map(|i| {
                TransactionInput {
                    utxo_id: UtxoId::new(TransactionHash::ZERO, i as u32),
                    script_sig: Script::from_bytes(vec![0x51]),
                }
            }).collect();
            
            let outputs: Vec<TransactionOutput> = values.into_iter().take(num_outputs).map(|value| {
                TransactionOutput {
                    value,
                    script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
                }
            }).collect();
            
            let original = Transaction::new(inputs, outputs).unwrap();
            let serialized = bincode::serialize(&original).unwrap();
            let deserialized: Transaction = bincode::deserialize(&serialized).unwrap();
            
            prop_assert_eq!(original.hash(), deserialized.hash());
        }
    }
}
```

### Transaction Input Tests
```rust
#[cfg(test)]
mod transaction_input_tests {
    use super::*;

    #[test]
    fn transaction_input_creation() {
        let tx_hash = TransactionHash::from_bytes([1; 32]);
        let output_index = 5u32;
        let script_sig = Script::from_bytes(vec![0x51, 0x76, 0xAC]);
        
        let input = TransactionInput {
            utxo_id: UtxoId::new(tx_hash, output_index),
            script_sig,
        };
        
        assert_eq!(input.utxo_id.transaction_hash, tx_hash);
        assert_eq!(input.utxo_id.output_index, output_index);
        assert_eq!(input.script_sig.to_bytes(), vec![0x51, 0x76, 0xAC]);
    }

    #[test]
    fn transaction_input_serialization() {
        let input = TransactionInput {
            utxo_id: UtxoId::new(TransactionHash::from_bytes([2; 32]), 10),
            script_sig: Script::from_bytes(vec![0x76, 0x88, 0xAC]),
        };
        
        let serialized = bincode::serialize(&input).unwrap();
        let deserialized: TransactionInput = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(input.utxo_id, deserialized.utxo_id);
        assert_eq!(input.script_sig.to_bytes(), deserialized.script_sig.to_bytes());
    }

    #[test]
    fn utxo_id_uniqueness() {
        let tx_hash1 = TransactionHash::from_bytes([1; 32]);
        let tx_hash2 = TransactionHash::from_bytes([2; 32]);
        
        let utxo_id1 = UtxoId::new(tx_hash1, 0);
        let utxo_id2 = UtxoId::new(tx_hash1, 1);
        let utxo_id3 = UtxoId::new(tx_hash2, 0);
        
        assert_ne!(utxo_id1, utxo_id2); // Same tx, different output
        assert_ne!(utxo_id1, utxo_id3); // Different tx, same output
        assert_ne!(utxo_id2, utxo_id3); // Different tx, different output
    }

    #[test]
    fn script_sig_validation() {
        // Valid script signatures
        let valid_scripts = vec![
            vec![0x51], // OP_1
            vec![0x76, 0xAC], // OP_DUP OP_CHECKSIG
            vec![0x21] + vec![0x02; 33] + vec![0x40] + vec![0x30; 64] + vec![0xAC], // Pubkey + Sig + OP_CHECKSIG
        ];
        
        for script_bytes in valid_scripts {
            let script = Script::from_bytes(script_bytes.clone());
            let input = TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
                script_sig: script,
            };
            
            assert_eq!(input.script_sig.to_bytes(), script_bytes);
        }
        
        // Test maximum script size
        let max_script = Script::from_bytes(vec![0x51; 10_000]); // 10KB
        let input = TransactionInput {
            utxo_id: UtxoId::new(TransactionHash::ZERO, 0),
            script_sig: max_script,
        };
        assert_eq!(input.script_sig.to_bytes().len(), 10_000);
    }
}
```

### Transaction Output Tests
```rust
#[cfg(test)]
mod transaction_output_tests {
    use super::*;

    #[test]
    fn transaction_output_creation() {
        let value = 1_500_000u64; // 1.5 BND
        let script_pubkey = Script::from_bytes(vec![0x76, 0xA9, 0xAC]); // OP_DUP OP_HASH160 OP_CHECKSIG
        
        let output = TransactionOutput {
            value,
            script_pubkey,
        };
        
        assert_eq!(output.value, value);
        assert_eq!(output.script_pubkey.to_bytes(), vec![0x76, 0xA9, 0xAC]);
    }

    #[test]
    fn transaction_output_serialization() {
        let output = TransactionOutput {
            value: 2_750_000,
            script_pubkey: Script::from_bytes(vec![0x51, 0x52, 0x53]),
        };
        
        let serialized = bincode::serialize(&output).unwrap();
        let deserialized: TransactionOutput = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(output.value, deserialized.value);
        assert_eq!(output.script_pubkey.to_bytes(), deserialized.script_pubkey.to_bytes());
    }

    #[test]
    fn output_value_validation() {
        // Test minimum value (1 satoshi = 0.000001 BND)
        let min_output = TransactionOutput {
            value: 1,
            script_pubkey: Script::from_bytes(vec![0x51]),
        };
        assert_eq!(min_output.value, 1);
        
        // Test maximum value (21M BND * 1M satoshis/BND)
        let max_output = TransactionOutput {
            value: 21_000_000_000_000u64,
            script_pubkey: Script::from_bytes(vec![0x51]),
        };
        assert_eq!(max_output.value, 21_000_000_000_000u64);
    }

    #[test]
    fn common_output_scripts() {
        // P2PK (Pay to Public Key)
        let p2pk_script = Script::from_bytes({
            let mut script = vec![0x21]; // Push 33 bytes
            script.extend_from_slice(&[0x02; 33]); // Compressed pubkey
            script.push(0xAC); // OP_CHECKSIG
            script
        });
        
        let p2pk_output = TransactionOutput {
            value: 1_000_000,
            script_pubkey: p2pk_script,
        };
        
        assert_eq!(p2pk_output.script_pubkey.to_bytes().len(), 35); // 1 + 33 + 1
        
        // P2PKH (Pay to Public Key Hash) - simplified
        let p2pkh_script = Script::from_bytes(vec![
            0x76, // OP_DUP
            0xA9, // OP_HASH160
            0x14, // Push 20 bytes
        ] + vec![0x12; 20] + vec![ // 20-byte hash
            0x88, // OP_EQUALVERIFY
            0xAC, // OP_CHECKSIG
        ]);
        
        let p2pkh_output = TransactionOutput {
            value: 500_000,
            script_pubkey: p2pkh_script,
        };
        
        assert_eq!(p2pkh_output.script_pubkey.to_bytes().len(), 25); // Standard P2PKH length
    }

    #[test]
    fn output_dust_limits() {
        // Test dust threshold (546 satoshis is Bitcoin's dust limit)
        let dust_threshold = 546u64;
        
        let dust_output = TransactionOutput {
            value: dust_threshold - 1,
            script_pubkey: Script::from_bytes(vec![0x51]),
        };
        
        let valid_output = TransactionOutput {
            value: dust_threshold,
            script_pubkey: Script::from_bytes(vec![0x51]),
        };
        
        // In production, dust outputs should be rejected
        assert!(dust_output.value < dust_threshold);
        assert!(valid_output.value >= dust_threshold);
    }
}
```

### Transaction Validation Integration Tests
```rust
#[cfg(test)]
mod transaction_validation_tests {
    use super::*;

    #[test]
    fn complete_transaction_validation() {
        // Create a simple valid transaction
        let prev_tx_hash = TransactionHash::from_bytes([5; 32]);
        let inputs = vec![
            TransactionInput {
                utxo_id: UtxoId::new(prev_tx_hash, 0),
                script_sig: Script::from_bytes(vec![0x51]), // OP_1 (placeholder signature)
            }
        ];
        
        let outputs = vec![
            TransactionOutput {
                value: 999_000, // 0.999 BND
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]), // OP_DUP OP_CHECKSIG
            }
        ];
        
        let transaction = Transaction::new(inputs, outputs).unwrap();
        
        // Mock UTXO set
        let mut utxo_set = HashMap::new();
        utxo_set.insert(UtxoId::new(prev_tx_hash, 0), Utxo {
            value: 1_000_000, // 1 BND
            script_pubkey: Script::from_bytes(vec![0x51]), // OP_1 (anyone can spend for test)
            block_height: 50,
        });
        
        // Validate transaction
        let validation_result = validate_transaction(&transaction, &utxo_set, 100);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn double_spend_detection() {
        let prev_tx_hash = TransactionHash::from_bytes([6; 32]);
        let utxo_id = UtxoId::new(prev_tx_hash, 0);
        
        // First transaction spending the UTXO
        let tx1 = Transaction::new(
            vec![TransactionInput {
                utxo_id,
                script_sig: Script::from_bytes(vec![0x51]),
            }],
            vec![TransactionOutput {
                value: 500_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }]
        ).unwrap();
        
        // Second transaction trying to spend the same UTXO
        let tx2 = Transaction::new(
            vec![TransactionInput {
                utxo_id,
                script_sig: Script::from_bytes(vec![0x51]),
            }],
            vec![TransactionOutput {
                value: 400_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }]
        ).unwrap();
        
        // Mock UTXO set with the unspent output
        let mut utxo_set = HashMap::new();
        utxo_set.insert(utxo_id, Utxo {
            value: 1_000_000,
            script_pubkey: Script::from_bytes(vec![0x51]),
            block_height: 50,
        });
        
        // First transaction should be valid
        assert!(validate_transaction(&tx1, &utxo_set, 100).is_ok());
        
        // Remove the UTXO (simulate it being spent)
        utxo_set.remove(&utxo_id);
        
        // Second transaction should fail (UTXO not found)
        let result = validate_transaction(&tx2, &utxo_set, 100);
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::UtxoNotFound { .. } => {}, // Expected
            _ => panic!("Expected UtxoNotFound error"),
        }
    }

    #[test]
    fn insufficient_balance_detection() {
        let prev_tx_hash = TransactionHash::from_bytes([7; 32]);
        
        // Transaction trying to spend more than available
        let transaction = Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(prev_tx_hash, 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }],
            vec![TransactionOutput {
                value: 2_000_000, // Trying to spend 2 BND
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }]
        ).unwrap();
        
        // Mock UTXO set with insufficient balance
        let mut utxo_set = HashMap::new();
        utxo_set.insert(UtxoId::new(prev_tx_hash, 0), Utxo {
            value: 1_000_000, // Only 1 BND available
            script_pubkey: Script::from_bytes(vec![0x51]),
            block_height: 50,
        });
        
        let result = validate_transaction(&transaction, &utxo_set, 100);
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::InsufficientBalance { .. } => {}, // Expected
            _ => panic!("Expected InsufficientBalance error"),
        }
    }

    #[test]
    fn script_validation_integration() {
        let prev_tx_hash = TransactionHash::from_bytes([8; 32]);
        
        // Transaction with script that should succeed
        let success_tx = Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(prev_tx_hash, 0),
                script_sig: Script::from_bytes(vec![0x51]), // OP_1 (pushes true)
            }],
            vec![TransactionOutput {
                value: 500_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }]
        ).unwrap();
        
        // Mock UTXO with script that expects true on stack
        let mut utxo_set = HashMap::new();
        utxo_set.insert(UtxoId::new(prev_tx_hash, 0), Utxo {
            value: 1_000_000,
            script_pubkey: Script::from_bytes(vec![0x69]), // OP_VERIFY (requires true on stack)
            block_height: 50,
        });
        
        let result = validate_transaction(&success_tx, &utxo_set, 100);
        assert!(result.is_ok());
        
        // Transaction with script that should fail
        let fail_tx = Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(prev_tx_hash, 0),
                script_sig: Script::from_bytes(vec![0x00]), // OP_0 (pushes false)
            }],
            vec![TransactionOutput {
                value: 500_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }]
        ).unwrap();
        
        let result = validate_transaction(&fail_tx, &utxo_set, 100);
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::ScriptExecutionFailed { .. } => {}, // Expected
            _ => panic!("Expected ScriptExecutionFailed error"),
        }
    }
}
```
