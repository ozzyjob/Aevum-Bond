# 📚 API Reference - Aevum & Bond

## 🎯 Overview
Comprehensive API documentation for the Aevum & Bond dual-ledger blockchain ecosystem.

## 🏗️ API Architecture

### Core Components
- **Bond Chain API**: PoW blockchain with pUTXO model
- **Aevum Chain API**: PoD blockchain with account model  
- **Shared Crypto API**: Post-quantum cryptographic primitives
- **P2P Network API**: Peer-to-peer networking functionality
- **CLI Tools API**: Command-line interface components

## 🔗 Bond Chain API

### Block Management

#### `Block`
Represents a block in the Bond blockchain.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Creates a new block with the given transactions and previous hash.
    pub fn new(
        transactions: Vec<Transaction>,
        previous_hash: [u8; 32],
        timestamp: u64,
    ) -> Self;

    /// Validates the block structure and contents.
    pub fn is_valid(&self) -> bool;

    /// Calculates the block hash.
    pub fn hash(&self) -> [u8; 32];

    /// Calculates the merkle root of all transactions.
    pub fn merkle_root(&self) -> [u8; 32];

    /// Returns the block size in bytes.
    pub fn size(&self) -> usize;
}
```

#### `BlockHeader`
Block metadata and proof-of-work information.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u64,
    pub difficulty: u32,
    pub nonce: u64,
}

impl BlockHeader {
    /// Creates a new block header.
    pub fn new(
        version: u32,
        previous_hash: [u8; 32],
        merkle_root: [u8; 32],
        timestamp: u64,
        difficulty: u32,
    ) -> Self;

    /// Validates the proof-of-work for this header.
    pub fn validate_pow(&self) -> bool;

    /// Calculates the header hash.
    pub fn hash(&self) -> [u8; 32];
}
```

### Transaction Management

#### `Transaction`
Bond chain transaction with UTXO model.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub timestamp: u64,
}

impl Transaction {
    /// Creates a new transaction.
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> Self;

    /// Calculates the transaction hash.
    pub fn hash(&self) -> [u8; 32];

    /// Validates the transaction structure.
    pub fn is_valid(&self) -> bool;

    /// Calculates total input value.
    pub fn input_value(&self, utxo_set: &UTXOSet) -> Result<u64, Error>;

    /// Calculates total output value.
    pub fn output_value(&self) -> u64;

    /// Calculates transaction fee.
    pub fn fee(&self, utxo_set: &UTXOSet) -> Result<u64, Error>;

    /// Signs the transaction with the given private key.
    pub fn sign(&mut self, private_key: &PrivateKey) -> Result<(), Error>;

    /// Verifies all signatures in the transaction.
    pub fn verify_signatures(&self, utxo_set: &UTXOSet) -> Result<bool, Error>;
}
```

#### `TransactionInput`
References a previous transaction output.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl TransactionInput {
    /// Creates a new transaction input.
    pub fn new(previous_output: OutPoint) -> Self;

    /// Signs this input with the given private key.
    pub fn sign(&mut self, private_key: &PrivateKey, tx_hash: &[u8; 32]) -> Result<(), Error>;

    /// Verifies the signature for this input.
    pub fn verify_signature(&self, tx_hash: &[u8; 32]) -> Result<bool, Error>;
}
```

#### `TransactionOutput`
Creates a new unspent output.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionOutput {
    pub value: u64,
    pub recipient: [u8; 32],
}

impl TransactionOutput {
    /// Creates a new transaction output.
    pub fn new(value: u64, recipient: [u8; 32]) -> Self;

    /// Checks if this output can be spent by the given public key.
    pub fn can_spend(&self, public_key: &PublicKey) -> bool;
}
```

### Mining Operations

#### `Miner`
Handles proof-of-work mining operations.

```rust
pub struct Miner {
    difficulty: u32,
    threads: usize,
}

impl Miner {
    /// Creates a new miner with the specified configuration.
    pub fn new(difficulty: u32, threads: usize) -> Self;

    /// Mines a block with the given transactions.
    pub async fn mine_block(
        &self,
        transactions: Vec<Transaction>,
        previous_hash: [u8; 32],
    ) -> Result<Block, MiningError>;

    /// Validates a proof-of-work solution.
    pub fn validate_pow(&self, block_header: &BlockHeader) -> bool;

    /// Calculates the current mining difficulty.
    pub fn calculate_difficulty(&self, blocks: &[Block]) -> u32;

