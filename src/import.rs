//! Extraction of text from CVs (PDF/DOCX/TXT) to feed the agent.
//!
//! All pure Rust: PDF via `pdf-extract`, DOCX via `zip` + `quick-xml` (collects
//! text nodes `<w:t>` from `word/document.xml`).

use anyhow::{bail, Context, Result};
use std::io::Read;
use std::path::Path;

/// Extracts plain text from the file, dispatching by extension.
pub fn extract_text(path: &Path) -> Result<String> {
    if !path.is_file() {
        bail!("file not found: {}", path.display());
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let text = match ext.as_str() {
        "pdf" => extract_pdf(path)?,
        "docx" => extract_docx(path)?,
        "txt" | "md" | "markdown" => std::fs::read_to_string(path).context("read text file")?,
        other => bail!("unsupported format: .{other} (use pdf, docx, txt or md)"),
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        bail!("could not extract text from {}", path.display());
    }
    Ok(trimmed.to_string())
}

fn extract_pdf(path: &Path) -> Result<String> {
    pdf_extract::extract_text(path).context("extract text from PDF")
}

/// Extracts the flowing text from a .docx (zip → word/document.xml → `<w:t>` nodes).
fn extract_docx(path: &Path) -> Result<String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let file = std::fs::File::open(path).context("open docx")?;
    let mut zip = zip::ZipArchive::new(file).context("docx is not a valid zip")?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .context("docx without word/document.xml")?
        .read_to_string(&mut xml)
        .context("read document.xml")?;

    let mut reader = Reader::from_str(&xml);
    let mut out = String::new();
    let mut in_text = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.local_name().as_ref() == b"t" {
                    in_text = true;
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"t" => in_text = false,
                // end of paragraph / break → new line
                b"p" => out.push('\n'),
                b"br" | b"tab" => out.push(' '),
                _ => {}
            },
            Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                b"br" => out.push('\n'),
                b"tab" => out.push('\t'),
                _ => {}
            },
            Ok(Event::Text(t)) if in_text => {
                out.push_str(&t.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(e) => bail!("error parsing docx: {e}"),
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("jobrabbit_import_test_{name}"))
    }

    #[test]
    fn extrai_txt() {
        let p = tmp("cv.txt");
        std::fs::write(&p, "John Dev\nRust, Go\n").unwrap();
        let t = extract_text(&p).unwrap();
        assert!(t.contains("John Dev"));
        assert!(t.contains("Rust"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn unsupported_format_errors() {
        let p = tmp("cv.rtf");
        std::fs::write(&p, "x").unwrap();
        assert!(extract_text(&p).is_err());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn missing_file_errors() {
        assert!(extract_text(Path::new("/nao/existe/cv.pdf")).is_err());
    }

    #[test]
    fn extrai_docx_construido() {
        // Builds a minimal .docx (zip with word/document.xml) and extracts the text.
        let p = tmp("cv.docx");
        let f = std::fs::File::create(&p).unwrap();
        let mut zw = zip::ZipWriter::new(f);
        let opts: zip::write::FileOptions<()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zw.start_file("word/document.xml", opts).unwrap();
        let xml = r#"<?xml version="1.0"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p><w:r><w:t>Maria Engineer</w:t></w:r></w:p>
                <w:p><w:r><w:t>Rust and Kubernetes</w:t></w:r></w:p>
              </w:body>
            </w:document>"#;
        zw.write_all(xml.as_bytes()).unwrap();
        zw.finish().unwrap();

        let t = extract_text(&p).unwrap();
        assert!(t.contains("Maria Engineer"), "text: {t}");
        assert!(t.contains("Rust and Kubernetes"), "text: {t}");
        let _ = std::fs::remove_file(&p);
    }
}
