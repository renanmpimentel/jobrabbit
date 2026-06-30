# Campos de Identidade + inHire — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Permitir que o agente preencha campos de identidade brasileiros obrigatórios (CPF, celular, nome, nascimento, cidade/estado) em formulários ATS externos, com um editor proativo na UI, política de prompt explícita e suporte dedicado ao inHire.

**Architecture:** Os 5 campos de identidade entram como respostas canônicas seedadas na tabela `answers` (uma nova constante `IDENTITY_FIELDS`). Uma rota `POST /api/answers` valida a key e grava direto no banco; uma página "Dados pessoais" (`Identity.tsx`) edita esses campos. O prompt ganha uma regra explícita de que dados de identidade do banco são do próprio usuário (preencher quando presentes, nunca inventar). O inHire vira um `Ats` detectado com playbook próprio.

**Tech Stack:** Rust (rusqlite, axum), React + TypeScript (Vite, react-i18next, lucide-react).

> NOTA pós-implementação: o editor "Dados pessoais", originalmente planejado como
> página separada (Task 5), foi posteriormente movido para dentro da aba **Config**
> a pedido do usuário (commit `refactor(web-ui): move personal data editor into Config tab`).

## Global Constraints

- Build/test SOMENTE via Docker: `make test` (fmt + clippy -D warnings + test + web build). Manter tudo verde.
- Clippy com `-D warnings`: zero warnings.
- COMMITS: NUNCA adicionar trailer `Co-Authored-By: Claude` neste repo.
- Os 5 campos de identidade e slugs (verbatim): `cpf` ("CPF"), `phone` ("Celular com DDD"), `full_name` ("Nome completo"), `birth_date` ("Data de nascimento"), `city_state` ("Cidade/Estado").
- Política de preenchimento: preenche se a resposta estiver no banco; se ausente, `pending answer_needed`. Nunca inventar número de documento.
- i18n: toda string visível usa `t("...")` com a chave presente em `en.json` E `pt-BR.json`.
- inHire: detecção por substring `inhire.`; slug `inhire`; nome `inHire`.

---

## File Structure

- `src/db/models.rs` — nova `IDENTITY_FIELDS`; helper `answer_label`.
- `src/db/mod.rs` — `seed_answers` seeda também `IDENTITY_FIELDS`; teste.
- `src/web/mod.rs` — rota+handler `POST /api/answers` com validação de key.
- `src/agent/prompts.rs` — regra de identidade nas "regras de execução" (en + pt-br).
- `src/ats.rs` — variante `Ats::InHire`, detecção, slug, playbook embutido + map.
- `src/playbooks/en/inhire.md`, `src/playbooks/pt-br/inhire.md` — playbooks novos.
- `web-ui/src/pages/Identity.tsx` — página "Dados pessoais" (depois movida p/ Config).
- `web-ui/src/App.tsx` — entrada no array `TABS` + import do ícone.
- `web-ui/src/locales/en.json`, `pt-BR.json` — chaves i18n.

---

## Task 1: Constante `IDENTITY_FIELDS`, seeding e helper `answer_label`

**Files:**
- Modify: `src/db/models.rs` (após `ANSWER_FIELDS`)
- Modify: `src/db/mod.rs` (`seed_answers`)
- Test: `src/db/mod.rs` (`mod tests`)

**Interfaces:**
- Produces:
  - `pub const IDENTITY_FIELDS: &[(&str, &str)]` — os 5 campos de identidade.
  - `pub fn answer_label(key: &str) -> Option<&'static str>` — retorna o label de uma key conhecida (busca em `ANSWER_FIELDS` e depois `IDENTITY_FIELDS`), ou `None`. Consumido pela Task 2.

- [ ] **Step 1: Teste que falha** (no `mod tests` de `src/db/mod.rs`):

```rust
    #[test]
    fn identity_fields_are_seeded() {
        let db = Db::open_in_memory().unwrap();
        let answers = db.get_answers().unwrap();
        for key in ["cpf", "phone", "full_name", "birth_date", "city_state"] {
            let a = answers
                .iter()
                .find(|a| a.key == key)
                .unwrap_or_else(|| panic!("identity field {key} not seeded"));
            assert_eq!(a.value, "", "{key} should start empty");
        }
    }
```

