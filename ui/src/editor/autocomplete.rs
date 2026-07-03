const LINK_AUTOCOMPLETE_INLINE_PREFIX_MIN_CHARS: usize = 2;
const LINK_AUTOCOMPLETE_QUERY_LIMIT: usize = 16;
const LINK_AUTOCOMPLETE_VISIBLE_ROWS: usize = 6;
const LINK_AUTOCOMPLETE_MENU_WIDTH: f32 = 340.0;
const LINK_AUTOCOMPLETE_ROW_HEIGHT: f32 = 30.0;
const LINK_AUTOCOMPLETE_MENU_PADDING: f32 = 5.0;
const LINK_AUTOCOMPLETE_MENU_GAP: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinkAutocompleteSource {
    Document,
    CanvasText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinkAutocompleteTrigger {
    Bracket,
    InlinePrefix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinkAutocompleteRange {
    start: Position,
    end: Position,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LinkAutocomplete {
    source: LinkAutocompleteSource,
    trigger: LinkAutocompleteTrigger,
    range: LinkAutocompleteRange,
    prefix: String,
    suggestions: Vec<LinkAutocompleteSuggestion>,
    selected_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinkAutocompleteSuggestionKind {
    Entity,
    Scene,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinkAutocompleteLineContext {
    CharacterCue,
    SceneHeading,
    General,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LinkAutocompleteSuggestion {
    kind: LinkAutocompleteSuggestionKind,
    display_name: String,
    entity_type: String,
    target: String,
    detail: String,
    score: i32,
}

#[derive(Component)]
struct LinkAutocompleteRoot;

#[derive(Component)]
struct LinkAutocompleteRow {
    index: usize,
}

#[derive(Component)]
struct LinkAutocompleteNameText {
    index: usize,
}

#[derive(Component)]
struct LinkAutocompleteTypeText {
    index: usize,
}

#[derive(Component)]
struct LinkAutocompleteTargetText {
    index: usize,
}

fn sync_link_autocomplete_context(
    mut state: ResMut<EditorState>,
    dialogs: Res<DialogState>,
) {
    if dialogs.pending.is_some() {
        state.close_link_autocomplete();
        return;
    }

    state.validate_link_autocomplete_context();
}

fn spawn_link_autocomplete_menu(parent: &mut ChildSpawnerCommands<'_>, font: Handle<Font>) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: px(LINK_AUTOCOMPLETE_MENU_WIDTH),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(LINK_AUTOCOMPLETE_MENU_PADDING)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.97, 0.98, 0.99, 0.97)),
            BorderColor::all(Color::srgba(0.10, 0.12, 0.14, 0.18)),
            ZIndex(94),
            LinkAutocompleteRoot,
        ))
        .with_children(|menu| {
            for index in 0..LINK_AUTOCOMPLETE_VISIBLE_ROWS {
                spawn_link_autocomplete_row(menu, font.clone(), index);
            }
        });
}

fn spawn_link_autocomplete_row(
    parent: &mut ChildSpawnerCommands<'_>,
    font: Handle<Font>,
    index: usize,
) {
    parent
        .spawn((
            Node {
                width: percent(100.0),
                height: px(LINK_AUTOCOMPLETE_ROW_HEIGHT),
                display: Display::None,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8.0),
                padding: UiRect::axes(px(8.0), px(0.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::NONE),
            LinkAutocompleteRow { index },
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_width: px(0.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
                LinkAutocompleteNameText { index },
            ));
            row.spawn((
                Node {
                    width: px(74.0),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 10.5,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
                LinkAutocompleteTypeText { index },
            ));
            row.spawn((
                Node {
                    width: px(112.0),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font,
                    font_size: 10.5,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
                LinkAutocompleteTargetText { index },
            ));
        });
}

fn sync_link_autocomplete_ui(
    state: Res<EditorState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caret_query: Query<
        (&PanelCaret, &Node, &Visibility),
        (Without<LinkAutocompleteRoot>, Without<LinkAutocompleteRow>),
    >,
    mut root_query: Query<
        &mut Node,
        (With<LinkAutocompleteRoot>, Without<LinkAutocompleteRow>),
    >,
    mut row_query: Query<(
        &LinkAutocompleteRow,
        &mut Node,
        &mut BackgroundColor,
    ), (With<LinkAutocompleteRow>, Without<LinkAutocompleteRoot>)>,
    mut text_queries: ParamSet<(
        Query<(&LinkAutocompleteNameText, &mut Text)>,
        Query<(&LinkAutocompleteTypeText, &mut Text, &mut TextColor)>,
        Query<(&LinkAutocompleteTargetText, &mut Text)>,
    )>,
) {
    let Ok(mut root) = root_query.single_mut() else {
        return;
    };
    let Some(active) = state
        .link_autocomplete
        .as_ref()
        .filter(|active| !active.suggestions.is_empty())
    else {
        root.display = Display::None;
        return;
    };

    let visible_count = active
        .suggestions
        .len()
        .min(LINK_AUTOCOMPLETE_VISIBLE_ROWS);
    if visible_count == 0 {
        root.display = Display::None;
        return;
    }

    let (left, top) = link_autocomplete_menu_position(&state, &window_query, &caret_query, visible_count);
    root.display = Display::Flex;
    root.left = px(left);
    root.top = px(top);
    root.height = px(
        (visible_count as f32 * LINK_AUTOCOMPLETE_ROW_HEIGHT)
            + (LINK_AUTOCOMPLETE_MENU_PADDING * 2.0),
    );

    for (row, mut node, mut background) in row_query.iter_mut() {
        let Some(suggestion) = active.suggestions.get(row.index) else {
            node.display = Display::None;
            continue;
        };
        if row.index >= visible_count {
            node.display = Display::None;
            continue;
        }

        node.display = Display::Flex;
        background.0 = if row.index == active.selected_index {
            Color::srgba(0.12, 0.34, 0.62, 0.18)
        } else {
            Color::NONE
        };

        sync_link_autocomplete_row_text(&mut text_queries, row.index, suggestion, &state);
    }
}

fn sync_link_autocomplete_row_text(
    text_queries: &mut ParamSet<(
        Query<(&LinkAutocompleteNameText, &mut Text)>,
        Query<(&LinkAutocompleteTypeText, &mut Text, &mut TextColor)>,
        Query<(&LinkAutocompleteTargetText, &mut Text)>,
    )>,
    index: usize,
    suggestion: &LinkAutocompleteSuggestion,
    state: &EditorState,
) {
    for (name, mut text) in text_queries.p0().iter_mut() {
        if name.index == index {
            **text = suggestion.display_name.clone();
            break;
        }
    }
    for (type_text, mut text, mut color) in text_queries.p1().iter_mut() {
        if type_text.index == index {
            **text = suggestion.entity_type.clone();
            color.0 = color_from_rgba(state.link_rgba_for_type(&suggestion.entity_type));
            break;
        }
    }
    for (target, mut text) in text_queries.p2().iter_mut() {
        if target.index == index {
            **text = suggestion.row_detail().to_string();
            break;
        }
    }
}

fn link_autocomplete_menu_position(
    state: &EditorState,
    window_query: &Query<&Window, With<PrimaryWindow>>,
    caret_query: &Query<
        (&PanelCaret, &Node, &Visibility),
        (Without<LinkAutocompleteRoot>, Without<LinkAutocompleteRow>),
    >,
    visible_count: usize,
) -> (f32, f32) {
    let mut left_top = link_autocomplete_caret_left_top(state, caret_query)
        .map(|(left, top)| (left + LINK_AUTOCOMPLETE_MENU_GAP, top + state.measured_line_step + LINK_AUTOCOMPLETE_MENU_GAP))
        .unwrap_or((320.0, 96.0));

    if let Ok(window) = window_query.single() {
        let menu_height = (visible_count as f32 * LINK_AUTOCOMPLETE_ROW_HEIGHT)
            + (LINK_AUTOCOMPLETE_MENU_PADDING * 2.0);
        let max_left = (window.width() - LINK_AUTOCOMPLETE_MENU_WIDTH - 8.0).max(8.0);
        left_top.0 = left_top.0.clamp(8.0, max_left);

        if left_top.1 + menu_height > window.height() - 28.0 {
            left_top.1 = (left_top.1 - menu_height - state.measured_line_step).max(8.0);
        }
    }

    left_top
}

fn link_autocomplete_caret_left_top(
    state: &EditorState,
    caret_query: &Query<
        (&PanelCaret, &Node, &Visibility),
        (Without<LinkAutocompleteRoot>, Without<LinkAutocompleteRow>),
    >,
) -> Option<(f32, f32)> {
    if state
        .link_autocomplete
        .as_ref()
        .is_some_and(|active| active.source == LinkAutocompleteSource::CanvasText)
    {
        return None;
    }

    let panel = state.active_panel_for_display_mode();
    caret_query
        .iter()
        .find(|(caret, _, _)| caret.kind == panel)
        .and_then(|(_, node, _)| Some((val_to_px(node.left)?, val_to_px(node.top)?)))
}

fn val_to_px(value: Val) -> Option<f32> {
    match value {
        Val::Px(value) => Some(value),
        _ => None,
    }
}

impl EditorState {
    fn close_link_autocomplete(&mut self) {
        self.link_autocomplete = None;
    }

    fn refresh_link_autocomplete_for_document_cursor(&mut self) {
        if !self.link_autocomplete_document_context_allowed() {
            self.close_link_autocomplete();
            return;
        }

        self.link_autocomplete = link_autocomplete_at_cursor(
            &self.document,
            self.cursor.position,
            LinkAutocompleteSource::Document,
        )
        .and_then(|active| self.with_link_autocomplete_suggestions(active));
    }

    fn refresh_link_autocomplete_for_canvas_text_cursor(&mut self, document: &Document) {
        if !self.link_autocomplete_canvas_context_allowed() {
            self.close_link_autocomplete();
            return;
        }

        self.link_autocomplete = link_autocomplete_at_cursor(
            document,
            self.canvas_text_cursor.position,
            LinkAutocompleteSource::CanvasText,
        )
        .and_then(|active| self.with_link_autocomplete_suggestions(active));
    }

    fn validate_link_autocomplete_context(&mut self) {
        let Some(active) = self.link_autocomplete.clone() else {
            return;
        };

        let current = match active.source {
            LinkAutocompleteSource::Document => {
                if !self.link_autocomplete_document_context_allowed() {
                    None
                } else {
                    link_autocomplete_at_cursor(
                        &self.document,
                        self.cursor.position,
                        LinkAutocompleteSource::Document,
                    )
                }
            }
            LinkAutocompleteSource::CanvasText => {
                if !self.link_autocomplete_canvas_context_allowed() {
                    None
                } else {
                    self.active_canvas_text_document().and_then(|document| {
                        link_autocomplete_at_cursor(
                            &document,
                            self.canvas_text_cursor.position,
                            LinkAutocompleteSource::CanvasText,
                        )
                    })
                }
            }
        };

        if let Some(current) = current.filter(|current| current.continues_trigger_from(&active)) {
            if current.range == active.range && current.prefix == active.prefix {
                self.link_autocomplete = Some(active);
            } else {
                self.link_autocomplete = self.with_link_autocomplete_suggestions(current);
            }
        } else {
            self.close_link_autocomplete();
        }
    }

    fn with_link_autocomplete_suggestions(
        &self,
        mut active: LinkAutocomplete,
    ) -> Option<LinkAutocomplete> {
        active.suggestions = self.link_autocomplete_suggestions(&active);
        active.selected_index = 0;
        if active.suggestions.is_empty() {
            None
        } else {
            Some(active)
        }
    }

    fn link_autocomplete_document_context_allowed(&self) -> bool {
        !self.workspace_focused
            && self.workspace_prompt.is_none()
            && self.command_menu.is_none()
            && !self.story_query_sheet.open
            && self.document_format != DocumentFormat::Canvas
            && (!self.vim_enabled || self.vim_mode == VimMode::Insert)
    }

    fn link_autocomplete_canvas_context_allowed(&self) -> bool {
        !self.workspace_focused
            && self.workspace_prompt.is_none()
            && self.command_menu.is_none()
            && !self.story_query_sheet.open
            && self.document_format == DocumentFormat::Canvas
            && self.canvas_editing_node_id.is_some()
            && (!self.vim_enabled || self.vim_mode == VimMode::Insert)
    }

    fn link_autocomplete_suggestions(
        &self,
        active: &LinkAutocomplete,
    ) -> Vec<LinkAutocompleteSuggestion> {
        let Some(database) = self.link_autocomplete_story_database() else {
            return Vec::new();
        };

        let context = self.link_autocomplete_line_context(active);
        let mut suggestions = self.link_autocomplete_entity_suggestions(database, active, context);
        suggestions.extend(self.link_autocomplete_scene_suggestions(database, active, context));
        suggestions.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.display_name.cmp(&right.display_name))
                .then_with(|| left.target.cmp(&right.target))
        });
        suggestions.truncate(LINK_AUTOCOMPLETE_QUERY_LIMIT);
        suggestions
    }

    fn link_autocomplete_story_database(&self) -> Option<&StoryIndexDatabase> {
        self.story_index
            .as_ref()
            .filter(|index| {
                matches!(
                    index.status,
                    EditorStoryIndexStatus::Ready
                        | EditorStoryIndexStatus::Created
                        | EditorStoryIndexStatus::Recreated
                )
            })
            .and_then(|index| index.database.as_ref())
    }

    fn link_autocomplete_line_context(
        &self,
        active: &LinkAutocomplete,
    ) -> LinkAutocompleteLineContext {
        if active.source != LinkAutocompleteSource::Document {
            return LinkAutocompleteLineContext::General;
        }

        match self
            .parsed
            .get(active.range.start.line)
            .map(|line| &line.kind)
        {
            Some(LineKind::Character) => LinkAutocompleteLineContext::CharacterCue,
            Some(LineKind::SceneHeading) => LinkAutocompleteLineContext::SceneHeading,
            _ => LinkAutocompleteLineContext::General,
        }
    }

    fn link_autocomplete_entity_suggestions(
        &self,
        database: &StoryIndexDatabase,
        active: &LinkAutocomplete,
        context: LinkAutocompleteLineContext,
    ) -> Vec<LinkAutocompleteSuggestion> {
        let query = normalized_link_autocomplete_query(&active.prefix);
        let mut entities = BTreeMap::<String, basscript_core::StoryIndexEntityRecord>::new();

        let indexed_entities = if query.is_empty() {
            database.all_entities()
        } else {
            database.entities_matching_text(&active.prefix, LINK_AUTOCOMPLETE_QUERY_LIMIT)
        };
        if let Ok(indexed_entities) = indexed_entities {
            for entity in indexed_entities {
                entities.insert(entity.target.clone(), entity);
            }
        }

        if !query.is_empty()
            && let Ok(all_entities) = database.all_entities()
        {
            for entity in all_entities {
                if entities.contains_key(&entity.target) {
                    continue;
                }
                if fuzzy_subsequence_match(&normalized_link_autocomplete_query(&entity.name), &query)
                    || fuzzy_subsequence_match(
                        &normalized_link_autocomplete_query(&entity.target),
                        &query,
                    )
                {
                    entities.insert(entity.target.clone(), entity);
                }
            }
        }

        let current_document = self.document.to_text();
        entities
            .into_values()
            .filter_map(|entity| {
                let score = score_entity_suggestion(&entity, &query, context, &current_document)?;
                Some(LinkAutocompleteSuggestion {
                    kind: LinkAutocompleteSuggestionKind::Entity,
                    display_name: entity.name,
                    entity_type: entity.entity_type,
                    detail: entity.relative_path,
                    target: entity.target,
                    score,
                })
            })
            .collect()
    }

    fn link_autocomplete_scene_suggestions(
        &self,
        database: &StoryIndexDatabase,
        active: &LinkAutocomplete,
        context: LinkAutocompleteLineContext,
    ) -> Vec<LinkAutocompleteSuggestion> {
        if context != LinkAutocompleteLineContext::SceneHeading {
            return Vec::new();
        }

        let query = normalized_link_autocomplete_query(&active.prefix);
        let Ok(scenes) = database.all_scenes() else {
            return Vec::new();
        };

        scenes
            .into_iter()
            .filter_map(|scene| {
                let score = score_scene_suggestion(&scene, &query)?;
                Some(LinkAutocompleteSuggestion {
                    kind: LinkAutocompleteSuggestionKind::Scene,
                    display_name: scene.heading_text,
                    entity_type: "scene".to_string(),
                    detail: scene.relative_path,
                    target: scene.scene_key,
                    score,
                })
            })
            .collect()
    }
}

impl LinkAutocomplete {
    fn continues_trigger_from(&self, previous: &Self) -> bool {
        self.source == previous.source
            && self.trigger == previous.trigger
            && self.range.start == previous.range.start
    }
}

impl LinkAutocompleteSuggestion {
    fn row_detail(&self) -> &str {
        match self.kind {
            LinkAutocompleteSuggestionKind::Entity => &self.target,
            LinkAutocompleteSuggestionKind::Scene if !self.detail.is_empty() => &self.detail,
            LinkAutocompleteSuggestionKind::Scene => &self.target,
        }
    }
}

fn link_autocomplete_at_cursor(
    document: &Document,
    cursor: Position,
    source: LinkAutocompleteSource,
) -> Option<LinkAutocomplete> {
    let line = document.line(cursor.line)?;
    let chars = line.chars().collect::<Vec<_>>();
    let column = cursor.column.min(chars.len());

    if let Some((start_column, prefix)) = bracket_link_autocomplete_trigger(&chars, column) {
        return Some(LinkAutocomplete {
            source,
            trigger: LinkAutocompleteTrigger::Bracket,
            range: LinkAutocompleteRange {
                start: Position {
                    line: cursor.line,
                    column: start_column,
                },
                end: Position {
                    line: cursor.line,
                    column,
                },
            },
            prefix,
            suggestions: Vec::new(),
            selected_index: 0,
        });
    }

    let (start_column, prefix) = inline_link_autocomplete_trigger(&chars, column)?;
    Some(LinkAutocomplete {
        source,
        trigger: LinkAutocompleteTrigger::InlinePrefix,
        range: LinkAutocompleteRange {
            start: Position {
                line: cursor.line,
                column: start_column,
            },
            end: Position {
                line: cursor.line,
                column,
            },
        },
        prefix,
        suggestions: Vec::new(),
        selected_index: 0,
    })
}

fn bracket_link_autocomplete_trigger(
    chars: &[char],
    cursor_column: usize,
) -> Option<(usize, String)> {
    if cursor_column == 0 {
        return None;
    }

    let mut index = cursor_column.min(chars.len());
    while index > 0 {
        index -= 1;
        match chars[index] {
            '[' => {
                let prefix = chars[index + 1..cursor_column].iter().collect::<String>();
                if prefix.chars().all(is_link_autocomplete_bracket_char) {
                    return Some((index, prefix));
                }
                return None;
            }
            ']' | '(' | ')' => return None,
            _ => {}
        }
    }

    None
}

fn inline_link_autocomplete_trigger(
    chars: &[char],
    cursor_column: usize,
) -> Option<(usize, String)> {
    if cursor_column == 0 || cursor_column > chars.len() {
        return None;
    }
    if !is_link_autocomplete_word_char(chars[cursor_column - 1]) {
        return None;
    }

    let mut start = cursor_column;
    while start > 0 && is_link_autocomplete_word_char(chars[start - 1]) {
        start -= 1;
    }

    let prefix_len = cursor_column.saturating_sub(start);
    if prefix_len < LINK_AUTOCOMPLETE_INLINE_PREFIX_MIN_CHARS {
        return None;
    }

    Some((start, chars[start..cursor_column].iter().collect()))
}

fn is_link_autocomplete_bracket_char(ch: char) -> bool {
    !matches!(ch, '[' | ']' | '(' | ')' | '\n' | '\r')
}

fn is_link_autocomplete_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '-' | '_' | '\'')
}

fn score_entity_suggestion(
    entity: &basscript_core::StoryIndexEntityRecord,
    query: &str,
    context: LinkAutocompleteLineContext,
    current_document: &str,
) -> Option<i32> {
    let name = normalized_link_autocomplete_query(&entity.name);
    let target = normalized_link_autocomplete_query(&entity.target);
    let entity_type = normalized_link_autocomplete_query(&entity.entity_type);
    let mut score = if query.is_empty() {
        20
    } else {
        best_text_match_score(query, &[(&name, 90), (&target, 80)]).or(Some(50))?
    };

    score += match context {
        LinkAutocompleteLineContext::CharacterCue if entity_type == "character" => 24,
        LinkAutocompleteLineContext::CharacterCue => -8,
        LinkAutocompleteLineContext::SceneHeading
            if matches!(entity_type.as_str(), "place" | "location" | "scene" | "setting") =>
        {
            24
        }
        LinkAutocompleteLineContext::SceneHeading => 0,
        LinkAutocompleteLineContext::General => 0,
    };

    if current_document_mentions_target(current_document, &entity.target) {
        score += 6;
    }

    Some(score)
}

fn score_scene_suggestion(
    scene: &basscript_core::StoryIndexSceneRecord,
    query: &str,
) -> Option<i32> {
    let heading = normalized_link_autocomplete_query(&scene.heading_text);
    let location = scene
        .location_text
        .as_deref()
        .map(normalized_link_autocomplete_query)
        .unwrap_or_default();
    let key = normalized_link_autocomplete_query(&scene.scene_key);

    let score = if query.is_empty() {
        30
    } else {
        best_text_match_score(query, &[(&heading, 86), (&location, 82), (&key, 60)])?
    };
    Some(score + 18)
}

fn best_text_match_score(query: &str, fields: &[(&str, i32)]) -> Option<i32> {
    let mut best = None::<i32>;
    for (field, base) in fields {
        let score = if field == &query {
            base.saturating_add(20)
        } else if field.starts_with(query) {
            *base
        } else if field.contains(query) {
            base.saturating_sub(28)
        } else if fuzzy_subsequence_match(field, query) {
            base.saturating_sub(48)
        } else {
            continue;
        };
        best = Some(best.map_or(score, |current| current.max(score)));
    }
    best
}

fn fuzzy_subsequence_match(field: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let mut query_chars = query.chars();
    let Some(mut needed) = query_chars.next() else {
        return true;
    };

    for ch in field.chars() {
        if ch != needed {
            continue;
        }
        let Some(next) = query_chars.next() else {
            return true;
        };
        needed = next;
    }

    false
}

fn normalized_link_autocomplete_query(input: &str) -> String {
    let mut normalized = String::new();
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                normalized.push(lower);
            }
        } else if matches!(ch, '-' | '_' | '\'') {
            normalized.push(ch);
        }
    }
    normalized
}

