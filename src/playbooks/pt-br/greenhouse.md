# Playbook — Greenhouse (boards.greenhouse.io)

Geralmente NÃO exige login (candidatura como convidado). Um formulário único por vaga.

## Fluxo
1. Abra a vaga e role até o formulário "Apply for this job".
2. Preencha: nome, email, telefone, LinkedIn, etc. (use perfil + BANCO DE RESPOSTAS).
3. Upload de currículo: use o arquivo informado (DOCX de preferência). O Greenhouse
   costuma autopreencher campos a partir do CV — confira.
4. Responda perguntas customizadas (knockouts: autorização de trabalho, visto, anos de
   experiência) pelo banco de respostas.
5. Cole a carta de apresentação se houver campo "Cover letter".
6. Clique em "Submit Application".

## Regras
- Campos com regex (telefone, etc.) → siga o formato pedido.
- Token CSRF muda por sessão — se o submit falhar, recarregue o form e tente de novo.
- Pergunta obrigatória sem resposta → `pending kind="answer_needed"` com `field_key`.
- Só reporte `applied` ao ver a confirmação ("Application submitted"/"Thank you").
