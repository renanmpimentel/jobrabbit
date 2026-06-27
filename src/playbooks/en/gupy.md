# Playbook — Gupy (gupy.io)

The most common ATS in Brazil. SINGLE login across companies (login.gupy.io) — once logged
in, it works for every Gupy job.

## Flow
1. Open the job URL. If a "Quick apply" / "Candidate-se" button appears, click it.
   (Brazilian Gupy sites label the apply button "Candidate-se".)
2. If it asks for login and you are NOT logged in → report `pending kind="login"` (platform: Gupy).
3. Gupy usually parses the CV automatically — check the pre-filled fields.
4. Answer the screening questions (they vary per company) using the ANSWER BANK.
   Common questions: salary expectation, start availability, willingness to relocate,
   disability status (PCD), LGPD consent (mandatory checkbox), English level, employment
   regime (CLT/PJ), years of experience, willingness to travel.
5. Tick the LGPD consent when required.
6. Advance through all steps and click "Submit application" ("Enviar candidatura").

## Rules
- If a required question has no answer in the bank → `pending kind="answer_needed"`
  with `field_key` (a short slug) and the exact question. Do NOT make up salary/sensitive data.
- CV upload: prefer DOCX. Use the provided file.
- Captcha → `pending kind="captcha"` and skip.
- Only report `applied` once you see the confirmation ("Application sent"/"registration complete").
