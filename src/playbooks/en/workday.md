# Playbook — Workday (*.myworkdayjobs.com)

Common at large companies. Usually REQUIRES creating/signing into a per-company account, and
the flow is long (several sections). It may have MFA that blocks automation.

## Flow
1. Open the job and click "Apply".
2. If it asks to log in/create an account and there is no session → `pending kind="login"`
   (describe: create/sign in to this company's Workday account). MFA/2FA → also `pending`.
3. "Autofill with Resume": upload the CV (DOCX) to fill fields automatically.
4. Fill in the sections (My Information, Experience, Education, Application Questions)
   using the profile + ANSWER BANK.
5. Accept terms/consent when required.
6. Advance through all sections and click "Submit".

## Rules
- Long, multi-section flow: don't give up midway; complete section by section.
- Dynamic dropdowns (city appears after country) — select in order and wait for them to load.
- A required question with no answer → `pending kind="answer_needed"` with `field_key`.
- Only report `applied` once you see the submission confirmation.
