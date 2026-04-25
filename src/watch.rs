use crate::config::Config;
use crate::notify;
use crate::snapshot;
use anyhow::Result;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

fn file_fingerprint(path: &std::path::Path) -> Option<(u64, u64)> {
    use std::time::UNIX_EPOCH;
    let metadata = std::fs::metadata(path).ok()?;
    let mtime = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((mtime, metadata.len()))
}

fn snapshot_fingerprints(cfg: &Config) -> HashMap<PathBuf, (u64, u64)> {
    let mut map: HashMap<PathBuf, (u64, u64)> = HashMap::new();
    for tracked in cfg.tracked_paths() {
        if tracked.is_file() {
            if let Some(fp) = file_fingerprint(&tracked) {
                map.insert(tracked, fp);
            }
        } else if tracked.is_dir() {
            for entry in walkdir::WalkDir::new(&tracked)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let p = entry.path().to_path_buf();
                if p.is_file() && !cfg.should_skip(&p) {
                    if let Some(fp) = file_fingerprint(&p) {
                        map.insert(p, fp);
                    }
                }
            }
        }
    }
    map

}

pub fn run(cfg: &Config, debounce_secs: u64) -> Result<()> {
    let paths = cfg.tracked_paths();

    if paths.is_empty() {
        anyhow::bail!("No paths to watch — check your ~/.qbdotsnap.toml");
    }

    println!("Watching {} paths (debounce: {}s)", paths.len(), debounce_secs);
    for p in &paths { println!("  {}", p.display()); }
    // println!("Press Ctrl+C to stop.\n");

    let (tx, rx) = mpsc::channel::<DebounceEventResult>();
    let mut debouncer = new_debouncer(Duration::from_secs(debounce_secs), tx)?;

    for path in &paths {
        if path.exists() {
            debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
        } else {
            eprintln!("warning: {} does not exist, skipping watch", path.display());
        }
    }

    let mut last_fingerprints = snapshot_fingerprints(cfg);

    for result in rx {
        match result {
            Ok(events) => {
                let current = snapshot_fingerprints(cfg);

                if current == last_fingerprints {
                    // No actual changes, likely just a temporary file update or similar
                    continue;
                }

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
                for p in &changed {
                    println!("  ~ {}", p);
                }

                match snapshot::take(cfg, Some("auto")) {
                    Ok(snap) => {
                        println!("  ✓ snapshot #{}: {}\n", snap.index, snap.id);
                        notify::send(
                            cfg,
                            notify::Event::SnapshotTaken {
                                index: snap.index,
                                id: &snap.id.clone(),
                                file_count: snap.file_count,
                            },
                        );
                        last_fingerprints = current;
                    }
                    Err(e) => eprintln!("  ✗ snapshot failed: {}\n", e),
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}