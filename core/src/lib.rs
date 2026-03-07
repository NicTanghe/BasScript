pub mod buffer;
pub mod links;
pub mod model;
pub mod parser;

pub use buffer::Document;
pub use links::{
    EntityCatalog, EntityDocument, EntityFrontMatter, EntityScaffold, EntitySuggestion,
    LinkDisplayText, LinkError, MentionResolution, ResolutionSource, ResolvedEntity, ScriptLink,
    ScriptLinkSyntax, SuggestedEntityResolution, SuggestionOrigin, UnresolvedEntityResolution,
    UnresolvedReason, extract_script_links, is_valid_target_key, render_script_link_text,
    scaffold_entity, script_link_contains_visible_column, script_link_visible_column_range,
};
pub use model::{Cursor, DocumentFormat, DocumentPath, LineKind, ParsedLine, Position};
pub use parser::{parse_document, parse_document_with_format};
