# 🤝 Contributing to Aevum & Bond

## 🎯 Welcome Contributors!

Thank you for your interest in contributing to the Aevum & Bond project! This guide will help you get started with contributing to our post-quantum dual-ledger blockchain ecosystem.

## 📋 Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Contribution Types](#contribution-types)
5. [Code Standards](#code-standards)
6. [Testing Requirements](#testing-requirements)
7. [Submission Process](#submission-process)
8. [Review Process](#review-process)

## 📜 Code of Conduct

### Our Pledge
We are committed to making participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, sex characteristics, gender identity and expression, level of experience, education, socio-economic status, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards
**Positive behavior includes:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior includes:**
- The use of sexualized language or imagery
- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate

## 🚀 Getting Started

### Prerequisites
- **Rust**: 1.70+ with stable toolchain
- **Git**: Version control familiarity
- **System**: Linux/macOS/Windows with WSL2
- **Knowledge**: Basic understanding of blockchain concepts

### Development Setup
```bash
# 1. Fork the repository on GitHub
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/Aevum-Bond.git
cd Aevum-Bond

# 3. Add upstream remote
git remote add upstream https://github.com/ozzyjob/Aevum-Bond.git

# 4. Install dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add clippy rustfmt

# 5. Build project
cargo build --all

# 6. Run tests
cargo test --all
```

### IDE Configuration
**VS Code Recommended Extensions:**
- rust-analyzer
- CodeLLDB
- GitHub Copilot
- GitLens
- Better TOML

**Settings for VS Code:**
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## 🔄 Development Workflow

### Branching Strategy
We use a **Git Flow** approach:

- `main`: Production-ready code
- `develop`: Integration branch for features
- `feature/*`: Individual feature development
- `hotfix/*`: Critical bug fixes
- `release/*`: Release preparation

### Feature Development Workflow
```bash
# 1. Sync with upstream
git fetch upstream
git checkout develop
git merge upstream/develop

# 2. Create feature branch
git checkout -b feature/your-feature-name

# 3. Make changes and commit
git add .
git commit -m "feat: add new consensus mechanism"

# 4. Keep branch updated
git fetch upstream
git rebase upstream/develop

# 5. Push to your fork
git push origin feature/your-feature-name

# 6. Create Pull Request
```

## 🎨 Contribution Types

### 1. Core Development
**Bond Chain (PoW + pUTXO)**
- Mining algorithm improvements
- UTXO set optimization
- Consensus mechanism enhancements
- Security improvements

**Aevum Chain (PoD + Accounts)**
- Validator selection algorithms
- Smart contract functionality
- State management optimization
- Performance improvements

### 2. Infrastructure Development
**Networking**
- P2P protocol improvements
- Peer discovery enhancements
- Network resilience features
- Protocol optimizations

**Tools & Utilities**
- CLI command additions
- Development tools
- Testing utilities
- Documentation generators

### 3. Security Contributions
- Cryptographic implementations
- Security audits
- Vulnerability assessments
- Penetration testing
- Security documentation

### 4. Documentation
- Technical documentation
- API documentation
- Tutorials and guides
- Architecture diagrams
- Code comments

### 5. Testing
- Unit test improvements
- Integration test additions
- Performance benchmarks
- Security test cases
- Fuzzing implementations

## 📝 Code Standards

### Rust Code Style
We follow the official Rust style guide with these additions:

#### Formatting
```bash
# Format code before committing
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

#### Linting
```bash
# Run clippy with strict settings
cargo clippy --all -- -D warnings

# Allow specific lints if justified
#[allow(clippy::too_many_arguments)]
```

#### Documentation
```rust
/// Brief description of the function.
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description of parameter
/// * `param2` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of possible errors
///
/// # Example
///
/// ```
/// use crate::example_function;
/// let result = example_function(arg1, arg2);
/// ```
pub fn example_function(param1: Type1, param2: Type2) -> Result<ReturnType, Error> {
    // Implementation
}
```

#### Error Handling
```rust
// Use thiserror for error definitions
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Network error")]
    NetworkError(#[from] std::io::Error),
}

// Use anyhow for error propagation in applications
use anyhow::{Context, Result};

fn process_data() -> Result<()> {
    let data = read_file("config.toml")
        .context("Failed to read configuration file")?;
    Ok(())
}
```

### Commit Message Convention
We use **Conventional Commits**:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples:**
```
feat(bond-core): implement new mining algorithm
fix(aevum-core): resolve validator selection bug
docs(readme): update installation instructions
test(network): add peer discovery integration tests
```

## 🧪 Testing Requirements

### Test Coverage Standards
- **Minimum Coverage**: 80% for new code
- **Target Coverage**: 90% for critical components
- **Security Code**: 95% coverage required

### Test Categories

#### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_validation() {
        let block = create_test_block();
        assert!(block.is_valid());
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

#### 2. Integration Tests
```rust
// tests/integration_test.rs
use aevum_bond::*;

#[tokio::test]
async fn test_full_transaction_flow() {
    let node = TestNode::new().await;
    let tx = create_test_transaction();
    
    let result = node.submit_transaction(tx).await;
    assert!(result.is_ok());
    
    // Verify transaction was processed
    assert!(node.transaction_exists(&tx.hash()).await);
}
```

#### 3. Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_transaction_serialization(
        tx in transaction_strategy()
    ) {
        let serialized = bincode::serialize(&tx)?;
        let deserialized: Transaction = bincode::deserialize(&serialized)?;
        prop_assert_eq!(tx, deserialized);
    }
}
```

#### 4. Benchmark Tests
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_mining(c: &mut Criterion) {
    c.bench_function("mine_block", |b| {
        b.iter(|| {
            let block = black_box(create_test_block());
            mine_block(block, DIFFICULTY)
        })
    });
}

criterion_group!(benches, benchmark_mining);
criterion_main!(benches);
```

### Running Tests
```bash
# Run all tests
cargo test --all

# Run tests with coverage
cargo tarpaulin --out Html

# Run specific test suite
cargo test --package bond-core

# Run benchmarks
cargo bench

# Run integration tests
cargo test --test integration_tests
```

## 📋 Submission Process

### Pull Request Checklist

**Before Submitting:**
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] Linting passes (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Commit messages follow convention
- [ ] Branch is up-to-date with develop

### Pull Request Template
```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All tests pass locally
- [ ] Manual testing completed

## Documentation
- [ ] Code comments added/updated
- [ ] Documentation updated
- [ ] API documentation updated (if applicable)

## Security
- [ ] Security implications considered
- [ ] Cryptographic changes reviewed
- [ ] No sensitive information exposed

## Performance
- [ ] Performance impact assessed
- [ ] Benchmarks run (if applicable)
- [ ] Memory usage considered

## Additional Notes
Any additional information or context.
```

## 👀 Review Process

### Review Criteria
**Technical Review:**
- Code quality and style
- Architecture adherence
- Security considerations
- Performance implications
- Test coverage adequacy

**Documentation Review:**
- Code documentation completeness
- User documentation accuracy
- API documentation clarity
- Example code validity

### Review Timeline
- Initial review: Within 2-3 business days
- Follow-up reviews: Within 1-2 business days
- Final approval: After all requirements met

### Reviewer Assignment
Reviews are assigned based on:
- Code area expertise
- Availability
- Previous contributions
- Security sensitivity

### Addressing Review Comments
```bash
# Make requested changes
git add .
git commit -m "fix: address review comments"

# Push updates
git push origin feature/your-feature-name

# Review will automatically update
```

## 🏆 Recognition

### Contributor Levels
**Community Member**: Anyone who contributes
**Regular Contributor**: 5+ merged PRs
**Core Contributor**: 20+ merged PRs + ongoing involvement
**Maintainer**: Trusted with repository access

### Contribution Recognition
- Contributors listed in AUTHORS.md
- Significant contributions mentioned in release notes
- Annual contributor appreciation
- Conference speaking opportunities

## 📞 Getting Help

### Communication Channels
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time community chat
- **Email**: security@aevum.bond (security issues only)

### Mentorship Program
New contributors can request mentorship for:
- First-time contributions
- Complex feature development
- Architecture understanding
- Career development in blockchain

## 🎯 Roadmap Participation

### Current Focus Areas
1. **Post-Quantum Cryptography**: ML-DSA implementation
2. **Consensus Optimization**: PoW and PoD improvements
3. **Network Resilience**: P2P protocol enhancements
4. **Developer Tools**: CLI and SDK improvements

### How to Get Involved
1. Check GitHub issues labeled "good first issue"
2. Join roadmap planning discussions
3. Propose new features via GitHub Discussions
4. Participate in architecture reviews

---

**Thank you for contributing to Aevum & Bond!**

Together, we're building the future of post-quantum blockchain technology.

---

**Contributing Guide Version**: 1.0.0  
**Last Updated**: September 27, 2025  
**Status**: Active ✅
