import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useFeedback, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Button, Empty, SkeletonRows, ErrorState } from "../ui";
import { useToast } from "../toast";

function fmtDate(s: string): string {
  const d = new Date(s);
  return isNaN(d.getTime()) ? s : d.toLocaleString();
}

export default function FeedbackPage() {
  const { t } = useTranslation();
  const feedback = useFeedback();
  const invalidate = useInvalidate();
  const toast = useToast();
  const [busy, setBusy] = useState(false);

  async function generate() {
    setBusy(true);
    try {
      await post("/feedback/run");
      invalidate();
      toast.success(t("common.started"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card>
      <CardHeader
        title={t("feedback.title")}
        hint={t("feedback.hint")}
        right={
          <Button variant="primary" disabled={busy} onClick={generate}>
            {busy ? t("feedback.generating") : t("feedback.generateAnalysis")}
          </Button>
        }
      />
      {feedback.isLoading ? (
        <SkeletonRows rows={3} />
      ) : feedback.isError ? (
        <ErrorState message={t("common.error")} retryLabel={t("common.retry")} onRetry={() => feedback.refetch()} />
      ) : !feedback.data?.length ? (
        <Empty>{t("feedback.noFeedback")}</Empty>
      ) : (
        <ul className="divide-y divide-border">
          {feedback.data.map((f) => (
            <li key={f.id} className="px-5 py-4">
              <div className="text-sm font-medium text-fg">{f.summary}</div>
              <pre className="mt-1.5 whitespace-pre-wrap font-sans text-sm leading-relaxed text-fg-muted">{f.suggestions}</pre>
              <div className="mt-2 font-mono text-[11px] text-fg-subtle">{fmtDate(f.created_at)}</div>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
