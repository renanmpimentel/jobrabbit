# Playbook — LinkedIn Easy Apply (linkedin.com/jobs)

Requer estar logado no LinkedIn (a sessão do Chrome normalmente já está).

## Fluxo
1. Abra a vaga. Procure o botão "Candidatura simplificada" / "Easy Apply".
   - Se só houver "Candidatar-se" (apply externo), siga para o site externo e use o
     playbook do ATS de destino (detecte pela nova URL).
2. Se não estiver logado → `pending kind="login"` (plataforma: LinkedIn).
3. Easy Apply é multi-etapas (1–5 telas): contato, currículo, perguntas, revisão.
4. Confirme/selecione o currículo (use o arquivo informado se pedir upload).
5. Responda perguntas de triagem pelo BANCO DE RESPOSTAS (anos de experiência, idiomas,
   autorização de trabalho, pretensão se houver campo).
6. Avance ("Avançar"/"Next") até "Revisar" e clique em "Enviar candidatura"/"Submit".

## Regras
- NÃO clique em "Enviar" antes de passar por todas as telas e revisar.
- Pergunta sem resposta no banco → `pending kind="answer_needed"` com `field_key`.
- Só reporte `applied` ao ver "Candidatura enviada".
- Rate limit: o LinkedIn é sensível a volume — respeite o ritmo (delays entre candidaturas).
