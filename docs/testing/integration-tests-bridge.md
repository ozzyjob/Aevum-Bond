# Camada 2: Integration Tests - Bridge Bond-Aevum

## 2.1 Testes de Integração Bridge Bond-Aevum

### Basic Bridge Operations
```rust
#[cfg(test)]
mod bridge_integration_tests {
    use super::*;
    use tokio_test;
    
    struct BridgeTestEnvironment {
        bond_node: TestBondNode,
        aevum_node: TestAevumNode,
        bridge: InterLedgerBridge,
        bond_wallet: TestWallet,
        aevum_wallet: TestWallet,
    }
    
    impl BridgeTestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            
            let bond_node = TestBondNode::new(temp_dir.path().join("bond")).await;
            let aevum_node = TestAevumNode::new(temp_dir.path().join("aevum")).await;
            
            let bridge = InterLedgerBridge::new(
                bond_node.rpc_endpoint(),
                aevum_node.rpc_endpoint(),
            ).await;
            
            let bond_wallet = TestWallet::new_bond();
            let aevum_wallet = TestWallet::new_aevum();
            
            Self {
                bond_node,
                aevum_node,
                bridge,
                bond_wallet,
                aevum_wallet,
            }
        }
        
        async fn fund_bond_wallet(&mut self, amount: u64) {
            // Mine blocks to fund test wallet
            self.bond_node.mine_blocks_to_address(
                &self.bond_wallet.address(),
                10, // blocks
                amount
            ).await.unwrap();
        }
    }

    #[tokio::test]
    async fn bridge_bond_to_aevum() {
        let mut env = BridgeTestEnvironment::new().await;
        env.fund_bond_wallet(10_000_000).await; // 10 BND
        
        let initial_bond_balance = env.bond_node
            .get_balance(&env.bond_wallet.address())
            .await
            .unwrap();
        assert_eq!(initial_bond_balance, 10_000_000);
        
        let initial_aevum_balance = env.aevum_node
            .get_balance(&env.aevum_wallet.address())
            .await
            .unwrap();
        assert_eq!(initial_aevum_balance, 0);
        
        // Initiate bridge transfer: 5 BND → 5 AEV
        let bridge_amount = 5_000_000;
        let bridge_tx = env.bridge.create_bond_to_aevum_transfer(
            &env.bond_wallet,
            &env.aevum_wallet.address(),
            bridge_amount,
        ).await.unwrap();
        
        // Submit to Bond network
        let bond_tx_hash = env.bond_node
            .submit_transaction(bridge_tx.bond_lock_tx)
            .await
            .unwrap();
        
        // Wait for Bond confirmation (6 blocks)
        env.bond_node.mine_blocks(6).await.unwrap();
        
        let bond_tx_status = env.bond_node
            .get_transaction_status(&bond_tx_hash)
            .await
            .unwrap();
        assert_eq!(bond_tx_status, TransactionStatus::Confirmed);
        
        // Bridge should detect the lock and create Aevum mint
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let aevum_mint_tx = env.bridge
            .get_pending_aevum_mint(&bond_tx_hash)
            .await
            .unwrap();
        assert!(aevum_mint_tx.is_some());
        
        // Process Aevum mint transaction
        let aevum_tx_hash = env.aevum_node
            .submit_transaction(aevum_mint_tx.unwrap())
            .await
            .unwrap();
        
        // Wait for Aevum confirmation
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify final balances
        let final_bond_balance = env.bond_node
            .get_balance(&env.bond_wallet.address())
            .await
            .unwrap();
        
        let final_aevum_balance = env.aevum_node
            .get_balance(&env.aevum_wallet.address())
            .await
            .unwrap();
        
        // Bond balance should be reduced (minus fees)
        assert!(final_bond_balance < initial_bond_balance - bridge_amount);
        
        // Aevum balance should increase by bridge amount
        assert_eq!(final_aevum_balance, bridge_amount);
        
        // Verify bridge state
        let bridge_state = env.bridge.get_transfer_state(&bond_tx_hash).await.unwrap();
        assert_eq!(bridge_state.status, BridgeTransferStatus::Completed);
        assert_eq!(bridge_state.amount, bridge_amount);
    }

    #[tokio::test]
    async fn bridge_aevum_to_bond() {
        let mut env = BridgeTestEnvironment::new().await;
        
        // First bridge Bond to Aevum to have AEV balance
        env.fund_bond_wallet(10_000_000).await;
        
        let bridge_tx = env.bridge.create_bond_to_aevum_transfer(
            &env.bond_wallet,
            &env.aevum_wallet.address(),
            8_000_000, // 8 BND → 8 AEV
        ).await.unwrap();
        
        env.bond_node.submit_transaction(bridge_tx.bond_lock_tx).await.unwrap();
        env.bond_node.mine_blocks(6).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let aevum_mint_tx = env.bridge
            .get_pending_aevum_mint(&bridge_tx.bond_tx_hash)
            .await
            .unwrap()
            .unwrap();
        
        env.aevum_node.submit_transaction(aevum_mint_tx).await.unwrap();
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Now test reverse bridge: AEV → BND
        let initial_aevum_balance = env.aevum_node
            .get_balance(&env.aevum_wallet.address())
            .await
            .unwrap();
        assert_eq!(initial_aevum_balance, 8_000_000);
        
        let reverse_amount = 3_000_000; // 3 AEV → 3 BND
        let reverse_bridge_tx = env.bridge.create_aevum_to_bond_transfer(
            &env.aevum_wallet,
            &env.bond_wallet.address(),
            reverse_amount,
        ).await.unwrap();
        
        // Submit burn transaction to Aevum
        let aevum_burn_hash = env.aevum_node
            .submit_transaction(reverse_bridge_tx.aevum_burn_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Bridge detects burn and creates Bond unlock
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let bond_unlock_tx = env.bridge
            .get_pending_bond_unlock(&aevum_burn_hash)
            .await
            .unwrap()
            .unwrap();
        
        let bond_unlock_hash = env.bond_node
            .submit_transaction(bond_unlock_tx)
            .await
            .unwrap();
        
        env.bond_node.mine_blocks(6).await.unwrap();
        
        // Verify reverse bridge completed
        let final_aevum_balance = env.aevum_node
            .get_balance(&env.aevum_wallet.address())
            .await
            .unwrap();
        
        let final_bond_balance = env.bond_node
            .get_balance(&env.bond_wallet.address())
            .await
            .unwrap();
        
        // AEV balance should be reduced
        assert_eq!(final_aevum_balance, initial_aevum_balance - reverse_amount);
        
        // BND balance should be increased (accounting for original bridge)
        let expected_bond_balance = 2_000_000 + reverse_amount; // 2 BND left + 3 BND unlocked
        assert!(final_bond_balance >= expected_bond_balance - 10_000); // Allow for fees
    }
}
```

