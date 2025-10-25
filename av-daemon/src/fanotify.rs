//! Fanotify-based real-time file monitoring for ARM64 Linux.
//!
//! This module implements permission-based interception using fanotify,
//! falling back to audit-only inotify when CAP_SYS_ADMIN is unavailable.

use std::fs::File;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

use anyhow::Context;
use tracing::{debug, error, info, warn};

use crate::config::ScannerConfig;
use crate::Scanner;

/// Fanotify monitoring mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorMode {
    /// Audit-only: log events but never block
    AuditOnly,
    /// Permission: can deny file access based on scan results
    Permission,
}

/// File access event from fanotify
#[derive(Debug)]
pub struct FileEvent {
    pub path: PathBuf,
    pub pid: i32,
    pub fd: i32,
    pub event_type: EventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Open,
    Execute,
    Modify,
}

/// Fanotify monitor for real-time scanning
pub struct FanotifyMonitor {
    scanner: Scanner,
    mode: MonitorMode,
    monitored_paths: Vec<PathBuf>,
}

impl FanotifyMonitor {
    pub fn new(scanner: Scanner, mode: MonitorMode, paths: Vec<PathBuf>) -> Self {
        Self {
            scanner,
            mode,
            monitored_paths: paths,
        }
    }

    /// Start monitoring (blocks until shutdown signal)
    pub async fn run(&self) -> anyhow::Result<()> {
        match self.mode {
            MonitorMode::Permission => self.run_permission_mode().await,
            MonitorMode::AuditOnly => self.run_audit_mode().await,
        }
    }

    async fn run_permission_mode(&self) -> anyhow::Result<()> {
        info!("Starting fanotify in PERMISSION mode");

        // Check if we can use fanotify permission mode
        if !can_use_fanotify_perm() {
            warn!("CAP_SYS_ADMIN not available, falling back to audit-only");
            return self.run_audit_mode().await;
        }

        #[cfg(target_os = "linux")]
        {
            use fanotify::high_level::{Fanotify, FanotifyMode, FanotifyResponse};

            let mut fan = Fanotify::new_with_nonblocking(FanotifyMode::CONTENT)?;

            // Mark monitored paths
            for path in &self.monitored_paths {
                fan.add_path(
                    fanotify::high_level::AddPathMode::FILE,
                    path,
                )?;
                info!("Monitoring path: {}", path.display());
            }

            loop {
                match fan.read_event() {
                    Ok(Some(event)) => {
                        let path = event.path().unwrap_or_else(|| PathBuf::from("<unknown>"));
                        debug!("fanotify event: {:?} on {}", event.mask(), path.display());

                        // Scan the file
                        let scan_result = self.scanner.scan_path(&path).await;

                        let response = match scan_result {
                            Ok(outcome) => {
                                match outcome.recommended_action {
                                    crate::RecommendedAction::Allow => {
                                        debug!("ALLOW: {}", path.display());
                                        FanotifyResponse::Allow
                                    }
                                    crate::RecommendedAction::Monitor => {
                                        warn!("SUSPICIOUS: {} (score: {:.2})", path.display(), outcome.heuristic_score.0);
                                        FanotifyResponse::Allow
                                    }
                                    crate::RecommendedAction::Quarantine => {
                                        error!("BLOCKED: {} - matches: {:?}", path.display(), outcome.signatures);
                                        FanotifyResponse::Deny
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Scan error for {}: {}, allowing", path.display(), e);
                                FanotifyResponse::Allow
                            }
                        };

                        // Send response
                        event.response(response)?;
                    }
                    Ok(None) => {
                        // No events, sleep briefly
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        error!("fanotify read error: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            anyhow::bail!("fanotify is only available on Linux");
        }
    }

    async fn run_audit_mode(&self) -> anyhow::Result<()> {
        info!("Starting in AUDIT-ONLY mode (inotify fallback)");

        use notify::{Watcher, RecursiveMode, Event};
        use tokio::sync::mpsc;

        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })?;

        // Watch monitored paths
        for path in &self.monitored_paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
            info!("Watching path (audit-only): {}", path.display());
        }

        while let Some(event) = rx.recv().await {
            for path in event.paths {
                debug!("inotify event: {:?} on {}", event.kind, path.display());

                // Scan file but don't block (audit-only)
                if let Ok(outcome) = self.scanner.scan_path(&path).await {
                    match outcome.recommended_action {
                        crate::RecommendedAction::Quarantine => {
                            warn!("DETECTED (audit): {} - matches: {:?}", path.display(), outcome.signatures);
                        }
                        crate::RecommendedAction::Monitor => {
                            info!("SUSPICIOUS (audit): {} - score: {:.2}", path.display(), outcome.heuristic_score.0);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

/// Check if we have CAP_SYS_ADMIN for fanotify permission mode
fn can_use_fanotify_perm() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        // Check /proc/self/status for CapEff
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("CapEff:") {
                    if let Some(cap_hex) = line.split_whitespace().nth(1) {
                        if let Ok(caps) = u64::from_str_radix(cap_hex, 16) {
                            // CAP_SYS_ADMIN = 1 << 21
                            return (caps & (1 << 21)) != 0;
                        }
                    }
                }
            }
        }
    }
    false
}
