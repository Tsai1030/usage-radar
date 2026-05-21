use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use chrono::Utc;
use notify::{EventKind, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::settings::{self, NOTIFY_THRESHOLD_PCT};
use crate::sources::{
    claude::ClaudeSource, codex::CodexSource, SourceId, UsageSnapshot, UsageSource, UsageWindow,
};

/// Hard fallback so we still poll even if every file watcher silently fails
/// (e.g. permissions, watch-limit on Linux). Most updates land via the watcher
/// within 1-2 s of the underlying log write.
const POLL_INTERVAL_SECS: u64 = 30;

/// After a watcher event arrives, wait this long collecting more events so a
/// burst of fs writes (which is what JSONL appending looks like) results in
/// one snapshot recompute, not dozens.
const WATCHER_DEBOUNCE_MS: u64 = 800;

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut codex = CodexSource::default();
        let mut claude = ClaudeSource::default();
        let mut thresholds = ThresholdTracker::default();

        // Wire up filesystem watchers. They send () to `rx` whenever a Claude
        // or Codex log file changes; if they fail to set up we silently fall
        // back to interval polling only.
        let (tx, rx) = mpsc::channel::<()>();
        let _watchers = setup_watchers(tx);

        // Initial tick so the UI doesn't wait for the first interval/event.
        emit_all(&app, &mut codex, &mut claude, &mut thresholds);

        loop {
            // Block until either a watcher event arrives OR the fallback
            // interval elapses.
            match rx.recv_timeout(Duration::from_secs(POLL_INTERVAL_SECS)) {
                Ok(_) => {
                    // Drain remaining events fired during a burst of appends.
                    std::thread::sleep(Duration::from_millis(WATCHER_DEBOUNCE_MS));
                    while rx.try_recv().is_ok() {}
                }
                Err(_) => {
                    // Timeout — just do a regular poll.
                }
            }
            emit_all(&app, &mut codex, &mut claude, &mut thresholds);
        }
    });
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

fn watch_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(h) = home_dir() {
        v.push(h.join(".claude").join("projects"));
        v.push(h.join(".codex").join("sessions"));
    }
    v
}

/// Register a recursive watcher per known log directory. Returns the watcher
/// objects so the caller can keep them alive (dropping a watcher stops it).
fn setup_watchers(tx: mpsc::Sender<()>) -> Vec<notify::RecommendedWatcher> {
    let mut out = Vec::new();
    for path in watch_paths() {
        if !path.exists() {
            continue;
        }
        let tx_inner = tx.clone();
        let watcher_result = notify::recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    ) {
                        let _ = tx_inner.send(());
                    }
                }
            },
        );
        let Ok(mut w) = watcher_result else { continue };
        if w.watch(&path, RecursiveMode::Recursive).is_ok() {
            out.push(w);
        }
    }
    out
}

fn emit_all(
    app: &AppHandle,
    codex: &mut CodexSource,
    claude: &mut ClaudeSource,
    thresholds: &mut ThresholdTracker,
) {
    let s1 = codex.poll();
    let s2 = claude.poll();
    thresholds.check(app, &s1);
    thresholds.check(app, &s2);
    let _ = app.emit("usage-update", s1);
    let _ = app.emit("usage-update", s2);
}

/// Remembers the last seen used_percent per (source, window) so we can fire a
/// notification exactly once when a bar crosses the threshold from below.
/// Resets implicitly when the value drops back down (e.g. session window
/// rolled over) — next crossing will re-arm and notify again.
#[derive(Default)]
struct ThresholdTracker {
    codex_session: Option<f32>,
    codex_weekly: Option<f32>,
    claude_session: Option<f32>,
    claude_weekly: Option<f32>,
}

impl ThresholdTracker {
    fn check(&mut self, app: &AppHandle, snapshot: &UsageSnapshot) {
        let source_label = match snapshot.source {
            SourceId::Codex => "Codex",
            SourceId::Claude => "Claude",
        };
        let (sess_slot, wk_slot) = match snapshot.source {
            SourceId::Codex => (&mut self.codex_session, &mut self.codex_weekly),
            SourceId::Claude => (&mut self.claude_session, &mut self.claude_weekly),
        };

        evaluate(app, source_label, "session", &snapshot.session, sess_slot);
        evaluate(app, source_label, "weekly", &snapshot.weekly, wk_slot);
    }
}

fn evaluate(
    app: &AppHandle,
    source_label: &str,
    window_label: &str,
    window: &Option<UsageWindow>,
    last: &mut Option<f32>,
) {
    let Some(w) = window else {
        *last = None;
        return;
    };
    let prev = *last;
    *last = Some(w.used_percent);

    if !settings::load().notify_at_threshold {
        return;
    }

    let crossed = prev
        .map(|p| p < NOTIFY_THRESHOLD_PCT && w.used_percent >= NOTIFY_THRESHOLD_PCT)
        .unwrap_or(false);
    if !crossed {
        return;
    }

    let body = format!(
        "{source_label} {window_label} at {pct:.0}% — resets {when}",
        pct = w.used_percent,
        when = format_until(w.resets_at),
    );
    let _ = app
        .notification()
        .builder()
        .title("Usage Radar")
        .body(body)
        .show();
}

fn format_until(target: chrono::DateTime<Utc>) -> String {
    let diff = target - Utc::now();
    let secs = diff.num_seconds();
    if secs <= 0 {
        return "now".into();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("in {}m", mins);
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("in {}h {}m", hours, mins % 60);
    }
    let days = hours / 24;
    format!("in {}d {}h", days, hours % 24)
}
