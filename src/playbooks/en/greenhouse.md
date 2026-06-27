# Playbook — Greenhouse (boards.greenhouse.io)

Usually does NOT require login (apply as a guest). A single form per job.

## Flow
1. Open the job and scroll to the "Apply for this job" form.
2. Fill in: name, email, phone, LinkedIn, etc. (use the profile + ANSWER BANK).
3. Résumé upload: use the provided file (DOCX preferred). Greenhouse often autofills
   fields from the CV — double-check.
4. Answer custom questions (knockouts: work authorization, visa, years of
   experience) from the answer bank.
5. Paste the cover letter if there is a "Cover letter" field.
6. Click "Submit Application".

## Rules
- Fields with regex (phone, etc.) → follow the requested format.
- The CSRF token changes per session — if the submit fails, reload the form and try again.
- A required question with no answer → `pending kind="answer_needed"` with `field_key`.
- Only report `applied` once you see the confirmation ("Application submitted"/"Thank you").
