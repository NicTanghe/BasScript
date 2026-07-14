pub(crate) const MARKDOWN_METADATA_PANEL_HEIGHT: f32 = 94.0;
pub(crate) const MARKDOWN_METADATA_PANEL_GAP: f32 = 10.0;
pub(crate) const MARKDOWN_METADATA_PANEL_PADDING: f32 = 8.0;
pub(crate) const MARKDOWN_METADATA_ROW_GAP: f32 = 6.0;
pub(crate) const MARKDOWN_METADATA_COLUMN_GAP: f32 = 7.0;
pub(crate) const MARKDOWN_METADATA_FIELD_HEIGHT: f32 = 28.0;
pub(crate) const MARKDOWN_METADATA_FIELD_HORIZONTAL_PADDING: f32 = 8.0;
pub(crate) const MARKDOWN_METADATA_DROPDOWN_ROW_HEIGHT: f32 = 24.0;
pub(crate) const MARKDOWN_METADATA_DROPDOWN_VISIBLE_ROWS: usize = 7;
pub(crate) const MARKDOWN_METADATA_FONT_SIZE: f32 = 11.0;
pub(crate) const MARKDOWN_METADATA_BORDER_WIDTH: f32 = 1.0;
pub(crate) const MARKDOWN_METADATA_DROPDOWN_OFFSET: f32 = 2.0;
pub(crate) const MARKDOWN_METADATA_DROPDOWN_MIN_WIDTH: f32 = 120.0;
pub(crate) const COMMON_MARKDOWN_METADATA_TYPES: [&str; 7] = [
    "character",
    "prop",
    "place",
    "faction",
    "concept",
    "scene",
    "plot",
];
pub(crate) const COMMON_MARKDOWN_METADATA_STATUSES: [&str; 4] =
    ["draft", "active", "final", "archived"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MarkdownMetadataField {
    Id,
    Target,
    Type,
    Name,
    Aliases,
    Status,
}

pub(crate) const MARKDOWN_METADATA_FIELDS: [MarkdownMetadataField; 6] = [
    MarkdownMetadataField::Id,
    MarkdownMetadataField::Target,
    MarkdownMetadataField::Type,
    MarkdownMetadataField::Name,
    MarkdownMetadataField::Aliases,
    MarkdownMetadataField::Status,
];

impl MarkdownMetadataField {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Target => "target",
            Self::Type => "type",
            Self::Name => "name",
            Self::Aliases => "aliases",
            Self::Status => "status",
        }
    }

    pub(crate) fn index(self) -> usize {
        MARKDOWN_METADATA_FIELDS
            .iter()
            .position(|field| *field == self)
            .unwrap_or(0)
    }

    pub(crate) fn is_dropdown(self) -> bool {
        matches!(self, Self::Type | Self::Status)
    }

    pub(crate) fn next(self) -> Self {
        let next = (self.index() + 1) % MARKDOWN_METADATA_FIELDS.len();
        MARKDOWN_METADATA_FIELDS[next]
    }

    pub(crate) fn previous(self) -> Self {
        let previous = self
            .index()
            .checked_sub(1)
            .unwrap_or(MARKDOWN_METADATA_FIELDS.len() - 1);
        MARKDOWN_METADATA_FIELDS[previous]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MarkdownMetadataFields {
    pub(crate) id: String,
    pub(crate) target: String,
    pub(crate) entity_type: String,
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) status: String,
    pub(crate) unknown_lines: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct MarkdownFrontMatterDisplay {
    pub(crate) closing_line_index: usize,
    pub(crate) has_bom: bool,
    pub(crate) fields: MarkdownMetadataFields,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MarkdownMetadataChoiceSets {
    pub(crate) type_choices: Vec<String>,
    pub(crate) status_choices: Vec<String>,
}

impl MarkdownMetadataChoiceSets {
    pub(crate) fn for_field(&self, field: MarkdownMetadataField) -> &[String] {
        match field {
            MarkdownMetadataField::Type => &self.type_choices,
            MarkdownMetadataField::Status => &self.status_choices,
            _ => &[],
        }
    }
}

#[derive(Component)]
pub(crate) struct MarkdownMetadataPanelRoot;

#[derive(Component)]
pub(crate) struct MarkdownMetadataRow;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownMetadataFieldButton {
    pub(crate) field: MarkdownMetadataField,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownMetadataFieldText {
    pub(crate) field: MarkdownMetadataField,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownMetadataDropdownRoot {
    pub(crate) field: MarkdownMetadataField,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownMetadataDropdownOptionButton {
    pub(crate) field: MarkdownMetadataField,
    pub(crate) slot_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownMetadataDropdownOptionText {
    pub(crate) field: MarkdownMetadataField,
    pub(crate) slot_index: usize,
}

pub(crate) fn spawn_markdown_metadata_controls(
    parent: &mut ChildSpawnerCommands<'_>,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: px(0.0),
                height: px(MARKDOWN_METADATA_PANEL_HEIGHT),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: px(MARKDOWN_METADATA_ROW_GAP),
                padding: UiRect::all(px(MARKDOWN_METADATA_PANEL_PADDING)),
                overflow: Overflow::visible(),
                border: UiRect::all(px(MARKDOWN_METADATA_BORDER_WIDTH)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.92, 0.93, 0.95, 0.96)),
            BorderColor::all(Color::srgba(0.10, 0.12, 0.14, 0.16)),
            RelativeCursorPosition::default(),
            ZIndex(8),
            GlobalZIndex(8),
            MarkdownMetadataPanelRoot,
        ))
        .with_children(|root| {
            for row_index in 0..2 {
                root.spawn((
                    Node {
                        width: percent(100.0),
                        height: px(MARKDOWN_METADATA_FIELD_HEIGHT),
                        flex_direction: FlexDirection::Row,
                        column_gap: px(MARKDOWN_METADATA_COLUMN_GAP),
                        overflow: Overflow::visible(),
                        ..default()
                    },
                    MarkdownMetadataRow,
                ))
                .with_children(|row| {
                    for field in MARKDOWN_METADATA_FIELDS
                        .iter()
                        .copied()
                        .skip(row_index * 3)
                        .take(3)
                    {
                        row.spawn(markdown_metadata_field_button(font.clone(), field));
                    }
                });
            }

            for field in [MarkdownMetadataField::Type, MarkdownMetadataField::Status] {
                root.spawn(markdown_metadata_dropdown(font.clone(), field));
            }
        });
}

pub(crate) fn markdown_metadata_field_button(
    font: Handle<Font>,
    field: MarkdownMetadataField,
) -> impl Bundle {
    (
        Button,
        Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: px(0.0),
            min_width: px(0.0),
            height: px(MARKDOWN_METADATA_FIELD_HEIGHT),
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(MARKDOWN_METADATA_FIELD_HORIZONTAL_PADDING), px(0.0)),
            overflow: Overflow::clip(),
            border: UiRect::all(px(MARKDOWN_METADATA_BORDER_WIDTH)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        BorderColor::all(Color::srgba(0.10, 0.12, 0.14, 0.12)),
        MarkdownMetadataFieldButton { field },
        children![(
            Text::new(""),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(MARKDOWN_METADATA_FONT_SIZE),
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
            MarkdownMetadataFieldText { field },
        )],
    )
}

pub(crate) fn markdown_metadata_dropdown(
    font: Handle<Font>,
    field: MarkdownMetadataField,
) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: px(0.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip(),
            border: UiRect::all(px(MARKDOWN_METADATA_BORDER_WIDTH)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.97, 0.98, 0.99, 0.98)),
        BorderColor::all(Color::srgba(0.10, 0.12, 0.14, 0.18)),
        ZIndex(12),
        GlobalZIndex(12),
        MarkdownMetadataDropdownRoot { field },
        children![
            markdown_metadata_dropdown_option(font.clone(), field, 0),
            markdown_metadata_dropdown_option(font.clone(), field, 1),
            markdown_metadata_dropdown_option(font.clone(), field, 2),
            markdown_metadata_dropdown_option(font.clone(), field, 3),
            markdown_metadata_dropdown_option(font.clone(), field, 4),
            markdown_metadata_dropdown_option(font.clone(), field, 5),
            markdown_metadata_dropdown_option(font, field, 6),
        ],
    )
}

pub(crate) fn markdown_metadata_dropdown_option(
    font: Handle<Font>,
    field: MarkdownMetadataField,
    slot_index: usize,
) -> impl Bundle {
    (
        Button,
        Node {
            width: percent(100.0),
            height: px(MARKDOWN_METADATA_DROPDOWN_ROW_HEIGHT),
            display: Display::None,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(MARKDOWN_METADATA_FIELD_HORIZONTAL_PADDING), px(0.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        MarkdownMetadataDropdownOptionButton { field, slot_index },
        children![(
            Text::new(""),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(MARKDOWN_METADATA_FONT_SIZE),
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
            MarkdownMetadataDropdownOptionText { field, slot_index },
        )],
    )
}

pub(crate) fn sync_markdown_metadata_controls_ui(
    state: Res<EditorState>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut root_query: Query<&mut Node, With<MarkdownMetadataPanelRoot>>,
    mut row_query: Query<
        &mut Node,
        (
            With<MarkdownMetadataRow>,
            Without<MarkdownMetadataPanelRoot>,
            Without<MarkdownMetadataFieldButton>,
            Without<MarkdownMetadataDropdownRoot>,
            Without<MarkdownMetadataDropdownOptionButton>,
        ),
    >,
    mut field_button_query: Query<
        (
            &MarkdownMetadataFieldButton,
            &mut Node,
            &mut BackgroundColor,
        ),
        (
            Without<MarkdownMetadataPanelRoot>,
            Without<MarkdownMetadataRow>,
            Without<MarkdownMetadataDropdownRoot>,
            Without<MarkdownMetadataDropdownOptionButton>,
        ),
    >,
    mut field_text_query: Query<
        (&MarkdownMetadataFieldText, &mut Text, &mut TextFont),
        Without<MarkdownMetadataDropdownOptionText>,
    >,
    mut dropdown_root_query: Query<
        (&MarkdownMetadataDropdownRoot, &mut Node),
        (
            Without<MarkdownMetadataPanelRoot>,
            Without<MarkdownMetadataRow>,
            Without<MarkdownMetadataFieldButton>,
            Without<MarkdownMetadataDropdownOptionButton>,
        ),
    >,
    mut option_button_query: Query<
        (
            &MarkdownMetadataDropdownOptionButton,
            &mut Node,
            &mut BackgroundColor,
        ),
        (
            Without<MarkdownMetadataPanelRoot>,
            Without<MarkdownMetadataRow>,
            Without<MarkdownMetadataFieldButton>,
            Without<MarkdownMetadataDropdownRoot>,
        ),
    >,
    mut option_text_query: Query<
        (
            &MarkdownMetadataDropdownOptionText,
            &mut Text,
            &mut TextFont,
        ),
        Without<MarkdownMetadataFieldText>,
    >,
) {
    let Some(front_matter) = markdown_front_matter_display(&state.document) else {
        hide_markdown_metadata_controls(&mut root_query);
        return;
    };
    if state.document_format != DocumentFormat::Markdown
        || !state.panel_visible(PanelKind::Processed)
    {
        hide_markdown_metadata_controls(&mut root_query);
        return;
    }
    if !markdown_metadata_controls_scroll_visible(&state) {
        hide_markdown_metadata_controls(&mut root_query);
        return;
    }

    let Some(processed_panel_size) = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor())
        .filter(|size| size.x > 1.0 && size.y > 1.0)
    else {
        hide_markdown_metadata_controls(&mut root_query);
        return;
    };

    let layout = processed_page_layout(processed_panel_size, &state);
    let metrics = MarkdownMetadataLayoutMetrics::for_zoom(state.zoom);
    let choice_sets = markdown_metadata_choice_sets(&state, &front_matter.fields);
    let anchor_line_in_page = state
        .processed_top_visual
        .min(layout.page_step_lines.max(1).saturating_sub(1));
    let anchor_offset_px =
        processed_anchor_scroll_offset_px(anchor_line_in_page, scaled_line_height(&state).max(1.0));
    let left = layout.geometry.paper_left - state.processed_horizontal_scroll;
    let top =
        PAGE_OUTER_MARGIN - markdown_metadata_scrolled_header_offset(&state) - anchor_offset_px
            + state.processed_zoom_anchor_bias_px;
    if let Ok(mut root) = root_query.single_mut() {
        root.display = Display::Flex;
        root.left = px(left);
        root.top = px(top);
        root.width = px(layout.geometry.paper_width);
        root.height = px(metrics.panel_height);
        root.row_gap = px(metrics.row_gap);
        root.padding = UiRect::all(px(metrics.panel_padding));
        root.border = UiRect::all(px(metrics.border_width));
    }

    for mut row in row_query.iter_mut() {
        row.height = px(metrics.field_height);
        row.column_gap = px(metrics.column_gap);
    }

    for (button, mut node, mut background) in field_button_query.iter_mut() {
        node.height = px(metrics.field_height);
        node.padding = UiRect::axes(px(metrics.field_horizontal_padding), px(0.0));
        node.border = UiRect::all(px(metrics.border_width));
        background.0 = if state.markdown_metadata_focus == Some(button.field) {
            BUTTON_PRESSED
        } else {
            BUTTON_NORMAL
        };
    }

    for (slot, mut text, mut text_font) in field_text_query.iter_mut() {
        **text = markdown_metadata_field_label(
            &front_matter.fields,
            slot.field,
            state.markdown_metadata_focus == Some(slot.field),
        );
        text_font.font_size = FontSize::Px(metrics.font_size);
    }

    for (dropdown, mut node) in dropdown_root_query.iter_mut() {
        let choices = choice_sets.for_field(dropdown.field);
        let open = state.markdown_metadata_dropdown == Some(dropdown.field) && !choices.is_empty();
        node.display = if open { Display::Flex } else { Display::None };
        node.left = px(metrics.dropdown_left(layout.geometry.paper_width, dropdown.field));
        node.top = px(metrics.dropdown_top(dropdown.field));
        node.width = px(metrics.dropdown_width(layout.geometry.paper_width));
        node.height = px(
            (choices.len().min(MARKDOWN_METADATA_DROPDOWN_VISIBLE_ROWS) as f32)
                * metrics.dropdown_row_height,
        );
        node.border = UiRect::all(px(metrics.border_width));
    }

    for (option, mut node, mut background) in option_button_query.iter_mut() {
        let choices = choice_sets.for_field(option.field);
        let choice = choices.get(option.slot_index);
        node.display = if state.markdown_metadata_dropdown == Some(option.field) && choice.is_some()
        {
            Display::Flex
        } else {
            Display::None
        };
        background.0 = if state.markdown_metadata_dropdown == Some(option.field)
            && option.slot_index == state.markdown_metadata_dropdown_highlight
        {
            BUTTON_PRESSED
        } else {
            BUTTON_NORMAL
        };
        node.height = px(metrics.dropdown_row_height);
        node.padding = UiRect::axes(px(metrics.field_horizontal_padding), px(0.0));
    }

    for (slot, mut text, mut text_font) in option_text_query.iter_mut() {
        let choices = choice_sets.for_field(slot.field);
        **text = choices.get(slot.slot_index).cloned().unwrap_or_default();
        text_font.font_size = FontSize::Px(metrics.font_size);
    }
}

pub(crate) fn hide_markdown_metadata_controls(
    root_query: &mut Query<&mut Node, With<MarkdownMetadataPanelRoot>>,
) {
    if let Ok(mut root) = root_query.single_mut() {
        root.display = Display::None;
    }
}

pub(crate) fn markdown_metadata_field_label(
    fields: &MarkdownMetadataFields,
    field: MarkdownMetadataField,
    focused: bool,
) -> String {
    let mut value = markdown_metadata_field_value(fields, field);
    if focused {
        value.push('_');
    }
    let suffix = if field.is_dropdown() { " v" } else { "" };
    format!(
        "{}: {}{}",
        field.label(),
        compact_markdown_metadata_value(&value),
        suffix
    )
}

pub(crate) fn compact_markdown_metadata_value(value: &str) -> String {
    let mut compact = value.replace('\n', " ");
    const LIMIT: usize = 64;
    if compact.chars().count() > LIMIT {
        compact = compact
            .chars()
            .take(LIMIT.saturating_sub(1))
            .collect::<String>();
        compact.push_str("...");
    }
    compact
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MarkdownMetadataLayoutMetrics {
    panel_height: f32,
    panel_gap: f32,
    panel_padding: f32,
    row_gap: f32,
    column_gap: f32,
    field_height: f32,
    field_horizontal_padding: f32,
    dropdown_row_height: f32,
    font_size: f32,
    border_width: f32,
    dropdown_offset: f32,
    dropdown_min_width: f32,
}

impl MarkdownMetadataLayoutMetrics {
    fn for_zoom(zoom: f32) -> Self {
        let zoom = zoom.max(f32::EPSILON);
        Self {
            panel_height: MARKDOWN_METADATA_PANEL_HEIGHT * zoom,
            panel_gap: MARKDOWN_METADATA_PANEL_GAP * zoom,
            panel_padding: MARKDOWN_METADATA_PANEL_PADDING * zoom,
            row_gap: MARKDOWN_METADATA_ROW_GAP * zoom,
            column_gap: MARKDOWN_METADATA_COLUMN_GAP * zoom,
            field_height: MARKDOWN_METADATA_FIELD_HEIGHT * zoom,
            field_horizontal_padding: MARKDOWN_METADATA_FIELD_HORIZONTAL_PADDING * zoom,
            dropdown_row_height: MARKDOWN_METADATA_DROPDOWN_ROW_HEIGHT * zoom,
            font_size: MARKDOWN_METADATA_FONT_SIZE * zoom,
            border_width: MARKDOWN_METADATA_BORDER_WIDTH * zoom,
            dropdown_offset: MARKDOWN_METADATA_DROPDOWN_OFFSET * zoom,
            dropdown_min_width: MARKDOWN_METADATA_DROPDOWN_MIN_WIDTH * zoom,
        }
    }

    fn header_offset(self) -> f32 {
        self.panel_height + self.panel_gap
    }

    fn dropdown_left(self, panel_width: f32, field: MarkdownMetadataField) -> f32 {
        let usable = (panel_width - self.panel_padding * 2.0).max(1.0);
        let column_width = ((usable - self.column_gap * 2.0) / 3.0).max(1.0);
        let column = field.index() % 3;
        self.panel_padding + column as f32 * (column_width + self.column_gap)
    }

    fn dropdown_top(self, field: MarkdownMetadataField) -> f32 {
        let row = field.index() / 3;
        self.panel_padding
            + row as f32 * (self.field_height + self.row_gap)
            + self.field_height
            + self.dropdown_offset
    }

    fn dropdown_width(self, panel_width: f32) -> f32 {
        let usable = (panel_width - self.panel_padding * 2.0).max(1.0);
        ((usable - self.column_gap * 2.0) / 3.0).max(self.dropdown_min_width)
    }
}

pub(crate) fn handle_markdown_metadata_buttons(
    field_query: Query<
        (&Interaction, &MarkdownMetadataFieldButton),
        (Changed<Interaction>, With<Button>),
    >,
    option_query: Query<
        (&Interaction, &MarkdownMetadataDropdownOptionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    if state.document_format != DocumentFormat::Markdown {
        return;
    }

    for (interaction, option) in option_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(front_matter) = markdown_front_matter_display(&state.document) else {
            state.clear_markdown_metadata_focus();
            return;
        };
        let choices =
            markdown_metadata_dropdown_choices(&state, &front_matter.fields, option.field);
        let Some(value) = choices.get(option.slot_index).cloned() else {
            continue;
        };
        let snapshot = state.history_snapshot();
        match state.set_markdown_metadata_field(option.field, &value) {
            Ok(()) => {
                state.push_undo_snapshot(snapshot);
                state.reparse_with_dirty_hint(0);
                state.markdown_metadata_focus = Some(option.field);
                state.markdown_metadata_dropdown = None;
                state.status_message = format!("Updated metadata {}.", option.field.label());
            }
            Err(error) => state.status_message = error,
        }
        return;
    }

    for (interaction, button) in field_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if markdown_front_matter_display(&state.document).is_none() {
            state.clear_markdown_metadata_focus();
            return;
        }

        state.close_link_autocomplete();
        state.workspace_focused = false;
        state.selection_anchor = None;
        let was_open = state.markdown_metadata_dropdown == Some(button.field);
        state.markdown_metadata_focus = Some(button.field);
        if button.field.is_dropdown() {
            state.markdown_metadata_dropdown = (!was_open).then_some(button.field);
            state.markdown_metadata_dropdown_highlight =
                markdown_metadata_current_choice_index(&state, button.field).unwrap_or(0);
        } else {
            state.markdown_metadata_dropdown = None;
            state.markdown_metadata_dropdown_highlight = 0;
        }
        state.status_message = format!("Metadata {}.", button.field.label());
        return;
    }
}

pub(crate) fn handle_markdown_metadata_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
) {
    if state.markdown_metadata_focus.is_none() && state.markdown_metadata_dropdown.is_none() {
        return;
    }
    if state.command_menu.is_some()
        || state.workspace_prompt.is_some()
        || state.story_query_sheet.open
        || state.document_format != DocumentFormat::Markdown
    {
        return;
    }

    let mut undo_snapshot = None::<EditorHistorySnapshot>;
    let mut edited = false;

    if keys.just_pressed(KeyCode::Escape) {
        if state.markdown_metadata_dropdown.take().is_some() {
            state.status_message = "Metadata dropdown closed.".to_string();
        } else {
            state.clear_markdown_metadata_focus();
            state.status_message = "Metadata field closed.".to_string();
        }
        for _ in keyboard_inputs.read() {}
        return;
    }

    if let Some(dropdown) = state.markdown_metadata_dropdown {
        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowUp) {
            let choice_count = markdown_front_matter_display(&state.document)
                .map(|front_matter| {
                    markdown_metadata_dropdown_choices(&state, &front_matter.fields, dropdown).len()
                })
                .unwrap_or(0)
                .min(MARKDOWN_METADATA_DROPDOWN_VISIBLE_ROWS);
            if choice_count > 0 {
                if keys.just_pressed(KeyCode::ArrowDown) {
                    state.markdown_metadata_dropdown_highlight =
                        (state.markdown_metadata_dropdown_highlight + 1).min(choice_count - 1);
                } else {
                    state.markdown_metadata_dropdown_highlight =
                        state.markdown_metadata_dropdown_highlight.saturating_sub(1);
                }
            }
            for _ in keyboard_inputs.read() {}
            return;
        }
    }

    for input in keyboard_inputs.read() {
        if !input.state.is_pressed() {
            continue;
        }

        if text_input_should_skip_for_shortcut(&keys, input, &state.keybinds) {
            continue;
        }

        match input.key_code {
            KeyCode::Tab => {
                let current = state
                    .markdown_metadata_focus
                    .unwrap_or(MarkdownMetadataField::Id);
                let next = if shift_modifier_pressed(&keys) {
                    current.previous()
                } else {
                    current.next()
                };
                state.markdown_metadata_focus = Some(next);
                state.markdown_metadata_dropdown = None;
                state.markdown_metadata_dropdown_highlight = 0;
            }
            KeyCode::Enter => {
                if let Some(dropdown) = state.markdown_metadata_dropdown {
                    let Some(front_matter) = markdown_front_matter_display(&state.document) else {
                        state.clear_markdown_metadata_focus();
                        return;
                    };
                    let choices =
                        markdown_metadata_dropdown_choices(&state, &front_matter.fields, dropdown);
                    if let Some(value) = choices
                        .get(state.markdown_metadata_dropdown_highlight)
                        .cloned()
                    {
                        if undo_snapshot.is_none() {
                            undo_snapshot = Some(state.history_snapshot());
                        }
                        if let Err(error) = state.set_markdown_metadata_field(dropdown, &value) {
                            state.status_message = error;
                            continue;
                        }
                        state.markdown_metadata_dropdown = None;
                        edited = true;
                    }
                } else {
                    state.markdown_metadata_dropdown = None;
                }
            }
            KeyCode::Backspace => {
                let Some(field) = state.markdown_metadata_focus else {
                    continue;
                };
                let Some(front_matter) = markdown_front_matter_display(&state.document) else {
                    state.clear_markdown_metadata_focus();
                    return;
                };
                let mut value = markdown_metadata_field_value(&front_matter.fields, field);
                if value.pop().is_some() {
                    if undo_snapshot.is_none() {
                        undo_snapshot = Some(state.history_snapshot());
                    }
                    if let Err(error) = state.set_markdown_metadata_field(field, &value) {
                        state.status_message = error;
                        continue;
                    }
                    edited = true;
                }
            }
            KeyCode::Delete => {}
            _ => {
                let Some(field) = state.markdown_metadata_focus else {
                    continue;
                };
                let Some(inserted_text) = input.text.as_ref() else {
                    continue;
                };
                if inserted_text.is_empty()
                    || !inserted_text.chars().all(is_printable_char)
                    || !markdown_metadata_insert_text_allowed(field, inserted_text)
                {
                    continue;
                }
                let Some(front_matter) = markdown_front_matter_display(&state.document) else {
                    state.clear_markdown_metadata_focus();
                    return;
                };
                let mut value = markdown_metadata_field_value(&front_matter.fields, field);
                value.push_str(inserted_text);
                if undo_snapshot.is_none() {
                    undo_snapshot = Some(state.history_snapshot());
                }
                if let Err(error) = state.set_markdown_metadata_field(field, &value) {
                    state.status_message = error;
                    continue;
                }
                state.markdown_metadata_dropdown = None;
                edited = true;
            }
        }
    }

    if edited {
        if let Some(snapshot) = undo_snapshot {
            state.push_undo_snapshot(snapshot);
        }
        state.reparse_with_dirty_hint(0);
        state.status_message = "Updated metadata.".to_string();
    }
}

