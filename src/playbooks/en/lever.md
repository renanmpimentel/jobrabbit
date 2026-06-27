# Playbook — Lever (jobs.lever.co)

Usually does NOT require login. A single form, with a CV upload that autofills fields.

## Flow
1. Open the job and click "Apply" / scroll to the form.
2. Upload the résumé (use the provided file) — Lever parses it and fills in
   name/email/experience. Check the fields.
3. Complete the required fields (full name, email, phone, LinkedIn/URLs).
4. Answer additional questions (EEO/diversity are optional — you may skip) and knockouts
   from the ANSWER BANK.
5. Paste the cover letter if there is an "Additional information" field.
6. Click "Submit application".

## Rules
- EEO/diversity fields are optional; do not block on them.
- A required question with no answer → `pending kind="answer_needed"` with `field_key`.
- Only report `applied` once you see "Application submitted"/the thank-you screen.
