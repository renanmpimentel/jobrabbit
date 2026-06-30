import { useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { Send, ExternalLink, Loader2, Image } from "lucide-react";
import { useStats, useApplications, useJobs, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Badge, Empty, Input, Button, StatCard, cn } from "../ui";
import { fadeUp, stagger } from "../motion";

function statusTone(status: string): string {
  if (status === "applied") return "green";
  if (status === "dry_run") return "yellow";
  if (status === "ready" || status === "pending") return "yellow";
  return "red";
}

export default function Applications() {
  const { t } = useTranslation();
  const stats = useStats();
  const applications = useApplications();
  const jobs = useJobs();
  const invalidate = useInvalidate();

  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);

  const apps = applications.data ?? [];
  const jobsMap = new Map((jobs.data ?? []).map((j) => [j.id, j]));

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
    <motion.div variants={stagger} initial="hidden" animate="show" className="space-y-6">
      {/* Stats */}
      <motion.div variants={stagger} className="grid grid-cols-2 gap-3 sm:grid-cols-2">
        <StatCard label={t("applications.totalLabel")} value={stats.data?.total_applications ?? 0} />
        <StatCard label={t("applications.appliedLabel")} value={stats.data?.applied ?? 0} />
      </motion.div>

      {/* Apply by URL Card */}
      <motion.div variants={fadeUp}>
        <Card>
          <CardHeader title={t("applications.title")} />
          <div className="space-y-3 px-5 py-4">
            <div className="flex items-center gap-2">
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
                    <Loader2 size={15} className="animate-spin" />
                    <span className="hidden sm:inline">{t("applications.applying")}</span>
                  </>
                ) : (
                  <>
                    <Send size={15} />
                    <span className="hidden sm:inline">{t("applications.applyButton")}</span>
                  </>
                )}
              </Button>
            </div>
            <p className="text-xs text-fg-muted">{t("applications.hint")}</p>
          </div>
        </Card>
      </motion.div>

      {/* Applications List */}
      <motion.div variants={fadeUp}>
        <Card>
          <CardHeader title={t("applications.listTitle")} />
          {apps.length === 0 ? (
            <Empty>{t("applications.empty")}</Empty>
          ) : (
            <ul className="divide-y divide-edge">
              {apps.map((app) => {
                const job = jobsMap.get(app.job_id);
                return (
                  <motion.li
                    key={app.id}
                    variants={fadeUp}
                    className="px-5 py-3.5 transition hover:bg-white/[0.02]"
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <Badge tone={statusTone(app.status)}>
                            {t(`applications.status.${app.status}`, app.status)}
                          </Badge>
                          {app.screenshot_path && (
                            <a
                              href={`/api/screenshot/${app.id}`}
                              target="_blank"
                              rel="noreferrer"
                              className="text-fg-muted hover:text-neon transition"
                              title={t("applications.viewProof")}
                            >
                              <Image size={14} />
                            </a>
                          )}
                        </div>
                        {job ? (
                          <a
                            href={job.url}
                            target="_blank"
                            rel="noreferrer"
                            className="group flex items-center gap-1.5 truncate text-sm text-fg hover:text-neon mb-1"
                            title={job.title}
                          >
                            <span className="truncate">
                              {job.title} <span className="text-fg-muted">@ {job.company}</span>
                            </span>
                            <ExternalLink size={12} className="shrink-0 opacity-0 transition group-hover:opacity-100" />
                          </a>
                        ) : (
                          <div className="text-sm text-fg-muted">{t("applications.jobNotFound")}</div>
                        )}
                        <div className="text-xs text-fg-dim mt-1">
                          {new Date(app.created_at).toLocaleDateString(undefined, {
                            year: "numeric",
                            month: "short",
                            day: "numeric",
                          })}
                        </div>
                      </div>
                    </div>
                  </motion.li>
                );
              })}
            </ul>
          )}
        </Card>
      </motion.div>
    </motion.div>
  );
}
