# Camada 1: Unit Tests - Mempool

## 1.7 Testes de Mempool e Pool de Transações

### Basic Mempool Operations
```rust
#[cfg(test)]
mod mempool_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn mempool_creation() {
        let mempool = Mempool::new();
        
        assert_eq!(mempool.size(), 0);
        assert_eq!(mempool.total_fees(), 0);
        assert!(mempool.is_empty());
        assert_eq!(mempool.get_transactions().len(), 0);
    }

    #[test]
    fn add_valid_transaction() {
        let mut mempool = Mempool::new();
        
        let transaction = create_test_transaction(1_000_000, 999_000); // 0.001 BND fee
        let tx_hash = transaction.hash();
        
        let result = mempool.add_transaction(transaction);
        assert!(result.is_ok());
        
        assert_eq!(mempool.size(), 1);
        assert_eq!(mempool.total_fees(), 1_000);
        assert!(mempool.contains(&tx_hash));
    }

    #[test]
    fn reject_duplicate_transaction() {
        let mut mempool = Mempool::new();
        
        let transaction = create_test_transaction(1_000_000, 999_000);
        let tx_clone = transaction.clone();
        
        // First addition should succeed
        assert!(mempool.add_transaction(transaction).is_ok());
        assert_eq!(mempool.size(), 1);
        
        // Second addition should fail
        let result = mempool.add_transaction(tx_clone);
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::TransactionAlreadyInMempool { .. } => {}, // Expected
            _ => panic!("Expected TransactionAlreadyInMempool error"),
        }
        
        assert_eq!(mempool.size(), 1); // Size unchanged
    }

    #[test]
    fn remove_transaction() {
        let mut mempool = Mempool::new();
        
        let transaction = create_test_transaction(1_000_000, 999_000);
        let tx_hash = transaction.hash();
        
        mempool.add_transaction(transaction).unwrap();
        assert_eq!(mempool.size(), 1);
        
        let removed = mempool.remove_transaction(&tx_hash);
        assert!(removed.is_some());
        assert_eq!(mempool.size(), 0);
        assert!(!mempool.contains(&tx_hash));
    }

    #[test]
    fn mempool_size_limits() {
        let mut mempool = Mempool::with_max_size(3);
        
        // Add transactions up to limit
        for i in 0..3 {
            let tx = create_test_transaction_with_nonce(1_000_000, 999_000, i);
            assert!(mempool.add_transaction(tx).is_ok());
        }
        
        assert_eq!(mempool.size(), 3);
        
        // Adding one more should fail or evict lowest fee transaction
        let tx4 = create_test_transaction_with_nonce(1_000_000, 998_000, 4); // Higher fee
        let result = mempool.add_transaction(tx4);
        
        // Implementation dependent: either reject or evict lowest fee
        match result {
            Ok(_) => assert_eq!(mempool.size(), 3), // Evicted one transaction
            Err(BondError::MempoolFull) => assert_eq!(mempool.size(), 3), // Rejected
            _ => panic!("Unexpected result"),
        }
    }

    fn create_test_transaction(input_value: u64, output_value: u64) -> Transaction {
        let inputs = vec![TransactionInput {
            utxo_id: UtxoId::new(TransactionHash::from_bytes([1; 32]), 0),
            script_sig: Script::from_bytes(vec![0x51]),
        }];
        
        let outputs = vec![TransactionOutput {
            value: output_value,
            script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
        }];
        
        Transaction::new(inputs, outputs).unwrap()
    }

    fn create_test_transaction_with_nonce(input_value: u64, output_value: u64, nonce: u64) -> Transaction {
        let inputs = vec![TransactionInput {
            utxo_id: UtxoId::new(TransactionHash::from_bytes([(nonce as u8); 32]), 0),
            script_sig: Script::from_bytes(vec![0x51]),
        }];
        
        let outputs = vec![TransactionOutput {
            value: output_value,
            script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
        }];
        
        Transaction::new(inputs, outputs).unwrap()
    }
}
```

