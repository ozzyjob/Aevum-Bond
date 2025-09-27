# Camada 2: Integration Tests - Sistema de Governança

## 2.4 Testes de Integração do Sistema de Governança

### Complete Governance Lifecycle Tests
```rust
#[cfg(test)]
mod governance_integration_tests {
    use super::*;
    use chrono::{DateTime, Utc};

    struct GovernanceTestEnvironment {
        aevum_node: TestAevumNode,
        governance_system: GovernanceSystem,
        validator_set: ValidatorSet,
        test_validators: Vec<TestValidator>,
        proposal_manager: ProposalManager,
        voting_system: VotingSystem,
    }

    impl GovernanceTestEnvironment {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let aevum_node = TestAevumNode::new(temp_dir.path()).await;
            let governance_system = GovernanceSystem::new(aevum_node.clone());
            let validator_set = ValidatorSet::new();
            let proposal_manager = ProposalManager::new();
            let voting_system = VotingSystem::new();
            
            // Create test validators with different stakes
            let mut test_validators = vec![];
            let stake_amounts = vec![10_000_000, 7_500_000, 5_000_000, 2_500_000, 1_000_000]; // AEV stakes
            
            for (i, stake) in stake_amounts.iter().enumerate() {
                let validator = TestValidator::new(format!("validator_{}", i));
                
                // Stake AEV to become validator
                aevum_node.fund_address(&validator.address(), *stake).await.unwrap();
                validator.stake_tokens(*stake, &aevum_node).await.unwrap();
                
                test_validators.push(validator);
            }
            
            Self {
                aevum_node,
                governance_system,
                validator_set,
                test_validators,
                proposal_manager,
                voting_system,
            }
        }
        
        async fn update_validator_set(&mut self) {
            // Update validator set based on current stakes
            for validator in &self.test_validators {
                let stake = self.aevum_node
                    .get_staked_amount(&validator.address())
                    .await
                    .unwrap();
                
                self.validator_set.update_validator(
                    validator.address().clone(),
                    stake,
                    validator.voting_power(stake),
                ).await.unwrap();
            }
        }
    }

    #[tokio::test]
    async fn complete_proposal_lifecycle() {
        let mut env = GovernanceTestEnvironment::new().await;
        env.update_validator_set().await;
        
        // Phase 1: Proposal Creation
        let proposer = &env.test_validators[0]; // Highest stake validator
        
        let proposal = Proposal {
            id: ProposalId::new(),
            title: "Increase Block Size Limit".to_string(),
            description: "Proposal to increase maximum block size from 1MB to 2MB".to_string(),
            proposal_type: ProposalType::ParameterChange {
                parameter: "max_block_size".to_string(),
                current_value: "1048576".to_string(), // 1MB
                proposed_value: "2097152".to_string(), // 2MB
            },
            proposer: proposer.address().clone(),
            creation_time: Utc::now(),
            voting_start_time: Utc::now() + chrono::Duration::hours(24),
            voting_end_time: Utc::now() + chrono::Duration::days(7),
            execution_time: Utc::now() + chrono::Duration::days(8),
            status: ProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            quorum_threshold: 5_000_000, // 50% of total staked AEV
            approval_threshold: 6_600_000, // 66% approval needed
        };
        
        // Submit proposal
        let proposal_tx = env.governance_system
            .create_proposal_transaction(&proposal, proposer)
            .await
            .unwrap();
        
        let proposal_tx_hash = env.aevum_node
            .submit_transaction(proposal_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify proposal creation
        let created_proposal = env.proposal_manager
            .get_proposal(&proposal.id)
            .await
            .unwrap();
        assert_eq!(created_proposal.title, proposal.title);
        assert_eq!(created_proposal.status, ProposalStatus::Active);
        
        // Phase 2: Voting Period
        // Simulate time passing to voting start
        env.aevum_node.advance_time_to(proposal.voting_start_time).await.unwrap();
        
        // Validators vote
        let voting_results = vec![
            (0, VoteChoice::For),     // 10M AEV - For
            (1, VoteChoice::For),     // 7.5M AEV - For  
            (2, VoteChoice::Against), // 5M AEV - Against
            (3, VoteChoice::For),     // 2.5M AEV - For
            (4, VoteChoice::Abstain), // 1M AEV - Abstain
        ];
        
        let mut vote_tx_hashes = vec![];
        for (validator_idx, vote_choice) in voting_results {
            let validator = &env.test_validators[validator_idx];
            
            let vote_tx = env.voting_system.create_vote_transaction(
                &proposal.id,
                vote_choice,
                validator,
            ).await.unwrap();
            
            let vote_tx_hash = env.aevum_node
                .submit_transaction(vote_tx)
                .await
                .unwrap();
            
            vote_tx_hashes.push(vote_tx_hash);
        }
        
        env.aevum_node.advance_blocks(5).await.unwrap();
        
        // Verify votes recorded
        let updated_proposal = env.proposal_manager
            .get_proposal(&proposal.id)
            .await
            .unwrap();
        
        // Expected votes: For = 20M AEV, Against = 5M AEV, Abstain = 1M AEV
        assert_eq!(updated_proposal.votes_for, 20_000_000);
        assert_eq!(updated_proposal.votes_against, 5_000_000);
        assert_eq!(updated_proposal.votes_abstain, 1_000_000);
        
        // Phase 3: Voting End & Tally
        env.aevum_node.advance_time_to(proposal.voting_end_time).await.unwrap();
        
        let tally_result = env.governance_system
            .tally_proposal_votes(&proposal.id)
            .await
            .unwrap();
        
        // Check quorum: (20M + 5M + 1M) = 26M > 5M threshold ✓
        assert!(tally_result.total_votes >= proposal.quorum_threshold);
        
        // Check approval: 20M / (20M + 5M) = 80% > 66% threshold ✓
        let approval_rate = tally_result.votes_for as f64 / 
                           (tally_result.votes_for + tally_result.votes_against) as f64;
        assert!(approval_rate >= 0.66);
        
        assert_eq!(tally_result.result, ProposalResult::Approved);
        
        // Phase 4: Execution
        env.aevum_node.advance_time_to(proposal.execution_time).await.unwrap();
        
        let execution_tx = env.governance_system
            .create_execution_transaction(&proposal.id)
            .await
            .unwrap();
        
        let execution_tx_hash = env.aevum_node
            .submit_transaction(execution_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify execution
        let final_proposal = env.proposal_manager
            .get_proposal(&proposal.id)
            .await
            .unwrap();
        assert_eq!(final_proposal.status, ProposalStatus::Executed);
        
        // Verify parameter actually changed
        let current_max_block_size = env.aevum_node
            .get_protocol_parameter("max_block_size")
            .await
            .unwrap();
        assert_eq!(current_max_block_size, "2097152");
    }

    #[tokio::test]
    async fn proposal_rejection_flow() {
        let mut env = GovernanceTestEnvironment::new().await;
        env.update_validator_set().await;
        
        let proposer = &env.test_validators[1];
        
        // Create unpopular proposal
        let proposal = Proposal {
            id: ProposalId::new(),
            title: "Reduce Block Rewards".to_string(),
            description: "Proposal to reduce block rewards by 50%".to_string(),
            proposal_type: ProposalType::ParameterChange {
                parameter: "block_reward".to_string(),
                current_value: "50000000".to_string(), // 50 AEV
                proposed_value: "25000000".to_string(), // 25 AEV
            },
            proposer: proposer.address().clone(),
            creation_time: Utc::now(),
            voting_start_time: Utc::now() + chrono::Duration::hours(1),
            voting_end_time: Utc::now() + chrono::Duration::days(3),
            execution_time: Utc::now() + chrono::Duration::days(4),
            status: ProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            quorum_threshold: 5_000_000,
            approval_threshold: 6_600_000,
        };
        
        // Submit proposal
        let proposal_tx = env.governance_system
            .create_proposal_transaction(&proposal, proposer)
            .await
            .unwrap();
        
        env.aevum_node.submit_transaction(proposal_tx).await.unwrap();
        env.aevum_node.advance_blocks(2).await.unwrap();
        
        // Start voting
        env.aevum_node.advance_time_to(proposal.voting_start_time).await.unwrap();
        
        // Most validators vote against
        let voting_results = vec![
            (0, VoteChoice::Against), // 10M AEV - Against
            (1, VoteChoice::For),     // 7.5M AEV - For
            (2, VoteChoice::Against), // 5M AEV - Against
            (3, VoteChoice::Against), // 2.5M AEV - Against
            (4, VoteChoice::Against), // 1M AEV - Against
        ];
        
        for (validator_idx, vote_choice) in voting_results {
            let validator = &env.test_validators[validator_idx];
            let vote_tx = env.voting_system.create_vote_transaction(
                &proposal.id,
                vote_choice,
                validator,
            ).await.unwrap();
            
            env.aevum_node.submit_transaction(vote_tx).await.unwrap();
        }
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        env.aevum_node.advance_time_to(proposal.voting_end_time).await.unwrap();
        
        // Tally votes
        let tally_result = env.governance_system
            .tally_proposal_votes(&proposal.id)
            .await
            .unwrap();
        
        // Expected: For = 7.5M, Against = 18.5M
        assert_eq!(tally_result.votes_for, 7_500_000);
        assert_eq!(tally_result.votes_against, 18_500_000);
        
        // Quorum met but approval failed: 7.5M / (7.5M + 18.5M) = 28.8% < 66%
        assert!(tally_result.total_votes >= proposal.quorum_threshold);
        let approval_rate = tally_result.votes_for as f64 / 
                           (tally_result.votes_for + tally_result.votes_against) as f64;
        assert!(approval_rate < 0.66);
        
        assert_eq!(tally_result.result, ProposalResult::Rejected);
        
        // Verify proposal marked as rejected
        let final_proposal = env.proposal_manager
            .get_proposal(&proposal.id)
            .await
            .unwrap();
        assert_eq!(final_proposal.status, ProposalStatus::Rejected);
        
        // Verify parameter unchanged
        let current_block_reward = env.aevum_node
            .get_protocol_parameter("block_reward")
            .await
            .unwrap();
        assert_eq!(current_block_reward, "50000000"); // Unchanged
    }

    #[tokio::test]
    async fn quorum_not_met_scenario() {
        let mut env = GovernanceTestEnvironment::new().await;
        env.update_validator_set().await;
        
        let proposer = &env.test_validators[0];
        
        let proposal = Proposal {
            id: ProposalId::new(),
            title: "Minor Protocol Update".to_string(),
            description: "Small technical update".to_string(),
            proposal_type: ProposalType::ParameterChange {
                parameter: "min_fee".to_string(),
                current_value: "1000".to_string(),
                proposed_value: "2000".to_string(),
            },
            proposer: proposer.address().clone(),
            creation_time: Utc::now(),
            voting_start_time: Utc::now() + chrono::Duration::hours(1),
            voting_end_time: Utc::now() + chrono::Duration::days(3),
            execution_time: Utc::now() + chrono::Duration::days(4),
            status: ProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            quorum_threshold: 15_000_000, // High quorum requirement
            approval_threshold: 6_600_000,
        };
        
        // Submit proposal
        let proposal_tx = env.governance_system
            .create_proposal_transaction(&proposal, proposer)
            .await
            .unwrap();
        
        env.aevum_node.submit_transaction(proposal_tx).await.unwrap();
        env.aevum_node.advance_blocks(2).await.unwrap();
        
        // Only some validators participate (low turnout)
        env.aevum_node.advance_time_to(proposal.voting_start_time).await.unwrap();
        
        let voting_results = vec![
            (0, VoteChoice::For),     // 10M AEV - For
            (3, VoteChoice::For),     // 2.5M AEV - For
            // Other validators don't vote
        ];
        
        for (validator_idx, vote_choice) in voting_results {
            let validator = &env.test_validators[validator_idx];
            let vote_tx = env.voting_system.create_vote_transaction(
                &proposal.id,
                vote_choice,
                validator,
            ).await.unwrap();
            
            env.aevum_node.submit_transaction(vote_tx).await.unwrap();
        }
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        env.aevum_node.advance_time_to(proposal.voting_end_time).await.unwrap();
        
        // Tally votes
        let tally_result = env.governance_system
            .tally_proposal_votes(&proposal.id)
            .await
            .unwrap();
        
        // Total votes: 12.5M < 15M quorum threshold
        assert_eq!(tally_result.total_votes, 12_500_000);
        assert!(tally_result.total_votes < proposal.quorum_threshold);
        assert_eq!(tally_result.result, ProposalResult::QuorumNotMet);
        
        // Verify proposal marked as failed due to quorum
        let final_proposal = env.proposal_manager
            .get_proposal(&proposal.id)
            .await
            .unwrap();
        assert_eq!(final_proposal.status, ProposalStatus::QuorumNotMet);
    }
}
```

