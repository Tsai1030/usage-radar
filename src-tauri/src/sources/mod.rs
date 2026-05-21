pub mod claude;
pub mod codex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceId {
    Codex,
    Claude,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SourceHealth {
    Ok,
    PathNotFound,
    PermissionDenied,
    SchemaMismatch,
    FileLocked,
    NoData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageWindow {
    pub used_percent: f32,
    /// Raw count behind the percentage (e.g. real user prompts for Claude).
    /// `None` when the source can't expose it (e.g. Codex only gives a %).
    pub current_count: Option<u32>,
    pub resets_at: DateTime<Utc>,
    pub window_minutes: u32,
    pub is_estimated: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageSnapshot {
    pub source: SourceId,
    pub session: Option<UsageWindow>,
    pub weekly: Option<UsageWindow>,
    pub plan_type: Option<String>,
    pub source_health: SourceHealth,
    pub fetched_at: DateTime<Utc>,
    /// When the underlying data was last produced (e.g. when Codex CLI last
    /// received a rate_limits response). `None` for live sources (Claude
    /// recomputes every poll). Frontend uses this to surface staleness.
    pub data_updated_at: Option<DateTime<Utc>>,
}

pub trait UsageSource: Send + Sync {
    fn id(&self) -> SourceId;
    fn poll(&mut self) -> UsageSnapshot;
}