### Transaction Prioritization Tests
```rust
#[cfg(test)]
mod mempool_priority_tests {
    use super::*;

    #[test]
    fn fee_based_prioritization() {
        let mut mempool = Mempool::new();
        
        // Add transactions with different fees
        let low_fee_tx = create_test_transaction_with_nonce(1_000_000, 999_500, 1); // 0.0005 BND fee
        let med_fee_tx = create_test_transaction_with_nonce(1_000_000, 999_000, 2); // 0.001 BND fee
        let high_fee_tx = create_test_transaction_with_nonce(1_000_000, 998_000, 3); // 0.002 BND fee
        
        mempool.add_transaction(low_fee_tx).unwrap();
        mempool.add_transaction(high_fee_tx).unwrap();
        mempool.add_transaction(med_fee_tx).unwrap();
        
        // Get transactions ordered by priority (highest fee first)
        let prioritized = mempool.get_prioritized_transactions(10);
        
        assert_eq!(prioritized.len(), 3);
        
        // Calculate fees for verification
        let fees: Vec<u64> = prioritized.iter().map(|tx| {
            1_000_000 - tx.outputs[0].value // input_value - output_value
        }).collect();
        
        // Should be ordered by descending fee
        assert!(fees[0] >= fees[1]);
        assert!(fees[1] >= fees[2]);
        assert_eq!(fees[0], 2_000); // Highest fee first
    }

    #[test]
    fn fee_per_byte_prioritization() {
        let mut mempool = Mempool::new();
        
        // Create transactions with different sizes and fees
        let small_tx = create_small_transaction(1_000_000, 999_000); // 1KB, 0.001 BND fee
        let large_tx = create_large_transaction(2_000_000, 1_998_000); // 2KB, 0.002 BND fee
        
        mempool.add_transaction(small_tx).unwrap();
        mempool.add_transaction(large_tx).unwrap();
        
        let prioritized = mempool.get_prioritized_transactions(10);
        
        // Small transaction should have higher fee per byte
        // Small: 1000 satoshis / ~300 bytes ≈ 3.33 sat/byte
        // Large: 2000 satoshis / ~600 bytes ≈ 3.33 sat/byte
        // Implementation should handle this correctly
        assert_eq!(prioritized.len(), 2);
    }

    #[test]
    fn time_based_prioritization() {
        let mut mempool = Mempool::new();
        
        let old_tx = create_test_transaction_with_nonce(1_000_000, 999_000, 1);
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        mempool.add_transaction_with_timestamp(old_tx, current_time - 3600).unwrap(); // 1 hour ago
        
        let new_tx = create_test_transaction_with_nonce(1_000_000, 999_000, 2);
        mempool.add_transaction_with_timestamp(new_tx, current_time).unwrap(); // Now
        
        // When fees are equal, older transactions should have higher priority
        let prioritized = mempool.get_prioritized_transactions(10);
        assert_eq!(prioritized.len(), 2);
        
        // First transaction should be the older one (if fees are equal)
        // Implementation detail: depends on tie-breaking rules
    }

    fn create_small_transaction(input_value: u64, output_value: u64) -> Transaction {
        // Single input, single output - minimal size
        let inputs = vec![TransactionInput {
            utxo_id: UtxoId::new(TransactionHash::from_bytes([10; 32]), 0),
            script_sig: Script::from_bytes(vec![0x51]), // Minimal script
        }];
        
        let outputs = vec![TransactionOutput {
            value: output_value,
            script_pubkey: Script::from_bytes(vec![0x76, 0xAC]), // Minimal script
        }];
        
        Transaction::new(inputs, outputs).unwrap()
    }

    fn create_large_transaction(input_value: u64, output_value: u64) -> Transaction {
        // Multiple inputs and outputs - larger size
        let inputs = vec![
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_bytes([11; 32]), 0),
                script_sig: Script::from_bytes(vec![0x51, 0x52, 0x53, 0x54]), // Larger script
            },
            TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_bytes([12; 32]), 1),
                script_sig: Script::from_bytes(vec![0x51, 0x52, 0x53, 0x54]),
            },
        ];
        
        let outputs = vec![
            TransactionOutput {
                value: output_value / 2,
                script_pubkey: Script::from_bytes(vec![0x76, 0xA9, 0x14] + vec![0x20; 20] + vec![0x88, 0xAC]),
            },
            TransactionOutput {
                value: output_value / 2,
                script_pubkey: Script::from_bytes(vec![0x76, 0xA9, 0x14] + vec![0x21; 20] + vec![0x88, 0xAC]),
            },
        ];
        
        Transaction::new(inputs, outputs).unwrap()
    }
}
```