fn current_document_mentions_target(document: &str, target: &str) -> bool {
    document.contains(&format!("]({target})")) || document.contains(&format!("[{target}]"))
}

#[cfg(test)]
mod link_autocomplete_tests {
    use super::*;
    use bevy::ecs::system::{IntoSystem, System};
    use bevy::prelude::World;
    use std::path::PathBuf;

    fn document_trigger(text: &str, column: usize) -> Option<LinkAutocomplete> {
        link_autocomplete_at_cursor(
            &Document::from_text(text),
            Position { line: 0, column },
            LinkAutocompleteSource::Document,
        )
    }

    #[test]
    fn bracket_trigger_opens_without_prefix() {
        let active = document_trigger("[", 1).expect("autocomplete");

        assert_eq!(active.trigger, LinkAutocompleteTrigger::Bracket);
        assert_eq!(active.prefix, "");
        assert_eq!(active.range.start.column, 0);
        assert_eq!(active.range.end.column, 1);
    }

    #[test]
    fn bracket_trigger_keeps_label_prefix() {
        let active = document_trigger("[Eo", 3).expect("autocomplete");

        assert_eq!(active.trigger, LinkAutocompleteTrigger::Bracket);
        assert_eq!(active.prefix, "Eo");
        assert_eq!(active.range.start.column, 0);
        assert_eq!(active.range.end.column, 3);
    }

