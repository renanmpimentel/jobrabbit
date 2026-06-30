//! Detection of ATS (job platform) by URL and loading the corresponding "playbook"
//! — the recipe for how to apply on that platform.
//!
//! Playbooks are data (markdown), one set per [`Locale`]. Defaults come embedded via
//! `include_str!`; they can be overridden by files in
//! `~/.local/share/jobrabbit/playbooks/<locale>/<slug>.md`.

use crate::locale::Locale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ats {
    Gupy,
    LinkedIn,
    Greenhouse,
    Lever,
    Workday,
    Ashby,
    SmartRecruiters,
    Indeed,
    Solides,
    Vagas,
    InfoJobs,
    InHire,
    Generic,
}

impl Ats {
    /// Human-readable platform name.
    pub fn name(self) -> &'static str {
        match self {
            Ats::Gupy => "Gupy",
            Ats::LinkedIn => "LinkedIn",
            Ats::Greenhouse => "Greenhouse",
            Ats::Lever => "Lever",
            Ats::Workday => "Workday",
            Ats::Ashby => "Ashby",
            Ats::SmartRecruiters => "SmartRecruiters",
            Ats::Indeed => "Indeed",
            Ats::Solides => "Solides Vagas",
            Ats::Vagas => "Vagas.com.br",
            Ats::InfoJobs => "InfoJobs",
            Ats::InHire => "inHire",
            Ats::Generic => "Generic",
        }
    }

    /// Slug used for playbook filename.
    pub fn slug(self) -> &'static str {
        match self {
            Ats::Gupy => "gupy",
            Ats::LinkedIn => "linkedin",
            Ats::Greenhouse => "greenhouse",
            Ats::Lever => "lever",
            Ats::Workday => "workday",
            Ats::InHire => "inhire",
            // Recognized ATS but no dedicated playbook yet → generic
            _ => "generic",
        }
    }
}

/// Detects the platform from the URL (case-insensitive, by domain/path).
pub fn detect(url: &str) -> Ats {
    let u = url.to_ascii_lowercase();
    let has = |needle: &str| u.contains(needle);

    if has("gupy.io") {
        Ats::Gupy
    } else if has("linkedin.com/jobs") {
        Ats::LinkedIn
    } else if has("greenhouse.io") {
        Ats::Greenhouse
    } else if has("jobs.lever.co") {
        Ats::Lever
    } else if has("myworkdayjobs.com") {
        Ats::Workday
    } else if has("ashbyhq.com") {
        Ats::Ashby
    } else if has("smartrecruiters.com") {
        Ats::SmartRecruiters
    } else if has("indeed.com") {
        Ats::Indeed
    } else if has("vagas.solides.com.br") {
        Ats::Solides
    } else if has("vagas.com.br") {
        Ats::Vagas
    } else if has("infojobs.com.br") {
        Ats::InfoJobs
    } else if has("inhire.") {
        Ats::InHire
    } else {
        Ats::Generic
    }
}

// Default embedded playbooks (English).
const PB_GUPY_EN: &str = include_str!("playbooks/en/gupy.md");
const PB_LINKEDIN_EN: &str = include_str!("playbooks/en/linkedin.md");
const PB_GREENHOUSE_EN: &str = include_str!("playbooks/en/greenhouse.md");
const PB_LEVER_EN: &str = include_str!("playbooks/en/lever.md");
const PB_WORKDAY_EN: &str = include_str!("playbooks/en/workday.md");
const PB_INHIRE_EN: &str = include_str!("playbooks/en/inhire.md");
const PB_GENERIC_EN: &str = include_str!("playbooks/en/generic.md");

// Default embedded playbooks (Brazilian Portuguese).
const PB_GUPY_PT_BR: &str = include_str!("playbooks/pt-br/gupy.md");
const PB_LINKEDIN_PT_BR: &str = include_str!("playbooks/pt-br/linkedin.md");
const PB_GREENHOUSE_PT_BR: &str = include_str!("playbooks/pt-br/greenhouse.md");
const PB_LEVER_PT_BR: &str = include_str!("playbooks/pt-br/lever.md");
const PB_WORKDAY_PT_BR: &str = include_str!("playbooks/pt-br/workday.md");
const PB_INHIRE_PT_BR: &str = include_str!("playbooks/pt-br/inhire.md");
const PB_GENERIC_PT_BR: &str = include_str!("playbooks/pt-br/generic.md");

