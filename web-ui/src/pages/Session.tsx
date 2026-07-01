import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { AlertOctagon } from "lucide-react";
import { useAgent } from "../events";
import { useInvalidate, usePending, post } from "../hooks";
import { useNav } from "../nav";
import { Card, CardHeader, Button, Callout, cn, Textarea } from "../ui";
import { useToast } from "../toast";

// Pendings that literally block the agent (vs. routine approvals).
const BLOCKING_KINDS = ["login", "captcha", "answer_needed", "required_field"];

const TOOL_ICONS = ["🔧", "🌐", "📸", "🖱️", "⌨️", "👀", "📎", "🔎", "💻", "📄", "✏️", "↩️", "🐛", "📡", "🗂️"];

function lineColor(l: string): string {
  if (l.startsWith("✖")) return "text-danger";
  if (l.startsWith("✔") || l.startsWith("✅")) return "text-accent";
  if (TOOL_ICONS.some((i) => l.startsWith(i))) return "text-warn";
  if (l.startsWith("▶") || l.startsWith("—") || l.startsWith("＋")) return "text-info";
  if (l.startsWith("📋") || l.startsWith("📊") || l.startsWith("👤") || l.startsWith("🗂") || l.startsWith("✨"))
    return "text-info";
  if (l.startsWith("⚠") || l.startsWith("ℹ") || l.startsWith("⏾")) return "text-fg-muted";
  return "text-fg";
}

export default function Session() {
  const { t } = useTranslation();
  const { log, connected, status } = useAgent();
  const invalidate = useInvalidate();
  const toast = useToast();
  const nav = useNav();
  const pending = usePending();
  const [follow, setFollow] = useState(true);
  const [filter, setFilter] = useState("");
  const [cmd, setCmd] = useState("");
  const [busy, setBusy] = useState(false);
  const boxRef = useRef<HTMLDivElement>(null);

  async function handleContinue() {
    setBusy(true);
    try {
      await post("/session/continue", { message: cmd });
      setCmd("");
      invalidate();
      toast.success(t("common.sent"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  }

  const lines = useMemo(
    () => (filter ? log.filter((l) => l.toLowerCase().includes(filter.toLowerCase())) : log),
    [log, filter],
  );

  useEffect(() => {
    if (follow && boxRef.current) boxRef.current.scrollTop = boxRef.current.scrollHeight;
  }, [lines, follow]);

  // Follow tracks whether the user is at the bottom: scrolling up pauses
  // auto-scroll, scrolling back to the bottom resumes it.
  const onScroll = () => {
    const el = boxRef.current;
    if (!el) return;
    setFollow(el.scrollHeight - el.scrollTop - el.clientHeight < 40);
  };

  const isRunning = status === "Running";

  // Banner: surface open pending actions right here, so a user watching the
  // execution log knows the agent is waiting on them without switching tabs.
  const openPendings = pending.data ?? [];
  const urgent = openPendings.find((p) => BLOCKING_KINDS.includes(p.kind)) ?? openPendings[0];
  const others = openPendings.length - 1;

  return (
    <div className="space-y-6">
      {urgent && (
        <Callout
          tone={BLOCKING_KINDS.includes(urgent.kind) ? "danger" : "warn"}
          icon={<AlertOctagon size={18} />}
          title={t("session.waitingTitle")}
          right={
            <Button variant="primary" size="sm" onClick={() => nav("pending")}>
              {t("session.goResolve")}
            </Button>
          }
        >
          <div className="text-sm text-fg">{urgent.description}</div>
          {others > 0 && (
            <div className="mt-1 text-xs text-fg-muted">{t("session.waitingMore", { count: others })}</div>
          )}
        </Callout>
      )}

      {/* Continue Box (shown only when not running) */}
      {!isRunning && (
        <Card>
          <CardHeader title={t("session.continueTitle")} />
          <div className="space-y-3 px-5 py-4">
            <Textarea
              value={cmd}
              onChange={(e) => setCmd(e.target.value)}
              placeholder={t("session.continuePlaceholder")}
              rows={3}
              disabled={busy}
            />
            <p className="text-xs text-fg-muted">{t("session.continueHint")}</p>
            <Button
              variant="primary"
              disabled={busy}
              onClick={handleContinue}
              className="w-full justify-center"
            >
              {t("session.continueButton")}
            </Button>
          </div>
        </Card>
      )}

      {/* Terminal Box */}
      <Card className="overflow-hidden">
        <CardHeader
          title={t("session.title")}
          hint={connected ? t("session.liveHint") : t("session.notConnected")}
          right={
            <div className="flex items-center gap-2">
              <span className="flex items-center gap-1.5 font-mono text-[11px] text-fg-muted">
                <span className={cn("h-1.5 w-1.5 rounded-full", connected ? "bg-accent animate-pulse" : "bg-fg-subtle")} />
                {connected ? t("ui.live") : t("ui.off")}
              </span>
              <input
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                placeholder={t("session.filterPlaceholder")}
                className="w-32 rounded-md border border-border bg-surface px-2 py-1 font-mono text-xs text-fg outline-none transition hover:border-border-strong focus:border-accent focus-visible:ring-2 focus-visible:ring-accent/30"
              />
              <Button variant={follow ? "primary" : "subtle"} onClick={() => setFollow((f) => !f)}>
                {follow ? t("session.following") : t("session.paused")}
              </Button>
            </div>
          }
        />
        <div className="relative">
          <div
            ref={boxRef}
            onScroll={onScroll}
            className="scroll-thin relative h-[62vh] overflow-auto bg-surface-2 px-5 py-4 font-mono text-xs leading-relaxed"
          >
            {lines.length === 0 ? (
              <div className="text-fg-subtle">
                <span className="text-accent">›</span> {t("session.noActivity")}
              </div>
            ) : (
              lines.map((l, i) => (
                <div key={i} className={cn("whitespace-pre-wrap break-words", lineColor(l))}>
                  {l}
                </div>
              ))
            )}
          </div>
        </div>
      </Card>
    </div>
  );
}
