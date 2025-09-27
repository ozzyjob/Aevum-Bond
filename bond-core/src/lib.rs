//! Bond Protocol Core
//! 
//! This crate implements the Bond blockchain protocol, a post-quantum 
//! Proof-of-Work store of value with programmable UTXOs (pUTXOs).
//!
//! # Architecture
//! 
//! - **Consensus**: Proof-of-Work with Keccak-256 hashing
//! - **Block Time**: 10 minutes
//! - **Block Size**: 4 MB (to accommodate post-quantum signatures)
//! - **Cryptography**: ML-DSA-65 (NIST Level 3 security)
//! - **Model**: Programmable UTXO (pUTXO)

pub mod block;
pub mod transaction;
pub mod utxo;
pub mod mining;
pub mod consensus;
pub mod script;
pub mod error;

pub use block::*;
pub use transaction::*;
pub use utxo::*;
pub use mining::*;
pub use consensus::*;
pub use script::*;
pub use error::*;
