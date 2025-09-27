# Camada 2: Integration Tests - Transaction Lifecycle

## 2.2 Testes de Fluxos Completos de Transação

### Complete Transaction Flow Tests
```rust
#[cfg(test)]
mod transaction_lifecycle_tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    struct TransactionTestEnvironment {
        node: TestBondNode,
        mempool: Arc<Mutex<Mempool>>,
        miner: TestMiner,
        wallet_alice: TestWallet,
        wallet_bob: TestWallet,
        wallet_charlie: TestWallet,
    }

    impl TransactionTestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let node = TestBondNode::new(temp_dir.path()).await;
            let mempool = Arc::new(Mutex::new(Mempool::new()));
            let miner = TestMiner::new(node.clone());
            
            let wallet_alice = TestWallet::new_bond();
            let wallet_bob = TestWallet::new_bond();
            let wallet_charlie = TestWallet::new_bond();
            
            Self {
                node,
                mempool,
                miner,
                wallet_alice,
                wallet_bob,
                wallet_charlie,
            }
        }
        
        async fn fund_wallet(&mut self, wallet: &TestWallet, amount: u64) {
            self.node.mine_blocks_to_address(
                &wallet.address(),
                5,
                amount
            ).await.unwrap();
        }
    }

    #[tokio::test]
    async fn simple_payment_flow() {
        let mut env = TransactionTestEnvironment::new().await;
        
        // Setup: Fund Alice with 10 BND
        env.fund_wallet(&env.wallet_alice, 10_000_000).await;
        
        let alice_initial_balance = env.node
            .get_balance(&env.wallet_alice.address())
            .await
            .unwrap();
        assert_eq!(alice_initial_balance, 10_000_000);
        
        let bob_initial_balance = env.node
            .get_balance(&env.wallet_bob.address())
            .await
            .unwrap();
        assert_eq!(bob_initial_balance, 0);
        
        // Step 1: Alice creates transaction to send 3 BND to Bob
        let payment_amount = 3_000_000;
        let fee_amount = 1_000; // 0.001 BND fee
        
        let transaction = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            payment_amount,
            fee_amount,
            &env.node.get_utxo_set().await.unwrap(),
        ).await.unwrap();
        
        let tx_hash = transaction.hash();
        
        // Step 2: Transaction submitted to mempool
        {
            let mut mempool = env.mempool.lock().unwrap();
            let result = mempool.add_transaction(transaction.clone());
            assert!(result.is_ok());
            assert_eq!(mempool.size(), 1);
        }
        
        // Step 3: Verify transaction in mempool
        let mempool_transactions = {
            let mempool = env.mempool.lock().unwrap();
            mempool.get_prioritized_transactions(10)
        };
        
        assert_eq!(mempool_transactions.len(), 1);
        assert_eq!(mempool_transactions[0].hash(), tx_hash);
        
        // Step 4: Miner picks up transaction and mines block
        let block = env.miner.mine_block_with_transactions(
            mempool_transactions
        ).await.unwrap();
        
        assert_eq!(block.transactions.len(), 2); // Coinbase + payment
        assert_eq!(block.transactions[1].hash(), tx_hash);
        
        // Step 5: Block added to chain
        env.node.add_block(block).await.unwrap();
        
        // Step 6: Transaction removed from mempool
        {
            let mut mempool = env.mempool.lock().unwrap();
            mempool.remove_confirmed_transactions(&[tx_hash]);
            assert_eq!(mempool.size(), 0);
        }
        
        // Step 7: Verify final balances
        let alice_final_balance = env.node
            .get_balance(&env.wallet_alice.address())
            .await
            .unwrap();
        
        let bob_final_balance = env.node
            .get_balance(&env.wallet_bob.address())
            .await
            .unwrap();
        
        // Alice should have: 10 BND - 3 BND (payment) - 0.001 BND (fee) = 6.999 BND
        let expected_alice_balance = alice_initial_balance - payment_amount - fee_amount;
        assert_eq!(alice_final_balance, expected_alice_balance);
        
        // Bob should have: 0 + 3 BND = 3 BND
        assert_eq!(bob_final_balance, payment_amount);
        
        // Step 8: Verify UTXO set updated correctly
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        
        // Alice's original UTXO should be spent
        let alice_original_utxos = env.node
            .get_utxos_for_address(&env.wallet_alice.address())
            .await
            .unwrap();
        
        // Bob should have new UTXO
        let bob_utxos = env.node
            .get_utxos_for_address(&env.wallet_bob.address())
            .await
            .unwrap();
        
        assert!(bob_utxos.len() > 0);
        assert_eq!(bob_utxos[0].value, payment_amount);
    }

    #[tokio::test]
    async fn multi_input_multi_output_transaction() {
        let mut env = TransactionTestEnvironment::new().await;
        
        // Fund Alice with multiple UTXOs
        env.fund_wallet(&env.wallet_alice, 2_000_000).await; // 2 BND
        env.fund_wallet(&env.wallet_alice, 3_000_000).await; // 3 BND
        env.fund_wallet(&env.wallet_alice, 4_000_000).await; // 4 BND
        
        let alice_initial_balance = env.node
            .get_balance(&env.wallet_alice.address())
            .await
            .unwrap();
        assert_eq!(alice_initial_balance, 9_000_000); // Total 9 BND
        
        // Create transaction: Alice sends 2 BND to Bob, 3 BND to Charlie
        let bob_amount = 2_000_000;
        let charlie_amount = 3_000_000;
        let fee = 5_000; // 0.005 BND
        
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        let alice_utxos = env.node
            .get_utxos_for_address(&env.wallet_alice.address())
            .await
            .unwrap();
        
        // Use first two UTXOs (2 BND + 3 BND = 5 BND input)
        let inputs = vec![
            TransactionInput {
                utxo_id: alice_utxos[0].id,
                script_sig: env.wallet_alice.create_script_sig(&alice_utxos[0]).await.unwrap(),
            },
            TransactionInput {
                utxo_id: alice_utxos[1].id,
                script_sig: env.wallet_alice.create_script_sig(&alice_utxos[1]).await.unwrap(),
            },
        ];
        
        let outputs = vec![
            TransactionOutput {
                value: bob_amount,
                script_pubkey: env.wallet_bob.get_script_pubkey(),
            },
            TransactionOutput {
                value: charlie_amount,
                script_pubkey: env.wallet_charlie.get_script_pubkey(),
            },
            // No change output needed since 2+3 = 2+3 exactly (plus fee from remaining UTXO)
        ];
        
        let transaction = Transaction::new(inputs, outputs).unwrap();
        let tx_hash = transaction.hash();
        
        // Submit and mine transaction
        {
            let mut mempool = env.mempool.lock().unwrap();
            mempool.add_transaction(transaction).unwrap();
        }
        
        let mempool_txs = {
            let mempool = env.mempool.lock().unwrap();
            mempool.get_prioritized_transactions(10)
        };
        
        let block = env.miner.mine_block_with_transactions(mempool_txs).await.unwrap();
        env.node.add_block(block).await.unwrap();
        
        // Verify final balances
        let alice_final = env.node.get_balance(&env.wallet_alice.address()).await.unwrap();
        let bob_final = env.node.get_balance(&env.wallet_bob.address()).await.unwrap();
        let charlie_final = env.node.get_balance(&env.wallet_charlie.address()).await.unwrap();
        
        // Alice spent 2 UTXOs (2+3 BND), still has 1 UTXO (4 BND)
        assert_eq!(alice_final, 4_000_000);
        assert_eq!(bob_final, bob_amount);
        assert_eq!(charlie_final, charlie_amount);
    }

    #[tokio::test]
    async fn transaction_replacement_rbf() {
        let mut env = TransactionTestEnvironment::new().await;
        env.fund_wallet(&env.wallet_alice, 5_000_000).await;
        
        // Create initial transaction with low fee
        let payment_amount = 2_000_000;
        let low_fee = 500; // 0.0005 BND
        
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        let tx_low_fee = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            payment_amount,
            low_fee,
            &utxo_set,
        ).await.unwrap();
        
        let tx_low_fee_hash = tx_low_fee.hash();
        
        // Add to mempool
        {
            let mut mempool = env.mempool.lock().unwrap();
            mempool.add_transaction(tx_low_fee).unwrap();
            assert_eq!(mempool.size(), 1);
        }
        
        // Create replacement transaction with higher fee (same inputs)
        let high_fee = 2_000; // 0.002 BND
        let tx_high_fee = env.wallet_alice.create_payment_transaction_with_specific_utxos(
            &env.wallet_bob.address(),
            payment_amount,
            high_fee,
            &utxo_set,
            &[tx_low_fee.inputs[0].utxo_id], // Same UTXO
        ).await.unwrap();
        
        let tx_high_fee_hash = tx_high_fee.hash();
        
        // Replace-by-Fee (RBF)
        {
            let mut mempool = env.mempool.lock().unwrap();
            let result = mempool.replace_transaction(tx_high_fee);
            assert!(result.is_ok());
            
            assert_eq!(mempool.size(), 1);
            assert!(!mempool.contains(&tx_low_fee_hash));
            assert!(mempool.contains(&tx_high_fee_hash));
        }
        
        // Mine the replacement transaction
        let mempool_txs = {
            let mempool = env.mempool.lock().unwrap();
            mempool.get_prioritized_transactions(10)
        };
        
        assert_eq!(mempool_txs.len(), 1);
        assert_eq!(mempool_txs[0].hash(), tx_high_fee_hash);
        
        let block = env.miner.mine_block_with_transactions(mempool_txs).await.unwrap();
        env.node.add_block(block).await.unwrap();
        
        // Verify only the high-fee transaction was mined
        let tx_status_low = env.node.get_transaction_status(&tx_low_fee_hash).await.unwrap();
        let tx_status_high = env.node.get_transaction_status(&tx_high_fee_hash).await.unwrap();
        
        assert_eq!(tx_status_low, TransactionStatus::Replaced);
        assert_eq!(tx_status_high, TransactionStatus::Confirmed);
        
        // Verify correct fee was paid
        let alice_balance = env.node.get_balance(&env.wallet_alice.address()).await.unwrap();
        let expected_balance = 5_000_000 - payment_amount - high_fee;
        assert_eq!(alice_balance, expected_balance);
    }
}
```

