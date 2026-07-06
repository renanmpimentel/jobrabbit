//! Environment diagnostic ("doctor"): checks dependencies and configuration.
//!
//! Shared logic between CLI (`--doctor`) and web (`GET /api/doctor`).
//! Each [`Check`] has a status and, when something is wrong, a correction hint.

use serde::Serialize;

use crate::config::Settings;
use crate::db::Db;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Status {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub name: String,
    pub status: Status,
    pub detail: String,
    /// How to fix, when `status != Ok`.
    pub hint: Option<String>,
}

impl Check {
    fn ok(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Ok,
            detail: detail.into(),
            hint: None,
        }
    }
    fn warn(name: &str, detail: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Warn,
            detail: detail.into(),
            hint: Some(hint.into()),
        }
    }
    fn fail(name: &str, detail: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Fail,
            detail: detail.into(),
            hint: Some(hint.into()),
        }
    }
}

/// First binary found in PATH among the candidates.
fn first_in_path(cands: &[&str]) -> Option<String> {
    cands
        .iter()
        .find_map(|b| crate::platform::which(b).map(|p| p.display().to_string()))
}

/// Locates a Chrome/Chromium install, per OS (PATH on Linux, app bundle on
/// macOS, Program Files on Windows).
fn find_chrome() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        first_in_path(&[
            "google-chrome",
            "google-chrome-stable",
            "chromium",
            "chromium-browser",
            "chrome",
        ])
    }
    #[cfg(target_os = "macos")]
    {
        [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ]
        .into_iter()
        .find(|p| std::path::Path::new(p).is_file())
        .map(|p| p.to_string())
        .or_else(|| first_in_path(&["google-chrome", "chromium"]))
    }
    #[cfg(target_os = "windows")]
    {
        ["ProgramFiles", "ProgramFiles(x86)", "LocalAppData"]
            .into_iter()
            .filter_map(std::env::var_os)
            .map(|base| {
                let mut p = std::path::PathBuf::from(base);
                p.push(r"Google\Chrome\Application\chrome.exe");
                p
            })
            .find(|p| p.is_file())
            .map(|p| p.display().to_string())
            .or_else(|| first_in_path(&["chrome"]))
    }
}

