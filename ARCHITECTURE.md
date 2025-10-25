# WinnCore Architecture

## System Design Overview

WinnCore is a modular antivirus suite built with security, performance, and transparency as core principles. This document describes the system architecture, component interactions, and design decisions.

---

## High-Level Architecture

```
┌────────────────────────────────────────────────────────────┐
│                     User Layer                              │
│  ┌──────────┐  ┌──────────┐  ┌─────────────────────────┐  │
│  │  CLI     │  │   GUI    │  │    System Tray          │  │
│  │ av-cli   │  │  Tauri   │  │    (Planned v0.3)       │  │
│  └─────┬────┘  └─────┬────┘  └────────┬────────────────┘  │
└────────┼─────────────┼────────────────┼───────────────────┘
         │             │                │
         │             │                │ IPC (Unix Sockets)
         │             │                │
┌────────┼─────────────┼────────────────┼───────────────────┐
│        │      Service Layer           │                    │
│  ┌─────▼─────────────▼────────────────▼──────────────┐    │
│  │          av-daemon (Background Service)            │    │
│  │  • Real-time monitoring (fanotify/inotify)         │    │
│  │  • Event processing & policy enforcement           │    │
│  │  • Signature updates & health monitoring           │    │
│  └──────┬─────────────────┬────────────────┬──────────┘    │
└─────────┼─────────────────┼────────────────┼───────────────┘
          │                 │                │
          │                 │                │
┌─────────┼─────────────────┼────────────────┼───────────────┐
│         │       Core Layer                 │                │
│  ┌──────▼──────────┐           ┌───────────▼─────────┐     │
│  │    av-core      │◄──────────┤   av-quarantine     │     │
│  │  • YARA engine  │           │  • AES-256-GCM      │     │
│  │  • Heuristics   │           │  • SHA-256 verify   │     │
│  │  • Entropy      │           │  • Restore logic    │     │
│  │  • Scanning     │           └─────────────────────┘     │
│  └──────┬──────────┘                                        │
│         │                                                   │
│  ┌──────▼──────────┐                                        │
│  │ av-signatures   │                                        │
│  │  • Ed25519      │                                        │
│  │  • TLS updates  │                                        │
│  │  • Versioning   │                                        │
│  └─────────────────┘                                        │
└───────────────────────────────────────────────────────────┘
```

---

## Component Details

### 1. av-core (Scanning Engine)

**Purpose**: Core malware detection logic

**Responsibilities**:
- Load and compile YARA rules
- Perform signature-based scanning
- Calculate Shannon entropy
- Heuristic scoring
- Telemetry collection

**Key Files**:
- `src/engine.rs`: YARA wrapper and scan orchestration
- `src/heuristics.rs`: Scoring algorithms
- `src/signatures.rs`: Rule management

**Security Boundaries**:
- Read-only file access (never mutates)
- Bounded memory (2MB max file read)
- Timeout protection (30s YARA scan limit)

**Performance**:
- ~250 MB/s scan throughput on Snapdragon ARM64
- <1% CPU when idle
- 4.5MB memory footprint

---

### 2. av-daemon (Real-Time Monitor)

**Purpose**: Background service for real-time protection

**Responsibilities**:
- Monitor filesystem events (fanotify/inotify)
- Respond to permission requests
- Enforce quarantine policies
- Manage signature updates
- Apply sandboxing policies

**Key Files**:
- `src/main.rs`: Service lifecycle
- `src/fanotify.rs`: Kernel event handling
- `src/security.rs`: AppArmor + seccomp

**Execution Model**:
- Runs as unprivileged `avdaemon` user
- Optional CAP_SYS_ADMIN for fanotify permission mode
- Audit-only fallback (inotify) if unprivileged

**Security**:
- AppArmor profile restricts filesystem access
- seccomp-bpf limits syscalls to whitelist
- systemd hardening (`ProtectSystem=strict`, `NoNewPrivileges`)

