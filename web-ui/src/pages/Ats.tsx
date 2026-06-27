import { useState } from "react";
import { useTranslation } from "react-i18next";
import Markdown from "react-markdown";
import { Download, Copy, Check } from "lucide-react";
import { useCvReview, useCvVersion, useJobs, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Button, Textarea, Empty } from "../ui";
import { cn } from "../ui";

function scoreColor(s: number): string {
  if (s >= 75) return "text-emerald-400";
  if (s >= 50) return "text-amber-300";
  return "text-red-400";
}

type Mode = "general" | "job" | "paste";

export default function Ats() {
  const { t } = useTranslation();
  const review = useCvReview();
  const version = useCvVersion();
  const jobs = useJobs();
  const invalidate = useInvalidate();

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
    } catch (e) {
      alert(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function copy() {
    if (!version.data) return;
    await navigator.clipboard.writeText(version.data.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  const r = review.data;
  const v = version.data;

  return (
    <div className="grid gap-4 lg:grid-cols-3">
      <div className="space-y-4 lg:col-span-2">
        <Card>
          <CardHeader
            title={t("ats.title")}
            hint={r ? `target: ${r.target}` : t("ats.notEvaluated")}
            right={
              r ? <span className={cn("text-2xl font-bold", scoreColor(r.score))}>{r.score}/100</span> : undefined
            }
          />
          <div className="p-4">
            {!r ? (
              <Empty>{t("ats.noReview")}</Empty>
            ) : (
              <div className="md text-sm text-slate-200">
                <Markdown>{r.report}</Markdown>
              </div>
            )}
          </div>
        </Card>

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
                  <span className="flex items-center overflow-hidden rounded-xl border border-edge">
                    <span className="bg-white/[0.04] px-2.5 py-1.5 text-xs text-fg-muted">
                      <Download size={14} className="inline" /> {t("ats.download")}
                    </span>
                    <a href="/api/cv-version/download?format=pdf" download className="border-l border-edge bg-neon px-3 py-1.5 text-sm font-semibold text-ink-900 hover:bg-neon-dim">
                      PDF
                    </a>
                    <a href="/api/cv-version/download?format=docx" download className="border-l border-edge bg-white/[0.04] px-3 py-1.5 text-sm text-fg hover:bg-white/[0.08]">
                      DOCX
                    </a>
                    <a href="/api/cv-version/download?format=md" download className="border-l border-edge bg-white/[0.04] px-3 py-1.5 text-sm text-fg hover:bg-white/[0.08]">
                      .md
                    </a>
                  </span>
                </div>
              ) : undefined
            }
          />
          <div className="p-4">
            {!v ? (
              <Empty>{t("ats.noCvVersion")}</Empty>
            ) : (
              <div className="md max-h-[55vh] overflow-auto scroll-thin rounded-xl border border-edge bg-ink-900/50 p-4 text-sm text-fg">
                <Markdown>{v.content}</Markdown>
              </div>
            )}
          </div>
        </Card>
      </div>

      <div>
        <Card>
          <CardHeader title={t("ats.targetAndActions")} />
          <div className="space-y-4 p-4">
            <div className="space-y-2">
              <label className="text-xs text-fg-muted">{t("ats.targetLabel")}</label>
              <div className="flex flex-col gap-1.5">
                {(["general", "job", "paste"] as Mode[]).map((m) => (
                  <button
                    key={m}
                    onClick={() => setMode(m)}
                    className={cn(
                      "rounded-lg border px-3 py-1.5 text-left text-sm transition",
                      mode === m ? "border-neon/50 bg-neon/10 text-neon" : "border-edge text-fg-muted hover:bg-white/[0.04]",
                    )}
                  >
                    {m === "general" ? t("ats.generalQuality") : m === "job" ? t("ats.againstJob") : t("ats.pasteDescription")}
                  </button>
                ))}
              </div>
            </div>

            {mode === "job" && (
              <select
                value={jobId}
                onChange={(e) => setJobId(e.target.value ? Number(e.target.value) : "")}
                className="w-full rounded-xl border border-edge bg-ink-850 px-3 py-2 text-sm text-fg"
              >
                <option value="">{t("ats.selectJob")}</option>
                {jobs.data?.map((j) => (
                  <option key={j.id} value={j.id}>
                    {j.title} @ {j.company}
                  </option>
                ))}
              </select>
            )}

            {mode === "paste" && (
              <Textarea rows={6} value={pasted} onChange={(e) => setPasted(e.target.value)} placeholder={t("ats.pasteJobPlaceholder")} />
            )}

            <div className="flex flex-col gap-2 border-t border-edge pt-3">
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
