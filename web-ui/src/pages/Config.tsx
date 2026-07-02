import { useEffect, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Loader2, Trash2 } from "lucide-react";
import { useSettings, useAnswers, useSources, useInvalidate, post, del, type Settings } from "../hooks";
import { LANGUAGES } from "../i18n";
import { Card, CardHeader, Button, Input, Toggle, Select, Badge, Empty, SkeletonRows, ErrorState } from "../ui";
import { useToast } from "../toast";

// Identity field keys (must match IDENTITY_FIELDS in src/db/models.rs) paired
// with their i18n label key.
const IDENTITY_FIELDS: { key: string; labelKey: string }[] = [
  { key: "full_name", labelKey: "identity.fullName" },
  { key: "cpf", labelKey: "identity.cpf" },
  { key: "phone", labelKey: "identity.phone" },
  { key: "birth_date", labelKey: "identity.birthDate" },
  { key: "city_state", labelKey: "identity.cityState" },
];

// Formats a birth date as DD/MM/AAAA while typing (digits only, max 8).
function maskBirthDate(v: string): string {
  const d = v.replace(/\D/g, "").slice(0, 8);
  if (d.length <= 2) return d;
  if (d.length <= 4) return `${d.slice(0, 2)}/${d.slice(2)}`;
  return `${d.slice(0, 2)}/${d.slice(2, 4)}/${d.slice(4)}`;
}

function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 px-4 py-3">
      <div>
        <div className="text-sm text-fg">{label}</div>
        {hint && <div className="text-xs text-fg-muted">{hint}</div>}
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

