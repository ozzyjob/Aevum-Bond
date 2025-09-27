pub use crate::mining::{Consensus, MiningConfig};
use crate::{Block, Transaction, BondError, BondResult};
use std::collections::HashMap;
use crate::{UtxoId, TransactionOutput};

/// Blockchain state and consensus validation
pub struct ChainState {
    /// Consensus rules
    consensus: Consensus,
    /// Chain of blocks (simplified - in production this would be optimized)
    blocks: Vec<Block>,
    /// UTXO set for transaction validation
    utxo_set: HashMap<UtxoId, TransactionOutput>,
    /// Current blockchain height
    height: u32,
    /// Total work (simplified proof-of-work accumulation)
    total_work: u64,
}

/// Chain statistics and metrics
#[derive(Debug, Clone)]
pub struct ChainStats {
    /// Current blockchain height
    pub height: u32,
    /// Total number of transactions
    pub total_transactions: u64,
    /// Size of UTXO set
    pub utxo_count: usize,
    /// Total value in circulation (in Elos)
    pub total_supply: u64,
    /// Average block time over last 100 blocks
    pub average_block_time: f64,
}

impl ChainState {
    /// Create a new chain state with genesis block
    pub fn new_with_genesis(genesis_block: Block) -> BondResult<Self> {
        let mut chain_state = Self {
            consensus: Consensus::new(),
            blocks: Vec::new(),
            utxo_set: HashMap::new(),
            height: 0,
            total_work: 0,
        };

        // Add genesis block
        chain_state.add_block(genesis_block)?;
        Ok(chain_state)
    }

    /// Add a new block to the chain
    pub fn add_block(&mut self, block: Block) -> BondResult<()> {
        // Validate the block against consensus rules
        self.consensus.validate_block(&block, &self.blocks)?;

        // Validate all transactions in the block
        for (tx_index, transaction) in block.transactions.iter().enumerate() {
            self.validate_transaction(transaction, tx_index == 0)?;
        }

        // Apply the block to the chain state
        self.apply_block(&block)?;

        // Add block to the chain
        self.blocks.push(block);
        self.height += 1;

        Ok(())
    }

    /// Validate a transaction against the current UTXO set
    fn validate_transaction(&self, transaction: &Transaction, is_coinbase: bool) -> BondResult<()> {
        // Basic transaction validation
        transaction.validate()?;

        if is_coinbase {
            if !transaction.is_coinbase() {
                return Err(BondError::InvalidTransaction {
                    reason: "First transaction in block must be coinbase".to_string(),
                });
            }
            return Ok(()); // Coinbase transactions don't need UTXO validation
        }

        if transaction.is_coinbase() {
            return Err(BondError::InvalidTransaction {
                reason: "Coinbase transaction can only be first in block".to_string(),
            });
        }

        // Check that all inputs reference valid UTXOs
        for input in &transaction.inputs {
            if !self.utxo_set.contains_key(&input.previous_output) {
                return Err(BondError::InvalidTransaction {
                    reason: format!("Referenced UTXO not found: {}", input.previous_output),
                });
            }
        }

        // Check that input value >= output value (fee validation)
        let input_value = transaction.total_input_value(&self.utxo_set)?;
        let output_value = transaction.total_output_value()?;

        if input_value < output_value {
            return Err(BondError::InsufficientFunds {
                required: output_value,
                available: input_value,
            });
        }

        // TODO: Validate script execution (will be implemented in Sprint 5)
        // For now, we assume all scripts are valid

        Ok(())
    }

    /// Apply a block to the chain state (update UTXO set)
    fn apply_block(&mut self, block: &Block) -> BondResult<()> {
        for transaction in &block.transactions {
            self.apply_transaction(transaction)?;
        }
        Ok(())
    }

    /// Apply a transaction to the UTXO set
    fn apply_transaction(&mut self, transaction: &Transaction) -> BondResult<()> {
        let tx_hash = transaction.hash()?;

        // Remove spent UTXOs (except for coinbase)
        if !transaction.is_coinbase() {
            for input in &transaction.inputs {
                if self.utxo_set.remove(&input.previous_output).is_none() {
                    return Err(BondError::DoubleSpending {
                        utxo_id: input.previous_output.to_string(),
                    });
                }
            }
        }

        // Add new UTXOs
        for (output_index, output) in transaction.outputs.iter().enumerate() {
            let utxo_id = UtxoId::new(tx_hash, output_index as u32);
            self.utxo_set.insert(utxo_id, output.clone());
        }

        Ok(())
    }