    /// Estimates mining time for current difficulty.
    pub fn estimate_mining_time(&self) -> Duration;
}
```

### UTXO Management

#### `UTXOSet`
Manages the unspent transaction output set.

```rust
pub struct UTXOSet {
    utxos: HashMap<OutPoint, TransactionOutput>,
}

impl UTXOSet {
    /// Creates a new empty UTXO set.
    pub fn new() -> Self;

    /// Adds outputs from a transaction to the UTXO set.
    pub fn add_transaction(&mut self, tx: &Transaction) -> Result<(), Error>;

    /// Removes spent outputs from the UTXO set.
    pub fn spend_outputs(&mut self, inputs: &[TransactionInput]) -> Result<(), Error>;

    /// Gets a UTXO by its outpoint.
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<&TransactionOutput>;

    /// Checks if a UTXO exists.
    pub fn contains(&self, outpoint: &OutPoint) -> bool;

    /// Returns the total value in the UTXO set.
    pub fn total_value(&self) -> u64;

    /// Returns the number of UTXOs.
    pub fn len(&self) -> usize;

    /// Validates the entire UTXO set.
    pub fn validate(&self) -> Result<(), Error>;
}
```

## ⚡ Aevum Chain API

### Account Management

#### `Account`
Represents an account in the Aevum blockchain.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub address: [u8; 32],
    pub balance: u64,
    pub nonce: u64,
    pub code: Option<Vec<u8>>,
    pub storage: HashMap<[u8; 32], [u8; 32]>,
}

impl Account {
    /// Creates a new account with the given address.
    pub fn new(address: [u8; 32]) -> Self;

    /// Checks if this is a contract account.
    pub fn is_contract(&self) -> bool;

    /// Gets a storage value by key.
    pub fn get_storage(&self, key: &[u8; 32]) -> Option<[u8; 32]>;

    /// Sets a storage value.
    pub fn set_storage(&mut self, key: [u8; 32], value: [u8; 32]);

    /// Calculates the account hash for state root.
    pub fn hash(&self) -> [u8; 32];
}
```

#### `AevumTransaction`
Account-based transaction for the Aevum chain.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AevumTransaction {
    pub from: [u8; 32],
    pub to: Option<[u8; 32]>,
    pub value: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Vec<u8>,
    pub nonce: u64,
    pub signature: [u8; 64],
}

impl AevumTransaction {
    /// Creates a new Aevum transaction.
    pub fn new(
        from: [u8; 32],
        to: Option<[u8; 32]>,
        value: u64,
        gas_limit: u64,
        gas_price: u64,
        data: Vec<u8>,
        nonce: u64,
    ) -> Self;

    /// Calculates the transaction hash.
    pub fn hash(&self) -> [u8; 32];

    /// Signs the transaction with the given private key.
    pub fn sign(&mut self, private_key: &PrivateKey) -> Result<(), Error>;

    /// Verifies the transaction signature.
    pub fn verify_signature(&self) -> Result<bool, Error>;

    /// Calculates the maximum transaction cost.
    pub fn max_cost(&self) -> u64;

    /// Checks if this is a contract creation transaction.
    pub fn is_contract_creation(&self) -> bool;
}
```

### State Management

#### `WorldState`
Manages the global state of the Aevum blockchain.

```rust
pub struct WorldState {
    accounts: HashMap<[u8; 32], Account>,
    state_root: [u8; 32],
    block_number: u64,
    gas_used: u64,
}

impl WorldState {
    /// Creates a new world state.
    pub fn new() -> Self;

    /// Gets an account by address.
    pub fn get_account(&self, address: &[u8; 32]) -> Option<&Account>;

    /// Gets a mutable reference to an account.
    pub fn get_account_mut(&mut self, address: &[u8; 32]) -> Option<&mut Account>;

    /// Creates a new account or returns existing one.
    pub fn get_or_create_account(&mut self, address: [u8; 32]) -> &mut Account;

    /// Transfers value from one account to another.
    pub fn transfer(
        &mut self,
        from: &[u8; 32],
        to: &[u8; 32],
        value: u64,
    ) -> Result<(), Error>;

    /// Applies a transaction to the world state.
    pub fn apply_transaction(&mut self, tx: &AevumTransaction) -> Result<Receipt, Error>;

    /// Calculates the state root hash.
    pub fn calculate_state_root(&self) -> [u8; 32];