---

### 3. av-quarantine (File Isolation)

**Purpose**: Secure storage for suspicious files

**Responsibilities**:
- Encrypt files with AES-256-GCM
- Verify integrity with SHA-256
- Manage quarantine database
- Restore with integrity checks

**Storage Format**:
```
/var/lib/av/quarantine/
├── <timestamp>-<sha256>         # Encrypted file
└── <timestamp>-<sha256>.json    # Metadata
```

**Security**:
- Copy-on-write (original untouched)
- Per-host encryption key
- Double-write verification
- Restore validates checksums

---

### 4. av-signatures (Update System)

**Purpose**: Securely fetch and verify signature updates

**Responsibilities**:
- Download YARA rule bundles
- Verify Ed25519 signatures
- TLS certificate pinning
- Rollback protection

**Update Flow**:
```
1. Fetch bundle from HTTPS endpoint
2. Verify TLS certificate (pinned)
3. Validate Ed25519 signature
4. Check semantic version (no downgrades)
5. Verify SHA-256 checksum
6. Atomically replace rules
```

---

### 5. av-cli (Command-Line Interface)

**Purpose**: User interaction and automation

**Commands**:
- `scan <path>`: On-demand scanning
- `realtime <on|off>`: Toggle daemon
- `quarantine list`: Show quarantined files
- `quarantine restore <id> <dest>`: Restore file
- `signatures update`: Fetch new rules
- `metrics`: Show statistics

**Output**:
- Human-readable by default
- `--json` flag for automation
- Exit codes follow POSIX conventions

---

## Component Communication

### IPC Mechanisms

| Source | Destination | Mechanism | Data Format |
|--------|-------------|-----------|-------------|
| av-cli | av-daemon | Unix socket | JSON |
| av-daemon | av-core | Direct (library) | Rust structs |
| av-daemon | av-quarantine | Direct (library) | Rust structs |
| av-signatures | Update server | HTTPS | JSON (signed) |

### Event Flow (Real-Time Scan)

```
1. Kernel detects file access (fanotify)
   ↓
2. av-daemon receives event
   ↓
3. av-daemon calls av-core.scan_path()
   ↓
4. av-core loads YARA rules
   ↓
5. av-core scans file content
   ↓
6. av-core calculates entropy
   ↓
7. av-core returns ScanOutcome
   ↓
8. av-daemon evaluates recommendation
   ↓
9a. Allow → send FanotifyResponse::Allow
9b. Quarantine → send FanotifyResponse::Deny + trigger quarantine
```

---

## Security Architecture

### Privilege Separation

```
┌─────────────────┬──────────────┬────────────────────┐
│ Component       │ User         │ Capabilities       │
├─────────────────┼──────────────┼────────────────────┤
│ av-cli          │ Current user │ None               │
│ av-daemon       │ avdaemon     │ None (default)     │
│                 │              │ CAP_SYS_ADMIN (opt)│
│ av-core (lib)   │ avdaemon     │ Inherited          │
│ av-quarantine   │ avdaemon     │ Inherited          │
└─────────────────┴──────────────┴────────────────────┘
```

### Sandboxing Layers

1. **AppArmor** (Mandatory Access Control)
   - Default-deny filesystem policy
   - Allow read: `/home/*`, `/tmp/*`, `/usr/lib/*`
   - Allow write: `/var/lib/av/*`, `/var/log/*`
   - Deny network (except AF_INET for updates)

2. **seccomp-bpf** (Syscall Filtering)
   - Whitelist: `read`, `write`, `open`, `stat`, `mmap`, etc.
   - Blacklist: `ptrace`, `reboot`, `kexec_load`, etc.
   - Platform: ARM64 only (SCMP_ARCH_AARCH64)

3. **systemd** (Resource Limits)
   - `ProtectSystem=strict`
   - `ProtectHome=true`
   - `NoNewPrivileges=true`
   - `PrivateTmp=true`
   - `RestrictNamespaces=true`

