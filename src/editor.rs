use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EditorBuffer {
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub dirty: bool,
    pub last_search: Option<String>,
}

#[derive(Debug, Default)]
pub struct EditorState {
    pub buffers: Vec<EditorBuffer>,
    pub active: Option<usize>,
}

impl EditorState {
    pub fn open_or_focus(&mut self, path: PathBuf) -> Result<(), String> {
        if let Some(index) = self.buffers.iter().position(|buffer| buffer.path == path) {
            self.active = Some(index);
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .map_err(|err| format!("failed to open {}: {err}", path.display()))?;
        let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
        if content.ends_with('\n') {
            lines.push(String::new());
        }
        if lines.is_empty() {
            lines.push(String::new());
        }

        self.buffers.push(EditorBuffer {
            path,
            lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            dirty: false,
            last_search: None,
        });
        self.active = Some(self.buffers.len() - 1);
        Ok(())
    }

    pub fn current(&self) -> Option<&EditorBuffer> {
        self.active.and_then(|index| self.buffers.get(index))
    }

    pub fn current_mut(&mut self) -> Option<&mut EditorBuffer> {
        self.active.and_then(|index| self.buffers.get_mut(index))
    }

    pub fn save_current(&mut self) -> Result<(), String> {
        let buffer = self
            .current_mut()
            .ok_or_else(|| "no open file to save".to_string())?;

        let content = buffer.lines.join("\n");
        fs::write(&buffer.path, content)
            .map_err(|err| format!("failed to save {}: {err}", buffer.path.display()))?;
        buffer.dirty = false;
        Ok(())
    }

    pub fn active_path(&self) -> Option<&Path> {
        self.current().map(|buffer| buffer.path.as_path())
    }

    pub fn active_dirty(&self) -> bool {
        self.current().map(|buffer| buffer.dirty).unwrap_or(false)
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(buffer) = self.current_mut() {
            let line = &mut buffer.lines[buffer.cursor_row];
            let byte_index = char_to_byte_index(line, buffer.cursor_col);
            line.insert(byte_index, ch);
            buffer.cursor_col += 1;
            buffer.dirty = true;
        }
    }

    pub fn insert_newline(&mut self) {
        if let Some(buffer) = self.current_mut() {
            let byte_index =
                char_to_byte_index(&buffer.lines[buffer.cursor_row], buffer.cursor_col);
            let tail = buffer.lines[buffer.cursor_row].split_off(byte_index);
            buffer.cursor_row += 1;
            buffer.cursor_col = 0;
            buffer.lines.insert(buffer.cursor_row, tail);
            buffer.dirty = true;
        }
    }

    pub fn backspace(&mut self) {
        if let Some(buffer) = self.current_mut() {
            if buffer.cursor_col > 0 {
                let line = &mut buffer.lines[buffer.cursor_row];
                let start = char_to_byte_index(line, buffer.cursor_col - 1);
                let end = char_to_byte_index(line, buffer.cursor_col);
                line.drain(start..end);
                buffer.cursor_col -= 1;
                buffer.dirty = true;
            } else if buffer.cursor_row > 0 {
                let current = buffer.lines.remove(buffer.cursor_row);
                buffer.cursor_row -= 1;
                let prev_len = char_count(&buffer.lines[buffer.cursor_row]);
                buffer.lines[buffer.cursor_row].push_str(&current);
                buffer.cursor_col = prev_len;
                buffer.dirty = true;
            }
        }
    }

    pub fn move_left(&mut self) {
        if let Some(buffer) = self.current_mut() {
            if buffer.cursor_col > 0 {
                buffer.cursor_col -= 1;
            } else if buffer.cursor_row > 0 {
                buffer.cursor_row -= 1;
                buffer.cursor_col = buffer.lines[buffer.cursor_row].len();
            }
        }
    }

    pub fn move_right(&mut self) {
        if let Some(buffer) = self.current_mut() {
            let len = char_count(&buffer.lines[buffer.cursor_row]);
            if buffer.cursor_col < len {
                buffer.cursor_col += 1;
            } else if buffer.cursor_row + 1 < buffer.lines.len() {
                buffer.cursor_row += 1;
                buffer.cursor_col = 0;
            }
        }
    }

    pub fn move_up(&mut self) {
        if let Some(buffer) = self.current_mut() {
            if buffer.cursor_row > 0 {
                buffer.cursor_row -= 1;
                buffer.cursor_col = buffer
                    .cursor_col
                    .min(char_count(&buffer.lines[buffer.cursor_row]));
            }
            buffer.scroll_row = buffer.scroll_row.min(buffer.cursor_row);
        }
    }

    pub fn move_down(&mut self) {
        if let Some(buffer) = self.current_mut() {
            if buffer.cursor_row + 1 < buffer.lines.len() {
                buffer.cursor_row += 1;
                buffer.cursor_col = buffer
                    .cursor_col
                    .min(char_count(&buffer.lines[buffer.cursor_row]));
            }
        }
    }

    pub fn page_up(&mut self, rows: usize) {
        for _ in 0..rows {
            self.move_up();
        }
    }

    pub fn page_down(&mut self, rows: usize) {
        for _ in 0..rows {
            self.move_down();
        }
    }

    pub fn ensure_cursor_visible(&mut self, viewport_height: usize) {
        if let Some(buffer) = self.current_mut() {
            if buffer.cursor_row < buffer.scroll_row {
                buffer.scroll_row = buffer.cursor_row;
            } else if buffer.cursor_row >= buffer.scroll_row + viewport_height {
                buffer.scroll_row = buffer
                    .cursor_row
                    .saturating_sub(viewport_height.saturating_sub(1));
            }
        }
    }

    pub fn search(&mut self, query: &str) -> bool {
        let Some(buffer) = self.current_mut() else {
            return false;
        };
        if query.is_empty() {
            return false;
        }

        let start_row = buffer.cursor_row;
        let start_col = buffer.cursor_col.saturating_add(1);
        let row_count = buffer.lines.len();

        for offset in 0..row_count {
            let row = (start_row + offset) % row_count;
            let haystack = &buffer.lines[row];
            let search_start = if row == start_row {
                char_to_byte_index(haystack, start_col.min(char_count(haystack)))
            } else {
                0
            };
            if let Some(index) = haystack[search_start..].find(query) {
                buffer.cursor_row = row;
                buffer.cursor_col = byte_to_char_index(haystack, search_start + index);
                buffer.last_search = Some(query.to_string());
                return true;
            }
        }

        false
    }

    pub fn next_tab(&mut self) {
        if self.buffers.is_empty() {
            return;
        }
        let current = self.active.unwrap_or(0);
        self.active = Some((current + 1) % self.buffers.len());
    }

    pub fn previous_tab(&mut self) {
        if self.buffers.is_empty() {
            return;
        }
        let current = self.active.unwrap_or(0);
        let next = if current == 0 {
            self.buffers.len() - 1
        } else {
            current - 1
        };
        self.active = Some(next);
    }

    pub fn close_current(&mut self) {
        if let Some(index) = self.active {
            self.buffers.remove(index);
            if self.buffers.is_empty() {
                self.active = None;
            } else if index >= self.buffers.len() {
                self.active = Some(self.buffers.len() - 1);
            }
        }
    }
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }

    value
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or_else(|| value.len())
}

fn byte_to_char_index(value: &str, byte_index: usize) -> usize {
    value[..byte_index.min(value.len())].chars().count()
}

pub fn highlight_line(path: &Path, line: &str) -> Line<'static> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let keywords = keywords_for(ext);
    let comment_marker = comment_marker_for(ext);
    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if let Some(marker) = comment_marker {
            let rest: String = chars[index..].iter().collect();
            if rest.starts_with(marker) {
                spans.push(Span::styled(
                    rest,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ));
                break;
            }
        }

        let ch = chars[index];
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let start = index;
            index += 1;
            while index < chars.len() {
                if chars[index] == '\\' {
                    index += 2;
                    continue;
                }
                if chars[index] == quote {
                    index += 1;
                    break;
                }
                index += 1;
            }
            let value: String = chars[start..index.min(chars.len())].iter().collect();
            spans.push(Span::styled(value, Style::default().fg(Color::Green)));
            continue;
        }

        if ch.is_ascii_digit() {
            let start = index;
            index += 1;
            while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '_') {
                index += 1;
            }
            let value: String = chars[start..index].iter().collect();
            spans.push(Span::styled(value, Style::default().fg(Color::Magenta)));
            continue;
        }

        if is_word_char(ch) {
            let start = index;
            index += 1;
            while index < chars.len() && is_word_char(chars[index]) {
                index += 1;
            }
            let word: String = chars[start..index].iter().collect();
            if keywords.contains(&word.as_str()) {
                spans.push(Span::styled(
                    word,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if word.chars().next().map(char::is_uppercase).unwrap_or(false) {
                spans.push(Span::styled(word, Style::default().fg(Color::Yellow)));
            } else {
                spans.push(Span::raw(word));
            }
            continue;
        }

        spans.push(Span::raw(ch.to_string()));
        index += 1;
    }

    Line::from(spans)
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn comment_marker_for(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" | "kt" | "kts" | "java" | "js" | "ts" | "c" | "cpp" | "h" | "swift" => Some("//"),
        "toml" | "py" | "sh" | "rb" | "yaml" | "yml" => Some("#"),
        _ => None,
    }
}

fn keywords_for(ext: &str) -> &'static [&'static str] {
    match ext {
        "rs" => &[
            "fn", "let", "mut", "struct", "enum", "impl", "pub", "use", "mod", "match", "if",
            "else", "return", "while", "for", "loop", "crate", "Self", "self", "trait",
        ],
        "kt" | "kts" => &[
            "fun",
            "val",
            "var",
            "class",
            "object",
            "interface",
            "when",
            "if",
            "else",
            "return",
            "package",
            "import",
            "private",
            "public",
            "internal",
            "data",
            "sealed",
        ],
        "java" => &[
            "class",
            "public",
            "private",
            "protected",
            "void",
            "static",
            "new",
            "return",
            "if",
            "else",
            "package",
            "import",
            "extends",
            "implements",
        ],
        "toml" => &["true", "false"],
        "gradle" => &["plugins", "dependencies", "android"],
        "xml" => &["android", "layout_width", "layout_height"],
        "py" => &[
            "def", "class", "import", "from", "return", "if", "elif", "else", "for", "while",
            "with", "as", "True", "False", "None",
        ],
        "sh" => &[
            "if", "then", "else", "fi", "for", "do", "done", "case", "esac", "function",
        ],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_wraps() {
        let mut editor = EditorState {
            buffers: vec![EditorBuffer {
                path: PathBuf::from("a.txt"),
                lines: vec!["alpha".into(), "beta".into()],
                cursor_row: 1,
                cursor_col: 1,
                scroll_row: 0,
                dirty: false,
                last_search: None,
            }],
            active: Some(0),
        };

        assert!(editor.search("alp"));
        let buffer = editor.current().unwrap();
        assert_eq!(buffer.cursor_row, 0);
        assert_eq!(buffer.cursor_col, 0);
    }

    #[test]
    fn highlights_keywords_and_strings() {
        let line = highlight_line(Path::new("main.rs"), "let name = \"thor\"; // hi");
        assert!(line.spans.len() >= 4);
    }

    #[test]
    fn edits_unicode_without_panicking() {
        let mut editor = EditorState {
            buffers: vec![EditorBuffer {
                path: PathBuf::from("a.txt"),
                lines: vec!["hé".into()],
                cursor_row: 0,
                cursor_col: 2,
                scroll_row: 0,
                dirty: false,
                last_search: None,
            }],
            active: Some(0),
        };

        editor.backspace();
        editor.insert_char('ö');
        editor.insert_newline();

        let buffer = editor.current().unwrap();
        assert_eq!(buffer.lines[0], "hö");
        assert_eq!(buffer.lines[1], "");
    }

    #[test]
    fn unicode_cursor_movement_uses_character_offsets() {
        let mut editor = EditorState {
            buffers: vec![EditorBuffer {
                path: PathBuf::from("a.txt"),
                lines: vec!["é漢z".into()],
                cursor_row: 0,
                cursor_col: 0,
                scroll_row: 0,
                dirty: false,
                last_search: None,
            }],
            active: Some(0),
        };

        editor.move_right();
        editor.move_right();
        assert_eq!(editor.current().unwrap().cursor_col, 2);

        editor.move_left();
        assert_eq!(editor.current().unwrap().cursor_col, 1);

        editor.insert_char('ß');
        assert_eq!(editor.current().unwrap().lines[0], "éß漢z");
    }
}