pub(crate) fn markdown_metadata_insert_text_allowed(
    field: MarkdownMetadataField,
    text: &str,
) -> bool {
    match field {
        MarkdownMetadataField::Target => text
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'),
        _ => true,
    }
}

pub(crate) fn markdown_metadata_current_choice_index(
    state: &EditorState,
    field: MarkdownMetadataField,
) -> Option<usize> {
    let front_matter = markdown_front_matter_display(&state.document)?;
    let current = markdown_metadata_field_value(&front_matter.fields, field);
    markdown_metadata_dropdown_choices(state, &front_matter.fields, field)
        .iter()
        .position(|choice| choice == &current)
}

pub(crate) fn markdown_metadata_dropdown_choices(
    state: &EditorState,
    fields: &MarkdownMetadataFields,
    field: MarkdownMetadataField,
) -> Vec<String> {
    markdown_metadata_choice_sets(state, fields)
        .for_field(field)
        .to_vec()
}

pub(crate) fn markdown_metadata_choice_sets(
    state: &EditorState,
    fields: &MarkdownMetadataFields,
) -> MarkdownMetadataChoiceSets {
    let mut type_choices = BTreeSet::<String>::new();
    let mut status_choices = BTreeSet::<String>::new();

    for choice in COMMON_MARKDOWN_METADATA_TYPES {
        type_choices.insert(choice.to_string());
    }
    for choice in COMMON_MARKDOWN_METADATA_STATUSES {
        status_choices.insert(choice.to_string());
    }

    if let Some(database) = state
        .story_index
        .as_ref()
        .and_then(|index| index.database.as_ref())
        && let Ok(entities) = database.all_entities()
    {
        for entity in entities {
            if !entity.entity_type.trim().is_empty() {
                type_choices.insert(entity.entity_type.trim().to_string());
            }
            if let Some(status) = entity.status.as_ref().map(|status| status.trim()) {
                if !status.is_empty() {
                    status_choices.insert(status.to_string());
                }
            }
        }
    }

    if !fields.entity_type.trim().is_empty() {
        type_choices.insert(fields.entity_type.clone());
    }
    if !fields.status.trim().is_empty() {
        status_choices.insert(fields.status.clone());
    }

    MarkdownMetadataChoiceSets {
        type_choices: type_choices.into_iter().collect(),
        status_choices: status_choices.into_iter().collect(),
    }
}

