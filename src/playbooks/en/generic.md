# Playbook — Generic (site with no dedicated playbook)

Use this when the ATS is not recognized. Drive the flow generally and carefully.

## Flow
1. Open the job and locate the apply button ("Apply"/"Submit"/"Candidate-se").
2. If it requires login and there is no session → `pending kind="login"`.
3. If there is a résumé upload, use the provided file (DOCX preferred).
4. Fill the fields with the profile + ANSWER BANK; complete ALL steps
   (multi-page forms: click "Next"/"Continue" to the end).
5. Tick mandatory consents (LGPD/terms).
6. Click to submit.

## Rules
- Don't give up at the first obstacle: scroll the page, find the right button, advance steps.
- A required question with no answer in the bank → `pending kind="answer_needed"` with `field_key`.
- Captcha → `pending kind="captcha"` and skip the job.
- Only report `applied` once you see an explicit confirmation; otherwise `failed` with the reason.
