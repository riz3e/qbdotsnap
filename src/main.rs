mod config;
mod snapshot;
mod diff;
mod export;
mod watch;
mod notify;

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
        #[arg(short, long)]
        label: Option<String>,
    },
    /// List all snapshots
    List,
    /// Diff two snapshots (defaults to last two)
    Diff {
        /// Snapshot selector: index, timestamp prefix, 'last', or 'prev'
        #[arg(default_value = "prev")]
        from: String,
        #[arg(default_value = "last")]
        to: String,
        /// Only show diff for a specific file
        #[arg(short, long, value_name = "PATH")]
        file: Option<String>,
    },
    /// Export a human-readable summary
    Export {
        #[arg(default_value = "last")]
        snapshot: String,
    },
    /// Restore files from a snapshot
    Restore {
        /// Snapshot selector: index, timestamp prefix, 'last', or 'prev'
        snapshot: String,
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Delete a snapshot
    Delete {
        /// Snapshot selector: index, timestamp prefix, 'last', or 'prev'
        snapshot: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Watch tracked paths and auto-snapshot on changes
    Watch {
        #[arg(short, long, default_value = "5")]
        debounce: u64,
    },
    /// Enable or disable desktop notifications
    Notifications {
        /// 'on' or 'off'
        state: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = config::load()?;

    match cli.command {
        Commands::Take { label } => {
            let snap = snapshot::take(&cfg, label.as_deref())?;
            println!("✓ Snapshot #{} taken: {}", snap.index, snap.id);
            println!("  {} files captured", snap.file_count);
            notify::send(&cfg, notify::Event::SnapshotTaken {
                index: snap.index,
                id: &snap.id.clone(),
                file_count: snap.file_count,
            });
        }
        Commands::List => {
            let snaps = snapshot::list(&cfg)?;
            if snaps.is_empty() {
                println!("No snapshots yet. Run `qbdotsnap take` to create one.");
            } else {
                println!("{:<6} {:<26} {:<8} {}", "#", "timestamp", "files", "label");
                println!("{}", "-".repeat(58));
                for s in snaps {
                    println!(
                        "{:<6} {:<26} {:<8} {}",
                        s.index,
                        s.id,
                        s.file_count,
                        s.label.as_deref().unwrap_or("-")
                    );
                }
            }
        }
        Commands::Diff { from, to, file } => {
            diff::run(&cfg, &from, &to, file.as_deref())?;
        }
        Commands::Export { snapshot } => {
            export::run(&cfg, &snapshot)?;
        }
        Commands::Restore { snapshot, dry_run } => {
            snapshot::restore(&cfg, &snapshot, dry_run)?;
        }
        Commands::Delete { snapshot, force } => {
            snapshot::delete(&cfg, &snapshot, force)?;
        }
        Commands::Watch { debounce } => {
            watch::run(&cfg, debounce)?;
        }
        Commands::Notifications { state } => {
            match state.to_lowercase().as_str() {
                "on" | "true" | "1" => {
                    cfg.notifications = true;
                    config::save(&cfg)?;
                    println!("✓ Notifications enabled.");
                }
                "off" | "false" | "0" => {
                    cfg.notifications = false;
                    config::save(&cfg)?;
                    println!("✓ Notifications disabled.");
                }
                other => {
                    anyhow::bail!("Unknown state '{}'. Use 'on' or 'off'.", other);
                }
            }
        }
    }

    Ok(())
}