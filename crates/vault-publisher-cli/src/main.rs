use clap::{Parser, Subcommand};
use publisher_core::config::PublisherConfig;
use publisher_core::daemon::Daemon;
use publisher_core::engine::PublisherEngine;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

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
    /// Run as background daemon with file watching and periodic reconciliation
    Daemon {
        /// Path to configuration JSON
        config: PathBuf,
        /// Profile ID
        #[arg(short, long)]
        profile: String,
        /// Quiet window for event debouncing in seconds (default 2)
        #[arg(short, long, default_value_t = 2)]
        quiet_seconds: u64,
        /// Periodic reconciliation interval in minutes (default 15)
        #[arg(short, long, default_value_t = 15)]
        interval_minutes: u64,
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
        Commands::Daemon {
            config,
            profile,
            quiet_seconds,
            interval_minutes,
        } => {
            let data = fs::read_to_string(&config)?;
            let parsed: PublisherConfig = serde_json::from_str(&data)?;
            println!("Starting daemon for profile '{}'...", profile);
            let mut daemon = Daemon::start(
                parsed,
                profile.clone(),
                Duration::from_secs(quiet_seconds),
                Duration::from_secs(interval_minutes * 60),
                move |report| {
                    println!(
                        "[{}] Run complete: +{} ~{} -{} ({} skipped, {} conflicts, {} warnings)",
                        chrono_lite(),
                        report.created_count,
                        report.updated_count,
                        report.archived_count,
                        report.skipped_count,
                        report.conflict_count,
                        report.warn_count
                    );
                },
            )?;

            println!("Daemon active with lock. Press Ctrl+C to terminate.");
            let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
            let r = running.clone();
            ctrlc_lite(move || {
                println!("\nShutdown signal received. Stopping daemon...");
                r.store(false, std::sync::atomic::Ordering::SeqCst);
            });

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(100));
            }

            daemon.stop();
            println!("Daemon stopped cleanly. Lock released.");
        }
    }

    Ok(())
}

fn chrono_lite() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

fn ctrlc_lite<F: FnOnce() + Send + 'static>(_handler: F) {
    // Basic signal hook stub for CLI termination
}
