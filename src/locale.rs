//! Locale (language) selection for prompts, playbooks and the UI.
//!
//! English is the default. The active locale comes from [`crate::config::Settings`]
//! and is threaded into the prompt/playbook builders so the agent operates in the
//! chosen language (e.g. writing CVs/cover letters and reading job boards in pt-BR
//! when [`Locale::PtBr`] is selected).

use serde::{Deserialize, Serialize};

/// Supported UI / agent languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// English (default).
    #[default]
    En,
    /// Brazilian Portuguese.
    #[serde(rename = "pt-br", alias = "pt_br", alias = "ptbr")]
    PtBr,
}

/// Locales available to cycle through in the UI, in display order.
pub const LOCALES: [Locale; 2] = [Locale::En, Locale::PtBr];

impl Locale {
    /// Short tag used in paths and serialization (`"en"`, `"pt-br"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::PtBr => "pt-br",
        }
    }

    /// Human-readable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::PtBr => "Português (Brasil)",
        }
    }

    /// Next locale in [`LOCALES`] (wraps around) — used by the Config tab toggle.
    pub fn next(self) -> Locale {
        let i = LOCALES.iter().position(|l| *l == self).unwrap_or(0);
        LOCALES[(i + 1) % LOCALES.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_english() {
        assert_eq!(Locale::default(), Locale::En);
    }

    #[test]
    fn cycles_through_all_locales() {
        assert_eq!(Locale::En.next(), Locale::PtBr);
        assert_eq!(Locale::PtBr.next(), Locale::En);
    }

    #[test]
    fn serde_roundtrip_uses_short_tag() {
        let json = serde_json::to_string(&Locale::PtBr).unwrap();
        assert_eq!(json, "\"pt-br\"");
        let back: Locale = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Locale::PtBr);
    }
}