### Bridge State Synchronization
```rust
#[cfg(test)]
mod bridge_sync_tests {
    use super::*;

    #[tokio::test]
    async fn bridge_state_recovery() {
        let mut env = BridgeTestEnvironment::new().await;
        env.fund_bond_wallet(5_000_000).await;
        
        // Start bridge transfer
        let bridge_tx = env.bridge.create_bond_to_aevum_transfer(
            &env.bond_wallet,
            &env.aevum_wallet.address(),
            2_000_000,
        ).await.unwrap();
        
        let bond_tx_hash = env.bond_node
            .submit_transaction(bridge_tx.bond_lock_tx)
            .await
            .unwrap();
        
        env.bond_node.mine_blocks(3).await.unwrap(); // Partial confirmation
        
        // Simulate bridge restart/recovery
        let recovered_bridge = InterLedgerBridge::new(
            env.bond_node.rpc_endpoint(),
            env.aevum_node.rpc_endpoint(),
        ).await;
        
        // Bridge should detect pending transfer
        let pending_transfers = recovered_bridge
            .scan_pending_transfers()
            .await
            .unwrap();
        
        assert_eq!(pending_transfers.len(), 1);
        assert_eq!(pending_transfers[0].bond_tx_hash, bond_tx_hash);
        assert_eq!(pending_transfers[0].status, BridgeTransferStatus::PendingBondConfirmation);
        
        // Complete the transfer
        env.bond_node.mine_blocks(3).await.unwrap(); // Complete Bond confirmation
        
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let aevum_mint_tx = recovered_bridge
            .get_pending_aevum_mint(&bond_tx_hash)
            .await
            .unwrap()
            .unwrap();
        
        env.aevum_node.submit_transaction(aevum_mint_tx).await.unwrap();
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify completed transfer
        let final_state = recovered_bridge
            .get_transfer_state(&bond_tx_hash)
            .await
            .unwrap();
        assert_eq!(final_state.status, BridgeTransferStatus::Completed);
    }

    #[tokio::test]
    async fn bridge_fork_handling() {
        let mut env = BridgeTestEnvironment::new().await;
        env.fund_bond_wallet(5_000_000).await;
        
        // Create bridge transfer
        let bridge_tx = env.bridge.create_bond_to_aevum_transfer(
            &env.bond_wallet,
            &env.aevum_wallet.address(),
            2_000_000,
        ).await.unwrap();
        
        let bond_tx_hash = env.bond_node
            .submit_transaction(bridge_tx.bond_lock_tx)
            .await
            .unwrap();
        
        env.bond_node.mine_blocks(6).await.unwrap();
        
        // Process Aevum mint
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let aevum_mint_tx = env.bridge
            .get_pending_aevum_mint(&bond_tx_hash)
            .await
            .unwrap()
            .unwrap();
        
        let aevum_tx_hash = env.aevum_node
            .submit_transaction(aevum_mint_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Simulate Bond chain reorganization (fork)
        let fork_point = env.bond_node.get_current_height().await.unwrap() - 3;
        env.bond_node.create_fork_from_height(fork_point).await.unwrap();
        
        // Mine alternative chain without the bridge transaction
        env.bond_node.mine_blocks(5).await.unwrap();
        
        // Bridge should detect the reorganization
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        let transfer_state = env.bridge
            .get_transfer_state(&bond_tx_hash)
            .await
            .unwrap();
        
        // Transfer should be marked as reorganized/invalid
        assert_eq!(transfer_state.status, BridgeTransferStatus::Reorganized);
        
        // Bridge should initiate rollback on Aevum side
        let rollback_tx = env.bridge
            .get_pending_aevum_rollback(&aevum_tx_hash)
            .await
            .unwrap();
        
        assert!(rollback_tx.is_some());
    }
}
```