    /// Validates the entire world state.
    pub fn validate(&self) -> Result<(), Error>;

    /// Creates a snapshot of the current state.
    pub fn snapshot(&self) -> StateSnapshot;

    /// Reverts to a previous snapshot.
    pub fn revert(&mut self, snapshot: StateSnapshot);
}
```

### Consensus (Proof of Delegation)

#### `Validator`
Represents a validator in the PoD consensus.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validator {
    pub address: [u8; 32],
    pub stake: u64,
    pub delegated_stake: u64,
    pub commission_rate: u16,
    pub is_active: bool,
    pub last_block_signed: u64,
}

impl Validator {
    /// Creates a new validator.
    pub fn new(address: [u8; 32], stake: u64, commission_rate: u16) -> Self;

    /// Calculates total voting power.
    pub fn voting_power(&self) -> u64;

    /// Checks if the validator is eligible to propose.
    pub fn can_propose(&self, current_block: u64) -> bool;

    /// Calculates validator rewards.
    pub fn calculate_rewards(&self, block_reward: u64) -> (u64, u64);

    /// Applies slashing for malicious behavior.
    pub fn slash(&mut self, amount: u64);
}
```

#### `ValidatorSet`
Manages the active validator set.

```rust
pub struct ValidatorSet {
    validators: Vec<Validator>,
    total_stake: u64,
}

impl ValidatorSet {
    /// Creates a new validator set.
    pub fn new() -> Self;

    /// Adds a validator to the set.
    pub fn add_validator(&mut self, validator: Validator) -> Result<(), Error>;

    /// Removes a validator from the set.
    pub fn remove_validator(&mut self, address: &[u8; 32]) -> Result<(), Error>;

    /// Selects the next block proposer.
    pub fn select_proposer(&self, block_number: u64, randomness: &[u8; 32]) -> Option<&Validator>;

    /// Updates validator stakes.
    pub fn update_stakes(&mut self, stake_updates: Vec<(Address, u64)>);

    /// Gets validators by voting power.
    pub fn by_voting_power(&self) -> Vec<&Validator>;

    /// Calculates the supermajority threshold.
    pub fn supermajority_threshold(&self) -> u64;
}
```

## 🔐 Shared Crypto API

### Post-Quantum Signatures

#### `MLDSAKeyPair`
ML-DSA key pair for post-quantum signatures.

```rust
pub struct MLDSAKeyPair {
    public_key: MLDSAPublicKey,
    private_key: MLDSAPrivateKey,
}

impl MLDSAKeyPair {
    /// Generates a new ML-DSA key pair.
    pub fn generate() -> Result<Self, CryptoError>;

    /// Creates a key pair from existing keys.
    pub fn from_keys(public_key: MLDSAPublicKey, private_key: MLDSAPrivateKey) -> Self;

    /// Gets the public key.
    pub fn public_key(&self) -> &MLDSAPublicKey;

    /// Signs a message with the private key.
    pub fn sign(&self, message: &[u8]) -> Result<MLDSASignature, CryptoError>;

    /// Verifies a signature against a message.
    pub fn verify(&self, message: &[u8], signature: &MLDSASignature) -> Result<bool, CryptoError>;

    /// Exports the key pair to bytes.
    pub fn to_bytes(&self) -> Vec<u8>;

    /// Imports a key pair from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError>;
}
```

#### `MLDSASignature`
ML-DSA signature for quantum-resistant authentication.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MLDSASignature {
    data: Vec<u8>,
}

impl MLDSASignature {
    /// Creates a signature from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self;

    /// Gets the signature as bytes.
    pub fn as_bytes(&self) -> &[u8];

    /// Verifies this signature against a message and public key.
    pub fn verify(
        &self,
        message: &[u8],
        public_key: &MLDSAPublicKey,
    ) -> Result<bool, CryptoError>;

    /// Returns the signature size in bytes.
    pub fn size(&self) -> usize;
}
```

### Hash Functions

#### `SHA3Hasher`
SHA-3 (Keccak) hash function implementation.

```rust
pub struct SHA3Hasher;

impl SHA3Hasher {
    /// Hashes the input data using SHA-3.
    pub fn hash(data: &[u8]) -> [u8; 32];

    /// Hashes multiple inputs.
    pub fn hash_multiple(inputs: &[&[u8]]) -> [u8; 32];

    /// Creates a hasher for streaming data.
    pub fn new_hasher() -> SHA3StreamHasher;
}

