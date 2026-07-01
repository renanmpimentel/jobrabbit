import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAgent } from "../events";
import { useInvalidate, post } from "../hooks";
import { Card, CardHeader, Button, cn, Textarea } from "../ui";

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
    } catch (e) {
      alert(String(e));
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

  return (
    <div className="space-y-6">
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
                className="w-32 rounded-lg border border-border bg-surface px-2 py-1 font-mono text-xs text-fg outline-none focus:border-accent/50"
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
