use crate::config::Config;
use crate::notify;
use crate::snapshot;
use anyhow::Result;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::sync::mpsc;
use std::time::Duration;

pub fn run(cfg: &Config, debounce_secs: u64) -> Result<()> {
    let paths = cfg.tracked_paths();

    if paths.is_empty() {
        anyhow::bail!("No paths to watch — check your ~/.qbdotsnap.toml");
    }

    println!("Watching {} paths (debounce: {}s)", paths.len(), debounce_secs);
    for p in &paths { println!("  {}", p.display()); }
    println!("Press Ctrl+C to stop.\n");

    let (tx, rx) = mpsc::channel::<DebounceEventResult>();
    let mut debouncer = new_debouncer(Duration::from_secs(debounce_secs), tx)?;

    for path in &paths {
        if path.exists() {
            debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
        } else {
            eprintln!("warning: {} does not exist, skipping watch", path.display());
        }
    }

    for result in rx {
        match result {
            Ok(events) => {
                let mut changed: Vec<String> = events
                    .iter()
                    .map(|e| e.path.to_string_lossy().into_owned())
                    .collect();
                changed.dedup();

                println!(
                    "[{}] change detected in {} file(s):",
                    chrono::Local::now().format("%H:%M:%S"),
                    changed.len()
                );
                for p in &changed { println!("  ~ {}", p); }

                match snapshot::take(cfg, Some("auto")) {
                    Ok(snap) => {
                        println!("  ✓ snapshot #{}: {}\n", snap.index, snap.id);
                        notify::send(cfg, notify::Event::SnapshotTaken {
                            index: snap.index,
                            id: &snap.id.clone(),
                            file_count: snap.file_count,
                        });
                    }
                    Err(e) => eprintln!("  ✗ snapshot failed: {}\n", e),
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}