# inHire (external ATS) — playbook

inHire forms (e.g. company.inhire.app) ask for identity fields up front,
before screening questions.

1. Open the job page in the logged-in Chrome.
2. Fill the identity fields from the ANSWER BANK first: full name, CPF,
   phone (with DDD), birth date, city/state. These are the user's own data —
   fill them when present.
3. Fill email and any URLs from the profile + answer bank.
4. Upload the CV file if requested.
5. Answer screening questions from the answer bank; if a required answer is
   missing, emit `answer_needed` with a stable `field_key`.
6. Advance every step ("Next"/"Continue") until an explicit confirmation.

- Never invent a document number — if CPF/phone is absent, emit `answer_needed`.
- Follow any requested field format (CPF mask, phone mask).
