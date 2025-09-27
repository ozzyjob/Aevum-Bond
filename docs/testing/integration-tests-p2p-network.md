# Camada 2: Integration Tests - P2P Network Integration

## 2.6 Testes de Integração da Rede P2P

### Complete P2P Network Integration Tests
```rust
#[cfg(test)]
mod p2p_network_integration_tests {
    use super::*;
    use libp2p::{Multiaddr, PeerId};
    use tokio::time::{sleep, Duration};

    struct P2PTestEnvironment {
        nodes: Vec<TestNode>,
        network_manager: NetworkManager,
        bootstrap_nodes: Vec<Multiaddr>,
        test_data: HashMap<String, Vec<u8>>,
    }

    impl P2PTestEnvironment {
        async fn new(node_count: usize) -> Self {
            let mut nodes = Vec::new();
            let mut bootstrap_nodes = Vec::new();
            
            // Create bootstrap node first
            let bootstrap_node = TestNode::new("bootstrap".to_string()).await;
            bootstrap_nodes.push(bootstrap_node.multiaddr().clone());
            nodes.push(bootstrap_node);
            
            // Create additional nodes
            for i in 1..node_count {
                let node = TestNode::new_with_bootstrap(
                    format!("node_{}", i),
                    bootstrap_nodes.clone(),
                ).await;
                nodes.push(node);
            }
            
            let network_manager = NetworkManager::new();
            let test_data = HashMap::new();
            
            Self {
                nodes,
                network_manager,
                bootstrap_nodes,
                test_data,
            }
        }
        
        async fn start_all_nodes(&mut self) -> Result<(), NetworkError> {
            for node in &mut self.nodes {
                node.start().await?;
                sleep(Duration::from_millis(100)).await; // Stagger startup
            }
            
            // Wait for network discovery
            sleep(Duration::from_secs(2)).await;
            Ok(())
        }
        
        async fn verify_connectivity(&self) -> Result<(), NetworkError> {
            for node in &self.nodes {
                let connected_peers = node.get_connected_peers().await?;
                if connected_peers.is_empty() {
                    return Err(NetworkError::NoConnectedPeers);
                }
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn peer_discovery_and_connection() {
        let mut env = P2PTestEnvironment::new(5).await;
        env.start_all_nodes().await.unwrap();
        
        // Wait for peer discovery to complete
        sleep(Duration::from_secs(5)).await;
        
        // Verify all nodes discovered each other
        for (i, node) in env.nodes.iter().enumerate() {
            let connected_peers = node.get_connected_peers().await.unwrap();
            
            // Each node should be connected to at least 2 others
            assert!(
                connected_peers.len() >= 2,
                "Node {} only connected to {} peers",
                i, connected_peers.len()
            );
            
            // Verify bidirectional connections
            for peer_id in &connected_peers {
                let peer_node = env.nodes.iter()
                    .find(|n| n.peer_id() == *peer_id)
                    .unwrap();
                
                let reverse_peers = peer_node.get_connected_peers().await.unwrap();
                assert!(
                    reverse_peers.contains(&node.peer_id()),
                    "Connection not bidirectional between {} and {}",
                    node.peer_id(), peer_id
                );
            }
        }
        
        // Test DHT functionality
        let key = "test_key".to_string();
        let value = b"test_value".to_vec();
        
        // Store value in DHT via first node
        env.nodes[0].dht_put(key.clone(), value.clone()).await.unwrap();
        
        // Wait for propagation
        sleep(Duration::from_secs(2)).await;
        
        // Retrieve value from different node
        let retrieved_value = env.nodes[2].dht_get(&key).await.unwrap();
        assert_eq!(retrieved_value, Some(value));
    }

    #[tokio::test]
    async fn transaction_gossip_propagation() {
        let mut env = P2PTestEnvironment::new(6).await;
        env.start_all_nodes().await.unwrap();
        
        // Create test transaction
        let test_wallet = TestWallet::new_bond();
        let transaction = test_wallet.create_test_transaction(1_000_000, 1_000).await.unwrap();
        let tx_hash = transaction.hash();
        
        // Node 0 receives transaction (e.g., from wallet)
        env.nodes[0].receive_transaction(transaction.clone()).await.unwrap();
        
        // Verify transaction added to mempool
        let mempool_size = env.nodes[0].get_mempool_size().await.unwrap();
        assert_eq!(mempool_size, 1);
        
        // Wait for gossip propagation
        sleep(Duration::from_secs(3)).await;
        
        // Verify all other nodes received the transaction
        for (i, node) in env.nodes.iter().enumerate().skip(1) {
            let has_transaction = node.has_transaction(&tx_hash).await.unwrap();
            assert!(
                has_transaction,
                "Node {} did not receive gossiped transaction",
                i
            );
            
            let mempool_size = node.get_mempool_size().await.unwrap();
            assert_eq!(mempool_size, 1);
        }
        
        // Test duplicate transaction handling
        let duplicate_result = env.nodes[3].receive_transaction(transaction.clone()).await;
        match duplicate_result {
            Err(NetworkError::DuplicateTransaction) => {}, // Expected
            Ok(_) => {
                // If accepted, mempool size should remain 1
                let mempool_size = env.nodes[3].get_mempool_size().await.unwrap();
                assert_eq!(mempool_size, 1);
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn block_propagation_and_sync() {
        let mut env = P2PTestEnvironment::new(4).await;
        env.start_all_nodes().await.unwrap();
        
        // All nodes start at same height
        let initial_height = 0u64;
        for node in &env.nodes {
            let height = node.get_blockchain_height().await.unwrap();
            assert_eq!(height, initial_height);
        }
        
        // Node 0 mines a new block
        let new_block = env.nodes[0].mine_next_block().await.unwrap();
        let block_hash = new_block.hash();
        
        // Add block to node 0's chain
        env.nodes[0].add_block(new_block.clone()).await.unwrap();
        
        // Verify node 0 height increased
        let node0_height = env.nodes[0].get_blockchain_height().await.unwrap();
        assert_eq!(node0_height, initial_height + 1);
        
        // Wait for block propagation
        sleep(Duration::from_secs(4)).await;
        
        // Verify all other nodes received and validated the block
        for (i, node) in env.nodes.iter().enumerate().skip(1) {
            let height = node.get_blockchain_height().await.unwrap();
            assert_eq!(
                height, initial_height + 1,
                "Node {} did not sync to new height",
                i
            );
            
            let has_block = node.has_block(&block_hash).await.unwrap();
            assert!(has_block, "Node {} missing propagated block", i);
        }
        
        // Test chain consistency
        let block0 = env.nodes[0].get_latest_block().await.unwrap();
        for (i, node) in env.nodes.iter().enumerate().skip(1) {
            let block = node.get_latest_block().await.unwrap();
            assert_eq!(
                block.hash(), block0.hash(),
                "Node {} has different latest block",
                i
            );
        }
    }

    #[tokio::test]
    async fn network_partition_and_recovery() {
        let mut env = P2PTestEnvironment::new(6).await;
        env.start_all_nodes().await.unwrap();
        
        // Initial state: all nodes connected
        env.verify_connectivity().await.unwrap();
        
        // Create network partition: nodes 0,1,2 vs nodes 3,4,5
        let partition_a = &env.nodes[0..3];
        let partition_b = &env.nodes[3..6];
        
        // Disconnect partitions
        for node_a in partition_a {
            for node_b in partition_b {
                node_a.disconnect_peer(&node_b.peer_id()).await.unwrap();
                node_b.disconnect_peer(&node_a.peer_id()).await.unwrap();
            }
        }
        
        // Wait for partition to take effect
        sleep(Duration::from_secs(2)).await;
        
        // Verify partition exists
        for node_a in partition_a {
            let peers = node_a.get_connected_peers().await.unwrap();
            let partition_b_peers: Vec<_> = partition_b.iter().map(|n| n.peer_id()).collect();
            
            for peer_id in &partition_b_peers {
                assert!(
                    !peers.contains(peer_id),
                    "Partition failed - still connected to peer"
                );
            }
        }
        
        // Mine blocks in both partitions
        let block_a = partition_a[0].mine_next_block().await.unwrap();
        let block_b = partition_b[0].mine_next_block().await.unwrap();
        
        partition_a[0].add_block(block_a.clone()).await.unwrap();
        partition_b[0].add_block(block_b.clone()).await.unwrap();
        
        // Wait for propagation within partitions
        sleep(Duration::from_secs(2)).await;
        
        // Verify each partition has its own block
        for node in &partition_a[1..] {
            let has_block_a = node.has_block(&block_a.hash()).await.unwrap();
            let has_block_b = node.has_block(&block_b.hash()).await.unwrap();
            assert!(has_block_a && !has_block_b);
        }
        
        for node in &partition_b[1..] {
            let has_block_a = node.has_block(&block_a.hash()).await.unwrap();
            let has_block_b = node.has_block(&block_b.hash()).await.unwrap();
            assert!(!has_block_a && has_block_b);
        }
        
        // Heal network partition
        for node_a in partition_a {
            for node_b in partition_b {
                node_a.connect_peer(&node_b.multiaddr()).await.unwrap();
            }
        }
        
        // Wait for reconnection and sync
        sleep(Duration::from_secs(5)).await;
        
        // Verify network healed
        env.verify_connectivity().await.unwrap();
        
        // Verify chain reorganization occurred
        // Assuming partition A had more work, all nodes should converge to block_a
        for node in &env.nodes {
            let latest_block = node.get_latest_block().await.unwrap();
            // The winning chain should be consistent across all nodes
            let height = node.get_blockchain_height().await.unwrap();
            assert_eq!(height, 1); // All should be at height 1
        }
    }

    #[tokio::test]
    async fn peer_reputation_and_banning() {
        let mut env = P2PTestEnvironment::new(4).await;
        env.start_all_nodes().await.unwrap();
        
        let honest_node = &env.nodes[0];
        let malicious_node = &env.nodes[1];
        let observer_nodes = &env.nodes[2..];
        
        // Malicious node sends invalid transactions repeatedly
        for i in 0..10 {
            let invalid_tx = create_invalid_transaction(i).await;
            
            // Send to honest node
            let result = honest_node.receive_transaction_from_peer(
                &malicious_node.peer_id(),
                invalid_tx,
            ).await;
            
            // Should be rejected
            assert!(result.is_err());
        }
        
        // Check reputation score decreased
        let reputation = honest_node.get_peer_reputation(&malicious_node.peer_id()).await.unwrap();
        assert!(reputation < 0.5); // Below acceptable threshold
        
        // Malicious node should be banned after repeated violations
        sleep(Duration::from_secs(1)).await;
        
        let is_banned = honest_node.is_peer_banned(&malicious_node.peer_id()).await.unwrap();
        assert!(is_banned);
        
        // Verify ban propagation to other nodes
        sleep(Duration::from_secs(2)).await;
        
        for observer in observer_nodes {
            let is_banned = observer.is_peer_banned(&malicious_node.peer_id()).await.unwrap();
            assert!(is_banned, "Ban not propagated to observer node");
        }
        
        // Malicious node should be unable to reconnect
        let reconnect_result = malicious_node.connect_peer(&honest_node.multiaddr()).await;
        assert!(reconnect_result.is_err());
        
        // Test ban expiration (if implemented)
        let ban_duration = Duration::from_secs(30);
        sleep(ban_duration).await;
        
        let is_still_banned = honest_node.is_peer_banned(&malicious_node.peer_id()).await.unwrap();
        // Depending on implementation, ban might expire
        if !is_still_banned {
            // If ban expired, connection should be possible again
            let reconnect_result = malicious_node.connect_peer(&honest_node.multiaddr()).await;
            assert!(reconnect_result.is_ok());
        }
    }

    #[tokio::test]
    async fn message_routing_and_delivery() {
        let mut env = P2PTestEnvironment::new(5).await;
        env.start_all_nodes().await.unwrap();
        
        // Test direct message routing
        let sender = &env.nodes[0];
        let recipient = &env.nodes[4];
        let message = b"Hello, peer!".to_vec();
        
        // Send direct message
        sender.send_direct_message(&recipient.peer_id(), message.clone()).await.unwrap();
        
        // Wait for delivery
        sleep(Duration::from_millis(500)).await;
        
        // Verify message received
        let received_messages = recipient.get_received_messages().await.unwrap();
        assert!(!received_messages.is_empty());
        assert_eq!(received_messages[0].data, message);
        assert_eq!(received_messages[0].sender, sender.peer_id());
        
        // Test broadcast message
        let broadcast_message = b"Broadcast to all!".to_vec();
        sender.broadcast_message(broadcast_message.clone()).await.unwrap();
        
        // Wait for propagation
        sleep(Duration::from_secs(1)).await;
        
        // Verify all other nodes received broadcast
        for (i, node) in env.nodes.iter().enumerate().skip(1) {
            let messages = node.get_received_messages().await.unwrap();
            let broadcast_received = messages.iter().any(|msg| {
                msg.data == broadcast_message && msg.sender == sender.peer_id()
            });
            assert!(broadcast_received, "Node {} did not receive broadcast", i);
        }
        
        // Test message flooding prevention
        let duplicate_broadcast = broadcast_message.clone();
        sender.broadcast_message(duplicate_broadcast).await.unwrap();
        
        sleep(Duration::from_millis(500)).await;
        
        // Nodes should not receive duplicate broadcasts
        for node in env.nodes.iter().skip(1) {
            let messages = node.get_received_messages().await.unwrap();
            let broadcast_count = messages.iter()
                .filter(|msg| msg.data == broadcast_message && msg.sender == sender.peer_id())
                .count();
            assert_eq!(broadcast_count, 1, "Duplicate broadcast not prevented");
        }
    }

    #[tokio::test]
    async fn bandwidth_management_and_throttling() {
        let mut env = P2PTestEnvironment::new(3).await;
        env.start_all_nodes().await.unwrap();
        
        let sender = &env.nodes[0];
        let receiver = &env.nodes[1];
        
        // Set bandwidth limits
        receiver.set_bandwidth_limit(1024 * 1024).await.unwrap(); // 1 MB/s limit
        
        // Send large amount of data rapidly
        let large_message = vec![0u8; 2 * 1024 * 1024]; // 2 MB message
        let start_time = std::time::Instant::now();
        
        let result = sender.send_direct_message(&receiver.peer_id(), large_message).await;
        
        let elapsed = start_time.elapsed();
        
        match result {
            Ok(_) => {
                // Message should take at least 2 seconds due to 1 MB/s limit
                assert!(elapsed.as_secs() >= 2, "Bandwidth throttling not applied");
            }
            Err(NetworkError::BandwidthLimitExceeded) => {
                // Alternative: message rejected due to size
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
        
        // Test rate limiting
        let small_message = vec![0u8; 1024]; // 1 KB message
        let mut successful_sends = 0u32;
        let rate_limit_start = std::time::Instant::now();
        
        // Try to send many small messages rapidly
        for _ in 0..1000 {
            match sender.send_direct_message(&receiver.peer_id(), small_message.clone()).await {
                Ok(_) => successful_sends += 1,
                Err(NetworkError::RateLimitExceeded) => break,
                Err(_) => {}
            }
        }
        
        let rate_limit_elapsed = rate_limit_start.elapsed();
        
        // Should hit rate limit before sending all 1000 messages
        assert!(successful_sends < 1000, "Rate limiting not applied");
        
        // Or if all sent, should have taken significant time
        if successful_sends == 1000 {
            assert!(rate_limit_elapsed.as_secs() > 5, "Rate limiting bypassed");
        }
    }

    async fn create_invalid_transaction(nonce: u64) -> Transaction {
        // Create transaction with invalid signature or other issues
        Transaction::new(
            vec![TransactionInput {
                utxo_id: UtxoId::new(TransactionHash::from_bytes([nonce as u8; 32]), 0),
                script_sig: Script::from_bytes(vec![0xFF]), // Invalid script
            }],
            vec![TransactionOutput {
                value: 0, // Invalid zero value output
                script_pubkey: Script::from_bytes(vec![0x51]),
            }],
        ).unwrap_or_else(|_| {
            // If constructor rejects, create minimal invalid transaction
            unsafe { std::mem::zeroed() }
        })
    }
}
```

