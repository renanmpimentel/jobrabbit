import { useState } from "react";
import { useTranslation } from "react-i18next";
import Markdown from "react-markdown";
import { Download, Copy, Check, Plus } from "lucide-react";
import { useCvReview, useCvVersion, useJobs, useInvalidate, post, type Keyword } from "../hooks";
import { Card, CardHeader, Button, Textarea, Empty, Select } from "../ui";
import { cn } from "../ui";
import { useToast } from "../toast";

// A single keyword chip: green when already in the CV, amber when missing.
function KeywordChip({ kw }: { kw: Keyword }) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs font-medium",
        kw.present ? "bg-success-tint text-success" : "bg-warn-tint text-warn",
      )}
    >
      {kw.present ? <Check size={11} /> : <Plus size={11} />}
      {kw.keyword}
    </span>
  );
}

function scoreColor(s: number): string {
  if (s >= 75) return "text-success";
  if (s >= 50) return "text-warn";
  return "text-danger";
}

// ATS score (0–100) as a compact ring/gauge — arc color follows the band.
function ScoreRing({ value }: { value: number }) {
  const radius = 26;
  const circ = 2 * Math.PI * radius;
  const pct = Math.max(0, Math.min(100, value)) / 100;
  return (
    <div
      className={cn("relative grid h-[72px] w-[72px] shrink-0 place-items-center", scoreColor(value))}
      role="img"
      aria-label={`ATS ${value}/100`}
    >
      <svg width="72" height="72" viewBox="0 0 72 72" className="-rotate-90">
        <circle cx="36" cy="36" r={radius} fill="none" stroke="rgb(var(--border))" strokeWidth="6" />
        <circle
          cx="36"
          cy="36"
          r={radius}
          fill="none"
          stroke="currentColor"
          strokeWidth="6"
          strokeLinecap="round"
          strokeDasharray={`${circ * pct} ${circ}`}
        />
      </svg>
      <div className="absolute text-center leading-none">
        <div className="text-base font-semibold tabular-nums text-fg">{value}</div>
        <div className="mt-0.5 text-[10px] text-fg-subtle">/100</div>
      </div>
    </div>
  );
}

type Mode = "general" | "job" | "paste";

