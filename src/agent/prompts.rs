//! Prompt templates for the agent.
//!
//! Prompts are assembled per [`Locale`] (English by default, pt-BR selectable) and
//! instruct `claude` (with Chrome access via the extension) to search for jobs,
//! evaluate fit against the profile and generate a CV/cover letter. They are plain
//! text: the real integration passes them via `claude -p "<prompt>"`.

use crate::db::models::{Profile, SearchVariant};
use crate::locale::Locale;

/// Block with the candidate profile, reused across all prompts.
fn profile_block(profile: &Profile, locale: Locale) -> String {
    let empty = match locale {
        Locale::En => "(not provided)",
        Locale::PtBr => "(não informado)",
    };
    let background = if profile.background.is_empty() {
        empty
    } else {
        &profile.background
    };
    let cv_base = if profile.cv_base.is_empty() {
        empty
    } else {
        &profile.cv_base
    };
    match locale {
        Locale::En => format!(
            "## Candidate profile\n\
             ### Background\n{background}\n\n\
             ### Base CV\n{cv_base}\n",
        ),
        Locale::PtBr => format!(
            "## Perfil do candidato\n\
             ### Background\n{background}\n\n\
             ### CV base\n{cv_base}\n",
        ),
    }
}

/// Prompt for a search round: find jobs for a variant and evaluate fit.
///
/// The agent should use Chrome (already logged in) to search and, for each
/// relevant job, return one JSON per line (NDJSON) with the expected fields —
/// easy to parse in jobRabbit.
pub fn search_and_evaluate(
    profile: &Profile,
    variant: &SearchVariant,
    apply_mode: &str,
    dry_run: bool,
    hybrid_threshold: f64,
    locale: Locale,
    language_filter: bool,
) -> String {
    match locale {
        Locale::En => {
            // Application instruction according to the mode chosen by the user.
            let policy = if dry_run {
                "## SIMULATION MODE (dry-run)\n\
                 Do NOT submit ANY application and do NOT modify the sites (don't click submit/apply).\n\
                 Only search, evaluate and generate CV/cover letter. Report each job with status \"dry_run\"."
                    .to_string()
            } else {
                match apply_mode {
                    "autonomous" => "## Policy: AUTONOMOUS\n\
                         For fit >= 0.7: generate CV/cover letter and SUBMIT the application on the site (status \"applied\";\n\
                         if it fails, \"failed\"). Blockers (captcha/login/field) → report pending."
                        .to_string(),
                    "hybrid" => format!(
                        "## Policy: HYBRID\n\
                         For fit >= {th:.2}: generate CV/cover letter and SUBMIT (status \"applied\").\n\
                         For 0.7 <= fit < {th:.2}: generate CV/cover letter but do NOT submit — report status \"ready\"\n\
                         (awaiting approval). Blockers → report pending.",
                        th = hybrid_threshold
                    ),
                    // "review" (default and safest)
                    _ => "## Policy: REVIEW (manual approval)\n\
                         Do NOT submit any application (don't click submit/apply). For fit >= 0.7:\n\
                         generate CV/cover letter and report status \"ready\" (the user approves later). Blockers → pending."
                        .to_string(),
                }
            };

            // Optional language filter: restrict to jobs in the active language.
            let language = if language_filter {
                "## Language: English only\n\
                 Only consider jobs whose description/requirements are in English.\n\
                 If the job is in another language (or requires fluency in another language in the process), do NOT apply:\n\
                 emit the `job` line normally and report the `application` with status \"skipped\".\n\n"
            } else {
                ""
            };

            format!(
                "You are the jobRabbit agent, helping this candidate apply to jobs.\n\n\
                 IMPORTANT: use the **Claude in Chrome** integration (the user's REAL Chrome, already logged in).\n\
                 Do NOT use Playwright or open your own browser — use the connected Chrome tools.\n\n\
                 {profile}\n\
                 ## Task\n\
                 Use Chrome to search for jobs matching the variant:\n\
                 - Label: {label}\n\
                 - Query: {query}\n\n\
                 For each relevant job, evaluate the *fit* (0.0 to 1.0) against the profile (seniority, stack,\n\
                 work model, requirements).\n\n\
                 {policy}\n\n\
                 {language}\
                 ## Output protocol (REQUIRED)\n\
                 Emit ONE JSON line per event (NDJSON), with no text outside the lines, no code fences.\n\
                 Types:\n\
                 - Job found/evaluated:\n\
                   {{\"type\":\"job\",\"title\":\"...\",\"company\":\"...\",\"url\":\"...\",\"source\":\"linkedin|gupy|indeed|...\",\"description\":\"summary\",\"fit_score\":0.0}}\n\
                 - Application (status per policy: applied|ready|dry_run|skipped|failed):\n\
                   {{\"type\":\"application\",\"url\":\"<job url>\",\"status\":\"ready\",\"cv\":\"<generated cv>\",\"cover\":\"<cover letter>\"}}\n\
                 - Pending (needs the user):\n\
                   {{\"type\":\"pending\",\"url\":\"<url>\",\"kind\":\"captcha|login|required_field\",\"description\":\"what's missing\"}}\n\
                 Always emit the `job` line before the `application`/`pending` for the same job (the `url` correlates them).",
                profile = profile_block(profile, locale),
                label = variant.label,
                query = variant.query,
                policy = policy,
                language = language,
            )
        }
        Locale::PtBr => {
            // Application instruction according to the mode chosen by the user.
            let politica = if dry_run {
                "## MODO SIMULAÇÃO (dry-run)\n\
                 NÃO submeta NENHUMA candidatura e NÃO altere os sites (não clique em enviar/aplicar).\n\
                 Apenas busque, avalie e gere CV/carta. Reporte cada vaga com status \"dry_run\"."
                    .to_string()
            } else {
                match apply_mode {
                    "autonomous" => "## Política: AUTÔNOMO\n\
                         Para fit >= 0.7: gere CV/carta e SUBMETA a candidatura no site (status \"applied\";\n\
                         se falhar, \"failed\"). Bloqueios (captcha/login/campo) → reporte pending."
                        .to_string(),
                    "hybrid" => format!(
                        "## Política: HÍBRIDO\n\
                         Para fit >= {th:.2}: gere CV/carta e SUBMETA (status \"applied\").\n\
                         Para 0.7 <= fit < {th:.2}: gere CV/carta mas NÃO submeta — reporte status \"ready\"\n\
                         (aguardando aprovação). Bloqueios → reporte pending.",
                        th = hybrid_threshold
                    ),
                    // "review" (default and safest)
                    _ => "## Política: REVISÃO (aprovação manual)\n\
                         NÃO submeta nenhuma candidatura (não clique em enviar/aplicar). Para fit >= 0.7:\n\
                         gere CV/carta e reporte status \"ready\" (o usuário aprova depois). Bloqueios → pending."
                        .to_string(),
                }
            };

            // Optional language filter: restrict to jobs in the active language.
            let idioma = if language_filter {
                "## Idioma: APENAS pt-BR\n\
                 Considere SOMENTE vagas cuja descrição/requisitos estejam em português (Brasil).\n\
                 Se a vaga estiver em inglês (ou exigir inglês fluente no processo), NÃO candidate:\n\
                 emita a linha `job` normalmente e reporte a `application` com status \"skipped\".\n\n"
            } else {
                ""
            };

            format!(
                "Você é o agente do jobRabbit, que ajuda este candidato a se candidatar a vagas.\n\n\
                 IMPORTANTE: use a integração **Claude in Chrome** (o Chrome REAL do usuário, já logado).\n\
                 NÃO use Playwright nem abra um navegador próprio — use as ferramentas do Chrome conectado.\n\n\
                 {perfil}\n\
                 ## Tarefa\n\
                 Use o Chrome para buscar vagas correspondentes à variante:\n\
                 - Rótulo: {label}\n\
                 - Busca: {query}\n\n\
                 Para cada vaga relevante, avalie o *fit* (0.0 a 1.0) contra o perfil (senioridade, stack,\n\
                 modelo de trabalho, requisitos).\n\n\
                 {politica}\n\n\
                 {idioma}\
                 ## Protocolo de saída (OBRIGATÓRIO)\n\
                 Emita UMA linha JSON por evento (NDJSON), sem texto fora das linhas, sem cercas de código.\n\
                 Tipos:\n\
                 - Vaga encontrada/avaliada:\n\
                   {{\"type\":\"job\",\"title\":\"...\",\"company\":\"...\",\"url\":\"...\",\"source\":\"linkedin|gupy|indeed|...\",\"description\":\"resumo\",\"fit_score\":0.0}}\n\
                 - Candidatura (status conforme a política: applied|ready|dry_run|skipped|failed):\n\
                   {{\"type\":\"application\",\"url\":\"<url da vaga>\",\"status\":\"ready\",\"cv\":\"<cv gerado>\",\"cover\":\"<carta>\"}}\n\
                 - Pendência (precisa do usuário):\n\
                   {{\"type\":\"pending\",\"url\":\"<url>\",\"kind\":\"captcha|login|required_field\",\"description\":\"o que falta\"}}\n\
                 Sempre emita a linha `job` antes da `application`/`pending` da mesma vaga (a `url` correlaciona).",
                perfil = profile_block(profile, locale),
                label = variant.label,
                query = variant.query,
                idioma = idioma,
            )
        }
    }
}

