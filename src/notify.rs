use crate::config::Config;
use std::process::Command;

pub enum Event<'a> {
    SnapshotTaken { index: u32, id: &'a str, file_count: usize },
    SnapshotPushed { index: u32, id: &'a str },
}

pub fn send(cfg: &Config, event: Event) {
    if !cfg.notifications {
        return;
    }

    let (summary, body) = match event {
        Event::SnapshotTaken { index, id, file_count } => (
            format!("qbdotsnap #{}", index),
            format!("snapshot taken • {} files\n{}", file_count, id),
        ),
        Event::SnapshotPushed { index, id } => (
            format!("qbdotsnap #{}", index),
            format!("pushed to git\n{}", id),
        ),
    };

    let _ = Command::new("notify-send")
        .args(["--app-name=qbdotsnap", "--urgency=low", &summary, &body])
        .status();
}