pub(crate) fn markdown_metadata_field_value(
    fields: &MarkdownMetadataFields,
    field: MarkdownMetadataField,
) -> String {
    match field {
        MarkdownMetadataField::Id => fields.id.clone(),
        MarkdownMetadataField::Target => fields.target.clone(),
        MarkdownMetadataField::Type => fields.entity_type.clone(),
        MarkdownMetadataField::Name => fields.name.clone(),
        MarkdownMetadataField::Aliases => fields.aliases.join(", "),
        MarkdownMetadataField::Status => fields.status.clone(),
    }
}

pub(crate) fn set_markdown_metadata_field_value(
    fields: &mut MarkdownMetadataFields,
    field: MarkdownMetadataField,
    value: &str,
) {
    match field {
        MarkdownMetadataField::Id => fields.id = value.trim().to_string(),
        MarkdownMetadataField::Target => fields.target = normalize_markdown_target_key(value),
        MarkdownMetadataField::Type => fields.entity_type = value.trim().to_string(),
        MarkdownMetadataField::Name => fields.name = value.to_string(),
        MarkdownMetadataField::Aliases => fields.aliases = parse_alias_input(value),
        MarkdownMetadataField::Status => fields.status = value.trim().to_string(),
    }
}

pub(crate) fn parse_alias_input(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|part| part.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

pub(crate) fn markdown_metadata_controls_scroll_visible(state: &EditorState) -> bool {
    markdown_metadata_full_header_offset(state) > 0.0
        && state.processed_header_scroll_progress < 1.0 - f32::EPSILON
}

pub(crate) fn markdown_metadata_full_header_offset(state: &EditorState) -> f32 {
    if state.document_format == DocumentFormat::Markdown
        && markdown_front_matter_display(&state.document).is_some()
    {
        MarkdownMetadataLayoutMetrics::for_zoom(state.zoom).header_offset()
    } else {
        0.0
    }
}

pub(crate) fn markdown_metadata_scrolled_header_offset(state: &EditorState) -> f32 {
    markdown_metadata_full_header_offset(state)
        * state.processed_header_scroll_progress.clamp(0.0, 1.0)
}

pub(crate) fn markdown_metadata_header_offset(state: &EditorState) -> f32 {
    markdown_metadata_full_header_offset(state)
        * (1.0 - state.processed_header_scroll_progress.clamp(0.0, 1.0))
}

pub(crate) fn markdown_metadata_hovered(
    query: &Query<&RelativeCursorPosition, With<MarkdownMetadataPanelRoot>>,
) -> bool {
    query.iter().any(RelativeCursorPosition::cursor_over)
}

pub(crate) fn markdown_front_matter_display(
    document: &Document,
) -> Option<MarkdownFrontMatterDisplay> {
    let lines = document.lines();
    if lines.len() < 3 {
        return None;
    }

    let first = lines.first()?;
    let has_bom = first.starts_with('\u{feff}');
    if !line_is_front_matter_delimiter(first, true) {
        return None;
    }

    let closing_line_index =
        lines.iter().enumerate().skip(1).find_map(|(index, line)| {
            line_is_front_matter_delimiter(line, false).then_some(index)
        })?;
    let fields = parse_markdown_front_matter_fields(&lines[1..closing_line_index]);
    Some(MarkdownFrontMatterDisplay {
        closing_line_index,
        has_bom,
        fields,
    })
}

pub(crate) fn line_is_front_matter_delimiter(line: &str, allow_bom: bool) -> bool {
    let line = if allow_bom {
        line.trim_start_matches('\u{feff}')
    } else {
        line
    };
    line.trim() == "---"
}

pub(crate) fn parse_markdown_front_matter_fields(lines: &[String]) -> MarkdownMetadataFields {
    let mut fields = MarkdownMetadataFields::default();
    let mut index = 0usize;

    while index < lines.len() {
        let line = &lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            fields.unknown_lines.push(line.clone());
            index += 1;
            continue;
        }

        let Some((key, value)) = trimmed.split_once(':') else {
            fields.unknown_lines.push(line.clone());
            index += 1;
            continue;
        };

        let key = key.trim();
        let value = value.trim();
        match key {
            "id" => fields.id = markdown_yaml_scalar(value),
            "target" => fields.target = markdown_yaml_scalar(value),
            "type" => fields.entity_type = markdown_yaml_scalar(value),
            "name" => fields.name = markdown_yaml_scalar(value),
            "status" => fields.status = markdown_yaml_scalar(value),
            "aliases" => {
                if value.is_empty() {
                    let mut aliases = Vec::<String>::new();
                    index += 1;
                    while let Some(alias_line) = lines.get(index) {
                        let alias_trimmed = alias_line.trim();
                        let Some(rest) = alias_trimmed.strip_prefix('-') else {
                            break;
                        };
                        let alias = markdown_yaml_scalar(rest.trim());
                        if !alias.is_empty() {
                            aliases.push(alias);
                        }
                        index += 1;
                    }
                    fields.aliases = aliases;
                    continue;
                }
                fields.aliases = parse_markdown_yaml_array(value).unwrap_or_else(|| {
                    let scalar = markdown_yaml_scalar(value);
                    (!scalar.is_empty()).then_some(scalar).into_iter().collect()
                });
            }
            _ => fields.unknown_lines.push(line.clone()),
        }

        index += 1;
    }

    fields
}

