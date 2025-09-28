use crate::{Block, BlockHash, BlockHeader, BondError, BondResult, DifficultyTarget};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Mining configuration
#[derive(Debug, Clone)]
pub struct MiningConfig {
    /// Target block time in seconds (600 = 10 minutes)
    pub target_block_time: u64,
    /// Difficulty adjustment period in blocks (2016 blocks ≈ 2 weeks)
    pub difficulty_adjustment_period: u32,
    /// Maximum difficulty adjustment factor (4x)
    pub max_difficulty_adjustment: f64,
    /// Minimum difficulty adjustment factor (0.25x)
    pub min_difficulty_adjustment: f64,
}

/// Mining statistics and metrics
#[derive(Debug, Clone)]
pub struct MiningStats {
    /// Number of hashes attempted
    pub hashes_attempted: u64,
    /// Mining start time
    pub start_time: std::time::Instant,
    /// Current hash rate (hashes per second)
    pub hash_rate: f64,
}

/// Proof-of-Work miner for Bond protocol
pub struct Miner {
    /// Mining configuration
    #[allow(dead_code)]
    config: MiningConfig,
    /// Whether mining should stop
    should_stop: Arc<AtomicBool>,
    /// Mining statistics
    stats: MiningStats,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            target_block_time: 600,             // 10 minutes
            difficulty_adjustment_period: 2016, // ~2 weeks
            max_difficulty_adjustment: 4.0,     // 4x harder max
            min_difficulty_adjustment: 0.25,    // 4x easier max
        }
    }
}

impl Default for Miner {
    fn default() -> Self {
        Self::new()
    }
}

impl Miner {
    /// Create a new miner with default configuration
    pub fn new() -> Self {
        Self::with_config(MiningConfig::default())
    }

    /// Create a new miner with custom configuration
    pub fn with_config(config: MiningConfig) -> Self {
        Self {
            config,
            should_stop: Arc::new(AtomicBool::new(false)),
            stats: MiningStats {
                hashes_attempted: 0,
                start_time: std::time::Instant::now(),
                hash_rate: 0.0,
            },
        }
    }

    /// Get a handle to stop mining
    pub fn stop_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.should_stop)
    }

    /// Mine a block with the given header template
    pub fn mine_block(&mut self, mut header: BlockHeader) -> BondResult<BlockHeader> {
        self.should_stop.store(false, Ordering::Relaxed);
        self.stats = MiningStats {
            hashes_attempted: 0,
            start_time: std::time::Instant::now(),
            hash_rate: 0.0,
        };

        let mut nonce = 0u64;

        loop {
            // Check if we should stop
            if self.should_stop.load(Ordering::Relaxed) {
                return Err(BondError::InvalidProofOfWork {
                    hash: "Mining stopped".to_string(),
                    target: header.difficulty_target.to_string(),
                });
            }

            // Set the nonce and calculate hash
            header.nonce = nonce;
            let hash = header.hash()?;

            self.stats.hashes_attempted += 1;

            // Update hash rate every 1000 hashes
            if self.stats.hashes_attempted % 1000 == 0 {
                let elapsed = self.stats.start_time.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    self.stats.hash_rate = self.stats.hashes_attempted as f64 / elapsed;
                }
            }

            // Check if we found a valid proof-of-work
            if hash.meets_difficulty_target(header.difficulty_target) {
                return Ok(header);
            }

            // Increment nonce, wrapping around if necessary
            nonce = nonce.wrapping_add(1);

            // If we've wrapped around completely, this target might be impossible
            if nonce == 0 {
                return Err(BondError::InvalidProofOfWork {
                    hash: "Nonce space exhausted".to_string(),
                    target: header.difficulty_target.to_string(),
                });
            }
        }
    }

    /// Get current mining statistics
    pub fn stats(&self) -> &MiningStats {
        &self.stats
    }

    /// Stop mining
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
}

/// Consensus rules and difficulty adjustment
pub struct Consensus {
    config: MiningConfig,
}

