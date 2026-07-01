import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useFeedback, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Button, Empty } from "../ui";

export default function FeedbackPage() {
  const { t } = useTranslation();
  const feedback = useFeedback();
  const invalidate = useInvalidate();
  const [busy, setBusy] = useState(false);

  async function generate() {
    setBusy(true);
    try {
      await post("/feedback/run");
      invalidate();
    } catch (e) {
      alert(String(e));
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
      {!feedback.data?.length ? (
        <Empty>{t("feedback.noFeedback")}</Empty>
      ) : (
        <ul className="divide-y divide-border">
          {feedback.data.map((f) => (
            <li key={f.id} className="px-4 py-3">
              <div className="text-sm font-medium text-fg">{f.summary}</div>
              <pre className="mt-1 whitespace-pre-wrap font-sans text-xs text-fg-muted">{f.suggestions}</pre>
              <div className="mt-1 text-[10px] text-fg-muted">{f.created_at}</div>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