/// Runs all environment/configuration checks.
///
/// Names, details and hints follow `settings.locale`, so the CLI (`--doctor`)
/// and the web UI (health banner + Health page) speak the user's language.
pub fn run(db: &Db, settings: &Settings) -> Vec<Check> {
    let mut checks = Vec::new();
    let pt = settings.locale == crate::locale::Locale::PtBr;
    // English by default; pt-BR when the locale asks for it.
    let tr = |en: &'static str, ptbr: &'static str| if pt { ptbr } else { en };

    // 1) Claude Code CLI — mandatory.
    match crate::platform::which(&settings.claude_bin) {
        Some(p) => checks.push(Check::ok(
            "Claude Code CLI",
            if pt {
                format!("`{}` encontrado em {}", settings.claude_bin, p.display())
            } else {
                format!("`{}` found in {}", settings.claude_bin, p.display())
            },
        )),
        None => checks.push(Check::fail(
            "Claude Code CLI",
            if pt {
                format!("`{}` não está no PATH", settings.claude_bin)
            } else {
                format!("`{}` is not in PATH", settings.claude_bin)
            },
            tr(
                "Install Claude Code and authenticate (`claude`); or adjust the path in the Config tab.",
                "Instale o Claude Code e autentique (`claude`); ou ajuste o caminho na aba Config.",
            ),
        )),
    }

    // 2) Browser backend — Chrome is only required for the Claude in Chrome integration;
    //    the Playwright backend brings its own browser with a persistent profile.
    match settings.browser_backend {
        crate::config::BrowserBackend::Chrome => match find_chrome() {
            Some(p) => checks.push(Check::ok(
                "Google Chrome",
                if pt {
                    format!("encontrado em {p} — confirme que a extensão Claude in Chrome está instalada e logada")
                } else {
                    format!("found in {p} — confirm that the Claude in Chrome extension is installed and logged in")
                },
            )),
            None => checks.push(Check::warn(
                "Google Chrome",
                tr("browser not found", "navegador não encontrado"),
                tr(
                    "Install Google Chrome and the Claude in Chrome extension (automation depends on logged-in Chrome), or switch the browser backend to Playwright in the Config tab.",
                    "Instale o Google Chrome e a extensão Claude in Chrome (a automação depende do Chrome logado), ou troque o backend de navegador para Playwright na aba Config.",
                ),
            )),
        },
        crate::config::BrowserBackend::Playwright => checks.push(Check::ok(
            tr("Browser backend", "Backend de navegador"),
            tr(
                "Playwright MCP — Chrome and the Claude in Chrome extension are not required; log in once in the Playwright browser (profile persists)",
                "Playwright MCP — Chrome e a extensão Claude in Chrome não são necessários; faça login uma vez no navegador do Playwright (o perfil persiste)",
            ),
        )),
    }

    // 3) Graphical session — Chrome and automatic browser opening depend on this.
    #[cfg(target_os = "linux")]
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        checks.push(Check::ok(
            tr("Graphical session", "Sessão gráfica"),
            tr("DISPLAY/Wayland detected", "DISPLAY/Wayland detectado"),
        ));
    } else {
        checks.push(Check::warn(
            tr("Graphical session", "Sessão gráfica"),
            tr("no DISPLAY/WAYLAND_DISPLAY", "sem DISPLAY/WAYLAND_DISPLAY"),
            tr(
                "Run on your desktop (with graphical environment) for the UI to open and Chrome to work.",
                "Rode no seu desktop (com ambiente gráfico) para a UI abrir e o Chrome funcionar.",
            ),
        ));
    }
    #[cfg(not(target_os = "linux"))]
    checks.push(Check::ok(
        tr("Graphical session", "Sessão gráfica"),
        tr("desktop OS", "SO desktop"),
    ));

    // 4) Link opener — used to open jobs/browser automatically.
    #[cfg(target_os = "linux")]
    if crate::platform::which("xdg-open").is_some() {
        checks.push(Check::ok(
            tr("Open links", "Abrir links"),
            tr("xdg-open available", "xdg-open disponível"),
        ));
    } else {
        checks.push(Check::warn(
            tr("Open links", "Abrir links"),
            tr("`xdg-open` not found", "`xdg-open` não encontrado"),
            tr(
                "Install `xdg-utils` to open jobs and the browser automatically.",
                "Instale `xdg-utils` para abrir vagas e o navegador automaticamente.",
            ),
        ));
    }
    #[cfg(target_os = "macos")]
    checks.push(Check::ok(
        tr("Open links", "Abrir links"),
        tr("open available", "open disponível"),
    ));
    #[cfg(target_os = "windows")]
    checks.push(Check::ok(
        tr("Open links", "Abrir links"),
        tr("start available", "start disponível"),
    ));

    // 5) Candidate profile.
    let profile = db.get_profile().unwrap_or_default();
    if profile.background.trim().is_empty() && profile.cv_base.trim().is_empty() {
        checks.push(Check::warn(
            tr("Candidate profile", "Perfil do candidato"),
            tr(
                "background and base CV empty",
                "background e CV base vazios",
            ),
            tr(
                "Fill in the Profile or import your CV/LinkedIn (Profile tab).",
                "Preencha o Perfil ou importe seu CV/LinkedIn (aba Perfil).",
            ),
        ));
    } else {
        checks.push(Check::ok(
            tr("Candidate profile", "Perfil do candidato"),
            if pt {
                format!("{} caracteres de CV base", profile.cv_base.len())
            } else {
                format!("{} chars of base CV", profile.cv_base.len())
            },
        ));
    }

    // 6) Active search variants.
    let ativas = db
        .list_variants()
        .unwrap_or_default()
        .into_iter()
        .filter(|v| v.enabled)
        .count();
    if ativas == 0 {
        checks.push(Check::warn(
            tr("Search variants", "Variantes de busca"),
            tr("no active variant", "nenhuma variante ativa"),
            tr(
                "Add/enable at least one search variant in the Profile tab.",
                "Adicione/ative pelo menos uma variante de busca na aba Perfil.",
            ),
        ));
    } else {
        checks.push(Check::ok(
            tr("Search variants", "Variantes de busca"),
            if pt {
                format!("{ativas} ativa(s)")
            } else {
                format!("{ativas} active")
            },
        ));
    }

    // 7) CV file for upload (optional).
    let cv_path = settings.cv_file_path.trim();
    if cv_path.is_empty() {
        checks.push(Check::ok(
            tr("CV file (upload)", "Arquivo de CV (upload)"),
            tr(
                "not set (optional — only needed when the site requires upload)",
                "não configurado (opcional — só necessário quando o site exige upload)",
            ),
        ));
    } else if std::path::Path::new(cv_path).is_file() {
        checks.push(Check::ok(
            tr("CV file (upload)", "Arquivo de CV (upload)"),
            cv_path.to_string(),
        ));
    } else {
        checks.push(Check::warn(
            tr("CV file (upload)", "Arquivo de CV (upload)"),
            if pt {
                format!("o caminho configurado não existe: {cv_path}")
            } else {
                format!("the configured path does not exist: {cv_path}")
            },
            tr(
                "Point to a valid PDF/DOCX in the Config tab (or import a CV from file).",
                "Aponte para um PDF/DOCX válido na aba Config (ou importe um CV de arquivo).",
            ),
        ));
    }

    // 8) Database / data directory.
    match crate::config::data_dir() {
        Ok(dir) => checks.push(Check::ok(
            tr("Database", "Banco de dados"),
            if pt {
                format!("acessível em {}", dir.display())
            } else {
                format!("accessible in {}", dir.display())
            },
        )),
        Err(e) => checks.push(Check::fail(
            tr("Database", "Banco de dados"),
            if pt {
                format!("não foi possível resolver o diretório de dados: {e}")
            } else {
                format!("could not resolve data directory: {e}")
            },
            tr(
                "Check permissions of XDG data directory (~/.local/share/jobrabbit).",
                "Verifique as permissões do diretório de dados XDG (~/.local/share/jobrabbit).",
            ),
        )),
    }

    checks
}

