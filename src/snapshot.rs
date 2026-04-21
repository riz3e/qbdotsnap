use crate::config::{Config};
use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub taken_at: String,
    pub label: Option<String>,
    pub files: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Original path on the system (e.g. /home/user/.zshrc)
    pub source: String,
    /// Path inside snapshot dir (e.g. home/user/.zshrc)
    pub stored: String,
}

pub struct SnapInfo {
    pub id: String,
    pub file_count: usize,
    pub label: Option<String>,
}

/// Take a new snapshot of all tracked files/dirs.
pub fn take(cfg: &Config, label: Option<&str>) -> Result<SnapInfo> {
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
        id: id.clone(),
        taken_at: Local::now().to_rfc3339(),
        label: label.map(String::from),
        files: entries,
    };

    let manifest_path = snap_root.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(manifest_path, json)?;

    Ok(SnapInfo {
        id,
        file_count,
        label: label.map(String::from),
    })
}

fn collect_files(
    path: &Path,
    cfg: &Config,
    snap_root: &Path,
    entries: &mut Vec<ManifestEntry>,
) -> Result<()> {
    if path.is_file() {
        copy_file(path, snap_root, entries)?;
    } else if path.is_dir() {
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.is_file() && !cfg.should_skip(p) {
                copy_file(p, snap_root, entries)?;
            }
        }
    }
    Ok(())
}

fn copy_file(src: &Path, snap_root: &Path, entries: &mut Vec<ManifestEntry>) -> Result<()> {
    // Strip leading '/' to get a relative path for storage
    let relative = src.strip_prefix("/").unwrap_or(src);
    let dest = snap_root.join(relative);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(src, &dest)
        .with_context(|| format!("Failed to copy {}", src.display()))?;

    entries.push(ManifestEntry {
        source: src.to_string_lossy().into_owned(),
        stored: relative.to_string_lossy().into_owned(),
    });

    Ok(())
}

/// List all snapshots, newest first.
pub fn list(cfg: &Config) -> Result<Vec<SnapInfo>> {
    let snap_dir = cfg.snapshot_dir();
    if !snap_dir.exists() {
        return Ok(vec![]);
    }

    let mut snaps: Vec<SnapInfo> = Vec::new();

    for entry in std::fs::read_dir(&snap_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }

        let manifest = load_manifest(&manifest_path)?;
        snaps.push(SnapInfo {
            id: manifest.id,
            file_count: manifest.files.len(),
            label: manifest.label,
        });
    }

    snaps.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(snaps)
}

/// Load a manifest by snapshot id, or by alias ("last", "prev").
pub fn load_manifest_by_id(cfg: &Config, id: &str) -> Result<Manifest> {
    let snap_dir = cfg.snapshot_dir();
    let resolved_id = resolve_id(cfg, id)?;
    let manifest_path = snap_dir.join(&resolved_id).join("manifest.json");
    load_manifest(&manifest_path)
}

pub fn resolve_id(cfg: &Config, id: &str) -> Result<String> {
    match id {
        "last" | "latest" => {
            let snaps = list(cfg)?;
            snaps.into_iter().next().map(|s| s.id)
                .context("No snapshots found")
        }
        "prev" | "previous" => {
            let snaps = list(cfg)?;
            snaps.into_iter().nth(1).map(|s| s.id)
                .context("Need at least two snapshots for 'prev'")
        }
        other => Ok(other.to_string()),
    }
}

pub fn load_manifest(path: &Path) -> Result<Manifest> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read manifest at {}", path.display()))?;
    serde_json::from_str(&content).context("Invalid manifest JSON")
}

/// Restore files from a snapshot back to their original locations.
pub fn restore(cfg: &Config, id: &str, dry_run: bool) -> Result<()> {
    let snap_dir = cfg.snapshot_dir();
    let manifest = load_manifest_by_id(cfg, id)?;

    if dry_run {
        println!("Dry run — would restore {} files from {}:", manifest.files.len(), manifest.id);
    } else {
        println!("Restoring {} files from {}...", manifest.files.len(), manifest.id);
    }

    for entry in &manifest.files {
        let src = snap_dir.join(&manifest.id).join(&entry.stored);
        let dest = PathBuf::from(&entry.source);

        println!("  {} -> {}", entry.stored, dest.display());

        if !dry_run {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dest)
                .with_context(|| format!("Failed to restore {}", dest.display()))?;
        }
    }

    if !dry_run {
        println!("✓ Restore complete.");
    }

    Ok(())
}

pub fn snap_path_for(cfg: &Config, id: &str, stored: &str) -> PathBuf {
    cfg.snapshot_dir().join(id).join(stored)
}
