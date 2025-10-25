use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{debug, warn};

use crate::config::ScannerConfig;
use crate::heuristics::{self, Score};

#[derive(Debug, Clone)]
pub struct ScanContext {
    pub target: PathBuf,
}

impl ScanContext {
    pub fn new(target: PathBuf) -> Self {
        Self { target }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMatch {
    pub rule: String,
    pub namespace: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntropyReport {
    pub mean_entropy: f32,
    pub suspicious_regions: Vec<(u64, u64)>,
}

/// YARA engine wrapper with lazy compilation
pub struct YaraEngine {
    compiler: Arc<RwLock<Option<yara::Compiler>>>,
    rules: Arc<RwLock<Option<yara::Rules>>>,
}

impl YaraEngine {
    pub fn new() -> Self {
        Self {
            compiler: Arc::new(RwLock::new(None)),
            rules: Arc::new(RwLock::new(None)),
        }
    }

    /// Load YARA rules from file
    pub fn load_rules(&self, rules_path: &str) -> anyhow::Result<()> {
        let mut compiler = yara::Compiler::new()?;
        compiler = compiler
            .add_rules_file(rules_path)
            .map_err(|e| anyhow::anyhow!("Failed to load YARA rules: {:?}", e))?;

        let rules = compiler
            .compile_rules()
            .map_err(|e| anyhow::anyhow!("Failed to compile YARA rules: {:?}", e))?;

        *self.compiler.write() = Some(compiler);
        *self.rules.write() = Some(rules);

        debug!("YARA rules loaded from {}", rules_path);
        Ok(())
    }

    /// Scan data with loaded rules
    pub fn scan(&self, data: &[u8]) -> anyhow::Result<Vec<SignatureMatch>> {
        let rules_guard = self.rules.read();
        let rules = rules_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("YARA rules not loaded"))?;

        let scan_results = rules
            .scan_mem(data, 30) // 30 second timeout
            .map_err(|e| anyhow::anyhow!("YARA scan failed: {:?}", e))?;

        let mut matches = Vec::new();
        for rule in scan_results {
            let mut metadata = HashMap::new();

            // Extract metadata
            for (key, value) in rule.metadatas {
                let json_value = match value {
                    yara::MetadataValue::Integer(i) => serde_json::json!(i),
                    yara::MetadataValue::String(s) => serde_json::json!(s),
                    yara::MetadataValue::Boolean(b) => serde_json::json!(b),
                };
                metadata.insert(key, json_value);
            }

            matches.push(SignatureMatch {
                rule: rule.identifier.clone(),
                namespace: rule.namespace.clone(),
                metadata: serde_json::to_value(metadata)?,
            });
        }

        if !matches.is_empty() {
            debug!("YARA detected {} rule matches", matches.len());
        }

        Ok(matches)
    }
}

impl Default for YaraEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn scan_path(config: &ScannerConfig, ctx: &ScanContext) -> anyhow::Result<crate::ScanOutcome> {
    let data = read_head(&ctx.target).await?;
    let signatures = evaluate_signatures(&data).await?;
    let heuristic_score = heuristics::score(&ctx.target, &data, config);
    let entropy = if config.enable_entropy_analysis {
        calculate_entropy(&data)
    } else {
        EntropyReport::default()
    };
    let recommended_action = heuristics::recommend(&signatures, heuristic_score, config);

    Ok(crate::ScanOutcome {
        path: ctx.target.display().to_string(),
        signatures,
        heuristic_score,
        entropy,
        recommended_action,
    })
}

async fn read_head(path: &PathBuf) -> anyhow::Result<Vec<u8>> {
    let metadata = tokio::fs::metadata(path).await?;
    let size = metadata.len();

    // Read up to 2MB for signature scanning
    let read_limit = std::cmp::min(size, 2 * 1024 * 1024);

    let mut file = File::open(path).await?;
    let mut buffer = Vec::with_capacity(read_limit as usize);
    file.take(read_limit).read_to_end(&mut buffer).await?;

    debug!("Read {} bytes from {}", buffer.len(), path.display());
    Ok(buffer)
}

async fn evaluate_signatures(data: &[u8]) -> anyhow::Result<Vec<SignatureMatch>> {
    // Initialize YARA engine with production rules
    let engine = YaraEngine::new();

    // Try to load production rules, fallback to example rules
    let rules_paths = vec![
        "rules/production.yar",
        "/etc/charmedwoa-av/rules/production.yar",
        "rules/example.yar",
    ];

    for path in rules_paths {
        if std::path::Path::new(path).exists() {
            match engine.load_rules(path) {
                Ok(_) => {
                    debug!("Loaded YARA rules from {}", path);
                    return engine.scan(data);
                }
                Err(e) => {
                    warn!("Failed to load rules from {}: {}", path, e);
                }
            }
        }
    }

    // No rules available - return empty matches
    warn!("No YARA rules found, scanning disabled");
    Ok(vec![])
}

fn calculate_entropy(data: &[u8]) -> EntropyReport {
    if data.is_empty() {
        return EntropyReport::default();
    }

    // Calculate Shannon entropy
    let mut freq = [0u32; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }

    let len = data.len() as f32;
    let entropy: f32 = freq
        .iter()
        .filter(|&&count| count > 0)
        .map(|&count| {
            let p = count as f32 / len;
            -p * p.log2()
        })
        .sum();

    // Detect suspicious high-entropy regions (possible encryption/packing)
    let mut suspicious_regions = Vec::new();
    const CHUNK_SIZE: usize = 4096;

    for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
        let chunk_entropy = calculate_chunk_entropy(chunk);
        // High entropy (> 7.5) suggests encryption or compression
        if chunk_entropy > 7.5 {
            let start = i * CHUNK_SIZE;
            let end = start + chunk.len();
            suspicious_regions.push((start as u64, end as u64));
        }
    }

    EntropyReport {
        mean_entropy: entropy,
        suspicious_regions,
    }
}

fn calculate_chunk_entropy(chunk: &[u8]) -> f32 {
    if chunk.is_empty() {
        return 0.0;
    }

    let mut freq = [0u32; 256];
    for &byte in chunk {
        freq[byte as usize] += 1;
    }

    let len = chunk.len() as f32;
    freq.iter()
        .filter(|&&count| count > 0)
        .map(|&count| {
            let p = count as f32 / len;
            -p * p.log2()
        })
        .sum()
}
