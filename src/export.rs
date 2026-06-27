//! Export of resume (markdown) to PDF and DOCX.
//!
//! The agent delivers the improved CV in simple markdown (headings `#`, bullets
//! `-`/`*`, emphasis `**`). Here we do light parsing into blocks and render
//! to both formats. The fonts (DejaVu, with accents) are embedded.

use anyhow::Result;

const FONT_REGULAR: &[u8] = include_bytes!("../assets/fonts/DejaVuSans.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../assets/fonts/DejaVuSans-Bold.ttf");

/// Logical block of the document.
enum Block {
    Heading(u8, String),
    Bullet(String),
    Para(String),
    Blank,
}

/// Removes simple inline markers (bold/code) to plain text.
fn clean(s: &str) -> String {
    s.replace("**", "").replace('`', "").trim().to_string()
}

/// Does light parsing of markdown into blocks.
fn parse_md(content: &str) -> Vec<Block> {
    content
        .lines()
        .map(|raw| {
            let t = raw.trim_start();
            if t.trim().is_empty() {
                Block::Blank
            } else if let Some(r) = t.strip_prefix("### ") {
                Block::Heading(3, clean(r))
            } else if let Some(r) = t.strip_prefix("## ") {
                Block::Heading(2, clean(r))
            } else if let Some(r) = t.strip_prefix("# ") {
                Block::Heading(1, clean(r))
            } else if let Some(r) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
                Block::Bullet(clean(r))
            } else {
                Block::Para(clean(t))
            }
        })
        .collect()
}

/// Renders the resume markdown to PDF (A4, embedded DejaVu font).
pub fn to_pdf(content: &str) -> Result<Vec<u8>> {
    use genpdf::{elements, fonts, style, Document, SimplePageDecorator};

    let family = fonts::FontFamily {
        regular: fonts::FontData::new(FONT_REGULAR.to_vec(), None)?,
        bold: fonts::FontData::new(FONT_BOLD.to_vec(), None)?,
        italic: fonts::FontData::new(FONT_REGULAR.to_vec(), None)?,
        bold_italic: fonts::FontData::new(FONT_BOLD.to_vec(), None)?,
    };
    let mut doc = Document::new(family);
    doc.set_title("Resume");
    doc.set_minimal_conformance();
    let mut deco = SimplePageDecorator::new();
    deco.set_margins(18);
    doc.set_page_decorator(deco);
    doc.set_font_size(11);

    for block in parse_md(content) {
        match block {
            Block::Heading(level, text) => {
                let size = match level {
                    1 => 19,
                    2 => 14,
                    _ => 12,
                };
                doc.push(elements::Break::new(0.3));
                let mut p = elements::Paragraph::default();
                p.push_styled(text, style::Style::new().bold().with_font_size(size));
                doc.push(p);
            }
            Block::Bullet(text) => {
                doc.push(elements::Paragraph::new(format!("•  {text}")));
            }
            Block::Para(text) => doc.push(elements::Paragraph::new(text)),
            Block::Blank => doc.push(elements::Break::new(0.5)),
        }
    }

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}

/// Renders the resume markdown to DOCX (editable).
pub fn to_docx(content: &str) -> Result<Vec<u8>> {
    use docx_rs::*;

    let mut docx = Docx::new();
    for block in parse_md(content) {
        let para = match block {
            Block::Heading(level, text) => {
                let sz = match level {
                    1 => 36,
                    2 => 28,
                    _ => 24,
                }; // half-points
                Paragraph::new().add_run(Run::new().add_text(text).bold().size(sz))
            }
            Block::Bullet(text) => {
                Paragraph::new().add_run(Run::new().add_text(format!("•  {text}")))
            }
            Block::Para(text) => Paragraph::new().add_run(Run::new().add_text(text)),
            Block::Blank => Paragraph::new(),
        };
        docx = docx.add_paragraph(para);
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    docx.build().pack(&mut buf)?;
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# John Dev\n\n## Summary\n- 8 years of Rust\n- Reduced cost 30%\n\nCommon text with accents: tion, ao, e.";

    #[test]
    fn generates_valid_pdf() {
        let pdf = to_pdf(SAMPLE).unwrap();
        assert!(pdf.starts_with(b"%PDF"), "must start with PDF header");
        assert!(pdf.len() > 1000);
    }

    #[test]
    fn generates_valid_docx() {
        let docx = to_docx(SAMPLE).unwrap();
        // DOCX is a zip: starts with "PK".
        assert_eq!(&docx[0..2], b"PK");
        assert!(docx.len() > 500);
    }
}
