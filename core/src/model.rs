use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub position: Position,
    pub preferred_column: usize,
}

impl Cursor {
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
        self.preferred_column = position.column;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineKind {
    Empty,
    SceneHeading,
    Action,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedLine {
    pub kind: LineKind,
    pub raw: String,
}

impl ParsedLine {
    pub fn processed_text(&self) -> String {
        let indent = " ".repeat(self.indent_width());

        match self.kind {
            LineKind::SceneHeading | LineKind::Transition | LineKind::Character => {
                format!("{indent}{}", self.raw.to_uppercase())
            }
            _ => format!("{indent}{}", self.raw),
        }
    }

    pub fn processed_column(&self, raw_column: usize) -> usize {
        self.indent_width().saturating_add(raw_column)
    }

    pub fn indent_width(&self) -> usize {
        match self.kind {
            LineKind::SceneHeading => 2,
            LineKind::Action => 0,
            LineKind::Character => 24,
            LineKind::Dialogue => 12,
            LineKind::Parenthetical => 18,
            LineKind::Transition => 40,
            LineKind::Empty => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentPath {
    pub load_path: PathBuf,
    pub save_path: PathBuf,
}

impl DocumentPath {
    pub fn new(load_path: impl AsRef<Path>, save_path: impl AsRef<Path>) -> Self {
        Self {
            load_path: load_path.as_ref().to_path_buf(),
            save_path: save_path.as_ref().to_path_buf(),
        }
    }
}
