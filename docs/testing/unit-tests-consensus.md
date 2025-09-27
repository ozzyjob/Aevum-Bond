# Camada 1: Unit Tests - Lógica de Consenso

## 1.4 Bond Protocol - Proof of Work (PoW)

### Mining Algorithm Tests
```rust
#[cfg(test)]
mod bond_consensus_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn mine_block_finds_valid_nonce() {
        let mut miner = Miner::new();
        
        // Create easy target for testing
        let easy_target = DifficultyTarget([0xFF; 32]);
        let header = create_mining_header(1, BlockHash::ZERO, MerkleRoot::ZERO, easy_target);
        
        let result = miner.mine_block(header);
        assert!(result.is_ok());
        
        let mined_header = result.unwrap();
        assert!(mined_header.validates_pow().unwrap());
        assert_ne!(mined_header.nonce, 0); // Should have found a nonce
    }

    #[test]
    fn difficulty_target_validation() {
        let easy_target = DifficultyTarget([0xFF; 32]);
        let hard_target = DifficultyTarget([0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 
                                           0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                                           0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                                           0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        
        // Easy hash should meet easy target
        let easy_hash = BlockHash::from_bytes([0x7F; 32]);
        assert!(easy_hash.meets_difficulty_target(easy_target));
        
        // Hard hash should not meet hard target
        let hard_hash = BlockHash::from_bytes([0xFF; 32]);
        assert!(!hard_hash.meets_difficulty_target(hard_target));
    }

    #[test]
    fn difficulty_adjustment_algorithm() {
        let consensus = Consensus::new();
        let current_target = DifficultyTarget([0x7F; 32]);
        
        // Simulate blocks mined too quickly (2x faster than target)
        let mut blocks = Vec::new();
        let base_time = chrono::Utc::now();
        
        for i in 0..2016 {
            let block_time = base_time + chrono::Duration::seconds((i * 300) as i64); // 5 min instead of 10 min
            let header = BlockHeader::new(
                1,
                BlockHash::ZERO,
                MerkleRoot::ZERO,
                block_time,
                current_target,
                0
            );
            blocks.push(Block::new(header, vec![]));
        }
        
        let new_target = consensus.calculate_next_difficulty(&blocks, current_target).unwrap();
        
        // Target should decrease (harder difficulty) when blocks are mined too fast
        assert!(new_target < current_target);
    }

    #[test]
    fn mining_with_different_difficulties() {
        let difficulties = vec![
            DifficultyTarget([0xFF; 32]),                    // Very easy
            DifficultyTarget([0x0F; 32]),                    // Easy
            DifficultyTarget([0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]), // Medium
        ];
        
        for (i, difficulty) in difficulties.iter().enumerate() {
            let mut miner = Miner::new();
            let header = create_mining_header(1, BlockHash::ZERO, MerkleRoot::ZERO, *difficulty);
            
            let start_time = std::time::Instant::now();
            let result = miner.mine_block(header);
            let duration = start_time.elapsed();
            
            assert!(result.is_ok(), "Mining failed for difficulty level {}", i);
            
            let mined_header = result.unwrap();
            assert!(mined_header.validates_pow().unwrap());
            
            println!("Difficulty {}: Found nonce {} in {:?}", i, mined_header.nonce, duration);
        }
    }

    #[test]
    fn block_reward_calculation() {
        let consensus = Consensus::new();
        let block_height = 1000;
        
        // Test adaptive inflation calculation
        let high_hashrate = 1_000_000_000; // High security
        let low_hashrate = 100_000;        // Low security
        
        let reward_high = consensus.calculate_block_reward(block_height, high_hashrate);
        let reward_low = consensus.calculate_block_reward(block_height, low_hashrate);
        
        // Lower hashrate should result in higher inflation to incentivize mining
        assert!(reward_low > reward_high);
        
        // Rewards should be within expected bounds (1.84% - 3.72% annually)
        let min_reward = 1_840_000; // ~1.84% of 100M BND
        let max_reward = 3_720_000; // ~3.72% of 100M BND
        
        assert!(reward_high >= min_reward);
        assert!(reward_low <= max_reward);
    }
}
```

## 1.5 Aevum Protocol - Proof of Dedication (PoD)

