//! Minimal text editor (with cursor) for TUI editable fields.
//!
//! Supports insertion/removal at cursor, navigation (←/→/Home/End) and multiple
//! lines (via `\n`). No external dependencies; indices are in **chars**.

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextEditor {
    content: String,
    cursor: usize, // index in chars
}

impl TextEditor {
    pub fn new(initial: &str) -> Self {
        Self {
            content: initial.to_string(),
            cursor: initial.chars().count(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn len_chars(&self) -> usize {
        self.content.chars().count()
    }

    /// Byte offset of n-th char (for mutations in `String`).
    fn byte_of(&self, char_idx: usize) -> usize {
        self.content
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.content.len())
    }

    pub fn insert(&mut self, c: char) {
        let byte = self.byte_of(self.cursor);
        self.content.insert(byte, c);
        self.cursor += 1;
    }

    pub fn newline(&mut self) {
        self.insert('\n');
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let start = self.byte_of(self.cursor - 1);
            let end = self.byte_of(self.cursor);
            self.content.replace_range(start..end, "");
            self.cursor -= 1;
        }
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        if self.cursor < self.len_chars() {
            self.cursor += 1;
        }
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.len_chars();
    }

    /// Content with a cursor marker (`▏`) inserted at position — for
    /// display (also visible in test snapshots).
    pub fn render_with_cursor(&self) -> String {
        let byte = self.byte_of(self.cursor);
        let mut s = self.content.clone();
        s.insert(byte, '▏');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_at_cursor() {
        let mut e = TextEditor::new("ac");
        e.left(); // cursor between a and c
        e.insert('b');
        assert_eq!(e.content(), "abc");
    }

    #[test]
    fn backspace_removes_before_cursor() {
        let mut e = TextEditor::new("abc");
        e.backspace();
        assert_eq!(e.content(), "ab");
        e.home();
        e.backspace(); // nothing before
        assert_eq!(e.content(), "ab");
    }

    #[test]
    fn navigation_clamps_at_bounds() {
        let mut e = TextEditor::new("ab");
        e.right();
        e.right(); // already at the end, does not pass
        assert_eq!(e.cursor(), 2);
        e.home();
        e.left();
        assert_eq!(e.cursor(), 0);
    }

    #[test]
    fn newline_and_unicode() {
        let mut e = TextEditor::new("ção");
        e.home();
        e.insert('A');
        assert_eq!(e.content(), "Ação");
        e.end();
        e.newline();
        e.insert('x');
        assert_eq!(e.content(), "Ação\nx");
    }

    #[test]
    fn cursor_marker() {
        let mut e = TextEditor::new("ab");
        e.home();
        assert_eq!(e.render_with_cursor(), "▏ab");
        e.end();
        assert_eq!(e.render_with_cursor(), "ab▏");
    }
}
