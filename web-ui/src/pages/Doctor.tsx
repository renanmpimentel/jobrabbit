import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { CheckCircle2, AlertTriangle, XCircle, RefreshCw } from "lucide-react";
import { useDoctor, type CheckStatus } from "../hooks";
import { Card, CardHeader, Button, Empty } from "../ui";
import { cn } from "../ui";
import { fadeUp, stagger } from "../motion";

const META: Record<CheckStatus, { icon: typeof CheckCircle2; color: string; ring: string }> = {
  Ok: { icon: CheckCircle2, color: "text-neon", ring: "border-neon/30" },
  Warn: { icon: AlertTriangle, color: "text-warn", ring: "border-warn/30" },
  Fail: { icon: XCircle, color: "text-danger", ring: "border-danger/40" },
};

export default function Doctor() {
  const { t } = useTranslation();
  const doctor = useDoctor();
  const checks = doctor.data ?? [];
  const counts = checks.reduce(
    (a, c) => ((a[c.status] = (a[c.status] ?? 0) + 1), a),
    {} as Record<CheckStatus, number>,
  );

  return (
    <div className="mx-auto max-w-3xl space-y-4">
      <Card>
        <CardHeader
          title={t("doctor.title")}
          hint={t("doctor.hint")}
          right={
            <div className="flex items-center gap-3">
              <span className="font-mono text-xs text-fg-muted">
                <span className="text-neon">{counts.Ok ?? 0} {t("doctor.ok")}</span> ·{" "}
                <span className="text-warn">{t("doctor.warnings", { count: counts.Warn ?? 0 })}</span> ·{" "}
                <span className="text-danger">{t("doctor.errors", { count: counts.Fail ?? 0 })}</span>
              </span>
              <Button onClick={() => doctor.refetch()} disabled={doctor.isFetching}>
                <RefreshCw size={14} className={cn(doctor.isFetching && "animate-spin")} /> {t("doctor.refresh")}
              </Button>
            </div>
          }
        />
        {checks.length === 0 ? (
          <Empty>{doctor.isLoading ? t("doctor.checking") : t("doctor.noResults")}</Empty>
        ) : (
          <motion.ul variants={stagger} initial="hidden" animate="show" className="divide-y divide-edge">
            {checks.map((c) => {
              const m = META[c.status];
              const Icon = m.icon;
              return (
                <motion.li variants={fadeUp} key={c.name} className="flex gap-3 px-5 py-3.5">
                  <Icon size={18} className={cn("mt-0.5 flex-shrink-0", m.color)} />
                  <div className="flex-1">
                    <div className="text-sm font-medium text-fg">{c.name}</div>
                    <div className="text-sm text-fg-muted">{c.detail}</div>
                    {c.hint && (
                      <div className={cn("mt-1.5 rounded-lg border bg-ink-850 px-2.5 py-1.5 text-xs text-fg-muted", m.ring)}>
                        ↳ {c.hint}
                      </div>
                    )}
                  </div>
                </motion.li>
              );
            })}
          </motion.ul>
        )}
      </Card>
    </div>
  );
}
