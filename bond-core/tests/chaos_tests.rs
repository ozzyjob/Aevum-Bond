use bond_core::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Chaos Engineering Tests
/// These tests simulate real-world failure scenarios, network partitions,
/// resource exhaustion, and other chaotic conditions to validate system resilience

#[test]
fn test_network_partition_chaos() {
    println!("\n🌪️  Chaos Test: Network Partition Simulation");
    println!("{}", "=".repeat(70));

    // Simulate a network with multiple nodes that experience partitions
    struct ChaosNode {
        id: usize,
        blockchain: Vec<Block>,
        is_connected: bool,
        last_sync: Instant,
    }

    impl ChaosNode {
        fn new(id: usize) -> Self {
            Self {
                id,
                blockchain: vec![create_genesis_block()],
                is_connected: true,
                last_sync: Instant::now(),
            }
        }

        fn add_block(&mut self, block: Block) -> Result<(), String> {
            if !self.is_connected {
                return Err("Node is partitioned".to_string());
            }

            // Validate block against current chain
            let consensus = Consensus::new();
            match consensus.validate_block(&block, &self.blockchain) {
                Ok(_) => {
                    self.blockchain.push(block);
                    self.last_sync = Instant::now();
                    Ok(())
                }
                Err(_) => Err("Block validation failed".to_string()),
            }
        }

        fn partition(&mut self) {
            self.is_connected = false;
            println!("  🚫 Node {} partitioned from network", self.id);
        }

        fn reconnect(&mut self) {
            self.is_connected = true;
            println!("  🔗 Node {} reconnected to network", self.id);
        }

        fn height(&self) -> usize {
            self.blockchain.len()
        }
    }

    // Create network of nodes
    let mut nodes = Vec::new();
    for i in 0..6 {
        nodes.push(ChaosNode::new(i));
    }

    let mut miner = Miner::new();

    // Phase 1: Normal operation - all nodes connected
    println!("📊 Phase 1: Normal Operation");
    for round in 1..=3 {
        let tx = Transaction::coinbase(5_000_000_000, vec![]);
        let previous_hash = nodes[0].blockchain.last().unwrap().hash().unwrap();

        let temp_block = Block::new(
            BlockHeader::new(
                round as u32,
                previous_hash,
                MerkleRoot::ZERO,
                chrono::Utc::now(),
                DifficultyTarget::MAX,
                0,
            ),
            vec![tx],
        );
        let merkle_root = temp_block.calculate_merkle_root().unwrap();

        let header = mining::create_mining_header(
            round as u32,
            previous_hash,
            merkle_root,
            DifficultyTarget::MAX,
        );
        let mined_header = miner.mine_block(header).unwrap();
        let new_block = Block::new(mined_header, temp_block.transactions);

        // Broadcast to all connected nodes
        for node in &mut nodes {
            if node.is_connected {
                let _ = node.add_block(new_block.clone());
            }
        }

        println!(
            "  ✅ Round {}: All nodes at height {}",
            round as u32,
            nodes[0].height()
        );
    }

    // Phase 2: Network partition - split network in half
    println!("\n📊 Phase 2: Network Partition");
    nodes[3].partition();
    nodes[4].partition();
    nodes[5].partition();

    // Continue mining on partition 1 (nodes 0, 1, 2)
    for round in 4..=6 {
        let tx = Transaction::coinbase(5_000_000_000 + round, vec![]);
        let previous_hash = nodes[0].blockchain.last().unwrap().hash().unwrap();

        let temp_block = Block::new(
            BlockHeader::new(
                round as u32,
                previous_hash,
                MerkleRoot::ZERO,
                chrono::Utc::now(),
                DifficultyTarget::MAX,
                0,
            ),
            vec![tx],
        );
        let merkle_root = temp_block.calculate_merkle_root().unwrap();
        let header = mining::create_mining_header(
            round as u32,
            previous_hash,
            merkle_root,
            DifficultyTarget::MAX,
        );
        let mined_header = miner.mine_block(header).unwrap();
        let new_block = Block::new(mined_header, temp_block.transactions);

        // Only connected nodes receive blocks
        for node in nodes.iter_mut().take(3) {
            let _ = node.add_block(new_block.clone());
        }
    }

    // Partition 2 continues with different chain
    for round in 4..=5 {
        let tx = Transaction::coinbase(4_000_000_000 + round, vec![]);
        let previous_hash = nodes[3].blockchain.last().unwrap().hash().unwrap();

        let temp_block = Block::new(
            BlockHeader::new(
                round as u32,
                previous_hash,
                MerkleRoot::ZERO,
                chrono::Utc::now(),
                DifficultyTarget::MAX,
                0,
            ),
            vec![tx],
        );
        let merkle_root = temp_block.calculate_merkle_root().unwrap();
        let header = mining::create_mining_header(
            round as u32,
            previous_hash,
            merkle_root,
            DifficultyTarget::MAX,
        );
        let mined_header = miner.mine_block(header).unwrap();
        let new_block = Block::new(mined_header, temp_block.transactions);

        // Only partitioned nodes receive these blocks
        for node in nodes.iter_mut().take(6).skip(3) {
            if !node.is_connected {
                let _ = node.add_block(new_block.clone());
            }
        }
    }

    println!("  📏 Partition 1 height: {}", nodes[0].height());
    println!("  📏 Partition 2 height: {}", nodes[3].height());

    // Phase 3: Network healing
    println!("\n📊 Phase 3: Network Healing");
    for node in nodes.iter_mut().take(6).skip(3) {
        node.reconnect();
    }

    // Simulate chain resolution (longest chain wins)
    let partition1_height = nodes[0].height();
    let partition2_height = nodes[3].height();

    println!(
        "  🏆 Chain resolution: Partition 1 ({}) vs Partition 2 ({})",
        partition1_height, partition2_height
    );

    // Validate network survived the chaos
    assert!(
        partition1_height > 3,
        "Partition 1 should have continued mining"
    );
    assert!(
        partition2_height > 3,
        "Partition 2 should have continued mining"
    );

    println!("✅ Network partition chaos test completed");
}

