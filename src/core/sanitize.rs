//! Sanitization of answer-bank values.
//!
//! Values reach the bank from several writers (agent `answer` protocol lines,
//! the web Pending/Profile pages, the TUI input) and are later injected into
//! prompts and typed verbatim into form fields. A value pasted from a chat
//! transcript can carry metadata (`05:57▮▮▮Claude responded: ...`) or be
//! accidentally duplicated; both must never reach a real form.

/// Cleans an answer value before persisting or injecting into a prompt.
///
/// - Strips leading transcript metadata (timestamp, separator glyphs and
///   labels like "Claude responded:") from the first lines.
/// - Collapses exact whole-text and consecutive-paragraph duplication.
/// - Trims surrounding whitespace.
///
/// Clean text passes through unchanged.
pub fn sanitize_answer_value(raw: &str) -> String {
    let stripped = strip_metadata_prefix(raw.trim());
    let deduped = collapse_duplication(stripped.trim());
    deduped.trim().to_string()
}

/// Labels that mark a transcript line as "assistant output" (case-insensitive).
const LABELS: &[&str] = &[
    "claude responded:",
    "claude respondeu:",
    "claude:",
    "assistant:",
    "resposta:",
];

/// Removes a leading `HH:MM` + separators + label prefix, repeatedly, as long
/// as one is actually present. Returns the remainder unchanged otherwise.
fn strip_metadata_prefix(text: &str) -> &str {
    let mut rest = text;
    loop {
        let after = strip_one_prefix(rest);
        if after.len() == rest.len() {
            return rest;
        }
        rest = after;
    }
}

/// Strips at most one metadata prefix from the start of `text`.
fn strip_one_prefix(text: &str) -> &str {
    let mut s = text.trim_start();
    let had_timestamp = {
        let after = strip_timestamp(s);
        let matched = after.len() != s.len();
        s = after;
        matched
    };
    let before_sep = s;
    s = strip_separators(s);
    let removed_sep = &before_sep[..before_sep.len() - s.len()];
    let had_glyph_sep = removed_sep.chars().any(|c| !c.is_whitespace());
    let lower = s.to_lowercase();
    for label in LABELS {
        if lower.starts_with(label) {
            return s[label.len()..].trim_start();
        }
    }
    // A timestamp followed by separator GLYPHS is metadata even without a
    // label; a bare timestamp (e.g. an answer that IS a time) must be kept.
    if had_timestamp && had_glyph_sep && !s.is_empty() {
        return s;
    }
    text
}

/// Strips a leading `HH:MM` timestamp (2 digits, ':', 2 digits).
fn strip_timestamp(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 5
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == b':'
        && b[3].is_ascii_digit()
        && b[4].is_ascii_digit()
        && !b.get(5).is_some_and(|c| c.is_ascii_digit() || *c == b':')
    {
        &s[5..]
    } else {
        s
    }
}

/// Strips separator glyphs commonly found between transcript metadata and
/// content: spaces, block elements (U+2580–U+259F), geometric shapes such as
/// `▮` (U+25A0–U+25FF), box-drawing bars, dashes.
fn strip_separators(s: &str) -> &str {
    s.trim_start_matches(|c: char| {
        c.is_whitespace()
            || ('\u{2580}'..='\u{25FF}').contains(&c)
            || matches!(c, '│' | '|' | '—' | '–' | '-' | '·' | '•' | '›' | '>')
    })
}

/// Collapses exact duplication: whole text repeated twice, and consecutive
/// identical paragraphs.
fn collapse_duplication(text: &str) -> String {
    let halved = collapse_doubled_whole(text);
    collapse_repeated_paragraphs(&halved)
}

/// If `text` is exactly two copies of the same half (joined by optional
/// whitespace), returns one copy; otherwise returns `text` as-is.
///
/// Only prose-sized halves (≥ 12 chars) are collapsed, so short repetitive
/// codes like "123123" are never corrupted.
fn collapse_doubled_whole(text: &str) -> String {
    let t = text.trim();
    for (i, _) in t.char_indices().skip(1) {
        let (a, b) = t.split_at(i);
        let (a, b) = (a.trim(), b.trim());
        if a.chars().count() >= 12 && a == b {
            return a.to_string();
        }
    }
    t.to_string()
}

/// Removes consecutive paragraphs (split on blank lines) that are identical
/// after trimming.
fn collapse_repeated_paragraphs(text: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for para in text.split("\n\n") {
        if out.last().map(|p| p.trim()) == Some(para.trim()) && !para.trim().is_empty() {
            continue;
        }
        out.push(para);
    }
    out.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_timestamp_marker_and_label() {
        assert_eq!(
            sanitize_answer_value("05:57▮▮▮Claude responded: O que me motiva é liderar."),
            "O que me motiva é liderar."
        );
    }

    #[test]
    fn strips_pt_label_with_bar_separator() {
        assert_eq!(
            sanitize_answer_value("12:03 │ Claude respondeu: Minha resposta."),
            "Minha resposta."
        );
    }

    #[test]
    fn strips_label_without_timestamp() {
        assert_eq!(sanitize_answer_value("Assistant: hello there"), "hello there");
    }

    #[test]
    fn collapses_exact_doubled_text() {
        assert_eq!(
            sanitize_answer_value("Me motiva crescer. Me motiva crescer."),
            "Me motiva crescer."
        );
    }

    #[test]
    fn collapses_repeated_paragraphs() {
        assert_eq!(
            sanitize_answer_value("Primeiro parágrafo.\n\nPrimeiro parágrafo.\n\nSegundo."),
            "Primeiro parágrafo.\n\nSegundo."
        );
    }

    #[test]
    fn clean_text_passes_through() {
        let clean = "Tenho 18 anos de experiência, disponível às 09:00.";
        assert_eq!(sanitize_answer_value(clean), clean);
        assert_eq!(sanitize_answer_value("20000"), "20000");
        assert_eq!(sanitize_answer_value("Remoto (100%)"), "Remoto (100%)");
    }

    #[test]
    fn bare_timestamp_answer_is_kept() {
        // A legit answer that IS a time must not be eaten.
        assert_eq!(sanitize_answer_value("09:30"), "09:30");
    }

    #[test]
    fn multiline_clean_text_is_preserved() {
        let clean = "Linha um.\n\nLinha dois, diferente.";
        assert_eq!(sanitize_answer_value(clean), clean);
    }

    #[test]
    fn prefixed_and_doubled_combined() {
        assert_eq!(
            sanitize_answer_value(
                "05:57▮▮▮Claude responded: Me motiva liderar times. Me motiva liderar times."
            ),
            "Me motiva liderar times."
        );
    }

    #[test]
    fn short_repetitive_codes_are_not_collapsed() {
        assert_eq!(sanitize_answer_value("123123"), "123123");
        assert_eq!(sanitize_answer_value("abab"), "abab");
    }
}