### Programmable UTXO Transaction Flows
```rust
#[cfg(test)]
mod putxo_transaction_flows {
    use super::*;

    #[tokio::test]
    async fn time_locked_transaction_flow() {
        let mut env = TransactionTestEnvironment::new().await;
        env.fund_wallet(&env.wallet_alice, 5_000_000).await;
        
        // Create time-locked pUTXO (unlocks at block height 50)
        let current_height = env.node.get_current_height().await.unwrap();
        let unlock_height = current_height + 10;
        
        let time_lock_script = Script::create_time_lock_script(unlock_height);
        let time_locked_utxo = env.wallet_alice.create_programmable_utxo(
            2_000_000, // 2 BND
            time_lock_script,
        ).await.unwrap();
        
        let lock_tx_hash = env.node.submit_transaction(time_locked_utxo.creation_tx).await.unwrap();
        
        // Mine block to confirm time-locked UTXO creation
        env.node.mine_blocks(1).await.unwrap();
        
        // Try to spend before unlock height (should fail)
        let early_spend_tx = env.wallet_alice.create_spend_programmable_utxo(
            &time_locked_utxo.utxo_id,
            &env.wallet_bob.address(),
            1_500_000,
            500,
        ).await.unwrap();
        
        let early_result = env.node.validate_transaction(&early_spend_tx).await;
        assert!(early_result.is_err());
        match early_result.unwrap_err() {
            BondError::TimeLockNotExpired { .. } => {}, // Expected
            _ => panic!("Expected TimeLockNotExpired error"),
        }
        
        // Mine blocks to reach unlock height
        let blocks_to_mine = unlock_height - env.node.get_current_height().await.unwrap();
        env.node.mine_blocks(blocks_to_mine as u32).await.unwrap();
        
        // Now spending should succeed
        let spend_tx = env.wallet_alice.create_spend_programmable_utxo(
            &time_locked_utxo.utxo_id,
            &env.wallet_bob.address(),
            1_500_000,
            500,
        ).await.unwrap();
        
        let spend_result = env.node.validate_transaction(&spend_tx).await;
        assert!(spend_result.is_ok());
        
        // Submit and mine the spending transaction
        let spend_tx_hash = env.node.submit_transaction(spend_tx).await.unwrap();
        env.node.mine_blocks(1).await.unwrap();
        
        // Verify Bob received the funds
        let bob_balance = env.node.get_balance(&env.wallet_bob.address()).await.unwrap();
        assert_eq!(bob_balance, 1_500_000);
    }

    #[tokio::test]
    async fn multi_signature_transaction_flow() {
        let mut env = TransactionTestEnvironment::new().await;
        env.fund_wallet(&env.wallet_alice, 5_000_000).await;
        
        // Create 2-of-3 multisig UTXO (Alice, Bob, Charlie)
        let multisig_script = Script::create_multisig_script(
            2, // Required signatures
            vec![
                env.wallet_alice.public_key(),
                env.wallet_bob.public_key(),
                env.wallet_charlie.public_key(),
            ],
        );
        
        let multisig_utxo = env.wallet_alice.create_programmable_utxo(
            3_000_000, // 3 BND
            multisig_script,
        ).await.unwrap();
        
        env.node.submit_transaction(multisig_utxo.creation_tx).await.unwrap();
        env.node.mine_blocks(1).await.unwrap();
        
        // Create spending transaction requiring 2 signatures
        let spend_amount = 2_500_000;
        let fee = 1_000;
        
        let mut spend_tx = Transaction::new(
            vec![TransactionInput {
                utxo_id: multisig_utxo.utxo_id,
                script_sig: Script::empty(), // Will be filled with signatures
            }],
            vec![TransactionOutput {
                value: spend_amount,
                script_pubkey: env.wallet_alice.get_script_pubkey(),
            }],
        ).unwrap();
        
        // Alice signs first
        let alice_signature = env.wallet_alice.sign_transaction(&spend_tx).await.unwrap();
        
        // Bob signs second
        let bob_signature = env.wallet_bob.sign_transaction(&spend_tx).await.unwrap();
        
        // Construct script_sig with 2 signatures
        let script_sig = Script::create_multisig_script_sig(vec![
            alice_signature,
            bob_signature,
        ]);
        
        spend_tx.inputs[0].script_sig = script_sig;
        
        // Validate and submit
        let validation_result = env.node.validate_transaction(&spend_tx).await;
        assert!(validation_result.is_ok());
        
        let spend_tx_hash = env.node.submit_transaction(spend_tx).await.unwrap();
        env.node.mine_blocks(1).await.unwrap();
        
        // Verify transaction confirmed
        let tx_status = env.node.get_transaction_status(&spend_tx_hash).await.unwrap();
        assert_eq!(tx_status, TransactionStatus::Confirmed);
    }

    #[tokio::test]
    async fn social_recovery_transaction_flow() {
        let mut env = TransactionTestEnvironment::new().await;
        env.fund_wallet(&env.wallet_alice, 10_000_000).await;
        
        // Create social recovery UTXO with 2-of-3 guardians
        let guardian1 = TestWallet::new_bond();
        let guardian2 = TestWallet::new_bond();
        let guardian3 = TestWallet::new_bond();
        
        let recovery_script = Script::create_social_recovery_script(
            env.wallet_alice.public_key(), // Owner
            vec![
                guardian1.public_key(),
                guardian2.public_key(),
                guardian3.public_key(),
            ], // Guardians
            2, // Required guardian signatures
            144, // Recovery delay (blocks)
        );
        
        let recovery_utxo = env.wallet_alice.create_programmable_utxo(
            5_000_000, // 5 BND
            recovery_script,
        ).await.unwrap();
        
        env.node.submit_transaction(recovery_utxo.creation_tx).await.unwrap();
        env.node.mine_blocks(1).await.unwrap();
        
        // Simulate Alice losing her keys - initiate recovery
        let recovery_address = env.wallet_bob.address(); // Recovery to Bob's address
        
        // Step 1: Guardian1 and Guardian2 initiate recovery
        let recovery_initiation_tx = guardian1.create_recovery_initiation(
            &recovery_utxo.utxo_id,
            &recovery_address,
            vec![
                guardian1.sign_recovery_message(b"recovery_init").await.unwrap(),
                guardian2.sign_recovery_message(b"recovery_init").await.unwrap(),
            ],
        ).await.unwrap();
        
        env.node.submit_transaction(recovery_initiation_tx).await.unwrap();
        env.node.mine_blocks(1).await.unwrap();
        
        // Step 2: Wait for recovery delay period
        env.node.mine_blocks(144).await.unwrap();
        
        // Step 3: Execute recovery
        let recovery_execution_tx = guardian1.create_recovery_execution(
            &recovery_utxo.utxo_id,
            &recovery_address,
            4_900_000, // Amount minus fees
        ).await.unwrap();
        
        let recovery_tx_hash = env.node.submit_transaction(recovery_execution_tx).await.unwrap();
        env.node.mine_blocks(1).await.unwrap();
        
        // Verify recovery successful
        let bob_balance = env.node.get_balance(&env.wallet_bob.address()).await.unwrap();
        assert_eq!(bob_balance, 4_900_000);
        
        let tx_status = env.node.get_transaction_status(&recovery_tx_hash).await.unwrap();
        assert_eq!(tx_status, TransactionStatus::Confirmed);
    }
}
```