/// Formats the bank of known answers to inject into the prompt.
pub fn answers_block(map: &std::collections::HashMap<String, String>, locale: Locale) -> String {
    if map.is_empty() {
        return match locale {
            Locale::En => {
                "(answer bank is empty — ask whatever you need via answer_needed)".to_string()
            }
            Locale::PtBr => {
                "(banco de respostas vazio — pergunte o que precisar via answer_needed)".to_string()
            }
        };
    }
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    let mut out = String::new();
    for k in keys {
        out.push_str(&format!("- {k}: {}\n", map[k]));
    }
    out
}

/// Prompt to apply for a job by pasting its URL (standalone flow).
/// Detects the ATS from the page, extracts the job details, automatically detects
/// the job description language, produces CV/cover letter in THAT language (not the UI locale),
/// evaluates fit, and either applies directly (if dry_run=false) or simulates (if dry_run=true).
#[allow(clippy::too_many_arguments)]
pub fn apply_by_url(
    url: &str,
    cv_file_path: &str,
    answers: &str,
    dry_run: bool,
    locale: Locale,
) -> String {
    let dry_run_hint = if dry_run {
        "\n- DRY-RUN MODE: simulate the entire flow but do NOT submit (status: \"dry_run\")."
    } else {
        "\n- LIVE MODE: submit the application after confirming it looks good."
    };

    let file_block = if cv_file_path.is_empty() {
        "If the site requires a résumé file UPLOAD and none is available, report\n\
         pending kind=\"required_field\" explaining that the CV file is missing."
            .to_string()
    } else {
        format!(
            "If the site requires a résumé UPLOAD, upload the file: {cv_file_path}\n\
             (use Chrome's file-upload tool to select it)."
        )
    };

    match locale {
        Locale::En => {
            format!(
                "You are the jobRabbit agent. The user wants to apply for a job using only its URL.\n\
                 Your mission is to EXTRACT the job details from the page, DETECT the job's language,\n\
                 and either APPLY DIRECTLY or SIMULATE (depending on dry-run mode).\n\n\
                 ## Execution flow\n\
                 1. Use Claude in Chrome to open the URL: {url}\n\
                 2. Extract job title, company name, and full job description (scroll if needed).\n\
                 3. **DETECT the LANGUAGE of the job description.** Determine if it's English, Portuguese,\n\
                    Spanish, or another language. This is CRITICAL.\n\
                 4. **Generate CV and cover letter IN THE DETECTED LANGUAGE** (NOT the UI locale).\n\
                    Use the answer bank and profile provided. Tailor both to the role.\n\
                 5. Detect the ATS platform from the page (inHire, LinkedIn, LinkedIn Jobs, etc.)\n\
                 6. Evaluate the job fit based on requirements vs. the candidate's profile.\n\
                 7. {dry_run_hint}\n\
                 8. If not dry-run, complete the full application flow per the platform playbook.\n\
                 9. After confirmation, capture a screenshot and include its absolute path.\n\n\
                 ## Candidate answer bank (use it to fill the fields)\n{answers}\n\n\
                 ## Execution rules\n\
                 - ALWAYS use the **Claude in Chrome** integration (REAL logged-in Chrome). NEVER use Playwright.\n\
                 - Drive the WHOLE flow per the ATS playbook, even multi-step (\"Next/Continue\").\n\
                 - Fill the fields with the answer bank, the CV/cover letter and the profile.\n\
                 - {file_block}\n\
                 - If a REQUIRED answer is missing and not in the bank (e.g. salary expectation),\n\
                   do NOT make it up: report `pending kind=\"answer_needed\"` with `field_key` (a short, stable\n\
                   slug, e.g. \"salary_expectation\") and the exact question in `description`. Stop the job.\n\
                 - Identity data the user provided in the answer bank (CPF, phone, full name,\n\
                   birth date, city/state) is the user's OWN data for their OWN application —\n\
                   FILL it into the form whenever present. Only emit `answer_needed` if it is\n\
                   absent. NEVER invent or guess a document number.\n\
                 - If login is needed and you are not logged in: `pending kind=\"login\"` (describe the platform).\n\
                 - Captcha: `pending kind=\"captcha\"` and skip.\n\
                 - Do NOT give up at the first obstacle: scroll the page, find the right button, advance steps.\n\
                 - Only report `applied` when you see an explicit CONFIRMATION (\"Application sent\"/\"submitted\").\n\
                 - After seeing the explicit submission confirmation, capture a screenshot via Claude in Chrome,\n\
                   save it as a PNG file, and include its absolute path in the `screenshot` field of the\n\
                   application JSON. BEST-EFFORT: if the screenshot cannot be captured, still report `applied`\n\
                   WITHOUT the screenshot field.\n\n\
                 ## Output (REQUIRED) — ONE JSON line per event, no text outside it:\n\
                 - Job extraction: {{\"type\":\"job\",\"url\":\"{url}\",\"title\":\"...\",\"company\":\"...\",\"detected_language\":\"en|pt|...\"}}\n\
                 - Submitted & CONFIRMED: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\"}}\n\
                 - With screenshot: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\",\"screenshot\":\"/absolute/path/to/screenshot.png\"}}\n\
                 - Dry-run: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"dry_run\"}}\n\
                 - Missing answer: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"answer_needed\",\"field_key\":\"salary_expectation\",\"description\":\"What is your salary expectation?\"}}\n\
                 - Blocker: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"login|captcha|required_field\",\"description\":\"what's missing\"}}\n\
                 - Not completed for another reason: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"failed\",\"description\":\"why\"}}\n\
                 When you discover a reusable candidate data point, you may emit: {{\"type\":\"answer\",\"key\":\"...\",\"label\":\"...\",\"value\":\"...\"}}"
            )
        }
        Locale::PtBr => {
            format!(
                "Você é o agente do jobRabbit. O usuário quer candidatar-se a uma vaga usando apenas sua URL.\n\
                 Sua missão é EXTRAIR os detalhes da vaga da página, DETECTAR o idioma da vaga,\n\
                 e CANDIDATAR-SE ou SIMULAR (dependendo do modo dry-run).\n\n\
                 ## Fluxo de execução\n\
                 1. Use o Claude in Chrome para abrir a URL: {url}\n\
                 2. Extraia título da vaga, nome da empresa e descrição completa da vaga (scrolle se necessário).\n\
                 3. **DETECTE o IDIOMA da descrição da vaga.** Determine se é inglês, português,\n\
                    espanhol ou outro idioma. Isso é CRÍTICO.\n\
                 4. **Gere CV e carta de apresentação NO IDIOMA DETECTADO** (NÃO a localidade da UI).\n\
                    Use o banco de respostas e perfil fornecidos. Adapte ambos ao cargo.\n\
                 5. Detecte a plataforma ATS na página (inHire, LinkedIn, LinkedIn Jobs, etc.)\n\
                 6. Avalie o fit da vaga com base nos requisitos vs. perfil do candidato.\n\
                 7. {dry_run_hint}\n\
                 8. Se não for dry-run, complete o fluxo completo de candidatura por playbook.\n\
                 9. Após confirmação, capture uma screenshot e inclua seu caminho absoluto.\n\n\
                 ## Banco de respostas do candidato (use para preencher os campos)\n{answers}\n\n\
                 ## Regras de execução\n\
                 - SEMPRE use a integração **Claude in Chrome** (Chrome real logado). NUNCA use Playwright.\n\
                 - Execute o FLUXO INTEIRO per o playbook ATS, mesmo multi-step (\"Próximo/Continuar\").\n\
                 - Preencha os campos com o banco de respostas, CV/carta e perfil.\n\
                 - {file_block}\n\
                 - Se uma resposta OBRIGATÓRIA estiver faltando e não estiver no banco (ex: expectativa salarial),\n\
                   NÃO invente: reporte `pending kind=\"answer_needed\"` com `field_key` (slug curto e estável,\n\
                   ex \"salary_expectation\") e a pergunta exata em `description`. Pare o trabalho.\n\
                 - Dados de identidade que o usuário forneceu no banco (CPF, telefone, nome completo,\n\
                   data nascimento, cidade/estado) são os PRÓPRIOS dados do usuário para SUA candidatura —\n\
                   PREENCHA no formulário quando presente. Só emita `answer_needed` se estiver ausente.\n\
                   NUNCA invente ou adivinhe um número de documento.\n\
                 - Se login é necessário e você não está logado: `pending kind=\"login\"` (descreva a plataforma).\n\
                 - Captcha: `pending kind=\"captcha\"` e passe.\n\
                 - NÃO desista no primeiro obstáculo: scrolle a página, ache o botão certo, avance passos.\n\
                 - Só reporte `applied` quando ver confirmação EXPLÍCITA (\"Candidatura enviada\"/\"submetido\").\n\
                 - Após a confirmação de submissão explícita, capture screenshot via Claude in Chrome,\n\
                   salve como arquivo PNG e inclua seu caminho absoluto no campo `screenshot` da\n\
                   aplicação JSON. BEST-EFFORT: se a screenshot não puder ser capturada, ainda reporte `applied`\n\
                   SEM o campo screenshot.\n\n\
                 ## Saída (OBRIGATÓRIO) — UMA linha JSON por evento, sem texto fora:\n\
                 - Extração de vaga: {{\"type\":\"job\",\"url\":\"{url}\",\"title\":\"...\",\"company\":\"...\",\"detected_language\":\"en|pt|...\"}}\n\
                 - Submetido & CONFIRMADO: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\"}}\n\
                 - Com screenshot: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\",\"screenshot\":\"/absolute/path/to/screenshot.png\"}}\n\
                 - Dry-run: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"dry_run\"}}\n\
                 - Resposta faltando: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"answer_needed\",\"field_key\":\"salary_expectation\",\"description\":\"Qual sua expectativa salarial?\"}}\n\
                 - Bloqueador: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"login|captcha|required_field\",\"description\":\"o que falta\"}}\n\
                 - Não completado por outro motivo: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"failed\",\"description\":\"por quê\"}}\n\
                 Quando descobrir um ponto de dado reutilizável do candidato, pode emitir: {{\"type\":\"answer\",\"key\":\"...\",\"label\":\"...\",\"value\":\"...\"}}"
            )
        }
    }
}

