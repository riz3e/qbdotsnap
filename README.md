# qbdotsnap

Snapshot and diff your dotfiles. Built for Arch/Hyprland users who break things.

mostly focused for Hyprland.

## Install

```bash
git clone https://github.com/riz3e/qbdotsnap.git
cd qbdotsnap
cargo install --path .
```

## Quick start

```bash
# First run creates ~/.qbdotsnap.toml with sensible defaults
qbdotsnap take

# Edit the config to add your paths
nano ~/.qbdotsnap.toml

# Take another snapshot with a label
qbdotsnap take --label "before hyprland update"

# List all snapshots
qbdotsnap list

# Diff the last two snapshots
qbdotsnap diff

# Diff specific snapshots
qbdotsnap diff 2025-04-20T10:00:00 2025-04-21T14:32:00

# Export a summary of the latest snapshot
qbdotsnap export

# Restore from a snapshot (dry run first!)
qbdotsnap restore --dry-run 2025-04-20T10:00:00
qbdotsnap restore 2025-04-20T10:00:00
```

## Config (`~/.qbdotsnap.toml`)

```toml
track = [
  "~/.zshrc",
  "~/.gitconfig",
  "~/.config/hypr",
  "~/.config/nvim",
]

skip_patterns = [
  "*.log",
  "*.sock",
  "hyprland.log",
]
```

## Snapshot layout

```
~/.qbdotsnap/
  2025-04-21T14:32:00/
    manifest.json
    home/user/.zshrc
    home/user/.gitconfig
    home/user/.config/hypr/hyprland.conf
    home/user/.config/hypr/binds.conf
```
