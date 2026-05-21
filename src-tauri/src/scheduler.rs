use std::time::Duration;

use chrono::Utc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::settings::{self, NOTIFY_THRESHOLD_PCT};
use crate::sources::{
    claude::ClaudeSource, codex::CodexSource, SourceId, UsageSnapshot, UsageSource, UsageWindow,
};

const POLL_INTERVAL_SECS: u64 = 30;

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut codex = CodexSource::default();
        let mut claude = ClaudeSource::default();
        let mut thresholds = ThresholdTracker::default();

        // Initial tick so the UI doesn't wait one interval to populate.
        emit_all(&app, &mut codex, &mut claude, &mut thresholds);

        loop {
            std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
            emit_all(&app, &mut codex, &mut claude, &mut thresholds);
        }
    });
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