fn embedded_playbook(slug: &str, locale: Locale) -> &'static str {
    match (locale, slug) {
        (Locale::En, "gupy") => PB_GUPY_EN,
        (Locale::En, "linkedin") => PB_LINKEDIN_EN,
        (Locale::En, "greenhouse") => PB_GREENHOUSE_EN,
        (Locale::En, "lever") => PB_LEVER_EN,
        (Locale::En, "workday") => PB_WORKDAY_EN,
        (Locale::En, "inhire") => PB_INHIRE_EN,
        (Locale::En, _) => PB_GENERIC_EN,
        (Locale::PtBr, "gupy") => PB_GUPY_PT_BR,
        (Locale::PtBr, "linkedin") => PB_LINKEDIN_PT_BR,
        (Locale::PtBr, "greenhouse") => PB_GREENHOUSE_PT_BR,
        (Locale::PtBr, "lever") => PB_LEVER_PT_BR,
        (Locale::PtBr, "workday") => PB_WORKDAY_PT_BR,
        (Locale::PtBr, "inhire") => PB_INHIRE_PT_BR,
        (Locale::PtBr, _) => PB_GENERIC_PT_BR,
    }
}

/// Returns the playbook for the ATS in the given locale. Prefers a user override in
/// `<data_dir>/playbooks/<locale>/<slug>.md`; otherwise uses the built-in default.
pub fn playbook(ats: Ats, locale: Locale) -> String {
    let slug = ats.slug();
    if let Ok(dir) = crate::config::data_dir() {
        let path = dir
            .join("playbooks")
            .join(locale.as_str())
            .join(format!("{slug}.md"));
        if let Ok(custom) = std::fs::read_to_string(&path) {
            if !custom.trim().is_empty() {
                return custom;
            }
        }
    }
    embedded_playbook(slug, locale).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_main_platforms() {
        assert_eq!(detect("https://company.gupy.io/jobs/123"), Ats::Gupy);
        assert_eq!(
            detect("https://www.linkedin.com/jobs/view/456"),
            Ats::LinkedIn
        );
        assert_eq!(
            detect("https://boards.greenhouse.io/acme/jobs/789"),
            Ats::Greenhouse
        );
        assert_eq!(detect("https://jobs.lever.co/acme/abc"), Ats::Lever);
        assert_eq!(
            detect("https://acme.wd1.myworkdayjobs.com/External/job/x"),
            Ats::Workday
        );
        assert_eq!(detect("https://random-site.com/job/1"), Ats::Generic);
    }

    #[test]
    fn detection_case_insensitive() {
        assert_eq!(detect("HTTPS://COMPANY.GUPY.IO/JOBS/1"), Ats::Gupy);
    }

    #[test]
    fn playbook_not_empty_for_all_locales() {
        for locale in [Locale::En, Locale::PtBr] {
            for ats in [
                Ats::Gupy,
                Ats::LinkedIn,
                Ats::Greenhouse,
                Ats::Lever,
                Ats::Workday,
                Ats::Generic,
                Ats::Indeed, // falls to generic
            ] {
                assert!(
                    !playbook(ats, locale).trim().is_empty(),
                    "empty playbook: {:?} / {:?}",
                    ats,
                    locale
                );
            }
        }
    }

    #[test]
    fn gupy_playbook_mentions_login() {
        assert!(playbook(Ats::Gupy, Locale::En)
            .to_lowercase()
            .contains("login"));
        assert!(playbook(Ats::Gupy, Locale::PtBr)
            .to_lowercase()
            .contains("login"));
    }

    #[test]
    fn detects_inhire() {
        assert_eq!(detect("https://flutter.inhire.app/jobs/123"), Ats::InHire);
        assert_eq!(detect("https://vaga.inhire.com.br/x"), Ats::InHire);
        assert_eq!(Ats::InHire.slug(), "inhire");
        assert_eq!(Ats::InHire.name(), "inHire");
    }

    #[test]
    fn inhire_playbook_not_empty() {
        assert!(!playbook(Ats::InHire, Locale::En).trim().is_empty());
        assert!(!playbook(Ats::InHire, Locale::PtBr).trim().is_empty());
        assert!(playbook(Ats::InHire, Locale::PtBr).contains("CPF"));
    }
}