pub(crate) fn parse_markdown_yaml_array(value: &str) -> Option<Vec<String>> {
    let value = value.trim();
    if !(value.starts_with('[') && value.ends_with(']')) {
        return None;
    }
    let inner = &value[1..value.len().saturating_sub(1)];
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::<String>::new();
    let mut current = String::new();
    let mut quote = None::<char>;
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        match quote {
            Some(active) if ch == active => {
                if active == '\'' && chars.peek() == Some(&'\'') {
                    current.push('\'');
                    chars.next();
                } else {
                    quote = None;
                    current.push(ch);
                }
            }
            Some(_) => current.push(ch),
            None if ch == '\'' || ch == '"' => {
                quote = Some(ch);
                current.push(ch);
            }
            None if ch == ',' => {
                let item = markdown_yaml_scalar(current.trim());
                if !item.is_empty() {
                    items.push(item);
                }
                current.clear();
            }
            None => current.push(ch),
        }
    }
    let item = markdown_yaml_scalar(current.trim());
    if !item.is_empty() {
        items.push(item);
    }
    Some(items)
}

pub(crate) fn markdown_yaml_scalar(value: &str) -> String {
    let trimmed = value.trim();
    match trimmed.chars().next() {
        Some('"') if trimmed.ends_with('"') && trimmed.len() >= 2 => {
            trimmed[1..trimmed.len().saturating_sub(1)].to_owned()
        }
        Some('\'') if trimmed.ends_with('\'') && trimmed.len() >= 2 => {
            trimmed[1..trimmed.len().saturating_sub(1)].replace("''", "'")
        }
        _ => trimmed.to_owned(),
    }
}