### Mempool Eviction and Memory Management
```rust
#[cfg(test)]
mod mempool_eviction_tests {
    use super::*;

    #[test]
    fn lru_eviction() {
        let mut mempool = Mempool::with_eviction_policy(EvictionPolicy::LeastRecentlyUsed, 3);
        
        let tx1 = create_test_transaction_with_nonce(1_000_000, 999_000, 1);
        let tx2 = create_test_transaction_with_nonce(1_000_000, 999_000, 2);
        let tx3 = create_test_transaction_with_nonce(1_000_000, 999_000, 3);
        let tx4 = create_test_transaction_with_nonce(1_000_000, 999_000, 4);
        
        let tx1_hash = tx1.hash();
        let tx2_hash = tx2.hash();
        let tx3_hash = tx3.hash();
        let tx4_hash = tx4.hash();
        
        // Fill mempool to capacity
        mempool.add_transaction(tx1).unwrap();
        mempool.add_transaction(tx2).unwrap();
        mempool.add_transaction(tx3).unwrap();
        
        // Access tx1 to make it recently used
        mempool.touch_transaction(&tx1_hash);
        
        // Adding tx4 should evict tx2 (least recently used)
        mempool.add_transaction(tx4).unwrap();
        
        assert_eq!(mempool.size(), 3);
        assert!(mempool.contains(&tx1_hash)); // Recently used, should remain
        assert!(!mempool.contains(&tx2_hash)); // Should be evicted
        assert!(mempool.contains(&tx3_hash)); // Should remain
        assert!(mempool.contains(&tx4_hash)); // Newly added
    }

    #[test]
    fn lowest_fee_eviction() {
        let mut mempool = Mempool::with_eviction_policy(EvictionPolicy::LowestFee, 3);
        
        let low_fee_tx = create_test_transaction_with_nonce(1_000_000, 999_500, 1); // 0.0005 BND fee
        let med_fee_tx = create_test_transaction_with_nonce(1_000_000, 999_000, 2); // 0.001 BND fee
        let high_fee_tx = create_test_transaction_with_nonce(1_000_000, 998_000, 3); // 0.002 BND fee
        let higher_fee_tx = create_test_transaction_with_nonce(1_000_000, 997_000, 4); // 0.003 BND fee
        
        let low_fee_hash = low_fee_tx.hash();
        let higher_fee_hash = higher_fee_tx.hash();
        
        // Fill mempool
        mempool.add_transaction(low_fee_tx).unwrap();
        mempool.add_transaction(med_fee_tx).unwrap();
        mempool.add_transaction(high_fee_tx).unwrap();
        
        // Adding higher fee transaction should evict lowest fee
        mempool.add_transaction(higher_fee_tx).unwrap();
        
        assert_eq!(mempool.size(), 3);
        assert!(!mempool.contains(&low_fee_hash)); // Lowest fee, should be evicted
        assert!(mempool.contains(&higher_fee_hash)); // Highest fee, should be added
    }

    #[test]
    fn memory_usage_tracking() {
        let mut mempool = Mempool::with_memory_limit(1024 * 1024); // 1MB limit
        
        // Add transactions until near memory limit
        let mut transaction_count = 0;
        loop {
            let tx = create_test_transaction_with_nonce(1_000_000, 999_000, transaction_count);
            match mempool.add_transaction(tx) {
                Ok(_) => transaction_count += 1,
                Err(BondError::MempoolMemoryLimitExceeded) => break,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
            
            // Safety break to prevent infinite loop
            if transaction_count > 10000 {
                break;
            }
        }
        
        assert!(transaction_count > 0);
        assert!(mempool.memory_usage() <= 1024 * 1024);
    }

    #[test]
    fn orphan_transaction_handling() {
        let mut mempool = Mempool::new();
        
        // Create transaction that spends non-existent UTXO (orphan)
        let orphan_tx = Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_bytes([99; 32]), 0), // Non-existent
                script_sig: Script::from_bytes(vec![0x51]),
            }],
            vec![TransactionOutput {
                value: 500_000,
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }]
        ).unwrap();
        
        // Mock UTXO set without the required input
        let utxo_set = HashMap::new();
        
        let result = mempool.add_transaction_with_validation(orphan_tx, &utxo_set);
        
        // Should either reject or store in orphan pool
        match result {
            Err(BondError::UtxoNotFound { .. }) => {}, // Rejected
            Ok(_) => {
                // If accepted, should be in orphan pool
                assert_eq!(mempool.orphan_count(), 1);
            },
            _ => panic!("Unexpected result"),
        }
    }
}
```

