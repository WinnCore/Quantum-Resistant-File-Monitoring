use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;

use av_core::{Scanner, ScannerConfig};
use av_quarantine::{QuarantineConfig, QuarantineManager};

#[derive(Parser, Debug)]
#[command(author, version, about = "CharmedWOA ARM64 Antivirus CLI", propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output JSON for machine parsing.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Scan { path: PathBuf },
    Realtime { state: Toggle },
    Quarantine { command: QuarantineCmd },
    Signatures { command: SignatureCmd },
    Metrics,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum Toggle {
    On,
    Off,
}

#[derive(Subcommand, Debug)]
enum QuarantineCmd {
    List,
    Restore { id: String, destination: PathBuf },
}

#[derive(Subcommand, Debug)]
enum SignatureCmd {
    Update,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let rt = Runtime::new()?;
    match cli.command {
        Commands::Scan { path } => run_scan(&rt, path, cli.json),
        Commands::Realtime { state } => set_realtime(state),
        Commands::Quarantine { command } => run_quarantine(command, cli.json),
        Commands::Signatures { command } => run_signatures(&rt, command),
        Commands::Metrics => show_metrics(cli.json),
    }
}

fn run_scan(rt: &Runtime, path: PathBuf, json: bool) -> anyhow::Result<()> {
    let cfg = ScannerConfig::default();
    let scanner = Scanner::new(cfg)?;
    let outcome = rt.block_on(scanner.scan_path(path))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!("Result: {:?}", outcome.recommended_action);
        println!("Signatures: {}", outcome.signatures.len());
        println!("Score: {:.3}", outcome.heuristic_score.0);
    }
    Ok(())
}

fn set_realtime(state: Toggle) -> anyhow::Result<()> {
    println!("Realtime mode set to {:?} (placeholder)", state);
    Ok(())
}

fn run_quarantine(cmd: QuarantineCmd, json: bool) -> anyhow::Result<()> {
    let manager = QuarantineManager::new(QuarantineConfig::default())?;
    match cmd {
        QuarantineCmd::List => {
            let entries = std::fs::read_dir("/var/lib/av/quarantine")?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().map(|ext| ext == "json").unwrap_or(false))
                .collect::<Vec<_>>();
            if json {
                println!("{}", serde_json::to_string_pretty(&entries.len())?);
            } else {
                println!("{} items in quarantine", entries.len());
            }
        }
        QuarantineCmd::Restore { id, destination } => {
            let metadata_path = format!("/var/lib/av/quarantine/{}.json", id);
            let record: av_quarantine::QuarantineRecord = serde_json::from_slice(&std::fs::read(metadata_path)?)?;
            manager.restore(&record, &destination)?;
            println!("Restored {}", id);
        }
    }
    Ok(())
}

fn run_signatures(rt: &Runtime, command: SignatureCmd) -> anyhow::Result<()> {
    match command {
        SignatureCmd::Update => {
            rt.block_on(async {
                println!("Updating signatures (placeholder)");
                Ok::<(), anyhow::Error>(())
            })?;
        }
    }
    Ok(())
}

fn show_metrics(json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::json!({"uptime": 0, "events": 0}));
    } else {
        println!("Uptime: 0s\nEvents processed: 0");
    }
    Ok(())
}
