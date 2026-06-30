# Design — Campos de identidade (CPF, celular…) + suporte inHire

**Data:** 2026-06-30
**Status:** Aprovado

## Problema

Em formulários ATS externos (ex.: inHire da Flutter Brazil), campos de
identidade obrigatórios — CPF e celular com DDD — aparecem logo no início.
Hoje o agente trava por três motivos combinados:

1. Esses campos não existem no banco de respostas (não são canônicos).
2. O agente se autorrecusa a preencher "documento de governo", mesmo sendo
   dado do próprio usuário — não há regra no código, é julgamento do modelo.
3. inHire não é detectado (cai no playbook genérico), sem orientação dedicada.

## Decisões

- **Campos suportados:** pacote BR essencial — `cpf`, `phone` (celular c/ DDD),
  `full_name`, `birth_date`, `city_state`.
- **Política de preenchimento:** preenche se estiver no banco; se ausente, gera
  `pending answer_needed`. Sem toggle de consentimento e sem aprovação extra.
- **inHire:** detecção dedicada + playbook próprio (en + pt-br).
- **Entrada dos dados:** editor proativo numa **página separada** ("Dados
  pessoais"), além do fluxo reativo já existente (tela Pending).

## Componentes

### 1. Campos de identidade — `src/db/models.rs` + `src/db/mod.rs`
- Nova constante `IDENTITY_FIELDS: &[(&str, &str)]` com os 5 campos:
  - `("cpf", "CPF")`
  - `("phone", "Celular com DDD")`
  - `("full_name", "Nome completo")`
  - `("birth_date", "Data de nascimento")`
  - `("city_state", "Cidade/Estado")`
- `seed_answers()` passa a semear `ANSWER_FIELDS` **e** `IDENTITY_FIELDS`
  (idempotente: bancos existentes recebem os novos campos no próximo open).
  Ambos vivem na mesma tabela `answers`, então o agente já os lê como banco
  de respostas.
- `IDENTITY_FIELDS` é `pub` para o web layer saber quais keys são de identidade.

### 2. Escrita direta no banco de respostas — `src/web/mod.rs`
- Hoje respostas só são gravadas via `POST /api/pending/:id/answer`.
- Nova rota `POST /api/answers` com corpo `{ "key": "...", "value": "..." }`:
  - Valida que `key` pertence a `ANSWER_FIELDS ∪ IDENTITY_FIELDS`; caso
    contrário, retorna `400 Bad Request`.
  - Resolve o `label` da key a partir das constantes e chama
    `db.set_answer(key, label, value)`. Retorna `ok()`.

### 3. Página "Dados pessoais" — `web-ui/`
- Novo arquivo `web-ui/src/pages/Identity.tsx`: card com um input por campo de
  identidade (os 5), carregados de `/api/answers` (`useAnswers`) e salvos via
  `post("/answers", { key, value })` seguido de `invalidate()`. Erro tratado
  com `alert(String(e))`, no padrão das outras telas.
- Registrar a página no array `TABS` de `web-ui/src/App.tsx`:
  `{ id: "identity", labelKey: "nav.identity", icon: IdCard, el: <Identity /> }`
  (ícone `IdCard` do lucide-react), posicionada após `profile`.
- Apenas os 5 campos de identidade têm editor; a triagem (18 campos) segue
  reativa como hoje.
- Chaves i18n novas em `en.json` e `pt-BR.json`, todas dedicadas: `nav.identity`,
  `identity.title` (título do card) e uma chave por campo
  (`identity.cpf`, `identity.phone`, `identity.fullName`, `identity.birthDate`,
  `identity.cityState`).

### 4. Política no prompt — `src/agent/prompts.rs` (en + pt-br)
- Adicionar instrução explícita, junto à seção que descreve o uso do banco de
  respostas: os campos de identidade do banco são **dados do próprio usuário,
  fornecidos por ele para as próprias candidaturas**. O agente DEVE preenchê-los
  no formulário quando presentes; gerar `answer_needed` apenas quando ausentes;
  e **nunca inventar/adivinhar** um número de documento. Isso remove a
  autorrecusa que travou o fluxo.

### 5. inHire — `src/ats.rs` + playbooks
- Novo variante `Ats::InHire` no enum, com:
  - `name()` → `"inHire"`.
  - `slug()` → `"inhire"`.
  - Detecção em `detect()`: substring `"inhire."` (cobre `*.inhire.app` e
    `inhire.com.br`). Posicionar antes do fallback `Generic`.
- Playbook dedicado `src/playbooks/en/inhire.md` e
  `src/playbooks/pt-br/inhire.md`, embutidos via `include_str!` e mapeados no
  carregamento de playbook (como os demais). Conteúdo: fluxo do inHire com
  campos de identidade no início, orientando a usar o banco de respostas
  (CPF/celular/nome/nascimento/cidade) e gerar `answer_needed` para o que faltar.

## Fluxo de dados

```
Página "Dados pessoais" ──POST /api/answers──┐
                                             ├─→ set_answer (tabela answers)
Tela Pending (answer_needed) ─POST /answer───┘
                                                   │
Busca → agente detecta inHire (ats.rs) → carrega playbook inhire
      → lê banco de respostas → preenche CPF/celular/... se presentes
      → answer_needed para os ausentes (nunca inventa documento)
```

## Tratamento de erros
- `POST /api/answers` com key desconhecida → `400` (`ApiError`); front mostra `alert`.
- `set_answer` em transação simples; erro → `internal` → `500`.

## Testes (Rust)
1. `seed_answers` cria os 5 campos de identidade (verificar `get_answers`).
2. `POST /api/answers` faz upsert de uma key válida e rejeita key desconhecida
   com `400` (teste de unidade do handler ou via validação da key).
3. `detect("https://flutter.inhire.app/jobs/123") == Ats::InHire` e
   `Ats::InHire.slug() == "inhire"`.
4. Playbook do inHire não-vazio em ambos os locales.

## Fora de escopo
- Editor para os 18 campos de triagem (seguem reativos).
- Criptografia/secrets para o CPF (fica em SQLite local, como as demais respostas).
- Pacote BR completo (RG, endereço completo, PIS/NIS).
- Toggle de consentimento / aprovação manual antes de submeter.
