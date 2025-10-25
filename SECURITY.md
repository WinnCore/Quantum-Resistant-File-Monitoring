# Security Policy

## Supported Versions

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| 0.1.x   | :white_check_mark: | Active development (scaffold) |
| < 0.1   | :x:                | Not released |

**Note**: This is a pre-release version. Security guarantees are being actively developed.

---

## Reporting a Vulnerability

We take security seriously. If you discover a vulnerability, please follow **responsible disclosure**:

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead:

1. **Email**: zw@winncore.com
2. **Subject**: `[SECURITY] Brief description`
3. **Include**:
   - Detailed description of the vulnerability
   - Steps to reproduce
   - Affected versions
   - Potential impact (confidentiality, integrity, availability)
   - Suggested remediation (if any)
   - Your contact information

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 5 business days
- **Fix Timeline**: Depends on severity (see below)
- **Disclosure**: Coordinated after fix is available

### Severity Levels

| Severity | Impact | Fix Timeline |
|----------|--------|--------------|
| **Critical** | Remote code execution, privilege escalation, quarantine bypass | 7 days |
| **High** | Information disclosure, DoS, sandbox escape | 14 days |
| **Medium** | Local privilege escalation, limited data exposure | 30 days |
| **Low** | Minor information leaks, non-security bugs | Next release |

### Coordinated Disclosure

We prefer **90-day coordinated disclosure**:
1. We acknowledge your report
2. We develop and test a fix
3. We release a security advisory and patched version
4. After 90 days (or when patch is widely deployed), you may publicly disclose

We will credit you in:
- CHANGELOG.md
- Security advisory
- Release notes (unless you prefer anonymity)

---

## Security Architecture

### Threat Model

**Assumptions**:
- Attacker has local user access on the system
- Attacker can create/modify files in monitored directories
- Attacker can observe daemon behavior (audit logs, process list)

**Out of Scope**:
- Physical access attacks
- Kernel vulnerabilities
- Supply chain attacks (beyond signature verification)
- Social engineering

### Attack Surface

#### 1. File Scanning (av-core)

**Risk**: Malformed files trigger parser bugs (buffer overflows, DoS)

**Mitigations**:
- Buffered I/O with size limits (256KB initial read)
- YARA engine sandboxed with seccomp
- Heuristic timeouts to prevent algorithmic complexity attacks
- Crash recovery in daemon (isolated per-file scanning)

#### 2. Quarantine (av-quarantine)

**Risk**: Attacker bypasses encryption, restores malicious file

**Mitigations**:
- AES-256-GCM authenticated encryption
- Per-host encryption key (derived from machine ID + secret)
- SHA-256 integrity verification on restore
- Copy-on-write semantics (original file untouched)
- Double-write verification

**Known Limitations**:
- Encryption key stored in memory (vulnerable to memory dumps if root compromised)
- No key rotation mechanism (v0.1.0)

#### 3. Signature Updates (av-signatures)

**Risk**: Attacker serves malicious signature bundle

**Mitigations**:
- Ed25519 signature verification (public key pinned)
- TLS certificate pinning for update endpoint
- Semantic versioning rollback protection
- Bundle checksum verification (SHA-256)

**Known Limitations**:
- No revocation mechanism for compromised signing keys (v0.1.0)
- Update channel is single-source (no mirrors)

#### 4. Real-Time Monitoring (av-daemon)

**Risk**: Privilege escalation via daemon, sandbox escape

**Mitigations**:
- Runs as unprivileged `avdaemon` user (no capabilities by default)
- AppArmor profile: default-deny filesystem access
- seccomp-bpf: syscall whitelist for ARM64
- systemd hardening: `ProtectSystem=strict`, `NoNewPrivileges=true`
- Optional Landlock confinement (experimental)

**Known Limitations**:
- fanotify requires CAP_SYS_ADMIN (if enabled, daemon runs with minimal capabilities)
- No eBPF integration yet (v0.1.0)

#### 5. CLI (av-cli)