    #[test]
    fn inline_trigger_requires_two_visible_word_characters() {
        assert!(document_trigger("e", 1).is_none());

        let active = document_trigger("eo", 2).expect("autocomplete");
        assert_eq!(active.trigger, LinkAutocompleteTrigger::InlinePrefix);
        assert_eq!(active.prefix, "eo");
    }

    #[test]
    fn inline_trigger_preserves_typed_case_for_matching() {
        let active = document_trigger("EOG", 3).expect("autocomplete");

        assert_eq!(active.trigger, LinkAutocompleteTrigger::InlinePrefix);
        assert_eq!(active.prefix, "EOG");
    }

    #[test]
    fn hard_delimiters_close_inline_trigger() {
        assert!(document_trigger("eo ", 3).is_none());
        assert!(document_trigger("eo.", 3).is_none());
    }

    #[test]
    fn closed_bracketed_link_does_not_trigger() {
        assert!(document_trigger("[Eoghan]", 8).is_none());
        assert!(document_trigger("[Eoghan](eoghan)", 16).is_none());
    }

    #[test]
    fn autocomplete_ui_system_queries_are_disjoint() {
        let mut world = World::new();
        let mut system = IntoSystem::into_system(sync_link_autocomplete_ui);

        system.initialize(&mut world);
    }

