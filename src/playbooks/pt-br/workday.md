# Playbook — Workday (*.myworkdayjobs.com)

Comum em grandes empresas. Costuma EXIGIR criar/entrar numa conta por empresa, e o fluxo
é longo (várias seções). Pode ter MFA que bloqueia automação.

## Fluxo
1. Abra a vaga e clique em "Apply" / "Candidatar-se".
2. Se pedir login/criar conta e não houver sessão → `pending kind="login"`
   (descreva: criar/entrar na conta Workday desta empresa). MFA/2FA → também `pending`.
3. "Autofill with Resume": faça upload do CV (DOCX) para preencher automaticamente.
4. Preencha as seções (My Information, Experience, Education, Application Questions)
   usando perfil + BANCO DE RESPOSTAS.
5. Aceite termos/consentimento quando exigido.
6. Avance todas as seções e clique em "Submit".

## Regras
- Fluxo longo e multi-seção: não desista no meio; complete seção a seção.
- Dropdowns dinâmicos (cidade aparece após país) — selecione na ordem e aguarde carregar.
- Pergunta obrigatória sem resposta → `pending kind="answer_needed"` com `field_key`.
- Só reporte `applied` ao ver a confirmação de envio.