export default function Ats() {
  const { t } = useTranslation();
  const review = useCvReview();
  const version = useCvVersion();
  const jobs = useJobs();
  const invalidate = useInvalidate();
  const toast = useToast();

  const [mode, setMode] = useState<Mode>("general");
  const [jobId, setJobId] = useState<number | "">("");
  const [pasted, setPasted] = useState("");
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);

  // Builds target string based on selected mode.
  function currentTarget(): string | undefined {
    if (mode === "job") {
      const j = jobs.data?.find((x) => x.id === jobId);
      return j ? `${j.title} @ ${j.company}\n${j.description}` : undefined;
    }
    if (mode === "paste") return pasted.trim() || undefined;
    return undefined; // general
  }

  async function run(path: string) {
    if (mode === "job" && jobId === "") return;
    if (mode === "paste" && !pasted.trim()) return;
    setBusy(true);
    try {
      await post(path, { target: currentTarget() });
      invalidate();
      toast.success(t("common.started"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  }

  // Regenerates the improved CV, weaving in the missing keywords (truthfully).
  async function applyKeywords() {
    const missing = (review.data?.keywords ?? []).filter((k) => !k.present).map((k) => k.keyword);
    if (missing.length === 0) return;
    setBusy(true);
    try {
      await post("/cv-improve/run", { target: currentTarget(), emphasize_keywords: missing });
      invalidate();
      toast.success(t("common.started"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function copy() {
    if (!version.data) return;
    await navigator.clipboard.writeText(version.data.content);
    setCopied(true);
    toast.success(t("ats.copied"));
    setTimeout(() => setCopied(false), 1500);
  }

  const r = review.data;
  const v = version.data;

  return (
    <div className="grid items-start gap-5 lg:grid-cols-3">
      <div className="space-y-5 lg:col-span-2">
        <Card>
          <CardHeader
            title={t("ats.title")}
            hint={r ? `target: ${r.target}` : t("ats.notEvaluated")}
            right={r ? <ScoreRing value={r.score} /> : undefined}
          />
          <div className="p-6">
            {!r ? (
              <Empty>{t("ats.noReview")}</Empty>
            ) : (
              <div className="md">
                <Markdown>{r.report}</Markdown>
              </div>
            )}
          </div>
        </Card>

        {r && r.keywords.length > 0 && (
          <Card>
            <CardHeader
              title={t("ats.keywordsTitle")}
              hint={t("ats.keywordsHint")}
              right={
                r.keywords.some((k) => !k.present) ? (
                  <Button variant="primary" size="sm" disabled={busy} onClick={applyKeywords}>
                    {t("ats.applyKeywords", { count: r.keywords.filter((k) => !k.present).length })}
                  </Button>
                ) : undefined
              }
            />
            <div className="space-y-3 p-5">
              {(["required", "preferred"] as const).map((imp) => {
                const items = r.keywords.filter((k) => k.importance === imp);
                if (!items.length) return null;
                return (
                  <div key={imp}>
                    <div className="mb-1.5 text-xs font-medium text-fg-muted">{t(`ats.importance.${imp}`)}</div>
                    <div className="flex flex-wrap gap-1.5">
                      {items.map((k, i) => (
                        <KeywordChip key={`${k.keyword}-${i}`} kw={k} />
                      ))}
                    </div>
                  </div>
                );
              })}
              <p className="text-xs text-fg-subtle">{t("ats.keywordsLegend")}</p>
            </div>
          </Card>
        )}

        <Card>
          <CardHeader
            title={t("ats.improvedCv")}
            hint={v ? `target: ${v.target}` : t("ats.improvedCvHint")}
            right={
              v ? (
                <div className="flex items-center gap-2">
                  <Button variant="ghost" onClick={copy}>
                    {copied ? <Check size={15} /> : <Copy size={15} />} {copied ? t("ats.copied") : t("ats.copy")}
                  </Button>
                  <span className="flex items-center overflow-hidden rounded-md border border-border">
                    <span className="bg-surface-2 px-2.5 py-1.5 text-xs text-fg-muted">
                      <Download size={14} className="inline" /> {t("ats.download")}
                    </span>
                    <a href="/api/cv-version/download?format=pdf" download className="border-l border-border bg-accent px-3 py-1.5 text-sm font-semibold text-accent-fg hover:opacity-90">
                      PDF
                    </a>
                    <a href="/api/cv-version/download?format=docx" download className="border-l border-border bg-surface-2 px-3 py-1.5 text-sm text-fg hover:bg-border/40">
                      DOCX
                    </a>
                    <a href="/api/cv-version/download?format=md" download className="border-l border-border bg-surface-2 px-3 py-1.5 text-sm text-fg hover:bg-border/40">
                      .md
                    </a>
                  </span>
                </div>
              ) : undefined
            }
          />
          <div className="p-6">
            {!v ? (
              <Empty>{t("ats.noCvVersion")}</Empty>
            ) : (
              <div className="md max-h-[55vh] overflow-auto scroll-thin rounded-md border border-border bg-surface-2 p-4">
                <Markdown>{v.content}</Markdown>
              </div>
            )}
          </div>
        </Card>
      </div>

      <div className="lg:sticky lg:top-20">
        <Card>
          <CardHeader title={t("ats.targetAndActions")} />
          <div className="space-y-4 p-5">
            <div className="space-y-2">
              <label className="text-xs text-fg-muted">{t("ats.targetLabel")}</label>
              <div className="flex flex-col gap-1.5">
                {(["general", "job", "paste"] as Mode[]).map((m) => (
                  <button
                    key={m}
                    onClick={() => setMode(m)}
                    className={cn(
                      "rounded-md border px-3 py-1.5 text-left text-sm transition-colors",
                      mode === m
                        ? "border-border-strong bg-surface-2 font-medium text-fg"
                        : "border-border text-fg-muted hover:bg-surface-2 hover:text-fg",
                    )}
                  >
                    {m === "general" ? t("ats.generalQuality") : m === "job" ? t("ats.againstJob") : t("ats.pasteDescription")}
                  </button>
                ))}
              </div>
            </div>

            {mode === "job" && (
              <Select
                wrapperClassName="w-full"
                value={jobId}
                onChange={(e) => setJobId(e.target.value ? Number(e.target.value) : "")}
              >
                <option value="">{t("ats.selectJob")}</option>
                {jobs.data?.map((j) => (
                  <option key={j.id} value={j.id}>
                    {j.title} @ {j.company}
                  </option>
                ))}
              </Select>
            )}

            {mode === "paste" && (
              <Textarea rows={6} value={pasted} onChange={(e) => setPasted(e.target.value)} placeholder={t("ats.pasteJobPlaceholder")} />
            )}

            <div className="flex flex-col gap-2 border-t border-border pt-3">
              <Button className="w-full justify-center" disabled={busy} onClick={() => run("/cv-review/run")}>
                {t("ats.evaluate")}
              </Button>
              <Button
                variant="primary"
                className="w-full justify-center"
                disabled={busy}
                onClick={() => run("/cv-improve/run")}
              >
                {busy ? t("ats.generating") : t("ats.generateImproved")}
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
