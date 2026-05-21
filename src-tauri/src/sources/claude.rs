use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc};
use serde_json::Value;

use super::{SourceHealth, SourceId, UsageSnapshot, UsageSource, UsageWindow};
use crate::settings;

const SESSION_WINDOW_MINUTES: u32 = 300;
const WEEKLY_WINDOW_MINUTES: u32 = 10080;
const SESSION_LENGTH_HOURS: i64 = 5;
const WEEKLY_LENGTH_DAYS: i64 = 7;

#[derive(Default)]
pub struct ClaudeSource;

impl UsageSource for ClaudeSource {
    fn id(&self) -> SourceId {
        SourceId::Claude
    }

    fn poll(&mut self) -> UsageSnapshot {
        let now = Utc::now();

        let Some(projects_dir) = claude_projects_dir() else {
            return empty(now, SourceHealth::PathNotFound);
        };
        if !projects_dir.exists() {
            return empty(now, SourceHealth::PathNotFound);
        }

        let app_settings = settings::load();
        let tier = app_settings.claude_tier;
        let (session_cap, weekly_cap) = app_settings.effective_caps();

        let weekly_start = now - Duration::days(WEEKLY_LENGTH_DAYS);

        // Collect ALL prompt timestamps so we can group them into 5h sessions
        // the way Anthropic does (session starts at first prompt, ends 5h later,
        // next prompt after that starts a new session). Rolling-5h was producing
        // misleadingly low counts after a gap because it dropped older prompts
        // that were still within Anthropic's current session window.
        let mut all_prompts: Vec<DateTime<Utc>> = Vec::new();
        let mut weekly_count: u32 = 0;
        let mut files_scanned: usize = 0;
        let mut read_errors: usize = 0;

        for jsonl in find_jsonls(&projects_dir) {
            files_scanned += 1;
            let content = match std::fs::read_to_string(&jsonl) {
                Ok(c) => c,
                Err(_) => {
                    read_errors += 1;
                    continue;
                }
            };
            for line in content.lines() {
                if !line.contains("\"type\":\"user\"") {
                    continue;
                }
                let val: Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if !is_real_user_prompt(&val) {
                    continue;
                }
                let Some(ts_str) = val.get("timestamp").and_then(|v| v.as_str()) else {
                    continue;
                };
                let ts = match DateTime::parse_from_rfc3339(ts_str) {
                    Ok(t) => t.with_timezone(&Utc),
                    Err(_) => continue,
                };
                if ts >= weekly_start {
                    weekly_count += 1;
                }
                all_prompts.push(ts);
            }
        }

        if files_scanned == 0 {
            return empty(now, SourceHealth::NoData);
        }

        all_prompts.sort();

        // Walk prompts and group into 5h sessions: a new session starts each
        // time a prompt is >= 5h after the current session's first prompt.
        let session_duration = Duration::hours(SESSION_LENGTH_HOURS);
        let mut session_start: Option<DateTime<Utc>> = None;
        let mut session_count: u32 = 0;
        for ts in &all_prompts {
            let belongs_to_current =
                session_start.is_some_and(|s| *ts < s + session_duration);
            if belongs_to_current {
                session_count += 1;
            } else {
                session_start = Some(*ts);
                session_count = 1;
            }
        }

        // Is the latest session still active?
        let (session_count, session_resets_at) = match session_start {
            Some(start) if now < start + session_duration => {
                (session_count, start + session_duration)
            }
            _ => (0, now + session_duration),
        };

        let session = UsageWindow {
            used_percent: pct(session_count, session_cap),
            current_count: Some(session_count),
            resets_at: session_resets_at,
            window_minutes: SESSION_WINDOW_MINUTES,
            is_estimated: true,
        };

        let weekly = UsageWindow {
            used_percent: pct(weekly_count, weekly_cap),
            current_count: Some(weekly_count),
            resets_at: next_monday_utc(now),
            window_minutes: WEEKLY_WINDOW_MINUTES,
            is_estimated: true,
        };

        let source_health = if read_errors > 0 && session_count == 0 && weekly_count == 0 {
            SourceHealth::FileLocked
        } else {
            SourceHealth::Ok
        };

        UsageSnapshot {
            source: SourceId::Claude,
            session: Some(session),
            weekly: Some(weekly),
            plan_type: Some(tier.label().to_string()),
            source_health,
            fetched_at: now,
            data_updated_at: Some(now), // Claude is always live (recomputed each poll)
        }
    }
}

fn empty(now: DateTime<Utc>, health: SourceHealth) -> UsageSnapshot {
    UsageSnapshot {
        source: SourceId::Claude,
        session: None,
        weekly: None,
        plan_type: None,
        source_health: health,
        fetched_at: now,
        data_updated_at: Some(now),
    }
}

fn pct(used: u32, cap: u32) -> f32 {
    if cap == 0 {
        return 0.0;
    }
    ((used as f32 / cap as f32) * 100.0).min(100.0)
}

/// A "real user prompt" is what we want to count for Anthropic's session/weekly
/// quota. Excludes tool_result messages, sub-agent sidechain turns, and any
/// non-external user types — those are downstream effects of one human prompt,
/// not separate prompts.
fn is_real_user_prompt(val: &Value) -> bool {
    let Some(obj) = val.as_object() else {
        return false;
    };
    if obj.get("type").and_then(|v| v.as_str()) != Some("user") {
        return false;
    }
    if obj.get("userType").and_then(|v| v.as_str()) != Some("external") {
        return false;
    }
    if obj.get("isSidechain").and_then(|v| v.as_bool()) == Some(true) {
        return false;
    }
    let Some(content) = obj.get("message").and_then(|m| m.get("content")) else {
        return false;
    };
    match content {
        Value::String(_) => true,
        Value::Array(arr) => !arr.iter().any(|item| {
            item.get("type").and_then(|v| v.as_str()) == Some("tool_result")
        }),
        _ => false,
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

fn claude_projects_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".claude").join("projects"))
}

fn find_jsonls(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk(&path, out);
        } else if ft.is_file() {
            if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case("jsonl"))
            {
                out.push(path);
            }
        }
    }
}

fn next_monday_utc(now: DateTime<Utc>) -> DateTime<Utc> {
    let weekday_from_monday = now.weekday().num_days_from_monday() as i64;
    let days_until_next_monday = if weekday_from_monday == 0 {
        7
    } else {
        7 - weekday_from_monday
    };
    let target_date = now.date_naive() + Duration::days(days_until_next_monday);
    target_date
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_utc()
}