/// Prompt to ACTUALLY SUBMIT an approved application (the approval step),
/// ATS-aware: receives the platform playbook and the answer bank.
#[allow(clippy::too_many_arguments)]
pub fn apply_for_job(
    job_title: &str,
    company: &str,
    url: &str,
    cv: &str,
    cover: &str,
    cv_file_path: &str,
    ats_name: &str,
    playbook: &str,
    answers: &str,
    locale: Locale,
) -> String {
    match locale {
        Locale::En => {
            let file_block = if cv_file_path.is_empty() {
                "If the site requires a résumé file UPLOAD and none is available, report\n\
                 pending kind=\"required_field\" explaining that the CV file is missing."
                    .to_string()
            } else {
                format!(
                    "If the site requires a résumé UPLOAD, upload the file: {cv_file_path}\n\
                     (use Chrome's file-upload tool to select it)."
                )
            };

            format!(
                "You are the jobRabbit agent. The user APPROVED this application — your mission is\n\
                 to actually COMPLETE the application on the site, from start to finish.\n\n\
                 ## Detected platform: {ats_name}\n\
                 Follow the PLAYBOOK below (recipe specific to this platform):\n\
                 ---\n{playbook}\n---\n\n\
                 ## Candidate answer bank (use it to fill the fields)\n{answers}\n\n\
                 ## Execution rules\n\
                 - ALWAYS use the **Claude in Chrome** integration (REAL logged-in Chrome). NEVER use Playwright.\n\
                 - Drive the WHOLE flow per the playbook, even multi-step (\"Next/Continue\").\n\
                 - Fill the fields with the answer bank, the CV/cover letter and the profile.\n\
                 - {file_block}\n\
                 - If a REQUIRED answer is missing and not in the bank (e.g. salary expectation),\n\
                   do NOT make it up: report `pending kind=\"answer_needed\"` with `field_key` (a short, stable\n\
                   slug, e.g. \"salary_expectation\") and the exact question in `description`. Stop the job.\n\
                 - Identity data the user provided in the answer bank (CPF, phone, full name,\n\
                   birth date, city/state) is the user's OWN data for their OWN application —\n\
                   FILL it into the form whenever present. Only emit `answer_needed` if it is\n\
                   absent. NEVER invent or guess a document number.\n\
                 - If login is needed and you are not logged in: `pending kind=\"login\"` (describe the platform).\n\
                 - Captcha: `pending kind=\"captcha\"` and skip.\n\
                 - Do NOT give up at the first obstacle: scroll the page, find the right button, advance steps.\n\
                 - Only report `applied` when you see an explicit CONFIRMATION (\"Application sent\"/\"submitted\").\n\
                 - After seeing the explicit submission confirmation, capture a screenshot via Claude in Chrome,\n\
                   save it as a PNG file, and include its absolute path in the `screenshot` field of the\n\
                   application JSON. BEST-EFFORT: if the screenshot cannot be captured, still report `applied`\n\
                   WITHOUT the screenshot field.\n\n\
                 ### Job\n- Role: {job_title}\n- Company: {company}\n- URL: {url}\n\n\
                 ### CV\n{cv}\n\n### Cover letter\n{cover}\n\n\
                 ## Output (REQUIRED) — ONE JSON line per event, no text outside it:\n\
                 - Sent and CONFIRMED: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\"}}\n\
                 - With screenshot: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\",\"screenshot\":\"/absolute/path/to/screenshot.png\"}}\n\
                 - Missing answer: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"answer_needed\",\"field_key\":\"salary_expectation\",\"description\":\"What is your salary expectation?\"}}\n\
                 - Blocker: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"login|captcha|required_field\",\"description\":\"what's missing\"}}\n\
                 - Not completed for another reason: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"failed\",\"description\":\"why\"}}\n\
                 When you discover a reusable candidate data point, you may emit: {{\"type\":\"answer\",\"key\":\"...\",\"label\":\"...\",\"value\":\"...\"}}",
            )
        }
        Locale::PtBr => {
            let bloco_arquivo = if cv_file_path.is_empty() {
                "Se o site exigir UPLOAD de arquivo de currículo e não houver um disponível, reporte\n\
                 pending kind=\"required_field\" explicando que falta o arquivo do CV."
                    .to_string()
            } else {
                format!(
                    "Se o site exigir UPLOAD de currículo, faça upload do arquivo: {cv_file_path}\n\
                     (use a ferramenta de upload de arquivo do Chrome para selecioná-lo)."
                )
            };

            format!(
                "Você é o agente do jobRabbit. O usuário APROVOU esta candidatura — sua missão é\n\
                 CONCLUIR a candidatura de verdade no site, do início ao fim.\n\n\
                 ## Plataforma detectada: {ats_name}\n\
                 Siga o PLAYBOOK abaixo (receita específica desta plataforma):\n\
                 ---\n{playbook}\n---\n\n\
                 ## Banco de respostas do candidato (use para preencher os campos)\n{answers}\n\n\
                 ## Regras de execução\n\
                 - Use SEMPRE a integração **Claude in Chrome** (Chrome REAL logado). NUNCA use Playwright.\n\
                 - Conduza TODO o fluxo conforme o playbook, mesmo multi-etapas (\"Próximo/Next/Continuar\").\n\
                 - Preencha os campos com o banco de respostas, o CV/carta e o perfil.\n\
                 - {bloco_arquivo}\n\
                 - Se faltar uma resposta OBRIGATÓRIA que não está no banco (ex.: pretensão salarial),\n\
                   NÃO invente: reporte `pending kind=\"answer_needed\"` com `field_key` (slug curto e\n\
                   estável, ex.: \"salary_expectation\") e a pergunta exata em `description`. Pare a vaga.\n\
                 - Dados de identidade que o usuário informou no banco de respostas (CPF, celular,\n\
                   nome completo, data de nascimento, cidade/estado) são dados do PRÓPRIO usuário\n\
                   para a PRÓPRIA candidatura — PREENCHA no formulário sempre que presentes. Só\n\
                   emita `answer_needed` se estiverem ausentes. NUNCA invente/adivinhe um número\n\
                   de documento.\n\
                 - Se precisar de login e não estiver logado: `pending kind=\"login\"` (descreva a plataforma).\n\
                 - Captcha: `pending kind=\"captcha\"` e pule.\n\
                 - NÃO desista no primeiro obstáculo: role a página, procure o botão certo, avance etapas.\n\
                 - Só reporte `applied` ao ver uma CONFIRMAÇÃO explícita (\"Candidatura enviada\"/\"submitted\").\n\
                 - Após ver a confirmação explícita de envio, capture uma screenshot via Claude in Chrome,\n\
                   salve como arquivo PNG, e inclua seu caminho absoluto no campo `screenshot` do JSON\n\
                   de candidatura. BEST-EFFORT: se a screenshot não puder ser capturada, ainda reporte `applied`\n\
                   SEM o campo screenshot.\n\n\
                 ### Vaga\n- Cargo: {job_title}\n- Empresa: {company}\n- URL: {url}\n\n\
                 ### CV\n{cv}\n\n### Carta de apresentação\n{cover}\n\n\
                 ## Saída (OBRIGATÓRIO) — UMA linha JSON por evento, sem texto fora dela:\n\
                 - Enviada e CONFIRMADA: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\"}}\n\
                 - Com screenshot: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"applied\",\"screenshot\":\"/caminho/absoluto/screenshot.png\"}}\n\
                 - Falta resposta: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"answer_needed\",\"field_key\":\"salary_expectation\",\"description\":\"Qual sua pretensão salarial?\"}}\n\
                 - Bloqueio: {{\"type\":\"pending\",\"url\":\"{url}\",\"kind\":\"login|captcha|required_field\",\"description\":\"o que falta\"}}\n\
                 - Não concluída por outro motivo: {{\"type\":\"application\",\"url\":\"{url}\",\"status\":\"failed\",\"description\":\"por quê\"}}\n\
                 Quando descobrir um dado reutilizável do candidato, pode emitir: {{\"type\":\"answer\",\"key\":\"...\",\"label\":\"...\",\"value\":\"...\"}}",
            )
        }
    }
}

