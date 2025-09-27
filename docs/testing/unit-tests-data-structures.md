# Camada 1: Unit Tests - Estruturas de Dados

## 1.1 Serialização e Desserialização Round-Trip

### Bond Protocol (BND)
```rust
#[cfg(test)]
mod serialization_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn block_roundtrip_serialization() {
        let block = create_test_block();
        let serialized = bincode::serialize(&block).unwrap();
        let deserialized: Block = bincode::deserialize(&serialized).unwrap();
        assert_eq!(block, deserialized);
    }

    #[test] 
    fn transaction_canonical_representation() {
        let tx = create_test_transaction();
        let hash1 = tx.hash().unwrap();
        
        // Serialize and deserialize
        let serialized = bincode::serialize(&tx).unwrap();
        let deserialized: Transaction = bincode::deserialize(&serialized).unwrap();
        let hash2 = deserialized.hash().unwrap();
        
        assert_eq!(hash1, hash2); // Canonical representation preserved
    }

    proptest! {
        #[test]
        fn programmable_utxo_roundtrip(
            value in 1u64..1_000_000_000_000,
            script_size in 0usize..1000
        ) {
            let script = Script::new(vec![0u8; script_size]);
            let utxo = ProgrammableUtxo::simple(
                UtxoId::new(TransactionHash::ZERO, 0),
                value,
                script,
                100
            );
            
            let serialized = bincode::serialize(&utxo).unwrap();
            let deserialized: ProgrammableUtxo = bincode::deserialize(&serialized).unwrap();
            
            prop_assert_eq!(utxo, deserialized);
        }
    }
}
```

### Aevum Protocol (AEV)
```rust
#[cfg(test)]
mod aevum_serialization_tests {
    #[test]
    fn account_state_roundtrip() {
        let account = SmartAccount::new_with_guardians(
            Address::generate(),
            1_000_000,
            vec![guardian1, guardian2]
        );
        
        let serialized = bincode::serialize(&account).unwrap();
        let deserialized: SmartAccount = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(account, deserialized);
        assert_eq!(account.guardians().len(), deserialized.guardians().len());
    }

    #[test]
    fn dedication_score_serialization() {
        let score = DedicationScore {
            stake_weight: 1000,
            time_locked: Duration::from_secs(86400 * 30),
            uptime_ratio: 0.98,
            governance_participation: 15,
        };
        
        let serialized = bincode::serialize(&score).unwrap();
        let deserialized: DedicationScore = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(score, deserialized);
    }
}
```
