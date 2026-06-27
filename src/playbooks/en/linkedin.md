# Playbook — LinkedIn Easy Apply (linkedin.com/jobs)

Requires being logged in to LinkedIn (the Chrome session usually already is).

## Flow
1. Open the job. Look for the "Easy Apply" button.
   - If there is only "Apply" (external apply), follow to the external site and use the
     destination ATS playbook (detect it from the new URL).
2. If not logged in → `pending kind="login"` (platform: LinkedIn).
3. Easy Apply is multi-step (1–5 screens): contact, résumé, questions, review.
4. Confirm/select the résumé (use the provided file if it asks for an upload).
5. Answer screening questions from the ANSWER BANK (years of experience, languages,
   work authorization, salary expectation if there is a field).
6. Advance ("Next") to "Review" and click "Submit application".

## Rules
- Do NOT click "Submit" before going through all screens and reviewing.
- A question with no answer in the bank → `pending kind="answer_needed"` with `field_key`.
- Only report `applied` once you see "Application sent".
- Rate limit: LinkedIn is sensitive to volume — respect the pace (delays between applications).
