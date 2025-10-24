# Contributing to CharmedWOA ARM64 Antivirus Suite

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Security Considerations](#security-considerations)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of experience level, gender, gender identity, sexual orientation, disability, personal appearance, body size, race, ethnicity, age, religion, or nationality.

### Expected Behavior

- Be respectful and constructive
- Use welcoming and inclusive language
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other contributors

### Unacceptable Behavior

- Harassment, intimidation, or discrimination
- Trolling, insulting comments, or personal attacks
- Publishing others' private information
- Other conduct which could reasonably be considered inappropriate

---

## Getting Started

### Prerequisites

1. **Rust 1.74+** with ARM64 target support
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   ```

2. **Development Dependencies**
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential libssl-dev pkg-config

   # Tools
   cargo install cargo-audit cargo-deny cargo-outdated
   ```

3. **Recommended IDE Setup**
   - **VS Code** with rust-analyzer extension
   - **CLion** with Rust plugin
   - **Vim/Neovim** with coc-rust-analyzer

### Fork and Clone

```bash
# Fork the repository on GitHub first
git clone https://github.com/YOUR_USERNAME/Quantum-Resistant-File-Monitoring.git
cd Quantum-Resistant-File-Monitoring

# Add upstream remote
git remote add upstream https://github.com/WinnCore/Quantum-Resistant-File-Monitoring.git
```

---

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions/improvements
- `security/` - Security fixes (coordinate with maintainers first)

### 2. Make Your Changes

Follow the [Coding Standards](#coding-standards) below.

### 3. Test Your Changes

```bash
# Format code
make fmt

# Run linter
make lint

# Run tests
make test

# Check for security vulnerabilities
cargo audit

# Check dependencies
cargo deny check
```

### 4. Commit Your Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat(av-core): add entropy analysis for packed binaries"
git commit -m "fix(av-daemon): prevent race condition in fanotify loop"
git commit -m "docs(readme): clarify quarantine restore workflow"
```

Commit types:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `style:` - Formatting changes
- `refactor:` - Code restructuring
- `perf:` - Performance improvements
- `test:` - Test additions
- `chore:` - Build/tooling changes
- `security:` - Security fixes

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

---

## Pull Request Process

### PR Checklist

Before submitting, ensure:

- [ ] Code compiles without warnings (`cargo build --all-features`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Clippy produces no warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all -- --check`)
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (for notable changes)
- [ ] Security implications considered
- [ ] No new dependencies without justification

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe testing performed

## Security Considerations
Any security implications?

## Checklist
- [ ] Tests pass
- [ ] Code formatted
- [ ] Documentation updated
```

### Review Process

1. **Automated Checks**: CI/CD must pass
2. **Peer Review**: At least one maintainer approval required
3. **Security Review**: Required for changes to:
   - av-core scanning engine
   - av-quarantine encryption
   - av-signatures verification
   - Sandboxing policies (AppArmor/seccomp)
4. **Merge**: Maintainers will merge after approval

---

## Coding Standards

### Rust Style Guidelines

1. **Follow `rustfmt` defaults**
   ```bash
   cargo fmt --all
   ```

2. **Use `clippy` recommendations**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Naming Conventions**
   - `snake_case` for functions, variables, modules
   - `PascalCase` for types, structs, enums
   - `SCREAMING_SNAKE_CASE` for constants

4. **Documentation**
   - Public APIs must have doc comments (`///`)
   - Include examples in doc comments
   - Explain safety invariants
   - Document panics/errors

Example:
```rust
/// Scans a file for malicious patterns.
///
/// # Arguments
///
/// * `path` - Path to the file to scan
///
/// # Returns
///
/// Returns `ScanOutcome` with recommended action.
///
/// # Errors
///
/// Returns error if file cannot be read or YARA rules fail to compile.
///
/// # Example
///
/// ```
/// use av_core::{Scanner, ScannerConfig};
///
/// let scanner = Scanner::new(ScannerConfig::default())?;
/// let outcome = scanner.scan_path("/tmp/suspicious")?;
/// ```
pub async fn scan_path<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<ScanOutcome> {
    // Implementation
}
```

### Safety-Critical Code

For code in `av-quarantine`, `av-signatures`, or crypto operations:

1. **Explicit error handling** - No `.unwrap()` or `.expect()`
2. **Input validation** - Validate all user input
3. **Memory safety** - Avoid `unsafe` unless absolutely necessary
4. **Audit trail** - Log security-relevant operations
5. **Constant-time operations** - For crypto comparisons

Example:
```rust
pub fn verify_signature(&self, data: &[u8], sig: &Signature) -> anyhow::Result<()> {
    // Constant-time comparison
    if self.key.verify_strict(data, sig).is_ok() {
        tracing::info!(key_id = %self.key_id, "signature verified");
        Ok(())
    } else {
        tracing::warn!(key_id = %self.key_id, "signature verification failed");
        anyhow::bail!("invalid signature")
    }
}
```

---

## Testing Requirements

### Unit Tests

Required for all new code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_scoring() {
        let config = ScannerConfig::default();
        let score = heuristics::score(Path::new("/tmp/test"), &[], &config);
        assert!(score.0 >= 0.0 && score.0 <= 1.0);
    }

    #[tokio::test]
    async fn test_async_scanning() {
        let scanner = Scanner::new(ScannerConfig::default()).unwrap();
        let outcome = scanner.scan_path("/tmp").await.unwrap();
        assert!(matches!(outcome.recommended_action, RecommendedAction::Allow));
    }
}
```

### Integration Tests

For cross-crate functionality:

```rust
// tests/integration.rs
use av_core::Scanner;
use av_quarantine::QuarantineManager;

#[tokio::test]
async fn test_scan_and_quarantine() {
    // Create test file
    // Scan
    // Quarantine
    // Verify integrity
}
```

### Security Tests

Required for:
- Quarantine encryption/decryption
- Signature verification
- Sandboxing policy enforcement

```rust
#[test]
fn test_quarantine_integrity() {
    let manager = QuarantineManager::new(QuarantineConfig::default()).unwrap();
    let record = manager.quarantine(Path::new("/tmp/test")).unwrap();

    // Tamper with encrypted file
    // Restore should fail
    assert!(manager.restore(&record, Path::new("/tmp/restored")).is_err());
}
```

---

## Security Considerations

### Responsible Disclosure

**DO NOT** publicly disclose security vulnerabilities. Instead:

1. Email security@charmedwoa.example (placeholder - update in production)
2. Include:
   - Vulnerability description
   - Steps to reproduce
   - Potential impact
   - Suggested fix (optional)
3. Wait for acknowledgment (within 48 hours)
4. Coordinate disclosure timeline

See [SECURITY.md](SECURITY.md) for full policy.

### Security Review Triggers

Changes requiring security review:
- Crypto operations (encryption, signing, verification)
- Sandboxing (AppArmor, seccomp, Landlock)
- Privilege escalation paths
- Input validation
- File quarantine workflows

### Prohibited Contributions

This project is **defensive security only**. We will **reject** PRs that:
- Add offensive capabilities (exploitation, payload delivery)
- Weaken sandboxing or security controls
- Introduce known vulnerabilities
- Bypass quarantine integrity checks

---

## Questions?

- **General Questions**: Open a [GitHub Discussion](https://github.com/WinnCore/Quantum-Resistant-File-Monitoring/discussions)
- **Bug Reports**: Create an [Issue](https://github.com/WinnCore/Quantum-Resistant-File-Monitoring/issues)
- **Security**: Email security@charmedwoa.example (see [SECURITY.md](SECURITY.md))

---

Thank you for contributing to making ARM64 Linux systems more secure!
