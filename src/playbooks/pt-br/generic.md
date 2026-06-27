# Playbook — Genérico (site sem playbook dedicado)

Use quando o ATS não é reconhecido. Conduza o fluxo de forma geral e cuidadosa.

## Fluxo
1. Abra a vaga e localize o botão de candidatura ("Candidatar-se"/"Apply"/"Enviar").
2. Se exigir login e não houver sessão → `pending kind="login"`.
3. Se houver upload de currículo, use o arquivo informado (DOCX de preferência).
4. Preencha os campos com perfil + BANCO DE RESPOSTAS; complete TODAS as etapas
   (formulários multi-página: clique em "Próximo"/"Continuar" até o final).
5. Marque consentimentos obrigatórios (LGPD/termos).
6. Clique para enviar.

## Regras
- Não desista no primeiro obstáculo: role a página, procure o botão correto, avance etapas.
- Pergunta obrigatória sem resposta no banco → `pending kind="answer_needed"` com `field_key`.
- Captcha → `pending kind="captcha"` e pule a vaga.
- Só reporte `applied` ao ver confirmação explícita; senão `failed` com o motivo.
