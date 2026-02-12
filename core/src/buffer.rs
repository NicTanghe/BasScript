use std::fs;
use std::io;
use std::path::Path;

use crate::model::Position;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    lines: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn from_text(text: &str) -> Self {
        let mut lines: Vec<String> = text
            .split('\n')
            .map(|line| line.trim_end_matches('\r').to_owned())
            .collect();

        if lines.is_empty() {
            lines.push(String::new());
        }

        Self { lines }
    }

    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        Ok(Self::from_text(&text))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        fs::write(path, self.to_text())
    }

    pub fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    pub fn line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(String::as_str)
    }

    pub fn line_len_chars(&self, line: usize) -> usize {
        self.line(line).map_or(0, char_count)
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn clamp_position(&self, position: Position) -> Position {
        let last_line = self.line_count().saturating_sub(1);
        let line = position.line.min(last_line);
        let max_col = self.line_len_chars(line);

        Position {
            line,
            column: position.column.min(max_col),
        }
    }

    pub fn move_left(&self, position: Position) -> Position {
        if position.column > 0 {
            return Position {
                line: position.line,
                column: position.column - 1,
            };
        }

        if position.line == 0 {
            return position;
        }

        let previous_line = position.line - 1;
        Position {
            line: previous_line,
            column: self.line_len_chars(previous_line),
        }
    }

    pub fn move_right(&self, position: Position) -> Position {
        let line_len = self.line_len_chars(position.line);
        if position.column < line_len {
            return Position {
                line: position.line,
                column: position.column + 1,
            };
        }

        let next_line = position.line + 1;
        if next_line < self.line_count() {
            Position {
                line: next_line,
                column: 0,
            }
        } else {
            position
        }
    }

    pub fn move_up(&self, position: Position, preferred_column: usize) -> Position {
        if position.line == 0 {
            return position;
        }

        let line = position.line - 1;
        let column = preferred_column.min(self.line_len_chars(line));
        Position { line, column }
    }

    pub fn move_down(&self, position: Position, preferred_column: usize) -> Position {
        let next_line = position.line + 1;
        if next_line >= self.line_count() {
            return position;
        }

        let column = preferred_column.min(self.line_len_chars(next_line));
        Position {
            line: next_line,
            column,
        }
    }

    pub fn insert_text(&mut self, position: Position, input: &str) -> Position {
        let mut position = self.clamp_position(position);

        for ch in input.chars() {
            position = if ch == '\n' {
                self.insert_newline(position)
            } else {
                self.insert_char(position, ch)
            };
        }

        position
    }

    pub fn insert_char(&mut self, position: Position, ch: char) -> Position {
        let position = self.clamp_position(position);
        let line = &mut self.lines[position.line];
        let byte_index = char_to_byte_index(line, position.column);
        line.insert(byte_index, ch);

        Position {
            line: position.line,
            column: position.column + 1,
        }
    }

    pub fn insert_newline(&mut self, position: Position) -> Position {
        let position = self.clamp_position(position);
        let current = &mut self.lines[position.line];
        let byte_index = char_to_byte_index(current, position.column);
        let tail = current.split_off(byte_index);
        self.lines.insert(position.line + 1, tail);

        Position {
            line: position.line + 1,
            column: 0,
        }
    }

    pub fn backspace(&mut self, position: Position) -> Position {
        let position = self.clamp_position(position);

        if position.column > 0 {
            let line = &mut self.lines[position.line];
            let start = char_to_byte_index(line, position.column - 1);
            let end = char_to_byte_index(line, position.column);
            line.replace_range(start..end, "");

            return Position {
                line: position.line,
                column: position.column - 1,
            };
        }

        if position.line == 0 {
            return position;
        }

        let current = self.lines.remove(position.line);
        let previous_line = position.line - 1;
        let previous_len = self.line_len_chars(previous_line);
        self.lines[previous_line].push_str(&current);

        Position {
            line: previous_line,
            column: previous_len,
        }
    }

    pub fn delete(&mut self, position: Position) -> Position {
        let position = self.clamp_position(position);
        let line_len = self.line_len_chars(position.line);

        if position.column < line_len {
            let line = &mut self.lines[position.line];
            let start = char_to_byte_index(line, position.column);
            let end = char_to_byte_index(line, position.column + 1);
            line.replace_range(start..end, "");
            return position;
        }

        if position.line + 1 >= self.line_count() {
            return position;
        }

        let next_line = self.lines.remove(position.line + 1);
        self.lines[position.line].push_str(&next_line);
        position
    }
}

fn char_count(input: &str) -> usize {
    input.chars().count()
}

fn char_to_byte_index(input: &str, column: usize) -> usize {
    if column == 0 {
        return 0;
    }

    input
        .char_indices()
        .map(|(byte, _)| byte)
        .nth(column)
        .unwrap_or(input.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_backspace_roundtrip() {
        let mut doc = Document::new();
        let mut cursor = Position::default();

        cursor = doc.insert_text(cursor, "INT. ROOM");
        cursor = doc.insert_newline(cursor);
        cursor = doc.insert_text(cursor, "Some action");

        assert_eq!(doc.line_count(), 2);
        assert_eq!(doc.line(0), Some("INT. ROOM"));
        assert_eq!(doc.line(1), Some("Some action"));

        cursor = doc.backspace(cursor);
        cursor = doc.backspace(cursor);

        assert_eq!(cursor, Position { line: 1, column: 9 });
        assert_eq!(doc.line(1), Some("Some acti"));
    }

    #[test]
    fn delete_joins_lines() {
        let mut doc = Document::from_text("A\nB");
        let cursor = doc.delete(Position { line: 0, column: 1 });

        assert_eq!(cursor, Position { line: 0, column: 1 });
        assert_eq!(doc.line_count(), 1);
        assert_eq!(doc.line(0), Some("AB"));
    }
}