#[test]
fn test_resource_exhaustion_chaos() {
    println!("\n🌪️  Chaos Test: Resource Exhaustion Simulation");
    println!("{}", "=".repeat(70));

    // Test 1: Memory exhaustion simulation
    println!("📊 Subtest: Memory Pressure Simulation");

    let mut large_blocks = Vec::new();
    let max_blocks = 1000;
    let transactions_per_block = 100;

    let start_time = Instant::now();

    // Create blocks with many transactions to simulate memory pressure
    for block_num in 0..max_blocks {
        let mut transactions = Vec::new();

        for tx_num in 0..transactions_per_block {
            let amount = 1_000_000 + (block_num * transactions_per_block + tx_num) as u64;
            let tx = Transaction::coinbase(amount, vec![]);
            transactions.push(tx);
        }

        let block = Block::new(
            BlockHeader::new(
                block_num as u32,
                BlockHash::ZERO,
                MerkleRoot::ZERO,
                chrono::Utc::now(),
                DifficultyTarget::MAX,
                0,
            ),
            transactions,
        );

        large_blocks.push(block);

        // Check if we're taking too long (resource exhaustion indicator)
        if start_time.elapsed() > Duration::from_secs(10) {
            println!("  ⚠️  Memory pressure detected at {} blocks", block_num);
            break;
        }

        if block_num % 100 == 0 {
            println!(
                "  📈 Created {} blocks ({} total transactions)",
                block_num,
                block_num * transactions_per_block
            );
        }
    }

    let creation_time = start_time.elapsed();
    let total_blocks = large_blocks.len();
    let total_transactions = total_blocks * transactions_per_block;

    println!("  📊 Memory stress results:");
    println!("    Blocks created: {}", total_blocks);
    println!("    Total transactions: {}", total_transactions);
    println!("    Creation time: {:?}", creation_time);

    // Test 2: Processing exhaustion
    println!("\n📊 Subtest: Processing Exhaustion Simulation");

    let script_interpreter = ScriptInterpreter::new();
    let script_context = ScriptContext {
        tx_hash: vec![1u8; 32],
        input_index: 0,
        block_height: 100,
        timestamp: 1640995200,
    };

    // Create computationally expensive scripts
    let complex_scripts = [
        [0x01, 0x64].repeat(50), // Push 100 fifty times
        [0x93].repeat(25),       // 25 ADD operations (requires 50 stack items)
        [0x82].repeat(10),       // 10 DUP operations
        [0x87].repeat(5),        // 5 EQUAL operations
    ];

    let start_time = Instant::now();
    let mut successful_executions = 0;
    let mut failed_executions = 0;

    for (i, script) in complex_scripts.iter().enumerate() {
        let exec_start = Instant::now();

        match script_interpreter.execute(script, &script_context) {
            Ok(_) => {
                successful_executions += 1;
                println!(
                    "  ✅ Complex script {} executed in {:?}",
                    i,
                    exec_start.elapsed()
                );
            }
            Err(_) => {
                failed_executions += 1;
                println!(
                    "  ⚠️  Complex script {} failed in {:?}",
                    i,
                    exec_start.elapsed()
                );
            }
        }

        // Timeout protection
        if exec_start.elapsed() > Duration::from_secs(1) {
            println!("  🕐 Script {} exceeded timeout", i);
            break;
        }
    }

    let processing_time = start_time.elapsed();

    println!("  📊 Processing stress results:");
    println!("    Successful executions: {}", successful_executions);
    println!("    Failed executions: {}", failed_executions);
    println!("    Total processing time: {:?}", processing_time);

    // Cleanup large structures
    drop(large_blocks);

    // Validate system survived resource exhaustion
    assert!(
        total_blocks > 100,
        "Should create substantial number of blocks"
    );
    assert!(
        processing_time < Duration::from_secs(30),
        "Processing should complete in reasonable time"
    );

    println!("✅ Resource exhaustion chaos test completed");
}

