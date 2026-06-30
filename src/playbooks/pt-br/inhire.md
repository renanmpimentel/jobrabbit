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