export default function Config() {
  const { t, i18n } = useTranslation();
  const settings = useSettings();
  const answers = useAnswers();
  const sources = useSources();
  const invalidate = useInvalidate();
  const toast = useToast();
  const [s, setS] = useState<Settings | null>(null);
  const [ident, setIdent] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [srcName, setSrcName] = useState("");
  const [srcDomain, setSrcDomain] = useState("");

  useEffect(() => {
    if (settings.data) setS(settings.data);
  }, [settings.data]);

  useEffect(() => {
    if (answers.data) {
      const next: Record<string, string> = {};
      for (const f of IDENTITY_FIELDS) {
        next[f.key] = answers.data.find((a) => a.key === f.key)?.value ?? "";
      }
      setIdent(next);
    }
  }, [answers.data]);

  if (!s) return <div className="text-sm text-fg-muted">{t("config.loading")}</div>;

  const set = <K extends keyof Settings>(k: K, v: Settings[K]) => setS({ ...s, [k]: v });

  // Saves both the settings and the personal-data (identity) answers in one go.
  const save = () => {
    setSaving(true);
    Promise.all([
      post("/settings", s),
      ...IDENTITY_FIELDS.map((f) => post("/answers", { key: f.key, value: ident[f.key] ?? "" })),
    ])
      .then(() => {
        invalidate();
        toast.success(t("common.saved"));
      })
      .catch((e) => toast.error(String(e)))
      .finally(() => setSaving(false));
  };

  // Changing the language updates both the UI (i18n) and the backend `locale`
  // (so the agent searches/writes in the same language), persisting immediately.
  const changeLanguage = (code: string) => {
    i18n.changeLanguage(code);
    const backendLocale = code === "pt-BR" ? "pt-br" : "en";
    const next = { ...s, locale: backendLocale };
    setS(next);
    post("/settings", next)
      .then(() => {
        invalidate();
        toast.success(t("common.saved"));
      })
      .catch((e) => toast.error(String(e)));
  };

  const resetRuns = () => {
    if (!confirm(t("config.resetRunsConfirm"))) return;
    post("/reset-runs")
      .then(() => {
        invalidate();
        toast.success(t("common.done"));
      })
      .catch((e) => toast.error(String(e)));
  };

  // Job sources persist immediately (like search variants), independent of Save.
  const addSource = () => {
    if (!srcDomain.trim()) return;
    post("/sources", { name: srcName, domain: srcDomain })
      .then(() => {
        setSrcName("");
        setSrcDomain("");
        invalidate();
        toast.success(t("common.saved"));
      })
      .catch((e) => toast.error(String(e)));
  };
  const toggleSource = (id: number) =>
    post(`/sources/${id}/toggle`)
      .then(invalidate)
      .catch((e) => toast.error(String(e)));
  const removeSource = (id: number) =>
    del(`/sources/${id}`)
      .then(() => {
        invalidate();
        toast.success(t("common.removed"));
      })
      .catch((e) => toast.error(String(e)));

  return (
    <div className="space-y-5">
      <Card>
        <CardHeader title={t("common.language")} />
        <div className="divide-y divide-border">
          <Row label={t("common.language")}>
            <Select value={i18n.language} onChange={(e) => changeLanguage(e.target.value)} className="py-1.5">
              {LANGUAGES.map((lang) => (
                <option key={lang.code} value={lang.code}>
                  {lang.label}
                </option>
              ))}
            </Select>
          </Row>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("config.application")} />
        <div className="divide-y divide-border">
          <Row label={t("config.humanReview")} hint={t("config.humanReviewHint")}>
            <Toggle
              on={s.require_human_review}
              onClick={() => set("require_human_review", !s.require_human_review)}
            />
          </Row>
          <Row label={t("config.modeLabel")} hint={t("config.modeHint")}>
            <Select value={s.apply_mode} onChange={(e) => set("apply_mode", e.target.value)} className="py-1.5">
              <option value="review">review</option>
              <option value="autonomous">autonomous</option>
              <option value="hybrid">hybrid</option>
            </Select>
          </Row>
          <Row label={t("config.hybridLabel")} hint={t("config.hybridHint")}>
            <Input
              type="number"
              step="0.05"
              min="0"
              max="1"
              value={s.hybrid_threshold}
              onChange={(e) => set("hybrid_threshold", Number(e.target.value))}
              className="w-24"
            />
          </Row>
          <Row label={t("config.dryRunLabel")} hint={t("config.dryRunHint")}>
            <Toggle on={s.dry_run} onClick={() => set("dry_run", !s.dry_run)} />
          </Row>
          <Row label={t("config.languageFilterLabel")} hint={t("config.languageFilterHint")}>
            <Toggle on={s.language_filter} onClick={() => set("language_filter", !s.language_filter)} />
          </Row>
          <Row label={t("config.workModelLabel")} hint={t("config.workModelHint")}>
            <Select value={s.work_model} onChange={(e) => set("work_model", e.target.value)} className="py-1.5">
              <option value="remote">{t("config.workModelRemote")}</option>
              <option value="onsite">{t("config.workModelOnsite")}</option>
              <option value="hybrid">{t("config.workModelHybrid")}</option>
            </Select>
          </Row>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("config.sourcesTitle")} hint={t("config.sourcesHint")} />
        {sources.isLoading ? (
          <SkeletonRows rows={3} />
        ) : sources.isError ? (
          <ErrorState message={t("common.error")} retryLabel={t("common.retry")} onRetry={() => sources.refetch()} />
        ) : !sources.data?.length ? (
          <Empty>{t("config.sourcesEmpty")}</Empty>
        ) : (
          <ul className="divide-y divide-border">
            {sources.data.map((src) => (
              <li key={src.id} className="flex items-center gap-3 px-4 py-3">
                <Toggle on={src.enabled} onClick={() => toggleSource(src.id)} />
                <div className="min-w-0 flex-1">
                  <div className="truncate text-sm text-fg">{src.name}</div>
                  <div className="truncate text-sm text-fg-muted">{src.domain}</div>
                </div>
                {!src.enabled && <Badge>{t("config.sourceDisabled")}</Badge>}
                {!src.builtin && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-fg-subtle hover:bg-danger-tint hover:text-danger"
                    aria-label={t("config.sourceRemove")}
                    onClick={() => removeSource(src.id)}
                  >
                    <Trash2 size={15} />
                  </Button>
                )}
              </li>
            ))}
          </ul>
        )}
        <div className="flex flex-wrap items-center gap-2 border-t border-border p-4">
          <Input
            value={srcName}
            onChange={(e) => setSrcName(e.target.value)}
            placeholder={t("config.sourceNamePlaceholder")}
            className="max-w-xs"
          />
          <Input
            value={srcDomain}
            onChange={(e) => setSrcDomain(e.target.value)}
            placeholder={t("config.sourceDomainPlaceholder")}
            className="max-w-xs"
          />
          <Button variant="primary" onClick={addSource}>
            {t("config.sourceAdd")}
          </Button>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("config.agent")} />
        <div className="divide-y divide-border">
          <Row label={t("config.useChrome")} hint={t("config.useChromeHint")}>
            <Toggle on={s.use_chrome} onClick={() => set("use_chrome", !s.use_chrome)} />
          </Row>
          <Row label={t("config.bypassPermissions")} hint={t("config.bypassPermissionsHint")}>
            <Toggle on={s.bypass_permissions} onClick={() => set("bypass_permissions", !s.bypass_permissions)} />
          </Row>
          <Row label={t("config.autoRun")}>
            <Toggle on={s.auto_run_on_idle} onClick={() => set("auto_run_on_idle", !s.auto_run_on_idle)} />
          </Row>
          <Row label={t("config.idleThreshold")}>
            <Input
              type="number"
              value={s.idle_threshold_secs}
              onChange={(e) => set("idle_threshold_secs", Number(e.target.value))}
              className="w-24"
            />
          </Row>
          <Row label={t("config.claudeBinary")}>
            <Input value={s.claude_bin} onChange={(e) => set("claude_bin", e.target.value)} className="w-48" />
          </Row>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("config.data")} />
        <div className="divide-y divide-border">
          <Row label={t("config.cvPath")}>
            <Input value={s.cv_file_path} onChange={(e) => set("cv_file_path", e.target.value)} className="w-64" />
          </Row>
          <Row label={t("config.linkedinUrl")}>
            <Input value={s.linkedin_url} onChange={(e) => set("linkedin_url", e.target.value)} className="w-64" />
          </Row>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("identity.title")} />
        <div className="divide-y divide-border">
          {IDENTITY_FIELDS.map((f) => (
            <Row key={f.key} label={t(f.labelKey)}>
              <Input
                value={ident[f.key] ?? ""}
                onChange={(e) =>
                  setIdent({
                    ...ident,
                    [f.key]: f.key === "birth_date" ? maskBirthDate(e.target.value) : e.target.value,
                  })
                }
                inputMode={f.key === "birth_date" ? "numeric" : undefined}
                placeholder={f.key === "birth_date" ? "DD/MM/AAAA" : undefined}
                className="w-64"
              />
            </Row>
          ))}
        </div>
      </Card>
      <p className="px-1 text-xs text-fg-muted">{t("identity.hint")}</p>

      <Card>
        <CardHeader title={t("config.dangerZone")} />
        <div className="divide-y divide-border">
          <Row label={t("config.resetRuns")} hint={t("config.resetRunsHint")}>
            <Button variant="danger" onClick={resetRuns}>
              {t("config.resetRuns")}
            </Button>
          </Row>
        </div>
      </Card>

      <Button variant="primary" onClick={save} disabled={saving}>
        {saving && <Loader2 size={14} className="animate-spin" />}
        {t("config.save")}
      </Button>
    </div>
  );
}