/// Summary: `(ok, warn, fail)`.
pub fn summary(checks: &[Check]) -> (usize, usize, usize) {
    let mut t = (0, 0, 0);
    for c in checks {
        match c.status {
            Status::Ok => t.0 += 1,
            Status::Warn => t.1 += 1,
            Status::Fail => t.2 += 1,
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[test]
    fn run_cobre_checks_essenciais() {
        let db = Db::open_in_memory().unwrap();
        let settings = Settings::default();
        let checks = run(&db, &settings);
        let nomes: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        assert!(nomes.contains(&"Claude Code CLI"));
        assert!(nomes.contains(&"Search variants"));
        assert!(nomes.contains(&"Database"));
        // the summary adds up to the total checks
        let (ok, warn, fail) = summary(&checks);
        assert_eq!(ok + warn + fail, checks.len());
        // in-memory database always accessible
        let db_check = checks.iter().find(|c| c.name == "Database").unwrap();
        assert_eq!(db_check.status, Status::Ok);
        // empty profile and no variants → warnings
        assert!(checks
            .iter()
            .any(|c| c.name == "Search variants" && c.status == Status::Warn));
    }

    #[test]
    fn checks_follow_the_ptbr_locale() {
        let db = Db::open_in_memory().unwrap();
        let settings = Settings {
            locale: crate::locale::Locale::PtBr,
            ..Settings::default()
        };
        let checks = run(&db, &settings);
        let nomes: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        // Localized names (product names like "Claude Code CLI" stay as-is).
        assert!(nomes.contains(&"Perfil do candidato"));
        assert!(nomes.contains(&"Variantes de busca"));
        assert!(nomes.contains(&"Banco de dados"));
        assert!(!nomes.contains(&"Candidate profile"));
        // Hints localized too.
        let variantes = checks
            .iter()
            .find(|c| c.name == "Variantes de busca")
            .unwrap();
        assert!(variantes.hint.as_deref().unwrap().contains("variante"));
    }
}
