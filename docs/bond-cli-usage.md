# Bond CLI - Ferramenta de Linha de Comando

A Bond CLI é uma ferramenta interativa para demonstrar e testar as funcionalidades do protocolo Bond.

## Instalação

```bash
cd /home/aevum/Dev-Muito/Aevum&Bold
cargo build --release --bin bond-cli
```

## Uso

### Comandos Disponíveis

#### `genesis` - Criar Bloco Gênesis
```bash
./target/release/bond-cli genesis
```

Cria e exibe informações do bloco gênesis do Bond, incluindo:
- Hash do bloco
- Recompensa do coinbase (5 BND)
- Mensagem gênesis
- Detalhes técnicos

#### `mine` - Simulação de Mineração
```bash
./target/release/bond-cli mine --difficulty <nivel>
```

Executa uma simulação de mineração PoW:
- `--difficulty`: Nível de dificuldade (1-4, padrão: 1000)
- Exibe hash rate, tempo de mineração e nonce encontrado

#### `validate` - Validação de Blockchain
```bash
./target/release/bond-cli validate
```

Executa validação completa da blockchain:
- Cria bloco gênesis
- Inicializa estado da chain
- Valida regras de consenso
- Exibe estatísticas da blockchain

#### `stats` - Estatísticas do Protocolo
```bash
./target/release/bond-cli stats
```

Exibe informações completas sobre o protocolo Bond:
- Arquitetura técnica
- Parâmetros econômicos
- Funcionalidades de segurança
- Progresso dos sprints

## Exemplos de Saída

### Genesis Block
```
🔨 Creating Bond Genesis Block...

✅ Genesis Block Created Successfully!
📋 Block Details:
   Hash: d438383951f33efd4b5d2d482e788938f48b1e27ae9f6b1f0f93875bcb717ce8
   Transactions: 1
   Coinbase Reward: 5000000000 Elos
   Block Size: 335 bytes
   Timestamp: 2025-09-01 00:00:00 UTC
   Genesis Message: "Aevum & Bond Genesis - Building the Post-Quantum Financial Future"
```

### Mining Simulation
```
⛏️  Starting Bond Mining Simulation...

✅ Block Mined Successfully!
📊 Mining Results:
   Nonce Found: 12
   Block Hash: 036fc5aca7276580d1b41b01f33acbbd2b6f50eb5c2cec1cfbf88468cc86affe
   Time Elapsed: 0.00s
   Hashes Attempted: 13
   Hash Rate: 275540 H/s
✅ Proof-of-Work Valid!
```

## Arquitetura

A CLI demonstra todos os componentes principais implementados no Sprint 1:

- **Block Creation**: Criação e validação de blocos
- **Mining Algorithm**: Algoritmo de mineração PoW com Keccak-256
- **Transaction Processing**: Sistema de transações e UTXOs
- **Consensus Rules**: Validação de regras de consenso
- **Chain State**: Gerenciamento de estado da blockchain

Esta ferramenta serve como demonstração prática das capacidades do protocolo Bond e base para o desenvolvimento das funcionalidades de rede P2P nos próximos sprints.
