mod config;
mod snapshot;
mod diff;
mod export;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qbdotsnap")]
#[command(about = "Snapshot and diff your dotfiles", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Take a new snapshot of all tracked files
    Take {
        /// Optional label for this snapshot
        #[arg(short, long)]
        label: Option<String>,
    },
    /// List all snapshots
    List,
    /// Diff two snapshots (defaults to last two)
    Diff {
        /// First snapshot timestamp (or 'last', 'prev')
        #[arg(default_value = "prev")]
        from: String,
        /// Second snapshot timestamp (defaults to latest)
        #[arg(default_value = "last")]
        to: String,
    },
    /// Export a human-readable summary of your current config
    Export {
        /// Snapshot to export (defaults to latest)
        #[arg(default_value = "last")]
        snapshot: String,
    },
    /// Restore files from a snapshot
    Restore {
        /// Snapshot timestamp to restore from
        snapshot: String,
        /// Dry run — show what would be restored without doing it
        #[arg(short, long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load()?;

    match cli.command {
        Commands::Take { label } => {
            let snap = snapshot::take(&cfg, label.as_deref())?;
            println!("✓ Snapshot taken: {}", snap.id);
            println!("  {} files captured", snap.file_count);
        }
        Commands::List => {
            let snaps = snapshot::list(&cfg)?;
            if snaps.is_empty() {
                println!("No snapshots yet. Run `qbdotsnap take` to create one.");
            } else {
                println!("{:<26} {:<8} {}", "timestamp", "files", "label");
                println!("{}", "-".repeat(50));
                for s in snaps {
                    println!(
                        "{:<26} {:<8} {}",
                        s.id,
                        s.file_count,
                        s.label.as_deref().unwrap_or("-")
                    );
                }
            }
        }
        Commands::Diff { from, to } => {
            diff::run(&cfg, &from, &to)?;
        }
        Commands::Export { snapshot } => {
            export::run(&cfg, &snapshot)?;
        }
        Commands::Restore { snapshot, dry_run } => {
            snapshot::restore(&cfg, &snapshot, dry_run)?;
        }
    }

    Ok(())
}