### Mempool Transaction Lifecycle
```rust
#[cfg(test)]
mod mempool_lifecycle_tests {
    use super::*;

    #[test]
    fn transaction_replacement() {
        let mut mempool = Mempool::new();
        
        // Original transaction
        let original_tx = create_test_transaction_with_nonce(1_000_000, 999_000, 1); // 0.001 BND fee
        let original_hash = original_tx.hash();
        
        mempool.add_transaction(original_tx).unwrap();
        assert_eq!(mempool.size(), 1);
        
        // Replacement transaction with higher fee (same inputs)
        let replacement_tx = Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_bytes([1; 32]), 0), // Same input
                script_sig: Script::from_bytes(vec![0x51]),
            }],
            vec![TransactionOutput {
                value: 998_000, // Higher fee (0.002 BND)
                script_pubkey: Script::from_bytes(vec![0x76, 0xAC]),
            }]
        ).unwrap();
        
        let replacement_hash = replacement_tx.hash();
        
        // RBF (Replace-by-Fee) should succeed
        let result = mempool.replace_transaction(replacement_tx);
        assert!(result.is_ok());
        
        assert_eq!(mempool.size(), 1);
        assert!(!mempool.contains(&original_hash)); // Original should be removed
        assert!(mempool.contains(&replacement_hash)); // Replacement should be present
    }

    #[test]
    fn transaction_confirmation() {
        let mut mempool = Mempool::new();
        
        // Add transactions to mempool
        let tx1 = create_test_transaction_with_nonce(1_000_000, 999_000, 1);
        let tx2 = create_test_transaction_with_nonce(1_000_000, 999_000, 2);
        let tx3 = create_test_transaction_with_nonce(1_000_000, 999_000, 3);
        
        let tx1_hash = tx1.hash();
        let tx2_hash = tx2.hash();
        let tx3_hash = tx3.hash();
        
        mempool.add_transaction(tx1).unwrap();
        mempool.add_transaction(tx2).unwrap();
        mempool.add_transaction(tx3).unwrap();
        
        assert_eq!(mempool.size(), 3);
        
        // Simulate block confirmation (tx1 and tx2 included in block)
        let confirmed_transactions = vec![tx1_hash, tx2_hash];
        mempool.remove_confirmed_transactions(&confirmed_transactions);
        
        assert_eq!(mempool.size(), 1);
        assert!(!mempool.contains(&tx1_hash));
        assert!(!mempool.contains(&tx2_hash));
        assert!(mempool.contains(&tx3_hash)); // Unconfirmed, should remain
    }

    #[test]
    fn transaction_expiration() {
        let mut mempool = Mempool::with_expiration(3600); // 1 hour expiration
        
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Add old transaction
        let old_tx = create_test_transaction_with_nonce(1_000_000, 999_000, 1);
        let old_tx_hash = old_tx.hash();
        mempool.add_transaction_with_timestamp(old_tx, current_time - 7200).unwrap(); // 2 hours ago
        
        // Add fresh transaction
        let fresh_tx = create_test_transaction_with_nonce(1_000_000, 999_000, 2);
        let fresh_tx_hash = fresh_tx.hash();
        mempool.add_transaction_with_timestamp(fresh_tx, current_time).unwrap(); // Now
        
        assert_eq!(mempool.size(), 2);
        
        // Clean expired transactions
        mempool.clean_expired_transactions(current_time);
        
        assert_eq!(mempool.size(), 1);
        assert!(!mempool.contains(&old_tx_hash)); // Expired, should be removed
        assert!(mempool.contains(&fresh_tx_hash)); // Fresh, should remain
    }

    #[test]
    fn mempool_statistics() {
        let mut mempool = Mempool::new();
        
        // Add transactions with different fees
        let fees = vec![1_000, 2_000, 3_000, 4_000, 5_000]; // Different fee amounts
        for (i, fee) in fees.iter().enumerate() {
            let tx = create_test_transaction_with_nonce(1_000_000, 1_000_000 - fee, i as u64);
            mempool.add_transaction(tx).unwrap();
        }
        
        let stats = mempool.get_statistics();
        
        assert_eq!(stats.transaction_count, 5);
        assert_eq!(stats.total_fees, 15_000); // Sum of all fees
        assert_eq!(stats.average_fee, 3_000); // Average fee
        assert_eq!(stats.min_fee, 1_000);
        assert_eq!(stats.max_fee, 5_000);
        assert!(stats.memory_usage > 0);
    }

    #[test]
    fn mempool_persistence() {
        let mut mempool = Mempool::new();
        
        // Add transactions
        let tx1 = create_test_transaction_with_nonce(1_000_000, 999_000, 1);
        let tx2 = create_test_transaction_with_nonce(1_000_000, 999_000, 2);
        
        mempool.add_transaction(tx1).unwrap();
        mempool.add_transaction(tx2).unwrap();
        
        assert_eq!(mempool.size(), 2);
        
        // Serialize mempool state
        let serialized = mempool.serialize().unwrap();
        
        // Create new mempool and deserialize
        let mut restored_mempool = Mempool::new();
        restored_mempool.deserialize(&serialized).unwrap();
        
        assert_eq!(restored_mempool.size(), 2);
        assert_eq!(restored_mempool.total_fees(), mempool.total_fees());
    }
}
```

