# Playbook — Gupy (gupy.io)

ATS mais comum no Brasil. Login ÚNICO entre empresas (login.gupy.io) — uma vez logado,
vale para todas as vagas Gupy.

## Fluxo
1. Abra a URL da vaga. Se aparecer "Candidatura rápida" / "Candidate-se", clique.
2. Se pedir login e você NÃO estiver logado → reporte `pending kind="login"` (plataforma: Gupy).
3. A Gupy costuma parsear o CV automaticamente — confira os campos pré-preenchidos.
4. Responda as perguntas de triagem (variam por empresa) usando o BANCO DE RESPOSTAS.
   Perguntas comuns: pretensão salarial, disponibilidade de início, mudança de cidade,
   PCD, consentimento LGPD (checkbox obrigatório), nível de inglês, regime (CLT/PJ),
   anos de experiência, disposição para viajar.
5. Marque o consentimento LGPD quando exigido.
6. Avance todas as etapas e clique em "Enviar candidatura".

## Regras
- Se uma pergunta obrigatória não tem resposta no banco → `pending kind="answer_needed"`
  com `field_key` (slug curto) e a pergunta exata. NÃO invente pretensão/dados sensíveis.
- Upload de CV: prefira DOCX. Use o arquivo informado.
- Captcha → `pending kind="captcha"` e pule.
- Só reporte `applied` ao ver a confirmação ("Candidatura enviada"/"inscrição realizada").
