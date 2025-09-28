use anyhow::Result;
use bond_core::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bond-cli")]
#[command(about = "Bond Protocol CLI - Post-Quantum Store of Value")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and display the genesis block
    Genesis,
    /// Mine a block
    Mine {
        /// Mining difficulty (lower = easier)
        #[arg(short, long, default_value = "1000")]
        difficulty: u32,
    },
    /// Validate a blockchain simulation
    Validate,
    /// Show mining and blockchain statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Genesis => handle_genesis().await,
        Commands::Mine { difficulty } => handle_mining(difficulty).await,
        Commands::Validate => handle_validation().await,
        Commands::Stats => handle_stats().await,
    }
}

async fn handle_genesis() -> Result<()> {
    println!("🔨 Creating Bond Genesis Block...\n");

    let genesis = create_genesis_block()?;
    let hash = genesis.hash()?;

    println!("✅ Genesis Block Created Successfully!");
    println!("📋 Block Details:");
    println!("   Hash: {}", hash);
    println!("   Transactions: {}", genesis.transactions.len());
    println!(
        "   Coinbase Reward: {} Elos",
        genesis.transactions[0].outputs[0].value
    );
    println!("   Block Size: {} bytes", genesis.size()?);
    println!("   Timestamp: {}", genesis.header.timestamp);

    // Show the coinbase message
    if let Some(coinbase_data) = genesis.transactions[0].inputs.first() {
        if let Ok(message) = String::from_utf8(coinbase_data.script_sig.code.clone()) {
            println!("   Genesis Message: \"{}\"", message);
        }
    }

    Ok(())
}

async fn handle_mining(difficulty: u32) -> Result<()> {
    println!("⛏️  Starting Bond Mining Simulation...\n");

    // Create a simple difficulty target
    let mut target_bytes = [0xFF; 32];
    // Make it easier by setting leading bytes to lower values
    for i in 0..(difficulty.min(4)) {
        target_bytes[i as usize] = 0x0F;
    }
    let difficulty_target = DifficultyTarget::from_bytes(target_bytes);

    // Create a mining header
    let header = create_mining_header(1, BlockHash::ZERO, MerkleRoot::ZERO, difficulty_target);

    println!("⚙️  Mining Parameters:");
    println!("   Target: {}", difficulty_target);
    println!("   Difficulty Level: {}", difficulty);

    // Start mining
    let mut miner = Miner::new();
    let start_time = std::time::Instant::now();

    println!("\n🔥 Mining in progress...");

    match miner.mine_block(header) {
        Ok(mined_header) => {
            let elapsed = start_time.elapsed();
            let stats = miner.stats();

            println!("✅ Block Mined Successfully!");
            println!("📊 Mining Results:");
            println!("   Nonce Found: {}", mined_header.nonce);
            println!("   Block Hash: {}", mined_header.hash()?);
            println!("   Time Elapsed: {:.2}s", elapsed.as_secs_f64());
            println!("   Hashes Attempted: {}", stats.hashes_attempted);
            println!("   Hash Rate: {:.0} H/s", stats.current_hash_rate());

            // Verify the proof-of-work
            if mined_header.validates_pow()? {
                println!("✅ Proof-of-Work Valid!");
            } else {
                println!("❌ Proof-of-Work Invalid!");
            }
        }
        Err(e) => {
            println!("❌ Mining Failed: {}", e);
        }
    }

    Ok(())
}

async fn handle_validation() -> Result<()> {
    println!("🔍 Running Bond Blockchain Validation Simulation...\n");

    // Create genesis block
    let genesis = create_genesis_block()?;
    println!("✅ Genesis block created");

    // Initialize chain state
    let chain_state = ChainState::new_with_genesis(genesis)?;
    println!("✅ Chain state initialized");

    // Validate the chain
    chain_state.validate_chain()?;
    println!("✅ Chain validation passed");

    // Show chain statistics
    let stats = chain_state.stats();
    println!("\n📊 Blockchain Statistics:");
    println!("   Height: {}", stats.height);
    println!("   Total Transactions: {}", stats.total_transactions);
    println!("   UTXO Count: {}", stats.utxo_count);
    println!(
        "   Total Supply: {} Elos ({} BND)",
        stats.total_supply,
        stats.total_supply / 1000
    );
    println!("   Average Block Time: {:.1}s", stats.average_block_time);

    // Show sample UTXO details
    if let Some((utxo_id, utxo)) = chain_state.utxo_set().iter().next() {
        println!("\n💰 Sample UTXO:");
        println!("   ID: {}", utxo_id);
        println!("   Value: {} Elos", utxo.value);
        println!("   Script Size: {} bytes", utxo.script_pubkey.size());
    }

    Ok(())
}

async fn handle_stats() -> Result<()> {
    println!("📊 Bond Protocol Statistics & Information\n");

    println!("🏗️  Protocol Architecture:");
    println!("   Name: Bond (BND)");
    println!("   Purpose: Post-Quantum Store of Value");
    println!("   Consensus: Proof-of-Work (PoW)");
    println!("   Hashing: Keccak-256 (SHA-3 variant)");
    println!("   Cryptography: ML-DSA-65 (NIST Level 3) [Coming in Sprint 2]");
    println!("   Block Time: 10 minutes");
    println!("   Block Size: 4 MB (for post-quantum signatures)");

    println!("\n💰 Economics:");
    println!("   Base Unit: Elo");
    println!("   Denomination: 1 BND = 1,000 Elos");
    println!("   Inflation: Adaptive (1.84% - 3.72% annually)");
    println!("   Fee Structure: 1 Elo per 250 bytes");

    println!("\n🔐 Security Features:");
    println!("   Programmable UTXOs (pUTXOs)");
    println!("   Social Recovery");
    println!("   Multi-Factor Authentication (MFA)");
    println!("   Time-based Locks");
    println!("   Rate Limiting");

    println!("\n🎯 Sprint Progress:");
    println!("   ✅ Sprint 1: Core Foundation (COMPLETE)");
    println!("   🚧 Sprint 2: Post-Quantum Security (NEXT)");
    println!("   ⏳ Sprint 3: P2P Network Foundation");
    println!("   ⏳ Sprint 4: Decentralized Consensus");

    println!("\n🌐 Ecosystem:");
    println!("   Aevum (AEV): High-speed transactional layer");
    println!("   Bond (BND): Secure store of value layer");
    println!("   Website: https://aevum.bond");

    Ok(())
}

/// Create a mining-ready block header with sensible defaults
pub fn create_mining_header(
    version: u32,
    previous_hash: BlockHash,
    merkle_root: MerkleRoot,
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
