# qbdotsnap

Snapshot and diff your dotfiles. Built for Arch/Hyprland users who break things.

mostly focused on Hyprland.

## Install

```bash
git clone https://github.com/riz3e/qbdotsnap.git
cd qbdotsnap
cargo install --path .
```

This puts the `qbdotsnap` binary in `~/.cargo/bin/`. Make sure that's in your PATH — add this to your `~/.zshrc` if it isn't already:

```zsh
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload: `source ~/.zshrc`

## First run

```bash
qbdotsnap take
```

On first run it creates `~/.qbdotsnap.toml` with sensible defaults for an Arch/Hyprland setup. Edit it to match your actual paths, then take another snapshot.

## Edit the config to add your paths

vim ~/.qbdotsnap.toml  

## Usage

```bash
# Take a snapshot (optionally with a label)
qbdotsnap take
qbdotsnap take --label "before hyprland update"

# List all snapshots
qbdotsnap list

# Diff the last two snapshots
qbdotsnap diff

# Diff specific snapshots by timestamp
qbdotsnap diff 2025-04-20T10:00:00 2025-04-21T14:32:00

# Export a human-readable summary of your setup
qbdotsnap export

# Restore — always dry-run first!
qbdotsnap restore --dry-run 2025-04-20T10:00:00
qbdotsnap restore 2025-04-20T10:00:00
```

## Config (~/.qbdotsnap.toml)

```toml
track = [
  "~/.zshrc",
  "~/.gitconfig",
  "~/.config/hypr",         # whole directory, tracked recursively
  "~/.config/quickshell",   # quickshell bar
  "~/.config/ags",          # end4/dots-hyprland AGS widgets
  "~/.config/nvim",
]

skip_patterns = [
  "*.log",
  "*.sock",
  "hyprland.log",
]
```

## Uninstall

```bash
cargo uninstall qbdotsnap
```

## Snapshot layout

```zsh
~/.qbdotsnap/
  2025-04-21T14:32:00/
    manifest.json
    home/user/.zshrc
    home/user/.gitconfig
    home/user/.config/hypr/hyprland.conf
    home/user/.config/hypr/binds.conf
    home/user/.config/quickshell/shell.qml
```