/// Prompt to generate a tailored CV for a specific job.
pub fn generate_cv(
    profile: &Profile,
    job_title: &str,
    company: &str,
    job_description: &str,
    locale: Locale,
) -> String {
    match locale {
        Locale::En => format!(
            "{profile}\n\
             ## Task\n\
             Generate a tailored CV (plain text, ready to paste) for the job below, highlighting\n\
             the most relevant experience from the profile and using keywords from the description.\n\n\
             ### Job\n- Role: {job_title}\n- Company: {company}\n- Description:\n{job_description}\n\n\
             Reply with the CV content ONLY, in English, no comments.",
            profile = profile_block(profile, locale),
        ),
        Locale::PtBr => format!(
            "{perfil}\n\
             ## Tarefa\n\
             Gere um CV customizado (texto puro, pronto para colar) para a vaga abaixo, destacando\n\
             a experiência mais relevante do perfil e usando as palavras-chave da descrição.\n\n\
             ### Vaga\n- Cargo: {job_title}\n- Empresa: {company}\n- Descrição:\n{job_description}\n\n\
             Responda APENAS com o conteúdo do CV, em português, sem comentários.",
            perfil = profile_block(profile, locale),
        ),
    }
}

/// Prompt to generate a cover letter for a job.
pub fn generate_cover_letter(
    profile: &Profile,
    job_title: &str,
    company: &str,
    job_description: &str,
    locale: Locale,
) -> String {
    match locale {
        Locale::En => format!(
            "{profile}\n\
             ## Task\n\
             Write a short (max 4 paragraphs) personalized cover letter for the job,\n\
             connecting the candidate's experience to the company's needs. Professional, direct tone.\n\n\
             ### Job\n- Role: {job_title}\n- Company: {company}\n- Description:\n{job_description}\n\n\
             Reply with the cover letter text ONLY, in English.",
            profile = profile_block(profile, locale),
        ),
        Locale::PtBr => format!(
            "{perfil}\n\
             ## Tarefa\n\
             Escreva uma carta de apresentação curta (máx. 4 parágrafos) e personalizada para a vaga,\n\
             conectando a experiência do candidato às necessidades da empresa. Tom profissional e direto.\n\n\
             ### Vaga\n- Cargo: {job_title}\n- Empresa: {company}\n- Descrição:\n{job_description}\n\n\
             Responda APENAS com o texto da carta, em português.",
            perfil = profile_block(profile, locale),
        ),
    }
}

