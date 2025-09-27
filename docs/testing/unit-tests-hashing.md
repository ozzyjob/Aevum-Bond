# Camada 1: Unit Tests - Hashing e Criptografia

## 1.2 Validação de Hashing Keccak-256

### Merkle Tree Validation
```rust
#[cfg(test)]
mod hashing_tests {
    use super::*;
    
    #[test]
    fn merkle_root_deterministic() {
        let transactions = vec![
            Transaction::coinbase(5_000_000_000, b"test1".to_vec()),
            Transaction::new(1, vec![], vec![], 0),
            Transaction::new(1, vec![], vec![], 1),
        ];
        
        let block = Block::new(create_test_header(), transactions.clone());
        let root1 = block.calculate_merkle_root().unwrap();
        let root2 = block.calculate_merkle_root().unwrap();
        
        assert_eq!(root1, root2); // Deterministic
        
        // Modify one transaction and verify root changes
        let mut modified_transactions = transactions;
        modified_transactions[0] = Transaction::coinbase(6_000_000_000, b"test1".to_vec());
        let modified_block = Block::new(create_test_header(), modified_transactions);
        let modified_root = modified_block.calculate_merkle_root().unwrap();
        
        assert_ne!(root1, modified_root); // Root changes with transaction modification
    }

    #[test]
    fn block_hash_invalidation() {
        let mut header = create_test_header();
        let original_hash = header.hash().unwrap();
        
        // Modify nonce
        header.nonce += 1;
        let modified_hash = header.hash().unwrap();
        
        assert_ne!(original_hash, modified_hash);
        
        // Modify timestamp
        header.timestamp = chrono::Utc::now();
        let timestamp_modified_hash = header.hash().unwrap();
        
        assert_ne!(modified_hash, timestamp_modified_hash);
    }

    #[test]
    fn empty_block_merkle_root() {
        let empty_block = Block::new(create_test_header(), vec![]);
        let root = empty_block.calculate_merkle_root().unwrap();
        
        assert_eq!(root, MerkleRoot::ZERO);
    }

    proptest! {
        #[test]
        fn merkle_root_single_tx_stability(
            reward in 1u64..10_000_000_000,
            message in prop::collection::vec(any::<u8>(), 0..100)
        ) {
            let tx = Transaction::coinbase(reward, message);
            let block = Block::new(create_test_header(), vec![tx.clone()]);
            let root = block.calculate_merkle_root().unwrap();
            
            // Single transaction merkle root should equal transaction hash
            let tx_hash = tx.hash().unwrap();
            prop_assert_eq!(root.as_bytes(), tx_hash.as_bytes());
        }
    }
}
```

## 1.3 Validação de Integridade de Hash

### Transaction Hash Stability
```rust
#[cfg(test)]
mod hash_integrity_tests {
    #[test]
    fn transaction_hash_immutability() {
        let tx = Transaction::new(
            1,
            vec![TransactionInput::new(
                UtxoId::new(TransactionHash::ZERO, 0),
                Script::new(b"signature".to_vec()),
                0
            )],
            vec![TransactionOutput::new(1000, Script::new(b"pubkey".to_vec()))],
            0
        );
        
        let hash1 = tx.hash().unwrap();
        
        // Hash multiple times should be identical
        for _ in 0..100 {
            let hash_n = tx.hash().unwrap();
            assert_eq!(hash1, hash_n);
        }
    }

    #[test]
    fn hash_collision_resistance() {
        let tx1 = Transaction::coinbase(1000, b"message1".to_vec());
        let tx2 = Transaction::coinbase(1000, b"message2".to_vec());
        
        let hash1 = tx1.hash().unwrap();
        let hash2 = tx2.hash().unwrap();
        
        assert_ne!(hash1, hash2); // Different inputs should produce different hashes
    }

    #[test]
    fn keccak256_test_vectors() {
        // Test against known Keccak-256 test vectors
        let test_cases = vec![
            ("", "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"),
            ("abc", "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"),
        ];
        
        for (input, expected) in test_cases {
            let mut hasher = Keccak256::new();
            hasher.update(input.as_bytes());
            let result = hasher.finalize();
            let hex_result = hex::encode(result);
            
            assert_eq!(hex_result, expected);
        }
    }
}
```
