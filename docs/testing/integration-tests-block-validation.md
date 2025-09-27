# Camada 2: Integration Tests - Block Validation

## 2.3 Testes de Validação Integrada de Blocos

### Complete Block Validation Flow
```rust
#[cfg(test)]
mod block_validation_integration_tests {
    use super::*;
    use std::collections::VecDeque;

    struct BlockValidationTestEnvironment {
        node: TestBondNode,
        miner: TestMiner,
        validator: BlockValidator,
        utxo_manager: UtxoManager,
        chain_state: ChainState,
        test_wallets: Vec<TestWallet>,
    }

    impl BlockValidationTestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let node = TestBondNode::new(temp_dir.path()).await;
            let miner = TestMiner::new(node.clone());
            let validator = BlockValidator::new();
            let utxo_manager = UtxoManager::new(temp_dir.path().join("utxos"));
            let chain_state = ChainState::new();
            
            let mut test_wallets = Vec::new();
            for _ in 0..5 {
                test_wallets.push(TestWallet::new_bond());
            }
            
            Self {
                node,
                miner,
                validator,
                utxo_manager,
                chain_state,
                test_wallets,
            }
        }
        
        async fn create_test_block_with_transactions(&mut self, tx_count: usize) -> Block {
            let mut transactions = vec![];
            
            // Add coinbase transaction
            let coinbase = self.miner.create_coinbase_transaction(
                &self.test_wallets[0].address(),
                self.node.get_current_height().await.unwrap() + 1,
            );
            transactions.push(coinbase);
            
            // Add regular transactions
            for i in 0..tx_count {
                if i < self.test_wallets.len() - 1 {
                    let tx = self.test_wallets[i].create_payment_transaction(
                        &self.test_wallets[i + 1].address(),
                        1_000_000,
                        1_000,
                        &self.node.get_utxo_set().await.unwrap(),
                    ).await.unwrap();
                    transactions.push(tx);
                }
            }
            
            self.miner.create_block_with_transactions(transactions).await.unwrap()
        }
    }

    #[tokio::test]
    async fn complete_block_validation_pipeline() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        // Fund first wallet for transactions
        env.node.mine_blocks_to_address(
            &env.test_wallets[0].address(),
            5,
            10_000_000,
        ).await.unwrap();
        
        let initial_height = env.node.get_current_height().await.unwrap();
        let initial_utxo_count = env.utxo_manager.get_utxo_count().await.unwrap();
        
        // Create block with transactions
        let block = env.create_test_block_with_transactions(3).await;
        let block_hash = block.hash();
        
        // Step 1: Basic block structure validation
        let structure_validation = env.validator.validate_block_structure(&block).await;
        assert!(structure_validation.is_ok());
        
        // Step 2: Block header validation
        let parent_block = env.node.get_block_by_height(initial_height).await.unwrap();
        let header_validation = env.validator.validate_block_header(
            &block.header,
            &parent_block.header,
            initial_height + 1,
        ).await;
        assert!(header_validation.is_ok());
        
        // Step 3: Transaction validation within block context
        let tx_validation = env.validator.validate_block_transactions(
            &block.transactions,
            &env.utxo_manager.get_current_set().await.unwrap(),
            initial_height + 1,
        ).await;
        assert!(tx_validation.is_ok());
        
        // Step 4: UTXO set consistency check
        let utxo_validation = env.validator.validate_utxo_consistency(
            &block,
            &mut env.utxo_manager,
        ).await;
        assert!(utxo_validation.is_ok());
        
        // Step 5: Full block validation
        let full_validation = env.validator.validate_complete_block(
            &block,
            &parent_block,
            &mut env.utxo_manager,
            &mut env.chain_state,
        ).await;
        assert!(full_validation.is_ok());
        
        // Step 6: Apply block to chain state
        env.node.add_block(block.clone()).await.unwrap();
        
        // Verify state changes
        let new_height = env.node.get_current_height().await.unwrap();
        assert_eq!(new_height, initial_height + 1);
        
        let new_utxo_count = env.utxo_manager.get_utxo_count().await.unwrap();
        // Should have more UTXOs (coinbase + transaction outputs - spent inputs)
        assert!(new_utxo_count >= initial_utxo_count);
        
        // Verify block retrievable
        let retrieved_block = env.node.get_block_by_hash(&block_hash).await.unwrap();
        assert_eq!(retrieved_block.hash(), block_hash);
    }

    #[tokio::test]
    async fn block_header_validation_comprehensive() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        let parent_block = env.node.get_latest_block().await.unwrap();
        let parent_height = env.node.get_current_height().await.unwrap();
        let next_height = parent_height + 1;
        
        // Create valid block header
        let mut valid_header = BlockHeader {
            previous_hash: parent_block.hash(),
            merkle_root: MerkleTree::compute_root(&[TransactionHash::ZERO]),
            timestamp: parent_block.header.timestamp + 600, // 10 minutes later
            target: env.node.calculate_next_target(&parent_block.header).await.unwrap(),
            nonce: 0,
            height: next_height,
        };
        
        // Mine to find valid nonce
        while !valid_header.meets_target() {
            valid_header.nonce += 1;
        }
        
        // Test 1: Valid header should pass
        let result = env.validator.validate_block_header(
            &valid_header,
            &parent_block.header,
            next_height,
        ).await;
        assert!(result.is_ok());
        
        // Test 2: Wrong previous hash
        let mut wrong_prev_hash = valid_header.clone();
        wrong_prev_hash.previous_hash = BlockHash::from_bytes([0xFF; 32]);
        
        let result = env.validator.validate_block_header(
            &wrong_prev_hash,
            &parent_block.header,
            next_height,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::InvalidPreviousHash { .. } => {}, // Expected
            _ => panic!("Expected InvalidPreviousHash error"),
        }
        
        // Test 3: Timestamp too early
        let mut early_timestamp = valid_header.clone();
        early_timestamp.timestamp = parent_block.header.timestamp - 1;
        early_timestamp.nonce = 0;
        while !early_timestamp.meets_target() {
            early_timestamp.nonce += 1;
        }
        
        let result = env.validator.validate_block_header(
            &early_timestamp,
            &parent_block.header,
            next_height,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::InvalidTimestamp { .. } => {}, // Expected
            _ => panic!("Expected InvalidTimestamp error"),
        }
        
        // Test 4: Wrong target difficulty
        let mut wrong_target = valid_header.clone();
        wrong_target.target = Target::from_compact(0x1d00ffff); // Easy target
        wrong_target.nonce = 0;
        while !wrong_target.meets_target() {
            wrong_target.nonce += 1;
        }
        
        let result = env.validator.validate_block_header(
            &wrong_target,
            &parent_block.header,
            next_height,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::InvalidTarget { .. } => {}, // Expected
            _ => panic!("Expected InvalidTarget error"),
        }
        
        // Test 5: Insufficient proof of work
        let mut insufficient_pow = valid_header.clone();
        insufficient_pow.nonce = 0; // Reset to invalid nonce
        
        let result = env.validator.validate_block_header(
            &insufficient_pow,
            &parent_block.header,
            next_height,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::InsufficientProofOfWork { .. } => {}, // Expected
            _ => panic!("Expected InsufficientProofOfWork error"),
        }
        
        // Test 6: Wrong height
        let mut wrong_height = valid_header.clone();
        wrong_height.height = next_height + 1; // Skip a height
        
        let result = env.validator.validate_block_header(
            &wrong_height,
            &parent_block.header,
            next_height,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BondError::InvalidBlockHeight { .. } => {}, // Expected
            _ => panic!("Expected InvalidBlockHeight error"),
        }
    }

    #[tokio::test]
    async fn transaction_validation_in_block_context() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        // Fund wallets for testing
        for wallet in &env.test_wallets[0..3] {
            env.node.mine_blocks_to_address(
                &wallet.address(),
                3,
                5_000_000,
            ).await.unwrap();
        }
        
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        let current_height = env.node.get_current_height().await.unwrap();
        
        // Create various types of transactions
        let mut transactions = vec![];
        
        // 1. Coinbase transaction
        let coinbase = env.miner.create_coinbase_transaction(
            &env.test_wallets[0].address(),
            current_height + 1,
        );
        transactions.push(coinbase);
        
        // 2. Valid payment transaction
        let payment_tx = env.test_wallets[0].create_payment_transaction(
            &env.test_wallets[1].address(),
            2_000_000,
            5_000,
            &utxo_set,
        ).await.unwrap();
        transactions.push(payment_tx);
        
        // 3. Multi-input transaction
        let multi_input_tx = env.test_wallets[1].create_multi_input_transaction(
            &env.test_wallets[2].address(),
            3_000_000,
            7_500,
            &utxo_set,
        ).await.unwrap();
        transactions.push(multi_input_tx);
        
        // Test 1: Valid transaction set
        let validation_result = env.validator.validate_block_transactions(
            &transactions,
            &utxo_set,
            current_height + 1,
        ).await;
        assert!(validation_result.is_ok());
        
        // Test 2: Duplicate transaction in block
        let mut duplicate_transactions = transactions.clone();
        duplicate_transactions.push(transactions[1].clone()); // Duplicate payment_tx
        
        let duplicate_result = env.validator.validate_block_transactions(
            &duplicate_transactions,
            &utxo_set,
            current_height + 1,
        ).await;
        assert!(duplicate_result.is_err());
        match duplicate_result.unwrap_err() {
            BondError::DuplicateTransactionInBlock { .. } => {}, // Expected
            _ => panic!("Expected DuplicateTransactionInBlock error"),
        }
        
        // Test 3: Double-spend within block
        let double_spend_tx = env.test_wallets[0].create_payment_transaction_with_specific_utxos(
            &env.test_wallets[2].address(),
            1_000_000,
            2_000,
            &utxo_set,
            &[transactions[1].inputs[0].utxo_id], // Same UTXO as payment_tx
        ).await.unwrap();
        
        let mut double_spend_transactions = vec![
            transactions[0].clone(), // Coinbase
            transactions[1].clone(), // Original payment
            double_spend_tx,         // Double spend
        ];
        
        let double_spend_result = env.validator.validate_block_transactions(
            &double_spend_transactions,
            &utxo_set,
            current_height + 1,
        ).await;
        assert!(double_spend_result.is_err());
        match double_spend_result.unwrap_err() {
            BondError::DoubleSpendInBlock { .. } => {}, // Expected
            _ => panic!("Expected DoubleSpendInBlock error"),
        }
        
        // Test 4: Invalid coinbase (multiple coinbases)
        let second_coinbase = env.miner.create_coinbase_transaction(
            &env.test_wallets[1].address(),
            current_height + 1,
        );
        
        let mut multiple_coinbase_transactions = vec![
            transactions[0].clone(), // First coinbase
            second_coinbase,         // Second coinbase
            transactions[1].clone(), // Regular transaction
        ];
        
        let multiple_coinbase_result = env.validator.validate_block_transactions(
            &multiple_coinbase_transactions,
            &utxo_set,
            current_height + 1,
        ).await;
        assert!(multiple_coinbase_result.is_err());
        match multiple_coinbase_result.unwrap_err() {
            BondError::MultipleCoinbaseTransactions => {}, // Expected
            _ => panic!("Expected MultipleCoinbaseTransactions error"),
        }
        
        // Test 5: Missing coinbase
        let no_coinbase_transactions = vec![
            transactions[1].clone(), // Regular transaction only
        ];
        
        let no_coinbase_result = env.validator.validate_block_transactions(
            &no_coinbase_transactions,
            &utxo_set,
            current_height + 1,
        ).await;
        assert!(no_coinbase_result.is_err());
        match no_coinbase_result.unwrap_err() {
            BondError::MissingCoinbaseTransaction => {}, // Expected
            _ => panic!("Expected MissingCoinbaseTransaction error"),
        }
    }

    #[tokio::test]
    async fn utxo_set_update_integration() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        // Fund test wallet
        env.node.mine_blocks_to_address(
            &env.test_wallets[0].address(),
            5,
            20_000_000,
        ).await.unwrap();
        
        let initial_utxo_set = env.utxo_manager.get_current_set().await.unwrap();
        let initial_count = initial_utxo_set.len();
        
        // Create block with transactions that modify UTXO set
        let mut transactions = vec![];
        
        // Coinbase (adds 1 UTXO)
        let coinbase = env.miner.create_coinbase_transaction(
            &env.test_wallets[1].address(),
            env.node.get_current_height().await.unwrap() + 1,
        );
        transactions.push(coinbase);
        
        // Payment transaction (spends 1 UTXO, creates 2 UTXOs - payment + change)
        let payment_tx = env.test_wallets[0].create_payment_transaction(
            &env.test_wallets[2].address(),
            5_000_000, // 5 BND payment
            10_000,    // 0.01 BND fee
            &initial_utxo_set,
        ).await.unwrap();
        
        let spent_utxo_id = payment_tx.inputs[0].utxo_id;
        transactions.push(payment_tx);
        
        // Create block
        let block = env.miner.create_block_with_transactions(transactions).await.unwrap();
        
        // Validate UTXO consistency
        let utxo_validation = env.validator.validate_utxo_consistency(
            &block,
            &mut env.utxo_manager,
        ).await;
        assert!(utxo_validation.is_ok());
        
        // Apply block and update UTXO set
        env.node.add_block(block.clone()).await.unwrap();
        env.utxo_manager.apply_block(&block).await.unwrap();
        
        // Verify UTXO set changes
        let final_utxo_set = env.utxo_manager.get_current_set().await.unwrap();
        
        // Should not contain spent UTXO
        assert!(!final_utxo_set.contains_key(&spent_utxo_id));
        
        // Should contain new UTXOs from coinbase and payment transaction
        let coinbase_utxo_id = UtxoId::new(block.transactions[0].hash(), 0);
        assert!(final_utxo_set.contains_key(&coinbase_utxo_id));
        
        let payment_output_utxo_id = UtxoId::new(block.transactions[1].hash(), 0);
        assert!(final_utxo_set.contains_key(&payment_output_utxo_id));
        
        // Verify balances
        let wallet2_balance = env.node.get_balance(&env.test_wallets[2].address()).await.unwrap();
        assert_eq!(wallet2_balance, 5_000_000); // Should have received payment
        
        // Verify UTXO count change
        let final_count = final_utxo_set.len();
        // initial_count - 1 (spent) + 1 (coinbase) + 1 (payment) + 1 (change) = initial_count + 2
        assert_eq!(final_count, initial_count + 2);
    }

    #[tokio::test]
    async fn block_merkle_root_validation() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        // Create transactions for block
        env.node.mine_blocks_to_address(
            &env.test_wallets[0].address(),
            3,
            10_000_000,
        ).await.unwrap();
        
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        let mut transactions = vec![];
        
        // Add coinbase
        let coinbase = env.miner.create_coinbase_transaction(
            &env.test_wallets[0].address(),
            env.node.get_current_height().await.unwrap() + 1,
        );
        transactions.push(coinbase);
        
        // Add regular transactions
        for i in 0..3 {
            let tx = env.test_wallets[0].create_payment_transaction(
                &env.test_wallets[(i + 1) % env.test_wallets.len()].address(),
                1_000_000,
                1_000,
                &utxo_set,
            ).await.unwrap();
            transactions.push(tx);
        }
        
        // Calculate correct Merkle root
        let tx_hashes: Vec<TransactionHash> = transactions.iter()
            .map(|tx| tx.hash())
            .collect();
        let correct_merkle_root = MerkleTree::compute_root(&tx_hashes);
        
        // Create block with correct Merkle root
        let mut block = env.miner.create_block_with_transactions(transactions.clone()).await.unwrap();
        assert_eq!(block.header.merkle_root, correct_merkle_root);
        
        // Test 1: Correct Merkle root should validate
        let validation = env.validator.validate_block_structure(&block).await;
        assert!(validation.is_ok());
        
        // Test 2: Incorrect Merkle root should fail
        let mut invalid_block = block.clone();
        invalid_block.header.merkle_root = MerkleRoot::from_bytes([0xFF; 32]);
        
        let invalid_validation = env.validator.validate_block_structure(&invalid_block).await;
        assert!(invalid_validation.is_err());
        match invalid_validation.unwrap_err() {
            BondError::InvalidMerkleRoot { .. } => {}, // Expected
            _ => panic!("Expected InvalidMerkleRoot error"),
        }
        
        // Test 3: Merkle root with modified transactions should fail
        let mut modified_tx_block = block.clone();
        // Modify last transaction amount
        if let Some(last_tx) = modified_tx_block.transactions.last_mut() {
            if let Some(output) = last_tx.outputs.first_mut() {
                output.value += 1; // Small modification
            }
        }
        // Keep original Merkle root (now incorrect)
        
        let modified_validation = env.validator.validate_block_structure(&modified_tx_block).await;
        assert!(modified_validation.is_err());
        match modified_validation.unwrap_err() {
            BondError::InvalidMerkleRoot { .. } => {}, // Expected
            _ => panic!("Expected InvalidMerkleRoot error"),
        }
    }
}
```