/// Prompt for the periodic feedback analysis.
pub fn analyze_feedback(profile: &Profile, results_summary: &str, locale: Locale) -> String {
    match locale {
        Locale::En => format!(
            "{profile}\n\
             ## Task\n\
             Based on the recent application results below, generate a short analysis and\n\
             actionable suggestions to improve the profile/search variants. Do NOT browse the web.\n\n\
             ### Recent results\n{results_summary}\n\n\
             ## Output format (REQUIRED)\n\
             Reply with a SINGLE JSON line (NDJSON), no text outside it, no fences:\n\
             {{\"type\":\"feedback\",\"summary\":\"summary up to 140 characters\",\"suggestions\":\"- suggestion 1\\n- suggestion 2\"}}",
            profile = profile_block(profile, locale),
        ),
        Locale::PtBr => format!(
            "{perfil}\n\
             ## Tarefa\n\
             Com base nos resultados recentes de candidaturas abaixo, gere uma análise curta e\n\
             sugestões acionáveis para melhorar o perfil/variantes de busca. NÃO navegue na web.\n\n\
             ### Resultados recentes\n{resumo}\n\n\
             ## Formato de saída (OBRIGATÓRIO)\n\
             Responda com UMA única linha JSON (NDJSON), sem texto fora dela, sem cercas:\n\
             {{\"type\":\"feedback\",\"summary\":\"resumo de até 140 caracteres\",\"suggestions\":\"- sugestão 1\\n- sugestão 2\"}}",
            perfil = profile_block(profile, locale),
            resumo = results_summary,
        ),
    }
}

/// Prompt to evaluate the résumé (ATS tab / resume checker). No browsing.
pub fn review_cv(cv_text: &str, target: Option<&str>, locale: Locale) -> String {
    match locale {
        Locale::En => {
            let target_block = match target {
                Some(a) if !a.trim().is_empty() => {
                    format!("## Target job (evaluate keyword MATCH against it)\n{a}\n",)
                }
                _ => "## No target job — evaluate only the overall ATS quality.\n".to_string(),
            };
            format!(
                "You are a résumé reviewer specialized in ATS. Do NOT browse the web.\n\n\
                 ## Scoring rubric (0-100)\n\
                 Evaluate the CV across these weighted ATS criteria:\n\
                 1. **Parseability & structure** (20 pts): Clear headings, no tables/multi-column layouts, \n\
                    easy for ATS parsers to follow.\n\
                 2. **Target keyword match** (25 pts, if target job given): Incorporates real keywords from \n\
                    job description; prioritizes exact matches (e.g. \"Python\", \"Kubernetes\", \"5+ years\").\n\
                 3. **Action verbs & strong bullets** (20 pts): Uses verbs like \"led\", \"delivered\", \n\
                    \"improved\", \"optimized\"; avoids passive language.\n\
                 4. **Quantified achievements** (15 pts): Bullets include numbers, percentages, metrics \n\
                    (e.g., \"Increased throughput by 40%\", \"Led team of 8\").\n\
                 5. **Contact & links present** (10 pts): Name, email, phone, LinkedIn/GitHub URLs visible \n\
                    and properly formatted.\n\
                 6. **Conciseness & relevance** (10 pts): No fluff; each line justifies its space.\n\n\
                 ## Target bar: 90/100\n\
                 - **≥90**: ATS-ready CV that passes screening filters and maximizes recruiter review.\n\
                 - **70-89**: Functional CV but with fixable gaps (missing keywords, weak bullets, or unclear structure).\n\
                 - **<70**: Significant issues that will likely be rejected by ATS filters (poor parsing, \n\
                    minimal quantification, or weak keyword match).\n\n\
                 {target_block}\n\
                 ## Résumé\n{cv_text}\n\n\
                 ## Output (REQUIRED) — a SINGLE JSON line (NDJSON), no text outside it, no fences:\n\
                 {{\"type\":\"cv_review\",\"score\":<0-100>,\"target\":\"<target summary or 'general'>\",\
                 \"report\":\"markdown: ## Score, ## Strengths, ## Issues, ## Suggestions (bullets)\"}}\n\
                 The `report` must use \\n for line breaks. Be specific and actionable. When explaining the score, \n\
                 reference the rubric above.",
            )
        }
        Locale::PtBr => {
            let bloco_alvo = match target {
                Some(a) if !a.trim().is_empty() => {
                    format!("## Vaga-alvo (avalie o MATCH de keywords contra ela)\n{a}\n",)
                }
                _ => "## Sem vaga-alvo — avalie apenas a qualidade ATS geral.\n".to_string(),
            };
            format!(
                "Você é um avaliador de currículos especialista em ATS. NÃO navegue na web.\n\n\
                 ## Rubrica de pontuação (0-100)\n\
                 Avalie o CV nestes critérios ATS ponderados:\n\
                 1. **Parseabilidade & estrutura** (20 pts): Headings claros, sem tabelas/layouts com múltiplas \n\
                    colunas, fácil para parsers ATS.\n\
                 2. **Match de palavras-chave do alvo** (25 pts, se vaga-alvo fornecida): Incorpora keywords \n\
                    reais da descrição; prioriza matches exatos (ex.: \"Python\", \"Kubernetes\", \"5+ anos\").\n\
                 3. **Verbos de ação & bullets fortes** (20 pts): Usa verbos como \"liderou\", \"entregou\", \n\
                    \"melhorou\", \"otimizou\"; evita linguagem passiva.\n\
                 4. **Realizações quantificadas** (15 pts): Bullets incluem números, percentuais, métricas \n\
                    (ex.: \"Aumentou throughput em 40%\", \"Liderou time de 8 pessoas\").\n\
                 5. **Contato & links presentes** (10 pts): Nome, email, telefone, URLs LinkedIn/GitHub \n\
                    visíveis e bem formatados.\n\
                 6. **Concisão & relevância** (10 pts): Sem clichês; cada linha justifica seu espaço.\n\n\
                 ## Barra-alvo: 90/100\n\
                 - **≥90**: CV pronto para ATS, passa pelos filtros de triagem e maximiza revisão por recrutador.\n\
                 - **70-89**: CV funcional mas com gaps corrigíveis (keywords faltando, bullets fracos, \n\
                    ou estrutura pouco clara).\n\
                 - **<70**: Problemas significativos que provavelmente serão rejeitados pelos filtros ATS \n\
                    (parsing ruim, quantificação mínima, ou match de keywords fraco).\n\n\
                 {bloco_alvo}\n\
                 ## Currículo\n{cv_text}\n\n\
                 ## Saída (OBRIGATÓRIO) — UMA única linha JSON (NDJSON), sem texto fora dela, sem cercas:\n\
                 {{\"type\":\"cv_review\",\"score\":<0-100>,\"target\":\"<resumo do alvo ou 'geral'>\",\
                 \"report\":\"markdown: ## Nota, ## Pontos fortes, ## Problemas, ## Sugestões (bullets)\"}}\n\
                 O `report` deve usar \\n para quebras de linha. Seja específico e acionável. Ao explicar a nota, \n\
                 referencie a rubrica acima.",
            )
        }
    }
}