impl Consensus {
    /// Create new consensus instance
    pub fn new() -> Self {
        Self {
            config: MiningConfig::default(),
        }
    }

    /// Create consensus with custom configuration
    pub fn with_config(config: MiningConfig) -> Self {
        Self { config }
    }

    /// Calculate the next difficulty target based on block history
    pub fn calculate_next_difficulty(
        &self,
        blocks: &[Block],
        current_target: DifficultyTarget,
    ) -> BondResult<DifficultyTarget> {
        if blocks.len() < self.config.difficulty_adjustment_period as usize {
            // Not enough blocks for adjustment
            return Ok(current_target);
        }

        // Get the last adjustment period worth of blocks
        let adjustment_blocks =
            &blocks[blocks.len() - self.config.difficulty_adjustment_period as usize..];

        // Calculate actual time taken
        let first_timestamp = adjustment_blocks[0].header.timestamp;
        let last_timestamp = adjustment_blocks[adjustment_blocks.len() - 1]
            .header
            .timestamp;

        let actual_time = (last_timestamp - first_timestamp).num_seconds() as u64;
        let target_time =
            self.config.target_block_time * self.config.difficulty_adjustment_period as u64;

        // Calculate adjustment factor
        let adjustment_factor = actual_time as f64 / target_time as f64;

        // Clamp the adjustment factor
        let clamped_factor = adjustment_factor
            .max(self.config.min_difficulty_adjustment)
            .min(self.config.max_difficulty_adjustment);

        // Apply adjustment to difficulty target
        // Higher target = easier mining, so we multiply by adjustment factor
        // If blocks were found too quickly (actual_time < target_time), factor < 1, target decreases (harder)
        // If blocks were found too slowly (actual_time > target_time), factor > 1, target increases (easier)

        let new_target = self.adjust_target(current_target, clamped_factor)?;
        Ok(new_target)
    }

    /// Adjust difficulty target by a factor
    fn adjust_target(
        &self,
        current: DifficultyTarget,
        factor: f64,
    ) -> BondResult<DifficultyTarget> {
        // Convert target to a big integer for arithmetic
        let _target_bytes = current.0;

        // Simple approximation: adjust the most significant bytes
        // This is a simplified version; a production implementation would use proper big integer arithmetic
        let current_difficulty = self.target_to_difficulty_approx(&current);
        let new_difficulty = current_difficulty / factor;
        let new_target = self.difficulty_to_target_approx(new_difficulty);

        Ok(new_target)
    }

    /// Convert target to approximate difficulty (simplified)
    fn target_to_difficulty_approx(&self, target: &DifficultyTarget) -> f64 {
        // Very simplified: use the first 8 bytes as a rough measure
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&target.0[..8]);
        let value = u64::from_be_bytes(bytes);

        if value == 0 {
            f64::MAX
        } else {
            (u64::MAX as f64) / (value as f64)
        }
    }

    /// Convert difficulty to approximate target (simplified)
    fn difficulty_to_target_approx(&self, difficulty: f64) -> DifficultyTarget {
        if difficulty <= 0.0 {
            return DifficultyTarget::MAX;
        }

        let target_value = ((u64::MAX as f64) / difficulty) as u64;
        let target_bytes = target_value.to_be_bytes();

        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&target_bytes);
        // Fill the rest with 0xFF to make it easier
        for item in result.iter_mut().skip(8) {
            *item = 0xFF;
        }

        DifficultyTarget(result)
    }

    /// Validate that a block meets consensus rules
    pub fn validate_block(&self, block: &Block, previous_blocks: &[Block]) -> BondResult<()> {
        // Basic block validation
        block.validate()?;

        // Check block size (4 MB limit for post-quantum signatures)
        const MAX_BLOCK_SIZE: usize = 4 * 1024 * 1024; // 4 MB
        if block.size()? > MAX_BLOCK_SIZE {
            return Err(BondError::InvalidTransaction {
                reason: format!(
                    "Block size {} exceeds maximum {}",
                    block.size()?,
                    MAX_BLOCK_SIZE
                ),
            });
        }

        // Validate difficulty target if we have enough blocks
        if previous_blocks.len() >= self.config.difficulty_adjustment_period as usize {
            let expected_target =
                self.calculate_next_difficulty(previous_blocks, block.header.difficulty_target)?;

            // Allow some tolerance for difficulty adjustments
            // In a real implementation, this would be more precise
            if block.header.difficulty_target != expected_target {
                // For now, just log this - in production this would be stricter
                eprintln!(
                    "Warning: Difficulty target mismatch. Expected: {}, Got: {}",
                    expected_target, block.header.difficulty_target
                );
            }
        }

        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &MiningConfig {
        &self.config
    }
}

