---
name: Agent Task
about: Create a task for GitHub Copilot AI Agents
title: '[AGENT] '
labels: ['agent', 'ai-task']
assignees: ''
---

## 🤖 Agent Assignment
Which AI agent should handle this task?
- [ ] Architecture Copilot (Rust workspace, modular design)
- [ ] Crypto Copilot (Post-quantum cryptography, ML-DSA)
- [ ] Blockchain Copilot (PoW, PoD, consensus, UTXO)
- [ ] Network Copilot (P2P, libp2p, distributed systems)

## 🎯 Task Description
Clear description of what the agent should accomplish:

### Primary Objective
What is the main goal of this task?

### Scope
What components/files will be affected?
- [ ] shared-crypto crate
- [ ] bond-core crate  
- [ ] aevum-core crate
- [ ] node binary
- [ ] Documentation
- [ ] Tests

## 📋 Requirements
Specific requirements for the task:

### Functional Requirements
- [ ] Requirement 1
- [ ] Requirement 2
- [ ] Requirement 3

### Technical Requirements
- [ ] Must compile without warnings
- [ ] Must include comprehensive tests
- [ ] Must follow Rust best practices
- [ ] Must maintain API compatibility

### Performance Requirements
- [ ] Performance benchmark (if applicable)
- [ ] Memory usage constraints
- [ ] Security considerations

## 🔧 Implementation Guidelines
Specific guidelines for the agent:

### Architecture Patterns
- Use modular design principles
- Follow dependency injection patterns
- Implement proper error handling

### Code Style
- Follow project formatting standards
- Include comprehensive documentation
- Use meaningful variable names

### Testing Strategy
- Include unit tests for all public functions
- Add integration tests for complex flows
- Ensure security test coverage

## 📁 Files to Create/Modify
List specific files that need to be created or modified:

### New Files
- [ ] `path/to/new_file.rs`
- [ ] `path/to/another_file.rs`

### Modified Files
- [ ] `existing/file.rs` - [description of changes]
- [ ] `Cargo.toml` - [dependency updates]

## 🧪 Acceptance Criteria
How will we know the task is complete?

### Definition of Done
- [ ] Code compiles successfully
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Performance benchmarks meet requirements
- [ ] Security review completed
- [ ] Code review approved

### Test Cases
Specific test cases that must pass:
```rust
#[test]
fn test_specific_functionality() {
    // Test implementation
}
```

## 🔗 Dependencies
What other tasks or issues must be completed first?
- Depends on #(issue number)
- Blocks #(issue number)
- Related to #(issue number)

## 📚 Reference Materials
Links to relevant documentation or specifications:
- [Architecture Documentation](docs/architecture.md)
- [API Specification](docs/api-spec.md)
- [Security Guidelines](docs/security.md)

## 🕒 Timeline
Expected timeline for completion:
- [ ] Sprint 2 (Post-Quantum Crypto)
- [ ] Sprint 3 (P2P Network) 
- [ ] Sprint 4 (Blockchain Sync)
- [ ] Current sprint
- [ ] Next sprint

## 💬 Agent Instructions
Specific instructions for the AI agent:

### Context
Provide relevant context about the current system state, recent changes, or important considerations.

### Constraints
Any limitations or constraints the agent should be aware of:
- Cannot modify core protocol structures
- Must maintain backward compatibility
- Resource limitations

### Preferences
Preferred approaches or patterns:
- Prefer async/await over threads
- Use Result<T, E> for error handling
- Follow existing naming conventions

## 📋 Checklist
- [ ] Task clearly defined
- [ ] Acceptance criteria specified
- [ ] Agent assignment clear
- [ ] Dependencies identified
- [ ] Timeline realistic
