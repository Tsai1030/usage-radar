import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import type { SourceHealth, SourceId, UsageSnapshot, UsageWindow } from "./types";

type Snapshots = Record<SourceId, UsageSnapshot | null>;
type Tier = "pro" | "max-5x" | "max-20x";
interface AppSettings {
  claude_tier: Tier;
  claude_session_cap_override?: number | null;
  claude_weekly_cap_override?: number | null;
  claude_session_calibration_pct?: number | null;
  claude_weekly_calibration_pct?: number | null;
  codex_session_pct_override?: number | null;
  codex_weekly_pct_override?: number | null;
}

const TIER_DEFAULTS: Record<Tier, { session: number; weekly: number }> = {
  "pro":     { session: 45,  weekly: 300 },
  "max-5x":  { session: 225, weekly: 1500 },
  "max-20x": { session: 900, weekly: 6000 },
};

const TABS: { id: SourceId; label: string }[] = [
  { id: "codex", label: "Codex" },
  { id: "claude", label: "Claude" },
];

const TIERS: { id: Tier; label: string; session: string; weekly: string }[] = [
  { id: "pro", label: "Pro", session: "~45 / 5h", weekly: "~300 / 7d" },
  { id: "max-5x", label: "Max 5×", session: "~225 / 5h", weekly: "~1,500 / 7d" },
  { id: "max-20x", label: "Max 20×", session: "~900 / 5h", weekly: "~6,000 / 7d" },
];

const CARD_WIDTH = 260;
const CARD_HEIGHT_NORMAL = 100;
const CARD_HEIGHT_SETTINGS_CLAUDE = 290;
const CARD_HEIGHT_SETTINGS_CODEX = 170;

function barColor(pct: number, brandVar: string): string {
  if (pct >= 85) return "var(--bar-red)";
  if (pct >= 60) return "var(--bar-yellow)";
  return `var(${brandVar})`;
}

function healthDotClass(health: SourceHealth | undefined): string {
  if (!health || health === "ok") return "dot dot-green";
  if (health === "schema_mismatch" || health === "file_locked" || health === "no_data")
    return "dot dot-yellow";
  return "dot dot-red";
}

function formatResets(iso: string | undefined): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    const diff = d.getTime() - Date.now();
    if (diff <= 0) return "now";
    const mins = Math.floor(diff / 60000);
    if (mins < 60) return `${mins}m`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ${mins % 60}m`;
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h`;
  } catch {
    return iso;
  }
}