    /// Get the current chain height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the latest block
    pub fn latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    /// Get a block by height
    pub fn get_block(&self, height: u32) -> Option<&Block> {
        if height < self.blocks.len() as u32 {
            Some(&self.blocks[height as usize])
        } else {
            None
        }
    }

    /// Get the UTXO set
    pub fn utxo_set(&self) -> &HashMap<UtxoId, TransactionOutput> {
        &self.utxo_set
    }

    /// Check if a UTXO exists
    pub fn has_utxo(&self, utxo_id: &UtxoId) -> bool {
        self.utxo_set.contains_key(utxo_id)
    }

    /// Get a UTXO by ID
    pub fn get_utxo(&self, utxo_id: &UtxoId) -> Option<&TransactionOutput> {
        self.utxo_set.get(utxo_id)
    }

    /// Calculate chain statistics
    pub fn stats(&self) -> ChainStats {
        let total_transactions = self.blocks.iter()
            .map(|b| b.transactions.len() as u64)
            .sum();

        let total_supply = self.utxo_set.values()
            .map(|utxo| utxo.value)
            .sum();

        let average_block_time = self.calculate_average_block_time();

        ChainStats {
            height: self.height,
            total_transactions,
            utxo_count: self.utxo_set.len(),
            total_supply,
            average_block_time,
        }
    }

    /// Calculate average block time over the last N blocks
    fn calculate_average_block_time(&self) -> f64 {
        const SAMPLE_SIZE: usize = 100;
        
        if self.blocks.len() < 2 {
            return 0.0;
        }

        let start_index = if self.blocks.len() > SAMPLE_SIZE {
            self.blocks.len() - SAMPLE_SIZE
        } else {
            0
        };

        let sample_blocks = &self.blocks[start_index..];
        if sample_blocks.len() < 2 {
            return 0.0;
        }

        let first_time = sample_blocks[0].header.timestamp;
        let last_time = sample_blocks[sample_blocks.len() - 1].header.timestamp;
        
        let total_time = (last_time - first_time).num_seconds() as f64;
        let block_count = (sample_blocks.len() - 1) as f64;

        if block_count > 0.0 {
            total_time / block_count
        } else {
            0.0
        }
    }

    /// Validate the entire chain (useful for initial sync)
    pub fn validate_chain(&self) -> BondResult<()> {
        for (i, block) in self.blocks.iter().enumerate() {
            let previous_blocks = &self.blocks[..i];
            self.consensus.validate_block(block, previous_blocks)?;
        }
        Ok(())
    }

    /// Get blocks in a range
    pub fn get_blocks_range(&self, start: u32, end: u32) -> Vec<&Block> {
        let start_idx = start.min(self.blocks.len() as u32) as usize;
        let end_idx = end.min(self.blocks.len() as u32) as usize;
        
        self.blocks[start_idx..end_idx].iter().collect()
    }

    /// Find UTXOs for a given script pattern (simplified)
    pub fn find_utxos_by_script(&self, script_pattern: &[u8]) -> Vec<(UtxoId, &TransactionOutput)> {
        self.utxo_set
            .iter()
            .filter(|(_, output)| output.script_pubkey.code == script_pattern)
            .map(|(id, output)| (*id, output))
            .collect()
    }

    /// Calculate the balance for UTXOs matching a script pattern
    pub fn calculate_balance(&self, script_pattern: &[u8]) -> u64 {
        self.find_utxos_by_script(script_pattern)
            .iter()
            .map(|(_, output)| output.value)
            .sum()
    }

    /// Get the consensus rules
    pub fn consensus(&self) -> &Consensus {
        &self.consensus
    }

    /// Get all blocks (be careful with this in production)
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
}