/// Prompt to GENERATE an improved version of the résumé (ATS-optimized).
/// No browsing — just rewrites based on the CV (and the target job, if any).
/// Instructs self-iteration internally to reach ≥90 score, max 3 passes.
pub fn improve_cv(cv_text: &str, target: Option<&str>, locale: Locale) -> String {
    match locale {
        Locale::En => {
            let target_block = match target {
                Some(a) if !a.trim().is_empty() => format!(
                    "## Target job (optimize the CV for this job, incorporating its real keywords)\n{a}\n",
                ),
                _ => "## No target job — optimize for overall ATS quality (keep it comprehensive).\n"
                    .to_string(),
            };
            format!(
                "You are a résumé and ATS expert. Do NOT browse the web and do NOT invent\n\
                 experiences, dates, companies or numbers that are not in the original CV —\n\
                 just REWRITE and REORGANIZE what already exists in a stronger way.\n\n\
                 ## Self-Iteration to ≥90\n\
                 Internally iterate on the CV rewrite using these steps (up to 3 passes maximum):\n\
                 1. Generate an improved version with ATS best practices (clear structure, action verbs, \n\
                    quantified metrics, relevant keywords).\n\
                 2. Mentally self-score the rewritten CV against the ATS rubric:\n\
                    - Parseability & structure (20 pts): Clear headings, no tables/columns.\n\
                    - Keyword match (25 pts): Incorporates target keywords if a job is given.\n\
                    - Action verbs (20 pts): Strong, active language.\n\
                    - Quantified achievements (15 pts): Includes numbers and metrics.\n\
                    - Contact & links (10 pts): Proper formatting, all present.\n\
                    - Conciseness (10 pts): No filler.\n\
                    TARGET: ≥90/100 to be ATS-ready.\n\
                 3. If your self-assessment is below 90, revise the CV and iterate again (step 1–2).\n\
                 4. Stop after 3 passes OR when you assess the CV at ≥90; do not exceed 3 iterations.\n\n\
                 Generate an IMPROVED version of the résumé below:\n\
                 - Clean, ATS-parseable structure (clear headings, no tables/columns).\n\
                 - Bullets with action verbs and QUANTIFIED achievements (use the original's numbers).\n\
                 - Incorporate relevant keywords (from the target job, if any).\n\
                 - English, concise, ready to send.\n\n\
                 {target_block}\n\
                 ## Original résumé\n{cv_text}\n\n\
                 ## Output (REQUIRED) — TWO JSON lines (NDJSON), no text outside them, no fences:\n\
                 First, emit the improved CV:\n\
                 {{\"type\":\"cv_version\",\"target\":\"<target summary or 'general'>\",\
                 \"content\":\"<full improved résumé in MARKDOWN>\"}}\n\
                 Then, emit your final self-assessment as a cv_review:\n\
                 {{\"type\":\"cv_review\",\"score\":<your assessed score ≥90>,\"target\":\"<same as above>\",\
                 \"report\":\"markdown: ## Score, ## Strengths, ## What was improved (bullets)\"}}\n\
                 In both `content` and `report`, use \\n for line breaks. Deliver the entire CV in the \n\
                 cv_version content, and explain your self-assessment logic in the review.",
            )
        }
        Locale::PtBr => {
            let bloco_alvo = match target {
                Some(a) if !a.trim().is_empty() => format!(
                    "## Vaga-alvo (otimize o CV para esta vaga, incorporando keywords reais dela)\n{a}\n",
                ),
                _ => "## Sem vaga-alvo — otimize para qualidade ATS geral (mantenha abrangente).\n"
                    .to_string(),
            };
            format!(
                "Você é um especialista em currículos e ATS. NÃO navegue na web e NÃO invente\n\
                 experiências, datas, empresas ou números que não estejam no CV original —\n\
                 apenas REESCREVA e REORGANIZE o que já existe de forma mais forte.\n\n\
                 ## Auto-Iteração para ≥90\n\
                 Internamente, itere sobre a reescrita do CV usando estes passos (máximo de 3 passes):\n\
                 1. Gere uma versão melhorada com as melhores práticas ATS (estrutura clara, verbos de ação, \n\
                    métricas quantificadas, keywords relevantes).\n\
                 2. Auto-avalie mentalmente o CV reescrito conforme a rubrica ATS:\n\
                    - Parseabilidade & estrutura (20 pts): Headings claros, sem tabelas/colunas.\n\
                    - Match de keywords (25 pts): Incorpora keywords da vaga se fornecida.\n\
                    - Verbos de ação (20 pts): Linguagem forte e ativa.\n\
                    - Realizações quantificadas (15 pts): Inclui números e métricas.\n\
                    - Contato & links (10 pts): Formatação correta, todos presentes.\n\
                    - Concisão (10 pts): Sem conteúdo desnecessário.\n\
                    META: ≥90/100 para estar pronto para ATS.\n\
                 3. Se sua auto-avaliação for abaixo de 90, revise o CV e itere novamente (passos 1–2).\n\
                 4. Pare após 3 passes OU quando avaliar o CV em ≥90; não ultrapasse 3 iterações.\n\n\
                 Gere uma versão MELHORADA do currículo abaixo:\n\
                 - Estrutura limpa e parseável por ATS (headings claros, sem tabelas/colunas).\n\
                 - Bullets com verbos de ação e conquistas QUANTIFICADAS (use os números do original).\n\
                 - Incorpore palavras-chave relevantes (da vaga-alvo, se houver).\n\
                 - Português, conciso, pronto para enviar.\n\n\
                 {bloco_alvo}\n\
                 ## Currículo original\n{cv_text}\n\n\
                 ## Saída (OBRIGATÓRIO) — DUAS linhas JSON (NDJSON), sem texto fora delas, sem cercas:\n\
                 Primeiro, emita o CV melhorado:\n\
                 {{\"type\":\"cv_version\",\"target\":\"<resumo do alvo ou 'geral'>\",\
                 \"content\":\"<currículo melhorado completo em MARKDOWN>\"}}\n\
                 Depois, emita sua auto-avaliação final como cv_review:\n\
                 {{\"type\":\"cv_review\",\"score\":<sua nota avaliada ≥90>,\"target\":\"<mesmo que acima>\",\
                 \"report\":\"markdown: ## Nota, ## Pontos fortes, ## O que foi melhorado (bullets)\"}}\n\
                 Em `content` e `report`, use \\n para quebras de linha. Entregue o CV inteiro no \n\
                 cv_version content, e explique sua lógica de auto-avaliação na review.",
            )
        }
    }
}

