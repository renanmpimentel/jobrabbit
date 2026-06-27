import { useEffect, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useSettings, useInvalidate, post, type Settings } from "../hooks";
import { LANGUAGES } from "../i18n";
import { Card, CardHeader, Button, Input, Toggle } from "../ui";

function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 px-4 py-3">
      <div>
        <div className="text-sm text-slate-100">{label}</div>
        {hint && <div className="text-xs text-fg-muted">{hint}</div>}
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

export default function Config() {
  const { t, i18n } = useTranslation();
  const settings = useSettings();
  const invalidate = useInvalidate();
  const [s, setS] = useState<Settings | null>(null);

  useEffect(() => {
    if (settings.data) setS(settings.data);
  }, [settings.data]);

  if (!s) return <div className="text-sm text-fg-muted">{t("config.loading")}</div>;

  const set = <K extends keyof Settings>(k: K, v: Settings[K]) => setS({ ...s, [k]: v });

  const save = () => post("/settings", s).then(invalidate).catch((e) => alert(String(e)));

  // Changing the language updates both the UI (i18n) and the backend `locale`
  // (so the agent searches/writes in the same language), persisting immediately.
  const changeLanguage = (code: string) => {
    i18n.changeLanguage(code);
    const backendLocale = code === "pt-BR" ? "pt-br" : "en";
    const next = { ...s, locale: backendLocale };
    setS(next);
    post("/settings", next).then(invalidate).catch((e) => alert(String(e)));
  };

  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <Card>
        <CardHeader title={t("common.language")} />
        <div className="divide-y divide-edge">
          <Row label={t("common.language")}>
            <select
              value={i18n.language}
              onChange={(e) => changeLanguage(e.target.value)}
              className="rounded-lg border border-edge bg-ink-850 px-3 py-1.5 text-sm text-fg"
            >
              {LANGUAGES.map((lang) => (
                <option key={lang.code} value={lang.code}>
                  {lang.label}
                </option>
              ))}
            </select>
          </Row>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("config.application")} />
        <div className="divide-y divide-edge">
          <Row label={t("config.modeLabel")} hint={t("config.modeHint")}>
            <select
              value={s.apply_mode}
              onChange={(e) => set("apply_mode", e.target.value)}
              className="rounded-lg border border-edge bg-ink-850 px-3 py-1.5 text-sm text-fg"
            >
              <option value="review">review</option>
              <option value="autonomous">autonomous</option>
              <option value="hybrid">hybrid</option>
            </select>
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
        </div>
      </Card>

      <Card>
        <CardHeader title={t("config.agent")} />
        <div className="divide-y divide-edge">
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
        <div className="divide-y divide-edge">
          <Row label={t("config.cvPath")}>
            <Input value={s.cv_file_path} onChange={(e) => set("cv_file_path", e.target.value)} className="w-64" />
          </Row>
          <Row label={t("config.linkedinUrl")}>
            <Input value={s.linkedin_url} onChange={(e) => set("linkedin_url", e.target.value)} className="w-64" />
          </Row>
        </div>
      </Card>

      <Button variant="primary" onClick={save}>
        {t("config.save")}
      </Button>
    </div>
  );
}
