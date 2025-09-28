use crate::{BondError, BondResult, Script, UtxoId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use shared_crypto::{signature, PublicKey, SecurityLevel, Signature};

/// Transaction hash (Keccak-256)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionHash([u8; 32]);

/// A complete transaction in the Bond protocol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction version
    pub version: u32,
    /// Transaction inputs (UTXOs being spent)
    pub inputs: Vec<TransactionInput>,
    /// Transaction outputs (new UTXOs being created)
    pub outputs: Vec<TransactionOutput>,
    /// Transaction locktime (block height or timestamp)
    pub locktime: u32,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
}

/// Transaction input referencing a UTXO to be spent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Reference to the UTXO being spent
    pub previous_output: UtxoId,
    /// Authorization script proving ownership
    pub script_sig: Script,
    /// Sequence number for replacement and locktime
    pub sequence: u32,
}

/// Transaction output creating a new UTXO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// Amount in Elos (1 BND = 1000 Elos)
    pub value: u64,
    /// Authorization script defining spending conditions
    pub script_pubkey: Script,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        version: u32,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        locktime: u32,
    ) -> Self {
        Self {
            version,
            inputs,
            outputs,
            locktime,
            timestamp: Utc::now(),
        }
    }

    /// Create a coinbase transaction (mining reward)
    pub fn coinbase(reward: u64, extra_data: Vec<u8>) -> Self {
        let coinbase_input = TransactionInput {
            previous_output: UtxoId::COINBASE,
            script_sig: Script::new(extra_data),
            sequence: 0xFFFFFFFF,
        };

        // For now, create a simple Pay-to-Public-Key-Hash script
        // This will be replaced with proper pUTXO scripts in Sprint 5
        let output = TransactionOutput {
            value: reward,
            script_pubkey: Script::new(vec![]), // Placeholder
        };

        Self::new(1, vec![coinbase_input], vec![output], 0)
    }

    /// Calculate the hash of this transaction
    pub fn hash(&self) -> BondResult<TransactionHash> {
        let serialized = bincode::serialize(self)?;
        let mut hasher = Keccak256::new();
        hasher.update(&serialized);
        let hash = hasher.finalize();
        Ok(TransactionHash(hash.into()))
    }

    /// Get the transaction ID (same as hash for now)
    pub fn id(&self) -> BondResult<TransactionHash> {
        self.hash()
    }

    /// Check if this is a coinbase transaction
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].previous_output == UtxoId::COINBASE
    }

    /// Calculate the total input value (requires UTXO set for validation)
    pub fn total_input_value(
        &self,
        utxo_set: &std::collections::HashMap<UtxoId, TransactionOutput>,
    ) -> BondResult<u64> {
        if self.is_coinbase() {
            return Ok(0); // Coinbase has no input value
        }

        let mut total = 0u64;
        for input in &self.inputs {
            if let Some(utxo) = utxo_set.get(&input.previous_output) {
                total =
                    total
                        .checked_add(utxo.value)
                        .ok_or_else(|| BondError::ArithmeticOverflow {
                            operation: "input value sum".to_string(),
                        })?;
            } else {
                return Err(BondError::InvalidTransaction {
                    reason: format!("Referenced UTXO not found: {}", input.previous_output),
                });
            }
        }
        Ok(total)
    }

    /// Calculate the total output value
    pub fn total_output_value(&self) -> BondResult<u64> {
        let mut total = 0u64;
        for output in &self.outputs {
            total =
                total
                    .checked_add(output.value)
                    .ok_or_else(|| BondError::ArithmeticOverflow {
                        operation: "output value sum".to_string(),
                    })?;
        }
        Ok(total)
    }

    /// Calculate the transaction fee
    pub fn fee(
        &self,
        utxo_set: &std::collections::HashMap<UtxoId, TransactionOutput>,
    ) -> BondResult<u64> {
        if self.is_coinbase() {
            return Ok(0); // Coinbase transactions have no fee
        }

        let input_value = self.total_input_value(utxo_set)?;
        let output_value = self.total_output_value()?;

        if input_value < output_value {
            return Err(BondError::InvalidTransaction {
                reason: "Output value exceeds input value".to_string(),
            });
        }

        Ok(input_value - output_value)
    }

    /// Validate the transaction structure and basic rules
    pub fn validate(&self) -> BondResult<()> {
        // Version check
        if self.version == 0 {
            return Err(BondError::InvalidTransaction {
                reason: "Transaction version cannot be zero".to_string(),
            });
        }

        // Input/output checks
        if !self.is_coinbase() && self.inputs.is_empty() {
            return Err(BondError::InvalidTransaction {
                reason: "Non-coinbase transaction must have at least one input".to_string(),
            });
        }

        if self.outputs.is_empty() {
            return Err(BondError::InvalidTransaction {
                reason: "Transaction must have at least one output".to_string(),
            });
        }

        // Coinbase validation
        if self.is_coinbase() && self.inputs.len() != 1 {
            return Err(BondError::InvalidTransaction {
                reason: "Coinbase transaction must have exactly one input".to_string(),
            });
        }

        // Value validation
        for output in &self.outputs {
            if output.value == 0 {
                return Err(BondError::InvalidTransaction {
                    reason: "Transaction output cannot have zero value".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get the size of the transaction in bytes
    pub fn size(&self) -> BondResult<usize> {
        let serialized = bincode::serialize(self)?;
        Ok(serialized.len())
    }

    /// Validate all signatures in the transaction
    /// This method will be enhanced in Sprint 5 with full pUTXO script validation
    pub fn validate_signatures(
        &self,
        utxo_set: &std::collections::HashMap<UtxoId, TransactionOutput>,
    ) -> BondResult<()> {
        if self.is_coinbase() {
            return Ok(()); // Coinbase transactions don't have signatures to validate
        }

        // For each input, validate the signature against the referenced UTXO
        for input in &self.inputs {
            let utxo = utxo_set.get(&input.previous_output).ok_or_else(|| {
                BondError::InvalidTransaction {
                    reason: format!("Referenced UTXO not found: {}", input.previous_output),
                }
            })?;

            // Get the message to be signed (transaction hash without signatures)
            let message = self.signature_hash(input)?;

            // Extract signature and public key from scripts (simplified for Sprint 2)
            // This will be replaced with full pUTXO script validation in Sprint 5
            if let Some((signature, public_key)) = self.extract_signature_and_key(input, utxo)? {
                let is_valid =
                    signature::verify(&signature, &message, &public_key).map_err(|e| {
                        BondError::CryptographicError {
                            reason: e.to_string(),
                        }
                    })?;

                if !is_valid {
                    return Err(BondError::InvalidTransaction {
                        reason: format!("Invalid signature for input: {}", input.previous_output),
                    });
                }
            }
        }

        Ok(())
    }

    /// Calculate the signature hash for a specific input
    /// This is the message that gets signed
    pub fn signature_hash(&self, input: &TransactionInput) -> BondResult<Vec<u8>> {
        // Create a copy of the transaction with all script_sig fields cleared
        let mut tx_copy = self.clone();
        for input_copy in &mut tx_copy.inputs {
            input_copy.script_sig = Script::new(vec![]);
        }

        // Serialize and hash
        let serialized = bincode::serialize(&tx_copy)?;
        let mut hasher = Keccak256::new();
        hasher.update(&serialized);
        hasher.update(input.previous_output.to_bytes());
        Ok(hasher.finalize().to_vec())
    }

    /// Extract signature and public key from transaction scripts
    /// TEMPORARY: Simplified implementation for Sprint 2
    /// TODO: Replace with full pUTXO script execution in Sprint 5
    pub fn extract_signature_and_key(
        &self,
        input: &TransactionInput,
        _utxo: &TransactionOutput, // Will be used in Sprint 5 for full pUTXO validation
    ) -> BondResult<Option<(Signature, PublicKey)>> {
        // For Sprint 2, we use a simplified signature format:
        // script_sig contains: [signature_bytes] [public_key_bytes]
        // script_pubkey contains: [public_key_hash] (for validation)

        let script_sig_data = input.script_sig.data();

        // Check if we have enough data for ML-DSA-65 (Bond uses Level 3 security)
        let expected_sig_size = SecurityLevel::Level3.signature_size();
        let expected_key_size = SecurityLevel::Level3.public_key_size();

        if script_sig_data.len() < expected_sig_size + expected_key_size {
            return Ok(None); // Not a signature script
        }

        // Extract signature and public key
        let signature_bytes = &script_sig_data[..expected_sig_size];
        let public_key_bytes =
            &script_sig_data[expected_sig_size..expected_sig_size + expected_key_size];

        let signature = Signature::from_bytes(signature_bytes.to_vec(), SecurityLevel::Level3)
            .map_err(|e| BondError::CryptographicError {
                reason: e.to_string(),
            })?;

        let public_key = PublicKey::from_bytes(public_key_bytes.to_vec(), SecurityLevel::Level3)
            .map_err(|e| BondError::CryptographicError {
                reason: e.to_string(),
            })?;

        Ok(Some((signature, public_key)))
    }
}

impl TransactionInput {
    /// Create a new transaction input
    pub fn new(previous_output: UtxoId, script_sig: Script, sequence: u32) -> Self {
        Self {
            previous_output,
            script_sig,
            sequence,
        }
    }
}

impl TransactionOutput {
    /// Create a new transaction output
    pub fn new(value: u64, script_pubkey: Script) -> Self {
        Self {
            value,
            script_pubkey,
        }
    }
}

impl TransactionHash {
    pub const ZERO: TransactionHash = TransactionHash([0u8; 32]);

    /// Create a new transaction hash from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::str::FromStr for TransactionHash {
    type Err = BondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| BondError::InvalidTransaction {
            reason: format!("Invalid transaction hash: {}", s),
        })?;

        if bytes.len() != 32 {
            return Err(BondError::InvalidTransaction {
                reason: format!("Transaction hash must be 32 bytes, got {}", bytes.len()),
            });
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(TransactionHash(array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coinbase_transaction() {
        let coinbase = Transaction::coinbase(5_000_000_000, b"Genesis Block".to_vec());

        assert!(coinbase.is_coinbase());
        assert_eq!(coinbase.inputs.len(), 1);
        assert_eq!(coinbase.outputs.len(), 1);
        assert_eq!(coinbase.outputs[0].value, 5_000_000_000);
    }

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::coinbase(1000, vec![]);
        let hash1 = tx.hash().unwrap();
        let hash2 = tx.hash().unwrap();

        // Hash should be deterministic
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, TransactionHash::ZERO);
    }

    #[test]
    fn test_transaction_validation() {
        let valid_tx = Transaction::coinbase(1000, vec![]);
        assert!(valid_tx.validate().is_ok());

        // Test invalid transaction (no outputs)
        let invalid_tx = Transaction::new(1, vec![], vec![], 0);
        assert!(invalid_tx.validate().is_err());
    }

    #[test]
    fn test_total_output_value() {
        let tx = Transaction::coinbase(1000, vec![]);
        assert_eq!(tx.total_output_value().unwrap(), 1000);
    }
}
