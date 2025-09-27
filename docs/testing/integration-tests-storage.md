# Camada 2: Integration Tests - Storage Layer Integration

## 2.7 Testes de Integração da Camada de Storage

### Complete Storage Integration Tests
```rust
#[cfg(test)]
mod storage_integration_tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;

    struct StorageTestEnvironment {
        temp_dir: TempDir,
        blockchain_storage: BlockchainStorage,
        utxo_storage: UtxoStorage,
        state_storage: StateStorage,
        mempool_storage: MempoolStorage,
        config_storage: ConfigStorage,
        node: TestNode,
    }

    impl StorageTestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_path = temp_dir.path().to_path_buf();
            
            let blockchain_storage = BlockchainStorage::new(base_path.join("blockchain")).await.unwrap();
            let utxo_storage = UtxoStorage::new(base_path.join("utxos")).await.unwrap();
            let state_storage = StateStorage::new(base_path.join("state")).await.unwrap();
            let mempool_storage = MempoolStorage::new(base_path.join("mempool")).await.unwrap();
            let config_storage = ConfigStorage::new(base_path.join("config")).await.unwrap();
            
            let node = TestNode::new_with_storage_path(base_path.clone()).await;
            
            Self {
                temp_dir,
                blockchain_storage,
                utxo_storage,
                state_storage,
                mempool_storage,
                config_storage,
                node,
            }
        }
        
        fn storage_path(&self) -> PathBuf {
            self.temp_dir.path().to_path_buf()
        }
    }

    #[tokio::test]
    async fn blockchain_storage_persistence() {
        let mut env = StorageTestEnvironment::new().await;
        
        // Create test blocks
        let genesis_block = create_genesis_block().await;
        let block1 = create_test_block(&genesis_block, 1).await;
        let block2 = create_test_block(&block1, 2).await;
        
        // Store blocks
        env.blockchain_storage.store_block(&genesis_block).await.unwrap();
        env.blockchain_storage.store_block(&block1).await.unwrap();
        env.blockchain_storage.store_block(&block2).await.unwrap();
        
        // Verify storage
        let stored_genesis = env.blockchain_storage.get_block_by_height(0).await.unwrap();
        assert_eq!(stored_genesis.hash(), genesis_block.hash());
        
        let stored_block1 = env.blockchain_storage.get_block_by_hash(&block1.hash()).await.unwrap();
        assert_eq!(stored_block1.height(), 1);
        
        // Test block retrieval by different criteria
        let latest_block = env.blockchain_storage.get_latest_block().await.unwrap();
        assert_eq!(latest_block.hash(), block2.hash());
        
        let current_height = env.blockchain_storage.get_current_height().await.unwrap();
        assert_eq!(current_height, 2);
        
        // Test block range retrieval
        let block_range = env.blockchain_storage.get_blocks_in_range(0, 2).await.unwrap();
        assert_eq!(block_range.len(), 3);
        assert_eq!(block_range[0].hash(), genesis_block.hash());
        assert_eq!(block_range[2].hash(), block2.hash());
        
        // Test persistence across restart
        drop(env.blockchain_storage);
        let restored_storage = BlockchainStorage::new(env.storage_path().join("blockchain")).await.unwrap();
        
        let restored_block1 = restored_storage.get_block_by_height(1).await.unwrap();
        assert_eq!(restored_block1.hash(), block1.hash());
        
        let restored_height = restored_storage.get_current_height().await.unwrap();
        assert_eq!(restored_height, 2);
    }

    #[tokio::test]
    async fn utxo_set_storage_and_updates() {
        let mut env = StorageTestEnvironment::new().await;
        
        // Create test UTXOs
        let utxo1 = Utxo {
            value: 1_000_000,
            script_pubkey: Script::from_bytes(vec![0x51]),
            block_height: 1,
        };
        
        let utxo2 = Utxo {
            value: 2_500_000,
            script_pubkey: Script::from_bytes(vec![0x52]),
            block_height: 1,
        };
        
        let utxo_id1 = UtxoId::new(TransactionHash::from_bytes([1; 32]), 0);
        let utxo_id2 = UtxoId::new(TransactionHash::from_bytes([2; 32]), 0);
        
        // Store UTXOs
        env.utxo_storage.add_utxo(utxo_id1, utxo1.clone()).await.unwrap();
        env.utxo_storage.add_utxo(utxo_id2, utxo2.clone()).await.unwrap();
        
        // Verify storage
        let stored_utxo1 = env.utxo_storage.get_utxo(&utxo_id1).await.unwrap();
        assert_eq!(stored_utxo1.value, utxo1.value);
        
        let utxo_count = env.utxo_storage.get_utxo_count().await.unwrap();
        assert_eq!(utxo_count, 2);
        
        // Test UTXO set snapshot
        let snapshot = env.utxo_storage.create_snapshot().await.unwrap();
        assert_eq!(snapshot.len(), 2);
        assert!(snapshot.contains_key(&utxo_id1));
        assert!(snapshot.contains_key(&utxo_id2));
        
        // Test UTXO spending (removal)
        env.utxo_storage.spend_utxo(&utxo_id1).await.unwrap();
        
        let spent_utxo = env.utxo_storage.get_utxo(&utxo_id1).await;
        assert!(spent_utxo.is_err()); // Should not exist
        
        let updated_count = env.utxo_storage.get_utxo_count().await.unwrap();
        assert_eq!(updated_count, 1);
        
        // Test batch operations
        let batch_utxos = vec![
            (UtxoId::new(TransactionHash::from_bytes([3; 32]), 0), Utxo {
                value: 500_000,
                script_pubkey: Script::from_bytes(vec![0x53]),
                block_height: 2,
            }),
            (UtxoId::new(TransactionHash::from_bytes([4; 32]), 0), Utxo {
                value: 750_000,
                script_pubkey: Script::from_bytes(vec![0x54]),
                block_height: 2,
            }),
        ];
        
        env.utxo_storage.batch_add_utxos(&batch_utxos).await.unwrap();
        
        let final_count = env.utxo_storage.get_utxo_count().await.unwrap();
        assert_eq!(final_count, 3); // 1 existing + 2 batch added
        
        // Test persistence
        drop(env.utxo_storage);
        let restored_storage = UtxoStorage::new(env.storage_path().join("utxos")).await.unwrap();
        
        let restored_count = restored_storage.get_utxo_count().await.unwrap();
        assert_eq!(restored_count, 3);
        
        let restored_utxo2 = restored_storage.get_utxo(&utxo_id2).await.unwrap();
        assert_eq!(restored_utxo2.value, utxo2.value);
    }

    #[tokio::test]
    async fn state_storage_and_checkpoints() {
        let mut env = StorageTestEnvironment::new().await;
        
        // Create initial state
        let initial_state = ChainState {
            height: 100,
            best_block_hash: BlockHash::from_bytes([1; 32]),
            total_supply: 21_000_000_000_000,
            difficulty_target: Target::from_compact(0x1d00ffff),
            last_checkpoint: Utc::now(),
        };
        
        // Store state
        env.state_storage.save_state(&initial_state).await.unwrap();
        
        // Verify storage
        let stored_state = env.state_storage.load_state().await.unwrap();
        assert_eq!(stored_state.height, initial_state.height);
        assert_eq!(stored_state.best_block_hash, initial_state.best_block_hash);
        assert_eq!(stored_state.total_supply, initial_state.total_supply);
        
        // Create checkpoint
        let checkpoint_height = 100;
        env.state_storage.create_checkpoint(checkpoint_height).await.unwrap();
        
        // Update state
        let updated_state = ChainState {
            height: 150,
            best_block_hash: BlockHash::from_bytes([2; 32]),
            total_supply: 21_000_050_000_000, // Increased by mining rewards
            difficulty_target: Target::from_compact(0x1d00fffe),
            last_checkpoint: Utc::now(),
        };
        
        env.state_storage.save_state(&updated_state).await.unwrap();
        
        // Verify update
        let current_state = env.state_storage.load_state().await.unwrap();
        assert_eq!(current_state.height, 150);
        
        // Test rollback to checkpoint
        env.state_storage.rollback_to_checkpoint(checkpoint_height).await.unwrap();
        
        let rolled_back_state = env.state_storage.load_state().await.unwrap();
        assert_eq!(rolled_back_state.height, initial_state.height);
        assert_eq!(rolled_back_state.best_block_hash, initial_state.best_block_hash);
        
        // Test checkpoint cleanup
        let checkpoint_list = env.state_storage.list_checkpoints().await.unwrap();
        assert_eq!(checkpoint_list.len(), 1);
        
        env.state_storage.cleanup_old_checkpoints(10).await.unwrap(); // Keep only last 10
        
        let cleaned_list = env.state_storage.list_checkpoints().await.unwrap();
        assert_eq!(cleaned_list.len(), 1); // Should still have the one checkpoint
    }

    #[tokio::test]
    async fn mempool_persistence_across_restarts() {
        let mut env = StorageTestEnvironment::new().await;
        
        // Create test transactions
        let tx1 = create_test_transaction(1_000_000, 1_000, 1).await;
        let tx2 = create_test_transaction(2_000_000, 2_000, 2).await;
        let tx3 = create_test_transaction(500_000, 500, 3).await;
        
        let tx_hashes = vec![tx1.hash(), tx2.hash(), tx3.hash()];
        
        // Add transactions to mempool
        env.node.add_transaction_to_mempool(tx1).await.unwrap();
        env.node.add_transaction_to_mempool(tx2).await.unwrap();
        env.node.add_transaction_to_mempool(tx3).await.unwrap();
        
        // Verify mempool state
        let mempool_size = env.node.get_mempool_size().await.unwrap();
        assert_eq!(mempool_size, 3);
        
        // Save mempool to storage
        env.mempool_storage.save_mempool_state(&env.node.get_mempool_transactions().await.unwrap()).await.unwrap();
        
        // Simulate node restart - clear mempool
        env.node.clear_mempool().await.unwrap();
        let empty_size = env.node.get_mempool_size().await.unwrap();
        assert_eq!(empty_size, 0);
        
        // Restore mempool from storage
        let restored_transactions = env.mempool_storage.load_mempool_state().await.unwrap();
        assert_eq!(restored_transactions.len(), 3);
        
        for tx in restored_transactions {
            env.node.add_transaction_to_mempool(tx).await.unwrap();
        }
        
        // Verify restoration
        let restored_size = env.node.get_mempool_size().await.unwrap();
        assert_eq!(restored_size, 3);
        
        for tx_hash in &tx_hashes {
            let has_tx = env.node.has_transaction_in_mempool(tx_hash).await.unwrap();
            assert!(has_tx, "Transaction not restored to mempool");
        }
        
        // Test selective persistence (only persist transactions above fee threshold)
        let high_fee_threshold = 1500; // 0.0015 BND
        env.mempool_storage.save_mempool_state_with_filter(
            &env.node.get_mempool_transactions().await.unwrap(),
            |tx| calculate_fee(tx) >= high_fee_threshold,
        ).await.unwrap();
        
        env.node.clear_mempool().await.unwrap();
        let filtered_transactions = env.mempool_storage.load_mempool_state().await.unwrap();
        
        // Only tx1 (1000 fee) and tx2 (2000 fee) should be restored, not tx3 (500 fee)
        assert_eq!(filtered_transactions.len(), 2);
        
        let restored_hashes: Vec<_> = filtered_transactions.iter().map(|tx| tx.hash()).collect();
        assert!(restored_hashes.contains(&tx_hashes[0])); // tx1
        assert!(restored_hashes.contains(&tx_hashes[1])); // tx2
        assert!(!restored_hashes.contains(&tx_hashes[2])); // tx3 filtered out
    }

    #[tokio::test]
    async fn configuration_storage_and_updates() {
        let mut env = StorageTestEnvironment::new().await;
        
        // Create initial configuration
        let initial_config = NodeConfig {
            network: NetworkConfig {
                listen_address: "0.0.0.0:8333".to_string(),
                max_peers: 125,
                bootstrap_nodes: vec!["bootstrap.bond.network:8333".to_string()],
            },
            mining: MiningConfig {
                enabled: true,
                threads: 4,
                target_block_time: 600, // 10 minutes
            },
            storage: StorageConfig {
                data_dir: "/var/lib/bond".to_string(),
                max_cache_size: 1024 * 1024 * 1024, // 1GB
                checkpoint_interval: 1000,
            },
        };
        
        // Save configuration
        env.config_storage.save_config(&initial_config).await.unwrap();
        
        // Verify storage
        let stored_config = env.config_storage.load_config().await.unwrap();
        assert_eq!(stored_config.network.max_peers, initial_config.network.max_peers);
        assert_eq!(stored_config.mining.threads, initial_config.mining.threads);
        
        // Test configuration updates
        let mut updated_config = stored_config.clone();
        updated_config.network.max_peers = 200;
        updated_config.mining.enabled = false;
        
        env.config_storage.save_config(&updated_config).await.unwrap();
        
        let reloaded_config = env.config_storage.load_config().await.unwrap();
        assert_eq!(reloaded_config.network.max_peers, 200);
        assert_eq!(reloaded_config.mining.enabled, false);
        
        // Test configuration versioning
        let config_version = env.config_storage.get_config_version().await.unwrap();
        assert!(config_version > 0);
        
        // Create backup before major change
        env.config_storage.create_config_backup().await.unwrap();
        
        // Make major change
        let mut major_change_config = reloaded_config.clone();
        major_change_config.storage.data_dir = "/new/data/dir".to_string();
        
        env.config_storage.save_config(&major_change_config).await.unwrap();
        
        // Test rollback capability
        env.config_storage.restore_config_backup().await.unwrap();
        
        let restored_config = env.config_storage.load_config().await.unwrap();
        assert_eq!(restored_config.storage.data_dir, updated_config.storage.data_dir);
    }

    #[tokio::test] 
    async fn storage_integrity_and_corruption_detection() {
        let mut env = StorageTestEnvironment::new().await;
        
        // Store test data
        let test_block = create_test_block(&create_genesis_block().await, 1).await;
        env.blockchain_storage.store_block(&test_block).await.unwrap();
        
        // Verify integrity check passes
        let integrity_result = env.blockchain_storage.verify_integrity().await.unwrap();
        assert!(integrity_result.is_valid);
        assert_eq!(integrity_result.corrupted_blocks.len(), 0);
        
        // Simulate corruption by directly modifying storage files
        let block_file_path = env.storage_path().join("blockchain").join("blocks").join("000001.blk");
        
        if block_file_path.exists() {
            // Corrupt a few bytes in the middle of the file
            let mut file_data = std::fs::read(&block_file_path).unwrap();
            if file_data.len() > 100 {
                file_data[50] = file_data[50].wrapping_add(1);
                file_data[51] = file_data[51].wrapping_add(1);
                std::fs::write(&block_file_path, &file_data).unwrap();
            }
        }
        
        // Integrity check should now detect corruption
        let corrupted_result = env.blockchain_storage.verify_integrity().await.unwrap();
        assert!(!corrupted_result.is_valid);
        assert!(corrupted_result.corrupted_blocks.len() > 0);
        
        // Test repair attempt
        let repair_result = env.blockchain_storage.attempt_repair().await;
        match repair_result {
            Ok(_) => {
                // If repair succeeded, verify integrity again
                let repaired_result = env.blockchain_storage.verify_integrity().await.unwrap();
                assert!(repaired_result.is_valid);
            }
            Err(StorageError::CorruptionUnrecoverable) => {
                // Corruption was too severe to repair automatically
                // This is acceptable behavior
            }
            Err(e) => panic!("Unexpected repair error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn concurrent_storage_access() {
        let mut env = StorageTestEnvironment::new().await;
        
        // Create multiple concurrent tasks accessing storage
        let mut handles = vec![];
        
        // Task 1: Continuously add blocks
        let blockchain_storage1 = env.blockchain_storage.clone();
        let handle1 = tokio::spawn(async move {
            for i in 1..=20 {
                let block = create_test_block_with_height(i).await;
                blockchain_storage1.store_block(&block).await.unwrap();
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        handles.push(handle1);
        
        // Task 2: Continuously read blocks
        let blockchain_storage2 = env.blockchain_storage.clone();
        let handle2 = tokio::spawn(async move {
            for _ in 0..50 {
                let height = (rand::random::<u64>() % 20) + 1;
                match blockchain_storage2.get_block_by_height(height).await {
                    Ok(_) => {}, // Block found
                    Err(StorageError::BlockNotFound) => {}, // Block not yet stored
                    Err(e) => panic!("Unexpected error: {:?}", e),
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });
        handles.push(handle2);
        
        // Task 3: Continuously update UTXO set
        let utxo_storage1 = env.utxo_storage.clone();
        let handle3 = tokio::spawn(async move {
            for i in 1..=30 {
                let utxo_id = UtxoId::new(TransactionHash::from_bytes([i as u8; 32]), 0);
                let utxo = Utxo {
                    value: 1_000_000 + i * 1000,
                    script_pubkey: Script::from_bytes(vec![0x51]),
                    block_height: i,
                };
                utxo_storage1.add_utxo(utxo_id, utxo).await.unwrap();
                tokio::time::sleep(Duration::from_millis(8)).await;
            }
        });
        handles.push(handle3);
        
        // Task 4: Continuously read UTXO set
        let utxo_storage2 = env.utxo_storage.clone();
        let handle4 = tokio::spawn(async move {
            for i in 1..=40 {
                let utxo_id = UtxoId::new(TransactionHash::from_bytes([i as u8; 32]), 0);
                match utxo_storage2.get_utxo(&utxo_id).await {
                    Ok(_) => {}, // UTXO found
                    Err(StorageError::UtxoNotFound) => {}, // UTXO not yet stored
                    Err(e) => panic!("Unexpected error: {:?}", e),
                }
                tokio::time::sleep(Duration::from_millis(6)).await;
            }
        });
        handles.push(handle4);
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify final state consistency
        let final_height = env.blockchain_storage.get_current_height().await.unwrap();
        assert_eq!(final_height, 20);
        
        let final_utxo_count = env.utxo_storage.get_utxo_count().await.unwrap();
        assert_eq!(final_utxo_count, 30);
        
        // Verify no corruption occurred during concurrent access
        let integrity_result = env.blockchain_storage.verify_integrity().await.unwrap();
        assert!(integrity_result.is_valid);
    }

    // Helper functions
    async fn create_genesis_block() -> Block {
        Block::genesis()
    }
    
    async fn create_test_block(parent: &Block, height: u64) -> Block {
        Block::new(
            BlockHeader {
                previous_hash: parent.hash(),
                merkle_root: MerkleRoot::ZERO,
                timestamp: parent.header.timestamp + 600,
                target: parent.header.target,
                nonce: 0,
                height,
            },
            vec![create_coinbase_transaction(height).await],
        ).unwrap()
    }
    
    async fn create_test_block_with_height(height: u64) -> Block {
        Block::new(
            BlockHeader {
                previous_hash: BlockHash::from_bytes([height as u8; 32]),
                merkle_root: MerkleRoot::ZERO,
                timestamp: 1234567890 + height * 600,
                target: Target::from_compact(0x1d00ffff),
                nonce: height,
                height,
            },
            vec![create_coinbase_transaction(height).await],
        ).unwrap()
    }
    
    async fn create_coinbase_transaction(height: u64) -> Transaction {
        Transaction::coinbase(height, Script::from_bytes(vec![0x51]))
    }
    
    async fn create_test_transaction(value: u64, fee: u64, nonce: u64) -> Transaction {
        Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_bytes([nonce as u8; 32]), 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }],
            vec![TransactionOutput {
                value: value - fee,
                script_pubkey: Script::from_bytes(vec![0x52]),
            }],
        ).unwrap()
    }
    
    fn calculate_fee(tx: &Transaction) -> u64 {
        // Simplified fee calculation for test
        1000 + tx.inputs.len() as u64 * 500 + tx.outputs.len() as u64 * 300
    }
}
```
