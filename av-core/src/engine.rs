use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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

pub async fn scan_path(config: &ScannerConfig, ctx: &ScanContext) -> anyhow::Result<crate::ScanOutcome> {
    let data = read_head(&ctx.target).await?;
    let signatures = evaluate_signatures(&data).await?;
    let heuristic_score = heuristics::score(&ctx.target, &data, config);
    let entropy = if config.enable_entropy_analysis {
        entropy(&data)
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
    let mut file = File::open(path).await?;
    let mut buffer = Vec::with_capacity(256 * 1024);
    file.take(256 * 1024).read_to_end(&mut buffer).await?;
    Ok(buffer)
}

async fn evaluate_signatures(_data: &[u8]) -> anyhow::Result<Vec<SignatureMatch>> {
    Ok(vec![])
}

fn entropy(_data: &[u8]) -> EntropyReport {
    EntropyReport::default()
}