/// Create a genesis block for the Bond blockchain
pub fn create_genesis_block() -> BondResult<Block> {
    use crate::{BlockHeader, BlockHash, MerkleRoot, DifficultyTarget};
    use chrono::{TimeZone, Utc};

    // Genesis block parameters
    let genesis_time = Utc.with_ymd_and_hms(2025, 9, 1, 0, 0, 0).unwrap();
    let genesis_reward = 5_000_000_000u64; // 5 BND in Elos
    let genesis_message = b"Aevum & Bond Genesis - Building the Post-Quantum Financial Future";

    // Create genesis transaction
    let genesis_tx = Transaction::coinbase(genesis_reward, genesis_message.to_vec());

    // Create block with transactions
    let transactions = vec![genesis_tx];
    let mut block = Block::new(
        BlockHeader::new(
            1,
            BlockHash::ZERO,
            MerkleRoot::ZERO, // Will be calculated
            genesis_time,
            DifficultyTarget::MAX, // Easy target for genesis
            0,
        ),
        transactions,
    );

    // Calculate and set the correct Merkle root
    let merkle_root = block.calculate_merkle_root()?;
    block.header.merkle_root = merkle_root;

    Ok(block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Transaction, TransactionInput, TransactionOutput, Script, UtxoId};

    #[test]
    fn test_genesis_block_creation() {
        let genesis = create_genesis_block().unwrap();
        
        assert_eq!(genesis.transactions.len(), 1);
        assert!(genesis.transactions[0].is_coinbase());
        assert_eq!(genesis.transactions[0].outputs[0].value, 5_000_000_000);
    }

    #[test]
    fn test_chain_state_creation() {
        let genesis = create_genesis_block().unwrap();
        let chain_state = ChainState::new_with_genesis(genesis).unwrap();
        
        assert_eq!(chain_state.height(), 1);
        assert!(chain_state.latest_block().is_some());
        assert_eq!(chain_state.utxo_set().len(), 1);
    }

    #[test]
    fn test_transaction_validation() {
        let genesis = create_genesis_block().unwrap();
        let chain_state = ChainState::new_with_genesis(genesis).unwrap();
        
        // Get the genesis UTXO
        let genesis_tx_hash = chain_state.latest_block().unwrap().transactions[0].hash().unwrap();
        let genesis_utxo_id = UtxoId::new(genesis_tx_hash, 0);
        
        // Create a valid transaction spending the genesis UTXO
        let input = TransactionInput::new(genesis_utxo_id, Script::empty(), 0);
        let output = TransactionOutput::new(1_000_000_000, Script::empty()); // 1 BND
        let tx = Transaction::new(1, vec![input], vec![output], 0);
        
        // This should validate successfully
        let result = chain_state.validate_transaction(&tx, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_utxo_management() {
        let genesis = create_genesis_block().unwrap();
        let chain_state = ChainState::new_with_genesis(genesis).unwrap();
        
        // Check genesis UTXO exists
        let genesis_tx_hash = chain_state.latest_block().unwrap().transactions[0].hash().unwrap();
        let genesis_utxo_id = UtxoId::new(genesis_tx_hash, 0);
        
        assert!(chain_state.has_utxo(&genesis_utxo_id));
        assert_eq!(chain_state.get_utxo(&genesis_utxo_id).unwrap().value, 5_000_000_000);
    }

    #[test]
    fn test_chain_stats() {
        let genesis = create_genesis_block().unwrap();
        let chain_state = ChainState::new_with_genesis(genesis).unwrap();
        
        let stats = chain_state.stats();
        assert_eq!(stats.height, 1);
        assert_eq!(stats.total_transactions, 1);
        assert_eq!(stats.utxo_count, 1);
        assert_eq!(stats.total_supply, 5_000_000_000);
    }

    #[test]
    fn test_double_spending_prevention() {
        let genesis = create_genesis_block().unwrap();
        let mut chain_state = ChainState::new_with_genesis(genesis).unwrap();
        
        let genesis_tx_hash = chain_state.latest_block().unwrap().transactions[0].hash().unwrap();
        let genesis_utxo_id = UtxoId::new(genesis_tx_hash, 0);
        
        // Create a transaction spending the genesis UTXO
        let input = TransactionInput::new(genesis_utxo_id, Script::empty(), 0);
        let output = TransactionOutput::new(1_000_000_000, Script::empty());
        let tx1 = Transaction::new(1, vec![input.clone()], vec![output.clone()], 0);
        
        // Apply the first transaction
        chain_state.apply_transaction(&tx1).unwrap();
        
        // Try to create another transaction spending the same UTXO
        let tx2 = Transaction::new(1, vec![input], vec![output], 0);
        
        // This should fail with double spending error
        let result = chain_state.apply_transaction(&tx2);
        assert!(result.is_err());
        
        if let Err(BondError::DoubleSpending { .. }) = result {
            // Expected error
        } else {
            panic!("Expected double spending error");
        }
    }
}