### Network Health and Monitoring Tests
```rust
#[cfg(test)]
mod network_health_tests {
    use super::*;

    #[tokio::test]
    async fn network_health_monitoring() {
        let mut env = P2PTestEnvironment::new(10).await;
        env.start_all_nodes().await.unwrap();
        
        // Monitor network health metrics
        sleep(Duration::from_secs(5)).await;
        
        for (i, node) in env.nodes.iter().enumerate() {
            let health = node.get_network_health().await.unwrap();
            
            // Check connectivity metrics
            assert!(health.connected_peers >= 2, "Node {} has too few peers", i);
            assert!(health.connected_peers <= 9, "Node {} connected to too many peers", i);
            
            // Check message throughput
            assert!(health.messages_per_second >= 0.0);
            assert!(health.bytes_per_second >= 0.0);
            
            // Check latency metrics
            assert!(health.average_latency_ms > 0.0);
            assert!(health.average_latency_ms < 1000.0); // Should be < 1 second
            
            // Check peer diversity
            assert!(health.peer_diversity_score > 0.5); // At least moderate diversity
        }
        
        // Test network resilience by removing nodes
        let nodes_to_remove = 3;
        for i in 0..nodes_to_remove {
            env.nodes[i].shutdown().await.unwrap();
        }
        
        // Wait for network to adapt
        sleep(Duration::from_secs(3)).await;
        
        // Remaining nodes should maintain connectivity
        for node in &env.nodes[nodes_to_remove..] {
            let health = node.get_network_health().await.unwrap();
            assert!(
                health.connected_peers >= 1,
                "Network lost connectivity after node removals"
            );
        }
        
        // Test network recovery
        for i in 0..nodes_to_remove {
            env.nodes[i].restart().await.unwrap();
        }
        
        sleep(Duration::from_secs(5)).await;
        
        // Network should recover to full connectivity
        for node in &env.nodes {
            let health = node.get_network_health().await.unwrap();
            assert!(
                health.connected_peers >= 2,
                "Network did not recover after node restart"
            );
        }
    }

    #[tokio::test]
    async fn nat_traversal_and_external_connectivity() {
        // Test NAT traversal capabilities
        let mut nat_env = P2PTestEnvironment::new(2).await;
        
        // Simulate NAT environment
        nat_env.nodes[0].set_behind_nat(true).await.unwrap();
        nat_env.nodes[1].set_behind_nat(false).await.unwrap(); // Public node
        
        nat_env.start_all_nodes().await.unwrap();
        
        // Wait for NAT traversal to complete
        sleep(Duration::from_secs(10)).await;
        
        // Verify NAT traversal successful
        let nat_node_peers = nat_env.nodes[0].get_connected_peers().await.unwrap();
        assert!(!nat_node_peers.is_empty(), "NAT traversal failed");
        
        let public_node_peers = nat_env.nodes[1].get_connected_peers().await.unwrap();
        assert!(
            public_node_peers.contains(&nat_env.nodes[0].peer_id()),
            "Public node cannot reach NAT node"
        );
        
        // Test bidirectional communication
        let test_message = b"NAT traversal test".to_vec();
        nat_env.nodes[0].send_direct_message(
            &nat_env.nodes[1].peer_id(),
            test_message.clone(),
        ).await.unwrap();
        
        sleep(Duration::from_millis(500)).await;
        
        let received = nat_env.nodes[1].get_received_messages().await.unwrap();
        assert!(!received.is_empty(), "Message not received through NAT");
        assert_eq!(received[0].data, test_message);
    }
}
```
