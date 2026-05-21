use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ClaudeTier {
    #[serde(rename = "pro")]
    Pro,
    #[serde(rename = "max-5x")]
    #[default]
    Max5x,
    #[serde(rename = "max-20x")]
    Max20x,
}

impl ClaudeTier {
    /// (session_cap, weekly_cap) — community-observed user-prompt counts,
    /// NOT official Anthropic numbers. Phase 2 should learn from real
    /// reset behavior.
    pub fn caps(&self) -> (u32, u32) {
        match self {
            ClaudeTier::Pro => (45, 300),
            ClaudeTier::Max5x => (225, 1500),
            ClaudeTier::Max20x => (900, 6000),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ClaudeTier::Pro => "pro",
            ClaudeTier::Max5x => "max-5x",
            ClaudeTier::Max20x => "max-20x",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub claude_tier: ClaudeTier,
    /// User-provided override for session cap; falls back to tier default when None.
    #[serde(default)]
    pub claude_session_cap_override: Option<u32>,
    /// User-provided override for weekly cap; falls back to tier default when None.
    #[serde(default)]
    pub claude_weekly_cap_override: Option<u32>,
    /// What the user last typed into the "Session %" field (UI-only, parser ignores).
    /// Persisted so the input keeps showing the calibration target even as live count drifts.
    #[serde(default)]
    pub claude_session_calibration_pct: Option<u32>,
    #[serde(default)]
    pub claude_weekly_calibration_pct: Option<u32>,
    /// User-supplied Codex usage % when the CLI rollout is stale (e.g. they used the
    /// web/IDE instead of CLI). Replaces the rate_limits value when set.
    #[serde(default)]
    pub codex_session_pct_override: Option<u32>,
    #[serde(default)]
    pub codex_weekly_pct_override: Option<u32>,
    /// Whether to surface a system notification when any bar crosses
    /// NOTIFY_THRESHOLD_PCT from below. Default on.
    #[serde(default = "default_notify_at_threshold")]
    pub notify_at_threshold: bool,
}

fn default_notify_at_threshold() -> bool {
    true
}

pub const NOTIFY_THRESHOLD_PCT: f32 = 85.0;

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            claude_tier: ClaudeTier::default(),
            claude_session_cap_override: None,
            claude_weekly_cap_override: None,
            claude_session_calibration_pct: None,
            claude_weekly_calibration_pct: None,
            codex_session_pct_override: None,
            codex_weekly_pct_override: None,
            notify_at_threshold: true,
        }
    }
}

impl AppSettings {
    /// Returns the effective (session_cap, weekly_cap) by applying any user
    /// overrides on top of the tier defaults.
    pub fn effective_caps(&self) -> (u32, u32) {
        let (default_session, default_weekly) = self.claude_tier.caps();
        let session = self.claude_session_cap_override.unwrap_or(default_session).max(1);
        let weekly = self.claude_weekly_cap_override.unwrap_or(default_weekly).max(1);
        (session, weekly)
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

pub fn settings_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".usage-radar").join("settings.json"))
}

pub fn load() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save(settings: &AppSettings) -> std::io::Result<()> {
    let path = settings_path()
        .ok_or_else(|| std::io::Error::other("home dir not resolvable"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    std::fs::write(&path, json)
}
