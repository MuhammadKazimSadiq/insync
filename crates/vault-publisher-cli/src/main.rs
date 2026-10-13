use clap::{Parser, Subcommand};
use publisher_core::config::PublisherConfig;
use publisher_core::engine::PublisherEngine;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vault-publisher")]
#[command(about = "Rule-based one-way vault publisher", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate configuration file
    Validate {
        /// Path to configuration JSON
        config: PathBuf,
    },
    /// Preview planned publishing operations
    Preview {
        /// Path to configuration JSON
        config: PathBuf,
        /// Profile ID
        #[arg(short, long)]
        profile: String,
    },
    /// Apply planned publishing operations
    Apply {
        /// Path to configuration JSON
        config: PathBuf,
        /// Profile ID
        #[arg(short, long)]
        profile: String,
    },
    /// Full reconciliation pass
    Reconcile {
        /// Path to configuration JSON
        config: PathBuf,
        /// Profile ID
        #[arg(short, long)]
        profile: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let engine = PublisherEngine::new();

    match cli.command {
        Commands::Validate { config } => {
            let data = fs::read_to_string(&config)?;
            let parsed: PublisherConfig = serde_json::from_str(&data)?;
            println!("Configuration valid: {} profiles found", parsed.profiles.len());
        }
        Commands::Preview { config, profile } => {
            let data = fs::read_to_string(&config)?;
            let parsed: PublisherConfig = serde_json::from_str(&data)?;
            let plan = engine.preview(&parsed, &profile)?;
            println!("Plan for profile '{}':", profile);
            println!("Found {} action(s):", plan.actions.len());
            for action in &plan.actions {
                println!(" - {:?}", action);
            }
        }
        Commands::Apply { config, profile } => {
            let data = fs::read_to_string(&config)?;
            let parsed: PublisherConfig = serde_json::from_str(&data)?;
            let plan = engine.preview(&parsed, &profile)?;
            let report = engine.apply(plan)?;
            println!("Applied successfully for profile '{}':", profile);
            println!("  Created: {}", report.created_count);
            println!("  Updated: {}", report.updated_count);
            println!("  Archived: {}", report.archived_count);
            println!("  Skipped: {}", report.skipped_count);
            println!("  Conflicts: {}", report.conflict_count);
            println!("  Warnings: {}", report.warn_count);
        }
        Commands::Reconcile { config, profile } => {
            let data = fs::read_to_string(&config)?;
            let parsed: PublisherConfig = serde_json::from_str(&data)?;
            let report = engine.reconcile(&parsed, &profile)?;
            println!("Reconciled successfully for profile '{}':", profile);
            println!("  Created: {}", report.created_count);
            println!("  Updated: {}", report.updated_count);
            println!("  Archived: {}", report.archived_count);
            println!("  Skipped: {}", report.skipped_count);
            println!("  Conflicts: {}", report.conflict_count);
            println!("  Warnings: {}", report.warn_count);
        }
    }

    Ok(())
}