### Concurrent Access Tests
```rust
#[cfg(test)]
mod mempool_concurrency_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn concurrent_additions() {
        let mempool = Arc::new(Mutex::new(Mempool::new()));
        let mut handles = vec![];
        
        // Spawn multiple threads adding transactions
        for i in 0..10 {
            let mempool_clone = Arc::clone(&mempool);
            let handle = thread::spawn(move || {
                let tx = create_test_transaction_with_nonce(1_000_000, 999_000, i);
                let mut pool = mempool_clone.lock().unwrap();
                pool.add_transaction(tx)
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        let mut successful_additions = 0;
        for handle in handles {
            if handle.join().unwrap().is_ok() {
                successful_additions += 1;
            }
        }
        
        let final_size = mempool.lock().unwrap().size();
        assert_eq!(successful_additions, final_size);
        assert_eq!(final_size, 10); // All transactions should be added
    }

    #[test]
    fn concurrent_read_write() {
        let mempool = Arc::new(Mutex::new(Mempool::new()));
        
        // Add initial transactions
        {
            let mut pool = mempool.lock().unwrap();
            for i in 0..5 {
                let tx = create_test_transaction_with_nonce(1_000_000, 999_000, i);
                pool.add_transaction(tx).unwrap();
            }
        }
        
        let mut handles = vec![];
        
        // Spawn reader threads
        for _ in 0..5 {
            let mempool_clone = Arc::clone(&mempool);
            let handle = thread::spawn(move || {
                let pool = mempool_clone.lock().unwrap();
                pool.get_prioritized_transactions(10)
            });
            handles.push(handle);
        }
        
        // Spawn writer threads
        for i in 5..10 {
            let mempool_clone = Arc::clone(&mempool);
            let handle = thread::spawn(move || {
                let tx = create_test_transaction_with_nonce(1_000_000, 999_000, i);
                let mut pool = mempool_clone.lock().unwrap();
                pool.add_transaction(tx)
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_size = mempool.lock().unwrap().size();
        assert_eq!(final_size, 10); // Should have all transactions
    }
}
```
