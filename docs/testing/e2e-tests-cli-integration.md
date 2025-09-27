# Camada 3: End-to-End Tests - CLI Integration

## 3.1 Testes de Integração CLI (Command Line Interface)

### Complete CLI Integration Tests
```rust
#[cfg(test)]
mod cli_integration_e2e_tests {
    use super::*;
    use std::process::{Command, Stdio};
    use tempfile::TempDir;
    use tokio::time::{timeout, Duration};
    use assert_cmd::prelude::*;
    use predicates::prelude::*;

    struct CLITestEnvironment {
        temp_dir: TempDir,
        bond_binary_path: PathBuf,
        aevum_binary_path: PathBuf,
        wallet_binary_path: PathBuf,
        test_data_dir: PathBuf,
        running_processes: Vec<tokio::process::Child>,
    }

    impl CLITestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_path = temp_dir.path().to_path_buf();
            
            // Build all CLI binaries
            let build_output = Command::new("cargo")
                .args(&["build", "--release", "--bins"])
                .output()
                .expect("Failed to build CLI binaries");
            
            assert!(build_output.status.success(), "CLI build failed: {}", 
                String::from_utf8_lossy(&build_output.stderr));
            
            let target_dir = base_path.join("target/release");
            
            Self {
                temp_dir,
                bond_binary_path: target_dir.join("bond-cli"),
                aevum_binary_path: target_dir.join("aevum-cli"),
                wallet_binary_path: target_dir.join("bond-wallet"),
                test_data_dir: base_path.join("test_data"),
                running_processes: Vec::new(),
            }
        }
        
        async fn setup_test_network(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            // Create test data directory
            std::fs::create_dir_all(&self.test_data_dir)?;
            
            // Start Bond node
            let bond_process = tokio::process::Command::new(&self.bond_binary_path)
                .args(&[
                    "node", "start",
                    "--data-dir", &self.test_data_dir.join("bond").to_string_lossy(),
                    "--network", "testnet",
                    "--mining", "false",
                    "--port", "18333",
                    "--rpc-port", "18332"
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            
            self.running_processes.push(bond_process);
            
            // Start Aevum node
            let aevum_process = tokio::process::Command::new(&self.aevum_binary_path)
                .args(&[
                    "node", "start", 
                    "--data-dir", &self.test_data_dir.join("aevum").to_string_lossy(),
                    "--network", "testnet",
                    "--port", "28333",
                    "--rpc-port", "28332"
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
                
            self.running_processes.push(aevum_process);
            
            // Wait for nodes to be ready
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            Ok(())
        }
        
        async fn cleanup(&mut self) {
            for mut process in self.running_processes.drain(..) {
                let _ = process.kill().await;
                let _ = process.wait().await;
            }
        }
    }

    impl Drop for CLITestEnvironment {
        fn drop(&mut self) {
            // Cleanup any remaining processes
            for process in &mut self.running_processes {
                let _ = process.start_kill();
            }
        }
    }

    #[tokio::test]
    async fn cli_node_lifecycle_operations() {
        let mut env = CLITestEnvironment::new().await;
        
        // Test: Initialize new node
        let mut cmd = Command::new(&env.bond_binary_path);
        cmd.args(&[
            "node", "init",
            "--data-dir", &env.test_data_dir.join("bond_init").to_string_lossy(),
            "--network", "testnet"
        ]);
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Node initialized successfully"))
            .stdout(predicate::str::contains("Genesis block created"));
        
        // Verify initialization created expected files
        assert!(env.test_data_dir.join("bond_init/config.toml").exists());
        assert!(env.test_data_dir.join("bond_init/genesis.json").exists());
        assert!(env.test_data_dir.join("bond_init/chaindata").exists());
        
        // Test: Start node in background
        let node_process = tokio::process::Command::new(&env.bond_binary_path)
            .args(&[
                "node", "start",
                "--data-dir", &env.test_data_dir.join("bond_init").to_string_lossy(),
                "--port", "19333",
                "--rpc-port", "19332",
                "--mining", "false"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start node");
        
        env.running_processes.push(node_process);
        
        // Wait for node to start
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        // Test: Check node status
        let mut status_cmd = Command::new(&env.bond_binary_path);
        status_cmd.args(&[
            "node", "status",
            "--rpc-port", "19332"
        ]);
        
        status_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Node Status: Running"))
            .stdout(predicate::str::contains("Block Height:"))
            .stdout(predicate::str::contains("Peer Count:"));
        
        // Test: Get node info
        let mut info_cmd = Command::new(&env.bond_binary_path);
        info_cmd.args(&[
            "node", "info",
            "--rpc-port", "19332"
        ]);
        
        info_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Network: testnet"))
            .stdout(predicate::str::contains("Version:"))
            .stdout(predicate::str::contains("Node ID:"));
        
        // Test: Stop node gracefully
        let mut stop_cmd = Command::new(&env.bond_binary_path);
        stop_cmd.args(&[
            "node", "stop",
            "--rpc-port", "19332"
        ]);
        
        stop_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Node stopped successfully"));
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn cli_wallet_operations() {
        let mut env = CLITestEnvironment::new().await;
        env.setup_test_network().await.unwrap();
        
        let wallet_dir = env.test_data_dir.join("wallet");
        
        // Test: Create new wallet
        let mut create_cmd = Command::new(&env.wallet_binary_path);
        create_cmd.args(&[
            "create",
            "--wallet-dir", &wallet_dir.to_string_lossy(),
            "--password", "test_password_123",
            "--network", "testnet"
        ]);
        
        create_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Wallet created successfully"))
            .stdout(predicate::str::contains("Address:"))
            .stdout(predicate::str::contains("Mnemonic:"));
        
        // Test: List wallets
        let mut list_cmd = Command::new(&env.wallet_binary_path);
        list_cmd.args(&[
            "list",
            "--wallet-dir", &wallet_dir.to_string_lossy()
        ]);
        
        list_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Available wallets:"))
            .stdout(predicate::str::contains("default"));
        
        // Test: Get wallet balance
        let mut balance_cmd = Command::new(&env.wallet_binary_path);
        balance_cmd.args(&[
            "balance",
            "--wallet-dir", &wallet_dir.to_string_lossy(),
            "--password", "test_password_123",
            "--rpc-port", "18332"
        ]);
        
        balance_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Balance:"))
            .stdout(predicate::str::contains("BND"));
        
        // Test: Generate new address
        let mut address_cmd = Command::new(&env.wallet_binary_path);
        address_cmd.args(&[
            "new-address",
            "--wallet-dir", &wallet_dir.to_string_lossy(),
            "--password", "test_password_123"
        ]);
        
        let address_output = address_cmd.assert()
            .success()
            .stdout(predicate::str::contains("New address:"))
            .get_output()
            .stdout
            .clone();
        
        let new_address = String::from_utf8(address_output).unwrap()
            .lines()
            .find(|line| line.contains("New address:"))
            .unwrap()
            .split(": ")
            .nth(1)
            .unwrap()
            .trim();
        
        // Test: List addresses
        let mut list_addresses_cmd = Command::new(&env.wallet_binary_path);
        list_addresses_cmd.args(&[
            "list-addresses",
            "--wallet-dir", &wallet_dir.to_string_lossy(),
            "--password", "test_password_123"
        ]);
        
        list_addresses_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Wallet addresses:"))
            .stdout(predicate::str::contains(new_address));
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn cli_mining_operations() {
        let mut env = CLITestEnvironment::new().await;
        env.setup_test_network().await.unwrap();
        
        // Test: Start mining
        let mut start_mining_cmd = Command::new(&env.bond_binary_path);
        start_mining_cmd.args(&[
            "mining", "start",
            "--rpc-port", "18332",
            "--threads", "2",
            "--address", "bc1test123456789abcdef"
        ]);
        
        start_mining_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Mining started"))
            .stdout(predicate::str::contains("Threads: 2"));
        
        // Wait for some mining activity
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Test: Check mining status
        let mut mining_status_cmd = Command::new(&env.bond_binary_path);
        mining_status_cmd.args(&[
            "mining", "status",
            "--rpc-port", "18332"
        ]);
        
        mining_status_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Mining Status:"))
            .stdout(predicate::str::contains("Hash Rate:"))
            .stdout(predicate::str::contains("Blocks Mined:"));
        
        // Test: Get mining statistics
        let mut mining_stats_cmd = Command::new(&env.bond_binary_path);
        mining_stats_cmd.args(&[
            "mining", "stats",
            "--rpc-port", "18332"
        ]);
        
        mining_stats_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Mining Statistics"))
            .stdout(predicate::str::contains("Average Block Time:"))
            .stdout(predicate::str::contains("Difficulty:"));
        
        // Test: Stop mining
        let mut stop_mining_cmd = Command::new(&env.bond_binary_path);
        stop_mining_cmd.args(&[
            "mining", "stop",
            "--rpc-port", "18332"
        ]);
        
        stop_mining_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Mining stopped"));
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn cli_transaction_operations() {
        let mut env = CLITestEnvironment::new().await;
        env.setup_test_network().await.unwrap();
        
        let wallet_dir = env.test_data_dir.join("tx_wallet");
        
        // Setup wallet for transactions
        let mut create_wallet_cmd = Command::new(&env.wallet_binary_path);
        create_wallet_cmd.args(&[
            "create",
            "--wallet-dir", &wallet_dir.to_string_lossy(),
            "--password", "tx_test_123",
            "--network", "testnet"
        ]);
        create_wallet_cmd.assert().success();
        
        // Test: Send transaction
        let mut send_cmd = Command::new(&env.wallet_binary_path);
        send_cmd.args(&[
            "send",
            "--wallet-dir", &wallet_dir.to_string_lossy(),
            "--password", "tx_test_123",
            "--to", "bc1test987654321fedcba",
            "--amount", "1.5",
            "--fee", "0.001",
            "--rpc-port", "18332"
        ]);
        
        // Note: This may fail due to insufficient funds in testnet, which is expected
        let send_result = send_cmd.assert();
        
        // Check if transaction was created (even if it fails due to funds)
        send_result.stdout(predicate::str::contains("Transaction").or(
            predicate::str::contains("Insufficient funds")
        ));
        
        // Test: List transactions
        let mut list_tx_cmd = Command::new(&env.wallet_binary_path);
        list_tx_cmd.args(&[
            "transactions",
            "--wallet-dir", &wallet_dir.to_string_lossy(),
            "--password", "tx_test_123",
            "--rpc-port", "18332"
        ]);
        
        list_tx_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Transaction History:"));
        
        // Test: Get transaction by ID (using a dummy ID for format testing)
        let mut get_tx_cmd = Command::new(&env.bond_binary_path);
        get_tx_cmd.args(&[
            "transaction", "get",
            "--tx-id", "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "--rpc-port", "18332"
        ]);
        
        // This should return "Transaction not found" which is expected
        get_tx_cmd.assert()
            .stdout(predicate::str::contains("Transaction not found").or(
                predicate::str::contains("Invalid transaction ID format")
            ));
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn cli_blockchain_queries() {
        let mut env = CLITestEnvironment::new().await;
        env.setup_test_network().await.unwrap();
        
        // Wait for network to stabilize
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        // Test: Get blockchain info
        let mut blockchain_info_cmd = Command::new(&env.bond_binary_path);
        blockchain_info_cmd.args(&[
            "blockchain", "info",
            "--rpc-port", "18332"
        ]);
        
        blockchain_info_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Blockchain Information"))
            .stdout(predicate::str::contains("Current Height:"))
            .stdout(predicate::str::contains("Best Block Hash:"))
            .stdout(predicate::str::contains("Total Supply:"));
        
        // Test: Get block by height
        let mut get_block_cmd = Command::new(&env.bond_binary_path);
        get_block_cmd.args(&[
            "blockchain", "block",
            "--height", "0", // Genesis block
            "--rpc-port", "18332"
        ]);
        
        get_block_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Block Information"))
            .stdout(predicate::str::contains("Height: 0"))
            .stdout(predicate::str::contains("Hash:"))
            .stdout(predicate::str::contains("Transactions:"));
        
        // Test: Get mempool info
        let mut mempool_cmd = Command::new(&env.bond_binary_path);
        mempool_cmd.args(&[
            "blockchain", "mempool",
            "--rpc-port", "18332"
        ]);
        
        mempool_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Mempool Information"))
            .stdout(predicate::str::contains("Transaction Count:"))
            .stdout(predicate::str::contains("Total Size:"));
        
        // Test: Get network peers
        let mut peers_cmd = Command::new(&env.bond_binary_path);
        peers_cmd.args(&[
            "network", "peers",
            "--rpc-port", "18332"
        ]);
        
        peers_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Connected Peers:"));
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn cli_configuration_management() {
        let mut env = CLITestEnvironment::new().await;
        let config_dir = env.test_data_dir.join("config_test");
        
        // Test: Generate default configuration
        let mut config_init_cmd = Command::new(&env.bond_binary_path);
        config_init_cmd.args(&[
            "config", "init",
            "--config-dir", &config_dir.to_string_lossy(),
            "--network", "testnet"
        ]);
        
        config_init_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Configuration initialized"))
            .stdout(predicate::str::contains("config.toml"));
        
        // Verify config file was created
        assert!(config_dir.join("config.toml").exists());
        
        // Test: Show current configuration
        let mut config_show_cmd = Command::new(&env.bond_binary_path);
        config_show_cmd.args(&[
            "config", "show",
            "--config-dir", &config_dir.to_string_lossy()
        ]);
        
        config_show_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Current Configuration:"))
            .stdout(predicate::str::contains("network = \"testnet\""))
            .stdout(predicate::str::contains("[network]"))
            .stdout(predicate::str::contains("[mining]"));
        
        // Test: Update configuration value
        let mut config_set_cmd = Command::new(&env.bond_binary_path);
        config_set_cmd.args(&[
            "config", "set",
            "--config-dir", &config_dir.to_string_lossy(),
            "--key", "mining.threads",
            "--value", "8"
        ]);
        
        config_set_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Configuration updated"))
            .stdout(predicate::str::contains("mining.threads = 8"));
        
        // Test: Validate configuration
        let mut config_validate_cmd = Command::new(&env.bond_binary_path);
        config_validate_cmd.args(&[
            "config", "validate",
            "--config-dir", &config_dir.to_string_lossy()
        ]);
        
        config_validate_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Configuration is valid"));
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn cli_cross_chain_bridge_operations() {
        let mut env = CLITestEnvironment::new().await;
        env.setup_test_network().await.unwrap();
        
        // Wait for both networks to be ready
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Test: Check bridge status
        let mut bridge_status_cmd = Command::new(&env.bond_binary_path);
        bridge_status_cmd.args(&[
            "bridge", "status",
            "--bond-rpc", "18332",
            "--aevum-rpc", "28332"
        ]);
        
        bridge_status_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Bridge Status"))
            .stdout(predicate::str::contains("Bond Network:"))
            .stdout(predicate::str::contains("Aevum Network:"));
        
        // Test: Initiate bridge transfer (will likely fail due to no funds, but tests CLI)
        let mut bridge_transfer_cmd = Command::new(&env.bond_binary_path);
        bridge_transfer_cmd.args(&[
            "bridge", "transfer",
            "--from", "bond",
            "--to", "aevum", 
            "--amount", "1.0",
            "--from-address", "bc1test123456789abcdef",
            "--to-address", "aev1test123456789abcdef",
            "--bond-rpc", "18332",
            "--aevum-rpc", "28332"
        ]);
        
        // Expected to fail due to insufficient funds, but should show proper error handling
        bridge_transfer_cmd.assert()
            .stdout(predicate::str::contains("Bridge transfer").or(
                predicate::str::contains("Insufficient funds")
            ).or(
                predicate::str::contains("Invalid address")
            ));
        
        // Test: List bridge transactions
        let mut bridge_history_cmd = Command::new(&env.bond_binary_path);
        bridge_history_cmd.args(&[
            "bridge", "history",
            "--bond-rpc", "18332",
            "--aevum-rpc", "28332"
        ]);
        
        bridge_history_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Bridge Transaction History"));
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn cli_help_and_version_commands() {
        let env = CLITestEnvironment::new().await;
        
        // Test: Bond CLI help
        let mut bond_help_cmd = Command::new(&env.bond_binary_path);
        bond_help_cmd.arg("--help");
        
        bond_help_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Bond Protocol CLI"))
            .stdout(predicate::str::contains("USAGE:"))
            .stdout(predicate::str::contains("SUBCOMMANDS:"))
            .stdout(predicate::str::contains("node"))
            .stdout(predicate::str::contains("mining"))
            .stdout(predicate::str::contains("blockchain"));
        
        // Test: Bond CLI version
        let mut bond_version_cmd = Command::new(&env.bond_binary_path);
        bond_version_cmd.arg("--version");
        
        bond_version_cmd.assert()
            .success()
            .stdout(predicate::str::contains("bond-cli"))
            .stdout(predicate::str::regex(r"\d+\.\d+\.\d+"));
        
        // Test: Wallet CLI help
        let mut wallet_help_cmd = Command::new(&env.wallet_binary_path);
        wallet_help_cmd.arg("--help");
        
        wallet_help_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Bond Wallet CLI"))
            .stdout(predicate::str::contains("USAGE:"))
            .stdout(predicate::str::contains("SUBCOMMANDS:"))
            .stdout(predicate::str::contains("create"))
            .stdout(predicate::str::contains("balance"))
            .stdout(predicate::str::contains("send"));
        
        // Test: Aevum CLI help  
        let mut aevum_help_cmd = Command::new(&env.aevum_binary_path);
        aevum_help_cmd.arg("--help");
        
        aevum_help_cmd.assert()
            .success()
            .stdout(predicate::str::contains("Aevum Protocol CLI"))
            .stdout(predicate::str::contains("USAGE:"))
            .stdout(predicate::str::contains("SUBCOMMANDS:"));
    }

    #[tokio::test]
    async fn cli_error_handling_and_validation() {
        let env = CLITestEnvironment::new().await;
        
        // Test: Invalid command
        let mut invalid_cmd = Command::new(&env.bond_binary_path);
        invalid_cmd.arg("invalid-command");
        
        invalid_cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Found argument 'invalid-command'"));
        
        // Test: Missing required arguments
        let mut missing_args_cmd = Command::new(&env.bond_binary_path);
        missing_args_cmd.args(&["node", "start"]);
        
        missing_args_cmd.assert()
            .failure()
            .stderr(predicate::str::contains("required arguments"));
        
        // Test: Invalid network parameter
        let mut invalid_network_cmd = Command::new(&env.bond_binary_path);
        invalid_network_cmd.args(&[
            "node", "init",
            "--data-dir", "/tmp/test",
            "--network", "invalid-network"
        ]);
        
        invalid_network_cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Invalid network"));
        
        // Test: Invalid port number
        let mut invalid_port_cmd = Command::new(&env.bond_binary_path);
        invalid_port_cmd.args(&[
            "node", "status",
            "--rpc-port", "99999"
        ]);
        
        invalid_port_cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Connection refused").or(
                predicate::str::contains("Invalid port")
            ));
        
        // Test: Wallet operations without password
        let mut no_password_cmd = Command::new(&env.wallet_binary_path);
        no_password_cmd.args(&[
            "balance",
            "--wallet-dir", "/tmp/nonexistent"
        ]);
        
        no_password_cmd.assert()
            .failure()
            .stderr(predicate::str::contains("password").or(
                predicate::str::contains("required arguments")
            ));
    }
}
```

### CLI Test Execution Strategy

```bash
#!/bin/bash
# scripts/run-cli-e2e-tests.sh

set -e

echo "🚀 Starting CLI End-to-End Tests"

# Build all CLI binaries
echo "📦 Building CLI binaries..."
cargo build --release --bins

# Create test environment
export TEST_DATA_DIR=$(mktemp -d)
export RUST_LOG=debug

echo "🧪 Running CLI integration tests..."

# Run individual test suites
cargo test cli_node_lifecycle_operations --test cli_e2e -- --nocapture
cargo test cli_wallet_operations --test cli_e2e -- --nocapture  
cargo test cli_mining_operations --test cli_e2e -- --nocapture
cargo test cli_transaction_operations --test cli_e2e -- --nocapture
cargo test cli_blockchain_queries --test cli_e2e -- --nocapture
cargo test cli_configuration_management --test cli_e2e -- --nocapture
cargo test cli_cross_chain_bridge_operations --test cli_e2e -- --nocapture
cargo test cli_help_and_version_commands --test cli_e2e -- --nocapture
cargo test cli_error_handling_and_validation --test cli_e2e -- --nocapture

echo "✅ All CLI E2E tests completed successfully!"

# Cleanup
rm -rf "$TEST_DATA_DIR"
```
