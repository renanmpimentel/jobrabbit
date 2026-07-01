import { useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { Send, ExternalLink, Loader2, Image } from "lucide-react";
import { useStats, useApplications, useJobs, useAppliedJobs, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Badge, Empty, Input, Button, StatCard, cn, Textarea } from "../ui";
import { fadeUp, stagger } from "../motion";

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
    } catch (e) {
      alert(String(e));
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
            {/* Filter Buttons */}
            <div className="flex flex-col gap-2">
              {(["available", "applied"] as const).map((f) => (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  className={cn(
                    "rounded-lg border px-4 py-2 text-left text-sm transition-colors",
                    filter === f
                      ? "border-accent/40 bg-accent/12 text-accent font-medium"
                      : "border-border text-fg-muted hover:bg-surface-2",
                  )}
                >
                  {f === "available" ? t("applications.filterAvailable") : t("applications.filterApplied")}
                </button>
              ))}
            </div>

            {/* Available Jobs */}
            {filter === "available" && (
              <>
                {allJobs.length === 0 ? (
                  <Empty>{t("applications.availableEmpty")}</Empty>
                ) : (
                  <ul className="divide-y divide-border -mx-6 -mb-5">
                    {allJobs.map((job) => (
                      <motion.li
                        key={job.id}
                        variants={fadeUp}
                        className="px-6 py-4 transition hover:bg-surface-2"
                      >
                        <div className="flex items-start justify-between gap-4">
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-2">
                              {job.fit_score !== null && (
                                <Badge tone="iris">
                                  {Math.round(job.fit_score)}%
                                </Badge>
                              )}
                            </div>
                            <a
                              href={job.url}
                              target="_blank"
                              rel="noreferrer"
                              className="group flex items-center gap-2 truncate text-sm text-fg hover:text-accent mb-2"
                              title={job.title}
                            >
                              <span className="truncate">
                                {job.title} <span className="text-fg-muted">@ {job.company}</span>
                              </span>
                              <ExternalLink size={12} className="shrink-0 opacity-0 transition group-hover:opacity-100" />
                            </a>
                            <div className="text-xs text-fg-subtle">
                              {job.source}
                            </div>
                          </div>
                        </div>
                      </motion.li>
                    ))}
                  </ul>
                )}
              </>
            )}

            {/* Applied Jobs */}
            {filter === "applied" && (
              <>
                {appJobs.length === 0 ? (
                  <Empty>{t("applications.appliedEmpty")}</Empty>
                ) : (
                  <ul className="divide-y divide-border -mx-6 -mb-5">
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
                                className="group flex items-center gap-2 truncate text-sm text-fg hover:text-accent mb-2"
                                title={job.title}
                              >
                                <span className="truncate">
                                  {job.title} <span className="text-fg-muted">@ {job.company}</span>
                                </span>
                                <ExternalLink size={12} className="shrink-0 opacity-0 transition group-hover:opacity-100" />
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
                                    <select
                                      value={app.stage ?? "applied"}
                                      onChange={(e) => {
                                        post(`/applications/${app.id}/track`, {
                                          stage: e.target.value,
                                          notes: noteValue || null,
                                        })
                                          .then(() => invalidate())
                                          .catch((e) => alert(String(e)));
                                      }}
                                      className="rounded-lg border border-border bg-surface px-3 py-1.5 text-xs text-fg outline-none transition focus:border-accent/40 focus:ring-1 focus:ring-accent/25"
                                    >
                                      <option value="applied">
                                        {t("applications.stage.applied")}
                                      </option>
                                      <option value="screening">
                                        {t("applications.stage.screening")}
                                      </option>
                                      <option value="interview">
                                        {t("applications.stage.interview")}
                                      </option>
                                      <option value="offer">
                                        {t("applications.stage.offer")}
                                      </option>
                                      <option value="rejected">
                                        {t("applications.stage.rejected")}
                                      </option>
                                    </select>
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
                                            setEditingNotes((prev) => {
                                              const next = { ...prev };
                                              delete next[app.id];
                                              return next;
                                            });
                                          })
                                          .catch((e) => alert(String(e)));
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
