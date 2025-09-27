# Camada 2: Integration Tests - Staking e Rewards

## 2.5 Testes de Integração do Sistema de Staking e Recompensas

### Complete Staking Lifecycle Tests
```rust
#[cfg(test)]
mod staking_rewards_integration_tests {
    use super::*;
    use std::collections::HashMap;

    struct StakingTestEnvironment {
        aevum_node: TestAevumNode,
        staking_system: StakingSystem,
        rewards_calculator: RewardsCalculator,
        test_stakers: Vec<TestStaker>,
        validators: Vec<TestValidator>,
        reward_pool: RewardPool,
    }

    impl StakingTestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let aevum_node = TestAevumNode::new(temp_dir.path()).await;
            let staking_system = StakingSystem::new(aevum_node.clone());
            let rewards_calculator = RewardsCalculator::new();
            let reward_pool = RewardPool::new();
            
            // Create test stakers with different stake amounts
            let mut test_stakers = vec![];
            let stake_amounts = vec![5_000_000, 3_000_000, 2_000_000, 1_000_000, 500_000]; // AEV amounts
            
            for (i, stake_amount) in stake_amounts.iter().enumerate() {
                let staker = TestStaker::new(format!("staker_{}", i));
                aevum_node.fund_address(&staker.address(), stake_amount + 100_000).await.unwrap();
                test_stakers.push(staker);
            }
            
            // Create validators
            let mut validators = vec![];
            for i in 0..3 {
                let validator = TestValidator::new(format!("validator_{}", i));
                aevum_node.fund_address(&validator.address(), 10_000_000).await.unwrap();
                validator.stake_as_validator(10_000_000, &aevum_node).await.unwrap();
                validators.push(validator);
            }
            
            Self {
                aevum_node,
                staking_system,
                rewards_calculator,
                test_stakers,
                validators,
                reward_pool,
            }
        }
    }

    #[tokio::test]
    async fn complete_staking_and_reward_cycle() {
        let mut env = StakingTestEnvironment::new().await;
        
        let staker = &env.test_stakers[0]; // 5M AEV staker
        let validator = &env.validators[0];
        let stake_amount = 3_000_000u64; // Stake 3M AEV
        
        // Record initial balances
        let initial_balance = env.aevum_node
            .get_balance(&staker.address())
            .await
            .unwrap();
        
        let initial_validator_power = env.aevum_node
            .get_validator_voting_power(&validator.address())
            .await
            .unwrap();
        
        // Phase 1: Stake tokens
        let stake_tx = env.staking_system.create_stake_transaction(
            staker,
            &validator.address(),
            stake_amount,
            StakingPeriod::OneYear, // 365 days lock period
        ).await.unwrap();
        
        let stake_tx_hash = env.aevum_node
            .submit_transaction(stake_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify staking recorded
        let stake_info = env.staking_system
            .get_stake_info(&staker.address(), &validator.address())
            .await
            .unwrap();
        
        assert_eq!(stake_info.amount, stake_amount);
        assert_eq!(stake_info.validator, validator.address());
        assert_eq!(stake_info.lock_period, StakingPeriod::OneYear);
        assert!(stake_info.unlock_time > Utc::now());
        
        // Verify balance reduced
        let post_stake_balance = env.aevum_node
            .get_balance(&staker.address())
            .await
            .unwrap();
        assert!(post_stake_balance < initial_balance - stake_amount);
        
        // Verify validator power increased
        let updated_validator_power = env.aevum_node
            .get_validator_voting_power(&validator.address())
            .await
            .unwrap();
        assert!(updated_validator_power > initial_validator_power);
        
        // Phase 2: Generate rewards over time
        let reward_periods = 10; // 10 reward periods
        let mut total_rewards = 0u64;
        
        for period in 0..reward_periods {
            // Advance time to next reward period (e.g., weekly)
            env.aevum_node.advance_time_by(chrono::Duration::days(7)).await.unwrap();
            env.aevum_node.advance_blocks(100).await.unwrap(); // Generate blocks for rewards
            
            // Calculate rewards for this period
            let period_rewards = env.rewards_calculator
                .calculate_staking_rewards(
                    &staker.address(),
                    &validator.address(),
                    period as u64,
                )
                .await
                .unwrap();
            
            total_rewards += period_rewards.amount;
            
            // Verify rewards accumulating
            let accumulated_rewards = env.staking_system
                .get_accumulated_rewards(&staker.address(), &validator.address())
                .await
                .unwrap();
            
            assert!(accumulated_rewards >= total_rewards);
        }
        
        // Phase 3: Claim rewards
        let claim_tx = env.staking_system.create_claim_rewards_transaction(
            staker,
            &validator.address(),
        ).await.unwrap();
        
        let claim_tx_hash = env.aevum_node
            .submit_transaction(claim_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify rewards claimed
        let post_claim_balance = env.aevum_node
            .get_balance(&staker.address())
            .await
            .unwrap();
        
        assert!(post_claim_balance > post_stake_balance);
        
        let claimed_rewards = post_claim_balance - post_stake_balance;
        assert!(claimed_rewards > 0);
        assert!(claimed_rewards <= total_rewards + 1000); // Allow for small rounding differences
        
        // Phase 4: Attempt early unstaking (should fail)
        let early_unstake_tx = env.staking_system.create_unstake_transaction(
            staker,
            &validator.address(),
            stake_amount / 2, // Try to unstake half
        ).await.unwrap();
        
        let early_unstake_result = env.aevum_node
            .validate_transaction(&early_unstake_tx)
            .await;
        
        // Should fail due to lock period
        assert!(early_unstake_result.is_err());
        match early_unstake_result.unwrap_err() {
            AevumError::StakeStillLocked { unlock_time } => {
                assert!(unlock_time > Utc::now());
            }
            _ => panic!("Expected StakeStillLocked error"),
        }
        
        // Phase 5: Wait for lock period to expire
        let stake_info = env.staking_system
            .get_stake_info(&staker.address(), &validator.address())
            .await
            .unwrap();
        
        env.aevum_node.advance_time_to(stake_info.unlock_time + chrono::Duration::hours(1)).await.unwrap();
        
        // Phase 6: Successful unstaking
        let unstake_tx = env.staking_system.create_unstake_transaction(
            staker,
            &validator.address(),
            stake_amount,
        ).await.unwrap();
        
        let unstake_tx_hash = env.aevum_node
            .submit_transaction(unstake_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(5).await.unwrap();
        
        // Verify unstaking successful
        let final_balance = env.aevum_node
            .get_balance(&staker.address())
            .await
            .unwrap();
        
        assert!(final_balance >= post_claim_balance + stake_amount - 10000); // Account for fees
        
        // Verify stake removed
        let final_stake_info = env.staking_system
            .get_stake_info(&staker.address(), &validator.address())
            .await;
        
        match final_stake_info {
            Ok(info) => assert_eq!(info.amount, 0),
            Err(_) => {}, // Stake completely removed
        }
        
        // Verify validator power decreased
        let final_validator_power = env.aevum_node
            .get_validator_voting_power(&validator.address())
            .await
            .unwrap();
        assert!(final_validator_power < updated_validator_power);
    }

    #[tokio::test]
    async fn multiple_stakers_reward_distribution() {
        let mut env = StakingTestEnvironment::new().await;
        
        let validator = &env.validators[0];
        let stakers = &env.test_stakers[0..3]; // Use first 3 stakers
        let stake_amounts = vec![2_000_000u64, 1_500_000u64, 1_000_000u64]; // Different stake amounts
        
        // All stakers stake to same validator
        let mut stake_tx_hashes = vec![];
        for (staker, &stake_amount) in stakers.iter().zip(stake_amounts.iter()) {
            let stake_tx = env.staking_system.create_stake_transaction(
                staker,
                &validator.address(),
                stake_amount,
                StakingPeriod::SixMonths,
            ).await.unwrap();
            
            let tx_hash = env.aevum_node.submit_transaction(stake_tx).await.unwrap();
            stake_tx_hashes.push(tx_hash);
        }
        
        env.aevum_node.advance_blocks(5).await.unwrap();
        
        // Generate rewards over multiple periods
        for _ in 0..5 {
            env.aevum_node.advance_time_by(chrono::Duration::days(30)).await.unwrap();
            env.aevum_node.advance_blocks(200).await.unwrap();
        }
        
        // Calculate expected reward distribution
        let total_staked: u64 = stake_amounts.iter().sum();
        let mut expected_rewards = vec![];
        
        for &stake_amount in &stake_amounts {
            let proportion = stake_amount as f64 / total_staked as f64;
            let total_validator_rewards = env.rewards_calculator
                .calculate_validator_total_rewards(&validator.address())
                .await
                .unwrap();
            
            let expected_reward = (total_validator_rewards as f64 * proportion) as u64;
            expected_rewards.push(expected_reward);
        }
        
        // Verify actual rewards match expected proportions
        for (i, staker) in stakers.iter().enumerate() {
            let actual_rewards = env.staking_system
                .get_accumulated_rewards(&staker.address(), &validator.address())
                .await
                .unwrap();
            
            let expected = expected_rewards[i];
            let tolerance = expected / 100; // 1% tolerance
            
            assert!(
                (actual_rewards as i64 - expected as i64).abs() < tolerance as i64,
                "Staker {} rewards mismatch: actual={}, expected={}", 
                i, actual_rewards, expected
            );
        }
        
        // Verify total rewards don't exceed pool
        let total_distributed: u64 = stakers.iter()
            .enumerate()
            .map(|(i, staker)| {
                env.staking_system
                    .get_accumulated_rewards(&staker.address(), &validator.address())
                    .block_on()
                    .unwrap()
            })
            .sum();
        
        let validator_total_rewards = env.rewards_calculator
            .calculate_validator_total_rewards(&validator.address())
            .await
            .unwrap();
        
        assert!(total_distributed <= validator_total_rewards);
    }

    #[tokio::test]
    async fn slashing_impact_on_stakers() {
        let mut env = StakingTestEnvironment::new().await;
        
        let validator = &env.validators[1];
        let staker = &env.test_stakers[0];
        let stake_amount = 2_000_000u64;
        
        // Staker stakes to validator
        let stake_tx = env.staking_system.create_stake_transaction(
            staker,
            &validator.address(),
            stake_amount,
            StakingPeriod::OneYear,
        ).await.unwrap();
        
        env.aevum_node.submit_transaction(stake_tx).await.unwrap();
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        let initial_stake_info = env.staking_system
            .get_stake_info(&staker.address(), &validator.address())
            .await
            .unwrap();
        assert_eq!(initial_stake_info.amount, stake_amount);
        
        // Validator misbehaves and gets slashed
        let slashing_evidence = SlashingEvidence {
            validator_address: validator.address().clone(),
            misbehavior_type: MisbehaviorType::DoubleSign,
            height: env.aevum_node.get_current_height().await.unwrap(),
            evidence_data: vec![0xAB, 0xCD],
            timestamp: Utc::now(),
        };
        
        let slashing_tx = env.aevum_node
            .create_slashing_transaction(&slashing_evidence)
            .await
            .unwrap();
        
        env.aevum_node.submit_transaction(slashing_tx).await.unwrap();
        env.aevum_node.advance_blocks(5).await.unwrap();
        
        // Verify staker's stake is reduced proportionally
        let slashed_stake_info = env.staking_system
            .get_stake_info(&staker.address(), &validator.address())
            .await
            .unwrap();
        
        // Assume 10% slashing rate for double signing
        let expected_remaining_stake = (stake_amount as f64 * 0.9) as u64;
        assert_eq!(slashed_stake_info.amount, expected_remaining_stake);
        
        // Verify slashing recorded
        let slashing_info = env.staking_system
            .get_slashing_info(&staker.address(), &validator.address())
            .await
            .unwrap();
        
        assert_eq!(slashing_info.slashed_amount, stake_amount - expected_remaining_stake);
        assert_eq!(slashing_info.reason, MisbehaviorType::DoubleSign);
        
        // Staker should still be able to unstake remaining amount after lock period
        env.aevum_node.advance_time_by(chrono::Duration::days(365)).await.unwrap();
        
        let unstake_tx = env.staking_system.create_unstake_transaction(
            staker,
            &validator.address(),
            expected_remaining_stake,
        ).await.unwrap();
        
        let result = env.aevum_node.submit_transaction(unstake_tx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn compound_staking_rewards() {
        let mut env = StakingTestEnvironment::new().await;
        
        let validator = &env.validators[2];
        let staker = &env.test_stakers[1];
        let initial_stake = 1_000_000u64;
        
        // Initial stake
        let stake_tx = env.staking_system.create_stake_transaction(
            staker,
            &validator.address(),
            initial_stake,
            StakingPeriod::TwoYears,
        ).await.unwrap();
        
        env.aevum_node.submit_transaction(stake_tx).await.unwrap();
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        let mut current_stake = initial_stake;
        
        // Compound rewards quarterly for one year
        for quarter in 0..4 {
            // Generate rewards for 3 months
            env.aevum_node.advance_time_by(chrono::Duration::days(90)).await.unwrap();
            env.aevum_node.advance_blocks(500).await.unwrap();
            
            // Check accumulated rewards
            let rewards = env.staking_system
                .get_accumulated_rewards(&staker.address(), &validator.address())
                .await
                .unwrap();
            
            if rewards > 10_000 { // Only compound if rewards are meaningful
                // Claim and immediately restake rewards
                let claim_tx = env.staking_system.create_claim_rewards_transaction(
                    staker,
                    &validator.address(),
                ).await.unwrap();
                
                env.aevum_node.submit_transaction(claim_tx).await.unwrap();
                env.aevum_node.advance_blocks(2).await.unwrap();
                
                let additional_stake_tx = env.staking_system.create_additional_stake_transaction(
                    staker,
                    &validator.address(),
                    rewards,
                ).await.unwrap();
                
                env.aevum_node.submit_transaction(additional_stake_tx).await.unwrap();
                env.aevum_node.advance_blocks(2).await.unwrap();
                
                current_stake += rewards;
                
                // Verify increased stake
                let stake_info = env.staking_system
                    .get_stake_info(&staker.address(), &validator.address())
                    .await
                    .unwrap();
                
                assert_eq!(stake_info.amount, current_stake);
            }
        }
        
        // Final stake should be higher than initial due to compounding
        assert!(current_stake > initial_stake);
        
        // Calculate effective APY
        let gain = current_stake - initial_stake;
        let apy = (gain as f64 / initial_stake as f64) * 100.0;
        
        // Should have positive yield
        assert!(apy > 0.0);
        println!("Effective APY after compounding: {:.2}%", apy);
    }

    #[tokio::test]
    async fn staking_pool_exhaustion_scenario() {
        let mut env = StakingTestEnvironment::new().await;
        
        // Set limited reward pool
        env.reward_pool.set_total_rewards(1_000_000).await.unwrap(); // 1M AEV total
        
        let validator = &env.validators[0];
        
        // Multiple stakers stake large amounts
        let large_stakers = &env.test_stakers;
        let stake_amounts = vec![10_000_000, 8_000_000, 6_000_000, 4_000_000, 2_000_000];
        
        for (staker, &stake_amount) in large_stakers.iter().zip(stake_amounts.iter()) {
            // Fund stakers with large amounts
            env.aevum_node.fund_address(&staker.address(), stake_amount + 100_000).await.unwrap();
            
            let stake_tx = env.staking_system.create_stake_transaction(
                staker,
                &validator.address(),
                stake_amount,
                StakingPeriod::OneYear,
            ).await.unwrap();
            
            env.aevum_node.submit_transaction(stake_tx).await.unwrap();
        }
        
        env.aevum_node.advance_blocks(5).await.unwrap();
        
        // Generate rewards until pool is exhausted
        let mut total_distributed = 0u64;
        let pool_limit = 1_000_000u64;
        
        for month in 0..12 {
            env.aevum_node.advance_time_by(chrono::Duration::days(30)).await.unwrap();
            env.aevum_node.advance_blocks(200).await.unwrap();
            
            // Calculate monthly reward distribution
            let available_rewards = env.reward_pool.get_available_rewards().await.unwrap();
            
            if available_rewards == 0 {
                // Pool exhausted
                break;
            }
            
            // Distribute available rewards proportionally
            let total_staked = stake_amounts.iter().sum::<u64>();
            
            for (staker, &stake_amount) in large_stakers.iter().zip(stake_amounts.iter()) {
                let proportion = stake_amount as f64 / total_staked as f64;
                let staker_reward = (available_rewards as f64 * proportion) as u64;
                
                total_distributed += staker_reward;
                
                if total_distributed >= pool_limit {
                    break;
                }
            }
        }
        
        // Verify rewards don't exceed pool limit
        assert!(total_distributed <= pool_limit);
        
        // Verify remaining rewards are zero or minimal
        let remaining_rewards = env.reward_pool.get_available_rewards().await.unwrap();
        assert!(remaining_rewards < 10_000); // Less than 1% of pool
        
        // Verify stakers can still unstake principal
        env.aevum_node.advance_time_by(chrono::Duration::days(365)).await.unwrap();
        
        let first_staker = &large_stakers[0];
        let unstake_tx = env.staking_system.create_unstake_transaction(
            first_staker,
            &validator.address(),
            stake_amounts[0],
        ).await.unwrap();
        
        let result = env.aevum_node.submit_transaction(unstake_tx).await;
        assert!(result.is_ok()); // Principal should always be withdrawable
    }
}
```

