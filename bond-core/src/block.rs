use crate::{BondError, BondResult, Transaction};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

/// Block header containing metadata and proof-of-work
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Version of the block format
    pub version: u32,
    /// Hash of the previous block
    pub previous_hash: BlockHash,
    /// Merkle root of all transactions in the block
    pub merkle_root: MerkleRoot,
    /// Block timestamp (Unix timestamp)
    pub timestamp: DateTime<Utc>,
    /// Current difficulty target for proof-of-work
    pub difficulty_target: DifficultyTarget,
    /// Nonce used for proof-of-work mining
    pub nonce: u64,
}

/// Complete block containing header and transactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// List of transactions in the block
    pub transactions: Vec<Transaction>,
}

/// 32-byte block hash (Keccak-256)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockHash([u8; 32]);

/// 32-byte Merkle root hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleRoot([u8; 32]);

/// Difficulty target for proof-of-work (higher value = easier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DifficultyTarget(pub [u8; 32]);

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        version: u32,
        previous_hash: BlockHash,
        merkle_root: MerkleRoot,
        timestamp: DateTime<Utc>,
        difficulty_target: DifficultyTarget,
        nonce: u64,
    ) -> Self {
        Self {
            version,
            previous_hash,
            merkle_root,
            timestamp,
            difficulty_target,
            nonce,
        }
    }

    /// Calculate the hash of this block header using Keccak-256
    pub fn hash(&self) -> BondResult<BlockHash> {
        let serialized = bincode::serialize(self)?;
        let mut hasher = Keccak256::new();
        hasher.update(&serialized);
        let hash = hasher.finalize();
        Ok(BlockHash(hash.into()))
    }

    /// Check if this block header satisfies the proof-of-work requirement
    pub fn validates_pow(&self) -> BondResult<bool> {
        let hash = self.hash()?;
        Ok(hash.meets_difficulty_target(self.difficulty_target))
    }
}

impl Block {
    /// Create a new block
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    /// Calculate the hash of this block
    pub fn hash(&self) -> BondResult<BlockHash> {
        self.header.hash()
    }

    /// Calculate the Merkle root of all transactions
    pub fn calculate_merkle_root(&self) -> BondResult<MerkleRoot> {
        if self.transactions.is_empty() {
            return Ok(MerkleRoot([0u8; 32]));
        }

        // Collect transaction hashes
        let mut hashes: Vec<[u8; 32]> = Vec::new();
        for tx in &self.transactions {
            hashes.push(*tx.hash()?.as_bytes());
        }

        // Build Merkle tree
        while hashes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in hashes.chunks(2) {
                let mut hasher = Keccak256::new();
                hasher.update(chunk[0]);

                // If odd number of hashes, duplicate the last one
                if chunk.len() == 2 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]);
                }

                let hash = hasher.finalize();
                next_level.push(hash.into());
            }

            hashes = next_level;
        }

        Ok(MerkleRoot(hashes[0]))
    }

    /// Validate the block structure and transactions
    pub fn validate(&self) -> BondResult<()> {
        // Validate proof-of-work
        if !self.header.validates_pow()? {
            let hash = self.hash()?;
            return Err(BondError::InvalidProofOfWork {
                hash: hash.to_string(),
                target: self.header.difficulty_target.to_string(),
            });
        }

        // Validate Merkle root
        let calculated_root = self.calculate_merkle_root()?;
        if calculated_root != self.header.merkle_root {
            return Err(BondError::InvalidBlockHash {
                expected: self.header.merkle_root.to_string(),
                actual: calculated_root.to_string(),
            });
        }

        // Validate each transaction
        for tx in &self.transactions {
            tx.validate()?;
        }

        Ok(())
    }

    /// Get the total size of the block in bytes
    pub fn size(&self) -> BondResult<usize> {
        let serialized = bincode::serialize(self)?;
        Ok(serialized.len())
    }
}

impl BlockHash {
    pub const ZERO: BlockHash = BlockHash([0u8; 32]);

    /// Create a new block hash from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Check if this hash meets the given difficulty target
    pub fn meets_difficulty_target(&self, target: DifficultyTarget) -> bool {
        self.0 <= target.0
    }
}

impl MerkleRoot {
    pub const ZERO: MerkleRoot = MerkleRoot([0u8; 32]);

    /// Create a new Merkle root from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of the root
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl DifficultyTarget {
    /// Maximum difficulty target (easiest mining)
    pub const MAX: DifficultyTarget = DifficultyTarget([0xFF; 32]);

    /// Minimum difficulty target (hardest mining)
    pub const MIN: DifficultyTarget = DifficultyTarget([0x00; 32]);

    /// Create a new difficulty target from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of the target
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::fmt::Display for MerkleRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::fmt::Display for DifficultyTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::str::FromStr for BlockHash {
    type Err = BondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| BondError::InvalidBlockHash {
            expected: "valid hex string".to_string(),
            actual: s.to_string(),
        })?;

        if bytes.len() != 32 {
            return Err(BondError::InvalidBlockHash {
                expected: "32 bytes".to_string(),
                actual: format!("{} bytes", bytes.len()),
            });
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(BlockHash(array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transaction;

    #[test]
    fn test_block_hash_calculation() {
        let header = BlockHeader::new(
            1,
            BlockHash::ZERO,
            MerkleRoot::ZERO,
            Utc::now(),
            DifficultyTarget::MAX,
            0,
        );

        let hash1 = header.hash().unwrap();
        let hash2 = header.hash().unwrap();

        // Hash should be deterministic
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_merkle_root_calculation() {
        let transactions = vec![
            Transaction::coinbase(100_000_000, vec![]), // Will implement coinbase later
        ];

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

        let merkle_root = block.calculate_merkle_root().unwrap();
        assert_ne!(merkle_root, MerkleRoot::ZERO);
    }

    #[test]
    fn test_difficulty_target_comparison() {
        let easy_target = DifficultyTarget([0xFF; 32]);
        let hard_target = DifficultyTarget([0x00; 32]);

        assert!(easy_target > hard_target);
        assert!(hard_target < easy_target);
    }
}
