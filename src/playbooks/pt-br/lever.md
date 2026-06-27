# Playbook — Lever (jobs.lever.co)

Geralmente NÃO exige login. Formulário único, com upload de CV que autopreenche campos.

## Fluxo
1. Abra a vaga e clique em "Apply" / role até o formulário.
2. Faça upload do currículo (use o arquivo informado) — o Lever parseia e preenche
   nome/email/experiência. Confira os campos.
3. Complete os campos obrigatórios (nome completo, email, telefone, LinkedIn/URLs).
4. Responda perguntas adicionais (EEO/diversidade são opcionais — pode pular) e knockouts
   pelo BANCO DE RESPOSTAS.
5. Cole carta de apresentação se houver "Additional information".
6. Clique em "Submit application".

## Regras
- Campos EEO/diversidade são opcionais; não bloqueie por eles.
- Pergunta obrigatória sem resposta → `pending kind="answer_needed"` com `field_key`.
- Só reporte `applied` ao ver "Application submitted"/tela de agradecimento.