impl Hasher for SHA3Hasher {
    fn hash(&self, data: &[u8]) -> [u8; 32] {
        SHA3Hasher::hash(data)
    }

    fn merkle_root(&self, leaves: &[[u8; 32]]) -> [u8; 32] {
        calculate_merkle_root(leaves)
    }
}
```

### Address Generation

#### `AddressGenerator`
Generates blockchain addresses from public keys.

```rust
pub struct AddressGenerator;

impl AddressGenerator {
    /// Generates a Bond chain address from a public key.
    pub fn bond_address(public_key: &MLDSAPublicKey) -> [u8; 32];

    /// Generates an Aevum chain address from a public key.
    pub fn aevum_address(public_key: &MLDSAPublicKey) -> [u8; 32];

    /// Validates an address format.
    pub fn validate_address(address: &[u8; 32], chain_type: ChainType) -> bool;

    /// Converts an address to a human-readable format.
    pub fn address_to_string(address: &[u8; 32]) -> String;

    /// Parses an address from a string.
    pub fn address_from_string(address_str: &str) -> Result<[u8; 32], Error>;
}
```

## 🌐 P2P Network API

### Network Node

#### `NetworkNode`
Main P2P network node implementation.

```rust
pub struct NetworkNode {
    peer_id: PeerId,
    swarm: Swarm<NetworkBehaviour>,
    bond_chain: Arc<BondChain>,
    aevum_chain: Arc<AevumChain>,
}

impl NetworkNode {
    /// Creates a new network node.
    pub async fn new(
        bond_chain: Arc<BondChain>,
        aevum_chain: Arc<AevumChain>,
    ) -> Result<Self, NetworkError>;

    /// Starts the network node.
    pub async fn start(&mut self, listen_addr: Multiaddr) -> Result<(), NetworkError>;

    /// Stops the network node.
    pub async fn stop(&mut self) -> Result<(), NetworkError>;

    /// Connects to a peer.
    pub async fn connect_peer(&mut self, peer_addr: Multiaddr) -> Result<(), NetworkError>;

    /// Disconnects from a peer.
    pub async fn disconnect_peer(&mut self, peer_id: PeerId) -> Result<(), NetworkError>;

    /// Broadcasts a message to all peers.
    pub async fn broadcast(&mut self, message: NetworkMessage) -> Result<(), NetworkError>;

    /// Sends a message to a specific peer.
    pub async fn send_to_peer(
        &mut self,
        peer_id: PeerId,
        message: NetworkMessage,
    ) -> Result<(), NetworkError>;

    /// Gets information about connected peers.
    pub fn connected_peers(&self) -> Vec<PeerInfo>;

    /// Gets network statistics.
    pub fn network_stats(&self) -> NetworkStats;
}
```

### Message Types

#### `NetworkMessage`
Network protocol messages.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    // Bond chain messages
    BondBlock(Block),
    BondTransaction(Transaction),
    BondBlockRequest(BlockRequest),
    BondBlockResponse(BlockResponse),
    
    // Aevum chain messages
    AevumBlock(AevumBlock),
    AevumTransaction(AevumTransaction),
    AevumStateRequest(StateRequest),
    AevumStateResponse(StateResponse),
    
    // General network messages
    PeerRequest(PeerRequest),
    PeerResponse(PeerResponse),
    Ping,
    Pong,
}

impl NetworkMessage {
    /// Serializes the message to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, NetworkError>;

    /// Deserializes a message from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, NetworkError>;

    /// Gets the message type.
    pub fn message_type(&self) -> MessageType;

    /// Gets the message size in bytes.
    pub fn size(&self) -> usize;
}
```

## 🛠️ CLI Tools API

### Command Interface

#### `CLI`
Main command-line interface.

```rust
#[derive(Parser)]
#[command(name = "aevum-bond")]
#[command(about = "Aevum & Bond blockchain CLI")]
pub struct CLI {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Node(NodeCommand),
    Wallet(WalletCommand),
    Mining(MiningCommand),
    Network(NetworkCommand),
    Dev(DevCommand),
}

impl CLI {
    /// Executes the CLI command.
    pub async fn execute(self) -> Result<(), CLIError>;

    /// Parses command line arguments.
    pub fn parse_args() -> Self;

    /// Gets the configuration for the command.
    pub fn get_config(&self) -> Result<Config, CLIError>;
}
```

#### Node Commands