**Risk**: Command injection, path traversal

**Mitigations**:
- Path validation (reject `../`, absolute paths outside monitored dirs)
- Clap argument parsing (no shell expansion)
- JSON output escaping
- Quarantine ID validation (UUID-only)

---

## Security Features

### Sandboxing Layers

| Layer | Technology | Status | Notes |
|-------|-----------|--------|-------|
| **Filesystem** | AppArmor | Enabled | Default-deny profile |
| **Syscalls** | seccomp-bpf | Enabled | ARM64 whitelist |
| **Process** | Landlock | Optional | Feature-gated (v0.2.0+) |
| **Network** | systemd | Enabled | RestrictAddressFamilies=AF_UNIX AF_INET |

### Cryptographic Primitives

| Use Case | Algorithm | Library | Key Management |
|----------|-----------|---------|----------------|
| Quarantine Encryption | AES-256-GCM | ring | Per-host (machine ID + secret) |
| Signature Verification | Ed25519 | ed25519-dalek | Public key pinned in binary |
| Integrity Checks | SHA-256 | sha2 | N/A |
| File Hashing | SHA-512 (planned) | sha2 | N/A |

### Audit Logging

All security-relevant events logged to:
- **systemd journal** (`journalctl -u av-daemon`)
- **Syslog** (if configured)
- **Metrics**: Prometheus/OpenTelemetry (v0.3.0+)

Logged events:
- Quarantine operations (add, restore, purge)
- Signature update attempts
- Sandbox policy violations
- Real-time monitoring mode changes
- High-confidence detections

---

## Security Roadmap

### v0.2.0 (Q1 2025)
- [ ] Key rotation for quarantine encryption
- [ ] Signature bundle revocation lists
- [ ] Memory sanitization for crypto keys
- [ ] Landlock integration (LSM)

### v0.3.0 (Q2 2025)
- [ ] eBPF probes for behavioral analysis
- [ ] Kernel audit integration
- [ ] SBOM generation (CycloneDX)
- [ ] Signed release artifacts

### v0.4.0 (Q3 2025)
- [ ] FIDO2/YubiKey integration for quarantine restore
- [ ] Multi-key signature verification
- [ ] Remote attestation for daemon integrity

---

## Known Vulnerabilities

### Current (v0.1.0)

None reported yet (pre-release).

### Historical

| CVE | Severity | Component | Fixed in | Description |
|-----|----------|-----------|----------|-------------|
| N/A | N/A | N/A | N/A | No vulnerabilities reported yet |

---

## Security Hygiene

### For Users

1. **Keep Updated**
   ```bash
   av-cli signatures update
   sudo apt update && sudo apt upgrade charmedwoa-av
   ```

2. **Verify Signatures**
   ```bash
   # Check .deb package signature (future feature)
   dpkg-sig --verify charmedwoa-av_0.1.0_aarch64.deb
   ```

3. **Monitor Logs**
   ```bash
   journalctl -u av-daemon.service -f
   ```

4. **Review Quarantine**
   ```bash
   av-cli quarantine list --json | jq
   ```

### For Developers

1. **Dependency Audits**
   ```bash
   cargo audit
   cargo deny check advisories
   ```

2. **Fuzzing** (future)
   ```bash
   cargo fuzz run scanner_input
   ```

3. **Static Analysis**
   ```bash
   cargo clippy -- -D warnings
   ```

4. **Security Testing**
   ```bash
   cargo test --tests security::
   ```

---

## Contact

- **Security Email**: zw@winncore.com
- **PGP Key**: [Future: Link to public key]
- **Security Advisories**: https://github.com/WinnCore/Quantum-Resistant-File-Monitoring/security/advisories

---

## Acknowledgments

We thank the following security researchers for responsible disclosure:

| Researcher | Vulnerability | Date | Bounty |
|------------|---------------|------|--------|
| (None yet) | - | - | - |

*Bug bounty program planned for v1.0.0 release.*

---

**Last Updated**: 2025-01-24
**Policy Version**: 1.0
