import { useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { AlertOctagon, HelpCircle, CheckCircle2, ExternalLink, ChevronRight, FileText } from "lucide-react";
import { usePending, useApplications, useInvalidate, post, type PendingAction, type Application } from "../hooks";
import {
  Card,
  CardHeader,
  Button,
  Input,
  Empty,
  Callout,
  SectionTitle,
  StatusBadge,
  SkeletonRows,
  ErrorState,
  cn,
} from "../ui";
import { fadeUp, stagger } from "../motion";
import { useToast } from "../toast";

type Tier = "blocking" | "decision" | "routine";

// Urgency mapping per DESIGN.md (presentation only — kinds/handlers unchanged):
// blocking = login/captcha/answer_needed (Pergunta); decision = required_field; routine = approval.
function tierOf(kind: string): Tier {
  if (kind === "login" || kind === "captcha" || kind === "answer_needed") return "blocking";
  if (kind === "approval") return "routine";
  return "decision"; // required_field and anything else needing a call
}

const TIER_ORDER: Tier[] = ["blocking", "decision", "routine"];

function useAct(onChange: () => void) {
  const { t } = useTranslation();
  const toast = useToast();
  const [busy, setBusy] = useState(false);
  async function act(fn: () => Promise<unknown>, successMsg?: string) {
    setBusy(true);
    try {
      await fn();
      onChange();
      toast.success(successMsg ?? t("common.done"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  }
  return { busy, act };
}

function OpenJobLink({ url }: { url: string }) {
  const { t } = useTranslation();
  return (
    <a
      href={url}
      target="_blank"
      rel="noreferrer"
      className="inline-flex items-center gap-1 text-sm text-accent hover:underline"
    >
      {t("pending.openJob")}
      <ExternalLink size={12} />
    </a>
  );
}

/// Blocking (login/captcha/answer) and decision (field) render as prominent callouts.
function EmphasizedItem({ p, tier, onChange }: { p: PendingAction; tier: Tier; onChange: () => void }) {
  const { t } = useTranslation();
  const [answer, setAnswer] = useState("");
  const { busy, act } = useAct(onChange);
  const Icon = tier === "blocking" ? AlertOctagon : HelpCircle;
  const kindLabel = t(`pending.kind.${p.kind}`, p.kind);

  return (
    <motion.li variants={fadeUp}>
      <Callout tone={tier === "blocking" ? "danger" : "info"} icon={<Icon size={18} />}>
        <div className="mb-1.5">
          <StatusBadge tier={tier}>{kindLabel}</StatusBadge>
        </div>
        <div className="text-sm text-fg">{p.description}</div>

        {p.kind === "answer_needed" && (
          <div className="mt-3 flex flex-wrap items-center gap-2">
            <Input
              value={answer}
              onChange={(e) => setAnswer(e.target.value)}
              placeholder={t("pending.answerPlaceholder")}
              aria-label={p.description}
              className="max-w-xs"
            />
            <Button
              variant="primary"
              size="sm"
              disabled={busy || !answer.trim()}
              onClick={() => act(() => post(`/pending/${p.id}/answer`, { value: answer }))}
            >
              {t("pending.answer")}
            </Button>
          </div>
        )}

        <div className="mt-3 flex items-center gap-3">
          {p.url && <OpenJobLink url={p.url} />}
          <Button variant="ghost" size="sm" disabled={busy} onClick={() => act(() => post(`/pending/${p.id}/resolve`))}>
            {t("pending.resolve")}
          </Button>
        </div>
      </Callout>
    </motion.li>
  );
}

/// Routine (approval) — compact row + expandable review of the proposed CV/cover
/// (the content the agent will fill on the site). Approve = strong; Resolve = ghost.
function RoutineItem({ p, app, onChange }: { p: PendingAction; app?: Application; onChange: () => void }) {
  const { t } = useTranslation();
  const { busy, act } = useAct(onChange);
  const [open, setOpen] = useState(false);
  const kindLabel = t(`pending.kind.${p.kind}`, p.kind);
  const hasContent = !!(app?.cv_generated || app?.cover_letter);

  return (
    <motion.li variants={fadeUp} className="px-4 py-2.5">
      <div className="flex items-center gap-3">
        <StatusBadge tier="routine">{kindLabel}</StatusBadge>
        <span className="min-w-0 flex-1 truncate text-sm text-fg">{p.description}</span>
        {hasContent && (
          <button
            onClick={() => setOpen((o) => !o)}
            aria-expanded={open}
            className="inline-flex items-center gap-1 text-xs text-fg-muted transition hover:text-fg"
          >
            <ChevronRight size={13} className={cn("transition-transform", open && "rotate-90")} />
            {t("pending.reviewContent")}
          </button>
        )}
        {p.url && <OpenJobLink url={p.url} />}
        <Button variant="primary" size="sm" disabled={busy} onClick={() => act(() => post(`/pending/${p.id}/approve`))}>
          {t("pending.approve")}
        </Button>
        <Button variant="ghost" size="sm" disabled={busy} onClick={() => act(() => post(`/pending/${p.id}/resolve`))}>
          {t("pending.resolve")}
        </Button>
      </div>

      {open && hasContent && (
        <div className="mt-3 space-y-3 rounded-md border border-border bg-surface-2 p-3">
          {app?.cv_generated && (
            <div>
              <div className="mb-1 flex items-center gap-1.5 text-xs font-medium text-fg-muted">
                <FileText size={13} /> {t("pending.cvLabel")}
              </div>
              <pre className="scroll-thin max-h-56 overflow-auto whitespace-pre-wrap font-sans text-sm leading-relaxed text-fg">
                {app.cv_generated}
              </pre>
            </div>
          )}
          {app?.cover_letter && (
            <div>
              <div className="mb-1 text-xs font-medium text-fg-muted">{t("pending.coverLabel")}</div>
              <pre className="scroll-thin max-h-56 overflow-auto whitespace-pre-wrap font-sans text-sm leading-relaxed text-fg">
                {app.cover_letter}
              </pre>
            </div>
          )}
        </div>
      )}
    </motion.li>
  );
}

function TierSection({
  tier,
  items,
  appByJob,
  onChange,
}: {
  tier: Tier;
  items: PendingAction[];
  appByJob: Map<number, Application>;
  onChange: () => void;
}) {
  const { t } = useTranslation();
  if (items.length === 0) return null;

  return (
    <section aria-label={t(`pending.tiers.${tier}`)}>
      <SectionTitle count={items.length}>{t(`pending.tiers.${tier}`)}</SectionTitle>
      <p className="mb-3 text-sm text-fg-subtle">{t(`pending.tiers.${tier}Hint`)}</p>

      {tier === "routine" ? (
        <Card>
          <motion.ul variants={stagger} initial="hidden" animate="show" className="divide-y divide-border">
            {items.map((p) => (
              <RoutineItem key={p.id} p={p} app={p.job_id != null ? appByJob.get(p.job_id) : undefined} onChange={onChange} />
            ))}
          </motion.ul>
        </Card>
      ) : (
        <motion.ul variants={stagger} initial="hidden" animate="show" className="space-y-2.5">
          {items.map((p) => (
            <EmphasizedItem key={p.id} p={p} tier={tier} onChange={onChange} />
          ))}
        </motion.ul>
      )}
    </section>
  );
}

export default function Pending() {
  const { t } = useTranslation();
  const pending = usePending();
  const applications = useApplications();
  const invalidate = useInvalidate();
  const appByJob = new Map((applications.data ?? []).map((a) => [a.job_id, a] as const));

  if (pending.isLoading) {
    return (
      <Card>
        <CardHeader title={t("pending.title")} hint={t("pending.hint")} />
        <SkeletonRows rows={4} />
      </Card>
    );
  }

  if (pending.isError) {
    return (
      <Card>
        <CardHeader title={t("pending.title")} />
        <ErrorState message={t("pending.error")} retryLabel={t("pending.retry")} onRetry={() => pending.refetch()} />
      </Card>
    );
  }

  const items = pending.data ?? [];

  if (items.length === 0) {
    return (
      <Card>
        <CardHeader title={t("pending.title")} hint={t("pending.hint")} />
        <Empty icon={<CheckCircle2 size={26} className="text-success" />} hint={t("pending.noActionsHint")}>
          {t("pending.noActions")}
        </Empty>
      </Card>
    );
  }

  const grouped: Record<Tier, PendingAction[]> = { blocking: [], decision: [], routine: [] };
  for (const p of items) grouped[tierOf(p.kind)].push(p);

  return (
    <div className="space-y-8">
      {TIER_ORDER.map((tier) => (
        <TierSection key={tier} tier={tier} items={grouped[tier]} appByJob={appByJob} onChange={invalidate} />
      ))}
    </div>
  );
}