function formatAge(iso: string | null | undefined): string | null {
  if (!iso) return null;
  try {
    const d = new Date(iso);
    const elapsed = Date.now() - d.getTime();
    if (elapsed < 5 * 60 * 1000) return null; // fresh, hide
    const mins = Math.floor(elapsed / 60000);
    if (mins < 60) return `${mins}m`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h`;
    const days = Math.floor(hours / 24);
    return `${days}d`;
  } catch {
    return null;
  }
}

function Row({
  label,
  w,
  estimated,
  brandVar,
}: {
  label: string;
  w: UsageWindow | null;
  estimated: boolean;
  brandVar: string;
}) {
  if (!w) {
    return (
      <div className="row">
        <span className="row-label">
          {label}
          {estimated && <span className="est-tag">est.</span>}
        </span>
        <div className="bar">
          <div className="bar-fill" style={{ width: "0%", background: "var(--bar-muted)" }} />
        </div>
        <span className="row-right muted">N/A</span>
      </div>
    );
  }
  const pct = Math.max(0, Math.min(100, w.used_percent));
  return (
    <div className="row">
      <span className="row-label">
        {label}
        {estimated && <span className="est-tag">est.</span>}
      </span>
      <div className="bar">
        <div className="bar-fill" style={{ width: `${pct}%`, background: barColor(pct, brandVar) }} />
      </div>
      <span className="row-right">
        <span className="row-pct">{pct.toFixed(0)}%</span>
        <span className="row-meta">{formatResets(w.resets_at)}</span>
      </span>
    </div>
  );
}

function ClaudeSettingsBody({
  tier,
  sessionPctInput,
  weeklyPctInput,
  sessionCount,
  weeklyCount,
  liveSessionPct,
  liveWeeklyPct,
  onChangeTier,
  onChangeSessionPct,
  onChangeWeeklyPct,
}: {
  tier: Tier;
  sessionPctInput: string;
  weeklyPctInput: string;
  sessionCount: number | null;
  weeklyCount: number | null;
  liveSessionPct: number | null;
  liveWeeklyPct: number | null;
  onChangeTier: (t: Tier) => void;
  onChangeSessionPct: (v: string) => void;
  onChangeWeeklyPct: (v: string) => void;
}) {
  return (
    <div className="settings-body">
      <div className="settings-section-label">Subscription</div>
      <div className="tier-list">
        {TIERS.map((t) => (
          <button
            key={t.id}
            className={`tier-row ${tier === t.id ? "active" : ""}`}
            onClick={() => onChangeTier(t.id)}
          >
            <span className={`tier-radio ${tier === t.id ? "checked" : ""}`} />
            <span className="tier-name">{t.label}</span>
            <span className="tier-caps">
              {t.session} <span className="muted">·</span> {t.weekly}
            </span>
          </button>
        ))}
      </div>

      <div className="settings-section-label">
        Calibrate to Anthropic web
        <span className="cap-counts">
          {sessionCount !== null && weeklyCount !== null
            ? `(${sessionCount}/5h · ${weeklyCount}/7d)`
            : ""}
        </span>
      </div>
      <div className="cap-row">
        <label className="cap-input">
          <span className="cap-label">Session %</span>
          <input
            type="text"
            inputMode="numeric"
            placeholder="—"
            value={sessionPctInput}
            onChange={(e) => onChangeSessionPct(e.target.value)}
          />
          <span className="cap-after">
            {liveSessionPct !== null && sessionPctInput
              ? `bar: ${liveSessionPct}%`
              : ""}
          </span>
        </label>
        <label className="cap-input">
          <span className="cap-label">Weekly %</span>
          <input
            type="text"
            inputMode="numeric"
            placeholder="—"
            value={weeklyPctInput}
            onChange={(e) => onChangeWeeklyPct(e.target.value)}
          />
          <span className="cap-after">
            {liveWeeklyPct !== null && weeklyPctInput
              ? `bar: ${liveWeeklyPct}%`
              : ""}
          </span>
        </label>
      </div>
      <div className="settings-foot muted">
        Type the % Anthropic shows. Re-type to recalibrate when drifted.
      </div>
    </div>
  );
}

function CodexSettingsBody({
  codexSessionPctInput,
  codexWeeklyPctInput,
  onChangeCodexSessionPct,
  onChangeCodexWeeklyPct,
}: {
  codexSessionPctInput: string;
  codexWeeklyPctInput: string;
  onChangeCodexSessionPct: (v: string) => void;
  onChangeCodexWeeklyPct: (v: string) => void;
}) {
  return (
    <div className="settings-body">
      <div className="settings-section-label">
        Manual override
        <span className="cap-counts">(when CLI is stale)</span>
      </div>
      <div className="cap-row">
        <label className="cap-input">
          <span className="cap-label">Session %</span>
          <input
            type="text"
            inputMode="numeric"
            placeholder="—"
            value={codexSessionPctInput}
            onChange={(e) => onChangeCodexSessionPct(e.target.value)}
          />
        </label>
        <label className="cap-input">
          <span className="cap-label">Weekly %</span>
          <input
            type="text"
            inputMode="numeric"
            placeholder="—"
            value={codexWeeklyPctInput}
            onChange={(e) => onChangeCodexWeeklyPct(e.target.value)}
          />
        </label>
      </div>
      <div className="settings-foot muted">
        Use ChatGPT/OpenAI dashboard's %. Empty = live CLI value.
      </div>
    </div>
  );
}

export default function App() {
  const [snapshots, setSnapshots] = useState<Snapshots>({ codex: null, claude: null });
  const [tab, setTab] = useState<SourceId>("codex");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [tier, setTier] = useState<Tier>("max-5x");
  const [sessionOverride, setSessionOverride] = useState<string>("");
  const [weeklyOverride, setWeeklyOverride] = useState<string>("");
  const [sessionPctInput, setSessionPctInput] = useState<string>("");
  const [weeklyPctInput, setWeeklyPctInput] = useState<string>("");
  const [codexSessionPctInput, setCodexSessionPctInput] = useState<string>("");
  const [codexWeeklyPctInput, setCodexWeeklyPctInput] = useState<string>("");
  const [savedFlash, setSavedFlash] = useState(false);

  useEffect(() => {
    invoke<UsageSnapshot[]>("get_initial_snapshots")
      .then((arr) => {
        const next: Snapshots = { codex: null, claude: null };
        for (const s of arr) next[s.source] = s;
        setSnapshots(next);
      })
      .catch(() => {
        // browser preview fallback
        const now = new Date();
        const plus = (h: number) => new Date(now.getTime() + h * 3600_000).toISOString();
        setSnapshots({
          codex: {
            source: "codex",
            session: { used_percent: 12, resets_at: plus(3.6), window_minutes: 300, is_estimated: false },
            weekly: { used_percent: 36, resets_at: plus(96), window_minutes: 10080, is_estimated: false },
            plan_type: "plus",
            source_health: "ok",
            fetched_at: now.toISOString(),
          },
          claude: {
            source: "claude",
            session: { used_percent: 16, resets_at: plus(2.25), window_minutes: 300, is_estimated: true },
            weekly: { used_percent: 15, resets_at: plus(72), window_minutes: 10080, is_estimated: true },
            plan_type: "max-5x",
            source_health: "ok",
            fetched_at: now.toISOString(),
          },
        });
      });

    invoke<AppSettings>("get_settings")
      .then((s) => {
        setTier(s.claude_tier);
        setSessionOverride(
          s.claude_session_cap_override != null ? String(s.claude_session_cap_override) : "",
        );
        setWeeklyOverride(
          s.claude_weekly_cap_override != null ? String(s.claude_weekly_cap_override) : "",
        );
        setSessionPctInput(
          s.claude_session_calibration_pct != null ? String(s.claude_session_calibration_pct) : "",
        );
        setWeeklyPctInput(
          s.claude_weekly_calibration_pct != null ? String(s.claude_weekly_calibration_pct) : "",
        );
        setCodexSessionPctInput(
          s.codex_session_pct_override != null ? String(s.codex_session_pct_override) : "",
        );
        setCodexWeeklyPctInput(
          s.codex_weekly_pct_override != null ? String(s.codex_weekly_pct_override) : "",
        );
      })
      .catch(() => undefined);

    const unListenUpdate = listen<UsageSnapshot>("usage-update", (e) => {
      setSnapshots((prev) => ({ ...prev, [e.payload.source]: e.payload }));
    });
    const unListenToggle = listen<boolean>("toggle-settings", (e) => {
      setSettingsOpen(e.payload);
    });
    return () => {
      unListenUpdate.then((fn) => fn()).catch(() => undefined);
      unListenToggle.then((fn) => fn()).catch(() => undefined);
    };
  }, []);

  // Resize window when settings toggle or active tab changes (each tab has its
  // own settings panel height because content differs).
  useEffect(() => {
    let targetHeight = CARD_HEIGHT_NORMAL;
    if (settingsOpen) {
      targetHeight =
        tab === "claude" ? CARD_HEIGHT_SETTINGS_CLAUDE : CARD_HEIGHT_SETTINGS_CODEX;
    }
    getCurrentWindow()
      .setSize(new LogicalSize(CARD_WIDTH, targetHeight))
      .catch(() => undefined);
  }, [settingsOpen, tab]);

  const sessionCount = snapshots.claude?.session?.current_count ?? null;
  const weeklyCount = snapshots.claude?.weekly?.current_count ?? null;

  // Display the % the user TYPED (sticky), not back-computed from drifting count.
  // Card bar still shows live % (= count / cap * 100) — that's the truth.
  const capDisplaySession = sessionOverride ? parseInt(sessionOverride, 10) || null : null;
  const capDisplayWeekly = weeklyOverride ? parseInt(weeklyOverride, 10) || null : null;

  // Live % the bar is currently showing — surfaced beside the calibration input
  // so the user can see "I set 39%, bar is now 35% — count drifted".
  const liveSessionPct = useMemo(() => {
    if (sessionCount === null || !capDisplaySession) return null;
    return Math.round((sessionCount / capDisplaySession) * 100);
  }, [sessionCount, capDisplaySession]);
  const liveWeeklyPct = useMemo(() => {
    if (weeklyCount === null || !capDisplayWeekly) return null;
    return Math.round((weeklyCount / capDisplayWeekly) * 100);
  }, [weeklyCount, capDisplayWeekly]);

  const persistSettings = (next: AppSettings) => {
    invoke("update_settings", { settingsIn: next })
      .then(() => {
        setSavedFlash(true);
        window.setTimeout(() => setSavedFlash(false), 1200);
      })
      .catch(console.error);
  };

  const buildSettings = (overrides: Partial<AppSettings> = {}): AppSettings => ({
    claude_tier: tier,
    claude_session_cap_override: capDisplaySession,
    claude_weekly_cap_override: capDisplayWeekly,
    claude_session_calibration_pct: sessionPctInput ? parseInt(sessionPctInput, 10) || null : null,
    claude_weekly_calibration_pct: weeklyPctInput ? parseInt(weeklyPctInput, 10) || null : null,
    codex_session_pct_override: codexSessionPctInput ? parseInt(codexSessionPctInput, 10) || null : null,
    codex_weekly_pct_override: codexWeeklyPctInput ? parseInt(codexWeeklyPctInput, 10) || null : null,
    ...overrides,
  });

  const updateTier = (t: Tier) => {
    setTier(t);
    persistSettings(buildSettings({ claude_tier: t }));
  };

  // User types target %. We snapshot current count, compute & save cap.
  // Also save the typed % so the input remembers what was calibrated to.
  const onChangeSessionPct = (raw: string) => {
    const clean = raw.replace(/[^\d]/g, "").slice(0, 3);
    setSessionPctInput(clean);
    if (clean === "" || sessionCount === null || sessionCount === 0) {
      setSessionOverride("");
      persistSettings(
        buildSettings({
          claude_session_cap_override: null,
          claude_session_calibration_pct: null,
        }),
      );
      return;
    }
    const pct = parseInt(clean, 10);
    if (!pct) return;
    const cap = Math.max(1, Math.round((sessionCount / pct) * 100));
    setSessionOverride(String(cap));
    persistSettings(
      buildSettings({
        claude_session_cap_override: cap,
        claude_session_calibration_pct: pct,
      }),
    );
  };

  const onChangeWeeklyPct = (raw: string) => {
    const clean = raw.replace(/[^\d]/g, "").slice(0, 3);
    setWeeklyPctInput(clean);
    if (clean === "" || weeklyCount === null || weeklyCount === 0) {
      setWeeklyOverride("");
      persistSettings(
        buildSettings({
          claude_weekly_cap_override: null,
          claude_weekly_calibration_pct: null,
        }),
      );
      return;
    }
    const pct = parseInt(clean, 10);
    if (!pct) return;
    const cap = Math.max(1, Math.round((weeklyCount / pct) * 100));
    setWeeklyOverride(String(cap));
    persistSettings(
      buildSettings({
        claude_weekly_cap_override: cap,
        claude_weekly_calibration_pct: pct,
      }),
    );
  };

  const onChangeCodexSessionPct = (raw: string) => {
    const clean = raw.replace(/[^\d]/g, "").slice(0, 3);
    setCodexSessionPctInput(clean);
    const pct = clean === "" ? null : parseInt(clean, 10) || null;
    persistSettings(buildSettings({ codex_session_pct_override: pct }));
  };

  const onChangeCodexWeeklyPct = (raw: string) => {
    const clean = raw.replace(/[^\d]/g, "").slice(0, 3);
    setCodexWeeklyPctInput(clean);
    const pct = clean === "" ? null : parseInt(clean, 10) || null;
    persistSettings(buildSettings({ codex_weekly_pct_override: pct }));
  };

  const active = snapshots[tab];
  const isClaude = tab === "claude";
  const brandVar = isClaude ? "--claude-brand" : "--codex-brand";
  const staleAge = formatAge(active?.data_updated_at);

  const healthClass = useMemo(
    () => TABS.map((t) => healthDotClass(snapshots[t.id]?.source_health)),
    [snapshots],
  );

  return (
    <div className="card" data-source={tab} data-tauri-drag-region>
      <div className="tabs" data-tauri-drag-region>
        {TABS.map((t, i) => (
          <button
            key={t.id}
            data-id={t.id}
            className={`tab ${tab === t.id ? "active" : ""}`}
            onClick={() => setTab(t.id)}
          >
            <span className={healthClass[i]} />
            {t.label}
          </button>
        ))}
        <div className="spacer" />
        {savedFlash && <span className="saved-flash">saved</span>}
        {settingsOpen ? (
          <button
            className="iconbtn"
            title="Close settings"
            onClick={() => setSettingsOpen(false)}
          >
            ×
          </button>
        ) : (
          <>
            <button
              className="iconbtn"
              title="Hide to tray"
              onClick={() => {
                getCurrentWindow().hide().catch(() => undefined);
              }}
            >
              −
            </button>
            <button
              className="iconbtn"
              title="Settings"
              onClick={() => setSettingsOpen(true)}
            >
              ⚙
            </button>
          </>
        )}
      </div>

      {settingsOpen ? (
        tab === "claude" ? (
          <ClaudeSettingsBody
            tier={tier}
            sessionPctInput={sessionPctInput}
            weeklyPctInput={weeklyPctInput}
            sessionCount={sessionCount}
            weeklyCount={weeklyCount}
            liveSessionPct={liveSessionPct}
            liveWeeklyPct={liveWeeklyPct}
            onChangeTier={updateTier}
            onChangeSessionPct={onChangeSessionPct}
            onChangeWeeklyPct={onChangeWeeklyPct}
          />
        ) : (
          <CodexSettingsBody
            codexSessionPctInput={codexSessionPctInput}
            codexWeeklyPctInput={codexWeeklyPctInput}
            onChangeCodexSessionPct={onChangeCodexSessionPct}
            onChangeCodexWeeklyPct={onChangeCodexWeeklyPct}
          />
        )
      ) : (
        <div className="body">
          {staleAge && <div className="stale-banner">stale · {staleAge} ago</div>}
          <Row label="Session" w={active?.session ?? null} estimated={isClaude} brandVar={brandVar} />
          <Row label="Weekly" w={active?.weekly ?? null} estimated={isClaude} brandVar={brandVar} />
        </div>
      )}
    </div>
  );
}
