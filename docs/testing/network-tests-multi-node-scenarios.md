# Camada 4: Network Tests - Multi-Node Scenarios

## 4.1 Testes de Cenários Multi-Nó

### Complete Multi-Node Network Tests
```rust
#[cfg(test)]
mod multi_node_network_tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::time::{timeout, Duration, Instant};
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

    struct MultiNodeTestEnvironment {
        temp_dir: TempDir,
        bond_network: BondNetworkCluster,
        aevum_network: AevumNetworkCluster,
        bridge_coordinators: Vec<BridgeCoordinator>,
        network_partitioner: NetworkPartitioner,
        metrics_collector: NetworkMetricsCollector,
        test_config: NetworkTestConfig,
    }

    struct BondNetworkCluster {
        nodes: Vec<BondNode>,
        miners: Vec<MinerNode>,
        full_nodes: Vec<FullNode>,
        light_nodes: Vec<LightNode>,
        network_topology: NetworkTopology,
    }

    struct AevumNetworkCluster {
        validators: Vec<ValidatorNode>,
        full_nodes: Vec<FullNode>,
        light_nodes: Vec<LightNode>,
        staking_pools: Vec<StakingPool>,
        network_topology: NetworkTopology,
    }

    struct NetworkTestConfig {
        bond_nodes_count: usize,
        aevum_nodes_count: usize,
        miners_count: usize,
        validators_count: usize,
        network_latency_ms: u64,
        packet_loss_rate: f64,
        bandwidth_limit_mbps: u64,
        partition_scenarios: Vec<PartitionScenario>,
    }

    #[derive(Debug, Clone)]
    struct PartitionScenario {
        name: String,
        duration: Duration,
        partitioned_nodes: Vec<String>,
        recovery_strategy: RecoveryStrategy,
    }

    #[derive(Debug, Clone)]
    enum RecoveryStrategy {
        Automatic,
        Manual,
        GradualReconnection { interval: Duration },
    }

    impl MultiNodeTestEnvironment {
        async fn new(config: NetworkTestConfig) -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_path = temp_dir.path().to_path_buf();
            
            // Initialize Bond network cluster
            let bond_network = BondNetworkCluster::new(
                base_path.join("bond_cluster"),
                config.bond_nodes_count,
                config.miners_count
            ).await;
            
            // Initialize Aevum network cluster
            let aevum_network = AevumNetworkCluster::new(
                base_path.join("aevum_cluster"),
                config.aevum_nodes_count,
                config.validators_count
            ).await;
            
            // Initialize bridge coordinators
            let bridge_coordinators = Self::setup_bridge_coordinators(
                &bond_network,
                &aevum_network,
                base_path.join("bridges")
            ).await;
            
            let network_partitioner = NetworkPartitioner::new();
            let metrics_collector = NetworkMetricsCollector::new();
            
            Self {
                temp_dir,
                bond_network,
                aevum_network,
                bridge_coordinators,
                network_partitioner,
                metrics_collector,
                test_config: config,
            }
        }
        
        async fn start_full_network(&mut self) -> Result<(), NetworkError> {
            println!("🌐 Starting complete multi-node network...");
            
            // Start Bond network
            self.bond_network.start_all_nodes().await?;
            println!("  ✅ Bond network started ({} nodes)", self.bond_network.nodes.len());
            
            // Start Aevum network
            self.aevum_network.start_all_nodes().await?;
            println!("  ✅ Aevum network started ({} nodes)", self.aevum_network.validators.len());
            
            // Start bridge coordinators
            for (i, bridge) in self.bridge_coordinators.iter_mut().enumerate() {
                bridge.start().await?;
                println!("  ✅ Bridge coordinator {} started", i + 1);
            }
            
            // Wait for network stabilization
            println!("  ⏳ Waiting for network stabilization...");
            tokio::time::sleep(Duration::from_secs(30)).await;
            
            // Verify all nodes are connected
            self.verify_network_connectivity().await?;
            println!("  ✅ Network connectivity verified");
            
            Ok(())
        }
        
        async fn verify_network_connectivity(&self) -> Result<(), NetworkError> {
            // Verify Bond network connectivity
            for (i, node) in self.bond_network.nodes.iter().enumerate() {
                let peer_count = node.get_peer_count().await?;
                if peer_count < (self.bond_network.nodes.len() - 1) / 2 {
                    return Err(NetworkError::InsufficientPeers {
                        node_id: i,
                        actual_peers: peer_count,
                        expected_min: (self.bond_network.nodes.len() - 1) / 2,
                    });
                }
            }
            
            // Verify Aevum network connectivity
            for (i, validator) in self.aevum_network.validators.iter().enumerate() {
                let peer_count = validator.get_peer_count().await?;
                if peer_count < (self.aevum_network.validators.len() - 1) / 2 {
                    return Err(NetworkError::InsufficientPeers {
                        node_id: i,
                        actual_peers: peer_count,
                        expected_min: (self.aevum_network.validators.len() - 1) / 2,
                    });
                }
            }
            
            Ok(())
        }
        
        async fn simulate_network_conditions(&mut self, conditions: NetworkConditions) -> Result<(), NetworkError> {
            self.network_partitioner.apply_conditions(conditions).await
        }
        
        async fn cleanup(&mut self) {
            // Stop all nodes gracefully
            let _ = self.bond_network.stop_all_nodes().await;
            let _ = self.aevum_network.stop_all_nodes().await;
            
            for bridge in &mut self.bridge_coordinators {
                let _ = bridge.stop().await;
            }
        }
        
        async fn setup_bridge_coordinators(
            bond_network: &BondNetworkCluster,
            aevum_network: &AevumNetworkCluster,
            base_path: PathBuf
        ) -> Vec<BridgeCoordinator> {
            let mut coordinators = Vec::new();
            
            // Create multiple bridge coordinators for redundancy
            for i in 0..3 {
                let coordinator = BridgeCoordinator::new(
                    base_path.join(format!("bridge_{}", i)),
                    bond_network.nodes[i % bond_network.nodes.len()].clone(),
                    aevum_network.validators[i % aevum_network.validators.len()].clone(),
                ).await.unwrap();
                
                coordinators.push(coordinator);
            }
            
            coordinators
        }
    }

    #[tokio::test]
    async fn large_scale_network_synchronization() {
        let config = NetworkTestConfig {
            bond_nodes_count: 10,
            aevum_nodes_count: 8,
            miners_count: 4,
            validators_count: 6,
            network_latency_ms: 100,
            packet_loss_rate: 0.01,
            bandwidth_limit_mbps: 100,
            partition_scenarios: vec![],
        };
        
        let mut env = MultiNodeTestEnvironment::new(config).await;
        env.start_full_network().await.unwrap();
        
        // Test: Generate load across network
        println!("📊 Starting large-scale network synchronization test");
        
        // Generate transactions across multiple nodes
        let mut transaction_handles = vec![];
        
        for i in 0..env.bond_network.nodes.len() {
            let node = env.bond_network.nodes[i].clone();
            let handle = tokio::spawn(async move {
                let mut successful_txs = 0;
                
                for j in 0..50 {
                    let tx = create_test_transaction(
                        format!("tx_{}_{}", i, j),
                        1_000_000 + j * 1000, // Varying amounts
                        10_000 // Standard fee
                    ).await;
                    
                    match node.submit_transaction(tx).await {
                        Ok(_) => {
                            successful_txs += 1;
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        Err(e) => {
                            println!("Transaction failed on node {}: {}", i, e);
                        }
                    }
                }
                
                successful_txs
            });
            
            transaction_handles.push(handle);
        }
        
        // Start mining on multiple miners
        for miner in &env.bond_network.miners {
            miner.start_mining().await.unwrap();
        }
        
        // Wait for transaction generation to complete
        let mut total_successful_txs = 0;
        for handle in transaction_handles {
            total_successful_txs += handle.await.unwrap();
        }
        
        println!("📈 Generated {} successful transactions", total_successful_txs);
        
        // Wait for network to process transactions
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // Mine blocks to confirm transactions
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
        
        // Verify synchronization across all nodes
        let start_time = Instant::now();
        let mut synchronization_complete = false;
        
        while start_time.elapsed() < Duration::from_secs(300) && !synchronization_complete {
            let mut block_heights = HashMap::new();
            
            // Collect block heights from all nodes
            for (i, node) in env.bond_network.nodes.iter().enumerate() {
                let height = node.get_current_block_height().await.unwrap();
                block_heights.insert(i, height);
            }
            
            // Check if all nodes are within 1 block of each other
            let min_height = *block_heights.values().min().unwrap();
            let max_height = *block_heights.values().max().unwrap();
            
            if max_height - min_height <= 1 {
                synchronization_complete = true;
                println!("✅ Network synchronization complete - all nodes within 1 block");
                println!("   Height range: {} - {}", min_height, max_height);
            } else {
                println!("⏳ Synchronization in progress - height range: {} - {}", min_height, max_height);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
        
        assert!(synchronization_complete, "Network should synchronize within 5 minutes");
        
        // Verify transaction consistency across nodes
        let sample_block_height = env.bond_network.nodes[0].get_current_block_height().await.unwrap();
        let reference_block = env.bond_network.nodes[0].get_block_by_height(sample_block_height - 1).await.unwrap();
        
        for (i, node) in env.bond_network.nodes.iter().enumerate() {
            let node_block = node.get_block_by_height(sample_block_height - 1).await.unwrap();
            assert_eq!(reference_block.hash(), node_block.hash(), 
                      "Block hash mismatch on node {}", i);
        }
        
        println!("✅ Transaction consistency verified across all nodes");
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn network_partition_recovery() {
        let config = NetworkTestConfig {
            bond_nodes_count: 6,
            aevum_nodes_count: 4,
            miners_count: 2,
            validators_count: 3,
            network_latency_ms: 50,
            packet_loss_rate: 0.005,
            bandwidth_limit_mbps: 50,
            partition_scenarios: vec![
                PartitionScenario {
                    name: "Split Brain Scenario".to_string(),
                    duration: Duration::from_secs(60),
                    partitioned_nodes: vec!["node_0".to_string(), "node_1".to_string(), "node_2".to_string()],
                    recovery_strategy: RecoveryStrategy::Automatic,
                }
            ],
        };
        
        let mut env = MultiNodeTestEnvironment::new(config).await;
        env.start_full_network().await.unwrap();
        
        println!("🔀 Testing network partition recovery");
        
        // Record initial state
        let initial_heights: HashMap<usize, u64> = {
            let mut heights = HashMap::new();
            for (i, node) in env.bond_network.nodes.iter().enumerate() {
                heights.insert(i, node.get_current_block_height().await.unwrap());
            }
            heights
        };
        
        // Create network partition - split network into two groups
        let partition_1 = vec![0, 1, 2]; // First group of nodes
        let partition_2 = vec![3, 4, 5]; // Second group of nodes
        
        env.network_partitioner.create_partition(partition_1.clone(), partition_2.clone()).await.unwrap();
        println!("  📡 Network partitioned - Group 1: {:?}, Group 2: {:?}", partition_1, partition_2);
        
        // Continue mining in both partitions
        for i in partition_1.iter() {
            if i < &env.bond_network.miners.len() {
                env.bond_network.miners[*i].start_mining().await.unwrap();
            }
        }
        
        for i in partition_2.iter() {
            if i < &env.bond_network.miners.len() {
                env.bond_network.miners[*i].start_mining().await.unwrap();
            }
        }
        
        // Let both partitions mine blocks separately
        tokio::time::sleep(Duration::from_secs(60)).await;
        
        println!("  ⛏️  Mining completed in both partitions");
        
        // Record state during partition
        let partition_heights: HashMap<usize, u64> = {
            let mut heights = HashMap::new();
            for (i, node) in env.bond_network.nodes.iter().enumerate() {
                heights.insert(i, node.get_current_block_height().await.unwrap());
            }
            heights
        };
        
        // Verify that partitions diverged
        let group1_height = partition_heights[&0];
        let group2_height = partition_heights[&3];
        
        println!("  📊 Partition heights - Group 1: {}, Group 2: {}", group1_height, group2_height);
        
        // Both groups should have made progress
        assert!(group1_height > initial_heights[&0], "Group 1 should have mined new blocks");
        assert!(group2_height > initial_heights[&3], "Group 2 should have mined new blocks");
        
        // Remove partition - heal the network
        env.network_partitioner.heal_partition().await.unwrap();
        println!("  🔗 Network partition healed - reconnecting nodes");
        
        // Wait for network to stabilize and reorganize
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // Verify network reorganization
        let recovery_start = Instant::now();
        let mut reorganization_complete = false;
        
        while recovery_start.elapsed() < Duration::from_secs(180) && !reorganization_complete {
            let mut heights = HashMap::new();
            let mut best_hashes = HashMap::new();
            
            for (i, node) in env.bond_network.nodes.iter().enumerate() {
                heights.insert(i, node.get_current_block_height().await.unwrap());
                best_hashes.insert(i, node.get_best_block_hash().await.unwrap());
            }
            
            // Check if all nodes converged to the same chain
            let unique_hashes: std::collections::HashSet<_> = best_hashes.values().collect();
            let min_height = *heights.values().min().unwrap();
            let max_height = *heights.values().max().unwrap();
            
            if unique_hashes.len() == 1 && max_height - min_height <= 1 {
                reorganization_complete = true;
                println!("  ✅ Network reorganization complete");
                println!("     Consensus height: {}", max_height);
                println!("     Consensus hash: {}", best_hashes.values().next().unwrap());
            } else {
                println!("  ⏳ Reorganization in progress - {} unique chains, height range: {}-{}", 
                        unique_hashes.len(), min_height, max_height);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
        
        assert!(reorganization_complete, "Network should reorganize within 3 minutes");
        
        // Verify the winning chain has the most work
        let final_height = env.bond_network.nodes[0].get_current_block_height().await.unwrap();
        let expected_min_height = std::cmp::max(group1_height, group2_height);
        
        assert!(final_height >= expected_min_height, 
               "Final chain should have at least the height of the longest partition");
        
        println!("✅ Network partition recovery test completed successfully");
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn node_discovery_and_dynamic_joining() {
        let config = NetworkTestConfig {
            bond_nodes_count: 5,
            aevum_nodes_count: 3,
            miners_count: 2,
            validators_count: 2,
            network_latency_ms: 25,
            packet_loss_rate: 0.001,
            bandwidth_limit_mbps: 200,
            partition_scenarios: vec![],
        };
        
        let mut env = MultiNodeTestEnvironment::new(config).await;
        
        // Start initial network with fewer nodes
        let initial_bond_nodes = 3;
        let initial_aevum_nodes = 2;
        
        // Start only subset of nodes initially
        for i in 0..initial_bond_nodes {
            env.bond_network.nodes[i].start().await.unwrap();
        }
        
        for i in 0..initial_aevum_nodes {
            env.aevum_network.validators[i].start().await.unwrap();
        }
        
        // Wait for initial network to stabilize
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        println!("🌱 Initial network started with {} Bond nodes and {} Aevum nodes", 
                initial_bond_nodes, initial_aevum_nodes);
        
        // Verify initial network connectivity
        for i in 0..initial_bond_nodes {
            let peer_count = env.bond_network.nodes[i].get_peer_count().await.unwrap();
            assert!(peer_count >= initial_bond_nodes - 1, 
                   "Node {} should connect to other initial nodes", i);
        }
        
        // Start mining to create blockchain activity
        env.bond_network.miners[0].start_mining().await.unwrap();
        
        // Generate some blockchain history
        tokio::time::sleep(Duration::from_secs(20)).await;
        
        let pre_join_height = env.bond_network.nodes[0].get_current_block_height().await.unwrap();
        println!("  📊 Pre-join blockchain height: {}", pre_join_height);
        
        // Dynamically add new nodes one by one
        for i in initial_bond_nodes..env.bond_network.nodes.len() {
            println!("  🔗 Adding Bond node {} to network", i);
            
            env.bond_network.nodes[i].start().await.unwrap();
            
            // Wait for node discovery and synchronization
            let sync_start = Instant::now();
            let mut sync_complete = false;
            
            while sync_start.elapsed() < Duration::from_secs(120) && !sync_complete {
                let peer_count = env.bond_network.nodes[i].get_peer_count().await.unwrap();
                let node_height = env.bond_network.nodes[i].get_current_block_height().await.unwrap();
                let network_height = env.bond_network.nodes[0].get_current_block_height().await.unwrap();
                
                if peer_count >= 2 && node_height >= network_height - 1 {
                    sync_complete = true;
                    println!("    ✅ Node {} synchronized - peers: {}, height: {}", 
                            i, peer_count, node_height);
                } else {
                    println!("    ⏳ Node {} syncing - peers: {}, height: {} (network: {})", 
                            i, peer_count, node_height, network_height);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
            
            assert!(sync_complete, "Node {} should synchronize within 2 minutes", i);
            
            // Brief pause between node additions
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        
        // Add remaining Aevum validators
        for i in initial_aevum_nodes..env.aevum_network.validators.len() {
            println!("  🔗 Adding Aevum validator {} to network", i);
            
            env.aevum_network.validators[i].start().await.unwrap();
            
            // Wait for validator synchronization
            let sync_start = Instant::now();
            let mut sync_complete = false;
            
            while sync_start.elapsed() < Duration::from_secs(60) && !sync_complete {
                let peer_count = env.aevum_network.validators[i].get_peer_count().await.unwrap();
                let validator_height = env.aevum_network.validators[i].get_current_block_height().await.unwrap();
                let network_height = env.aevum_network.validators[0].get_current_block_height().await.unwrap();
                
                if peer_count >= 1 && validator_height >= network_height - 1 {
                    sync_complete = true;
                    println!("    ✅ Validator {} synchronized - peers: {}, height: {}", 
                            i, peer_count, validator_height);
                } else {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
            
            assert!(sync_complete, "Validator {} should synchronize within 1 minute", i);
        }
        
        // Verify final network state
        println!("🔍 Verifying final network state");
        
        // All Bond nodes should be well connected
        for (i, node) in env.bond_network.nodes.iter().enumerate() {
            let peer_count = node.get_peer_count().await.unwrap();
            let node_height = node.get_current_block_height().await.unwrap();
            
            assert!(peer_count >= 3, "Node {} should have at least 3 peers", i);
            assert!(node_height >= pre_join_height, "Node {} should have current blockchain state", i);
            
            println!("  📊 Bond node {}: {} peers, height {}", i, peer_count, node_height);
        }
        
        // All Aevum validators should be connected
        for (i, validator) in env.aevum_network.validators.iter().enumerate() {
            let peer_count = validator.get_peer_count().await.unwrap();
            
            assert!(peer_count >= 1, "Validator {} should have at least 1 peer", i);
            
            println!("  📊 Aevum validator {}: {} peers", i, peer_count);
        }
        
        println!("✅ Dynamic node joining test completed successfully");
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn load_balancing_and_performance() {
        let config = NetworkTestConfig {
            bond_nodes_count: 8,
            aevum_nodes_count: 6,
            miners_count: 3,
            validators_count: 4,
            network_latency_ms: 20,
            packet_loss_rate: 0.0,
            bandwidth_limit_mbps: 1000, // High bandwidth for performance test
            partition_scenarios: vec![],
        };
        
        let mut env = MultiNodeTestEnvironment::new(config).await;
        env.start_full_network().await.unwrap();
        
        println!("⚡ Starting load balancing and performance test");
        
        // Start performance monitoring
        env.metrics_collector.start_monitoring().await;
        
        // Start all miners
        for miner in &env.bond_network.miners {
            miner.start_mining().await.unwrap();
        }
        
        // Generate high transaction load
        let load_test_duration = Duration::from_secs(120);
        let transactions_per_second = 50;
        let total_transactions = (load_test_duration.as_secs() * transactions_per_second as u64) as usize;
        
        println!("  📈 Generating {} transactions over {} seconds", 
                total_transactions, load_test_duration.as_secs());
        
        let start_time = Instant::now();
        let mut transaction_handles = vec![];
        let nodes_count = env.bond_network.nodes.len();
        
        for i in 0..total_transactions {
            let node_index = i % nodes_count;
            let node = env.bond_network.nodes[node_index].clone();
            
            let handle = tokio::spawn(async move {
                let tx = create_load_test_transaction(i).await;
                let submit_start = Instant::now();
                
                match node.submit_transaction(tx).await {
                    Ok(tx_hash) => {
                        TransactionResult {
                            success: true,
                            submission_time: submit_start.elapsed(),
                            tx_hash: Some(tx_hash),
                            error: None,
                        }
                    }
                    Err(e) => {
                        TransactionResult {
                            success: false,
                            submission_time: submit_start.elapsed(),
                            tx_hash: None,
                            error: Some(e.to_string()),
                        }
                    }
                }
            });
            
            transaction_handles.push(handle);
            
            // Control transaction rate
            let target_interval = Duration::from_millis(1000 / transactions_per_second);
            tokio::time::sleep(target_interval).await;
            
            if start_time.elapsed() >= load_test_duration {
                break;
            }
        }
        
        // Wait for all transactions to complete
        let mut successful_transactions = 0;
        let mut failed_transactions = 0;
        let mut total_submission_time = Duration::from_secs(0);
        
        for handle in transaction_handles {
            let result = handle.await.unwrap();
            if result.success {
                successful_transactions += 1;
            } else {
                failed_transactions += 1;
            }
            total_submission_time += result.submission_time;
        }
        
        let success_rate = successful_transactions as f64 / (successful_transactions + failed_transactions) as f64;
        let average_submission_time = total_submission_time / (successful_transactions + failed_transactions) as u32;
        
        println!("  📊 Transaction Results:");
        println!("    ✅ Successful: {} ({:.1}%)", successful_transactions, success_rate * 100.0);
        println!("    ❌ Failed: {}", failed_transactions);
        println!("    ⏱️  Average submission time: {:?}", average_submission_time);
        
        // Assert performance requirements
        assert!(success_rate >= 0.95, "Transaction success rate should be at least 95%");
        assert!(average_submission_time < Duration::from_millis(100), 
               "Average submission time should be under 100ms");
        
        // Wait for transaction processing
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // Collect network metrics
        let metrics = env.metrics_collector.collect_metrics().await;
        
        println!("  📊 Network Performance Metrics:");
        println!("    🔗 Average peer count: {:.1}", metrics.average_peer_count);
        println!("    📡 Message propagation time: {:?}", metrics.average_message_propagation_time);
        println!("    💾 Memory usage per node: {:.1} MB", metrics.average_memory_usage_mb);
        println!("    🔥 CPU usage per node: {:.1}%", metrics.average_cpu_usage_percent);
        println!("    🌐 Network bandwidth usage: {:.1} MB/s", metrics.network_bandwidth_mbps);
        
        // Assert resource usage is within limits
        assert!(metrics.average_memory_usage_mb < 1024.0, "Memory usage should be under 1GB per node");
        assert!(metrics.average_cpu_usage_percent < 80.0, "CPU usage should be under 80%");
        assert!(metrics.average_message_propagation_time < Duration::from_millis(500), 
               "Message propagation should be under 500ms");
        
        println!("✅ Load balancing and performance test completed successfully");
        
        env.cleanup().await;
    }

    // Helper structures and functions
    
    struct TransactionResult {
        success: bool,
        submission_time: Duration,
        tx_hash: Option<String>,
        error: Option<String>,
    }
    
    struct NetworkMetrics {
        average_peer_count: f64,
        average_message_propagation_time: Duration,
        average_memory_usage_mb: f64,
        average_cpu_usage_percent: f64,
        network_bandwidth_mbps: f64,
    }
    
    #[derive(Debug)]
    enum NetworkError {
        InsufficientPeers { node_id: usize, actual_peers: usize, expected_min: usize },
        SynchronizationTimeout { node_id: usize, timeout: Duration },
        PartitionError(String),
        NodeStartupError { node_id: usize, error: String },
    }
    
    impl std::fmt::Display for NetworkError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NetworkError::InsufficientPeers { node_id, actual_peers, expected_min } => {
                    write!(f, "Node {} has {} peers, expected at least {}", node_id, actual_peers, expected_min)
                }
                NetworkError::SynchronizationTimeout { node_id, timeout } => {
                    write!(f, "Node {} failed to synchronize within {:?}", node_id, timeout)
                }
                NetworkError::PartitionError(msg) => write!(f, "Partition error: {}", msg),
                NetworkError::NodeStartupError { node_id, error } => {
                    write!(f, "Node {} startup error: {}", node_id, error)
                }
            }
        }
    }
    
    impl std::error::Error for NetworkError {}
    
    async fn create_test_transaction(id: String, amount: u64, fee: u64) -> Transaction {
        // Create a realistic test transaction
        Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_string(&id), 0),
                script_sig: Script::from_bytes(vec![0x51]), // Simple script
            }],
            vec![TransactionOutput {
                value: amount - fee,
                script_pubkey: Script::from_bytes(vec![0x52]),
            }],
        ).unwrap()
    }
    
    async fn create_load_test_transaction(index: usize) -> Transaction {
        let amount = 1_000_000 + (index as u64 * 1000); // Varying amounts
        let fee = 10_000 + (index as u64 * 10); // Varying fees
        
        Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_bytes([index as u8; 32]), 0),
                script_sig: Script::from_bytes(vec![0x51]),
            }],
            vec![TransactionOutput {
                value: amount - fee,
                script_pubkey: Script::from_bytes(vec![0x52, index as u8]),
            }],
        ).unwrap()
    }
}
```
