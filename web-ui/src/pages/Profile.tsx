import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useProfile, useVariants, useInvalidate, post, del } from "../hooks";
import { Card, CardHeader, Button, Textarea, Input, Toggle, Empty, Badge } from "../ui";

export default function Profile() {
  const { t } = useTranslation();
  const profile = useProfile();
  const variants = useVariants();
  const invalidate = useInvalidate();

  const [bg, setBg] = useState("");
  const [cv, setCv] = useState("");
  const [label, setLabel] = useState("");
  const [query, setQuery] = useState("");
  const [cvPath, setCvPath] = useState("");
  const [linkedin, setLinkedin] = useState("");

  useEffect(() => {
    if (profile.data) {
      setBg(profile.data.background);
      setCv(profile.data.cv_base);
    }
  }, [profile.data]);

  const saveProfile = () =>
    post("/profile", { background: bg, cv_base: cv }).then(invalidate).catch((e) => alert(String(e)));

  const addVariant = () => {
    if (!label.trim() || !query.trim()) return;
    post("/variants", { label, query })
      .then(() => {
        setLabel("");
        setQuery("");
        invalidate();
      })
      .catch((e) => alert(String(e)));
  };

  const doImport = (body: object) => post("/import", body).then(invalidate).catch((e) => alert(String(e)));

  return (
    <div className="space-y-6">
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader title={t("profile.background")} />
          <div className="p-4">
            <Textarea rows={8} value={bg} onChange={(e) => setBg(e.target.value)} />
          </div>
        </Card>
        <Card>
          <CardHeader title={t("profile.cvBase")} />
          <div className="p-4">
            <Textarea rows={8} value={cv} onChange={(e) => setCv(e.target.value)} />
          </div>
        </Card>
      </div>
      <Button variant="primary" onClick={saveProfile}>
        {t("profile.saveProfile")}
      </Button>

      <Card>
        <CardHeader title={t("profile.searchVariants")} hint={t("profile.searchVariantsHint")} />
        {!variants.data?.length ? (
          <Empty>{t("profile.noVariants")}</Empty>
        ) : (
          <ul className="divide-y divide-border">
            {variants.data.map((v) => (
              <li key={v.id} className="flex items-center gap-3 px-4 py-2.5">
                <Toggle on={v.enabled} onClick={() => post(`/variants/${v.id}/toggle`).then(invalidate)} />
                <div className="flex-1">
                  <div className="text-sm text-fg">{v.label}</div>
                  <div className="text-xs text-fg-muted">{v.query}</div>
                </div>
                {!v.enabled && <Badge>{t("profile.disabled")}</Badge>}
                <Button variant="danger" onClick={() => del(`/variants/${v.id}`).then(invalidate)}>
                  {t("profile.remove")}
                </Button>
              </li>
            ))}
          </ul>
        )}
        <div className="flex flex-wrap items-center gap-2 border-t border-border p-4">
          <Input
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder={t("profile.labelPlaceholder")}
            className="max-w-xs"
          />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("profile.queryPlaceholder")}
            className="max-w-xs"
          />
          <Button variant="primary" onClick={addVariant}>
            {t("profile.addVariant")}
          </Button>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("profile.importProfile")} hint={t("profile.importProfileHint")} />
        <div className="space-y-3 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <Input
              value={cvPath}
              onChange={(e) => setCvPath(e.target.value)}
              placeholder={t("profile.cvPathPlaceholder")}
              className="max-w-md"
            />
            <Button onClick={() => doImport({ cv_path: cvPath })} disabled={!cvPath.trim()}>
              {t("profile.importCv")}
            </Button>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Input
              value={linkedin}
              onChange={(e) => setLinkedin(e.target.value)}
              placeholder={t("profile.linkedinPlaceholder")}
              className="max-w-md"
            />
            <Button onClick={() => doImport({ linkedin_url: linkedin })} disabled={!linkedin.trim()}>
              {t("profile.importLinkedin")}
            </Button>
          </div>
        </div>
      </Card>
    </div>
  );
}