### Bridge Security and Edge Cases
```rust
#[cfg(test)]
mod bridge_security_tests {
    use super::*;

    #[tokio::test]
    async fn prevent_double_spending_bridge() {
        let mut env = BridgeTestEnvironment::new().await;
        env.fund_bond_wallet(5_000_000).await;
        
        // Create first bridge transfer
        let bridge_tx1 = env.bridge.create_bond_to_aevum_transfer(
            &env.bond_wallet,
            &env.aevum_wallet.address(),
            2_000_000,
        ).await.unwrap();
        
        // Try to create second bridge transfer with same UTXOs
        let bridge_tx2_result = env.bridge.create_bond_to_aevum_transfer(
            &env.bond_wallet,
            &env.aevum_wallet.address(),
            1_500_000,
        ).await;
        
        // Should fail if trying to double-spend same UTXOs
        if bridge_tx2_result.is_ok() {
            let bond_tx_hash1 = env.bond_node
                .submit_transaction(bridge_tx1.bond_lock_tx)
                .await
                .unwrap();
            
            let bond_tx_hash2_result = env.bond_node
                .submit_transaction(bridge_tx2_result.unwrap().bond_lock_tx)
                .await;
            
            // Second transaction should be rejected by Bond network
            assert!(bond_tx_hash2_result.is_err());
        }
    }

    #[tokio::test]
    async fn bridge_timeout_handling() {
        let mut env = BridgeTestEnvironment::new().await;
        env.fund_bond_wallet(3_000_000).await;
        
        // Create bridge transfer
        let bridge_tx = env.bridge.create_bond_to_aevum_transfer(
            &env.bond_wallet,
            &env.aevum_wallet.address(),
            1_000_000,
        ).await.unwrap();
        
        let bond_tx_hash = env.bond_node
            .submit_transaction(bridge_tx.bond_lock_tx)
            .await
            .unwrap();
        
        // Confirm on Bond but don't process Aevum mint
        env.bond_node.mine_blocks(6).await.unwrap();
        
        // Wait for bridge timeout (simulate network issues)
        let timeout_duration = env.bridge.get_timeout_duration();
        tokio::time::sleep(timeout_duration + tokio::time::Duration::from_secs(10)).await;
        
        // Bridge should mark transfer as timed out
        let transfer_state = env.bridge
            .get_transfer_state(&bond_tx_hash)
            .await
            .unwrap();
        
        assert_eq!(transfer_state.status, BridgeTransferStatus::TimedOut);
        
        // User should be able to recover locked funds
        let recovery_tx = env.bridge
            .create_bond_recovery_transaction(&bond_tx_hash, &env.bond_wallet)
            .await
            .unwrap();
        
        let recovery_hash = env.bond_node
            .submit_transaction(recovery_tx)
            .await
            .unwrap();
        
        env.bond_node.mine_blocks(6).await.unwrap();
        
        // Verify funds recovered
        let final_balance = env.bond_node
            .get_balance(&env.bond_wallet.address())
            .await
            .unwrap();
        
        // Should recover most funds (minus fees)
        assert!(final_balance >= 2_900_000); // Allow for transaction fees
    }

    #[tokio::test]
    async fn bridge_concurrent_transfers() {
        let mut env = BridgeTestEnvironment::new().await;
        env.fund_bond_wallet(20_000_000).await; // Fund for multiple transfers
        
        let mut handles = vec![];
        let transfer_count = 5;
        let amount_per_transfer = 2_000_000;
        
        // Create multiple concurrent bridge transfers
        for i in 0..transfer_count {
            let bridge = env.bridge.clone();
            let bond_wallet = env.bond_wallet.clone();
            let aevum_address = env.aevum_wallet.address();
            
            let handle = tokio::spawn(async move {
                bridge.create_bond_to_aevum_transfer(
                    &bond_wallet,
                    &aevum_address,
                    amount_per_transfer,
                ).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all transfers to be created
        let mut bridge_txs = vec![];
        for handle in handles {
            let bridge_tx = handle.await.unwrap().unwrap();
            bridge_txs.push(bridge_tx);
        }
        
        assert_eq!(bridge_txs.len(), transfer_count);
        
        // Submit all transfers to Bond network
        let mut bond_tx_hashes = vec![];
        for bridge_tx in bridge_txs {
            let hash = env.bond_node
                .submit_transaction(bridge_tx.bond_lock_tx)
                .await
                .unwrap();
            bond_tx_hashes.push(hash);
        }
        
        // Confirm all transfers
        env.bond_node.mine_blocks(6).await.unwrap();
        
        // Process all Aevum mints
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        let mut aevum_tx_hashes = vec![];
        for bond_hash in &bond_tx_hashes {
            let mint_tx = env.bridge
                .get_pending_aevum_mint(bond_hash)
                .await
                .unwrap()
                .unwrap();
            
            let aevum_hash = env.aevum_node
                .submit_transaction(mint_tx)
                .await
                .unwrap();
            
            aevum_tx_hashes.push(aevum_hash);
        }
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify all transfers completed
        for bond_hash in bond_tx_hashes {
            let state = env.bridge
                .get_transfer_state(&bond_hash)
                .await
                .unwrap();
            assert_eq!(state.status, BridgeTransferStatus::Completed);
        }
        
        // Verify total Aevum balance
        let total_aevum_balance = env.aevum_node
            .get_balance(&env.aevum_wallet.address())
            .await
            .unwrap();
        
        let expected_total = transfer_count as u64 * amount_per_transfer;
        assert_eq!(total_aevum_balance, expected_total);
    }
}
```
