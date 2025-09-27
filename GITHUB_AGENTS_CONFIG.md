# 🤖 GitHub Copilot Agent Configuration
# Configuração dos Agentes de IA para o Projeto Aevum & Bond

## 🎯 Agent Tasks - Tarefas dos Agentes

### 📋 Objetivo Geral
Gerar a estrutura inicial (scaffold) para um workspace Cargo em Rust para a blockchain Aevum & Bond, com foco em máxima modularidade e estabelecendo as fundações para o desenvolvimento futuro.

### 🛠️ Tarefas Específicas do Agente

#### 1. 🏗️ Inicializar o Workspace Cargo
```yaml
task_id: "workspace_init"
priority: "critical"
description: "Crie um novo workspace Cargo chamado aevum-bond-protocol"
requirements:
  - Workspace name: "aevum-bond-protocol"
  - Configure Cargo.toml na raiz
  - Inclua todos os crates membros
```

#### 2. 📦 Criar os Crates Membros
```yaml
task_id: "create_crates"
priority: "critical"
description: "Dentro do workspace, gere os seguintes crates"
crates:
  shared-crypto:
    type: "lib"
    purpose: "Abstrair primitivas criptográficas pós-quânticas (ML-DSA)"
    
  bond-core:
    type: "lib" 
    purpose: "Lógica de consenso PoW, modelo pUTXOs, estruturas Bond"
    
  aevum-core:
    type: "lib"
    purpose: "Lógica de consenso PoD, modelo Contas, estruturas Aevum"
    
  node:
    type: "bin"
    purpose: "Executável principal, CLI, rede P2P"
```

#### 3. 📋 Adicionar Dependências Específicas
```yaml
task_id: "add_dependencies"
priority: "high"
description: "Edite o Cargo.toml de cada crate para adicionar dependências"
dependencies:
  shared-crypto:
    - "pqcrypto"
    
  bond-core:
    - "serde = { version = \"1.0\", features = [\"derive\"] }"
    - "bincode = \"1.3\""
    - "sha3 = \"0.10\""
    
  node:
    - "clap = { version = \"4.0\", features = [\"derive\"] }"
    - "rust-libp2p = \"0.50\""
```

#### 4. 🏗️ Gerar Código Placeholder
```yaml
task_id: "placeholder_code"
priority: "medium"
description: "No arquivo lib.rs do crate bond-core"
requirements:
  - Criar structs públicas: Block, Transaction
  - Adicionar campos básicos
  - Incluir #[derive(Serialize, Deserialize)]
```

#### 5. ⚙️ Configurar Dependências do Nó Principal
```yaml
task_id: "node_dependencies"
priority: "high"
description: "No Cargo.toml do crate node"
dependencies:
  - "bond-core = { path = \"../bond-core\" }"
  - "aevum-core = { path = \"../aevum-core\" }"
  - "shared-crypto = { path = \"../shared-crypto\" }"
```

## 🤖 Configuração dos Agentes GitHub

### Agent 1: Architecture Agent
```yaml
name: "Architecture Copilot"
role: "Arquiteto de Sistema"
specialization: "Rust, Blockchain, Modular Architecture"
responsibilities:
  - Estrutura do workspace Cargo
  - Design de APIs entre crates
  - Padrões arquiteturais
  - Modularidade e separação de responsabilidades
keywords: ["workspace", "cargo", "architecture", "modular", "crate"]
```

### Agent 2: Crypto Agent
```yaml
name: "Crypto Copilot"
role: "Especialista em Criptografia"
specialization: "Post-Quantum Cryptography, ML-DSA, Security"
responsibilities:
  - Implementação de primitivas criptográficas
  - Integração pqcrypto
  - Assinaturas digitais
  - Hashing e validação
keywords: ["crypto", "pqcrypto", "ml-dsa", "signature", "hash"]
```

### Agent 3: Blockchain Agent
```yaml
name: "Blockchain Copilot"
role: "Especialista em Blockchain"
specialization: "PoW, PoD, UTXO, Consensus"
responsibilities:
  - Estruturas de dados blockchain
  - Algoritmos de consenso
  - Transações e blocos
  - Validação de estado
keywords: ["blockchain", "pow", "pod", "utxo", "consensus", "block", "transaction"]
```

### Agent 4: Network Agent
```yaml
name: "Network Copilot"
role: "Especialista em Rede P2P"
specialization: "libp2p, Networking, Distributed Systems"
responsibilities:
  - Implementação rede P2P
  - Protocols de comunicação
  - Sincronização de nodes
  - Discovery de peers
keywords: ["p2p", "libp2p", "network", "peer", "sync"]
```

## 📁 Estrutura de Arquivos Esperada

