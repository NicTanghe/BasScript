pub mod buffer;
pub mod canvas;
pub mod links;
pub mod model;
pub mod parser;
pub mod story_index;

pub use buffer::Document;
pub use canvas::{
    CanvasDocument, CanvasEdge, CanvasNode, CanvasNodeKind, CanvasParseError,
    parse_canvas_document, update_canvas_node_position, update_canvas_text_node_content,
};
pub use links::{
    EntityCatalog, EntityDocument, EntityFrontMatter, EntityScaffold, EntitySuggestion,
    LinkDisplayText, LinkError, MentionResolution, ResolutionSource, ResolvedEntity, ScriptLink,
    ScriptLinkSyntax, SuggestedEntityResolution, SuggestionOrigin, UnresolvedEntityResolution,
    UnresolvedReason, extract_script_links, is_valid_target_key, render_script_link_text,
    scaffold_entity, script_link_contains_visible_column, script_link_visible_column_range,
};
pub use model::{Cursor, DocumentFormat, DocumentPath, ImageEmbed, LineKind, ParsedLine, Position};
pub use parser::{parse_document, parse_document_with_format};
pub use story_index::{
    IndexedFileKind, STORY_INDEX_DATABASE_NAME, STORY_INDEX_DIR_NAME, STORY_INDEX_SCHEMA_VERSION,
    StoryIndexDatabase, StoryIndexError, StoryIndexOpenReport, StoryIndexOpenStatus,
    StoryIndexRecoveryReason, StoryIndexScanReport, story_index_database_path,
};
