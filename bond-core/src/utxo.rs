use crate::{BondError, BondResult, TransactionHash};
use serde::{Deserialize, Serialize};

/// Unique identifier for a UTXO (Unspent Transaction Output)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UtxoId {
    /// Transaction hash that created this UTXO
    pub tx_hash: TransactionHash,
    /// Output index within the transaction
    pub output_index: u32,
}

/// A programmable UTXO (pUTXO) with enhanced capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgrammableUtxo {
    /// Unique identifier for this UTXO
    pub id: UtxoId,
    /// Value in Elos (1 BND = 1000 Elos)
    pub value: u64,
    /// Authorization script defining spending conditions
    pub script_pubkey: Script,
    /// Block height when this UTXO was created
    pub block_height: u32,
    /// Additional metadata for programmable features
    pub metadata: UtxoMetadata,
}

/// Metadata for programmable UTXO features
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtxoMetadata {
    /// Recovery guardians (for social recovery)
    pub guardians: Vec<GuardianConfig>,
    /// Multi-factor authentication requirements
    pub mfa_config: Option<MfaConfig>,
    /// Time-based spending restrictions
    pub time_locks: Vec<TimeLock>,
    /// Spending rate limits
    pub rate_limits: Option<RateLimit>,
}

/// Guardian configuration for social recovery
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardianConfig {
    /// Guardian's public key (will be post-quantum)
    pub public_key: Vec<u8>, // Placeholder for ML-DSA public key
    /// Required confirmation period in blocks
    pub confirmation_period: u32,
    /// Guardian's weight in recovery decisions
    pub weight: u32,
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MfaConfig {
    /// Required number of factors for spending
    pub required_factors: u32,
    /// Available authentication methods
    pub methods: Vec<AuthMethod>,
}

/// Authentication methods for MFA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Hardware security key
    HardwareKey { public_key: Vec<u8> },
    /// Biometric authentication (hash of biometric template)
    Biometric { template_hash: [u8; 32] },
    /// Time-based one-time password
    Totp { shared_secret_hash: [u8; 32] },
    /// SMS verification (phone number hash)
    Sms { phone_hash: [u8; 32] },
}

/// Time-based spending restrictions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeLock {
    /// Lock type
    pub lock_type: TimeLockType,
    /// Value (block height or Unix timestamp)
    pub value: u64,
}

/// Types of time locks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeLockType {
    /// Absolute block height
    BlockHeight,
    /// Absolute Unix timestamp
    Timestamp,
    /// Relative blocks from UTXO creation
    RelativeBlocks,
    /// Relative time from UTXO creation
    RelativeTime,
}

/// Rate limiting configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum value that can be spent per time window
    pub max_value_per_window: u64,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Current window start timestamp
    pub current_window_start: u64,
    /// Value spent in current window
    pub spent_in_window: u64,
}

/// Simple script for basic authorization (will be enhanced in Sprint 5)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    /// Script bytecode
    pub code: Vec<u8>,
}

impl UtxoId {
    /// Special UTXO ID for coinbase transactions
    pub const COINBASE: UtxoId = UtxoId {
        tx_hash: TransactionHash::ZERO,
        output_index: 0xFFFFFFFF,
    };

    /// Create a new UTXO ID
    pub fn new(tx_hash: TransactionHash, output_index: u32) -> Self {
        Self {
            tx_hash,
            output_index,
        }
    }

    /// Convert UTXO ID to bytes for hashing/signing
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.tx_hash.as_bytes());
        bytes.extend_from_slice(&self.output_index.to_le_bytes());
        bytes
    }
}

impl ProgrammableUtxo {
    /// Create a new programmable UTXO
    pub fn new(id: UtxoId, value: u64, script_pubkey: Script, block_height: u32) -> Self {
        Self {
            id,
            value,
            script_pubkey,
            block_height,
            metadata: UtxoMetadata::default(),
        }
    }

    /// Create a simple UTXO without programmable features
    pub fn simple(id: UtxoId, value: u64, script_pubkey: Script, block_height: u32) -> Self {
        Self::new(id, value, script_pubkey, block_height)
    }