pub(crate) fn write_markdown_front_matter_fields(fields: &MarkdownMetadataFields) -> Vec<String> {
    let mut lines = Vec::<String>::new();
    if !fields.id.trim().is_empty() {
        lines.push(format!("id: {}", yaml_scalar(&fields.id)));
    }
    lines.push(format!("target: {}", yaml_scalar(&fields.target)));
    lines.push(format!("type: {}", yaml_scalar(&fields.entity_type)));
    lines.push(format!("name: {}", yaml_scalar(&fields.name)));
    lines.push(format!("aliases: {}", yaml_array(&fields.aliases)));
    if !fields.status.trim().is_empty() {
        lines.push(format!("status: {}", yaml_scalar(&fields.status)));
    }
    lines.extend(fields.unknown_lines.iter().cloned());
    lines
}

pub(crate) fn yaml_array(values: &[String]) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| yaml_scalar(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(crate) fn yaml_scalar(value: &str) -> String {
    let trimmed = value.trim();
    if yaml_plain_scalar_safe(trimmed) {
        return trimmed.to_string();
    }
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn yaml_plain_scalar_safe(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if matches!(
        value.to_ascii_lowercase().as_str(),
        "true" | "false" | "null" | "~"
    ) {
        return false;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

pub(crate) fn normalize_markdown_target_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub(crate) fn normalize_markdown_front_matter_target(document: &Document) -> Option<Document> {
    let front_matter = markdown_front_matter_display(document)?;
    let normalized = normalize_markdown_target_key(&front_matter.fields.target);
    if normalized == front_matter.fields.target {
        return None;
    }

    let mut lines = document.lines().to_vec();
    for line_index in 1..front_matter.closing_line_index {
        let line = lines.get(line_index)?;
        let trimmed = line.trim();
        let Some((key, _)) = trimmed.split_once(':') else {
            continue;
        };
        if key.trim() != "target" {
            continue;
        }

        let indent: String = line
            .chars()
            .take_while(|ch| ch.is_ascii_whitespace())
            .collect();
        lines[line_index] = format!("{indent}target: {}", yaml_scalar(&normalized));
        return Some(Document::from_text(&lines.join("\n")));
    }

    None
}

impl EditorState {
    pub(crate) fn clear_markdown_metadata_focus(&mut self) {
        self.markdown_metadata_focus = None;
        self.markdown_metadata_dropdown = None;
        self.markdown_metadata_dropdown_highlight = 0;
    }

    pub(crate) fn markdown_metadata_input_active(&self) -> bool {
        self.markdown_metadata_focus.is_some() || self.markdown_metadata_dropdown.is_some()
    }

    pub(crate) fn set_markdown_metadata_field(
        &mut self,
        field: MarkdownMetadataField,
        value: &str,
    ) -> Result<(), String> {
        let Some(front_matter) = markdown_front_matter_display(&self.document) else {
            return Err("No YAML front matter found.".to_string());
        };
        let mut fields = front_matter.fields.clone();
        set_markdown_metadata_field_value(&mut fields, field, value);
        if !fields.target.is_empty() && !basscript_core::is_valid_target_key(&fields.target) {
            return Err(format!("Invalid target key `{}`.", fields.target));
        }

        let mut lines = self.document.lines().to_vec();
        let mut replacement = Vec::<String>::new();
        replacement.push(if front_matter.has_bom {
            "\u{feff}---".to_string()
        } else {
            "---".to_string()
        });
        replacement.extend(write_markdown_front_matter_fields(&fields));
        replacement.push("---".to_string());
        lines.splice(0..=front_matter.closing_line_index, replacement);

        self.document = Document::from_text(&lines.join("\n"));
        self.cursor.position = self.document.clamp_position(self.cursor.position);
        self.cursor.preferred_column = self
            .cursor
            .preferred_column
            .min(self.document.line_len_chars(self.cursor.position.line));
        self.processed_top_visual = 0;
        self.processed_header_scroll_progress = 0.0;
        self.processed_zoom_anchor_bias_px = 0.0;
        Ok(())
    }
}

#[cfg(test)]
mod markdown_metadata_tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn front_matter_layout_metrics_follow_document_zoom() {
        let zoomed_out = MarkdownMetadataLayoutMetrics::for_zoom(0.6);
        let metrics = MarkdownMetadataLayoutMetrics::for_zoom(1.65);

        assert_close(zoomed_out.panel_height, 56.4);
        assert_close(zoomed_out.header_offset(), 62.4);
        assert_close(zoomed_out.font_size, 6.6);
        assert_close(metrics.panel_height, 155.1);
        assert_close(metrics.header_offset(), 171.6);
        assert_close(metrics.field_height, 46.2);
        assert_close(metrics.dropdown_row_height, 39.6);
        assert_close(metrics.font_size, 18.15);
        assert_close(metrics.dropdown_top(MarkdownMetadataField::Type), 62.7);
        assert_close(metrics.dropdown_top(MarkdownMetadataField::Status), 118.8);
        assert_close(metrics.dropdown_width(300.0), 198.0);
    }

    #[test]
    fn front_matter_header_reservation_uses_document_zoom() {
        let mut world = World::new();
        let mut state = EditorState::from_world(&mut world);
        state.document_format = DocumentFormat::Markdown;
        state.document = Document::from_text("---\nid: entity_eoghan_001\n---\nBody");
        state.zoom = 1.65;

        assert_close(markdown_metadata_header_offset(&state), 171.6);

        state.processed_header_scroll_progress = 0.6;
        let metrics = MarkdownMetadataLayoutMetrics::for_zoom(state.zoom);
        let panel_top = PAGE_OUTER_MARGIN - markdown_metadata_scrolled_header_offset(&state);
        let paper_top = PAGE_OUTER_MARGIN + markdown_metadata_header_offset(&state);
        assert_close(
            paper_top - (panel_top + metrics.panel_height),
            metrics.panel_gap,
        );
    }

    #[test]
    fn front_matter_ui_system_queries_are_disjoint() {
        let mut world = World::new();
        let mut system = IntoSystem::into_system(sync_markdown_metadata_controls_ui);

        system.initialize(&mut world);
    }

    #[test]
    fn parses_front_matter_with_bom_and_aliases() {
        let document = Document::from_text(
            "\u{feff}---\nid: entity_the_unruly_market_001\ntarget: the-unruly-market\ntype: plot\nname: 'The Unruly Market'\naliases: ['Market', 'bazaar']\nstatus: draft\n---\nBody\n",
        );
        let display = markdown_front_matter_display(&document).expect("front matter");
        assert!(display.has_bom);
        assert_eq!(display.closing_line_index, 7);
        assert_eq!(display.fields.target, "the-unruly-market");
        assert_eq!(display.fields.entity_type, "plot");
        assert_eq!(display.fields.name, "The Unruly Market");
        assert_eq!(display.fields.aliases, vec!["Market", "bazaar"]);
    }

    #[test]
    fn writes_known_fields_and_preserves_unknown_lines() {
        let mut fields = MarkdownMetadataFields {
            id: "entity_the_unruly_market_001".to_string(),
            target: "the-unruly-market".to_string(),
            entity_type: "plot".to_string(),
            name: "The Unruly Market".to_string(),
            aliases: vec!["Market".to_string()],
            status: "draft".to_string(),
            unknown_lines: vec!["custom: value".to_string()],
        };
        set_markdown_metadata_field_value(&mut fields, MarkdownMetadataField::Name, "New Name");
        assert_eq!(
            write_markdown_front_matter_fields(&fields),
            vec![
                "id: entity_the_unruly_market_001",
                "target: the-unruly-market",
                "type: plot",
                "name: 'New Name'",
                "aliases: [Market]",
                "status: draft",
                "custom: value",
            ]
        );
    }

    #[test]
    fn lowercases_target_metadata_values() {
        let mut fields = MarkdownMetadataFields::default();
        set_markdown_metadata_field_value(&mut fields, MarkdownMetadataField::Target, " Elisah ");
        assert_eq!(fields.target, "elisah");
    }

    #[test]
    fn normalizes_front_matter_target_without_rewriting_body() {
        let document = Document::from_text(
            "---\nid: entity_elisah_001\ntarget: 'Elisah'\ntype: character\nname: 'Elisah'\naliases: [elisah]\n---\nBody [Elisah](Elisah)\n",
        );
        let normalized =
            normalize_markdown_front_matter_target(&document).expect("target should change");

        assert_eq!(
            normalized.to_text(),
            "---\nid: entity_elisah_001\ntarget: elisah\ntype: character\nname: 'Elisah'\naliases: [elisah]\n---\nBody [Elisah](Elisah)\n"
        );
    }
}
#[allow(unused_imports)]
use super::*;
