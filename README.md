# qbdotsnap

Snapshot and diff your dotfiles. Built for Arch + Hyprland.

## Install

```bash
git clone https://github.com/riz3e/qbdotsnap.git
cd qbdotsnap
cargo install --path .
```

Make sure `~/.cargo/bin` is in your PATH:

```zsh
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.zshrc
```

## First run

```bash
qbdotsnap take
```

On first run it creates `~/.qbdotsnap.toml` with sensible defaults for an Arch/Hyprland setup. Edit it to match your actual paths, then take another snapshot.

## Edit the config to add your paths

```bash
vim ~/.qbdotsnap.toml  
```

## Usage

```bash
# Take a snapshot
qbdotsnap take
qbdotsnap take --label "before hyprland update"

# List snapshots
qbdotsnap list

# Diff — use index, timestamp prefix, 'last', or 'prev'
qbdotsnap diff                            # last two snapshots
qbdotsnap diff 9 12                       # by index
qbdotsnap diff prev last --file ~/.config/hypr/hyprland.conf

# Restore
qbdotsnap restore --dry-run 11            # dry run first
qbdotsnap restore 11

# Delete (keeps index gaps — no renumbering)
qbdotsnap delete 9                        # prompts for confirmation
qbdotsnap delete 9 --force

# Export a summary of your setup
qbdotsnap export
qbdotsnap export 11

# Watch for changes and auto-snapshot
qbdotsnap watch
qbdotsnap watch --debounce 600 # seconds after last change

# Toggle desktop notifications
qbdotsnap notifications on
qbdotsnap notifications off
```

## Notifications

Uses `notify-send` — no extra dependencies. Shows one notification per event:

- regular snapshot: `qbdotsnap #12 — snapshot taken • 34 files`
- git push (when configured): `qbdotsnap #12 — pushed to git`

Toggle anytime with `qbdotsnap notifications on/off` — updates `~/.qbdotsnap.toml` directly.

## Auto-snapshot with systemd

```bash
mkdir -p ~/.config/systemd/user
cp systemd/qbdotsnap.service ~/.config/systemd/user/
systemctl --user enable --now qbdotsnap

# View logs
journalctl --user -u qbdotsnap -f
```

## Config (`~/.qbdotsnap.toml`)

```toml
notifications = true   # toggle with `qbdotsnap notifications on/off`

track = [
  "~/.zshrc",
  "~/.gitconfig",
  "~/.config/hypr",
  "~/.config/quickshell",
  "~/.config/ags",
  "~/.config/nvim",
]

skip_patterns = [
  "*.log",
  "*.sock",
  "hyprland.log",
]

# Optional git integration [NOT WORKING]
# [git]
# remote = "git@github.com:yourname/dotfiles.git"
# branch = "main"
# auto_push = false
```

## Snapshot layout

```bash
~/.qbdotsnap/
  .counter                     ← global index counter
  2026-04-22T17:27:54/
    manifest.json              ← index, timestamp, label, file list
    home/qrob/.zshrc
    home/qrob/.config/hypr/hyprland.conf
    home/qrob/.config/quickshell/shell.qml
```

## Uninstall

```bash
systemctl --user disable --now qbdotsnap
cargo uninstall qbdotsnap
```