    /// Check if this UTXO can be spent given current conditions
    pub fn can_spend(&self, current_block_height: u32, current_timestamp: u64) -> BondResult<bool> {
        // Check time locks
        for time_lock in &self.metadata.time_locks {
            if !self.check_time_lock(time_lock, current_block_height, current_timestamp)? {
                return Ok(false);
            }
        }

        // Check rate limits
        if let Some(rate_limit) = &self.metadata.rate_limits {
            if !self.check_rate_limit(rate_limit, current_timestamp)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a time lock is satisfied
    fn check_time_lock(
        &self,
        time_lock: &TimeLock,
        current_block_height: u32,
        current_timestamp: u64,
    ) -> BondResult<bool> {
        match time_lock.lock_type {
            TimeLockType::BlockHeight => Ok(current_block_height >= time_lock.value as u32),
            TimeLockType::Timestamp => Ok(current_timestamp >= time_lock.value),
            TimeLockType::RelativeBlocks => {
                let target_height = self
                    .block_height
                    .checked_add(time_lock.value as u32)
                    .ok_or_else(|| BondError::ArithmeticOverflow {
                        operation: "relative block height calculation".to_string(),
                    })?;
                Ok(current_block_height >= target_height)
            }
            TimeLockType::RelativeTime => {
                // For relative time, we'd need the block timestamp when UTXO was created
                // This is a simplification for now
                Ok(true)
            }
        }
    }

    /// Check if rate limit allows spending
    fn check_rate_limit(&self, rate_limit: &RateLimit, current_timestamp: u64) -> BondResult<bool> {
        let window_end = rate_limit
            .current_window_start
            .checked_add(rate_limit.window_seconds)
            .ok_or_else(|| BondError::ArithmeticOverflow {
                operation: "rate limit window calculation".to_string(),
            })?;

        if current_timestamp >= window_end {
            // New window, spending is allowed
            Ok(true)
        } else {
            // Same window, check if under limit
            Ok(rate_limit.spent_in_window < rate_limit.max_value_per_window)
        }
    }

    /// Add a guardian to this UTXO
    pub fn add_guardian(&mut self, guardian: GuardianConfig) {
        self.metadata.guardians.push(guardian);
    }

    /// Set MFA configuration
    pub fn set_mfa_config(&mut self, mfa_config: MfaConfig) {
        self.metadata.mfa_config = Some(mfa_config);
    }

    /// Add a time lock
    pub fn add_time_lock(&mut self, time_lock: TimeLock) {
        self.metadata.time_locks.push(time_lock);
    }

    /// Set rate limiting
    pub fn set_rate_limit(&mut self, rate_limit: RateLimit) {
        self.metadata.rate_limits = Some(rate_limit);
    }
}

impl UtxoMetadata {
    /// Create empty metadata
    pub fn new() -> Self {
        Self {
            guardians: Vec::new(),
            mfa_config: None,
            time_locks: Vec::new(),
            rate_limits: None,
        }
    }
}

impl Default for UtxoMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Script {
    /// Create a new script
    pub fn new(code: Vec<u8>) -> Self {
        Self { code }
    }

    /// Create an empty script
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Check if the script is empty
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Get the script size in bytes
    pub fn size(&self) -> usize {
        self.code.len()
    }

    /// Get the script data (raw bytes)
    pub fn data(&self) -> &[u8] {
        &self.code
    }

    /// Get mutable access to script data
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }
}

impl std::fmt::Display for UtxoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tx_hash, self.output_index)
    }
}

impl std::str::FromStr for UtxoId {
    type Err = BondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(BondError::InvalidUtxo {
                reason: format!("Invalid UTXO ID format: {}", s),
            });
        }

        let tx_hash = parts[0].parse().map_err(|_| BondError::InvalidUtxo {
            reason: format!("Invalid transaction hash in UTXO ID: {}", parts[0]),
        })?;

        let output_index = parts[1].parse().map_err(|_| BondError::InvalidUtxo {
            reason: format!("Invalid output index in UTXO ID: {}", parts[1]),
        })?;

        Ok(UtxoId::new(tx_hash, output_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionHash;

    #[test]
    fn test_utxo_id_creation() {
        let tx_hash = TransactionHash::ZERO;
        let utxo_id = UtxoId::new(tx_hash, 0);

        assert_eq!(utxo_id.tx_hash, tx_hash);
        assert_eq!(utxo_id.output_index, 0);
    }

    #[test]
    fn test_programmable_utxo_creation() {
        let utxo_id = UtxoId::new(TransactionHash::ZERO, 0);
        let script = Script::empty();
        let utxo = ProgrammableUtxo::simple(utxo_id, 1000, script, 100);

        assert_eq!(utxo.id, utxo_id);
        assert_eq!(utxo.value, 1000);
        assert_eq!(utxo.block_height, 100);
    }

    #[test]
    fn test_time_lock_validation() {
        let utxo_id = UtxoId::new(TransactionHash::ZERO, 0);
        let script = Script::empty();
        let mut utxo = ProgrammableUtxo::simple(utxo_id, 1000, script, 100);

        // Add a block height time lock
        utxo.add_time_lock(TimeLock {
            lock_type: TimeLockType::BlockHeight,
            value: 200,
        });

        // Should not be spendable before block 200
        assert!(!utxo.can_spend(150, 0).unwrap());

        // Should be spendable at block 200 or later
        assert!(utxo.can_spend(200, 0).unwrap());
    }

    #[test]
    fn test_utxo_id_display() {
        let utxo_id = UtxoId::new(TransactionHash::ZERO, 5);
        let display = format!("{}", utxo_id);

        assert!(display.contains(":5"));
    }
}
