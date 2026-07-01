import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Trash2 } from "lucide-react";
import { useProfile, useVariants, useInvalidate, post, del } from "../hooks";
import { Card, CardHeader, Button, Textarea, Input, Toggle, Empty, Badge, SkeletonRows, ErrorState } from "../ui";
import { useToast } from "../toast";

export default function Profile() {
  const { t } = useTranslation();
  const profile = useProfile();
  const variants = useVariants();
  const invalidate = useInvalidate();
  const toast = useToast();

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
    post("/profile", { background: bg, cv_base: cv })
      .then(() => {
        invalidate();
        toast.success(t("common.saved"));
      })
      .catch((e) => toast.error(String(e)));

  const addVariant = () => {
    if (!label.trim() || !query.trim()) return;
    post("/variants", { label, query })
      .then(() => {
        setLabel("");
        setQuery("");
        invalidate();
        toast.success(t("common.saved"));
      })
      .catch((e) => toast.error(String(e)));
  };

  const removeVariant = (id: number) =>
    del(`/variants/${id}`)
      .then(() => {
        invalidate();
        toast.success(t("common.removed"));
      })
      .catch((e) => toast.error(String(e)));

  const toggleVariant = (id: number) =>
    post(`/variants/${id}/toggle`)
      .then(invalidate)
      .catch((e) => toast.error(String(e)));

  const doImport = (body: object) =>
    post("/import", body)
      .then(() => {
        invalidate();
        toast.success(t("common.started"));
      })
      .catch((e) => toast.error(String(e)));

  return (
    <div className="space-y-6">
      {/* Profile fields — Save grouped in the header, next to the fields */}
      <Card>
        <CardHeader
          title={t("profile.title")}
          right={
            <Button variant="primary" onClick={saveProfile}>
              {t("profile.saveProfile")}
            </Button>
          }
        />
        <div className="grid gap-5 p-6 md:grid-cols-2">
          <div className="space-y-1.5">
            <label className="text-sm font-medium text-fg">{t("profile.background")}</label>
            <Textarea rows={8} value={bg} onChange={(e) => setBg(e.target.value)} />
          </div>
          <div className="space-y-1.5">
            <label className="text-sm font-medium text-fg">{t("profile.cvBase")}</label>
            <Textarea rows={8} value={cv} onChange={(e) => setCv(e.target.value)} />
          </div>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("profile.searchVariants")} hint={t("profile.searchVariantsHint")} />
        {variants.isLoading ? (
          <SkeletonRows rows={3} />
        ) : variants.isError ? (
          <ErrorState message={t("common.error")} retryLabel={t("common.retry")} onRetry={() => variants.refetch()} />
        ) : !variants.data?.length ? (
          <Empty>{t("profile.noVariants")}</Empty>
        ) : (
          <ul className="divide-y divide-border">
            {variants.data.map((v) => (
              <li key={v.id} className="flex items-center gap-3 px-4 py-3">
                <Toggle on={v.enabled} onClick={() => toggleVariant(v.id)} />
                <div className="min-w-0 flex-1">
                  <div className="truncate text-sm text-fg">{v.label}</div>
                  <div className="truncate text-sm text-fg-muted">{v.query}</div>
                </div>
                {!v.enabled && <Badge>{t("profile.disabled")}</Badge>}
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-fg-subtle hover:bg-danger-tint hover:text-danger"
                  aria-label={t("profile.remove")}
                  onClick={() => removeVariant(v.id)}
                >
                  <Trash2 size={15} />
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
