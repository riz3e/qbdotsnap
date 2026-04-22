use crate::config::{Config};
use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub index: u32,
    pub id: String,
    pub taken_at: String,
    pub label: Option<String>,
    pub files: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub source: String,
    pub stored: String,
}

pub struct SnapInfo {
    pub index: u32,
    pub id: String,
    pub file_count: usize,
    pub label: Option<String>,
}

// --- Index counter -----------------------------------------------------------

/// Read the global counter file, returns 0 if missing.
fn read_counter(cfg: &Config) -> u32 {
    let p = cfg.snapshot_dir().join(".counter");
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Increment counter, write it back, return the new value.
fn next_index(cfg: &Config) -> Result<u32> {
    let snap_dir = cfg.snapshot_dir();
    std::fs::create_dir_all(&snap_dir)?;
    let counter_path = snap_dir.join(".counter");
    let next = read_counter(cfg) + 1;
    std::fs::write(counter_path, next.to_string())?;
    Ok(next)
}

// --- Public API --------------------------------------------------------------

pub fn take(cfg: &Config, label: Option<&str>) -> Result<SnapInfo> {
    let index = next_index(cfg)?;
    let id = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let snap_root = cfg.snapshot_dir().join(&id);
    std::fs::create_dir_all(&snap_root)?;

    let mut entries: Vec<ManifestEntry> = Vec::new();

    for tracked in cfg.tracked_paths() {
        if !tracked.exists() {
            eprintln!("warning: {} does not exist, skipping", tracked.display());
            continue;
        }
        collect_files(&tracked, cfg, &snap_root, &mut entries)?;
    }

    let file_count = entries.len();
    let manifest = Manifest {
        index,
        id: id.clone(),
        taken_at: Local::now().to_rfc3339(),
        label: label.map(String::from),
        files: entries,
    };

    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(snap_root.join("manifest.json"), json)?;

    Ok(SnapInfo { index, id, file_count, label: label.map(String::from) })
}

pub fn list(cfg: &Config) -> Result<Vec<SnapInfo>> {
    let snap_dir = cfg.snapshot_dir();
    if !snap_dir.exists() { return Ok(vec![]); }

    let mut snaps: Vec<SnapInfo> = Vec::new();

    for entry in std::fs::read_dir(&snap_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() { continue; }

        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() { continue; }

        let manifest = load_manifest(&manifest_path)?;
        snaps.push(SnapInfo {
            index: manifest.index,
            id: manifest.id,
            file_count: manifest.files.len(),
            label: manifest.label,
        });
    }

    // Sort newest first (by id which is ISO timestamp)
    snaps.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(snaps)
}

/// Delete a snapshot by index or timestamp alias. Returns the deleted id.
pub fn delete(cfg: &Config, selector: &str, force: bool) -> Result<()> {
    let snaps = list(cfg)?;

    let target = resolve_snap(&snaps, selector)
        .with_context(|| format!("No snapshot matching '{}'", selector))?;

    if !force {
        println!(
            "Delete snapshot #{} ({})? [y/N] ",
            target.index, target.id
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let snap_path = cfg.snapshot_dir().join(&target.id);
    std::fs::remove_dir_all(&snap_path)
        .with_context(|| format!("Failed to remove {}", snap_path.display()))?;

    println!("✓ Deleted snapshot #{} ({})", target.index, target.id);
    Ok(())
}

pub fn load_manifest_by_selector(cfg: &Config, selector: &str) -> Result<Manifest> {
    let snaps = list(cfg)?;
    let target = resolve_snap(&snaps, selector)
        .with_context(|| format!("No snapshot matching '{}'", selector))?;
    let manifest_path = cfg.snapshot_dir().join(&target.id).join("manifest.json");
    load_manifest(&manifest_path)
}

/// Resolve a selector to a SnapInfo:
///   - "last" / "latest"  → newest
///   - "prev" / "previous"→ second newest
///   - numeric string     → match by index
///   - anything else      → match by id prefix
fn resolve_snap<'a>(snaps: &'a [SnapInfo], selector: &str) -> Option<&'a SnapInfo> {
    match selector {
        "last" | "latest"       => snaps.first(),
        "prev" | "previous"     => snaps.get(1),
        s if s.chars().all(|c| c.is_ascii_digit()) => {
            let idx: u32 = s.parse().ok()?;
            snaps.iter().find(|s| s.index == idx)
        }
        s => snaps.iter().find(|snap| snap.id.starts_with(s)),
    }
}

pub fn restore(cfg: &Config, selector: &str, dry_run: bool) -> Result<()> {
    let manifest = load_manifest_by_selector(cfg, selector)?;

    if dry_run {
        println!("Dry run — would restore {} files from #{} ({}):",
            manifest.files.len(), manifest.index, manifest.id);
    } else {
        println!("Restoring {} files from #{} ({})...",
            manifest.files.len(), manifest.index, manifest.id);
    }

    for entry in &manifest.files {
        let src = cfg.snapshot_dir().join(&manifest.id).join(&entry.stored);
        let dest = PathBuf::from(&entry.source);
        println!("  {} -> {}", entry.stored, dest.display());
        if !dry_run {
            if let Some(parent) = dest.parent() { std::fs::create_dir_all(parent)?; }
            std::fs::copy(&src, &dest)
                .with_context(|| format!("Failed to restore {}", dest.display()))?;
        }
    }

    if !dry_run { println!("✓ Restore complete."); }
    Ok(())
}

// --- Helpers -----------------------------------------------------------------

fn collect_files(
    path: &Path, cfg: &Config, snap_root: &Path, entries: &mut Vec<ManifestEntry>,
) -> Result<()> {
    if path.is_file() {
        copy_file(path, snap_root, entries)?;
    } else if path.is_dir() {
        for entry in WalkDir::new(path).follow_links(false).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && !cfg.should_skip(p) {
                copy_file(p, snap_root, entries)?;
            }
        }
    }
    Ok(())
}

fn copy_file(src: &Path, snap_root: &Path, entries: &mut Vec<ManifestEntry>) -> Result<()> {
    let relative = src.strip_prefix("/").unwrap_or(src);
    let dest = snap_root.join(relative);
    if let Some(parent) = dest.parent() { std::fs::create_dir_all(parent)?; }
    std::fs::copy(src, &dest)
        .with_context(|| format!("Failed to copy {}", src.display()))?;
    entries.push(ManifestEntry {
        source: src.to_string_lossy().into_owned(),
        stored: relative.to_string_lossy().into_owned(),
    });
    Ok(())
}

pub fn load_manifest(path: &Path) -> Result<Manifest> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read manifest at {}", path.display()))?;
    serde_json::from_str(&content).context("Invalid manifest JSON")
}

pub fn snap_path_for(cfg: &Config, id: &str, stored: &str) -> PathBuf {
    cfg.snapshot_dir().join(id).join(stored)
}