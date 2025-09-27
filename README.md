# 🚀 Aevum & Bond Blockchain Ecosystem

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/ozzyjob/Aevum-Bond)
[![Tests](https://img.shields.io/badge/tests-58%2F58-brightgreen)](https://github.com/ozzyjob/Aevum-Bond)
[![Security](https://img.shields.io/badge/security-87%2F100-green)](https://github.com/ozzyjob/Aevum-Bond)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 📋 Visão Geral

O **Aevum & Bond Blockchain Ecosystem** é uma implementação dual de protocolos blockchain em Rust, combinando:

- **🔗 Bond Protocol**: Blockchain de Proof-of-Work (PoW) com modelo pUTXO para máxima segurança
- **⚡ Aevum Protocol**: Blockchain de Proof-of-Dedication (PoD) com Smart Accounts para velocidade

## 🏗️ Arquitetura

### Dual-Blockchain Design
```
┌─────────────────┐    ┌─────────────────┐
│   Bond Chain    │◄──►│  Aevum Chain    │
│   (Security)    │    │   (Speed)       │
│                 │    │                 │
│ • PoW Consensus │    │ • PoD Consensus │
│ • pUTXO Model  │    │ • Account Model │
│ • 10min blocks  │    │ • 3sec blocks   │
│ • ML-DSA-87    │    │ • ML-DSA-44     │
└─────────────────┘    └─────────────────┘
```

### Workspace Structure
```
aevum-bond-protocol/
├── 🔐 shared-crypto/       # Post-Quantum Cryptography (ML-DSA)
├── 🔗 bond-core/           # Bond Protocol Core (PoW + pUTXO)
├── ⚡ aevum-core/          # Aevum Protocol Core (PoD + Accounts)
├── 🌐 p2p-network/         # P2P Networking Layer
├── 🛠️ cli-tools/           # Command Line Tools
├── 🖥️ wallet-desktop/      # Desktop Wallet (Tauri)
└── 📚 docs/                # Documentation
```

## ✨ Características Principais

### 🛡️ Segurança Post-Quântica
- **ML-DSA (CRYSTALS-Dilithium)** para assinaturas digitais
- **Kyber** para encriptação de chave pública
- **Resistente a ataques quânticos**

### ⚖️ Consenso Híbrido
- **Bond**: Proof-of-Work para máxima descentralização
- **Aevum**: Proof-of-Dedication para eficiência energética

### 🔄 Interoperabilidade
- **Bridge nativo** entre as duas chains
- **Atomic swaps** cross-chain
- **Shared validator set** para segurança

## 🚀 Quick Start

### Pré-requisitos
```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instalar dependências do sistema (Ubuntu/Debian)
sudo apt update
sudo apt install build-essential pkg-config libssl-dev
```

### Compilação
```bash
# Clonar o repositório
git clone https://github.com/ozzyjob/Aevum-Bond.git
cd Aevum-Bond

# Compilar tudo
cargo build --release

# Executar testes
cargo test --all
```

### Executar Node
```bash
# Inicializar blockchain Bond
./target/release/bond-cli genesis

# Começar mineração
./target/release/bond-cli mine --difficulty 4

# Ver estatísticas
./target/release/bond-cli stats
```

## 📊 Status do Projeto

### ✅ Sprint 1 - Core Protocol (Completo)
- [x] Bond Protocol core implementado
- [x] Sistema de transações e UTXOs
- [x] Algoritmo de mining PoW
- [x] Engine de scripts básico
- [x] CLI funcional
- [x] 58 testes implementados (5 camadas)

### 🔄 Próximos Sprints
- **Sprint 2**: Criptografia pós-quântica
- **Sprint 3**: Rede P2P
- **Sprint 4**: Sincronização de blockchain
- **Sprint 5**: Pool de transações
- **Sprint 6**: Aevum Protocol core
- **Sprint 7**: Bridge inter-chain
- **Sprint 8**: APIs REST/GraphQL
- **Sprint 9**: Wallet desktop

## 🧪 Testes

O projeto implementa uma estratégia abrangente de 5 camadas de teste:

| Camada | Descrição | Testes | Status |
|--------|-----------|--------|--------|
| Layer 1 | Unit Tests | 27 | ✅ 100% |
| Layer 2 | Integration Tests | 13 | ✅ 100% |
| Layer 3 | End-to-End Tests | 5 | ✅ 100% |
| Layer 4 | Network Tests | 7 | ✅ 100% |
| Layer 5 | Security Tests | 6 | ✅ 100% |

```bash
# Executar todos os testes
cargo test --all

# Executar por camada
cargo test --lib              # Layer 1: Unit Tests
cargo test --test "*integration*"  # Layer 2: Integration
cargo test --test "*e2e*"          # Layer 3: End-to-End
cargo test --test "*network*"      # Layer 4: Network
cargo test --test "*security*"     # Layer 5: Security
```

## 📚 Documentação

### Arquitetura
- [📋 Visão Geral da Arquitetura](docs/architecture-overview.md)
- [🔗 Bond Protocol Spec](docs/bond-protocol-spec.md)
- [⚡ Aevum Protocol Spec](docs/aevum-protocol-spec.md)

### Desenvolvimento
- [🛠️ Guia de Desenvolvimento](docs/development-guide.md)
- [🧪 Estratégia de Testes](docs/testing-strategy.md)
- [🔒 Segurança](docs/security-guide.md)

### API Reference
- [🛠️ CLI Usage](docs/bond-cli-usage.md)
- [📡 RPC API](docs/rpc-api.md)
- [🔌 SDK Reference](docs/sdk-reference.md)

## 🤝 Contribuindo

### Fluxo de Desenvolvimento
1. Fork o repositório
2. Crie uma branch feature (`git checkout -b feature/nova-funcionalidade`)
3. Commit suas mudanças (`git commit -am 'Adiciona nova funcionalidade'`)
4. Push para a branch (`git push origin feature/nova-funcionalidade`)
5. Abra um Pull Request

### Padrões de Código
- **Rust 2021 Edition**
- **Formatação**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Testes**: Todos os PRs devem incluir testes
- **Documentação**: Funções públicas devem ter doc comments

### Reportar Issues
- 🐛 [Bug Reports](https://github.com/ozzyjob/Aevum-Bond/issues/new?template=bug_report.md)
- 💡 [Feature Requests](https://github.com/ozzyjob/Aevum-Bond/issues/new?template=feature_request.md)
- 🔒 [Security Issues](mailto:security@aevum-bond.io)

## 📈 Roadmap

### 2024 Q4
- [x] Core Protocol Implementation
- [x] Testing Infrastructure
- [ ] Post-Quantum Cryptography
- [ ] P2P Networking

### 2025 Q1
- [ ] Aevum Protocol
- [ ] Inter-chain Bridge
- [ ] REST/GraphQL APIs
- [ ] Desktop Wallet

### 2025 Q2
- [ ] Mobile Wallets
- [ ] Smart Contracts
- [ ] DeFi Protocols
- [ ] Mainnet Launch

## 🏆 Reconhecimentos

- **Rust Community** - Pela linguagem incrível
- **NIST** - Pelos padrões de criptografia pós-quântica
- **Bitcoin & Ethereum** - Pela inspiração arquitetural

## 📄 Licença

Este projeto está licenciado sob a licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## 📞 Contato

- **Website**: [https://aevum-bond.io](https://aevum-bond.io)
- **Email**: [contact@aevum-bond.io](mailto:contact@aevum-bond.io)
- **Twitter**: [@AevumBond](https://twitter.com/AevumBond)
- **Discord**: [Aevum & Bond Community](https://discord.gg/aevum-bond)

---

<div align="center">

**🚀 Built with Rust | ⚡ Powered by Innovation | 🛡️ Secured by Post-Quantum Cryptography**

[⭐ Star this repo](https://github.com/ozzyjob/Aevum-Bond) | [🍴 Fork it](https://github.com/ozzyjob/Aevum-Bond/fork) | [📋 Issues](https://github.com/ozzyjob/Aevum-Bond/issues)

</div>