- [ ] **Step 2:** `make test` → FALHA (campos não semeados).

- [ ] **Step 3: `IDENTITY_FIELDS` e `answer_label` em `models.rs`** (após `ANSWER_FIELDS`):

```rust
/// Brazilian identity fields the user fills in proactively (page "Dados
/// pessoais"). Seeded into the same `answers` table as the triage fields.
pub const IDENTITY_FIELDS: &[(&str, &str)] = &[
    ("cpf", "CPF"),
    ("phone", "Celular com DDD"),
    ("full_name", "Nome completo"),
    ("birth_date", "Data de nascimento"),
    ("city_state", "Cidade/Estado"),
];

/// Canonical label for a known answer key (triage or identity), or `None` if the
/// key is not recognized. Used to validate/label direct answer writes.
pub fn answer_label(key: &str) -> Option<&'static str> {
    ANSWER_FIELDS
        .iter()
        .chain(IDENTITY_FIELDS.iter())
        .find(|(k, _)| *k == key)
        .map(|(_, label)| *label)
}
```

- [ ] **Step 4: `seed_answers` seeda ambos** (`src/db/mod.rs`):

```rust
    fn seed_answers(&self) -> Result<()> {
        for (key, label) in ANSWER_FIELDS.iter().chain(IDENTITY_FIELDS.iter()) {
            self.conn.execute(
                "INSERT OR IGNORE INTO answers (key, label, value, updated_at)
                 VALUES (?1, ?2, '', ?3)",
                params![key, label, now_rfc3339()],
            )?;
        }
        Ok(())
    }
```

- [ ] **Step 5:** `make test` → PASS.
- [ ] **Step 6:** Commit: `feat(db): seed Brazilian identity fields + answer_label helper`

---

## Task 2: Rota `POST /api/answers` com validação de key

**Files:**
- Modify: `src/web/mod.rs` (rota + struct + handler)
- Test: `src/web/mod.rs` (validação via `models::answer_label`)

**Interfaces:**
- Consumes: `db::models::answer_label` (Task 1); helpers `internal`, `ok`, `AppState`, `ApiError`; `db.set_answer`.
- Produces: rota `POST /api/answers` com corpo `{ "key", "value" }`.

- [ ] **Step 1: Teste** (em `#[cfg(test)] mod tests` de `src/web/mod.rs`):

```rust
#[cfg(test)]
mod tests {
    use crate::db::models::answer_label;

    #[test]
    fn answer_label_known_and_unknown() {
        assert_eq!(answer_label("cpf"), Some("CPF"));
        assert_eq!(answer_label("salary_expectation"), Some("Salary expectation"));
        assert_eq!(answer_label("not_a_real_key"), None);
    }
}
```

- [ ] **Step 2:** `make test` → compila e passa (helper já existe na Task 1).

- [ ] **Step 3: Registrar rota** (mesclar GET+POST numa linha `/api/answers`):

```rust
        .route("/api/answers", get(get_answers).post(post_answer))
```
(removendo a linha antiga que só tinha `get(get_answers)`).

- [ ] **Step 4: Struct + handler** (após `get_answers`):

```rust
#[derive(Deserialize)]
struct AnswerBody {
    key: String,
    value: String,
}

/// Grava/atualiza uma resposta do banco diretamente (usado pela página "Dados
/// pessoais"). Valida que a key é conhecida; caso contrário, 400.
async fn post_answer(
    State(s): State<AppState>,
    Json(body): Json<AnswerBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let label = crate::db::models::answer_label(&body.key)
        .ok_or((StatusCode::BAD_REQUEST, format!("unknown answer key: {}", body.key)))?;
    let db = s.db.lock().unwrap();
    db.set_answer(&body.key, label, &body.value).map_err(internal)?;
    Ok(ok())
}
```

- [ ] **Step 5:** `make test` → PASS.
- [ ] **Step 6:** Commit: `feat(web): add POST /api/answers with key validation`

---

## Task 3: Política de identidade no prompt (en + pt-br)

**Files:**
- Modify: `src/agent/prompts.rs` (regras de execução do `apply_for_job`, en e pt-br)
- Test: `src/agent/prompts.rs` (`mod tests`, teste do `apply_for_job`)

