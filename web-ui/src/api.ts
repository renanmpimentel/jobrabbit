// Types mirror the structs serialized by the backend Rust (serde).

export interface Stats {
  total_jobs: number;
  total_applications: number;
  applied: number;
  pending_actions: number;
}

export interface Job {
  id: number;
  title: string;
  company: string;
  url: string;
  source: string;
  description: string;
  fit_score: number | null;
  found_at: string;
}

export interface PendingAction {
  id: number;
  job_id: number | null;
  kind: string;
  description: string;
  url: string | null;
  field_key: string | null;
  resolved: boolean;
  created_at: string;
}

export interface Application {
  id: number;
  job_id: number;
  status: string;
  cv_generated: string | null;
  cover_letter: string | null;
  screenshot_path: string | null;
  stage: string | null;
  notes: string | null;
  created_at: string;
}

export interface Feedback {
  id: number;
  summary: string;
  suggestions: string;
  created_at: string;
}

export interface SearchVariant {
  id: number;
  label: string;
  query: string;
  enabled: boolean;
  created_at: string;
}

export interface JobSource {
  id: number;
  name: string;
  domain: string;
  enabled: boolean;
  builtin: boolean;
  created_at: string;
}

export interface Profile {
  background: string;
  cv_base: string;
  updated_at: string;
}

export interface Answer {
  key: string;
  label: string;
  value: string;
  updated_at: string;
}

export interface CvReview {
  id: number;
  score: number;
  target: string;
  report: string;
  created_at: string;
}

export interface CvVersion {
  id: number;
  target: string;
  content: string;
  created_at: string;
}

export interface Settings {
  claude_bin: string;
  idle_threshold_secs: number;
  auto_run_on_idle: boolean;
  use_chrome: boolean;
  bypass_permissions: boolean;
  apply_mode: string;
  require_human_review: boolean;
  hybrid_threshold: number;
  dry_run: boolean;
  language_filter: boolean;
  work_model: string;
  locale: string;
  cv_file_path: string;
  linkedin_url: string;
}

export type CheckStatus = "Ok" | "Warn" | "Fail";

export interface DoctorCheck {
  name: string;
  status: CheckStatus;
  detail: string;
  hint: string | null;
}

// Enum externamente etiquetado: "Idle" | "Running" | { Error: string }.
export type AgentStatus = "Idle" | "Running" | { Error: string };

export interface PendingNotify {
  kind: string;
  description: string;
}

export interface WebEvent {
  event: string;
  logs: string[];
  status: AgentStatus | null;
  focus_tab: number | null;
  refresh: boolean;
  notify?: PendingNotify | null;
}

// ---- fetch helpers ---------------------------------------------------------

async function handle<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || `${res.status} ${res.statusText}`);
  }
  const text = await res.text();
  return (text ? JSON.parse(text) : null) as T;
}

export function getJSON<T>(path: string): Promise<T> {
  return fetch(`/api${path}`).then((r) => handle<T>(r));
}

export function post<T = unknown>(path: string, body?: unknown): Promise<T> {
  return fetch(`/api${path}`, {
    method: "POST",
    headers: body !== undefined ? { "Content-Type": "application/json" } : {},
    body: body !== undefined ? JSON.stringify(body) : undefined,
  }).then((r) => handle<T>(r));
}

export function del<T = unknown>(path: string): Promise<T> {
  return fetch(`/api${path}`, { method: "DELETE" }).then((r) => handle<T>(r));
}

export function statusLabel(s: AgentStatus): { key: string; param?: string } {
  if (s === "Idle") return { key: "status.idle" };
  if (s === "Running") return { key: "status.running" };
  return { key: "status.error", param: s.Error };
}

export function isRunning(s: AgentStatus): boolean {
  return s === "Running";
}
