# Camada 4: Network Tests - Fork Resolution

## 4.3 Testes de Resolução de Forks

### Complete Fork Resolution Tests
```rust
#[cfg(test)]
mod fork_resolution_tests {
    use super::*;
    use std::collections::{HashMap, HashSet, VecDeque};
    use tokio::time::{timeout, Duration, Instant};
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

    struct ForkResolutionTestEnvironment {
        temp_dir: TempDir,
        network_nodes: Vec<NetworkNode>,
        fork_scenarios: HashMap<String, ForkScenario>,
        consensus_monitor: ConsensusMonitor,
        fork_analyzer: ForkAnalyzer,
        test_config: ForkTestConfig,
    }

    struct NetworkNode {
        node_id: String,
        node_type: NetworkNodeType,
        chain_state: Arc<RwLock<ChainState>>,
        peer_connections: Arc<Mutex<Vec<String>>>,
        fork_detector: ForkDetector,
        consensus_rules: ConsensusRules,
        mining_capability: Option<MiningCapability>,
    }

    #[derive(Debug, Clone)]
    enum NetworkNodeType {
        Miner { hash_rate: u64 },
        FullNode,
        Validator { stake: u64 },
        LightClient,
    }

    #[derive(Debug, Clone)]
    struct ForkScenario {
        name: String,
        description: String,
        fork_type: ForkType,
        trigger_conditions: Vec<TriggerCondition>,
        expected_resolution: ExpectedResolution,
        max_resolution_time: Duration,
    }

    #[derive(Debug, Clone)]
    enum ForkType {
        /// Two competing chains of equal length
        CompetingChains { 
            chain_a_miners: Vec<String>,
            chain_b_miners: Vec<String>,
            fork_depth: u64,
        },
        /// Temporary network partition creating divergent chains
        NetworkPartition {
            partition_duration: Duration,
            partition_groups: Vec<Vec<String>>,
        },
        /// Reorganization due to longer chain discovery
        ChainReorganization {
            original_height: u64,
            competing_height: u64,
            work_difference: f64,
        },
        /// Double spending attack scenario
        DoubleSpendAttack {
            attacker_nodes: Vec<String>,
            target_transactions: Vec<String>,
            attack_depth: u64,
        },
        /// Selfish mining attack
        SelfishMining {
            attacker_hash_rate: f64,
            withholding_strategy: WithholdingStrategy,
        },
        /// 51% attack simulation
        MajorityAttack {
            attacker_hash_rate: f64,
            attack_duration: Duration,
            attack_strategy: AttackStrategy,
        },
    }

    #[derive(Debug, Clone)]
    enum WithholdingStrategy {
        AlwaysWithhold,
        ConditionalRelease { lead_threshold: u64 },
        RandomizedRelease { probability: f64 },
    }

    #[derive(Debug, Clone)]
    enum AttackStrategy {
        ChainRewrite { target_blocks: u64 },
        DoubleSpend { transactions: Vec<String> },
        CensorshipAttack { target_addresses: Vec<String> },
    }

    #[derive(Debug, Clone)]
    struct TriggerCondition {
        condition_type: ConditionType,
        parameters: HashMap<String, String>,
    }

    #[derive(Debug, Clone)]
    enum ConditionType {
        BlockHeight(u64),
        NetworkPartition,
        MinerActivation(String),
        TransactionSubmission(String),
        TimeDelay(Duration),
    }

    #[derive(Debug, Clone)]
    struct ExpectedResolution {
        winning_chain_criteria: WinningChainCriteria,
        resolution_mechanism: ResolutionMechanism,
        expected_outcome: ExpectedOutcome,
    }

    #[derive(Debug, Clone)]
    enum WinningChainCriteria {
        LongestChain,
        MostWork,
        FinalizerVotes,
        CombinedScore { work_weight: f64, finality_weight: f64 },
    }

    #[derive(Debug, Clone)]
    enum ResolutionMechanism {
        ProofOfWork,
        ProofOfStake,
        HybridConsensus,
        ChainlocksMechanism,
    }

    #[derive(Debug, Clone)]
    enum ExpectedOutcome {
        SingleChainConsensus { expected_winner: Option<String> },
        PartialConsensus { acceptable_divergence: u64 },
        AttackMitigation { expected_defense: DefenseType },
    }

    #[derive(Debug, Clone)]
    enum DefenseType {
        ChainReorganization,
        TransactionReversion,
        NetworkIsolation,
        CheckpointActivation,
    }

    struct ForkTestConfig {
        network_size: usize,
        miners_count: usize,
        validators_count: usize,
        simulation_duration: Duration,
        block_time_target: Duration,
        fork_detection_threshold: u64,
        reorganization_limit: u64,
    }

    impl ForkResolutionTestEnvironment {
        async fn new(config: ForkTestConfig) -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_path = temp_dir.path().to_path_buf();
            
            // Initialize network nodes
            let mut network_nodes = Vec::new();
            
            // Create miners
            for i in 0..config.miners_count {
                let miner = NetworkNode::new_miner(
                    format!("miner_{}", i),
                    base_path.join(format!("miner_{}", i)),
                    1000 + i as u64 * 500, // Varying hash rates
                ).await;
                network_nodes.push(miner);
            }
            
            // Create validators for hybrid consensus
            for i in 0..config.validators_count {
                let validator = NetworkNode::new_validator(
                    format!("validator_{}", i),
                    base_path.join(format!("validator_{}", i)),
                    10000 + i as u64 * 5000, // Varying stakes
                ).await;
                network_nodes.push(validator);
            }
            
            // Create additional full nodes
            let full_nodes_count = config.network_size - config.miners_count - config.validators_count;
            for i in 0..full_nodes_count {
                let full_node = NetworkNode::new_full_node(
                    format!("full_node_{}", i),
                    base_path.join(format!("full_node_{}", i)),
                ).await;
                network_nodes.push(full_node);
            }
            
            let fork_scenarios = Self::initialize_fork_scenarios();
            let consensus_monitor = ConsensusMonitor::new();
            let fork_analyzer = ForkAnalyzer::new();
            
            Self {
                temp_dir,
                network_nodes,
                fork_scenarios,
                consensus_monitor,
                fork_analyzer,
                test_config: config,
            }
        }
        
        fn initialize_fork_scenarios() -> HashMap<String, ForkScenario> {
            let mut scenarios = HashMap::new();
            
            // Scenario 1: Competing chains
            scenarios.insert("competing_chains".to_string(), ForkScenario {
                name: "Competing Chains".to_string(),
                description: "Two groups of miners create competing chains simultaneously".to_string(),
                fork_type: ForkType::CompetingChains {
                    chain_a_miners: vec!["miner_0".to_string(), "miner_1".to_string()],
                    chain_b_miners: vec!["miner_2".to_string(), "miner_3".to_string()],
                    fork_depth: 5,
                },
                trigger_conditions: vec![
                    TriggerCondition {
                        condition_type: ConditionType::BlockHeight(100),
                        parameters: HashMap::new(),
                    }
                ],
                expected_resolution: ExpectedResolution {
                    winning_chain_criteria: WinningChainCriteria::LongestChain,
                    resolution_mechanism: ResolutionMechanism::ProofOfWork,
                    expected_outcome: ExpectedOutcome::SingleChainConsensus { expected_winner: None },
                },
                max_resolution_time: Duration::from_secs(120),
            });
            
            // Scenario 2: Network partition
            scenarios.insert("network_partition".to_string(), ForkScenario {
                name: "Network Partition".to_string(),
                description: "Network splits into isolated groups, creating divergent chains".to_string(),
                fork_type: ForkType::NetworkPartition {
                    partition_duration: Duration::from_secs(60),
                    partition_groups: vec![
                        vec!["miner_0".to_string(), "miner_1".to_string(), "full_node_0".to_string()],
                        vec!["miner_2".to_string(), "miner_3".to_string(), "full_node_1".to_string()],
                    ],
                },
                trigger_conditions: vec![
                    TriggerCondition {
                        condition_type: ConditionType::BlockHeight(50),
                        parameters: HashMap::new(),
                    }
                ],
                expected_resolution: ExpectedResolution {
                    winning_chain_criteria: WinningChainCriteria::MostWork,
                    resolution_mechanism: ResolutionMechanism::ProofOfWork,
                    expected_outcome: ExpectedOutcome::SingleChainConsensus { expected_winner: None },
                },
                max_resolution_time: Duration::from_secs(180),
            });
            
            // Scenario 3: Double spend attack
            scenarios.insert("double_spend_attack".to_string(), ForkScenario {
                name: "Double Spend Attack".to_string(),
                description: "Attacker attempts to reverse transactions by mining competing chain".to_string(),
                fork_type: ForkType::DoubleSpendAttack {
                    attacker_nodes: vec!["miner_0".to_string()],
                    target_transactions: vec!["tx_double_spend_1".to_string()],
                    attack_depth: 6,
                },
                trigger_conditions: vec![
                    TriggerCondition {
                        condition_type: ConditionType::TransactionSubmission("tx_double_spend_1".to_string()),
                        parameters: HashMap::new(),
                    }
                ],
                expected_resolution: ExpectedResolution {
                    winning_chain_criteria: WinningChainCriteria::LongestChain,
                    resolution_mechanism: ResolutionMechanism::ProofOfWork,
                    expected_outcome: ExpectedOutcome::AttackMitigation { 
                        expected_defense: DefenseType::ChainReorganization 
                    },
                },
                max_resolution_time: Duration::from_secs(300),
            });
            
            // Scenario 4: 51% attack
            scenarios.insert("majority_attack".to_string(), ForkScenario {
                name: "51% Attack".to_string(),
                description: "Malicious miner with majority hash rate attempts chain rewrite".to_string(),
                fork_type: ForkType::MajorityAttack {
                    attacker_hash_rate: 0.6, // 60% of network hash rate
                    attack_duration: Duration::from_secs(120),
                    attack_strategy: AttackStrategy::ChainRewrite { target_blocks: 10 },
                },
                trigger_conditions: vec![
                    TriggerCondition {
                        condition_type: ConditionType::BlockHeight(75),
                        parameters: HashMap::new(),
                    }
                ],
                expected_resolution: ExpectedResolution {
                    winning_chain_criteria: WinningChainCriteria::LongestChain,
                    resolution_mechanism: ResolutionMechanism::ProofOfWork,
                    expected_outcome: ExpectedOutcome::AttackMitigation { 
                        expected_defense: DefenseType::CheckpointActivation 
                    },
                },
                max_resolution_time: Duration::from_secs(240),
            });
            
            scenarios
        }
        
        async fn start_network(&mut self) -> Result<(), ForkResolutionError> {
            println!("🌐 Starting fork resolution test network with {} nodes", self.network_nodes.len());
            
            // Start all nodes
            for node in &mut self.network_nodes {
                node.start().await?;
            }
            
            // Establish peer connections (full mesh for testing)
            self.establish_peer_connections().await?;
            
            // Wait for network stabilization
            tokio::time::sleep(Duration::from_secs(10)).await;
            
            // Start consensus monitoring
            self.consensus_monitor.start_monitoring(&self.network_nodes).await?;
            
            println!("  ✅ Network started and monitoring active");
            Ok(())
        }
        
        async fn establish_peer_connections(&mut self) -> Result<(), ForkResolutionError> {
            let node_ids: Vec<String> = self.network_nodes.iter().map(|n| n.node_id.clone()).collect();
            
            for node in &mut self.network_nodes {
                let other_nodes: Vec<String> = node_ids.iter()
                    .filter(|id| *id != &node.node_id)
                    .cloned()
                    .collect();
                
                node.connect_to_peers(other_nodes).await?;
            }
            
            Ok(())
        }
        
        async fn execute_fork_scenario(&mut self, scenario_name: &str) -> Result<ForkResolutionResult, ForkResolutionError> {
            let scenario = self.fork_scenarios.get(scenario_name)
                .ok_or_else(|| ForkResolutionError::ScenarioNotFound(scenario_name.to_string()))?
                .clone();
            
            println!("🍴 Executing fork scenario: {}", scenario.name);
            println!("   📝 {}", scenario.description);
            
            let start_time = Instant::now();
            
            // Initialize scenario tracking
            let mut fork_result = ForkResolutionResult {
                scenario_name: scenario_name.to_string(),
                execution_duration: Duration::from_secs(0),
                fork_detected: false,
                fork_detection_time: None,
                resolution_achieved: false,
                resolution_time: None,
                winning_chain: None,
                resolution_details: ResolutionDetails::default(),
                network_health: NetworkHealth::default(),
            };
            
            // Execute trigger conditions
            for condition in &scenario.trigger_conditions {
                self.execute_trigger_condition(condition).await?;
            }
            
            // Execute the specific fork scenario
            match &scenario.fork_type {
                ForkType::CompetingChains { chain_a_miners, chain_b_miners, fork_depth } => {
                    self.execute_competing_chains_scenario(chain_a_miners, chain_b_miners, *fork_depth).await?;
                }
                ForkType::NetworkPartition { partition_duration, partition_groups } => {
                    self.execute_network_partition_scenario(*partition_duration, partition_groups).await?;
                }
                ForkType::DoubleSpendAttack { attacker_nodes, target_transactions, attack_depth } => {
                    self.execute_double_spend_attack(attacker_nodes, target_transactions, *attack_depth).await?;
                }
                ForkType::MajorityAttack { attacker_hash_rate, attack_duration, attack_strategy } => {
                    self.execute_majority_attack(*attacker_hash_rate, *attack_duration, attack_strategy).await?;
                }
                _ => {
                    return Err(ForkResolutionError::UnsupportedScenario(scenario.fork_type.clone()));
                }
            }
            
            // Monitor for fork detection
            let fork_detection = self.monitor_fork_detection(scenario.max_resolution_time).await?;
            fork_result.fork_detected = fork_detection.detected;
            fork_result.fork_detection_time = fork_detection.detection_time;
            
            if fork_result.fork_detected {
                println!("  🔍 Fork detected at {:?}", fork_result.fork_detection_time.unwrap());
                
                // Monitor resolution process
                let resolution = self.monitor_fork_resolution(
                    &scenario.expected_resolution,
                    scenario.max_resolution_time
                ).await?;
                
                fork_result.resolution_achieved = resolution.achieved;
                fork_result.resolution_time = resolution.resolution_time;
                fork_result.winning_chain = resolution.winning_chain;
                fork_result.resolution_details = resolution.details;
            }
            
            fork_result.execution_duration = start_time.elapsed();
            fork_result.network_health = self.assess_network_health().await?;
            
            println!("  ✅ Scenario execution completed in {:?}", fork_result.execution_duration);
            
            Ok(fork_result)
        }
        
        async fn execute_competing_chains_scenario(
            &mut self,
            chain_a_miners: &[String],
            chain_b_miners: &[String],
            fork_depth: u64
        ) -> Result<(), ForkResolutionError> {
            println!("  ⛏️  Creating competing chains scenario");
            
            // Get current blockchain height
            let current_height = self.network_nodes[0].get_current_height().await?;
            
            // Isolate the two mining groups temporarily
            self.isolate_mining_groups(chain_a_miners, chain_b_miners).await?;
            
            // Let each group mine their own chain
            let mining_duration = Duration::from_secs(fork_depth * 10); // Assume 10s per block
            
            tokio::select! {
                _ = self.mine_with_group(chain_a_miners, fork_depth) => {},
                _ = self.mine_with_group(chain_b_miners, fork_depth) => {},
                _ = tokio::time::sleep(mining_duration) => {},
            }
            
            // Reconnect the groups to trigger fork resolution
            self.reconnect_mining_groups(chain_a_miners, chain_b_miners).await?;
            
            println!("    🔗 Mining groups reconnected, fork resolution should begin");
            
            Ok(())
        }
        
        async fn execute_network_partition_scenario(
            &mut self,
            partition_duration: Duration,
            partition_groups: &[Vec<String>]
        ) -> Result<(), ForkResolutionError> {
            println!("  🌐 Creating network partition scenario");
            
            // Create network partitions
            self.create_network_partitions(partition_groups).await?;
            
            // Continue mining in each partition
            let partition_futures: Vec<_> = partition_groups.iter().enumerate().map(|(i, group)| {
                let group_clone = group.clone();
                async move {
                    let target_blocks = 5 + i as u64; // Different number of blocks per partition
                    self.mine_with_partition(&group_clone, target_blocks).await
                }
            }).collect();
            
            // Wait for partition duration
            tokio::select! {
                _ = futures::future::join_all(partition_futures) => {},
                _ = tokio::time::sleep(partition_duration) => {},
            }
            
            // Heal network partition
            self.heal_network_partitions().await?;
            
            println!("    🔗 Network partition healed, resolution should begin");
            
            Ok(())
        }
        
        async fn execute_double_spend_attack(
            &mut self,
            attacker_nodes: &[String],
            target_transactions: &[String],
            attack_depth: u64
        ) -> Result<(), ForkResolutionError> {
            println!("  💰 Executing double spend attack scenario");
            
            // Submit original transaction to network
            let original_tx = target_transactions[0].clone();
            self.submit_transaction_to_network(&original_tx).await?;
            
            // Wait for transaction to be included in a block
            tokio::time::sleep(Duration::from_secs(15)).await;
            
            // Attacker starts mining private chain with conflicting transaction
            let conflicting_tx = format!("{}_conflicting", original_tx);
            
            for attacker in attacker_nodes {
                let attacker_node = self.get_node_mut(attacker)?;
                
                // Isolate attacker temporarily
                attacker_node.isolate_from_network().await?;
                
                // Create conflicting transaction
                attacker_node.create_conflicting_transaction(&original_tx, &conflicting_tx).await?;
                
                // Mine private chain
                attacker_node.mine_private_chain(attack_depth).await?;
                
                // Release private chain to network
                attacker_node.reconnect_to_network().await?;
                attacker_node.broadcast_private_chain().await?;
            }
            
            println!("    📡 Private chain broadcast, double spend detection should trigger");
            
            Ok(())
        }
        
        async fn execute_majority_attack(
            &mut self,
            attacker_hash_rate: f64,
            attack_duration: Duration,
            attack_strategy: &AttackStrategy
        ) -> Result<(), ForkResolutionError> {
            println!("  ⚡ Executing 51% attack scenario with {:.1}% hash rate", attacker_hash_rate * 100.0);
            
            // Identify nodes to control for majority hash rate
            let total_hash_rate: u64 = self.network_nodes.iter()
                .filter_map(|n| match &n.node_type {
                    NetworkNodeType::Miner { hash_rate } => Some(*hash_rate),
                    _ => None,
                })
                .sum();
            
            let required_hash_rate = (total_hash_rate as f64 * attacker_hash_rate) as u64;
            let mut controlled_hash_rate = 0u64;
            let mut controlled_miners = Vec::new();
            
            for node in &self.network_nodes {
                if let NetworkNodeType::Miner { hash_rate } = &node.node_type {
                    controlled_miners.push(node.node_id.clone());
                    controlled_hash_rate += hash_rate;
                    
                    if controlled_hash_rate >= required_hash_rate {
                        break;
                    }
                }
            }
            
            println!("    🎯 Controlling {} miners with {} total hash rate", 
                    controlled_miners.len(), controlled_hash_rate);
            
            // Execute attack strategy
            match attack_strategy {
                AttackStrategy::ChainRewrite { target_blocks } => {
                    self.execute_chain_rewrite_attack(&controlled_miners, *target_blocks, attack_duration).await?;
                }
                AttackStrategy::DoubleSpend { transactions } => {
                    self.execute_coordinated_double_spend(&controlled_miners, transactions, attack_duration).await?;
                }
                AttackStrategy::CensorshipAttack { target_addresses } => {
                    self.execute_censorship_attack(&controlled_miners, target_addresses, attack_duration).await?;
                }
            }
            
            Ok(())
        }
        
        async fn monitor_fork_detection(&mut self, timeout_duration: Duration) -> Result<ForkDetectionResult, ForkResolutionError> {
            let start_time = Instant::now();
            
            while start_time.elapsed() < timeout_duration {
                // Check each node for fork detection
                for node in &self.network_nodes {
                    if let Some(fork_info) = node.check_fork_detection().await? {
                        return Ok(ForkDetectionResult {
                            detected: true,
                            detection_time: Some(start_time.elapsed()),
                            detecting_node: Some(node.node_id.clone()),
                            fork_info: Some(fork_info),
                        });
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            
            Ok(ForkDetectionResult {
                detected: false,
                detection_time: None,
                detecting_node: None,
                fork_info: None,
            })
        }
        
        async fn monitor_fork_resolution(
            &mut self,
            expected_resolution: &ExpectedResolution,
            timeout_duration: Duration
        ) -> Result<ResolutionMonitorResult, ForkResolutionError> {
            let start_time = Instant::now();
            
            while start_time.elapsed() < timeout_duration {
                // Check for consensus convergence
                let consensus_status = self.check_network_consensus().await?;
                
                if consensus_status.converged {
                    let resolution_time = start_time.elapsed();
                    
                    // Verify resolution matches expectations
                    let resolution_valid = self.validate_resolution(&consensus_status, expected_resolution).await?;
                    
                    return Ok(ResolutionMonitorResult {
                        achieved: true,
                        resolution_time: Some(resolution_time),
                        winning_chain: consensus_status.winning_chain,
                        details: ResolutionDetails {
                            resolution_mechanism: expected_resolution.resolution_mechanism.clone(),
                            blocks_reorganized: consensus_status.blocks_reorganized,
                            transactions_affected: consensus_status.transactions_affected,
                            network_agreement: consensus_status.agreement_percentage,
                        },
                    });
                }
                
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            
            Ok(ResolutionMonitorResult {
                achieved: false,
                resolution_time: None,
                winning_chain: None,
                details: ResolutionDetails::default(),
            })
        }
        
        async fn check_network_consensus(&self) -> Result<ConsensusStatus, ForkResolutionError> {
            let mut chain_tips = HashMap::new();
            let mut block_heights = HashMap::new();
            
            // Collect chain state from all nodes
            for node in &self.network_nodes {
                let best_hash = node.get_best_block_hash().await?;
                let height = node.get_current_height().await?;
                
                *chain_tips.entry(best_hash.clone()).or_insert(0) += 1;
                block_heights.insert(node.node_id.clone(), height);
            }
            
            // Determine if consensus is achieved
            let total_nodes = self.network_nodes.len();
            let (winning_hash, agreement_count) = chain_tips.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(hash, count)| (hash.clone(), *count))
                .unwrap_or_default();
            
            let agreement_percentage = agreement_count as f64 / total_nodes as f64;
            let converged = agreement_percentage >= 0.8; // 80% agreement threshold
            
            // Calculate reorganization metrics
            let heights: Vec<u64> = block_heights.values().cloned().collect();
            let min_height = heights.iter().min().cloned().unwrap_or(0);
            let max_height = heights.iter().max().cloned().unwrap_or(0);
            let blocks_reorganized = max_height - min_height;
            
            Ok(ConsensusStatus {
                converged,
                winning_chain: Some(winning_hash),
                agreement_percentage,
                blocks_reorganized,
                transactions_affected: 0, // Would need transaction analysis
            })
        }
        
        async fn assess_network_health(&self) -> Result<NetworkHealth, ForkResolutionError> {
            let mut total_peers = 0;
            let mut total_sync_time = Duration::from_secs(0);
            let mut active_nodes = 0;
            
            for node in &self.network_nodes {
                if node.is_active().await? {
                    active_nodes += 1;
                    total_peers += node.get_peer_count().await?;
                    total_sync_time += node.get_last_sync_duration().await?;
                }
            }
            
            let average_peers = if active_nodes > 0 { total_peers / active_nodes } else { 0 };
            let average_sync_time = if active_nodes > 0 { total_sync_time / active_nodes as u32 } else { Duration::from_secs(0) };
            
            Ok(NetworkHealth {
                active_nodes: active_nodes,
                total_nodes: self.network_nodes.len(),
                average_peer_count: average_peers,
                average_sync_time,
                network_partition_detected: false, // Would need deeper analysis
            })
        }
        
        // Helper methods for scenario execution
        async fn isolate_mining_groups(&mut self, group_a: &[String], group_b: &[String]) -> Result<(), ForkResolutionError> {
            // Implementation for isolating mining groups
            Ok(())
        }
        
        async fn reconnect_mining_groups(&mut self, group_a: &[String], group_b: &[String]) -> Result<(), ForkResolutionError> {
            // Implementation for reconnecting mining groups
            Ok(())
        }
        
        async fn mine_with_group(&self, miners: &[String], target_blocks: u64) -> Result<(), ForkResolutionError> {
            // Implementation for coordinated mining
            Ok(())
        }
        
        async fn get_node_mut(&mut self, node_id: &str) -> Result<&mut NetworkNode, ForkResolutionError> {
            self.network_nodes.iter_mut()
                .find(|n| n.node_id == node_id)
                .ok_or_else(|| ForkResolutionError::NodeNotFound(node_id.to_string()))
        }
        
        // Additional helper methods would be implemented here...
    }

    #[tokio::test]
    async fn competing_chains_resolution() {
        let config = ForkTestConfig {
            network_size: 8,
            miners_count: 4,
            validators_count: 2,
            simulation_duration: Duration::from_secs(300),
            block_time_target: Duration::from_secs(10),
            fork_detection_threshold: 2,
            reorganization_limit: 10,
        };
        
        let mut env = ForkResolutionTestEnvironment::new(config).await;
        env.start_network().await.unwrap();
        
        println!("🍴 Testing competing chains fork resolution");
        
        let result = env.execute_fork_scenario("competing_chains").await.unwrap();
        
        println!("📊 Fork Resolution Results:");
        println!("  🔍 Fork detected: {}", result.fork_detected);
        if let Some(detection_time) = result.fork_detection_time {
            println!("  ⏱️  Detection time: {:?}", detection_time);
        }
        println!("  ✅ Resolution achieved: {}", result.resolution_achieved);
        if let Some(resolution_time) = result.resolution_time {
            println!("  ⏱️  Resolution time: {:?}", resolution_time);
        }
        if let Some(ref winning_chain) = result.winning_chain {
            println!("  🏆 Winning chain: {}", winning_chain);
        }
        
        // Assert test requirements
        assert!(result.fork_detected, "Fork should be detected");
        assert!(result.resolution_achieved, "Fork should be resolved");
        assert!(result.resolution_time.unwrap() < Duration::from_secs(120), 
               "Resolution should complete within 2 minutes");
        assert!(result.network_health.active_nodes >= 6, 
               "At least 75% of nodes should remain active");
        
        println!("✅ Competing chains resolution test completed successfully");
    }

    #[tokio::test]
    async fn network_partition_fork_handling() {
        let config = ForkTestConfig {
            network_size: 10,
            miners_count: 4,
            validators_count: 2,
            simulation_duration: Duration::from_secs(360),
            block_time_target: Duration::from_secs(10),
            fork_detection_threshold: 1,
            reorganization_limit: 20,
        };
        
        let mut env = ForkResolutionTestEnvironment::new(config).await;
        env.start_network().await.unwrap();
        
        println!("🌐 Testing network partition fork handling");
        
        let result = env.execute_fork_scenario("network_partition").await.unwrap();
        
        println!("📊 Network Partition Results:");
        println!("  🕒 Total execution time: {:?}", result.execution_duration);
        println!("  🔍 Fork detected: {}", result.fork_detected);
        println!("  ✅ Resolution achieved: {}", result.resolution_achieved);
        println!("  🔄 Blocks reorganized: {}", result.resolution_details.blocks_reorganized);
        println!("  🤝 Network agreement: {:.1}%", result.resolution_details.network_agreement * 100.0);
        
        // Assert requirements
        assert!(result.fork_detected, "Network partition should create detectable fork");
        assert!(result.resolution_achieved, "Fork should be resolved after partition heals");
        assert!(result.resolution_details.network_agreement >= 0.8, 
               "At least 80% of nodes should agree on final chain");
        assert!(result.network_health.active_nodes >= 8, 
               "Most nodes should remain active after partition");
        
        println!("✅ Network partition fork handling test completed successfully");
    }

    #[tokio::test]
    async fn double_spend_attack_detection_and_mitigation() {
        let config = ForkTestConfig {
            network_size: 12,
            miners_count: 6,
            validators_count: 3,
            simulation_duration: Duration::from_secs(600),
            block_time_target: Duration::from_secs(10),
            fork_detection_threshold: 1,
            reorganization_limit: 15,
        };
        
        let mut env = ForkResolutionTestEnvironment::new(config).await;
        env.start_network().await.unwrap();
        
        println!("💰 Testing double spend attack detection and mitigation");
        
        let result = env.execute_fork_scenario("double_spend_attack").await.unwrap();
        
        println!("📊 Double Spend Attack Results:");
        println!("  🎯 Attack detected: {}", result.fork_detected);
        println!("  🛡️  Mitigation successful: {}", result.resolution_achieved);
        
        if result.fork_detected {
            println!("  ⏱️  Detection time: {:?}", result.fork_detection_time.unwrap());
            
            if result.resolution_achieved {
                println!("  🔄 Resolution time: {:?}", result.resolution_time.unwrap());
                println!("  📊 Defense mechanism: {:?}", result.resolution_details.resolution_mechanism);
            }
        }
        
        // Assert security requirements
        assert!(result.fork_detected, "Double spend attack should be detected");
        
        if result.resolution_achieved {
            // If attack was mitigated
            assert!(result.resolution_time.unwrap() < Duration::from_secs(300), 
                   "Attack mitigation should complete within 5 minutes");
            assert!(result.resolution_details.blocks_reorganized <= 15, 
                   "Reorganization should be limited to prevent deep rewrites");
        }
        
        // Network should remain healthy
        assert!(result.network_health.active_nodes >= 10, 
               "Network should maintain most nodes during attack");
        
        println!("✅ Double spend attack test completed successfully");
    }

    #[tokio::test]
    async fn majority_attack_resilience() {
        let config = ForkTestConfig {
            network_size: 15,
            miners_count: 8,
            validators_count: 4,
            simulation_duration: Duration::from_secs(480),
            block_time_target: Duration::from_secs(10),
            fork_detection_threshold: 2,
            reorganization_limit: 25,
        };
        
        let mut env = ForkResolutionTestEnvironment::new(config).await;
        env.start_network().await.unwrap();
        
        println!("⚡ Testing 51% attack resilience");
        
        let result = env.execute_fork_scenario("majority_attack").await.unwrap();
        
        println!("📊 Majority Attack Results:");
        println!("  🚨 Attack detected: {}", result.fork_detected);
        println!("  🛡️  Defense activated: {}", result.resolution_achieved);
        println!("  ⏱️  Execution duration: {:?}", result.execution_duration);
        
        if result.fork_detected {
            println!("  🔍 Detection time: {:?}", result.fork_detection_time.unwrap());
            
            if result.resolution_achieved {
                println!("  🔒 Defense time: {:?}", result.resolution_time.unwrap());
                println!("  🛡️  Defense type: {:?}", result.resolution_details.resolution_mechanism);
                println!("  📊 Network agreement: {:.1}%", result.resolution_details.network_agreement * 100.0);
            }
        }
        
        // Assert resilience requirements
        assert!(result.fork_detected, "51% attack should be detected");
        
        // The network should either successfully defend or maintain stability
        if result.resolution_achieved {
            // Successful defense
            assert!(result.resolution_details.network_agreement >= 0.7, 
                   "Defense should maintain network consensus");
        } else {
            // Attack succeeded but network should remain functional
            assert!(result.network_health.active_nodes >= 12, 
                   "Network should maintain functionality even under attack");
        }
        
        // Performance requirements
        assert!(result.execution_duration < Duration::from_secs(480), 
               "Test should complete within time limit");
        
        println!("✅ Majority attack resilience test completed successfully");
    }

    // Helper structures and enums
    
    #[derive(Debug, Clone)]
    struct ForkResolutionResult {
        scenario_name: String,
        execution_duration: Duration,
        fork_detected: bool,
        fork_detection_time: Option<Duration>,
        resolution_achieved: bool,
        resolution_time: Option<Duration>,
        winning_chain: Option<String>,
        resolution_details: ResolutionDetails,
        network_health: NetworkHealth,
    }
    
    #[derive(Debug, Clone, Default)]
    struct ResolutionDetails {
        resolution_mechanism: ResolutionMechanism,
        blocks_reorganized: u64,
        transactions_affected: usize,
        network_agreement: f64,
    }
    
    #[derive(Debug, Clone, Default)]
    struct NetworkHealth {
        active_nodes: usize,
        total_nodes: usize,
        average_peer_count: usize,
        average_sync_time: Duration,
        network_partition_detected: bool,
    }
    
    struct ForkDetectionResult {
        detected: bool,
        detection_time: Option<Duration>,
        detecting_node: Option<String>,
        fork_info: Option<ForkInfo>,
    }
    
    struct ResolutionMonitorResult {
        achieved: bool,
        resolution_time: Option<Duration>,
        winning_chain: Option<String>,
        details: ResolutionDetails,
    }
    
    struct ConsensusStatus {
        converged: bool,
        winning_chain: Option<String>,
        agreement_percentage: f64,
        blocks_reorganized: u64,
        transactions_affected: usize,
    }
    
    #[derive(Debug)]
    enum ForkResolutionError {
        NetworkError(String),
        NodeNotFound(String),
        ScenarioNotFound(String),
        UnsupportedScenario(ForkType),
        ConsensusError(String),
        TimeoutError,
    }
    
    impl std::fmt::Display for ForkResolutionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ForkResolutionError::NetworkError(msg) => write!(f, "Network error: {}", msg),
                ForkResolutionError::NodeNotFound(node) => write!(f, "Node not found: {}", node),
                ForkResolutionError::ScenarioNotFound(scenario) => write!(f, "Scenario not found: {}", scenario),
                ForkResolutionError::UnsupportedScenario(fork_type) => write!(f, "Unsupported scenario: {:?}", fork_type),
                ForkResolutionError::ConsensusError(msg) => write!(f, "Consensus error: {}", msg),
                ForkResolutionError::TimeoutError => write!(f, "Operation timeout"),
            }
        }
    }
    
    impl std::error::Error for ForkResolutionError {}
    
    // Mock implementations would be added here for the test infrastructure...
}
```