4. **Landlock** (Optional, v0.3+)
   - Path-based access control
   - Complementary to AppArmor
   - Feature-gated (`landlock_confine`)

---

## Data Flow

### Scanning Pipeline

```
File → Read (bounded) → YARA → Signatures
  ↓                        ↓
Entropy ← Chunks (4KB)     Metadata
  ↓                        ↓
Heuristic Scoring ← Combine
  ↓
RecommendedAction
```

### Quarantine Workflow

```
Suspicious File
  ↓
Copy to /tmp (COW)
  ↓
SHA-256 hash
  ↓
Encrypt (AES-256-GCM)
  ↓
Write to /var/lib/av/quarantine/
  ↓
Write metadata JSON
  ↓
Verify (double-read + hash check)
  ↓
Delete original (optional, user consent)
```

---

## Performance Optimizations

### ARM64-Specific

- **NEON SIMD**: Optional feature flag for crypto/hashing
- **Hardware AES**: Ring library uses ARM64 crypto extensions
- **Native compilation**: `--target aarch64-unknown-linux-gnu`

### Memory Management

- **Bounded reads**: Max 2MB per file scan
- **Lazy YARA loading**: Rules compiled once, reused
- **Zero-copy**: Use `&[u8]` slices instead of cloning
- **Lockless where possible**: `Arc<RwLock<T>>` for rule cache

### I/O Optimization

- **Async I/O**: Tokio for non-blocking file reads
- **Rayon parallelism**: Multi-threaded directory scanning (optional)
- **Bloom filters**: Fast negative matches (planned v0.3)

---

## Deployment Model

### Systemd Integration

```
/usr/lib/systemd/system/av-daemon.service
    ↓
ExecStart=/usr/lib/charmedwoa-av/av-daemon
    ↓
Restart=on-failure (10s delay)
    ↓
After=network-online.target
```

### File Layout

```
/usr/lib/charmedwoa-av/
├── av-daemon        # Binary
└── av-cli           # Binary

/etc/charmedwoa-av/
├── daemon.toml      # Configuration
└── rules/           # YARA rules
    └── production.yar

/var/lib/av/
├── quarantine/      # Encrypted files
└── signatures/      # Downloaded updates

/var/log/charmedwoa-av/
└── daemon.log       # Audit logs
```

---

## Threat Model

### In Scope

- Local malware execution prevention
- File integrity monitoring
- Ransomware detection (entropy + behavior)
- Crypto miner detection
- Backdoor/rootkit indicators

### Out of Scope

- Kernel-level rootkits (requires separate LSM)
- Network-based attacks (firewall responsibility)
- Zero-day exploits (heuristics only)
- Physical access attacks

### Assumptions

- Kernel is trusted (UEFI Secure Boot recommended)
- User has legitimate access to the system
- Update server not compromised (Ed25519 + TLS pinning)

---

## Future Architecture Changes

### v0.3.0 (Planned)
- GUI dashboard (Tauri + React)
- Cloud threat intelligence (optional)
- Behavioral analysis engine
- eBPF probes for kernel events

### v0.4.0 (Planned)
- Multi-platform support (x86_64, RISC-V)
- Distributed scanning (client/server)
- Machine learning heuristics
- AV-TEST certification

---

## References

- [YARA Documentation](https://yara.readthedocs.io/)
- [fanotify(7) man page](https://man7.org/linux/man-pages/man7/fanotify.7.html)
- [AppArmor Wiki](https://gitlab.com/apparmor/apparmor/-/wikis/home)
- [seccomp-bpf](https://www.kernel.org/doc/html/latest/userspace-api/seccomp_filter.html)
- [Tokio Async Runtime](https://tokio.rs/)

---

**Last Updated**: 2025-01-24
**Version**: 0.2.0
**Author**: Zachary Winn <zw@winncore.com>
