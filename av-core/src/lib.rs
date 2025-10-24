//! Core scanning library for the CharmedWOA ARM64 antivirus suite.
//!
//! This crate stays entirely in user space, obeying read-only defaults.
//! It exposes scanning primitives, heuristics, and extension points that
//! higher-level components (daemon, CLI, GUI) consume. All potentially
//! destructive actions (quarantine, remediation) are modelled as opt-in
//! workflows where callers must explicitly request mutation.
//!
//! Safety guarantees:
//! - All file interactions default to read-only and buffer bounded I/O.
//! - Heavy workloads flow through a bounded task pool to preserve system
//!   responsiveness on Snapdragon 8cx hardware.
//! - Optional NEON acceleration is guard-railed behind the `neon_accel`
//!   feature and runtime CPU feature detection.
//! - YARA-compatible rules are validated before execution, and every
//!   decision passes through the heuristic fusion layer for suppressions.

pub mod config;
pub mod engine;
pub mod heuristics;
pub mod monitoring;
pub mod signatures;
pub mod telemetry;

pub use config::ScannerConfig;

use std::path::Path;

/// High-level scanning interface that callers use to analyse a path.
///
/// Scanning is strictly read-only: callers receive a `ScanOutcome`
/// describing detections and recommended next steps. Escalations such as
/// quarantine must be carried out by the quarantine manager and require
/// explicit authorization from the initiating user.
pub struct Scanner {
    config: ScannerConfig,
}

impl Scanner {
    /// Construct a new scanner with the supplied configuration. The
    /// configuration is validated early so that feature-specific
    /// requirements (fanotify availability, NEON support, etc.) are surfaced
    /// before monitoring begins.
    pub fn new(config: ScannerConfig) -> anyhow::Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Perform a synchronous scan of the provided path.
    ///
    /// This method never mutates the target; it reads data using buffered
    /// I/O and returns heuristic scores and signature matches.
    pub async fn scan_path<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<ScanOutcome> {
        let context = engine::ScanContext::new(path.as_ref().to_path_buf());
        let result = engine::scan_path(&self.config, &context).await?;
        Ok(result)
    }
}

/// Result of a scan, containing structured metadata suitable for JSON
/// serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanOutcome {
    pub path: String,
    pub signatures: Vec<engine::SignatureMatch>,
    pub heuristic_score: heuristics::Score,
    pub entropy: engine::EntropyReport,
    pub recommended_action: RecommendedAction,
}

/// The scanner only *recommends* actions; mutating options are left to
/// higher-level components that enforce opt-in, reversible workflows.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum RecommendedAction {
    /// No malicious indicators were observed.
    Allow,
    /// Suspicious traits were observed; suggest monitoring.
    Monitor,
    /// High confidence detection; quarantine recommended but not automatic.
    Quarantine,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validates_configuration_before_scanning() {
        let cfg = ScannerConfig::default();
        let scanner = Scanner::new(cfg).expect("config should validate");
        let empty = scanner
            .scan_path(std::env::temp_dir())
            .await
            .expect("scan succeeds");
        assert!(matches!(empty.recommended_action, RecommendedAction::Allow));
    }
}