```
aevum-bond-protocol/
├── Cargo.toml                 # Workspace configuration
├── Cargo.lock                 # Dependency lock file
├── README.md                  # Project documentation
├── LICENSE                    # Project license
├── .gitignore                 # Git ignore rules
├── .github/                   # GitHub configurations
│   ├── workflows/             # CI/CD workflows
│   └── ISSUE_TEMPLATE/        # Issue templates
├── docs/                      # Documentation
│   ├── architecture.md
│   ├── api-reference.md
│   └── development-guide.md
├── shared-crypto/             # Post-quantum cryptography crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ml_dsa.rs         # ML-DSA implementation
│   │   ├── kyber.rs          # Kyber key encapsulation
│   │   └── hash.rs           # Hash functions
│   └── tests/
├── bond-core/                 # Bond blockchain core
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── block.rs          # Block structure
│   │   ├── transaction.rs    # Transaction structure
│   │   ├── consensus.rs      # PoW consensus
│   │   ├── utxo.rs          # UTXO model
│   │   └── mining.rs        # Mining algorithms
│   └── tests/
├── aevum-core/               # Aevum blockchain core
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── account.rs        # Account model
│   │   ├── smart_account.rs  # Smart accounts
│   │   ├── consensus.rs      # PoD consensus
│   │   └── state.rs         # State management
│   └── tests/
└── node/                     # Main executable
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs           # Main entry point
    │   ├── cli.rs            # CLI interface
    │   ├── p2p.rs           # P2P networking
    │   └── rpc.rs           # RPC server
    └── tests/
```

## 🎯 Prompts Específicos para Agentes

### Prompt para Architecture Agent
```
Como arquiteto de sistemas Rust especializado em blockchain, sua tarefa é:

1. Criar um workspace Cargo modular para a blockchain Aevum & Bond
2. Organizar os crates de forma que maximize a reutilização de código
3. Definir APIs claras entre os crates
4. Estabelecer padrões de código consistentes
5. Garantir separação adequada de responsabilidades

Foque em modularidade, testabilidade e extensibilidade. Use as melhores práticas do ecossistema Rust.
```

### Prompt para Crypto Agent
```
Como especialista em criptografia pós-quântica, implemente:

1. Wrapper Rust para primitivas ML-DSA (CRYSTALS-Dilithium)
2. Integração com a biblioteca pqcrypto
3. Funções de hash criptográficas (SHA-3, BLAKE3)
4. Geração segura de chaves e assinaturas
5. Validação e verificação criptográfica

Priorize segurança, performance e facilidade de uso. Documente todos os aspectos de segurança.
```

### Prompt para Blockchain Agent
```
Como especialista em blockchain, desenvolva:

1. Estruturas de dados para Block e Transaction
2. Modelo UTXO para Bond chain
3. Modelo de contas para Aevum chain
4. Algoritmos de consenso PoW e PoD
5. Validação de estado e transições

Foque em correção, eficiência e resistência a ataques. Implemente testes abrangentes.
```

### Prompt para Network Agent
```
Como especialista em redes P2P, implemente:

1. Camada de rede usando rust-libp2p
2. Protocols de discovery e comunicação
3. Sincronização entre nodos
4. Gestão de peers e conexões
5. Tratamento de falhas de rede

Priorize robustez, escalabilidade e segurança da rede. Considere scenarios adversários.
```

## 🔧 Comandos de Desenvolvimento

### Setup Inicial
```bash
# Criar workspace
cargo new --lib aevum-bond-protocol
cd aevum-bond-protocol

# Criar crates
cargo new --lib shared-crypto
cargo new --lib bond-core  
cargo new --lib aevum-core
cargo new --bin node

# Build everything
cargo build --all

# Run tests
cargo test --all

# Format code
cargo fmt --all

# Lint code
cargo clippy --all
```

### Verificação de Qualidade
```bash
# Check compilation
cargo check --all-targets

# Run security audit
cargo audit

# Check dependencies
cargo outdated

# Generate documentation
cargo doc --all --no-deps
```

## 📋 Checklist de Entrega

### ✅ Estrutura Base
- [ ] Workspace Cargo.toml configurado
- [ ] Todos os crates criados
- [ ] Dependências adicionadas
- [ ] Código placeholder implementado

### ✅ Qualidade
- [ ] Código compila sem warnings
- [ ] Testes básicos passam
- [ ] Documentação básica presente
- [ ] Linting limpo (clippy)

### ✅ GitHub Integration
- [ ] Repository inicializado
- [ ] README.md completo
- [ ] LICENSE adicionada
- [ ] .gitignore configurado
- [ ] GitHub Actions setup

## 🚀 Next Steps Após Setup

1. **Implementar criptografia pós-quântica** em `shared-crypto`
2. **Desenvolver estruturas blockchain** em `bond-core` e `aevum-core`
3. **Criar CLI funcional** em `node`
4. **Adicionar testes abrangentes** em todos os crates
5. **Documentar APIs** e criar exemplos de uso
6. **Setup CI/CD** com GitHub Actions
7. **Criar releases** e publicar documentação

---

**🤖 Instruções finais para os agentes**: Trabalhem de forma colaborativa, mantenham comunicação clara sobre interfaces entre crates, e priorizem sempre a qualidade e segurança do código. Cada agente deve focar em sua especialização mas estar ciente do sistema como um todo.
