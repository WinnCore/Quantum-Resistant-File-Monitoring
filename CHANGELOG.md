# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-01-24

### Added
- **YARA Integration**: Real signature scanning with production rules
  - EICAR test file detection
  - UPX-packed ARM64 ELF detection
  - Reverse shell indicators
  - Crypto mining patterns
  - Ransomware-like file encryptor detection
  - Suspicious syscall density heuristics
- **Shannon Entropy Analysis**: Detects packed/encrypted binaries
  - Per-file mean entropy calculation
  - High-entropy region detection (>7.5 bits/byte)
  - Suspicious 4KB chunk analysis
- **Integration Tests**: Real malware detection tests
  - EICAR test file scanning
  - Clean file false-positive verification
  - Entropy detection for encrypted data
  - UPX packed binary detection
- **fanotify Module**: Real-time monitoring framework (av-daemon/src/fanotify.rs)
  - Permission mode with file blocking capability
  - Audit-only mode (unprivileged fallback)
  - CAP_SYS_ADMIN detection

### Changed
- `av-core/src/engine.rs`: Wire YARA runtime (not stubs)
- Scanning now reads up to 2MB (was 256KB)
- Entropy calculation with chunked analysis
- Contact changed to zw@winncore.com

### Security
- YARA rules loaded with 30-second timeout
- Memory-safe entropy calculation
- Read limits prevent OOM on large files

## [0.1.0] - 2025-01-24

### Added
- Initial scaffold for CharmedWOA ARM64 Antivirus Suite
- **av-core**: Core scanning library with YARA stubs, heuristics, and telemetry
- **av-daemon**: Real-time monitoring daemon with fanotify/inotify placeholders
- **av-quarantine**: Quarantine manager with AES-256-GCM encryption and SHA-256 integrity
- **av-signatures**: Signature update subsystem with Ed25519 verification
- **av-cli**: Command-line interface for scanning, quarantine, and realtime toggle
- AppArmor profile with default-deny filesystem access
- seccomp-bpf policy for ARM64 syscall whitelisting
- systemd service unit with hardening (ProtectSystem=strict, NoNewPrivileges)
- Build automation (Makefile, .deb packaging script)
- Comprehensive documentation (README, CONTRIBUTING, SECURITY)
- GitHub Actions CI/CD workflows
- Apache 2.0 license

### Security
- Read-only scanning by default (no file mutations)
- Unprivileged daemon execution (runs as `avdaemon` user)
- Multi-layer sandboxing (AppArmor + seccomp + systemd)
- Quarantine integrity verification (SHA-256 checksums)
- Encrypted quarantine storage (AES-256-GCM)
- Graceful degradation for missing kernel features

### Notes
- This is a **scaffold release** for development and testing
- YARA runtime integration pending (v0.2.0)
- Real-time monitoring starts in audit-only mode
- Designed for ARM64 platforms (Lenovo ThinkPad X13s, AWS Graviton, Raspberry Pi 4+)

[Unreleased]: https://github.com/WinnCore/Quantum-Resistant-File-Monitoring/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/WinnCore/Quantum-Resistant-File-Monitoring/releases/tag/v0.1.0