    #[test]
    fn entity_scoring_prefers_prefix_match_over_fuzzy_match() {
        let eoghan = test_entity("eoghan", "character", "Eoghan");
        let fuzzy = test_entity("earl-ogden", "character", "Earl Ogden");
        let query = normalized_link_autocomplete_query("eog");

        let eoghan_score =
            score_entity_suggestion(&eoghan, &query, LinkAutocompleteLineContext::General, "")
                .expect("score");
        let fuzzy_score =
            score_entity_suggestion(&fuzzy, &query, LinkAutocompleteLineContext::General, "")
                .expect("score");

        assert!(eoghan_score > fuzzy_score);
    }

    #[test]
    fn entity_scoring_boosts_character_cue_characters() {
        let character = test_entity("eoghan", "character", "Eoghan");
        let prop = test_entity("eoghan-photo", "prop", "Eoghan Photo");
        let query = normalized_link_autocomplete_query("eog");

        let character_score = score_entity_suggestion(
            &character,
            &query,
            LinkAutocompleteLineContext::CharacterCue,
            "",
        )
        .expect("score");
        let prop_score =
            score_entity_suggestion(&prop, &query, LinkAutocompleteLineContext::CharacterCue, "")
                .expect("score");

        assert!(character_score > prop_score);
    }

    #[test]
    fn entity_scoring_boosts_scene_heading_places() {
        let place = test_entity("kitchen", "place", "Kitchen");
        let prop = test_entity("kitchen-knife", "prop", "Kitchen Knife");
        let query = normalized_link_autocomplete_query("kit");

        let place_score = score_entity_suggestion(
            &place,
            &query,
            LinkAutocompleteLineContext::SceneHeading,
            "",
        )
        .expect("score");
        let prop_score =
            score_entity_suggestion(&prop, &query, LinkAutocompleteLineContext::SceneHeading, "")
                .expect("score");

        assert!(place_score > prop_score);
    }

    fn test_entity(
        target: &str,
        entity_type: &str,
        name: &str,
    ) -> basscript_core::StoryIndexEntityRecord {
        basscript_core::StoryIndexEntityRecord {
            target: target.to_string(),
            entity_type: entity_type.to_string(),
            name: name.to_string(),
            status: None,
            path: PathBuf::from(format!("{target}.md")),
            relative_path: format!("{target}.md"),
        }
    }
}