- [ ] **Step 1:** Estender o teste do `apply_for_job` (após `assert!(apply.contains("answer_needed"));`):

```rust
        assert!(
            apply.contains("CPF"),
            "must include the identity-data fill policy"
        );
```

- [ ] **Step 2:** `make test` → FALHA (prompt não menciona "CPF").

- [ ] **Step 3: Regra no prompt EN** (após a regra de "missing answer"):

```rust
                 - Identity data the user provided in the answer bank (CPF, phone, full name,\n\
                   birth date, city/state) is the user's OWN data for their OWN application —\n\
                   FILL it into the form whenever present. Only emit `answer_needed` if it is\n\
                   absent. NEVER invent or guess a document number.\n\
```

- [ ] **Step 4: Regra no prompt pt-br** (após "Se faltar uma resposta OBRIGATÓRIA…"):

```rust
                 - Dados de identidade que o usuário informou no banco de respostas (CPF, celular,\n\
                   nome completo, data de nascimento, cidade/estado) são dados do PRÓPRIO usuário\n\
                   para a PRÓPRIA candidatura — PREENCHA no formulário sempre que presentes. Só\n\
                   emita `answer_needed` se estiverem ausentes. NUNCA invente/adivinhe um número\n\
                   de documento.\n\
```

- [ ] **Step 5:** `make test` → PASS.
- [ ] **Step 6:** Commit: `feat(agent): allow filling user-provided identity data, never invent it`

---

## Task 4: Detecção e playbook do inHire

**Files:**
- Create: `src/playbooks/en/inhire.md`, `src/playbooks/pt-br/inhire.md`
- Modify: `src/ats.rs` (enum, `name`, `slug`, `detect`, consts, `embedded_playbook`, testes)

**Interfaces:**
- Produces: `Ats::InHire`. `detect("...inhire...")` → `Ats::InHire`.

- [ ] **Step 1: Playbook EN** (`src/playbooks/en/inhire.md`):

```markdown
# inHire (external ATS) — playbook

inHire forms (e.g. company.inhire.app) ask for identity fields up front,
before screening questions.

1. Open the job page in the logged-in Chrome.
2. Fill the identity fields from the ANSWER BANK first: full name, CPF,
   phone (with DDD), birth date, city/state. These are the user's own data —
   fill them when present.
3. Fill email and any URLs from the profile + answer bank.
4. Upload the CV file if requested.
5. Answer screening questions from the answer bank; if a required answer is
   missing, emit `answer_needed` with a stable `field_key`.
6. Advance every step ("Next"/"Continue") until an explicit confirmation.

- Never invent a document number — if CPF/phone is absent, emit `answer_needed`.
- Follow any requested field format (CPF mask, phone mask).
```

- [ ] **Step 2: Playbook pt-br** (`src/playbooks/pt-br/inhire.md`):

```markdown
# inHire (ATS externo) — playbook

Formulários inHire (ex.: empresa.inhire.app) pedem dados de identidade logo no
início, antes das perguntas de triagem.

1. Abra a página da vaga no Chrome logado.
2. Preencha primeiro os campos de identidade pelo BANCO DE RESPOSTAS: nome
   completo, CPF, celular (com DDD), data de nascimento, cidade/estado. São
   dados do próprio usuário — preencha quando presentes.
3. Preencha email e URLs a partir do perfil + banco de respostas.
4. Faça upload do arquivo de CV se for pedido.
5. Responda às perguntas de triagem pelo banco; se faltar uma resposta
   obrigatória, emita `answer_needed` com um `field_key` estável.
6. Avance todas as etapas ("Próximo"/"Continuar") até uma confirmação explícita.

- Nunca invente número de documento — se CPF/celular faltar, emita `answer_needed`.
- Siga o formato pedido em cada campo (máscara de CPF, de celular).
```

- [ ] **Step 3: Testes** (no `mod tests` de `src/ats.rs`):