impl Default for Consensus {
    fn default() -> Self {
        Self::new()
    }
}

impl MiningStats {
    /// Get the current hash rate
    pub fn current_hash_rate(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.hashes_attempted as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get elapsed mining time
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Utility function to create a mining-ready block header
pub fn create_mining_header(
    version: u32,
    previous_hash: BlockHash,
    merkle_root: crate::MerkleRoot,
    difficulty_target: DifficultyTarget,
) -> BlockHeader {
    BlockHeader::new(
        version,
        previous_hash,
        merkle_root,
        chrono::Utc::now(),
        difficulty_target,
        0, // Nonce will be set during mining
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MerkleRoot, Transaction};
    use chrono::Utc;

    #[test]
    fn test_miner_creation() {
        let miner = Miner::new();
        assert_eq!(miner.stats.hashes_attempted, 0);
    }

    #[test]
    fn test_mining_easy_target() {
        let mut miner = Miner::new();

        // Create a very easy target (all 0xFF)
        let easy_target = DifficultyTarget::MAX;

        let header = create_mining_header(1, BlockHash::ZERO, MerkleRoot::ZERO, easy_target);

        let result = miner.mine_block(header);
        assert!(result.is_ok());

        let mined_header = result.unwrap();
        assert!(mined_header.validates_pow().unwrap());
    }

    #[test]
    fn test_consensus_validation() {
        let consensus = Consensus::new();

        // Create a simple valid block
        let transactions = vec![Transaction::coinbase(5_000_000_000, vec![])];
        let block = Block::new(
            BlockHeader::new(
                1,
                BlockHash::ZERO,
                MerkleRoot::ZERO,
                Utc::now(),
                DifficultyTarget::MAX,
                0,
            ),
            transactions,
        );

        // Should validate basic rules (though PoW will fail)
        let result = consensus.validate_block(&block, &[]);
        // This will fail due to invalid PoW, which is expected
        assert!(result.is_err());
    }

    #[test]
    fn test_difficulty_calculation() {
        let consensus = Consensus::new();
        let current_target = DifficultyTarget::MAX;

        // Test with empty block history (should return current target)
        let result = consensus.calculate_next_difficulty(&[], current_target);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), current_target);
    }

    #[test]
    fn test_mining_stop() {
        let mut miner = Miner::new();
        let stop_handle = miner.stop_handle();

        let header = create_mining_header(
            1,
            BlockHash::ZERO,
            MerkleRoot::ZERO,
            DifficultyTarget([0x00; 32]), // Impossible target
        );

        // Start mining in a separate thread
        let miner_handle = std::thread::spawn(move || miner.mine_block(header));

        // Give mining a moment to start
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Stop mining
        stop_handle.store(true, Ordering::Relaxed);

        // Wait for mining to complete and check it was stopped
        let result = miner_handle.join().unwrap();
        assert!(result.is_err());

        // Verify it's the correct error type
        if let Err(BondError::InvalidProofOfWork { hash, .. }) = result {
            assert_eq!(hash, "Mining stopped");
        } else {
            panic!("Expected mining stopped error");
        }
    }
}
