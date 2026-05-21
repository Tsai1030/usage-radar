use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc};
use serde_json::Value;

use super::{SourceHealth, SourceId, UsageSnapshot, UsageSource, UsageWindow};
use crate::settings;

const SESSION_WINDOW_MINUTES: u32 = 300;
const WEEKLY_WINDOW_MINUTES: u32 = 10080;
const SESSION_LENGTH_HOURS: i64 = 5;
const WEEKLY_LENGTH_DAYS: i64 = 7;
const CACHE_PRUNE_DAYS: i64 = 8;

#[derive(Default)]
pub struct ClaudeSource {
    /// Per-file byte offset we've already parsed up to. New polls only read
    /// from this offset onward, so a 50 MB log stays cheap to poll.
    file_offsets: HashMap<PathBuf, u64>,
    /// Cached prompt timestamps gathered across polls, kept sorted ascending
    /// and pruned to the last `CACHE_PRUNE_DAYS` so memory stays bounded.
    prompt_cache: Vec<DateTime<Utc>>,
}

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

        // Anthropic's weekly is calendar-aligned (Monday 00:00 UTC), not a
        // 7d rolling window — match that so calibration stays stable instead
        // of drifting as old prompts roll out of the window.
        let weekly_start = current_week_start_utc(now);
        let weekly_end = weekly_start + Duration::days(WEEKLY_LENGTH_DAYS);

        let was_cold = self.prompt_cache.is_empty();
        let cache_cutoff = now - Duration::days(CACHE_PRUNE_DAYS);

        let files = find_jsonls(&projects_dir);
        if files.is_empty() && self.prompt_cache.is_empty() {
            return empty(now, SourceHealth::NoData);
        }

        // Detect rotation: if any tracked file shrank below its known offset,
        // we can't reliably trust our offset map. Easiest safe behaviour: clear
        // both the cache and offsets, re-read everything from scratch.
        let mut rotation = false;
        for path in &files {
            if let Some(off) = self.file_offsets.get(path) {
                let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                if size < *off {
                    rotation = true;
                    break;
                }
            }
        }
        if rotation {
            self.file_offsets.clear();
            self.prompt_cache.clear();
        }

        // Forget offsets for files that have disappeared since last poll
        let current_set: HashSet<&PathBuf> = files.iter().collect();
        self.file_offsets.retain(|p, _| current_set.contains(p));

        let mut user_lines_seen_this_poll: usize = 0;
        let mut files_scanned: usize = 0;
        let mut read_errors: usize = 0;

        for path in &files {
            files_scanned += 1;
            let size = match std::fs::metadata(path) {
                Ok(m) => m.len(),
                Err(_) => {
                    read_errors += 1;
                    continue;
                }
            };
            let prev = *self.file_offsets.get(path).unwrap_or(&0);
            if size == prev {
                continue; // nothing new since last poll
            }
            let new_offset = read_new_prompts_from(
                path,
                prev,
                &mut self.prompt_cache,
                &mut user_lines_seen_this_poll,
            );
            match new_offset {
                Ok(advanced_to) => {
                    self.file_offsets.insert(path.clone(), advanced_to);
                }
                Err(_) => {
                    read_errors += 1;
                }
            }
        }

        // Schema drift check is only meaningful when we have a cold start: we
        // saw a lot of candidate lines but extracted zero prompts.
        if was_cold && user_lines_seen_this_poll >= 50 && self.prompt_cache.is_empty() {
            return empty(now, SourceHealth::SchemaMismatch);
        }

        if files_scanned == 0 && self.prompt_cache.is_empty() {
            return empty(now, SourceHealth::NoData);
        }

        // Prune + normalise the cache
        self.prompt_cache.retain(|t| *t >= cache_cutoff);
        self.prompt_cache.sort();
        self.prompt_cache.dedup();

        // Sessionise: walk forward, a new session starts when a prompt is
        // >= 5h after the current session's first prompt.
        let session_duration = Duration::hours(SESSION_LENGTH_HOURS);
        let mut session_start: Option<DateTime<Utc>> = None;
        let mut session_count: u32 = 0;
        for ts in &self.prompt_cache {
            let belongs = session_start.is_some_and(|s| *ts < s + session_duration);
            if belongs {
                session_count += 1;
            } else {
                session_start = Some(*ts);
                session_count = 1;
            }
        }

        let (session_count, session_resets_at) = match session_start {
            Some(start) if now < start + session_duration => {
                (session_count, start + session_duration)
            }
            _ => (0, now + session_duration),
        };

        // Weekly is a simple count over the calendar week
        let weekly_count = self
            .prompt_cache
            .iter()
            .filter(|t| **t >= weekly_start)
            .count() as u32;

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
            resets_at: weekly_end,
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
            data_updated_at: Some(now),
        }
    }
}

/// Open `path`, seek to `from_offset`, parse complete new lines for user
/// prompts, and push timestamps into `cache`. Returns the new offset that
/// has been fully processed (always ends at a newline so partially-written
/// trailing lines are re-read on the next poll).
fn read_new_prompts_from(
    path: &Path,
    from_offset: u64,
    cache: &mut Vec<DateTime<Utc>>,
    user_lines_seen: &mut usize,
) -> std::io::Result<u64> {
    let mut file = std::fs::File::open(path)?;
    file.seek(SeekFrom::Start(from_offset))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    // Only commit up to the last newline — anything after may be a partial
    // line still being appended.
    let usable_len = match buf.rfind('\n') {
        Some(pos) => pos + 1,
        None => 0,
    };
    let usable = &buf[..usable_len];

    for line in usable.lines() {
        if !line.contains("\"type\":\"user\"") {
            continue;
        }
        *user_lines_seen += 1;
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
        cache.push(ts);
    }

    Ok(from_offset + usable_len as u64)
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

/// Monday 00:00 UTC of the week containing `now`.
fn current_week_start_utc(now: DateTime<Utc>) -> DateTime<Utc> {
    let weekday_from_monday = now.weekday().num_days_from_monday() as i64;
    let target_date = now.date_naive() - Duration::days(weekday_from_monday);
    target_date
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_utc()
}