### Block Chain Reorganization Tests
```rust
#[cfg(test)]
mod block_reorganization_tests {
    use super::*;

    #[tokio::test]
    async fn simple_chain_reorganization() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        // Mine initial chain: Genesis -> A -> B
        let block_a = env.miner.mine_next_block().await.unwrap();
        env.node.add_block(block_a.clone()).await.unwrap();
        
        let block_b = env.miner.mine_next_block().await.unwrap();
        env.node.add_block(block_b.clone()).await.unwrap();
        
        let main_chain_height = env.node.get_current_height().await.unwrap();
        assert_eq!(main_chain_height, 2);
        
        // Create alternative chain: Genesis -> A' -> B' -> C'
        // A' has same height as A but different content
        let block_a_prime = env.miner.mine_block_with_different_nonce(&block_a).await.unwrap();
        let block_b_prime = env.miner.mine_block_after(&block_a_prime).await.unwrap();
        let block_c_prime = env.miner.mine_block_after(&block_b_prime).await.unwrap();
        
        // Alternative chain should be longer and trigger reorganization
        assert_eq!(block_c_prime.header.height, 3);
        
        // Submit alternative chain
        let reorg_result = env.node.try_reorganization(vec![
            block_a_prime.clone(),
            block_b_prime.clone(),
            block_c_prime.clone(),
        ]).await;
        
        assert!(reorg_result.is_ok());
        
        // Verify reorganization occurred
        let new_height = env.node.get_current_height().await.unwrap();
        assert_eq!(new_height, 3);
        
        let current_tip = env.node.get_latest_block().await.unwrap();
        assert_eq!(current_tip.hash(), block_c_prime.hash());
        
        // Verify old chain is marked as orphaned
        let block_b_status = env.node.get_block_status(&block_b.hash()).await.unwrap();
        assert_eq!(block_b_status, BlockStatus::Orphaned);
        
        // Verify UTXO set consistency after reorganization
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        let utxo_validation = env.validator.validate_utxo_set_consistency(&utxo_set).await;
        assert!(utxo_validation.is_ok());
    }

    #[tokio::test]
    async fn reorganization_with_conflicting_transactions() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        // Fund test wallet
        env.node.mine_blocks_to_address(
            &env.test_wallets[0].address(),
            5,
            10_000_000,
        ).await.unwrap();
        
        let initial_height = env.node.get_current_height().await.unwrap();
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        
        // Create two conflicting transactions (double-spend)
        let tx_to_bob = env.test_wallets[0].create_payment_transaction(
            &env.test_wallets[1].address(),
            3_000_000,
            5_000,
            &utxo_set,
        ).await.unwrap();
        
        let tx_to_charlie = env.test_wallets[0].create_payment_transaction_with_specific_utxos(
            &env.test_wallets[2].address(),
            2_500_000,
            7_500,
            &utxo_set,
            &[tx_to_bob.inputs[0].utxo_id], // Same UTXO
        ).await.unwrap();
        
        // Chain A: Include tx_to_bob
        let block_a = env.miner.create_block_with_transactions(vec![
            env.miner.create_coinbase_transaction(
                &env.test_wallets[0].address(),
                initial_height + 1,
            ),
            tx_to_bob.clone(),
        ]).await.unwrap();
        
        env.node.add_block(block_a.clone()).await.unwrap();
        
        // Verify Bob received payment
        let bob_balance = env.node.get_balance(&env.test_wallets[1].address()).await.unwrap();
        assert_eq!(bob_balance, 3_000_000);
        
        // Chain B: Alternative chain including tx_to_charlie
        let block_b_prime = env.miner.create_block_with_transactions_at_height(vec![
            env.miner.create_coinbase_transaction(
                &env.test_wallets[0].address(),
                initial_height + 1,
            ),
            tx_to_charlie.clone(),
        ], initial_height + 1).await.unwrap();
        
        let block_c_prime = env.miner.mine_block_after(&block_b_prime).await.unwrap();
        
        // Try reorganization to longer chain
        let reorg_result = env.node.try_reorganization(vec![
            block_b_prime.clone(),
            block_c_prime.clone(),
        ]).await;
        
        assert!(reorg_result.is_ok());
        
        // Verify reorganization effects
        let new_height = env.node.get_current_height().await.unwrap();
        assert_eq!(new_height, initial_height + 2);
        
        // Bob should no longer have the payment (tx_to_bob was orphaned)
        let bob_final_balance = env.node.get_balance(&env.test_wallets[1].address()).await.unwrap();
        assert_eq!(bob_final_balance, 0);
        
        // Charlie should now have the payment
        let charlie_balance = env.node.get_balance(&env.test_wallets[2].address()).await.unwrap();
        assert_eq!(charlie_balance, 2_500_000);
        
        // Original transaction should be marked as orphaned
        let tx_bob_status = env.node.get_transaction_status(&tx_to_bob.hash()).await.unwrap();
        assert_eq!(tx_bob_status, TransactionStatus::Orphaned);
        
        // New transaction should be confirmed
        let tx_charlie_status = env.node.get_transaction_status(&tx_to_charlie.hash()).await.unwrap();
        assert_eq!(tx_charlie_status, TransactionStatus::Confirmed);
    }

    #[tokio::test]
    async fn deep_reorganization_limits() {
        let mut env = BlockValidationTestEnvironment::new().await;
        
        // Build main chain of 50 blocks
        let main_chain_length = 50;
        let mut main_chain_blocks = vec![];
        
        for _ in 0..main_chain_length {
            let block = env.miner.mine_next_block().await.unwrap();
            env.node.add_block(block.clone()).await.unwrap();
            main_chain_blocks.push(block);
        }
        
        let main_chain_height = env.node.get_current_height().await.unwrap();
        assert_eq!(main_chain_height, main_chain_length);
        
        // Try to create alternative chain that would cause very deep reorganization
        // Alternative chain starting from block 10
        let fork_point = 10;
        let alt_chain_length = 60; // Longer than main chain
        
        let mut alt_chain = vec![];
        let fork_parent = &main_chain_blocks[fork_point - 1];
        
        let mut current_parent = fork_parent.clone();
        for i in 0..alt_chain_length {
            let alt_block = env.miner.mine_block_after(&current_parent).await.unwrap();
            alt_chain.push(alt_block.clone());
            current_parent = alt_block;
        }
        
        // Attempt reorganization
        let reorg_result = env.node.try_reorganization(alt_chain).await;
        
        // Deep reorganization should be rejected (security measure)
        match reorg_result {
            Err(BondError::ReorganizationTooDeep { depth, max_depth }) => {
                assert!(depth > max_depth);
                assert!(max_depth <= 100); // Reasonable limit
            }
            Ok(_) => {
                // If allowed, verify it worked correctly
                let new_height = env.node.get_current_height().await.unwrap();
                assert!(new_height > main_chain_height);
            }
            _ => panic!("Unexpected reorganization result"),
        }
    }
}
```
