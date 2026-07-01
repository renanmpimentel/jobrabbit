import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { AlertTriangle, XCircle, ChevronRight, ExternalLink } from "lucide-react";
import { useStats, useJobs, useDoctor } from "../hooks";
import { useNav } from "../nav";
import { Card, CardHeader, Empty, StatCard, ScoreBadge, SkeletonRows, ErrorState, cn } from "../ui";
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
        "flex w-full items-center gap-4 rounded-md border border-l-[3px] px-4 py-3 text-left transition-colors",
        critical
          ? "border-border border-l-danger bg-danger-tint hover:brightness-[0.99]"
          : "border-border border-l-warn bg-warn-tint hover:brightness-[0.99]",
      )}
    >
      <Icon size={18} className={critical ? "text-danger" : "text-warn"} />
      <div className="flex-1">
        <div className="text-sm font-medium text-fg">
          {critical
            ? t("dashboard.healthBannerFails", { count: fails })
            : t("dashboard.healthBannerWarnings", { count: warns })}
        </div>
        <div className="text-sm text-fg-muted">
          {problems.slice(0, 3).map((p) => p.name).join(" · ")}
          {problems.length > 3 ? " …" : ""}
        </div>
      </div>
      <span className={cn("flex items-center gap-1 text-sm", critical ? "text-danger" : "text-warn")}>
        {t("dashboard.viewHealth")} <ChevronRight size={14} />
      </span>
    </motion.button>
  );
}

export default function Dashboard() {
  const { t } = useTranslation();
  const stats = useStats();
  const jobs = useJobs();
  const s = stats.data;

  // Present ordered by fit (desc); nulls last. Presentation-only sort.
  const rankedJobs = [...(jobs.data ?? [])]
    .sort((a, b) => (b.fit_score ?? -1) - (a.fit_score ?? -1))
    .slice(0, 25);

  return (
    <motion.div variants={stagger} initial="hidden" animate="show" className="space-y-8">
      <HealthBanner />

      <motion.div variants={stagger} className="grid grid-cols-2 gap-4 sm:grid-cols-4">
        <StatCard label={t("dashboard.jobsFound")} value={s?.total_jobs ?? 0} />
        <StatCard label={t("dashboard.applications")} value={s?.total_applications ?? 0} />
        <StatCard label={t("dashboard.applied")} value={s?.applied ?? 0} />
        <StatCard label={t("dashboard.pendingActions")} value={s?.pending_actions ?? 0} />
      </motion.div>

      <motion.div variants={fadeUp}>
        <Card>
          <CardHeader title={t("dashboard.recentJobs")} hint={t("dashboard.recentJobsHint")} />
          {jobs.isLoading ? (
            <SkeletonRows rows={6} />
          ) : jobs.isError ? (
            <ErrorState message={t("common.error")} retryLabel={t("common.retry")} onRetry={() => jobs.refetch()} />
          ) : !jobs.data?.length ? (
            <Empty>{t("dashboard.noJobs")}</Empty>
          ) : (
            <ul className="divide-y divide-border">
              {rankedJobs.map((j) => (
                <li key={j.id} className="group flex items-center gap-4 px-4 py-3 transition-colors hover:bg-surface-2">
                  <ScoreBadge value={j.fit_score} />
                  <a
                    href={j.url}
                    target="_blank"
                    rel="noreferrer"
                    className="min-w-0 flex-1"
                    title={j.title}
                  >
                    <div className="flex items-center gap-1.5">
                      <span className="truncate text-sm font-medium leading-snug text-fg transition-colors group-hover:text-accent">
                        {j.title}
                      </span>
                      <ExternalLink size={12} className="shrink-0 text-fg-subtle opacity-0 transition group-hover:opacity-100" />
                    </div>
                    <div className="truncate text-sm text-fg-subtle">{j.company}</div>
                  </a>
                  {j.source && <span className="hidden font-mono text-xs text-fg-subtle sm:inline">{j.source}</span>}
                </li>
              ))}
            </ul>
          )}
        </Card>
      </motion.div>
    </motion.div>
  );
}
