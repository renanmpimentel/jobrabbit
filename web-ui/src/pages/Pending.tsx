import { useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { usePending, useInvalidate, post, type PendingAction } from "../hooks";
import { Card, CardHeader, Button, Badge, Empty, Input } from "../ui";
import { fadeUp, stagger } from "../motion";

const KIND: Record<string, { tone: string }> = {
  approval: { tone: "iris" },
  answer_needed: { tone: "yellow" },
  login: { tone: "red" },
  captcha: { tone: "red" },
  required_field: { tone: "yellow" },
};

function PendingItem({ p, onChange }: { p: PendingAction; onChange: () => void }) {
  const { t } = useTranslation();
  const [answer, setAnswer] = useState("");
  const [busy, setBusy] = useState(false);
  const meta = KIND[p.kind] ?? { tone: "slate" };
  const kindLabel = t(`pending.kind.${p.kind}`, p.kind);

  async function act(fn: () => Promise<unknown>) {
    setBusy(true);
    try {
      await fn();
      onChange();
    } catch (e) {
      alert(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <motion.li variants={fadeUp} className="px-5 py-3.5">
      <div className="flex items-center gap-2">
        <Badge tone={meta.tone}>{kindLabel}</Badge>
        <span className="flex-1 text-sm text-slate-100">{p.description}</span>
        {p.url && (
          <a href={p.url} target="_blank" rel="noreferrer" className="text-xs text-neon hover:underline">
            {t("pending.openJob")}
          </a>
        )}
      </div>
      <div className="mt-2 flex items-center gap-2">
        {p.kind === "approval" && (
          <Button variant="primary" disabled={busy} onClick={() => act(() => post(`/pending/${p.id}/approve`))}>
            {t("pending.approve")}
          </Button>
        )}
        {p.kind === "answer_needed" && (
          <>
            <Input
              value={answer}
              onChange={(e) => setAnswer(e.target.value)}
              placeholder={t("pending.answerPlaceholder")}
              className="max-w-xs"
            />
            <Button
              variant="primary"
              disabled={busy || !answer.trim()}
              onClick={() => act(() => post(`/pending/${p.id}/answer`, { value: answer }))}
            >
              {t("pending.answer")}
            </Button>
          </>
        )}
        <Button variant="ghost" disabled={busy} onClick={() => act(() => post(`/pending/${p.id}/resolve`))}>
          {t("pending.resolve")}
        </Button>
      </div>
    </motion.li>
  );
}

export default function Pending() {
  const { t } = useTranslation();
  const pending = usePending();
  const invalidate = useInvalidate();
  const items = pending.data ?? [];

  return (
    <Card>
      <CardHeader title={t("pending.title")} hint={t("pending.hint")} />
      {items.length === 0 ? (
        <Empty>{t("pending.noActions")}</Empty>
      ) : (
        <motion.ul variants={stagger} initial="hidden" animate="show" className="divide-y divide-edge">
          {items.map((p) => (
            <PendingItem key={p.id} p={p} onChange={invalidate} />
          ))}
        </motion.ul>
      )}
    </Card>
  );
}
