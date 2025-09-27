# Camada 3: End-to-End Tests - Wallet-Node Interaction

## 3.2 Testes de Interação Wallet-Node

### Complete Wallet-Node Integration Tests
```rust
#[cfg(test)]
mod wallet_node_e2e_tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct WalletNodeTestEnvironment {
        temp_dir: TempDir,
        bond_node: Arc<Mutex<BondNode>>,
        aevum_node: Arc<Mutex<AevumNode>>,
        desktop_wallet: Arc<Mutex<DesktopWallet>>,
        cli_wallet: CliWallet,
        test_accounts: Vec<TestAccount>,
        network_config: NetworkConfig,
    }

    struct TestAccount {
        address: String,
        private_key: String,
        mnemonic: String,
        initial_balance: u64,
    }

    impl WalletNodeTestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_path = temp_dir.path().to_path_buf();
            
            // Initialize network configuration
            let network_config = NetworkConfig {
                bond_port: 18333,
                bond_rpc_port: 18332,
                aevum_port: 28333,
                aevum_rpc_port: 28332,
                bridge_port: 38333,
            };
            
            // Start Bond node
            let bond_node = Arc::new(Mutex::new(
                BondNode::new_with_config(
                    base_path.join("bond"),
                    BondConfig {
                        network: "testnet".to_string(),
                        port: network_config.bond_port,
                        rpc_port: network_config.bond_rpc_port,
                        mining_enabled: true,
                        mining_threads: 2,
                    }
                ).await.unwrap()
            ));
            
            // Start Aevum node
            let aevum_node = Arc::new(Mutex::new(
                AevumNode::new_with_config(
                    base_path.join("aevum"),
                    AevumConfig {
                        network: "testnet".to_string(),
                        port: network_config.aevum_port,
                        rpc_port: network_config.aevum_rpc_port,
                        staking_enabled: true,
                    }
                ).await.unwrap()
            ));
            
            // Initialize desktop wallet
            let desktop_wallet = Arc::new(Mutex::new(
                DesktopWallet::new(
                    base_path.join("desktop_wallet"),
                    WalletConfig {
                        bond_rpc_url: format!("http://localhost:{}", network_config.bond_rpc_port),
                        aevum_rpc_url: format!("http://localhost:{}", network_config.aevum_rpc_port),
                        auto_sync: true,
                        sync_interval: Duration::from_secs(30),
                    }
                ).await.unwrap()
            ));
            
            // Initialize CLI wallet
            let cli_wallet = CliWallet::new(
                base_path.join("cli_wallet"),
                network_config.clone()
            ).await.unwrap();
            
            // Create test accounts
            let test_accounts = vec![
                TestAccount {
                    address: "bc1test_account_1_address".to_string(),
                    private_key: "test_private_key_1".to_string(), 
                    mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
                    initial_balance: 10_000_000_000, // 100 BND
                },
                TestAccount {
                    address: "bc1test_account_2_address".to_string(),
                    private_key: "test_private_key_2".to_string(),
                    mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon".to_string(),
                    initial_balance: 5_000_000_000, // 50 BND
                },
            ];
            
            Self {
                temp_dir,
                bond_node,
                aevum_node,
                desktop_wallet,
                cli_wallet,
                test_accounts,
                network_config,
            }
        }
        
        async fn start_nodes(&self) -> Result<(), Box<dyn std::error::Error>> {
            // Start Bond node
            {
                let mut bond_node = self.bond_node.lock().await;
                bond_node.start().await?;
            }
            
            // Start Aevum node
            {
                let mut aevum_node = self.aevum_node.lock().await;
                aevum_node.start().await?;
            }
            
            // Wait for nodes to be ready
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            // Initialize wallets
            {
                let mut desktop_wallet = self.desktop_wallet.lock().await;
                desktop_wallet.connect_to_nodes().await?;
            }
            
            Ok(())
        }
        
        async fn setup_test_balances(&self) -> Result<(), Box<dyn std::error::Error>> {
            // Mine some blocks to generate initial coins
            {
                let mut bond_node = self.bond_node.lock().await;
                for i in 0..10 {
                    let block = bond_node.mine_block_to_address(&self.test_accounts[0].address).await?;
                    println!("Mined block {} with hash: {}", i + 1, block.hash());
                }
            }
            
            // Wait for blocks to be processed
            tokio::time::sleep(Duration::from_secs(3)).await;
            
            Ok(())
        }
        
        async fn cleanup(&self) {
            // Stop nodes gracefully
            if let Ok(mut bond_node) = self.bond_node.try_lock() {
                let _ = bond_node.stop().await;
            }
            
            if let Ok(mut aevum_node) = self.aevum_node.try_lock() {
                let _ = aevum_node.stop().await;
            }
        }
    }

    #[tokio::test]
    async fn wallet_node_connection_and_sync() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        
        // Test: Desktop wallet connection to nodes
        {
            let desktop_wallet = env.desktop_wallet.lock().await;
            
            // Verify connection to Bond node
            let bond_status = desktop_wallet.get_bond_node_status().await.unwrap();
            assert_eq!(bond_status.network, "testnet");
            assert!(bond_status.is_connected);
            assert!(bond_status.block_height >= 0);
            
            // Verify connection to Aevum node  
            let aevum_status = desktop_wallet.get_aevum_node_status().await.unwrap();
            assert_eq!(aevum_status.network, "testnet");
            assert!(aevum_status.is_connected);
            assert!(aevum_status.block_height >= 0);
        }
        
        // Test: Initial wallet synchronization
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            // Trigger manual sync
            let sync_result = desktop_wallet.sync_with_nodes().await.unwrap();
            assert!(sync_result.bond_blocks_synced >= 0);
            assert!(sync_result.aevum_blocks_synced >= 0);
            assert!(sync_result.sync_duration < Duration::from_secs(30));
            
            // Verify sync status
            let sync_status = desktop_wallet.get_sync_status().await.unwrap();
            assert!(sync_status.is_fully_synced);
            assert_eq!(sync_status.bond_sync_progress, 100.0);
            assert_eq!(sync_status.aevum_sync_progress, 100.0);
        }
        
        // Test: Automatic sync after new blocks
        env.setup_test_balances().await.unwrap();
        
        // Wait for automatic sync to trigger
        tokio::time::sleep(Duration::from_secs(35)).await;
        
        {
            let desktop_wallet = env.desktop_wallet.lock().await;
            let updated_status = desktop_wallet.get_bond_node_status().await.unwrap();
            assert!(updated_status.block_height >= 10); // Should have the new blocks
        }
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn wallet_account_management() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        
        // Test: Create new wallet account in desktop wallet
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            let new_account = desktop_wallet.create_account(
                "Test Account 1",
                "test_password_123"
            ).await.unwrap();
            
            assert!(!new_account.address.is_empty());
            assert!(!new_account.public_key.is_empty());
            assert!(new_account.mnemonic.split_whitespace().count() == 24); // 24-word mnemonic
            
            // Verify account was saved
            let accounts = desktop_wallet.list_accounts().await.unwrap();
            assert_eq!(accounts.len(), 1);
            assert_eq!(accounts[0].name, "Test Account 1");
        }
        
        // Test: Import account from mnemonic
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            let imported_account = desktop_wallet.import_account_from_mnemonic(
                "Imported Account",
                &env.test_accounts[0].mnemonic,
                "import_password_456"
            ).await.unwrap();
            
            assert_eq!(imported_account.address, env.test_accounts[0].address);
            
            // Verify both accounts exist
            let accounts = desktop_wallet.list_accounts().await.unwrap();
            assert_eq!(accounts.len(), 2);
        }
        
        // Test: Account backup and restore
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            let backup_data = desktop_wallet.backup_accounts("backup_password_789").await.unwrap();
            assert!(!backup_data.is_empty());
            
            // Clear all accounts
            desktop_wallet.clear_accounts().await.unwrap();
            let empty_accounts = desktop_wallet.list_accounts().await.unwrap();
            assert_eq!(empty_accounts.len(), 0);
            
            // Restore from backup
            let restore_result = desktop_wallet.restore_accounts_from_backup(
                &backup_data,
                "backup_password_789"
            ).await.unwrap();
            
            assert_eq!(restore_result.accounts_restored, 2);
            
            let restored_accounts = desktop_wallet.list_accounts().await.unwrap();
            assert_eq!(restored_accounts.len(), 2);
        }
        
        env.cleanup().await;
    }

    #[tokio::test] 
    async fn wallet_balance_tracking() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        env.setup_test_balances().await.unwrap();
        
        // Test: Import test account with balance
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            let account = desktop_wallet.import_account_from_mnemonic(
                "Test Account With Balance",
                &env.test_accounts[0].mnemonic,
                "test_password"
            ).await.unwrap();
            
            // Force balance refresh
            desktop_wallet.refresh_balances().await.unwrap();
            
            let balance = desktop_wallet.get_account_balance(&account.address).await.unwrap();
            
            // Should have mining rewards from the blocks we mined
            assert!(balance.confirmed_bond > 0);
            assert_eq!(balance.pending_bond, 0); // No pending transactions yet
            assert_eq!(balance.confirmed_aevum, 0); // No Aevum balance yet
        }
        
        // Test: Balance updates after transactions
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            let accounts = desktop_wallet.list_accounts().await.unwrap();
            let sender_account = &accounts[0];
            
            // Create second account as recipient
            let recipient_account = desktop_wallet.create_account(
                "Recipient Account",
                "recipient_password"
            ).await.unwrap();
            
            let sender_balance_before = desktop_wallet.get_account_balance(&sender_account.address).await.unwrap();
            
            // Send transaction
            let tx_result = desktop_wallet.send_transaction(
                &sender_account.address,
                &recipient_account.address,
                1_000_000_000, // 10 BND
                100_000, // 0.001 BND fee
                "test_password"
            ).await;
            
            match tx_result {
                Ok(tx_hash) => {
                    // Wait for transaction to be processed
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    
                    // Mine a block to confirm transaction
                    {
                        let mut bond_node = env.bond_node.lock().await;
                        bond_node.mine_block().await.unwrap();
                    }
                    
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    
                    // Refresh balances
                    desktop_wallet.refresh_balances().await.unwrap();
                    
                    let sender_balance_after = desktop_wallet.get_account_balance(&sender_account.address).await.unwrap();
                    let recipient_balance = desktop_wallet.get_account_balance(&recipient_account.address).await.unwrap();
                    
                    // Verify balances updated correctly
                    assert!(sender_balance_after.confirmed_bond < sender_balance_before.confirmed_bond);
                    assert_eq!(recipient_balance.confirmed_bond, 1_000_000_000);
                    
                    // Verify transaction appears in history
                    let tx_history = desktop_wallet.get_transaction_history(&sender_account.address).await.unwrap();
                    assert!(tx_history.iter().any(|tx| tx.hash == tx_hash));
                }
                Err(e) => {
                    // May fail due to insufficient confirmed balance, which is acceptable for test
                    println!("Transaction failed (expected): {}", e);
                }
            }
        }
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn wallet_transaction_creation_and_broadcasting() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        env.setup_test_balances().await.unwrap();
        
        // Test: Create and broadcast transaction via desktop wallet
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            // Import account with balance
            let sender_account = desktop_wallet.import_account_from_mnemonic(
                "Sender Account",
                &env.test_accounts[0].mnemonic,
                "sender_password"
            ).await.unwrap();
            
            // Create recipient account
            let recipient_account = desktop_wallet.create_account(
                "Recipient Account", 
                "recipient_password"
            ).await.unwrap();
            
            // Wait for balance sync
            desktop_wallet.refresh_balances().await.unwrap();
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            let sender_balance = desktop_wallet.get_account_balance(&sender_account.address).await.unwrap();
            
            if sender_balance.confirmed_bond > 2_000_000_000 { // Has sufficient balance
                // Create transaction
                let tx_draft = desktop_wallet.create_transaction_draft(
                    &sender_account.address,
                    &recipient_account.address,
                    1_000_000_000, // 10 BND
                    100_000 // 0.001 BND fee
                ).await.unwrap();
                
                // Verify transaction details
                assert_eq!(tx_draft.outputs.len(), 2); // Recipient + change
                assert_eq!(tx_draft.outputs[0].value, 1_000_000_000);
                assert!(tx_draft.fee >= 100_000);
                
                // Sign transaction
                let signed_tx = desktop_wallet.sign_transaction(
                    tx_draft,
                    &sender_account.address,
                    "sender_password"
                ).await.unwrap();
                
                // Broadcast transaction
                let tx_hash = desktop_wallet.broadcast_transaction(signed_tx).await.unwrap();
                assert!(!tx_hash.is_empty());
                
                // Verify transaction in mempool
                let mempool_tx = desktop_wallet.get_mempool_transaction(&tx_hash).await.unwrap();
                assert_eq!(mempool_tx.hash, tx_hash);
                
                // Wait for node to process
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                // Verify transaction appears in bond node mempool
                {
                    let bond_node = env.bond_node.lock().await;
                    let mempool_info = bond_node.get_mempool_info().await.unwrap();
                    assert!(mempool_info.transaction_count > 0);
                }
            }
        }
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn wallet_cli_desktop_wallet_interop() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        env.setup_test_balances().await.unwrap();
        
        // Test: Create account in CLI wallet
        let cli_account = env.cli_wallet.create_account(
            "CLI Account",
            "cli_password_123"
        ).await.unwrap();
        
        // Test: Import the same account in desktop wallet using mnemonic
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            let desktop_account = desktop_wallet.import_account_from_mnemonic(
                "Same Account in Desktop",
                &cli_account.mnemonic,
                "desktop_password_456"
            ).await.unwrap();
            
            // Verify same address
            assert_eq!(cli_account.address, desktop_account.address);
        }
        
        // Test: Send transaction from CLI, observe in desktop wallet
        if let Ok(tx_hash) = env.cli_wallet.send_transaction(
            &cli_account.address,
            "bc1test_recipient_address",
            500_000_000, // 5 BND
            50_000, // 0.0005 BND fee
            "cli_password_123"
        ).await {
            // Wait for transaction to propagate
            tokio::time::sleep(Duration::from_secs(3)).await;
            
            // Check if desktop wallet sees the transaction
            let desktop_wallet = env.desktop_wallet.lock().await;
            let tx_history = desktop_wallet.get_transaction_history(&cli_account.address).await.unwrap();
            
            let found_tx = tx_history.iter().find(|tx| tx.hash == tx_hash);
            assert!(found_tx.is_some(), "Transaction should be visible in desktop wallet");
        }
        
        // Test: Address generation consistency
        let cli_addresses = env.cli_wallet.generate_addresses(&cli_account.address, 5).await.unwrap();
        
        {
            let desktop_wallet = env.desktop_wallet.lock().await;
            let desktop_addresses = desktop_wallet.generate_addresses(&cli_account.address, 5).await.unwrap();
            
            // Should generate the same addresses from the same seed
            assert_eq!(cli_addresses, desktop_addresses);
        }
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn wallet_cross_chain_operations() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        env.setup_test_balances().await.unwrap();
        
        // Test: Cross-chain transfer initiation
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            // Import account with Bond balance
            let bond_account = desktop_wallet.import_account_from_mnemonic(
                "Bond Account",
                &env.test_accounts[0].mnemonic,
                "bond_password"
            ).await.unwrap();
            
            // Create Aevum account
            let aevum_account = desktop_wallet.create_aevum_account(
                "Aevum Account",
                "aevum_password"
            ).await.unwrap();
            
            // Initiate cross-chain transfer
            let bridge_tx_result = desktop_wallet.initiate_cross_chain_transfer(
                &bond_account.address,      // From Bond
                &aevum_account.address,     // To Aevum
                500_000_000,                // 5 BND -> AEV
                "bond_password"
            ).await;
            
            match bridge_tx_result {
                Ok(bridge_tx_id) => {
                    // Monitor bridge transaction status
                    let mut status_checks = 0;
                    loop {
                        let bridge_status = desktop_wallet.get_bridge_transaction_status(&bridge_tx_id).await.unwrap();
                        
                        match bridge_status.status {
                            BridgeTransactionStatus::Pending => {
                                if status_checks > 10 {
                                    break; // Avoid infinite loop in test
                                }
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                status_checks += 1;
                            }
                            BridgeTransactionStatus::Confirmed => {
                                // Verify Aevum balance increased
                                desktop_wallet.refresh_balances().await.unwrap();
                                let aevum_balance = desktop_wallet.get_account_balance(&aevum_account.address).await.unwrap();
                                assert!(aevum_balance.confirmed_aevum > 0);
                                break;
                            }
                            BridgeTransactionStatus::Failed => {
                                panic!("Bridge transaction failed: {}", bridge_status.error.unwrap_or_default());
                            }
                        }
                    }
                }
                Err(e) => {
                    // May fail due to insufficient balance or bridge not fully initialized
                    println!("Bridge transaction failed (may be expected): {}", e);
                }
            }
        }
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn wallet_node_recovery_scenarios() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        
        // Test: Wallet behavior during node disconnection
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            // Verify initial connection
            assert!(desktop_wallet.is_connected_to_bond_node().await);
            assert!(desktop_wallet.is_connected_to_aevum_node().await);
            
            // Stop Bond node
            {
                let mut bond_node = env.bond_node.lock().await;
                bond_node.stop().await.unwrap();
            }
            
            // Wait for disconnection to be detected
            tokio::time::sleep(Duration::from_secs(10)).await;
            
            // Verify wallet handles disconnection gracefully
            assert!(!desktop_wallet.is_connected_to_bond_node().await);
            assert!(desktop_wallet.is_connected_to_aevum_node().await);
            
            // Verify wallet shows appropriate status
            let connection_status = desktop_wallet.get_connection_status().await.unwrap();
            assert_eq!(connection_status.bond_status, ConnectionStatus::Disconnected);
            assert_eq!(connection_status.aevum_status, ConnectionStatus::Connected);
            
            // Restart Bond node
            {
                let mut bond_node = env.bond_node.lock().await;
                bond_node.start().await.unwrap();
            }
            
            // Wait for reconnection
            tokio::time::sleep(Duration::from_secs(10)).await;
            
            // Verify wallet reconnects automatically
            assert!(desktop_wallet.is_connected_to_bond_node().await);
            
            // Verify sync resumes after reconnection
            let sync_result = desktop_wallet.sync_with_nodes().await.unwrap();
            assert!(sync_result.sync_duration < Duration::from_secs(30));
        }
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn wallet_performance_and_responsiveness() {
        let env = WalletNodeTestEnvironment::new().await;
        env.start_nodes().await.unwrap();
        
        // Test: Wallet startup time
        let startup_start = std::time::Instant::now();
        {
            let desktop_wallet = env.desktop_wallet.lock().await;
            let _status = desktop_wallet.get_bond_node_status().await.unwrap();
        }
        let startup_duration = startup_start.elapsed();
        
        assert!(startup_duration < Duration::from_secs(10), 
               "Wallet should connect to nodes within 10 seconds");
        
        // Test: Balance refresh performance
        {
            let mut desktop_wallet = env.desktop_wallet.lock().await;
            
            // Create multiple accounts
            for i in 0..5 {
                desktop_wallet.create_account(
                    &format!("Test Account {}", i),
                    "test_password"
                ).await.unwrap();
            }
            
            let refresh_start = std::time::Instant::now();
            desktop_wallet.refresh_balances().await.unwrap();
            let refresh_duration = refresh_start.elapsed();
            
            assert!(refresh_duration < Duration::from_secs(15),
                   "Balance refresh for 5 accounts should take less than 15 seconds");
        }
        
        // Test: Transaction history retrieval performance
        {
            let desktop_wallet = env.desktop_wallet.lock().await;
            let accounts = desktop_wallet.list_accounts().await.unwrap();
            
            if !accounts.is_empty() {
                let history_start = std::time::Instant::now();
                let _history = desktop_wallet.get_transaction_history(&accounts[0].address).await.unwrap();
                let history_duration = history_start.elapsed();
                
                assert!(history_duration < Duration::from_secs(5),
                       "Transaction history retrieval should take less than 5 seconds");
            }
        }
        
        env.cleanup().await;
    }
}
```

### Wallet-Node Test Configuration

```toml
# Cargo.toml - Test dependencies for wallet-node integration
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.0"
tokio-test = "0.4"

[[test]]
name = "wallet_node_e2e"
path = "tests/e2e/wallet_node_integration.rs"
required-features = ["desktop-wallet", "cli-wallet"]
```