### Validator Management Integration
```rust
#[cfg(test)]
mod validator_management_tests {
    use super::*;

    #[tokio::test]
    async fn validator_registration_flow() {
        let mut env = GovernanceTestEnvironment::new().await;
        
        // Create new validator candidate
        let new_validator = TestValidator::new("new_validator".to_string());
        let required_stake = 1_000_000u64; // 1M AEV minimum stake
        
        // Fund the candidate
        env.aevum_node.fund_address(
            &new_validator.address(),
            required_stake + 100_000, // Extra for fees
        ).await.unwrap();
        
        // Step 1: Stake tokens
        let stake_tx = new_validator.create_stake_transaction(
            required_stake,
            &env.aevum_node,
        ).await.unwrap();
        
        let stake_tx_hash = env.aevum_node
            .submit_transaction(stake_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Step 2: Register as validator
        let registration_tx = new_validator.create_validator_registration(
            ValidatorMetadata {
                name: "New Validator".to_string(),
                description: "Test validator for integration tests".to_string(),
                website: Some("https://example.com".to_string()),
                commission_rate: 0.05, // 5% commission
            },
        ).await.unwrap();
        
        let registration_tx_hash = env.aevum_node
            .submit_transaction(registration_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(5).await.unwrap();
        
        // Step 3: Verify registration
        let validator_info = env.validator_set
            .get_validator(&new_validator.address())
            .await
            .unwrap();
        
        assert_eq!(validator_info.stake, required_stake);
        assert_eq!(validator_info.status, ValidatorStatus::Active);
        assert_eq!(validator_info.commission_rate, 0.05);
        
        // Step 4: Verify voting power calculated correctly
        let total_stake = env.validator_set.get_total_stake().await.unwrap();
        let expected_voting_power = (required_stake as f64 / total_stake as f64) * 100.0;
        
        assert!((validator_info.voting_power - expected_voting_power).abs() < 0.01);
        
        // Step 5: Verify can participate in governance
        let test_proposal = Proposal {
            id: ProposalId::new(),
            title: "Test Proposal".to_string(),
            description: "Testing new validator voting".to_string(),
            proposal_type: ProposalType::TextProposal,
            proposer: new_validator.address().clone(),
            creation_time: Utc::now(),
            voting_start_time: Utc::now() + chrono::Duration::hours(1),
            voting_end_time: Utc::now() + chrono::Duration::days(1),
            execution_time: Utc::now() + chrono::Duration::days(2),
            status: ProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            quorum_threshold: 1_000_000,
            approval_threshold: 500_000,
        };
        
        let proposal_tx = env.governance_system
            .create_proposal_transaction(&test_proposal, &new_validator)
            .await
            .unwrap();
        
        let result = env.aevum_node.submit_transaction(proposal_tx).await;
        assert!(result.is_ok()); // New validator can create proposals
    }

    #[tokio::test]
    async fn validator_slashing_flow() {
        let mut env = GovernanceTestEnvironment::new().await;
        env.update_validator_set().await;
        
        let misbehaving_validator = &env.test_validators[2]; // 5M AEV stake
        let initial_stake = env.aevum_node
            .get_staked_amount(&misbehaving_validator.address())
            .await
            .unwrap();
        
        // Simulate misbehavior detection (double signing)
        let evidence = SlashingEvidence {
            validator_address: misbehaving_validator.address().clone(),
            misbehavior_type: MisbehaviorType::DoubleSign,
            height: env.aevum_node.get_current_height().await.unwrap(),
            evidence_data: vec![0x01, 0x02, 0x03], // Simplified evidence
            timestamp: Utc::now(),
        };
        
        // Submit slashing evidence
        let slashing_tx = env.governance_system
            .create_slashing_transaction(&evidence)
            .await
            .unwrap();
        
        let slashing_tx_hash = env.aevum_node
            .submit_transaction(slashing_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(5).await.unwrap();
        
        // Verify slashing applied
        let slashed_validator = env.validator_set
            .get_validator(&misbehaving_validator.address())
            .await
            .unwrap();
        
        // Double signing typically results in 5% slash
        let expected_slashed_amount = (initial_stake as f64 * 0.05) as u64;
        let remaining_stake = initial_stake - expected_slashed_amount;
        
        assert_eq!(slashed_validator.stake, remaining_stake);
        assert_eq!(slashed_validator.status, ValidatorStatus::Slashed);
        
        // Verify slashed tokens burned/redistributed
        let slashing_info = env.governance_system
            .get_slashing_info(&evidence.validator_address)
            .await
            .unwrap();
        
        assert_eq!(slashing_info.slashed_amount, expected_slashed_amount);
        assert_eq!(slashing_info.misbehavior_type, MisbehaviorType::DoubleSign);
        
        // Verify validator cannot participate in consensus temporarily
        let validator_status = env.validator_set
            .get_validator_status(&misbehaving_validator.address())
            .await
            .unwrap();
        
        assert_eq!(validator_status, ValidatorStatus::Slashed);
        
        // Test rehabilitation after slashing period
        let slashing_period = chrono::Duration::days(30);
        env.aevum_node.advance_time_by(slashing_period).await.unwrap();
        
        // Submit rehabilitation request
        let rehab_tx = misbehaving_validator
            .create_rehabilitation_request()
            .await
            .unwrap();
        
        env.aevum_node.submit_transaction(rehab_tx).await.unwrap();
        env.aevum_node.advance_blocks(10).await.unwrap();
        
        // Verify rehabilitation
        let rehabilitated_validator = env.validator_set
            .get_validator(&misbehaving_validator.address())
            .await
            .unwrap();
        
        assert_eq!(rehabilitated_validator.status, ValidatorStatus::Active);
        // Stake remains reduced
        assert_eq!(rehabilitated_validator.stake, remaining_stake);
    }

    #[tokio::test]
    async fn validator_delegation_flow() {
        let mut env = GovernanceTestEnvironment::new().await;
        
        let validator = &env.test_validators[0]; // 10M AEV stake
        let delegator = TestWallet::new_aevum();
        
        // Fund delegator
        let delegation_amount = 2_000_000u64; // 2M AEV
        env.aevum_node.fund_address(
            &delegator.address(),
            delegation_amount + 50_000, // Extra for fees
        ).await.unwrap();
        
        let initial_validation_power = env.validator_set
            .get_validator(&validator.address())
            .await
            .unwrap()
            .voting_power;
        
        // Step 1: Delegate tokens
        let delegation_tx = delegator.create_delegation_transaction(
            &validator.address(),
            delegation_amount,
        ).await.unwrap();
        
        let delegation_tx_hash = env.aevum_node
            .submit_transaction(delegation_tx)
            .await
            .unwrap();
        
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Step 2: Verify delegation recorded
        let delegation_info = env.governance_system
            .get_delegation(&delegator.address(), &validator.address())
            .await
            .unwrap();
        
        assert_eq!(delegation_info.amount, delegation_amount);
        assert_eq!(delegation_info.validator, validator.address());
        assert_eq!(delegation_info.delegator, delegator.address());
        
        // Step 3: Verify validator power increased
        env.update_validator_set().await;
        let updated_validator = env.validator_set
            .get_validator(&validator.address())
            .await
            .unwrap();
        
        assert_eq!(updated_validator.stake, 10_000_000 + delegation_amount);
        assert!(updated_validator.voting_power > initial_validation_power);
        
        // Step 4: Test reward distribution
        env.aevum_node.advance_blocks(100).await.unwrap(); // Generate rewards
        
        let validator_rewards = env.governance_system
            .calculate_validator_rewards(&validator.address())
            .await
            .unwrap();
        
        let delegator_rewards = env.governance_system
            .calculate_delegator_rewards(&delegator.address(), &validator.address())
            .await
            .unwrap();
        
        // Delegator should receive rewards proportional to delegation
        let delegation_ratio = delegation_amount as f64 / (10_000_000 + delegation_amount) as f64;
        let expected_delegator_share = (validator_rewards.total_rewards as f64 * delegation_ratio * 0.95) as u64; // 95% after commission
        
        assert!((delegator_rewards.amount as f64 - expected_delegator_share as f64).abs() / expected_delegator_share as f64 < 0.01);
        
        // Step 5: Test undelegation
        let undelegate_tx = delegator.create_undelegation_transaction(
            &validator.address(),
            delegation_amount / 2, // Undelegate half
        ).await.unwrap();
        
        env.aevum_node.submit_transaction(undelegate_tx).await.unwrap();
        env.aevum_node.advance_blocks(3).await.unwrap();
        
        // Verify partial undelegation
        let updated_delegation = env.governance_system
            .get_delegation(&delegator.address(), &validator.address())
            .await
            .unwrap();
        
        assert_eq!(updated_delegation.amount, delegation_amount / 2);
    }
}
```