/// Common output format for profile building (NDJSON `profile` line).
fn profile_format(locale: Locale) -> &'static str {
    match locale {
        Locale::En => {
            "## Output format (REQUIRED)\n\
             Reply with a SINGLE JSON line (NDJSON), no text outside it, no code fences:\n\
             {\"type\":\"profile\",\"background\":\"concise professional summary (2-4 sentences)\",\
             \"cv_base\":\"clean, structured CV as text\",\
             \"variants\":[{\"label\":\"short label\",\"query\":\"search terms\"}]}\n\
             Suggest 2 to 4 search variants consistent with the candidate's seniority and stack.\n\n\
             Then, for EACH screening data point you can infer from the material (do NOT make up\n\
             salary expectation), emit one extra answer line:\n\
             {\"type\":\"answer\",\"key\":\"<key>\",\"label\":\"<label>\",\"value\":\"<value>\"}\n\
             Useful keys: english_level, years_experience, education_level, linkedin_url, github_url,\n\
             work_model, preferred_city, authorized_work_br. One line per known data point."
        }
        Locale::PtBr => {
            "## Formato de saída (OBRIGATÓRIO)\n\
             Responda com UMA única linha JSON (NDJSON), sem texto fora dela, sem cercas de código:\n\
             {\"type\":\"profile\",\"background\":\"resumo profissional conciso (2-4 frases)\",\
             \"cv_base\":\"CV limpo e estruturado em texto\",\
             \"variants\":[{\"label\":\"rótulo curto\",\"query\":\"termos de busca\"}]}\n\
             Sugira de 2 a 4 variantes de busca coerentes com a senioridade e a stack do candidato.\n\n\
             Em seguida, para CADA dado de triagem que você conseguir inferir do material (NÃO invente\n\
             pretensão salarial), emita uma linha extra de resposta:\n\
             {\"type\":\"answer\",\"key\":\"<chave>\",\"label\":\"<rótulo>\",\"value\":\"<valor>\"}\n\
             Chaves úteis: english_level, years_experience, education_level, linkedin_url, github_url,\n\
             work_model, preferred_city, authorized_work_br. Uma linha por dado conhecido."
        }
    }
}

/// Prompt to build the profile from the TEXT of a résumé (PDF/DOCX/TXT).
pub fn build_profile(cv_text: &str, locale: Locale) -> String {
    match locale {
        Locale::En => format!(
            "You are the jobRabbit agent. Do NOT browse the web or use tools.\n\
             From the résumé below, build the candidate profile.\n\n\
             ### Résumé (extracted text)\n{cv_text}\n\n\
             {format}",
            format = profile_format(locale),
        ),
        Locale::PtBr => format!(
            "Você é o agente do jobRabbit. NÃO navegue na web nem use ferramentas.\n\
             A partir do currículo abaixo, construa o perfil do candidato.\n\n\
             ### Currículo (texto extraído)\n{cv_text}\n\n\
             {formato}",
            formato = profile_format(locale),
        ),
    }
}

/// Prompt to build the profile using TWO sources: the CV text (primary) and
/// LinkedIn (enrichment via Chrome).
pub fn build_profile_combined(cv_text: &str, url: &str, locale: Locale) -> String {
    match locale {
        Locale::En => format!(
            "You are the jobRabbit agent. Build the candidate profile by combining TWO sources:\n\
             1) The CV below (primary, most complete source).\n\
             2) LinkedIn at {url} — use the Claude in Chrome integration (REAL logged-in Chrome,\n\
                NOT Playwright) to enrich/update (recent roles, skills, summary).\n\
                If you can't access it, use the CV only.\n\n\
             ### Résumé (extracted text)\n{cv_text}\n\n\
             {format}",
            format = profile_format(locale),
        ),
        Locale::PtBr => format!(
            "Você é o agente do jobRabbit. Construa o perfil do candidato combinando DUAS fontes:\n\
             1) O CV abaixo (fonte primária e mais completa).\n\
             2) O LinkedIn em {url} — use a integração Claude in Chrome (Chrome REAL logado,\n\
                NÃO Playwright) para enriquecer/atualizar (cargos recentes, skills, resumo).\n\
                Se não conseguir acessar, use só o CV.\n\n\
             ### Currículo (texto extraído)\n{cv_text}\n\n\
             {formato}",
            formato = profile_format(locale),
        ),
    }
}

