# qbdotsnap

Snapshot and diff your dotfiles. Built for Arch/Hyprland users who break things.

mostly focused on Hyprland.

## Install

```bash
git clone https://github.com/riz3e/qbdotsnap.git
cd qbdotsnap
cargo install --path .
```

Make sure `~/.cargo/bin` is in your PATH:

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

```bash
vim ~/.qbdotsnap.toml  
```

## Usage

```bash
# Take a snapshot (optionally with a label)
qbdotsnap take
qbdotsnap take --label "before hyprland update"

# List snapshots
qbdotsnap list

# Diff last two snapshots
qbdotsnap diff

# Diff and filter to one file
qbdotsnap diff --file ~/.config/hypr/hyprland.conf

# Diff specific snapshots
qbdotsnap diff 2025-04-20T10:00:00 2025-04-21T14:32:00 --file ~/.zshrc

# Export a summary of your setup
qbdotsnap export

# Restore (dry run first!)
qbdotsnap restore --dry-run 2025-04-20T10:00:00
qbdotsnap restore 2025-04-20T10:00:00

# Watch for changes and auto-snapshot
qbdotsnap watch
qbdotsnap watch --debounce 600   # wait 10min after last change
```

## Auto-snapshot with systemd

```bash
# Install the service
mkdir -p ~/.config/systemd/user
cp systemd/qbdotsnap.service ~/.config/systemd/user/

# Enable and start
systemctl --user enable --now qbdotsnap

# Check logs
journalctl --user -u qbdotsnap -f
```

## Config (~/.qbdotsnap.toml)

```toml
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
```

## Uninstall

```bash
systemctl --user disable --now qbdotsnap  # if using systemd
cargo uninstall qbdotsnap
```
