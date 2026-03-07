use std::{error::Error, fmt, io, ops::Range, path::PathBuf};

mod entities;
mod syntax;

pub use syntax::{
    extract_script_links, is_valid_target_key, render_script_link_text,
    script_link_contains_visible_column, script_link_visible_column_range,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptLinkSyntax {
    TargetOnly,
    LabelledTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptLink {
    pub span: Range<usize>,
    pub label: String,
    pub target: String,
    pub syntax: ScriptLinkSyntax,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkDisplayText {
    pub text: String,
    pub display_to_raw: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityFrontMatter {
    pub id: String,
    pub target: String,
    pub entity_type: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityDocument {
    pub metadata: EntityFrontMatter,
    pub path: PathBuf,
    pub body: String,
}

#[derive(Clone, Debug, Default)]
pub struct EntityCatalog {
    entities: std::collections::BTreeMap<String, EntityDocument>,
    alias_index: std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolutionSource {
    ExplicitLink,
    Alias,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEntity {
    pub source: ResolutionSource,
    pub mention: String,
    pub target: String,
    pub path: PathBuf,
    pub entity: EntityFrontMatter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionOrigin {
    EntityMatch,
    AmbiguousAlias,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntitySuggestion {
    pub target: String,
    pub name: String,
    pub score: u16,
    pub origin: SuggestionOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestedEntityResolution {
    pub mention: String,
    pub suggestions: Vec<EntitySuggestion>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnresolvedReason {
    MissingTargetFile,
    InvalidTarget,
    UnknownMention,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityScaffold {
    pub target: String,
    pub path: PathBuf,
    pub markdown: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedEntityResolution {
    pub mention: String,
    pub explicit_target: Option<String>,
    pub reason: UnresolvedReason,
    pub scaffold: Option<EntityScaffold>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionResolution {
    Resolved(ResolvedEntity),
    Suggested(SuggestedEntityResolution),
    Unresolved(UnresolvedEntityResolution),
}

#[derive(Debug)]
pub enum LinkError {
    Io(io::Error),
    MissingFrontMatter {
        path: PathBuf,
    },
    UnterminatedFrontMatter {
        path: PathBuf,
    },
    MissingField {
        path: PathBuf,
        field: &'static str,
    },
    MalformedFrontMatterLine {
        path: PathBuf,
        line: usize,
        content: String,
    },
    InvalidTargetKey {
        path: Option<PathBuf>,
        target: String,
    },
    FilenameTargetMismatch {
        path: PathBuf,
        file_stem: String,
        target: String,
    },
    NonUtf8FileStem {
        path: PathBuf,
    },
    DuplicateTarget {
        target: String,
        first: PathBuf,
        second: PathBuf,
    },
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::MissingFrontMatter { path } => {
                write!(f, "{} is missing YAML front matter", path.display())
            }
            Self::UnterminatedFrontMatter { path } => {
                write!(f, "{} has unterminated YAML front matter", path.display())
            }
            Self::MissingField { path, field } => {
                write!(f, "{} is missing required field `{field}`", path.display())
            }
            Self::MalformedFrontMatterLine {
                path,
                line,
                content,
            } => write!(
                f,
                "{} has malformed YAML front matter on line {}: {}",
                path.display(),
                line,
                content
            ),
            Self::InvalidTargetKey { path, target } => match path {
                Some(path) => write!(
                    f,
                    "{} declares invalid target key `{target}`",
                    path.display()
                ),
                None => write!(f, "invalid target key `{target}`"),
            },
            Self::FilenameTargetMismatch {
                path,
                file_stem,
                target,
            } => write!(
                f,
                "{} uses file stem `{file_stem}` but front matter target is `{target}`",
                path.display()
            ),
            Self::NonUtf8FileStem { path } => {
                write!(f, "{} does not have a UTF-8 file stem", path.display())
            }
            Self::DuplicateTarget {
                target,
                first,
                second,
            } => write!(
                f,
                "duplicate target `{target}` found in {} and {}",
                first.display(),
                second.display()
            ),
        }
    }
}

impl Error for LinkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for LinkError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub use entities::scaffold_entity;

#[cfg(test)]
mod tests;
