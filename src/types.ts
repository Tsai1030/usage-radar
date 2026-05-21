export type SourceId = "codex" | "claude";

export type SourceHealth =
  | "ok"
  | "path_not_found"
  | "permission_denied"
  | "schema_mismatch"
  | "file_locked"
  | "no_data";

export interface UsageWindow {
  used_percent: number;
  current_count: number | null;
  resets_at: string;
  window_minutes: number;
  is_estimated: boolean;
}

export interface UsageSnapshot {
  source: SourceId;
  session: UsageWindow | null;
  weekly: UsageWindow | null;
  plan_type: string | null;
  source_health: SourceHealth;
  fetched_at: string;
  data_updated_at: string | null;
}