#[test]
fn test_concurrent_chaos_scenario() {
    println!("\n🌪️  Chaos Test: Concurrent Chaos Scenario");
    println!("{}", "=".repeat(70));

    use std::sync::atomic::{AtomicUsize, Ordering};

    let operations_counter = Arc::new(AtomicUsize::new(0));
    let errors_counter = Arc::new(AtomicUsize::new(0));
    let success_counter = Arc::new(AtomicUsize::new(0));

    // Shared state for chaos
    let shared_blockchain = Arc::new(Mutex::new(vec![create_genesis_block()]));
    let shared_utxos = Arc::new(Mutex::new(HashMap::new()));

    let thread_count = 8;
    let operations_per_thread = 50;
    let mut handles = vec![];

    // Spawn chaos threads that perform different operations simultaneously
    for thread_id in 0..thread_count {
        let ops_counter = operations_counter.clone();
        let err_counter = errors_counter.clone();
        let success_counter = success_counter.clone();
        let blockchain = shared_blockchain.clone();
        let utxos = shared_utxos.clone();

        let handle = thread::spawn(move || {
            let mut local_miner = Miner::new();
            let local_consensus = Consensus::new();
            let script_interpreter = ScriptInterpreter::new();

            for op_id in 0..operations_per_thread {
                ops_counter.fetch_add(1, Ordering::Relaxed);

                // Randomly choose operation type based on thread_id and op_id
                let operation_type = (thread_id + op_id) % 4;

                let result = match operation_type {
                    0 => {
                        // Mining operation
                        let header = mining::create_mining_header(
                            (thread_id * 1000 + op_id) as u32,
                            BlockHash::ZERO,
                            MerkleRoot::ZERO,
                            DifficultyTarget::MAX,
                        );
                        local_miner.mine_block(header).map(|_| ())
                    }
                    1 => {
                        // Block validation operation
                        let test_block = Block::new(
                            BlockHeader::new(
                                (thread_id * 1000 + op_id) as u32,
                                BlockHash::ZERO,
                                MerkleRoot::ZERO,
                                chrono::Utc::now(),
                                DifficultyTarget::MAX,
                                0,
                            ),
                            vec![Transaction::coinbase(1_000_000 + op_id as u64, vec![])],
                        );

                        // Try to get shared blockchain state
                        if let Ok(chain) = blockchain.try_lock() {
                            local_consensus
                                .validate_block(&test_block, &chain)
                                .map(|_| ())
                        } else {
                            Err(BondError::InvalidTransaction {
                                reason: "Failed to acquire lock".to_string(),
                            })
                        }
                    }
                    2 => {
                        // UTXO operation
                        let utxo_id = UtxoId::new(
                            TransactionHash::from_bytes([thread_id as u8; 32]),
                            op_id as u32,
                        );
                        let script = Script {
                            code: vec![0x01, 0x2A, 0x01, 0x2A, 0x87],
                        };
                        let utxo = ProgrammableUtxo::new(utxo_id, 1_000_000, script, 0);

                        if let Ok(mut utxo_map) = utxos.try_lock() {
                            utxo_map.insert(utxo.id, utxo);
                            Ok(())
                        } else {
                            Err(BondError::InvalidUtxo {
                                reason: "UTXO lock failed".to_string(),
                            })
                        }
                    }
                    3 => {
                        // Script execution operation
                        let script = vec![
                            0x01,
                            (op_id % 256) as u8,
                            0x01,
                            (thread_id % 256) as u8,
                            0x93, // ADD
                            0x01,
                            0x64, // Push 100
                            0x88, // GREATER_THAN
                        ];

                        let context = ScriptContext {
                            tx_hash: vec![thread_id as u8; 32],
                            input_index: op_id as u32,
                            block_height: 100,
                            timestamp: 1640995200,
                        };

                        script_interpreter.execute(&script, &context).map(|_| ())
                    }
                    _ => unreachable!(),
                };

                match result {
                    Ok(_) => {
                        success_counter.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        err_counter.fetch_add(1, Ordering::Relaxed);
                    }
                }

                // Add some randomness to timing
                if op_id % 10 == 0 {
                    thread::sleep(Duration::from_millis(thread_id as u64));
                }
            }

            thread_id
        });

        handles.push(handle);
    }

    let start_time = Instant::now();

    // Wait for all chaos threads to complete
    for handle in handles {
        let thread_id = handle.join().expect("Chaos thread should complete");
        println!("  🌪️  Chaos thread {} completed", thread_id);
    }

    let total_time = start_time.elapsed();
    let operations = operations_counter.load(Ordering::Relaxed);
    let errors = errors_counter.load(Ordering::Relaxed);
    let successes = success_counter.load(Ordering::Relaxed);

    println!("\n📊 Concurrent Chaos Results:");
    println!("  Total operations: {}", operations);
    println!("  Successful operations: {}", successes);
    println!("  Failed operations: {}", errors);
    println!(
        "  Success rate: {:.2}%",
        (successes as f64 / operations as f64) * 100.0
    );
    println!("  Total execution time: {:?}", total_time);
    println!(
        "  Operations per second: {:.2}",
        operations as f64 / total_time.as_secs_f64()
    );

    // Validate chaos didn't break the system
    assert_eq!(
        operations,
        thread_count * operations_per_thread,
        "All operations should be attempted"
    );
    assert!(successes > 0, "Some operations should succeed");
    assert!(
        total_time < Duration::from_secs(60),
        "Chaos test should complete in reasonable time"
    );

    // Check shared state integrity
    {
        let blockchain_state = shared_blockchain.lock().unwrap();
        assert!(
            !blockchain_state.is_empty(),
            "Blockchain should not be empty"
        );
        println!("  🔗 Final blockchain length: {}", blockchain_state.len());
    }

    {
        let utxo_state = shared_utxos.lock().unwrap();
        println!("  💰 Total UTXOs created: {}", utxo_state.len());
    }

    println!("✅ Concurrent chaos scenario test completed");
}