### Dedication Score Calculation Tests
```rust
#[cfg(test)]
mod aevum_consensus_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn dedication_score_calculation() {
        let validator = ValidatorInfo {
            address: Address::generate(),
            stake_amount: 1_000_000,
            stake_duration: Duration::from_secs(86400 * 90), // 90 days
            uptime_ratio: 0.98,
            governance_votes: 25,
            total_possible_votes: 30,
        };
        
        let score = calculate_dedication_score(&validator);
        
        // Verify each component
        assert!(score.stake_weight > 0);
        assert!(score.time_commitment > 0);
        assert!(score.reliability > 0);
        assert!(score.engagement > 0);
        
        // Total score should be weighted combination
        let expected_total = (
            score.stake_weight * STAKE_WEIGHT_MULTIPLIER +
            score.time_commitment * TIME_MULTIPLIER +
            score.reliability * UPTIME_MULTIPLIER +
            score.engagement * GOVERNANCE_MULTIPLIER
        ) / 4;
        
        assert_eq!(score.total_score(), expected_total);
    }

    #[test]
    fn validator_selection_probability() {
        let validators = vec![
            create_validator(1_000_000, 30, 0.99, 20), // High performer
            create_validator(500_000, 15, 0.95, 10),   // Medium performer  
            create_validator(100_000, 5, 0.85, 2),     // Low performer
        ];
        
        let probabilities = calculate_selection_probabilities(&validators);
        
        // High performer should have highest probability
        assert!(probabilities[0] > probabilities[1]);
        assert!(probabilities[1] > probabilities[2]);
        
        // Sum should equal 1.0
        let sum: f64 = probabilities.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn stake_weight_linear_scaling() {
        let base_validator = create_validator(100_000, 30, 0.98, 15);
        let double_stake_validator = create_validator(200_000, 30, 0.98, 15);
        
        let base_score = calculate_dedication_score(&base_validator);
        let double_score = calculate_dedication_score(&double_stake_validator);
        
        // Stake weight should scale linearly
        assert_eq!(double_score.stake_weight, base_score.stake_weight * 2);
    }

    #[test]
    fn time_lock_bonus_calculation() {
        let short_lock = create_validator(1_000_000, 7, 0.98, 15);   // 1 week
        let medium_lock = create_validator(1_000_000, 30, 0.98, 15); // 1 month
        let long_lock = create_validator(1_000_000, 365, 0.98, 15);  // 1 year
        
        let short_score = calculate_dedication_score(&short_lock);
        let medium_score = calculate_dedication_score(&medium_lock);
        let long_score = calculate_dedication_score(&long_lock);
        
        // Longer time locks should have higher commitment scores
        assert!(medium_score.time_commitment > short_score.time_commitment);
        assert!(long_score.time_commitment > medium_score.time_commitment);
        
        // But with diminishing returns (logarithmic scaling)
        let short_to_medium_gain = medium_score.time_commitment - short_score.time_commitment;
        let medium_to_long_gain = long_score.time_commitment - medium_score.time_commitment;
        assert!(short_to_medium_gain > medium_to_long_gain);
    }

    #[test]
    fn uptime_penalty_calculation() {
        let perfect_uptime = create_validator(1_000_000, 30, 1.0, 15);   // 100%
        let good_uptime = create_validator(1_000_000, 30, 0.95, 15);     // 95%
        let poor_uptime = create_validator(1_000_000, 30, 0.80, 15);     // 80%
        
        let perfect_score = calculate_dedication_score(&perfect_uptime);
        let good_score = calculate_dedication_score(&good_uptime);
        let poor_score = calculate_dedication_score(&poor_uptime);
        
        assert!(perfect_score.reliability > good_score.reliability);
        assert!(good_score.reliability > poor_score.reliability);
        
        // Poor uptime should have significant penalty
        let penalty_ratio = poor_score.reliability / perfect_score.reliability;
        assert!(penalty_ratio < 0.85); // More than 15% penalty for 20% downtime
    }

    #[test]
    fn governance_participation_boost() {
        let non_participant = create_validator(1_000_000, 30, 0.98, 0);  // 0% participation
        let active_participant = create_validator(1_000_000, 30, 0.98, 20); // High participation
        
        let non_participant_score = calculate_dedication_score(&non_participant);
        let active_score = calculate_dedication_score(&active_participant);
        
        assert!(active_score.engagement > non_participant_score.engagement);
        
        // Governance participation should provide meaningful boost
        let total_boost = active_score.total_score() / non_participant_score.total_score();
        assert!(total_boost > 1.1); // At least 10% boost for active participation
    }

    proptest! {
        #[test]
        fn dedication_score_bounds_checking(
            stake in 1u64..10_000_000_000,
            days in 1u32..365,
            uptime in 0.5f64..1.0,
            votes in 0u32..100
        ) {
            let validator = create_validator(stake, days, uptime, votes);
            let score = calculate_dedication_score(&validator);
            
            // All components should be non-negative
            prop_assert!(score.stake_weight >= 0);
            prop_assert!(score.time_commitment >= 0);
            prop_assert!(score.reliability >= 0);
            prop_assert!(score.engagement >= 0);
            
            // Total score should be reasonable bounds
            prop_assert!(score.total_score() > 0);
            prop_assert!(score.total_score() < u64::MAX / 2); // Prevent overflow
        }
    }
}
```

### Consensus Algorithm Tests
```rust
#[cfg(test)]
mod aevum_pod_tests {
    #[test]
    fn validator_rotation_fairness() {
        let validators = create_test_validator_set(100);
        let mut selection_counts = vec![0; validators.len()];
        
        // Run 10000 selections
        for _ in 0..10000 {
            let selected = select_validator_by_pod(&validators);
            selection_counts[selected] += 1;
        }
        
        // Higher dedication score validators should be selected more often
        let high_score_validator = validators.iter()
            .enumerate()
            .max_by_key(|(_, v)| calculate_dedication_score(v).total_score())
            .unwrap();
        
        let low_score_validator = validators.iter()
            .enumerate()
            .min_by_key(|(_, v)| calculate_dedication_score(v).total_score())
            .unwrap();
        
        assert!(selection_counts[high_score_validator.0] > selection_counts[low_score_validator.0]);
        
        // But low score validators should still get some selections (fairness)
        assert!(selection_counts[low_score_validator.0] > 0);
    }

    #[test]
    fn block_finalization_threshold() {
        let validators = create_test_validator_set(21); // 21 validators for clear majority
        let mut signatures = Vec::new();
        
        // Collect signatures from majority of validators (2/3 + 1 = 15)
        for i in 0..15 {
            let signature = sign_block_with_validator(&validators[i], create_test_block());
            signatures.push(signature);
        }
        
        let finalization_result = check_block_finalization(&signatures, &validators);
        assert!(finalization_result.is_finalized);
        assert!(finalization_result.voting_power_ratio > 0.66);
        
        // Test with insufficient signatures (only 10 out of 21)
        signatures.truncate(10);
        let insufficient_result = check_block_finalization(&signatures, &validators);
        assert!(!insufficient_result.is_finalized);
    }
}
```
