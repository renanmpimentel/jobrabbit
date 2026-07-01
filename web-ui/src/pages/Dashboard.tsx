import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { AlertTriangle, XCircle, ChevronRight, ExternalLink } from "lucide-react";
import { useStats, useJobs, useDoctor } from "../hooks";
import { useNav } from "../nav";
import { Card, CardHeader, Badge, Empty, StatCard, cn } from "../ui";
import { fadeUp, stagger } from "../motion";

function HealthBanner() {
  const { t } = useTranslation();
  const doctor = useDoctor();
  const nav = useNav();
  const problems = (doctor.data ?? []).filter((c) => c.status !== "Ok");
  if (problems.length === 0) return null;

  const fails = problems.filter((c) => c.status === "Fail").length;
  const warns = problems.length - fails;
  const critical = fails > 0;
  const Icon = critical ? XCircle : AlertTriangle;

  return (
    <motion.button
      variants={fadeUp}
      onClick={() => nav("doctor")}
      className={cn(
        "flex w-full items-center gap-3 rounded-2xl border px-4 py-3 text-left transition",
        critical
          ? "border-danger/40 bg-danger/10 hover:bg-danger/15"
          : "border-warn/40 bg-warn/10 hover:bg-warn/15",
      )}
    >
      <Icon size={20} className={critical ? "text-danger" : "text-warn"} />
      <div className="flex-1">
        <div className="text-sm font-medium text-fg">
          {critical
            ? t("dashboard.healthBannerFails", { count: fails })
            : t("dashboard.healthBannerWarnings", { count: warns })}
        </div>
        <div className="text-xs text-fg-muted">
          {problems.slice(0, 3).map((p) => p.name).join(" · ")}
          {problems.length > 3 ? " …" : ""}
        </div>
      </div>
      <span className={cn("flex items-center gap-1 text-xs", critical ? "text-danger" : "text-warn")}>
        {t("dashboard.viewHealth")} <ChevronRight size={14} />
      </span>
    </motion.button>
  );
}

function fitTone(fit: number | null): string {
  if (fit == null) return "slate";
  if (fit >= 0.75) return "green";
  if (fit >= 0.5) return "yellow";
  return "red";
}

export default function Dashboard() {
  const { t } = useTranslation();
  const stats = useStats();
  const jobs = useJobs();
  const s = stats.data;

  return (
    <motion.div variants={stagger} initial="hidden" animate="show" className="space-y-6">
      <HealthBanner />

      <motion.div variants={stagger} className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <StatCard label={t("dashboard.jobsFound")} value={s?.total_jobs ?? 0} />
        <StatCard label={t("dashboard.applications")} value={s?.total_applications ?? 0} />
        <StatCard label={t("dashboard.applied")} value={s?.applied ?? 0} />
        <StatCard label={t("dashboard.pendingActions")} value={s?.pending_actions ?? 0} />
      </motion.div>

      <motion.div variants={fadeUp}>
        <Card>
          <CardHeader title={t("dashboard.recentJobs")} hint={t("dashboard.recentJobsHint")} />
          {!jobs.data?.length ? (
            <Empty>{t("dashboard.noJobs")}</Empty>
          ) : (
            <ul className="divide-y divide-border">
              {jobs.data.slice(0, 25).map((j) => (
                <li key={j.id} className="flex items-center gap-3 px-5 py-2.5 transition hover:bg-surface-2">
                  <Badge tone={fitTone(j.fit_score)}>
                    {j.fit_score != null ? j.fit_score.toFixed(2) : "—"}
                  </Badge>
                  <a
                    href={j.url}
                    target="_blank"
                    rel="noreferrer"
                    className="group flex flex-1 items-center gap-1.5 truncate text-sm text-fg hover:text-accent"
                    title={j.title}
                  >
                    <span className="truncate">
                      {j.title} <span className="text-fg-muted">@ {j.company}</span>
                    </span>
                    <ExternalLink size={12} className="shrink-0 opacity-0 transition group-hover:opacity-100" />
                  </a>
                  {j.source && <span className="font-mono text-[11px] text-fg-subtle">{j.source}</span>}
                </li>
              ))}
            </ul>
          )}
        </Card>
      </motion.div>
    </motion.div>
  );
}