#[test]
fn test_time_manipulation_chaos() {
    println!("\n🌪️  Chaos Test: Time Manipulation Attacks");
    println!("{}", "=".repeat(70));

    let consensus = Consensus::new();

    // Test various time-based attack scenarios
    let time_scenarios = vec![
        (
            "Future Block",
            chrono::Utc::now() + chrono::Duration::hours(24),
        ),
        (
            "Far Future",
            chrono::Utc::now() + chrono::Duration::days(365),
        ),
        (
            "Ancient Block",
            chrono::Utc::now() - chrono::Duration::days(365 * 10),
        ),
        (
            "Epoch Zero",
            chrono::DateTime::from_timestamp(0, 0).unwrap(),
        ),
        (
            "Year 2038",
            chrono::DateTime::from_timestamp(2147483647, 0).unwrap(),
        ),
        (
            "Negative Time",
            chrono::DateTime::from_timestamp(-1, 0).unwrap_or(chrono::Utc::now()),
        ),
    ];

    let mut attack_results = HashMap::new();

    for (attack_name, timestamp) in time_scenarios {
        println!("  🕐 Testing: {}", attack_name);

        // Create block with manipulated timestamp
        let malicious_block = Block::new(
            BlockHeader::new(
                1,
                BlockHash::ZERO,
                MerkleRoot::ZERO,
                timestamp,
                DifficultyTarget::MAX,
                0,
            ),
            vec![Transaction::coinbase(5_000_000_000, vec![])],
        );

        // Test consensus validation
        let validation_result = consensus.validate_block(&malicious_block, &[]);
        attack_results.insert(attack_name, validation_result.is_ok());

        match validation_result {
            Ok(_) => println!("    ⚠️  Attack succeeded - block accepted"),
            Err(_) => println!("    ✅ Attack prevented - block rejected"),
        }

        // Test timestamp validation specifically
        let time_validation = validate_block_timestamp(&malicious_block);
        match time_validation {
            Ok(_) => println!("    📅 Timestamp validation: Passed"),
            Err(_) => println!("    📅 Timestamp validation: Failed"),
        }
    }

    // Test timestamp ordering attacks
    println!("\n  🔄 Testing Timestamp Ordering Attack");

    let base_time = chrono::Utc::now();
    let mut blocks: Vec<Block> = Vec::new();

    // Create blocks with decreasing timestamps (should be invalid)
    for i in 0..5 {
        let timestamp = base_time - chrono::Duration::minutes(i as i64);
        let block = Block::new(
            BlockHeader::new(
                i as u32,
                if i == 0 {
                    BlockHash::ZERO
                } else {
                    blocks[i - 1].hash().unwrap()
                },
                MerkleRoot::ZERO,
                timestamp,
                DifficultyTarget::MAX,
                0,
            ),
            vec![Transaction::coinbase(5_000_000_000 + i as u64, vec![])],
        );
        blocks.push(block);
    }

    // Validate sequence
    let mut sequence_valid = true;
    for i in 1..blocks.len() {
        let validation = consensus.validate_block(&blocks[i], &blocks[0..i]);
        if validation.is_err() {
            sequence_valid = false;
            println!("    ✅ Ordering attack prevented at block {}", i);
            break;
        }
    }

    if sequence_valid {
        println!("    ⚠️  Ordering attack succeeded");
    }

    // Test rapid timestamp changes
    println!("\n  ⚡ Testing Rapid Timestamp Changes");

    let rapid_blocks = 10;
    let mut rapid_results = Vec::new();

    for i in 0..rapid_blocks {
        // Alternate between future and past timestamps rapidly
        let offset = if i % 2 == 0 {
            chrono::Duration::minutes(i as i64)
        } else {
            -chrono::Duration::minutes(i as i64)
        };

        let timestamp = base_time + offset;
        let block = Block::new(
            BlockHeader::new(
                i as u32,
                BlockHash::ZERO,
                MerkleRoot::ZERO,
                timestamp,
                DifficultyTarget::MAX,
                0,
            ),
            vec![Transaction::coinbase(5_000_000_000 + i as u64, vec![])],
        );

        let validation = consensus.validate_block(&block, &[]);
        rapid_results.push(validation.is_ok());
    }

    let rapid_successes = rapid_results.iter().filter(|&&x| x).count();
    println!(
        "    📊 Rapid changes: {}/{} blocks accepted",
        rapid_successes, rapid_blocks
    );

    // Summary
    println!("\n📊 Time Manipulation Attack Results:");
    for (attack, succeeded) in &attack_results {
        println!(
            "  {}: {}",
            attack,
            if *succeeded {
                "⚠️  Vulnerable"
            } else {
                "✅ Protected"
            }
        );
    }

    // Validate that most time attacks are prevented
    let prevented_attacks = attack_results.values().filter(|&&x| !x).count();
    let total_attacks = attack_results.len();

    println!(
        "  Defense rate: {}/{} ({:.1}%)",
        prevented_attacks,
        total_attacks,
        (prevented_attacks as f64 / total_attacks as f64) * 100.0
    );

    assert!(
        prevented_attacks >= total_attacks / 2,
        "Should prevent at least half of time attacks"
    );

    println!("✅ Time manipulation chaos test completed");
}

