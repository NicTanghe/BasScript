use std::path::{Path, PathBuf};

use crate::links::{ScriptLink, render_script_link_text};

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
    MarkdownHeading,
    MarkdownListItem,
    MarkdownQuote,
    MarkdownCodeFence,
    MarkdownCode,
    MarkdownRule,
    MarkdownParagraph,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentFormat {
    Fountain,
    Markdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedLine {
    pub kind: LineKind,
    pub raw: String,
    pub script_links: Vec<ScriptLink>,
    pub markdown_heading_level: Option<u8>,
}

impl ParsedLine {
    pub fn processed_text(&self) -> String {
        let indent = " ".repeat(self.indent_width());
        let visible_text = render_script_link_text(&self.raw).text;

        match self.kind {
            LineKind::SceneHeading | LineKind::Transition | LineKind::Character => {
                format!("{indent}{}", visible_text.to_uppercase())
            }
            _ => format!("{indent}{visible_text}"),
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
            LineKind::MarkdownHeading => 0,
            LineKind::MarkdownListItem => 0,
            LineKind::MarkdownQuote => 0,
            LineKind::MarkdownCodeFence => 0,
            LineKind::MarkdownCode => 0,
            LineKind::MarkdownRule => 0,
            LineKind::MarkdownParagraph => 0,
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

impl DocumentFormat {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        match extension.as_deref() {
            Some("md") | Some("markdown") => Self::Markdown,
            _ => Self::Fountain,
        }
    }
}