### Reward Calculation and Distribution Tests
```rust
#[cfg(test)]
mod reward_calculation_tests {
    use super::*;

    #[tokio::test]
    async fn linear_reward_calculation() {
        let mut env = StakingTestEnvironment::new().await;
        
        let validator = &env.validators[0];
        let base_reward_rate = 0.08; // 8% annual rate
        
        // Set up different staking periods and amounts
        let test_cases = vec![
            (1_000_000u64, StakingPeriod::OneMonth, 0.08 / 12.0),   // Monthly rate
            (2_000_000u64, StakingPeriod::SixMonths, 0.08 / 2.0),   // Half-year rate
            (3_000_000u64, StakingPeriod::OneYear, 0.08),           // Annual rate
            (4_000_000u64, StakingPeriod::TwoYears, 0.08 * 2.0),   // Two-year total
        ];
        
        for (i, (amount, period, expected_rate)) in test_cases.iter().enumerate() {
            let staker = &env.test_stakers[i];
            
            // Fund and stake
            env.aevum_node.fund_address(&staker.address(), amount + 100_000).await.unwrap();
            
            let stake_tx = env.staking_system.create_stake_transaction(
                staker,
                &validator.address(),
                *amount,
                *period,
            ).await.unwrap();
            
            env.aevum_node.submit_transaction(stake_tx).await.unwrap();
            
            // Fast-forward to end of staking period
            let period_duration = match period {
                StakingPeriod::OneMonth => chrono::Duration::days(30),
                StakingPeriod::SixMonths => chrono::Duration::days(180),
                StakingPeriod::OneYear => chrono::Duration::days(365),
                StakingPeriod::TwoYears => chrono::Duration::days(730),
            };
            
            env.aevum_node.advance_time_by(period_duration).await.unwrap();
            env.aevum_node.advance_blocks(100).await.unwrap();
            
            // Calculate rewards
            let actual_rewards = env.rewards_calculator
                .calculate_final_rewards(&staker.address(), &validator.address())
                .await
                .unwrap();
            
            let expected_rewards = (*amount as f64 * expected_rate) as u64;
            let tolerance = expected_rewards / 100; // 1% tolerance
            
            assert!(
                (actual_rewards as i64 - expected_rewards as i64).abs() < tolerance as i64,
                "Case {}: actual={}, expected={}, amount={}, rate={}",
                i, actual_rewards, expected_rewards, amount, expected_rate
            );
        }
    }

    #[tokio::test]
    async fn dynamic_reward_adjustment() {
        let mut env = StakingTestEnvironment::new().await;
        
        let validator = &env.validators[0];
        let staker = &env.test_stakers[0];
        let stake_amount = 1_000_000u64;
        
        env.aevum_node.fund_address(&staker.address(), stake_amount + 100_000).await.unwrap();
        
        // Stake tokens
        let stake_tx = env.staking_system.create_stake_transaction(
            staker,
            &validator.address(),
            stake_amount,
            StakingPeriod::OneYear,
        ).await.unwrap();
        
        env.aevum_node.submit_transaction(stake_tx).await.unwrap();
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Track rewards over time with different network conditions
        let mut reward_history = vec![];
        
        // Period 1: Low network activity (higher rewards)
        env.aevum_node.set_network_activity_level(NetworkActivity::Low).await.unwrap();
        env.aevum_node.advance_time_by(chrono::Duration::days(30)).await.unwrap();
        env.aevum_node.advance_blocks(50).await.unwrap(); // Low block production
        
        let rewards_period1 = env.rewards_calculator
            .calculate_period_rewards(&staker.address(), &validator.address(), 1)
            .await
            .unwrap();
        reward_history.push(rewards_period1);
        
        // Period 2: High network activity (lower rewards)
        env.aevum_node.set_network_activity_level(NetworkActivity::High).await.unwrap();
        env.aevum_node.advance_time_by(chrono::Duration::days(30)).await.unwrap();
        env.aevum_node.advance_blocks(200).await.unwrap(); // High block production
        
        let rewards_period2 = env.rewards_calculator
            .calculate_period_rewards(&staker.address(), &validator.address(), 2)
            .await
            .unwrap();
        reward_history.push(rewards_period2);
        
        // Period 3: Medium network activity (medium rewards)
        env.aevum_node.set_network_activity_level(NetworkActivity::Medium).await.unwrap();
        env.aevum_node.advance_time_by(chrono::Duration::days(30)).await.unwrap();
        env.aevum_node.advance_blocks(100).await.unwrap(); // Medium block production
        
        let rewards_period3 = env.rewards_calculator
            .calculate_period_rewards(&staker.address(), &validator.address(), 3)
            .await
            .unwrap();
        reward_history.push(rewards_period3);
        
        // Verify reward adjustment based on network activity
        // Low activity should yield highest rewards
        assert!(rewards_period1 > rewards_period2);
        assert!(rewards_period1 > rewards_period3);
        
        // High activity should yield lowest rewards
        assert!(rewards_period2 < rewards_period3);
        
        // Verify total rewards are within expected bounds
        let total_rewards = reward_history.iter().sum::<u64>();
        let expected_quarterly_rewards = (stake_amount as f64 * 0.08 / 4.0) as u64; // 2% quarterly at 8% annual
        let tolerance = expected_quarterly_rewards / 4; // 25% tolerance for dynamic adjustment
        
        assert!(
            (total_rewards as i64 - expected_quarterly_rewards as i64).abs() < tolerance as i64,
            "Total rewards {} outside tolerance of expected {}",
            total_rewards, expected_quarterly_rewards
        );
    }
}
```
