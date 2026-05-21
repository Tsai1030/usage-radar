use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::sources::{claude::ClaudeSource, codex::CodexSource, UsageSource};

const POLL_INTERVAL_SECS: u64 = 30;

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut codex = CodexSource::default();
        let mut claude = ClaudeSource::default();

        // Emit one tick immediately so the UI doesn't have to wait the first interval.
        emit_all(&app, &mut codex, &mut claude);

        loop {
            std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
            emit_all(&app, &mut codex, &mut claude);
        }
    });
}

fn emit_all(app: &AppHandle, codex: &mut CodexSource, claude: &mut ClaudeSource) {
    let _ = app.emit("usage-update", codex.poll());
    let _ = app.emit("usage-update", claude.poll());
}
