use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{SourceHealth, SourceId, UsageSnapshot, UsageSource, UsageWindow};
use crate::settings;

const SESSION_WINDOW_MINUTES: u32 = 300;
const WEEKLY_WINDOW_MINUTES: u32 = 10080;

#[derive(Default)]
pub struct CodexSource;

impl UsageSource for CodexSource {
    fn id(&self) -> SourceId {
        SourceId::Codex
    }

    fn poll(&mut self) -> UsageSnapshot {
        let now = Utc::now();

        let Some(sessions_dir) = codex_sessions_dir() else {
            return empty(now, SourceHealth::PathNotFound, None);
        };
        if !sessions_dir.exists() {
            return empty(now, SourceHealth::PathNotFound, None);
        }

        let (newest_path, newest_modified) = match find_newest_rollout(&sessions_dir) {
            Some(pair) => pair,
            None => return empty(now, SourceHealth::NoData, None),
        };

        let data_updated_at = Some(DateTime::<Utc>::from(newest_modified));

        let rate_limits = match read_latest_rate_limits(&newest_path) {
            Ok(Some(v)) => v,
            Ok(None) => return empty(now, SourceHealth::NoData, data_updated_at),
            Err(err) => {
                let health = match err.kind() {
                    std::io::ErrorKind::PermissionDenied => SourceHealth::PermissionDenied,
                    _ => SourceHealth::FileLocked,
                };
                return empty(now, health, data_updated_at);
            }
        };

        let mut snapshot = map_rate_limits_to_snapshot(&rate_limits, now);
        snapshot.data_updated_at = data_updated_at;
        apply_manual_override(&mut snapshot);
        snapshot
    }
}

/// Replace the live-derived percent with the user's manual entry when set.
/// Used when CLI data is stale because the user used the web/IDE.
fn apply_manual_override(snapshot: &mut UsageSnapshot) {
    let s = settings::load();
    if let (Some(pct), Some(window)) = (s.codex_session_pct_override, snapshot.session.as_mut()) {
        window.used_percent = pct as f32;
    }
    if let (Some(pct), Some(window)) = (s.codex_weekly_pct_override, snapshot.weekly.as_mut()) {
        window.used_percent = pct as f32;
    }
}

fn empty(
    now: DateTime<Utc>,
    health: SourceHealth,
    data_updated_at: Option<DateTime<Utc>>,
) -> UsageSnapshot {
    UsageSnapshot {
        source: SourceId::Codex,
        session: None,
        weekly: None,
        plan_type: None,
        source_health: health,
        fetched_at: now,
        data_updated_at,
    }
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

fn codex_sessions_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".codex").join("sessions"))
}

fn find_newest_rollout(root: &Path) -> Option<(PathBuf, SystemTime)> {
    let mut newest: Option<(PathBuf, SystemTime)> = None;
    walk(root, &mut newest);
    newest
}

fn walk(dir: &Path, newest: &mut Option<(PathBuf, SystemTime)>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk(&path, newest);
        } else if ft.is_file() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !(name.starts_with("rollout-") && name.ends_with(".jsonl")) {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            let Ok(modified) = meta.modified() else { continue };
            if newest.as_ref().is_none_or(|(_, t)| modified > *t) {
                *newest = Some((path, modified));
            }
        }
    }
}

fn read_latest_rate_limits(file: &Path) -> std::io::Result<Option<Value>> {
    let content = std::fs::read_to_string(file)?;
    let mut latest: Option<Value> = None;
    for line in content.lines() {
        if !line.contains("rate_limits") {
            continue;
        }
        let val: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(rl) = find_rate_limits_in(&val) {
            latest = Some(rl.clone());
        }
    }
    Ok(latest)
}

fn find_rate_limits_in(val: &Value) -> Option<&Value> {
    if let Value::Object(obj) = val {
        if let Some(rl) = obj.get("rate_limits") {
            if rl.is_object() {
                return Some(rl);
            }
        }
        for (_, v) in obj {
            if let Some(rl) = find_rate_limits_in(v) {
                return Some(rl);
            }
        }
    }
    None
}

fn map_rate_limits_to_snapshot(rl: &Value, now: DateTime<Utc>) -> UsageSnapshot {
    let mut session: Option<UsageWindow> = None;
    let mut weekly: Option<UsageWindow> = None;
    let plan_type = rl
        .get("plan_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Don't assume primary=session: free plan has primary=weekly only.
    // Read window_minutes to decide which slot each entry maps to.
    for slot_name in ["primary", "secondary"] {
        let Some(slot) = rl.get(slot_name) else { continue };
        if !slot.is_object() {
            continue;
        }
        let used_percent_raw = slot
            .get("used_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let window_minutes = slot
            .get("window_minutes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let resets_at_unix = slot.get("resets_at").and_then(|v| v.as_i64()).unwrap_or(0);
        let resets_at = DateTime::from_timestamp(resets_at_unix, 0).unwrap_or(now);

        // If the reset time has already passed, the window already rolled over.
        // The CLI's last-known % is no longer meaningful — show 0% honestly
        // rather than pretending the stale value is current.
        let used_percent = if resets_at <= now { 0.0 } else { used_percent_raw };

        let window = UsageWindow {
            used_percent,
            current_count: None,
            resets_at,
            window_minutes,
            is_estimated: false,
        };

        match window_minutes {
            SESSION_WINDOW_MINUTES => session = Some(window),
            WEEKLY_WINDOW_MINUTES => weekly = Some(window),
            _ => {} // unknown window, ignore
        }
    }

    UsageSnapshot {
        source: SourceId::Codex,
        session,
        weekly,
        plan_type,
        source_health: SourceHealth::Ok,
        fetched_at: now,
        data_updated_at: None, // set by caller from rollout file mtime
    }
}