```rust
    #[test]
    fn detects_inhire() {
        assert_eq!(detect("https://flutter.inhire.app/jobs/123"), Ats::InHire);
        assert_eq!(detect("https://vaga.inhire.com.br/x"), Ats::InHire);
        assert_eq!(Ats::InHire.slug(), "inhire");
        assert_eq!(Ats::InHire.name(), "inHire");
    }

    #[test]
    fn inhire_playbook_not_empty() {
        assert!(!playbook(Ats::InHire, Locale::En).trim().is_empty());
        assert!(!playbook(Ats::InHire, Locale::PtBr).trim().is_empty());
        assert!(playbook(Ats::InHire, Locale::PtBr).contains("CPF"));
    }
```

- [ ] **Step 4:** `make test` → FALHA (compilação: `Ats::InHire` não existe).

- [ ] **Step 5: Edições em `ats.rs`:**
  - `enum Ats`: adicionar `InHire,` após `InfoJobs,`.
  - `name()`: `Ats::InHire => "inHire",`.
  - `slug()`: `Ats::InHire => "inhire",` (antes do `_ => "generic"`).
  - `detect()`: ramo `} else if has("inhire.") { Ats::InHire` (antes do `Generic`).
  - consts: `const PB_INHIRE_EN: &str = include_str!("playbooks/en/inhire.md");` e
    `const PB_INHIRE_PT_BR: &str = include_str!("playbooks/pt-br/inhire.md");`.
  - `embedded_playbook`: `(Locale::En, "inhire") => PB_INHIRE_EN,` e
    `(Locale::PtBr, "inhire") => PB_INHIRE_PT_BR,` (antes dos catch-all `_`).

- [ ] **Step 6:** `make test` → PASS.
- [ ] **Step 7:** Commit: `feat(ats): detect inHire and add dedicated playbook`

---

## Task 5: Página "Dados pessoais" na UI

> Posteriormente movida para a aba Config (ver nota no topo).

**Files:**
- Create: `web-ui/src/pages/Identity.tsx`
- Modify: `web-ui/src/App.tsx` (import do ícone + da página, entrada em `TABS`)
- Modify: `web-ui/src/locales/en.json`, `web-ui/src/locales/pt-BR.json`

**Interfaces:**
- Consumes: rota `POST /api/answers` (Task 2); campos seedados (Task 1); `useAnswers`, `post`, `useInvalidate`, `Card`, `CardHeader`, `Button`, `Input`.

- [ ] **Step 1: Chaves i18n `en.json`** — `nav.identity: "Personal data"` e bloco:

```json
  "identity": {
    "title": "Personal data",
    "hint": "Used to fill identity fields on external ATS forms (inHire, etc.)",
    "cpf": "CPF",
    "phone": "Mobile (with area code)",
    "fullName": "Full name",
    "birthDate": "Date of birth",
    "cityState": "City/State",
    "save": "Save personal data"
  },
```

- [ ] **Step 2: Chaves i18n `pt-BR.json`** — `nav.identity: "Dados pessoais"` e bloco:

```json
  "identity": {
    "title": "Dados pessoais",
    "hint": "Usados para preencher campos de identidade em formulários ATS externos (inHire, etc.)",
    "cpf": "CPF",
    "phone": "Celular com DDD",
    "fullName": "Nome completo",
    "birthDate": "Data de nascimento",
    "cityState": "Cidade/Estado",
    "save": "Salvar dados pessoais"
  },
```

- [ ] **Step 3: `Identity.tsx`:**

```tsx
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAnswers, useInvalidate, post } from "../hooks";
import { Card, CardHeader, Button, Input } from "../ui";

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
```

- [ ] **Step 4: `App.tsx`:** importar `IdCard` do lucide-react; `import Identity from "./pages/Identity";`; entrada em `TABS` após `profile`:
  `{ id: "identity", labelKey: "nav.identity", icon: IdCard, el: <Identity /> }`.

- [ ] **Step 5:** `make test` → PASS (web build).
- [ ] **Step 6:** Commit: `feat(web-ui): add Personal data page to edit identity fields`

---

## Self-Review

- `IDENTITY_FIELDS` + seeding + `answer_label` → Task 1.
- `POST /api/answers` com validação 400 → Task 2.
- Política de identidade no prompt (en + pt-br) → Task 3.
- inHire: enum, name, slug, detect, playbooks, map → Task 4.
- Página "Dados pessoais" + nav + i18n → Task 5.
- Keys `cpf/phone/full_name/birth_date/city_state` idênticas entre Rust, TS e i18n.
