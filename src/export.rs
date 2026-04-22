use crate::config::Config;
use crate::snapshot::load_manifest_by_selector;
use anyhow::Result;
use std::collections::HashMap;

pub fn run(cfg: &Config, id: &str) -> Result<()> {
    let manifest = load_manifest_by_selector(cfg, id)?;
    let snap_dir = cfg.snapshot_dir();

    println!("╔══════════════════════════════════════════════════╗");
    println!("║           machine DNA — qbdotsnap export           ║");
    println!("╠══════════════════════════════════════════════════╣");
    println!("║  snapshot : {:<37}║", manifest.id);
    println!(
        "║  label    : {:<37}║",
        manifest.label.as_deref().unwrap_or("(none)")
    );
    println!("║  files    : {:<37}║", manifest.files.len());
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    // Group files by top-level directory
    let mut groups: HashMap<String, Vec<&crate::snapshot::ManifestEntry>> = HashMap::new();

    for entry in &manifest.files {
        let parts: Vec<&str> = entry.stored.splitn(4, '/').collect();
        // stored paths look like "home/user/.config/hypr/hyprland.conf"
        // group by the config category (e.g. ".config/hypr", ".zshrc")
        let group_key = if parts.len() >= 3 && parts[2].starts_with('.') {
            if parts.len() >= 4 && parts[2] == ".config" {
                format!("~/.config/{}", parts[3].split('/').next().unwrap_or(""))
            } else {
                format!("~/{}", parts[2])
            }
        } else {
            entry.stored.clone()
        };
        groups.entry(group_key).or_default().push(entry);
    }

    let mut sorted_groups: Vec<(&String, &Vec<&crate::snapshot::ManifestEntry>)> =
        groups.iter().collect();
    sorted_groups.sort_by_key(|(k, _)| k.as_str());

    for (group, entries) in &sorted_groups {
        println!("  \x1b[1m{}\x1b[0m", group);

        for entry in entries.iter().take(8) {
            let path = snap_dir.join(&manifest.id).join(&entry.stored);
            let size = std::fs::metadata(&path)
                .map(|m| format_size(m.len()))
                .unwrap_or_else(|_| "?".into());

            let lines = std::fs::read_to_string(&path)
                .map(|c| format!("{} lines", c.lines().count()))
                .unwrap_or_default();

            println!("    {} ({}, {})", entry.source, size, lines);
        }

        if entries.len() > 8 {
            println!("    … and {} more files", entries.len() - 8);
        }

        println!();
    }

    // Print a short preview of key files
    let preview_targets = ["/.zshrc", "/.bashrc", "/.gitconfig"];
    let mut previewed = false;

    for entry in &manifest.files {
        if preview_targets.iter().any(|t| entry.source.ends_with(t)) {
            let path = snap_dir.join(&manifest.id).join(&entry.stored);
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let preview: Vec<&str> = content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
                .take(8)
                .collect();

            if !preview.is_empty() {
                if !previewed {
                    println!("── key file previews ──────────────────────────────");
                    previewed = true;
                }
                println!("\n  \x1b[1m{}\x1b[0m", entry.source);
                for line in preview {
                    println!("    {}", line);
                }
            }
        }
    }

    if previewed {
        println!();
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    match bytes {
        b if b < 1024 => format!("{} B", b),
        b if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
    }
}