```rust
#[derive(Subcommand)]
pub enum NodeCommand {
    Start {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    Stop,
    Status,
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

impl NodeCommand {
    /// Executes the node command.
    pub async fn execute(self) -> Result<(), CLIError>;
}
```

#### Wallet Commands

```rust
#[derive(Subcommand)]
pub enum WalletCommand {
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        chain: ChainType,
    },
    List,
    Balance {
        #[arg(long)]
        address: String,
    },
    Send {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: u64,
    },
    Import {
        #[arg(long)]
        private_key: String,
    },
    Export {
        #[arg(long)]
        address: String,
    },
}

impl WalletCommand {
    /// Executes the wallet command.
    pub async fn execute(self) -> Result<(), CLIError>;
}
```

## 📊 Error Types

### Core Errors

```rust
#[derive(thiserror::Error, Debug)]
pub enum BondError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Mining error: {0}")]
    MiningError(String),
    #[error("UTXO not found: {0:?}")]
    UTXONotFound(OutPoint),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Cryptographic error: {0}")]
    CryptoError(#[from] CryptoError),
}

#[derive(thiserror::Error, Debug)]
pub enum AevumError {
    #[error("Invalid account: {0:?}")]
    InvalidAccount([u8; 32]),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Insufficient gas")]
    InsufficientGas,
    #[error("Contract execution failed: {0}")]
    ContractExecutionFailed(String),
    #[error("State error: {0}")]
    StateError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("Key generation failed")]
    KeyGenerationFailed,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid private key")]
    InvalidPrivateKey,
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    #[error("Message serialization failed")]
    SerializationFailed,
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Network timeout")]
    Timeout,
}
```

## 🧪 Testing Utilities

### Test Helpers

```rust
/// Creates a test block with the given transactions.
pub fn create_test_block(transactions: Vec<Transaction>) -> Block;

/// Creates a test transaction with random inputs and outputs.
pub fn create_test_transaction() -> Transaction;

/// Creates a test UTXO set with predefined UTXOs.
pub fn create_test_utxo_set() -> UTXOSet;

/// Creates a test ML-DSA key pair.
pub fn create_test_keypair() -> MLDSAKeyPair;

/// Creates a test network node for integration testing.
pub async fn create_test_network_node() -> NetworkNode;

/// Property test strategies for generating arbitrary data.
pub mod strategies {
    use proptest::prelude::*;
    
    pub fn arbitrary_transaction() -> impl Strategy<Value = Transaction>;
    pub fn arbitrary_block() -> impl Strategy<Value = Block>;
    pub fn arbitrary_account() -> impl Strategy<Value = Account>;
}
```

## 📋 Usage Examples

### Basic Bond Chain Usage

```rust
use bond_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new blockchain
    let mut blockchain = BondChain::new();
    
    // Create a transaction
    let mut tx = Transaction::new(vec![], vec![]);
    
    // Create a block with the transaction
    let block = Block::new(vec![tx], [0u8; 32], 1234567890);
    
    // Add the block to the blockchain
    blockchain.add_block(block).await?;
    
    println!("Blockchain height: {}", blockchain.height());
    
    Ok(())
}
```

### Basic Aevum Chain Usage

```rust
use aevum_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new world state
    let mut state = WorldState::new();
    
    // Create accounts
    let alice = [1u8; 32];
    let bob = [2u8; 32];
    
    // Create accounts with initial balances
    state.get_or_create_account(alice).balance = 1000;
    state.get_or_create_account(bob).balance = 500;
    
    // Transfer funds
    state.transfer(&alice, &bob, 100)?;
    
    println!("Alice balance: {}", state.get_account(&alice).unwrap().balance);
    println!("Bob balance: {}", state.get_account(&bob).unwrap().balance);
    
    Ok(())
}
```

### Cryptographic Operations

```rust
use shared_crypto::*;

fn main() -> Result<(), CryptoError> {
    // Generate a key pair
    let keypair = MLDSAKeyPair::generate()?;
    
    // Sign a message
    let message = b"Hello, Aevum & Bond!";
    let signature = keypair.sign(message)?;
    
    // Verify the signature
    let is_valid = keypair.verify(message, &signature)?;
    println!("Signature valid: {}", is_valid);
    
    // Generate an address
    let address = AddressGenerator::bond_address(keypair.public_key());
    println!("Address: {}", hex::encode(address));
    
    Ok(())
}
```

---

**API Reference Version**: 1.0.0  
**Last Updated**: September 27, 2025  
**Status**: Complete ✅