/// Prompt to build the profile by browsing the candidate's LinkedIn.
pub fn build_profile_from_linkedin(url: &str, locale: Locale) -> String {
    match locale {
        Locale::En => format!(
            "You are the jobRabbit agent. Use the **Claude in Chrome** integration (REAL Chrome,\n\
             already logged in, NOT Playwright) to open the LinkedIn\n\
             profile at {url} and extract experience, roles, skills and summary.\n\
             With that, build the candidate profile.\n\n\
             {format}",
            format = profile_format(locale),
        ),
        Locale::PtBr => format!(
            "Você é o agente do jobRabbit. Use a integração **Claude in Chrome** (Chrome REAL já\n\
             logado, NÃO Playwright) para abrir o perfil do\n\
             LinkedIn em {url} e extrair experiência, cargos, skills e resumo.\n\
             Com isso, construa o perfil do candidato.\n\n\
             {formato}",
            formato = profile_format(locale),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Profile, SearchVariant};

    fn sample_profile() -> Profile {
        Profile {
            background: "Dev backend 8 anos, Rust/Go, remoto".into(),
            cv_base: "Experiência: Acme, Globex".into(),
            updated_at: "2026-01-01".into(),
        }
    }

    #[test]
    fn search_includes_profile_and_variant() {
        let v = SearchVariant {
            id: 1,
            label: "Senior Remote".into(),
            query: "senior rust remote".into(),
            enabled: true,
            created_at: "x".into(),
        };
        let p = search_and_evaluate(
            &sample_profile(),
            &v,
            "review",
            false,
            0.9,
            Locale::En,
            false,
        );
        assert!(p.contains("Rust/Go"));
        assert!(p.contains("Senior Remote"));
        assert!(p.contains("senior rust remote"));
        assert!(p.contains("NDJSON"));
        assert!(p.contains("fit_score"));
    }

    #[test]
    fn search_applies_language_filter() {
        let v = SearchVariant {
            id: 1,
            label: "L".into(),
            query: "q".into(),
            enabled: true,
            created_at: "x".into(),
        };
        let without = search_and_evaluate(
            &sample_profile(),
            &v,
            "review",
            false,
            0.9,
            Locale::En,
            false,
        );
        assert!(!without.contains("Language: English only"));

        let with = search_and_evaluate(
            &sample_profile(),
            &v,
            "review",
            false,
            0.9,
            Locale::En,
            true,
        );
        assert!(with.contains("Language: English only"));
        assert!(with.contains("skipped"));

        // pt-BR locale keeps the original Portuguese filter text.
        let pt = search_and_evaluate(
            &sample_profile(),
            &v,
            "review",
            false,
            0.9,
            Locale::PtBr,
            true,
        );
        assert!(pt.contains("APENAS pt-BR"));
    }

    #[test]
    fn search_respects_apply_mode() {
        let v = SearchVariant {
            id: 1,
            label: "L".into(),
            query: "q".into(),
            enabled: true,
            created_at: "x".into(),
        };
        let review = search_and_evaluate(
            &sample_profile(),
            &v,
            "review",
            false,
            0.9,
            Locale::En,
            false,
        );
        assert!(review.contains("REVIEW"));
        assert!(review.contains("Do NOT submit"));

        let auto = search_and_evaluate(
            &sample_profile(),
            &v,
            "autonomous",
            false,
            0.9,
            Locale::En,
            false,
        );
        assert!(auto.contains("AUTONOMOUS"));
        assert!(auto.contains("SUBMIT"));

        let dry = search_and_evaluate(
            &sample_profile(),
            &v,
            "autonomous",
            true,
            0.9,
            Locale::En,
            false,
        );
        assert!(dry.contains("SIMULATION") || dry.contains("dry-run"));
        assert!(dry.contains("dry_run"));

        let apply = apply_for_job(
            "Dev",
            "Acme",
            "https://x/1",
            "cv",
            "cover",
            "/cv.pdf",
            "Gupy",
            "PLAYBOOK_GUPY_HERE",
            "- english_level: advanced\n",
            Locale::En,
        );
        assert!(apply.contains("APPROVED"));
        assert!(apply.contains("https://x/1"));
        assert!(apply.contains("/cv.pdf"), "must instruct file upload");
        assert!(
            apply.contains("PLAYBOOK_GUPY_HERE"),
            "must inject the playbook"
        );
        assert!(
            apply.contains("english_level"),
            "must inject the answer bank"
        );
        assert!(apply.contains("answer_needed"));
        assert!(
            apply.contains("CPF"),
            "must include the identity-data fill policy"
        );
    }

    #[test]
    fn review_cv_includes_target_and_format() {
        let general = review_cv("my cv", None, Locale::En);
        assert!(general.contains("cv_review"));
        assert!(general.to_lowercase().contains("parseability"));
        assert!(general.contains("90"), "EN review_cv must mention target bar of 90");
        let with_target = review_cv("my cv", Some("Eng Manager Kafka"), Locale::En);
        assert!(with_target.contains("Eng Manager Kafka"));
        assert!(with_target.contains("match"));
    }

    #[test]
    fn improve_cv_iterates_to_90() {
        let en_prompt = improve_cv("my cv", Some("Senior Backend"), Locale::En);
        assert!(en_prompt.contains("cv_version"), "must emit cv_version");
        assert!(en_prompt.contains("cv_review"), "must emit cv_review after iteration");
        assert!(en_prompt.contains("90"), "must target 90 score in iteration");
        assert!(en_prompt.contains("iterate") || en_prompt.contains("pass"),
                "must mention iteration/passes for self-evaluation");

        let pt_prompt = improve_cv("meu cv", Some("Gerente Sênior"), Locale::PtBr);
        assert!(pt_prompt.contains("cv_version"));
        assert!(pt_prompt.contains("cv_review"));
        assert!(pt_prompt.contains("90"), "PT improve_cv must target 90 score");
    }

    #[test]
    fn answers_block_formats() {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert("english_level".to_string(), "advanced".to_string());
        let b = answers_block(&m, Locale::En);
        assert!(b.contains("english_level: advanced"));
        assert!(answers_block(&HashMap::new(), Locale::En).contains("empty"));
    }

    #[test]
    fn cv_and_cover_include_job_data() {
        let cv = generate_cv(
            &sample_profile(),
            "Staff Engineer",
            "Acme",
            "Rust, Kafka",
            Locale::En,
        );
        assert!(cv.contains("Staff Engineer"));
        assert!(cv.contains("Acme"));
        let cover = generate_cover_letter(
            &sample_profile(),
            "Staff Engineer",
            "Acme",
            "Rust, Kafka",
            Locale::En,
        );
        assert!(cover.contains("cover letter"));
        assert!(cover.contains("Acme"));
    }

    #[test]
    fn build_profile_includes_cv_and_format() {
        let p = build_profile("João Dev — Rust, 10 anos", Locale::En);
        assert!(p.contains("João Dev"));
        assert!(p.contains("\"type\":\"profile\""));
        assert!(p.contains("variants"));
        let l = build_profile_from_linkedin("https://linkedin.com/in/joao", Locale::En);
        assert!(l.contains("linkedin.com/in/joao"));
        assert!(l.contains("\"type\":\"profile\""));
    }

    #[test]
    fn empty_profile_does_not_break() {
        let p = Profile::default();
        let out = generate_cv(&p, "Dev", "X", "desc", Locale::En);
        assert!(out.contains("(not provided)"));
    }

    #[test]
    fn apply_by_url_mentions_language_and_url() {
        let prompt = apply_by_url(
            "https://example.com/job/123",
            "/cv.pdf",
            "- english_level: advanced\n",
            false,
            Locale::En,
        );
        assert!(prompt.contains("https://example.com/job/123"), "must include the URL");
        assert!(
            prompt.to_lowercase().contains("language"),
            "must mention language detection"
        );
        assert!(prompt.contains("application"), "must mention application");
        assert!(prompt.contains("/cv.pdf"), "must include cv_file_path");
        assert!(prompt.contains("english_level"), "must inject answer bank");
        assert!(
            prompt.contains("DETECT the LANGUAGE"),
            "must instruct language detection"
        );
        assert!(
            prompt.contains("detected_language"),
            "must output detected_language field"
        );
        assert!(prompt.contains("Claude in Chrome"), "must use Chrome");

        // Test Portuguese locale
        let pt_prompt = apply_by_url(
            "https://example.com/job/456",
            "",
            "- english_level: avançado\n",
            true,
            Locale::PtBr,
        );
        assert!(pt_prompt.contains("https://example.com/job/456"));
        assert!(pt_prompt.to_lowercase().contains("idioma"));
        assert!(pt_prompt.contains("dry_run"));
    }
}