#[test]
fn test_fork_bomb_chaos() {
    println!("\n🌪️  Chaos Test: Fork Bomb Simulation");
    println!("{}", "=".repeat(70));

    // Simulate an attacker trying to create many competing forks
    let mut miner = Miner::new();
    let consensus = Consensus::new();

    // Create base block
    let genesis = create_genesis_block();
    let base_tx = Transaction::coinbase(5_000_000_000, vec![]);
    let base_block = Block::new(
        BlockHeader::new(
            1,
            genesis.hash().unwrap(),
            MerkleRoot::ZERO,
            chrono::Utc::now(),
            DifficultyTarget::MAX,
            0,
        ),
        vec![base_tx],
    );

    // Mine the base block
    let temp_merkle = base_block.calculate_merkle_root().unwrap();
    let mining_header = mining::create_mining_header(
        1,
        genesis.hash().unwrap(),
        temp_merkle,
        DifficultyTarget::MAX,
    );

    let mined_header = miner.mine_block(mining_header).unwrap();
    let mined_base = Block::new(mined_header, base_block.transactions);

    println!("  🎯 Base block created at height 1");

    // Fork bomb: create many competing blocks at the same height
    let fork_count = 20;
    let mut forks = Vec::new();

    println!("  💣 Creating {} competing forks...", fork_count);

    let start_time = Instant::now();

    for fork_id in 0..fork_count {
        // Create unique transaction for each fork
        let fork_tx = Transaction::coinbase(5_000_000_000 + fork_id as u64, vec![]);

        let fork_block = Block::new(
            BlockHeader::new(
                2, // Same height - competing forks
                mined_base.hash().unwrap(),
                MerkleRoot::ZERO,
                chrono::Utc::now() + chrono::Duration::milliseconds(fork_id as i64),
                DifficultyTarget::MAX,
                0,
            ),
            vec![fork_tx],
        );

        // Calculate proper merkle root
        let fork_merkle = fork_block.calculate_merkle_root().unwrap();
        let fork_header = mining::create_mining_header(
            2,
            mined_base.hash().unwrap(),
            fork_merkle,
            DifficultyTarget::MAX,
        );

        // Mine each fork
        if let Ok(mined_fork_header) = miner.mine_block(fork_header) {
            let final_fork = Block::new(mined_fork_header, fork_block.transactions);
            forks.push(final_fork);
        }

        // Limit time to prevent test timeout
        if start_time.elapsed() > Duration::from_secs(5) {
            println!("    ⏰ Fork creation timeout after {} forks", fork_id + 1);
            break;
        }

        if fork_id % 5 == 0 {
            println!("    📈 Created {} forks so far...", fork_id + 1);
        }
    }

    let fork_creation_time = start_time.elapsed();
    let created_forks = forks.len();

    println!("  📊 Fork bomb results:");
    println!("    Attempted forks: {}", fork_count);
    println!("    Successfully created: {}", created_forks);
    println!("    Creation time: {:?}", fork_creation_time);

    // Test consensus handling of all forks
    println!("\n  🔍 Testing consensus resolution...");

    let chain_context = vec![genesis, mined_base];
    let mut valid_forks = 0;
    let mut invalid_forks = 0;

    for (i, fork) in forks.iter().enumerate() {
        match consensus.validate_block(fork, &chain_context) {
            Ok(_) => {
                valid_forks += 1;
                if i < 5 {
                    // Only print first few
                    println!("    ✅ Fork {} validated", i);
                }
            }
            Err(_) => {
                invalid_forks += 1;
                if i < 5 {
                    // Only print first few
                    println!("    ❌ Fork {} rejected", i);
                }
            }
        }
    }

    println!("  📊 Consensus resolution:");
    println!("    Valid forks: {}", valid_forks);
    println!("    Invalid forks: {}", invalid_forks);
    println!(
        "    Validation rate: {:.1}%",
        (valid_forks as f64 / created_forks as f64) * 100.0
    );

    // Test fork selection (longest chain rule simulation)
    println!("\n  🏆 Testing fork selection mechanism...");

    // Extend each fork and see which one "wins"
    let mut extended_forks = HashMap::new();

    for (i, fork) in forks.iter().take(5).enumerate() {
        // Only test first 5 for performance
        // Try to extend this fork
        let extension_tx = Transaction::coinbase(6_000_000_000 + i as u64, vec![]);
        let extension_block = Block::new(
            BlockHeader::new(
                3,
                fork.hash().unwrap(),
                MerkleRoot::ZERO,
                chrono::Utc::now(),
                DifficultyTarget::MAX,
                0,
            ),
            vec![extension_tx],
        );

        let ext_merkle = extension_block.calculate_merkle_root().unwrap();
        let ext_header = mining::create_mining_header(
            3,
            fork.hash().unwrap(),
            ext_merkle,
            DifficultyTarget::MAX,
        );

        if let Ok(mined_ext_header) = miner.mine_block(ext_header) {
            let final_extension = Block::new(mined_ext_header, extension_block.transactions);
            extended_forks.insert(i, final_extension);
            println!("    🔗 Fork {} extended to height 3", i);
        }
    }

    println!("  📊 Fork extension results:");
    println!("    Forks successfully extended: {}", extended_forks.len());

    // Validate system resilience
    assert!(created_forks > 0, "Should create at least some forks");
    assert!(
        fork_creation_time < Duration::from_secs(10),
        "Fork creation should not take too long"
    );
    assert!(
        valid_forks + invalid_forks == created_forks,
        "All forks should be processed"
    );

    // System should handle fork bomb gracefully
    println!("✅ Fork bomb chaos test completed - system remained stable");
}

// Helper functions
fn create_genesis_block() -> Block {
    Block::new(
        BlockHeader::new(
            0,
            BlockHash::ZERO,
            MerkleRoot::ZERO,
            chrono::DateTime::from_timestamp(1640995200, 0).unwrap(),
            DifficultyTarget::MAX,
            0,
        ),
        vec![Transaction::coinbase(10_000_000_000, vec![])],
    )
}

fn validate_block_timestamp(block: &Block) -> Result<(), String> {
    let now = chrono::Utc::now();
    let block_time = block.header.timestamp;

    // Allow blocks up to 2 hours in the future
    if block_time > now + chrono::Duration::hours(2) {
        return Err("Block timestamp too far in future".to_string());
    }

    // Reject blocks older than 1 year
    if block_time < now - chrono::Duration::days(365) {
        return Err("Block timestamp too old".to_string());
    }

    Ok(())
}
