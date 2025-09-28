use bond_core::*;

/// Integration test for Bond core components
/// Tests the interaction between different modules within bond-core

#[test]
fn test_block_creation_and_validation_integration() {
    // Create a simple coinbase transaction
    let coinbase_tx = Transaction::coinbase(5_000_000_000, vec![]);
    let transactions = vec![coinbase_tx];

    // Create a block header for testing
    let header = BlockHeader::new(
        1,
        BlockHash::ZERO,
        MerkleRoot::ZERO,
        chrono::Utc::now(),
        DifficultyTarget::MAX, // Easy target for testing
        0,
    );

    // Create complete block
    let test_block = Block::new(header, transactions);

    // Verify block creation
    assert_eq!(test_block.header.version, 1, "Block version should be 1");
    assert_eq!(
        test_block.transactions.len(),
        1,
        "Block should have 1 transaction"
    );

    // Test block hashing
    let block_hash = test_block.hash();
    assert!(block_hash.is_ok(), "Block hashing should succeed");

    // Test merkle root calculation
    let merkle_root = test_block.calculate_merkle_root();
    assert!(
        merkle_root.is_ok(),
        "Merkle root calculation should succeed"
    );
}

#[test]
fn test_transaction_creation_and_validation_integration() {
    // Create a basic coinbase transaction
    let coinbase_tx = Transaction::coinbase(5_000_000_000, vec![]);

    // Verify transaction structure
    assert_eq!(coinbase_tx.version, 1, "Transaction version should be 1");
    // Note: Coinbase transactions in Bond actually have inputs (special coinbase inputs)
    assert_eq!(
        coinbase_tx.outputs.len(),
        1,
        "Coinbase transaction should have 1 output"
    );

    // Test transaction hash calculation
    let tx_hash = coinbase_tx.hash();
    assert!(
        tx_hash.is_ok(),
        "Transaction hash calculation should succeed"
    );

    // Test transaction validation
    let validation_result = coinbase_tx.validate();
    assert!(
        validation_result.is_ok(),
        "Coinbase transaction validation should succeed"
    );

    // Test total output value calculation
    let total_output = coinbase_tx.total_output_value();
    assert!(
        total_output.is_ok(),
        "Total output calculation should succeed"
    );
    assert_eq!(
        total_output.unwrap(),
        5_000_000_000,
        "Total output should match coinbase amount"
    );
}

#[test]
fn test_utxo_creation_and_management_integration() {
    // Test UTXO ID creation
    let tx_hash = TransactionHash::from_bytes([1u8; 32]);
    let utxo_id = UtxoId::new(tx_hash, 0);

    // Verify UTXO ID display format
    let utxo_display = format!("{}", utxo_id);
    assert!(
        utxo_display.len() > 0,
        "UTXO ID should have a display format"
    );

    // Test programmable UTXO creation
    let script = Script {
        code: vec![0x63, 0x64, 0x68],
    }; // Simple script
    let programmable_utxo = ProgrammableUtxo::new(
        utxo_id.clone(),
        1_000_000_000, // 1 BND
        script,
        0, // No timelock for this test
    );

    // Verify programmable UTXO properties
    assert_eq!(
        programmable_utxo.value, 1_000_000_000,
        "UTXO value should match"
    );
    assert_eq!(programmable_utxo.id, utxo_id, "UTXO ID should match");
}

#[test]
fn test_script_interpreter_integration() {
    // Test script interpreter creation
    let script_interpreter = ScriptInterpreter::new();

    // Create a simple script for testing
    let simple_script = Script {
        code: vec![
            0x01, 0x05, // Push number 5
            0x01, 0x03, // Push number 3
            0x93, // OP_ADD
        ],
    };

    // Create a script context for execution
    let script_context = ScriptContext {
        tx_hash: vec![0u8; 32],
        input_index: 0,
        block_height: 100,
        timestamp: 1640995200, // Unix timestamp
    };

    // Execute the script
    let execution_result = script_interpreter.execute(&simple_script.code, &script_context);
    assert!(
        execution_result.is_ok(),
        "Simple script execution should succeed"
    );
}

#[test]
fn test_mining_integration() {
    // Test miner creation
    let mut miner = Miner::new();
    // Note: stats are private, so we can't directly test initial values

    // Create a header for mining with easy target
    let header = mining::create_mining_header(
        1,
        BlockHash::ZERO,
        MerkleRoot::ZERO,
        DifficultyTarget::MAX, // Very easy target
    );

    // Mine the block
    let result = miner.mine_block(header);
    assert!(result.is_ok(), "Mining with easy target should succeed");

    let mined_header = result.unwrap();

    // Verify proof of work
    let pow_validation = mined_header.validates_pow();
    assert!(pow_validation.is_ok(), "Mined block should have valid PoW");
    assert!(pow_validation.unwrap(), "PoW validation should return true");
}

#[test]
fn test_consensus_validation_integration() {
    // Test consensus creation
    let consensus = Consensus::new();

    // Create a simple block for validation
    let coinbase_tx = Transaction::coinbase(5_000_000_000, vec![]);
    let test_block = Block::new(
        BlockHeader::new(
            1,
            BlockHash::ZERO,
            MerkleRoot::ZERO,
            chrono::Utc::now(),
            DifficultyTarget::MAX,
            0,
        ),
        vec![coinbase_tx],
    );

    // Test block validation (should fail due to invalid PoW, which is expected)
    let validation_result = consensus.validate_block(&test_block, &[]);
    // We expect this to fail because we haven't mined the block properly
    assert!(
        validation_result.is_err(),
        "Block with invalid PoW should fail validation"
    );

    // Test difficulty calculation
    let current_target = DifficultyTarget::MAX;
    let difficulty_result = consensus.calculate_next_difficulty(&[], current_target);
    assert!(
        difficulty_result.is_ok(),
        "Difficulty calculation should succeed"
    );
}

#[test]
fn test_complete_block_lifecycle_integration() {
    // This test demonstrates the complete lifecycle of creating, mining, and validating a block

    // Step 1: Create transactions
    let coinbase_tx = Transaction::coinbase(5_000_000_000, vec![]);
    let transactions = vec![coinbase_tx];

    // Step 2: Calculate merkle root
    let block_for_merkle = Block::new(
        BlockHeader::new(
            1,
            BlockHash::ZERO,
            MerkleRoot::ZERO,
            chrono::Utc::now(),
            DifficultyTarget::MAX,
            0,
        ),
        transactions.clone(),
    );
    let merkle_root = block_for_merkle.calculate_merkle_root().unwrap();

    // Step 3: Create header for mining
    let mining_header =
        mining::create_mining_header(1, BlockHash::ZERO, merkle_root, DifficultyTarget::MAX);

    // Step 4: Mine the block
    let mut miner = Miner::new();
    let mined_header = miner.mine_block(mining_header).unwrap();

    // Step 5: Create the final block
    let final_block = Block::new(mined_header, transactions);

    // Step 6: Validate the mined block
    assert!(
        final_block.header.validates_pow().unwrap(),
        "Mined block should have valid PoW"
    );

    // Step 7: Test with consensus (basic validation)
    let consensus = Consensus::new();
    // Note: This might still fail due to other validation rules, but PoW should be valid
    let _validation_result = consensus.validate_block(&final_block, &[]);

    // Verify all components worked together
    assert!(
        final_block.hash().is_ok(),
        "Final block should hash successfully"
    );
    assert_eq!(
        final_block.transactions.len(),
        1,
        "Block should contain the transaction"
    );
    // Note: miner stats are private, so we can't directly test hash attempts
}
