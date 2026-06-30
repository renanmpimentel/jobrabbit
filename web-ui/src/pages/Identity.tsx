import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAnswers, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Button, Input } from "../ui";

// Identity field keys (must match IDENTITY_FIELDS in src/db/models.rs) paired
// with their i18n label key.
const FIELDS: { key: string; labelKey: string }[] = [
  { key: "full_name", labelKey: "identity.fullName" },
  { key: "cpf", labelKey: "identity.cpf" },
  { key: "phone", labelKey: "identity.phone" },
  { key: "birth_date", labelKey: "identity.birthDate" },
  { key: "city_state", labelKey: "identity.cityState" },
];

export default function Identity() {
  const { t } = useTranslation();
  const answers = useAnswers();
  const invalidate = useInvalidate();
  const [values, setValues] = useState<Record<string, string>>({});

  useEffect(() => {
    if (answers.data) {
      const next: Record<string, string> = {};
      for (const f of FIELDS) {
        next[f.key] = answers.data.find((a) => a.key === f.key)?.value ?? "";
      }
      setValues(next);
    }
  }, [answers.data]);

  const save = () =>
    Promise.all(
      FIELDS.map((f) => post("/answers", { key: f.key, value: values[f.key] ?? "" }))
    )
      .then(invalidate)
      .catch((e) => alert(String(e)));

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader title={t("identity.title")} />
        <div className="divide-y divide-edge">
          {FIELDS.map((f) => (
            <div key={f.key} className="flex items-center justify-between gap-4 px-4 py-3">
              <div className="text-sm text-slate-100">{t(f.labelKey)}</div>
              <Input
                value={values[f.key] ?? ""}
                onChange={(e) => setValues({ ...values, [f.key]: e.target.value })}
                className="w-64"
              />
            </div>
          ))}
        </div>
      </Card>
      <p className="text-xs text-fg-muted">{t("identity.hint")}</p>
      <Button variant="primary" onClick={save}>
        {t("identity.save")}
      </Button>
    </div>
  );
}
