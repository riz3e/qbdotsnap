use crate::config::{expand_tilde, Config};
use crate::snapshot::{load_manifest_by_id, snap_path_for};
use anyhow::Result;
use similar::{ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};

pub fn run(cfg: &Config, from_id: &str, to_id: &str, file_filter: Option<&str>) -> Result<()> {
    let from = load_manifest_by_id(cfg, from_id)?;
    let to = load_manifest_by_id(cfg, to_id)?;

    println!("Diff: {} → {}", from.id, to.id);

    // If --file given, resolve it to an absolute path for matching
    let filter_abs: Option<String> = file_filter.map(|f| {
        expand_tilde(f).to_string_lossy().into_owned()
    });

    // Build maps: stored_path -> ManifestEntry
    let from_map: HashMap<&str, &crate::snapshot::ManifestEntry> =
        from.files.iter().map(|e| (e.stored.as_str(), e)).collect();
    let to_map: HashMap<&str, &crate::snapshot::ManifestEntry> =
        to.files.iter().map(|e| (e.stored.as_str(), e)).collect();

    let from_keys: HashSet<&str> = from_map.keys().copied().collect();
    let to_keys: HashSet<&str> = to_map.keys().copied().collect();

    // Helper: should we include this entry given the filter?
    let matches_filter = |entry: &crate::snapshot::ManifestEntry| -> bool {
        match &filter_abs {
            None => true,
            Some(f) => entry.source.contains(f.as_str()) || f.contains(entry.source.as_str()),
        }
    };

    if filter_abs.is_some() {
        println!("Filter: {}\n", filter_abs.as_deref().unwrap());
    } else {
        println!();
    }

    // New files
    let mut new_files: Vec<&&str> = to_keys.difference(&from_keys).collect();
    new_files.sort();
    for key in &new_files {
        let entry = to_map[**key];
        if matches_filter(entry) {
            println!("\x1b[32m+ {}\x1b[0m  (new file)", key);
        }
    }

    // Deleted files
    let mut deleted_files: Vec<&&str> = from_keys.difference(&to_keys).collect();
    deleted_files.sort();
    for key in &deleted_files {
        let entry = from_map[**key];
        if matches_filter(entry) {
            println!("\x1b[31m- {}\x1b[0m  (deleted)", key);
        }
    }

    // Changed files
    let mut changed_count: i32 = 0;
    let mut unchanged_count: i32 = 0;
    let mut filtered_out: i32 = 0;

    let mut common_keys: Vec<&&str> = from_keys.intersection(&to_keys).collect();
    common_keys.sort();

    for key in common_keys {
        let entry = from_map[*key];
        if !matches_filter(entry) {
            filtered_out += 1;
            continue;
        }

        let from_path = snap_path_for(cfg, &from.id, key);
        let to_path = snap_path_for(cfg, &to.id, key);

        let from_content = std::fs::read_to_string(&from_path).unwrap_or_default();
        let to_content = std::fs::read_to_string(&to_path).unwrap_or_default();

        if from_content == to_content {
            unchanged_count += 1;
            continue;
        }

        changed_count += 1;
        println!("\n\x1b[1m{}\x1b[0m", key);
        println!("{}", "-".repeat(50));

        let diff = TextDiff::from_lines(&from_content, &to_content);

        for group in diff.grouped_ops(3) {
            for op in &group {
                for change in diff.iter_changes(op) {
                    let (sign, color) = match change.tag() {
                        ChangeTag::Delete => ("-", "\x1b[31m"),
                        ChangeTag::Insert => ("+", "\x1b[32m"),
                        ChangeTag::Equal  => (" ", "\x1b[0m"),
                    };
                    let lineno = match change.tag() {
                        ChangeTag::Delete => change.old_index().map(|n| n + 1),
                        ChangeTag::Insert => change.new_index().map(|n| n + 1),
                        ChangeTag::Equal  => change.old_index().map(|n| n + 1),
                    };
                    let lineno_str = lineno
                        .map(|n| format!("{:<5}", n))
                        .unwrap_or_else(|| "     ".to_string());
                    let line = change.value().trim_end_matches('\n');
                    println!("{}{}  {}  {}\x1b[0m", color, sign, lineno_str, line);
                }
            }
            println!("\x1b[90m     ···\x1b[0m");
        }
    }

    // Summary
    println!();
    if filter_abs.is_some() && filtered_out > 0 {
        println!("({} files hidden by --file filter)", filtered_out);
    }
    println!(
        "Summary: {} changed, {} new, {} deleted, {} unchanged",
        changed_count,
        new_files.len(),
        deleted_files.len(),
        unchanged_count
    );

    Ok(())
}