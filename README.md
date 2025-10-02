# Quantum-Resistant File Monitoring

A high-performance, ARM64-optimized file monitoring system with SHA-512 cryptographic hashing for post-quantum security considerations.

[![Platform](https://img.shields.io/badge/platform-Linux%20ARM64%20%7C%20x86__64-blue)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)]()

## Overview

SecureMonitor is a lightweight file integrity monitoring system built in Rust, specifically optimized for ARM64 processors (Snapdragon X Elite, AWS Graviton, Raspberry Pi 4+). It uses SHA-512 hashing to provide 256-bit quantum security against Grover's algorithm attacks.

### Key Features

- **Quantum-Resistant Hashing**: SHA-512 provides 256-bit security even against quantum computers
- **ARM64 Optimized**: Compiled with native CPU optimizations for Snapdragon and Graviton processors
- **Extremely Lightweight**: Uses only 4-5MB RAM during active monitoring
- **Recursive Monitoring**: Automatically watches subdirectories without manual configuration
- **Smart Path Filtering**: Excludes high-noise directories (node_modules, .git, build artifacts)
- **Real-time SQLite Logging**: All events stored in queryable database
- **Large File Handling**: Configurable thresholds prevent performance degradation

## Performance Benchmarks

Tested on ThinkPad X13s Gen 1 (Snapdragon ARM64):

| Metric | Result |
|--------|--------|
| Memory Usage | 4.4 MB RAM |
| Binary Size | 3.1 MB |
| Hash Speed (SHA-512) | ~300 MB/s |
| Stress Test | 150 events/second, 0 errors |
| CPU Usage (idle) | <1% |
| CPU Usage (active) | ~5% |

## Why Quantum-Resistant?

SHA-512 provides 256-bit security against quantum computers using Grover's algorithm, compared to SHA-256's 128-bit quantum security.

### Security Comparison

| Algorithm | Classical Security | Quantum Security |
|-----------|-------------------|------------------|
| MD5 | Broken | N/A |
| SHA-1 | Broken | N/A |
| SHA-256 | 256-bit | 128-bit |
| **SHA-512** | **512-bit** | **256-bit** |

## Installation

### Prerequisites

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