### Transaction Validation Integration
```rust
#[cfg(test)]
mod transaction_validation_integration {
    use super::*;

    #[tokio::test]
    async fn comprehensive_transaction_validation() {
        let mut env = TransactionTestEnvironment::new().await;
        env.fund_wallet(&env.wallet_alice, 10_000_000).await;
        
        // Test various validation scenarios in sequence
        
        // 1. Valid transaction
        let valid_tx = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            1_000_000,
            1_000,
            &env.node.get_utxo_set().await.unwrap(),
        ).await.unwrap();
        
        let validation = env.node.validate_transaction(&valid_tx).await;
        assert!(validation.is_ok());
        
        // 2. Double-spend attempt
        let double_spend_tx = env.wallet_alice.create_payment_transaction_with_specific_utxos(
            &env.wallet_charlie.address(),
            2_000_000,
            1_000,
            &env.node.get_utxo_set().await.unwrap(),
            &[valid_tx.inputs[0].utxo_id], // Same UTXO
        ).await.unwrap();
        
        // Submit first transaction
        env.node.submit_transaction(valid_tx).await.unwrap();
        
        // Double-spend should be rejected
        let double_spend_result = env.node.validate_transaction(&double_spend_tx).await;
        assert!(double_spend_result.is_err());
        
        // 3. Insufficient balance
        let insufficient_tx = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            50_000_000, // More than Alice has
            1_000,
            &env.node.get_utxo_set().await.unwrap(),
        ).await;
        
        assert!(insufficient_tx.is_err());
        
        // 4. Invalid script
        let mut invalid_script_tx = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            500_000,
            1_000,
            &env.node.get_utxo_set().await.unwrap(),
        ).await.unwrap();
        
        // Corrupt the script signature
        invalid_script_tx.inputs[0].script_sig = Script::from_bytes(vec![0xFF, 0xFF]);
        
        let invalid_result = env.node.validate_transaction(&invalid_script_tx).await;
        assert!(invalid_result.is_err());
        
        // 5. Zero-value output (dust)
        let dust_outputs = vec![
            TransactionOutput {
                value: 0, // Zero value
                script_pubkey: env.wallet_bob.get_script_pubkey(),
            }
        ];
        
        let alice_utxos = env.node.get_utxos_for_address(&env.wallet_alice.address()).await.unwrap();
        let dust_tx = Transaction::new(
            vec![TransactionInput {
                utxo_id: alice_utxos[0].id,
                script_sig: env.wallet_alice.create_script_sig(&alice_utxos[0]).await.unwrap(),
            }],
            dust_outputs,
        );
        
        assert!(dust_tx.is_err()); // Should fail at construction
    }

    #[tokio::test]
    async fn transaction_fee_validation() {
        let mut env = TransactionTestEnvironment::new().await;
        env.fund_wallet(&env.wallet_alice, 5_000_000).await;
        
        let utxo_set = env.node.get_utxo_set().await.unwrap();
        
        // Test minimum fee requirement
        let low_fee_tx = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            1_000_000,
            1, // 1 satoshi fee (too low)
            &utxo_set,
        ).await.unwrap();
        
        let low_fee_validation = env.node.validate_transaction(&low_fee_tx).await;
        
        // Depending on policy, might be rejected or accepted with warning
        match low_fee_validation {
            Err(BondError::InsufficientFee { .. }) => {}, // Expected
            Ok(_) => {
                // If accepted, should be lowest priority in mempool
                let mut mempool = env.mempool.lock().unwrap();
                mempool.add_transaction(low_fee_tx).unwrap();
                
                let prioritized = mempool.get_prioritized_transactions(10);
                // Should be last in priority order
            }
            _ => panic!("Unexpected validation result"),
        }
        
        // Test reasonable fee
        let good_fee_tx = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            1_000_000,
            5_000, // 0.005 BND fee
            &utxo_set,
        ).await.unwrap();
        
        let good_fee_validation = env.node.validate_transaction(&good_fee_tx).await;
        assert!(good_fee_validation.is_ok());
        
        // Test excessive fee (likely user error)
        let high_fee_tx = env.wallet_alice.create_payment_transaction(
            &env.wallet_bob.address(),
            1_000_000,
            1_000_000, // 1 BND fee (100% fee rate)
            &utxo_set,
        ).await.unwrap();
        
        let high_fee_validation = env.node.validate_transaction(&high_fee_tx).await;
        
        // Should validate but might trigger warning
        assert!(high_fee_validation.is_ok());
        
        // Fee calculation should be correct
        let calculated_fee = high_fee_tx.calculate_fee(&utxo_set).unwrap();
        assert_eq!(calculated_fee, 1_000_000);
    }
}
```
