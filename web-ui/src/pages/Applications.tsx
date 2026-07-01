import { useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { Send, ExternalLink, Loader2, Image } from "lucide-react";
import { useStats, useApplications, useJobs, useAppliedJobs, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Badge, Empty, Input, Button, StatCard, cn, Textarea, Select, ScoreBadge, SectionTitle, SkeletonRows, ErrorState } from "../ui";
import { fadeUp, stagger } from "../motion";
import type { Job } from "../hooks";
import { useToast } from "../toast";

type Band = "strong" | "medium" | "weak" | "unscored";
const BANDS: Band[] = ["strong", "medium", "weak", "unscored"];

// Fit band per DESIGN.md score scale (presentation grouping only).
function fitBand(fit: number | null): Band {
  if (fit == null) return "unscored";
  if (fit >= 0.75) return "strong";
  if (fit >= 0.55) return "medium";
  return "weak";
}

function statusTone(status: string): string {
  if (status === "applied") return "green";
  if (status === "dry_run") return "yellow";
  if (status === "ready" || status === "pending") return "yellow";
  return "red";
}

function stageTone(stage: string | null | undefined): string {
  if (!stage) stage = "applied";
  if (stage === "applied") return "slate";
  if (stage === "screening") return "yellow";
  if (stage === "interview") return "iris";
  if (stage === "offer") return "green";
  if (stage === "rejected") return "red";
  return "slate";
}

export default function Applications() {
  const { t } = useTranslation();
  const stats = useStats();
  const applications = useApplications();
  const jobs = useJobs();
  const appliedJobs = useAppliedJobs();
  const invalidate = useInvalidate();
  const toast = useToast();

  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [filter, setFilter] = useState<"available" | "applied">("available");
  const [editingNotes, setEditingNotes] = useState<Record<number, string>>({});

  const apps = applications.data ?? [];
  const allJobs = jobs.data ?? [];
  const appJobs = appliedJobs.data ?? [];
  const jobsMap = new Map(allJobs.map((j) => [j.id, j]));
  const appsByJobId = new Map(apps.map((a) => [a.job_id, a]));

  async function applyByUrl() {
    if (!url.trim()) return;
    setBusy(true);
    try {
      await post("/apply-url", { url });
      setUrl("");
      invalidate();
      toast.success(t("common.sent"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <motion.div variants={stagger} initial="hidden" animate="show" className="space-y-8">
      {/* Stats */}
      <motion.div variants={stagger} className="grid grid-cols-2 gap-4 sm:grid-cols-2">
        <StatCard label={t("applications.totalLabel")} value={stats.data?.total_applications ?? 0} />
        <StatCard label={t("applications.appliedLabel")} value={stats.data?.applied ?? 0} />
      </motion.div>

      {/* Apply by URL Card */}
      <motion.div variants={fadeUp}>
        <Card>
          <CardHeader title={t("applications.title")} />
          <div className="space-y-4 px-6 py-5">
            <div className="flex items-center gap-3">
              <Input
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !busy && url.trim()) {
                    applyByUrl();
                  }
                }}
                placeholder={t("applications.urlPlaceholder")}
                disabled={busy}
              />
              <Button
                variant="primary"
                disabled={busy || !url.trim()}
                onClick={applyByUrl}
                className="shrink-0"
              >
                {busy ? (
                  <>
                    <Loader2 size={14} className="animate-spin" />
                    <span className="hidden sm:inline">{t("applications.applying")}</span>
                  </>
                ) : (
                  <>
                    <Send size={14} />
                    <span className="hidden sm:inline">{t("applications.applyButton")}</span>
                  </>
                )}
              </Button>
            </div>
            <p className="text-xs text-fg-muted">{t("applications.hint")}</p>
          </div>
        </Card>
      </motion.div>

      {/* Filter and Jobs/Applications List */}
      <motion.div variants={fadeUp}>
        <Card>
          <CardHeader title={t("applications.listTitle")} />
          <div className="space-y-5 px-6 py-5">
            {/* Filter — segmented control */}
            <div className="inline-flex rounded-md border border-border bg-surface-2 p-0.5">
              {(["available", "applied"] as const).map((f) => (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  className={cn(
                    "rounded-[5px] px-4 py-1.5 text-sm transition-colors",
                    filter === f
                      ? "bg-accent text-accent-fg font-semibold"
                      : "text-fg-muted hover:text-fg",
                  )}
                >
                  {f === "available" ? t("applications.filterAvailable") : t("applications.filterApplied")}
                </button>
              ))}
            </div>

            {/* Available Jobs — grouped by fit band, ScoreBadge unified */}
            {filter === "available" && (
              <>
                {jobs.isLoading ? (
                  <SkeletonRows rows={4} />
                ) : jobs.isError ? (
                  <ErrorState message={t("common.error")} retryLabel={t("common.retry")} onRetry={() => jobs.refetch()} />
                ) : allJobs.length === 0 ? (
                  <Empty>{t("applications.availableEmpty")}</Empty>
                ) : (
                  <div className="space-y-6">
                    {BANDS.map((band) => {
                      const items = allJobs
                        .filter((j) => fitBand(j.fit_score) === band)
                        .sort((a, b) => (b.fit_score ?? -1) - (a.fit_score ?? -1));
                      if (items.length === 0) return null;
                      return (
                        <div key={band}>
                          <SectionTitle count={items.length}>{t(`applications.bands.${band}`)}</SectionTitle>
                          <ul className="overflow-hidden rounded-md border border-border divide-y divide-border">
                            {items.map((job: Job) => (
                              <motion.li
                                key={job.id}
                                variants={fadeUp}
                                className="group/link flex items-center gap-4 px-4 py-3 transition-colors hover:bg-surface-2"
                              >
                                <ScoreBadge value={job.fit_score} />
                                <a
                                  href={job.url}
                                  target="_blank"
                                  rel="noreferrer"
                                  className="min-w-0 flex-1"
                                  title={job.title}
                                >
                                  <div className="flex items-center gap-1.5">
                                    <span className="truncate text-sm font-medium leading-snug text-fg transition-colors group-hover/link:text-accent">
                                      {job.title}
                                    </span>
                                    <ExternalLink size={12} className="shrink-0 text-fg-subtle opacity-0 transition group-hover/link:opacity-100" />
                                  </div>
                                  <div className="truncate text-sm text-fg-subtle">{job.company}</div>
                                </a>
                                {job.source && <span className="hidden font-mono text-xs text-fg-subtle sm:inline">{job.source}</span>}
                              </motion.li>
                            ))}
                          </ul>
                        </div>
                      );
                    })}
                  </div>
                )}
              </>
            )}

            {/* Applied Jobs */}
            {filter === "applied" && (
              <>
                {appliedJobs.isLoading ? (
                  <SkeletonRows rows={4} />
                ) : appliedJobs.isError ? (
                  <ErrorState message={t("common.error")} retryLabel={t("common.retry")} onRetry={() => appliedJobs.refetch()} />
                ) : appJobs.length === 0 ? (
                  <Empty>{t("applications.appliedEmpty")}</Empty>
                ) : (
                  <ul className="overflow-hidden rounded-md border border-border divide-y divide-border">
                    {appJobs.map((job) => {
                      const app = appsByJobId.get(job.id);
                      const noteValue = editingNotes[app?.id ?? 0] ?? (app?.notes ?? "");
                      return (
                        <motion.li
                          key={job.id}
                          variants={fadeUp}
                          className="px-6 py-4 transition hover:bg-surface-2"
                        >
                          <div className="flex items-start justify-between gap-4">
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2 mb-2">
                                {app && (
                                  <>
                                    <Badge tone={statusTone(app.status)}>
                                      {t(`applications.status.${app.status}`, app.status)}
                                    </Badge>
                                    {app.screenshot_path && (
                                      <a
                                        href={`/api/screenshot/${app.id}`}
                                        target="_blank"
                                        rel="noreferrer"
                                        className="text-fg-muted hover:text-accent transition"
                                        title={t("applications.viewProof")}
                                      >
                                        <Image size={14} />
                                      </a>
                                    )}
                                  </>
                                )}
                              </div>
                              <a
                                href={job.url}
                                target="_blank"
                                rel="noreferrer"
                                className="group/link mb-1 flex items-center gap-2"
                                title={job.title}
                              >
                                <span className="truncate text-sm font-medium leading-snug text-fg transition-colors group-hover/link:text-accent">
                                  {job.title} <span className="font-normal text-fg-muted">@ {job.company}</span>
                                </span>
                                <ExternalLink size={12} className="shrink-0 text-fg-subtle opacity-0 transition group-hover/link:opacity-100" />
                              </a>
                              {app && (
                                <div className="text-xs text-fg-subtle mt-2">
                                  {new Date(app.created_at).toLocaleDateString(undefined, {
                                    year: "numeric",
                                    month: "short",
                                    day: "numeric",
                                  })}
                                </div>
                              )}
                              {app && (
                                <div className="mt-4 space-y-3">
                                  <div className="flex items-center gap-3">
                                    <label className="text-xs text-fg-muted font-medium">
                                      {t("applications.stageLabel")}:
                                    </label>
                                    <Select
                                      value={app.stage ?? "applied"}
                                      onChange={(e) => {
                                        post(`/applications/${app.id}/track`, {
                                          stage: e.target.value,
                                          notes: noteValue || null,
                                        })
                                          .then(() => {
                                            invalidate();
                                            toast.success(t("common.saved"));
                                          })
                                          .catch((e) => toast.error(String(e)));
                                      }}
                                      className="py-1.5 text-xs"
                                    >
                                      <option value="applied">{t("applications.stage.applied")}</option>
                                      <option value="screening">{t("applications.stage.screening")}</option>
                                      <option value="interview">{t("applications.stage.interview")}</option>
                                      <option value="offer">{t("applications.stage.offer")}</option>
                                      <option value="rejected">{t("applications.stage.rejected")}</option>
                                    </Select>
                                    <Badge tone={stageTone(app.stage)}>
                                      {t(`applications.stage.${app.stage ?? "applied"}`)}
                                    </Badge>
                                  </div>
                                  <div>
                                    <label className="text-xs text-fg-muted font-medium block mb-1">
                                      {t("applications.notesLabel")}
                                    </label>
                                    <Textarea
                                      value={noteValue}
                                      onChange={(e) => {
                                        setEditingNotes({
                                          ...editingNotes,
                                          [app.id]: e.target.value,
                                        });
                                      }}
                                      onBlur={() => {
                                        post(`/applications/${app.id}/track`, {
                                          stage: app.stage ?? "applied",
                                          notes: noteValue || null,
                                        })
                                          .then(() => {
                                            invalidate();
                                            toast.success(t("common.saved"));
                                            setEditingNotes((prev) => {
                                              const next = { ...prev };
                                              delete next[app.id];
                                              return next;
                                            });
                                          })
                                          .catch((e) => toast.error(String(e)));
                                      }}
                                      placeholder={t("applications.notesPlaceholder")}
                                      className="text-xs resize-none"
                                      rows={2}
                                    />
                                  </div>
                                </div>
                              )}
                              {app?.screenshot_path && (
                                <a
                                  href={`/api/screenshot/${app.id}`}
                                  target="_blank"
                                  rel="noreferrer"
                                  className="mt-2 inline-block"
                                  title={t("applications.viewProof")}
                                >
                                  <img
                                    src={`/api/screenshot/${app.id}`}
                                    alt={t("applications.viewProof")}
                                    className="h-24 w-auto max-w-[240px] rounded-md border border-border object-cover object-top transition hover:border-accent/50"
                                  />
                                </a>
                              )}
                            </div>
                          </div>
                        </motion.li>
                      );
                    })}
                  </ul>
                )}
              </>
            )}
          </div>
        </Card>
      </motion.div>
    </motion.div>
  );
}
