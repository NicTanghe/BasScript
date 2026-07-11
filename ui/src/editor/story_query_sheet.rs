use basscript_core::{StoryIndexAppearanceRecord, StoryIndexEntityRecord, StoryIndexSceneRecord};
use serde::Deserialize;

pub(crate) const STORY_QUERY_RESULT_WIDTH_PERCENT: f32 = 61.803;
pub(crate) const STORY_QUERY_MENU_WIDTH_PERCENT: f32 = 38.197;
pub(crate) const STORY_QUERY_DROPDOWN_VISIBLE_OPTIONS: usize = 8;
pub(crate) const STORY_QUERY_MAX_CATEGORY_ROWS: usize = 8;
pub(crate) const DEFAULT_STORY_TAXONOMY_RON: &str = r#"(
	categories: [
		(
			id: "props",
			label: "Props",
			types: ["prop", "object", "artifact", "tool", "weapon", "document", "clothing"],
		),
		(
			id: "environment",
			label: "Environment",
			types: ["place", "location", "set", "set-piece", "set_dressing", "furniture", "building", "vehicle", "weather"],
		),
		(
			id: "characters",
			label: "Characters",
			types: ["character"],
		),
		(
			id: "fauna",
			label: "Fauna",
			types: ["animal", "creature", "monster", "mount"],
		),
		(
			id: "flora",
			label: "Flora",
			types: ["plant", "tree", "flower", "fungus"],
		),
	],
)"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoryQueryKind {
    DialogueByCharacter,
    DialogueBetweenCharacters,
    DialogueBetweenAtScene,
    CategoriesInScene,
    AppearancesOfEntity,
}

pub(crate) const STORY_QUERY_KINDS: [StoryQueryKind; 5] = [
    StoryQueryKind::DialogueByCharacter,
    StoryQueryKind::DialogueBetweenCharacters,
    StoryQueryKind::DialogueBetweenAtScene,
    StoryQueryKind::CategoriesInScene,
    StoryQueryKind::AppearancesOfEntity,
];

impl StoryQueryKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::DialogueByCharacter => "All dialogue by character",
            Self::DialogueBetweenCharacters => "Dialogue between characters",
            Self::DialogueBetweenAtScene => "Dialogue at scene/location",
            Self::CategoriesInScene => "Entities by category",
            Self::AppearancesOfEntity => "Appearances of entity",
        }
    }

    pub(crate) fn index(self) -> usize {
        STORY_QUERY_KINDS
            .iter()
            .position(|kind| *kind == self)
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoryQuerySceneScope {
    Current,
    Selected,
    All,
}

pub(crate) const STORY_QUERY_SCENE_SCOPES: [StoryQuerySceneScope; 3] = [
    StoryQuerySceneScope::Current,
    StoryQuerySceneScope::Selected,
    StoryQuerySceneScope::All,
];

impl StoryQuerySceneScope {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Current => "Current scene",
            Self::Selected => "Selected scene",
            Self::All => "All scenes",
        }
    }

    pub(crate) fn index(self) -> usize {
        STORY_QUERY_SCENE_SCOPES
            .iter()
            .position(|scope| *scope == self)
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct StoryTaxonomyConfig {
    #[serde(default)]
    pub(crate) categories: Vec<StoryTaxonomyCategory>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct StoryTaxonomyCategory {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) types: Vec<String>,
}

pub(crate) struct StoryTaxonomyLoad {
    pub(crate) taxonomy: StoryTaxonomyConfig,
    pub(crate) notice: Option<String>,
}

impl Default for StoryTaxonomyConfig {
    fn default() -> Self {
        Self::from_ron(DEFAULT_STORY_TAXONOMY_RON)
            .expect("DEFAULT_STORY_TAXONOMY_RON must be valid")
    }
}

impl StoryTaxonomyConfig {
    pub(crate) fn from_ron(contents: &str) -> Result<Self, String> {
        let mut taxonomy = ron::from_str::<Self>(contents)
            .map_err(|error| format!("Could not parse taxonomy RON: {error}"))?;
        taxonomy.normalize();
        if taxonomy.categories.is_empty() {
            return Err("Taxonomy must define at least one non-empty category.".to_string());
        }
        Ok(taxonomy)
    }

    pub(crate) fn normalize(&mut self) {
        for category in &mut self.categories {
            category.id = category.id.trim().to_ascii_lowercase();
            category.label = category.label.trim().to_string();
            category.types = category
                .types
                .iter()
                .map(|entity_type| normalize_story_taxonomy_key(entity_type))
                .filter(|entity_type| !entity_type.is_empty())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
        }
        self.categories
            .retain(|category| !category.id.is_empty() && !category.types.is_empty());
    }
}

impl StoryTaxonomyCategory {
    pub(crate) fn display_label(&self) -> &str {
        if self.label.is_empty() {
            &self.id
        } else {
            &self.label
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoryQueryOutputFormat {
    Fountain,
    Markdown,
}

#[derive(Clone, Debug)]
pub(crate) struct StoryQuerySheet {
    pub(crate) open: bool,
    pub(crate) query_kind: StoryQueryKind,
    pub(crate) characters: Vec<StoryIndexEntityRecord>,
    pub(crate) entities: Vec<StoryIndexEntityRecord>,
    pub(crate) scenes: Vec<StoryIndexSceneRecord>,
    pub(crate) character_a_index: usize,
    pub(crate) character_b_index: usize,
    pub(crate) entity_index: usize,
    pub(crate) scene_index: usize,
    pub(crate) scene_scope: StoryQuerySceneScope,
    pub(crate) category_indices: Vec<usize>,
    pub(crate) taxonomy: StoryTaxonomyConfig,
    pub(crate) taxonomy_notice: Option<String>,
    pub(crate) open_dropdown: Option<StoryQueryDropdownKind>,
    pub(crate) result_scroll_visual: usize,
    pub(crate) result_scroll_anchor_bias_px: f32,
    pub(crate) result_horizontal_scroll: f32,
    pub(crate) visual_lines: Vec<ProcessedVisualLine>,
    pub(crate) visual_layout_signature: Option<StoryQueryLayoutSignature>,
    pub(crate) result_title: String,
    pub(crate) result_status: String,
    pub(crate) result_text: String,
    pub(crate) result_format: StoryQueryOutputFormat,
    pub(crate) dialogue_double_space_newline: bool,
    pub(crate) non_dialogue_double_space_newline: bool,
    pub(crate) source_targets: Vec<StoryQuerySourceTarget>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryLayoutSignature {
    pub(crate) output_format: StoryQueryOutputFormat,
    pub(crate) wrap_columns: usize,
    pub(crate) lines_per_page: usize,
    pub(crate) spacer_lines: usize,
    pub(crate) dialogue_double_space_newline: bool,
    pub(crate) non_dialogue_double_space_newline: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct StoryQuerySourceTarget {
    pub(crate) path: PathBuf,
    pub(crate) line: usize,
}

impl Default for StoryQuerySheet {
    fn default() -> Self {
        Self {
            open: false,
            query_kind: StoryQueryKind::DialogueByCharacter,
            characters: Vec::new(),
            entities: Vec::new(),
            scenes: Vec::new(),
            character_a_index: 0,
            character_b_index: 1,
            entity_index: 0,
            scene_index: 0,
            scene_scope: StoryQuerySceneScope::Current,
            category_indices: vec![0],
            taxonomy: StoryTaxonomyConfig::default(),
            taxonomy_notice: None,
            open_dropdown: None,
            result_scroll_visual: 0,
            result_scroll_anchor_bias_px: 0.0,
            result_horizontal_scroll: 0.0,
            visual_lines: Vec::new(),
            visual_layout_signature: None,
            result_title: "Story Query Sheet".to_string(),
            result_status: "Ready.".to_string(),
            result_text: "No result yet.".to_string(),
            result_format: StoryQueryOutputFormat::Markdown,
            dialogue_double_space_newline: false,
            non_dialogue_double_space_newline: false,
            source_targets: Vec::new(),
        }
    }
}

#[derive(Component)]
pub(crate) struct StoryQuerySheetRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoryQuerySheetTextSlot {
    Title,
    Status,
    QueryKind,
    SceneScope,
    PrimaryCharacter,
    SecondaryCharacter,
    Entity,
    Scene,
    Category(usize),
    AddCategory,
    Page,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoryQueryDropdownKind {
    QueryKind,
    SceneScope,
    PrimaryCharacter,
    SecondaryCharacter,
    Entity,
    Scene,
    Category(usize),
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoryQuerySheetAction {
    ToggleDropdown(StoryQueryDropdownKind),
    SelectDropdownOption {
        kind: StoryQueryDropdownKind,
        slot_index: usize,
    },
    Run,
    OpenFirstSource,
    AddCategory,
    Close,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryControlRow {
    pub(crate) slot: StoryQuerySheetTextSlot,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryDropdownOptionsRoot {
    pub(crate) kind: StoryQueryDropdownKind,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryDropdownOptionNode {
    pub(crate) kind: StoryQueryDropdownKind,
    pub(crate) slot_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryDropdownOptionText {
    pub(crate) kind: StoryQueryDropdownKind,
    pub(crate) slot_index: usize,
}

#[derive(Component)]
pub(crate) struct StoryQueryResultPanel;

#[derive(Component)]
pub(crate) struct StoryQueryResultCanvas;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryPaper {
    pub(crate) slot: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryRenderedText {
    pub(crate) slot: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoryQueryRenderedLineSpan {
    pub(crate) slot: usize,
    pub(crate) line_offset: usize,
    pub(crate) part_index: usize,
}

pub(crate) struct StoryQueryRunOutput {
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) format: StoryQueryOutputFormat,
    pub(crate) text: String,
    pub(crate) source_targets: Vec<StoryQuerySourceTarget>,
}

pub(crate) type StoryQueryCategoryGroup =
    BTreeMap<usize, BTreeMap<String, Vec<StoryIndexAppearanceRecord>>>;

pub(crate) fn story_query_sheet_bundle(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: percent(100.0),
            height: percent(100.0),
            display: Display::None,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            ..default()
        },
        BackgroundColor(COLOR_PANEL_BG),
        ZIndex(92),
        GlobalZIndex(92),
        StoryQuerySheetRoot,
        children![(
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                ..default()
            },
            children![
                story_query_sheet_page_bundle(font.clone()),
                story_query_sheet_menu_bundle(font.clone()),
            ],
        )],
    )
}

pub(crate) fn story_query_sheet_page_bundle(_font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: percent(STORY_QUERY_RESULT_WIDTH_PERCENT),
            height: percent(100.0),
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(COLOR_PANEL_BODY_PROCESSED),
        RelativeCursorPosition::default(),
        StoryQueryResultPanel,
        children![(
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
                ..default()
            },
            UiTransform::default(),
            StoryQueryResultCanvas,
        )],
    )
}

pub(crate) fn story_query_sheet_menu_bundle(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: percent(STORY_QUERY_MENU_WIDTH_PERCENT),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(7.0),
            padding: UiRect::all(px(14.0)),
            overflow: Overflow::clip(),
            border: UiRect::left(px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.10)),
        BackgroundColor(Color::srgb(0.88, 0.89, 0.91)),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3.0),
                    padding: UiRect::bottom(px(3.0)),
                    ..default()
                },
                children![
                    story_query_text(
                        font.clone(),
                        "Story Query Sheet",
                        16.0,
                        COLOR_TEXT_MAIN,
                        StoryQuerySheetTextSlot::Title,
                    ),
                    story_query_text(
                        font.clone(),
                        "",
                        11.0,
                        COLOR_TEXT_MUTED,
                        StoryQuerySheetTextSlot::Status,
                    ),
                    story_query_text(
                        font.clone(),
                        "",
                        10.0,
                        COLOR_TEXT_MUTED,
                        StoryQuerySheetTextSlot::Page,
                    ),
                ],
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::QueryKind,
                StoryQueryDropdownKind::QueryKind,
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::SceneScope,
                StoryQueryDropdownKind::SceneScope,
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(0),
                StoryQueryDropdownKind::Category(0),
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(1),
                StoryQueryDropdownKind::Category(1),
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(2),
                StoryQueryDropdownKind::Category(2),
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(3),
                StoryQueryDropdownKind::Category(3),
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(4),
                StoryQueryDropdownKind::Category(4),
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(5),
                StoryQueryDropdownKind::Category(5),
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(6),
                StoryQueryDropdownKind::Category(6),
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Category(7),
                StoryQueryDropdownKind::Category(7),
            ),
            story_query_add_category_button_row(font.clone()),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::PrimaryCharacter,
                StoryQueryDropdownKind::PrimaryCharacter,
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::SecondaryCharacter,
                StoryQueryDropdownKind::SecondaryCharacter,
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Entity,
                StoryQueryDropdownKind::Entity,
            ),
            story_query_dropdown_row(
                font.clone(),
                StoryQuerySheetTextSlot::Scene,
                StoryQueryDropdownKind::Scene,
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8.0),
                    ..default()
                },
                children![
                    story_query_static_button(font.clone(), "Run", StoryQuerySheetAction::Run),
                    story_query_static_button(
                        font.clone(),
                        "Open source",
                        StoryQuerySheetAction::OpenFirstSource,
                    ),
                    story_query_static_button(font.clone(), "Close", StoryQuerySheetAction::Close),
                ],
            ),
        ],
    )
}

pub(crate) fn setup_story_query_sheet_result_spans(
    mut commands: Commands,
    canvas_query: Query<Entity, With<StoryQueryResultCanvas>>,
    fonts: Res<EditorFonts>,
) {
    let regular_font = fonts.regular.clone();
    let span_capacity = processed_page_step_lines().max(1);

    for entity in canvas_query.iter() {
        commands.entity(entity).with_children(|parent| {
            for slot in 0..PROCESSED_PAPER_CAPACITY {
                let slot_font = regular_font.clone();
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        UiTransform::default(),
                        BackgroundColor(COLOR_PAPER),
                        Visibility::Hidden,
                        ZIndex(0),
                        StoryQueryPaper { slot },
                    ))
                    .with_children(|paper| {
                        paper
                            .spawn((
                                Text::new(""),
                                TextLayout::no_wrap(),
                                TextFont {
                                    font: slot_font.clone().into(),
                                    font_size: FontSize::Px(FONT_SIZE),
                                    ..default()
                                },
                                LineHeight::Px(LINE_HEIGHT),
                                TextColor(COLOR_ACTION),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(PAGE_TEXT_MARGIN_LEFT),
                                    top: px(PAGE_TEXT_MARGIN_TOP),
                                    width: px((A4_WIDTH_POINTS
                                        - PAGE_TEXT_MARGIN_LEFT
                                        - PAGE_TEXT_MARGIN_RIGHT)
                                        .max(1.0)),
                                    height: px((A4_HEIGHT_POINTS
                                        - PAGE_TEXT_MARGIN_TOP
                                        - PAGE_TEXT_MARGIN_BOTTOM)
                                        .max(1.0)),
                                    overflow: Overflow::visible(),
                                    ..default()
                                },
                                UiTransform::default(),
                                ZIndex(1),
                                RelativeCursorPosition::default(),
                                StoryQueryRenderedText { slot },
                            ))
                            .with_children(|text_parent| {
                                for line_offset in 0..span_capacity {
                                    for part_index in 0..PROCESSED_LINE_SPAN_PARTS {
                                        text_parent.spawn((
                                            TextSpan::new(""),
                                            TextFont {
                                                font: slot_font.clone().into(),
                                                font_size: FontSize::Px(FONT_SIZE),
                                                ..default()
                                            },
                                            LineHeight::Px(LINE_HEIGHT),
                                            TextColor(COLOR_ACTION),
                                            StoryQueryRenderedLineSpan {
                                                slot,
                                                line_offset,
                                                part_index,
                                            },
                                        ));
                                    }
                                }
                            });
                    });
            }
        });
    }
}

pub(crate) fn story_query_dropdown_row(
    font: Handle<Font>,
    slot: StoryQuerySheetTextSlot,
    kind: StoryQueryDropdownKind,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        StoryQueryControlRow { slot },
        children![
            story_query_control_button(
                font.clone(),
                slot,
                StoryQuerySheetAction::ToggleDropdown(kind),
            ),
            story_query_dropdown_options_bundle(font, kind),
        ],
    )
}

pub(crate) fn story_query_add_category_button_row(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        },
        StoryQueryControlRow {
            slot: StoryQuerySheetTextSlot::AddCategory,
        },
        children![story_query_control_button(
            font,
            StoryQuerySheetTextSlot::AddCategory,
            StoryQuerySheetAction::AddCategory,
        )],
    )
}

pub(crate) fn story_query_dropdown_options_bundle(
    font: Handle<Font>,
    kind: StoryQueryDropdownKind,
) -> impl Bundle {
    (
        Node {
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: px(2.0),
            padding: UiRect::all(px(3.0)),
            border: UiRect::all(px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.12)),
        BackgroundColor(Color::srgb(0.82, 0.84, 0.86)),
        StoryQueryDropdownOptionsRoot { kind },
        children![
            story_query_dropdown_option_button(font.clone(), kind, 0),
            story_query_dropdown_option_button(font.clone(), kind, 1),
            story_query_dropdown_option_button(font.clone(), kind, 2),
            story_query_dropdown_option_button(font.clone(), kind, 3),
            story_query_dropdown_option_button(font.clone(), kind, 4),
            story_query_dropdown_option_button(font.clone(), kind, 5),
            story_query_dropdown_option_button(font.clone(), kind, 6),
            story_query_dropdown_option_button(font, kind, 7),
        ],
    )
}

pub(crate) fn story_query_dropdown_option_button(
    font: Handle<Font>,
    kind: StoryQueryDropdownKind,
    slot_index: usize,
) -> impl Bundle {
    (
        Button,
        StoryQuerySheetAction::SelectDropdownOption { kind, slot_index },
        StoryQueryDropdownOptionNode { kind, slot_index },
        Node {
            display: Display::None,
            min_width: px(0.0),
            padding: UiRect::axes(px(7.0), px(5.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(""),
            TextLayout::no_wrap(),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(10.0),
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
            StoryQueryDropdownOptionText { kind, slot_index },
        )],
    )
}

pub(crate) fn story_query_control_button(
    font: Handle<Font>,
    slot: StoryQuerySheetTextSlot,
    action: StoryQuerySheetAction,
) -> impl Bundle {
    (
        Button,
        action,
        Node {
            flex_grow: 1.0,
            min_width: px(0.0),
            padding: UiRect::axes(px(8.0), px(6.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(""),
            TextLayout::no_wrap(),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
            slot,
        )],
    )
}

pub(crate) fn story_query_static_button(
    font: Handle<Font>,
    label: &str,
    action: StoryQuerySheetAction,
) -> impl Bundle {
    (
        Button,
        action,
        Node {
            padding: UiRect::axes(px(8.0), px(6.0)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(label),
            TextLayout::no_wrap(),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
        )],
    )
}

pub(crate) fn story_query_text(
    font: Handle<Font>,
    text: &str,
    font_size: f32,
    color: Color,
    slot: StoryQuerySheetTextSlot,
) -> impl Bundle {
    (
        Text::new(text),
        TextLayout::no_wrap(),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(font_size),
            ..default()
        },
        LineHeight::Px(font_size + 2.0),
        TextColor(color),
        slot,
    )
}

pub(crate) fn sync_story_query_sheet_ui(
    mut state: ResMut<EditorState>,
    fonts: Res<EditorFonts>,
    mut root_query: Query<
        &mut Node,
        (
            With<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionsRoot>,
            Without<StoryQueryDropdownOptionNode>,
            Without<StoryQueryPaper>,
            Without<StoryQueryRenderedText>,
        ),
    >,
    mut row_query: Query<
        (&StoryQueryControlRow, &mut Node),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryDropdownOptionsRoot>,
            Without<StoryQueryDropdownOptionNode>,
            Without<StoryQueryPaper>,
            Without<StoryQueryRenderedText>,
        ),
    >,
    mut dropdown_root_query: Query<
        (&StoryQueryDropdownOptionsRoot, &mut Node),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionNode>,
            Without<StoryQueryPaper>,
            Without<StoryQueryRenderedText>,
        ),
    >,
    mut option_node_query: Query<
        (
            &StoryQueryDropdownOptionNode,
            &mut Node,
            &mut BackgroundColor,
        ),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionsRoot>,
            Without<StoryQueryPaper>,
            Without<StoryQueryRenderedText>,
        ),
    >,
    mut text_query: Query<
        (&StoryQuerySheetTextSlot, &mut Text),
        Without<StoryQueryDropdownOptionText>,
    >,
    mut option_text_query: Query<
        (&StoryQueryDropdownOptionText, &mut Text),
        Without<StoryQuerySheetTextSlot>,
    >,
    result_panel_query: Query<&ComputedNode, With<StoryQueryResultPanel>>,
    mut story_paper_query: Query<
        (
            &StoryQueryPaper,
            &mut Node,
            &mut Visibility,
            &mut BackgroundColor,
            &mut UiTransform,
        ),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionsRoot>,
            Without<StoryQueryDropdownOptionNode>,
            Without<StoryQueryRenderedText>,
        ),
    >,
    mut rendered_text_node_query: Query<
        (&StoryQueryRenderedText, &mut Node, &mut UiTransform),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionsRoot>,
            Without<StoryQueryDropdownOptionNode>,
            Without<StoryQueryPaper>,
        ),
    >,
    mut rendered_span_query: Query<
        (
            &StoryQueryRenderedLineSpan,
            &mut TextSpan,
            &mut TextFont,
            &mut LineHeight,
            &mut TextColor,
        ),
        Without<ProcessedPaperLineSpan>,
    >,
) {
    let sheet_open = state.story_query_sheet.open;
    if let Ok(mut root) = root_query.single_mut() {
        root.display = if sheet_open {
            Display::Flex
        } else {
            Display::None
        };
    }

    if !sheet_open {
        return;
    }

    let result_panel_size = result_panel_query
        .single()
        .ok()
        .map(|computed| computed.size() * computed.inverse_scale_factor())
        .unwrap_or(Vec2::ZERO);
    let document_format = story_query_document_format(state.story_query_sheet.result_format);
    let result_layout =
        processed_page_layout_for_format(result_panel_size, &state, document_format, 0.0);
    let result_geometry = result_layout.geometry;
    let page_step_lines = result_layout.page_step_lines.max(1);
    let lines_per_page = result_layout.lines_per_page.max(1).min(page_step_lines);
    let processed_font_size = scaled_font_size(&state);
    let processed_line_height = scaled_line_height(&state).max(1.0);
    let processed_page_step_pixels = processed_page_step_px(&result_geometry, state.zoom);
    let (horizontal_min, horizontal_max) =
        story_query_horizontal_scroll_bounds(&result_layout, result_panel_size);
    let layout_signature = StoryQueryLayoutSignature {
        output_format: state.story_query_sheet.result_format,
        wrap_columns: result_layout.wrap_columns,
        lines_per_page: result_layout.lines_per_page,
        spacer_lines: result_layout.spacer_lines,
        dialogue_double_space_newline: state.story_query_sheet.dialogue_double_space_newline,
        non_dialogue_double_space_newline: state
            .story_query_sheet
            .non_dialogue_double_space_newline,
    };

    {
        let sheet = &mut state.story_query_sheet;
        sheet.ensure_visual_lines_for_layout(layout_signature);
        sheet.result_horizontal_scroll = sheet
            .result_horizontal_scroll
            .clamp(horizontal_min, horizontal_max);
        if sheet.visual_lines.is_empty() {
            sheet.result_scroll_visual = 0;
            sheet.result_scroll_anchor_bias_px = 0.0;
        } else {
            sheet.result_scroll_visual = sheet
                .result_scroll_visual
                .min(sheet.visual_lines.len().saturating_sub(1));
        }
    }

    let processed_anchor_offset_px = processed_anchor_scroll_offset_px_from_lines(
        &state,
        &state.story_query_sheet.visual_lines,
        state.story_query_sheet.result_scroll_visual,
        page_step_lines,
        processed_line_height,
    );
    let processed_view_capacity = page_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let processed_view = build_processed_view(
        &state.story_query_sheet.visual_lines,
        state.story_query_sheet.result_scroll_visual,
        page_step_lines,
        processed_view_capacity,
    );
    let first_visible_page = processed_view.start_index / page_step_lines;
    let total_pages =
        processed_page_count_for_lines(&state.story_query_sheet.visual_lines, page_step_lines);
    let page_label = story_query_page_label(
        &state.story_query_sheet,
        first_visible_page,
        page_step_lines,
    );

    for (paper, mut node, mut visibility, mut color, mut transform) in story_paper_query.iter_mut()
    {
        let page_index = first_visible_page.saturating_add(paper.slot);
        if page_index >= total_pages {
            *visibility = Visibility::Hidden;
            continue;
        }

        let page_top = processed_page_top_for_slot(
            &result_geometry,
            paper.slot,
            processed_page_step_pixels,
            processed_anchor_offset_px,
        ) + state.story_query_sheet.result_scroll_anchor_bias_px;
        let page_left =
            result_geometry.paper_left - state.story_query_sheet.result_horizontal_scroll;

        node.left = px(page_left);
        node.top = px(page_top);
        node.width = px(result_geometry.paper_width);
        node.height = px(result_geometry.paper_height);
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
        color.0 = COLOR_PAPER;
        *visibility = Visibility::Visible;
    }

    for (_, mut node, mut transform) in rendered_text_node_query.iter_mut() {
        node.left = px(result_geometry.text_left - result_geometry.paper_left);
        node.top = px(result_geometry.text_top - result_geometry.paper_top);
        node.width = px(result_geometry.text_width);
        node.height = px(result_geometry.text_height);
        node.overflow = Overflow::visible();
        transform.scale = Vec2::ONE;
        transform.translation = Val2::ZERO;
    }

    let sheet = &state.story_query_sheet;
    for (row, mut node) in row_query.iter_mut() {
        node.display = if story_query_control_visible(sheet, row.slot) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (root, mut node) in dropdown_root_query.iter_mut() {
        let slot = story_query_dropdown_slot(root.kind);
        node.display =
            if sheet.open_dropdown == Some(root.kind) && story_query_control_visible(sheet, slot) {
                Display::Flex
            } else {
                Display::None
            };
    }

    for (option, mut node, mut background) in option_node_query.iter_mut() {
        let choices = story_query_dropdown_choices(&state, option.kind);
        let start = story_query_dropdown_window_start(&state, option.kind, &choices);
        let choice = choices.get(start + option.slot_index);
        node.display = if sheet.open_dropdown == Some(option.kind) && choice.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        *background = if choice
            .map(|choice| choice.value == story_query_dropdown_current_value(sheet, option.kind))
            .unwrap_or(false)
        {
            BackgroundColor(BUTTON_PRESSED)
        } else {
            BackgroundColor(BUTTON_NORMAL)
        };
    }

    for (slot, mut text) in text_query.iter_mut() {
        **text = match slot {
            StoryQuerySheetTextSlot::Title => sheet.result_title.clone(),
            StoryQuerySheetTextSlot::Status => sheet.result_status.clone(),
            StoryQuerySheetTextSlot::QueryKind => format!("Query: {}", sheet.query_kind.label()),
            StoryQuerySheetTextSlot::SceneScope => {
                format!("Scope: {}", sheet.scene_scope.label())
            }
            StoryQuerySheetTextSlot::Category(slot_index) => {
                let label = sheet
                    .category_indices
                    .get(*slot_index)
                    .map(|category_index| selected_category_label(sheet, *category_index))
                    .unwrap_or_else(|| "none".to_string());
                format!(
                    "Category {}: {}",
                    slot_index + 1,
                    compact_story_query_label(&label)
                )
            }
            StoryQuerySheetTextSlot::AddCategory => "Add category".to_string(),
            StoryQuerySheetTextSlot::PrimaryCharacter => {
                format!(
                    "Character A: {}",
                    compact_story_query_label(&selected_character_label(
                        sheet,
                        sheet.character_a_index
                    ))
                )
            }
            StoryQuerySheetTextSlot::SecondaryCharacter => {
                format!(
                    "Character B: {}",
                    compact_story_query_label(&selected_character_label(
                        sheet,
                        sheet.character_b_index
                    ))
                )
            }
            StoryQuerySheetTextSlot::Entity => {
                format!(
                    "Entity: {}",
                    compact_story_query_label(&selected_entity_label(sheet, sheet.entity_index))
                )
            }
            StoryQuerySheetTextSlot::Scene => {
                format!(
                    "Scene: {}",
                    compact_story_query_label(&selected_scene_label(&state))
                )
            }
            StoryQuerySheetTextSlot::Page => page_label.clone(),
        };
    }

    for (option, mut text) in option_text_query.iter_mut() {
        let choices = story_query_dropdown_choices(&state, option.kind);
        let start = story_query_dropdown_window_start(&state, option.kind, &choices);
        **text = choices
            .get(start + option.slot_index)
            .map(|choice| {
                let label = compact_story_query_label(&choice.label);
                if choice.value == story_query_dropdown_current_value(sheet, option.kind) {
                    format!("* {label}")
                } else {
                    format!("  {label}")
                }
            })
            .unwrap_or_default();
    }

    apply_story_query_rendered_page_styles(
        &mut rendered_span_query,
        sheet,
        &state,
        &fonts,
        document_format,
        first_visible_page,
        page_step_lines,
        lines_per_page,
        processed_font_size,
        processed_line_height,
    );
}

pub(crate) fn apply_story_query_rendered_page_styles(
    rendered_span_query: &mut Query<
        (
            &StoryQueryRenderedLineSpan,
            &mut TextSpan,
            &mut TextFont,
            &mut LineHeight,
            &mut TextColor,
        ),
        Without<ProcessedPaperLineSpan>,
    >,
    sheet: &StoryQuerySheet,
    state: &EditorState,
    fonts: &EditorFonts,
    document_format: DocumentFormat,
    first_visible_page: usize,
    page_step_lines: usize,
    lines_per_page: usize,
    font_size: f32,
    line_height: f32,
) {
    let page_step_lines = page_step_lines.max(1);
    let lines_per_page = lines_per_page.max(1).min(page_step_lines);

    for (span, mut text_span, mut text_font, mut text_line_height, mut text_color) in
        rendered_span_query.iter_mut()
    {
        let page_index = first_visible_page.saturating_add(span.slot);
        let line_offset = span.line_offset.min(page_step_lines.saturating_sub(1));
        let page_start = page_index.saturating_mul(page_step_lines);
        let global_index = page_start.saturating_add(line_offset);

        if line_offset >= lines_per_page {
            **text_span = String::new();
            apply_font_variant_to_text_font(
                &mut text_font,
                fonts,
                FontVariant::Regular,
                document_format,
            );
            text_font.font_size = FontSize::Px(font_size);
            *text_line_height = LineHeight::Px(line_height);
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        };

        let Some(visual_line) = sheet.visual_lines.get(global_index) else {
            **text_span = if span.part_index == 0 && line_offset + 1 < lines_per_page {
                "\n".to_owned()
            } else {
                String::new()
            };
            apply_font_variant_to_text_font(
                &mut text_font,
                fonts,
                FontVariant::Regular,
                document_format,
            );
            text_font.font_size = FontSize::Px(font_size);
            *text_line_height = LineHeight::Px(line_height);
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        };

        let (style, allow_link_color) = if visual_line.is_spacer {
            (transparent_line_render_style(), false)
        } else if let Some(render_override) = visual_line.render_override.as_ref() {
            (
                processed_line_style_for_kind(
                    &render_override.kind,
                    render_override.markdown_heading_level,
                ),
                true,
            )
        } else {
            (default_line_render_style(), true)
        };

        let used_fragment_count = processed_visual_fragment_count(visual_line);
        let Some(mut fragment) = processed_visual_fragment_for_part(visual_line, span.part_index)
        else {
            **text_span = String::new();
            apply_font_variant_to_text_font(
                &mut text_font,
                fonts,
                FontVariant::Regular,
                document_format,
            );
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        };

        if span.part_index + 1 == used_fragment_count && line_offset + 1 < lines_per_page {
            fragment.text.push('\n');
        }

        let effective_variant = font_variant_for_processed_fragment(style.font_variant, &fragment);
        apply_font_variant_to_text_font(&mut text_font, fonts, effective_variant, document_format);
        text_font.font_size = FontSize::Px(font_size * style.font_scale);
        *text_line_height = LineHeight::Px(line_height * style.line_height_scale);
        **text_span = fragment.text;
        text_color.0 = if allow_link_color && fragment.is_link {
            story_query_link_color_for_target(sheet, state, fragment.link_target.as_deref())
        } else {
            style.color
        };
    }
}

pub(crate) fn handle_story_query_sheet_buttons(
    interaction_query: Query<
        (&Interaction, &StoryQuerySheetAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            StoryQuerySheetAction::ToggleDropdown(kind) => {
                let slot = story_query_dropdown_slot(*kind);
                if story_query_control_visible(&state.story_query_sheet, slot) {
                    state.story_query_sheet.open_dropdown =
                        if state.story_query_sheet.open_dropdown == Some(*kind) {
                            None
                        } else {
                            Some(*kind)
                        };
                }
            }
            StoryQuerySheetAction::SelectDropdownOption { kind, slot_index } => {
                let choices = story_query_dropdown_choices(&state, *kind);
                let start = story_query_dropdown_window_start(&state, *kind, &choices);
                if let Some(choice) = choices.get(start + *slot_index) {
                    apply_story_query_dropdown_choice(
                        &mut state.story_query_sheet,
                        *kind,
                        choice.value,
                    );
                    state.story_query_sheet.open_dropdown = None;
                    state.story_query_sheet.reset_result_scroll();
                    state.story_query_sheet.result_status = "Selection changed.".to_string();
                }
            }
            StoryQuerySheetAction::Run => state.run_story_query_sheet(),
            StoryQuerySheetAction::OpenFirstSource => state.open_first_story_query_source(),
            StoryQuerySheetAction::AddCategory => {
                add_story_query_category(&mut state.story_query_sheet);
                state.story_query_sheet.open_dropdown = None;
                state.story_query_sheet.reset_result_scroll();
                state.story_query_sheet.result_status = "Category field added.".to_string();
            }
            StoryQuerySheetAction::Close => state.story_query_sheet.open = false,
        }
    }
}

pub(crate) fn handle_story_query_sheet_link_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    rendered_text_query: Query<
        (
            &StoryQueryRenderedText,
            &RelativeCursorPosition,
            &ComputedNode,
            &ComputedTextBlock,
        ),
        With<StoryQueryRenderedText>,
    >,
    result_panel_query: Query<&ComputedNode, With<StoryQueryResultPanel>>,
    mut state: ResMut<EditorState>,
) {
    if !state.story_query_sheet.open || !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let result_panel_size = result_panel_query
        .single()
        .ok()
        .map(|computed| computed.size() * computed.inverse_scale_factor())
        .unwrap_or(Vec2::ZERO);
    let document_format = story_query_document_format(state.story_query_sheet.result_format);
    let result_layout =
        processed_page_layout_for_format(result_panel_size, &state, document_format, 0.0);
    let page_step_lines = result_layout.page_step_lines.max(1);
    let lines_per_page = result_layout.lines_per_page.max(1).min(page_step_lines);
    let processed_view_capacity = page_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let processed_view = build_processed_view(
        &state.story_query_sheet.visual_lines,
        state.story_query_sheet.result_scroll_visual,
        page_step_lines,
        processed_view_capacity,
    );
    let first_visible_page = processed_view.start_index / page_step_lines;

    let target = rendered_text_query.iter().find_map(
        |(rendered_text, relative_cursor, computed, text_block)| {
            if !relative_cursor.cursor_over() {
                return None;
            }
            let normalized = relative_cursor.normalized?;
            let size = computed.size() * computed.inverse_scale_factor();
            let local_x = (normalized.x + 0.5) * size.x;
            let local_y = (normalized.y + 0.5) * size.y;
            story_query_link_target_at_position(
                &state.story_query_sheet,
                text_block,
                computed.inverse_scale_factor(),
                local_x,
                local_y,
                rendered_text.slot,
                first_visible_page,
                page_step_lines,
                lines_per_page,
                document_format,
                state.zoom,
            )
        },
    );

    if let Some(target) = target {
        state.open_story_query_link_target(target);
    }
}

pub(crate) fn handle_story_query_sheet_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
) {
    if !state.story_query_sheet.open {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        if state.story_query_sheet.open_dropdown.is_some() {
            state.story_query_sheet.open_dropdown = None;
        } else {
            state.story_query_sheet.open = false;
            state.status_message = "Closed story query sheet.".to_string();
        }
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        step_open_story_query_dropdown(&mut state, -1);
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        step_open_story_query_dropdown(&mut state, 1);
    } else if keys.just_pressed(KeyCode::Enter) {
        if state.story_query_sheet.open_dropdown.is_some() {
            state.story_query_sheet.open_dropdown = None;
        } else {
            state.run_story_query_sheet();
        }
    }
}

pub(crate) fn handle_story_query_sheet_mouse_scroll(
    mut mouse_wheels: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    result_panel_query: Query<
        (&RelativeCursorPosition, &ComputedNode),
        With<StoryQueryResultPanel>,
    >,
    mut state: ResMut<EditorState>,
) {
    if !state.story_query_sheet.open {
        return;
    }

    let result_panel = result_panel_query.single().ok();
    let Some((relative_cursor, computed)) = result_panel else {
        for _ in mouse_wheels.read() {}
        return;
    };

    if !relative_cursor.cursor_over() {
        for _ in mouse_wheels.read() {}
        return;
    }

    let result_panel_size = computed.size() * computed.inverse_scale_factor();
    let shift_horizontal = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if platform_shortcut_modifier_pressed(&keys) {
        let mut zoom_steps = 0.0_f32;
        for wheel in mouse_wheels.read() {
            let y = match wheel.unit {
                MouseScrollUnit::Line => wheel.y,
                MouseScrollUnit::Pixel => wheel.y / 120.0,
            };
            zoom_steps += y;
        }

        if zoom_steps.abs() > f32::EPSILON {
            let next_zoom = state.zoom + zoom_steps * ZOOM_STEP;
            state.set_zoom(next_zoom);
            state.status_message = format!("Zoom: {}%", state.zoom_percent());
        }
        return;
    }

    let mut vertical_delta_lines = 0.0_f32;
    let mut horizontal_delta_px = 0.0_f32;

    for wheel in mouse_wheels.read() {
        let mut dx = wheel.x;
        let mut dy = wheel.y;
        if shift_horizontal && dx.abs() <= f32::EPSILON {
            dx = -dy;
            dy = 0.0;
        }

        match wheel.unit {
            MouseScrollUnit::Line => {
                vertical_delta_lines += -dy;
                horizontal_delta_px += -dx * 32.0;
            }
            MouseScrollUnit::Pixel => {
                vertical_delta_lines += -dy / scaled_line_height(&state).max(1.0);
                horizontal_delta_px += -dx;
            }
        }
    }

    let mut scrolled = false;
    if horizontal_delta_px.abs() > f32::EPSILON {
        scrolled |=
            apply_story_query_horizontal_scroll(&mut state, result_panel_size, horizontal_delta_px);
    }
    if vertical_delta_lines.abs() > f32::EPSILON {
        scrolled |=
            apply_story_query_vertical_scroll(&mut state, result_panel_size, vertical_delta_lines);
    }

    if scrolled {
        state.reset_blink();
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StoryQueryDropdownChoice {
    pub(crate) value: usize,
    pub(crate) label: String,
}

pub(crate) fn story_query_dropdown_slot(kind: StoryQueryDropdownKind) -> StoryQuerySheetTextSlot {
    match kind {
        StoryQueryDropdownKind::QueryKind => StoryQuerySheetTextSlot::QueryKind,
        StoryQueryDropdownKind::SceneScope => StoryQuerySheetTextSlot::SceneScope,
        StoryQueryDropdownKind::PrimaryCharacter => StoryQuerySheetTextSlot::PrimaryCharacter,
        StoryQueryDropdownKind::SecondaryCharacter => StoryQuerySheetTextSlot::SecondaryCharacter,
        StoryQueryDropdownKind::Entity => StoryQuerySheetTextSlot::Entity,
        StoryQueryDropdownKind::Scene => StoryQuerySheetTextSlot::Scene,
        StoryQueryDropdownKind::Category(slot_index) => {
            StoryQuerySheetTextSlot::Category(slot_index)
        }
    }
}

pub(crate) fn story_query_control_visible(
    sheet: &StoryQuerySheet,
    slot: StoryQuerySheetTextSlot,
) -> bool {
    match slot {
        StoryQuerySheetTextSlot::Title
        | StoryQuerySheetTextSlot::Status
        | StoryQuerySheetTextSlot::Page => true,
        StoryQuerySheetTextSlot::QueryKind => true,
        StoryQuerySheetTextSlot::SceneScope => matches!(
            sheet.query_kind,
            StoryQueryKind::DialogueBetweenAtScene | StoryQueryKind::CategoriesInScene
        ),
        StoryQuerySheetTextSlot::Scene => {
            story_query_control_visible(sheet, StoryQuerySheetTextSlot::SceneScope)
                && sheet.scene_scope == StoryQuerySceneScope::Selected
        }
        StoryQuerySheetTextSlot::PrimaryCharacter => matches!(
            sheet.query_kind,
            StoryQueryKind::DialogueByCharacter
                | StoryQueryKind::DialogueBetweenCharacters
                | StoryQueryKind::DialogueBetweenAtScene
        ),
        StoryQuerySheetTextSlot::SecondaryCharacter => matches!(
            sheet.query_kind,
            StoryQueryKind::DialogueBetweenCharacters | StoryQueryKind::DialogueBetweenAtScene
        ),
        StoryQuerySheetTextSlot::Entity => sheet.query_kind == StoryQueryKind::AppearancesOfEntity,
        StoryQuerySheetTextSlot::Category(slot_index) => {
            sheet.query_kind == StoryQueryKind::CategoriesInScene
                && slot_index < sheet.category_indices.len()
        }
        StoryQuerySheetTextSlot::AddCategory => story_query_can_add_category(sheet),
    }
}

pub(crate) fn story_query_dropdown_choices(
    state: &EditorState,
    kind: StoryQueryDropdownKind,
) -> Vec<StoryQueryDropdownChoice> {
    let sheet = &state.story_query_sheet;
    match kind {
        StoryQueryDropdownKind::QueryKind => STORY_QUERY_KINDS
            .iter()
            .enumerate()
            .map(|(index, kind)| StoryQueryDropdownChoice {
                value: index,
                label: kind.label().to_string(),
            })
            .collect(),
        StoryQueryDropdownKind::SceneScope => STORY_QUERY_SCENE_SCOPES
            .iter()
            .enumerate()
            .map(|(index, scope)| StoryQueryDropdownChoice {
                value: index,
                label: scope.label().to_string(),
            })
            .collect(),
        StoryQueryDropdownKind::PrimaryCharacter => sheet
            .characters
            .iter()
            .enumerate()
            .map(|(index, entity)| StoryQueryDropdownChoice {
                value: index,
                label: entity_label(entity),
            })
            .collect(),
        StoryQueryDropdownKind::SecondaryCharacter => sheet
            .characters
            .iter()
            .enumerate()
            .map(|(index, entity)| StoryQueryDropdownChoice {
                value: index,
                label: entity_label(entity),
            })
            .collect(),
        StoryQueryDropdownKind::Entity => sheet
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| StoryQueryDropdownChoice {
                value: index,
                label: entity_label(entity),
            })
            .collect(),
        StoryQueryDropdownKind::Scene => sheet
            .scenes
            .iter()
            .enumerate()
            .map(|(index, scene)| StoryQueryDropdownChoice {
                value: index + 1,
                label: scene_label(scene),
            })
            .collect(),
        StoryQueryDropdownKind::Category(slot_index) => sheet
            .taxonomy
            .categories
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                sheet
                    .category_indices
                    .iter()
                    .enumerate()
                    .all(|(other_slot, selected)| other_slot == slot_index || selected != index)
            })
            .map(|(index, category)| StoryQueryDropdownChoice {
                value: index,
                label: category.display_label().to_string(),
            })
            .collect(),
    }
}

pub(crate) fn story_query_dropdown_current_value(
    sheet: &StoryQuerySheet,
    kind: StoryQueryDropdownKind,
) -> usize {
    match kind {
        StoryQueryDropdownKind::QueryKind => sheet.query_kind.index(),
        StoryQueryDropdownKind::SceneScope => sheet.scene_scope.index(),
        StoryQueryDropdownKind::PrimaryCharacter => sheet.character_a_index,
        StoryQueryDropdownKind::SecondaryCharacter => sheet.character_b_index,
        StoryQueryDropdownKind::Entity => sheet.entity_index,
        StoryQueryDropdownKind::Scene => sheet.scene_index,
        StoryQueryDropdownKind::Category(slot_index) => {
            sheet.category_indices.get(slot_index).copied().unwrap_or(0)
        }
    }
}

pub(crate) fn story_query_dropdown_window_start(
    state: &EditorState,
    kind: StoryQueryDropdownKind,
    choices: &[StoryQueryDropdownChoice],
) -> usize {
    if choices.len() <= STORY_QUERY_DROPDOWN_VISIBLE_OPTIONS {
        return 0;
    }

    let current = story_query_dropdown_current_value(&state.story_query_sheet, kind);
    let selected = choices
        .iter()
        .position(|choice| choice.value == current)
        .unwrap_or(0);
    selected.saturating_sub(3).min(
        choices
            .len()
            .saturating_sub(STORY_QUERY_DROPDOWN_VISIBLE_OPTIONS),
    )
}

pub(crate) fn apply_story_query_dropdown_choice(
    sheet: &mut StoryQuerySheet,
    kind: StoryQueryDropdownKind,
    value: usize,
) {
    match kind {
        StoryQueryDropdownKind::QueryKind => {
            if let Some(query_kind) = STORY_QUERY_KINDS.get(value).copied() {
                sheet.query_kind = query_kind;
            }
        }
        StoryQueryDropdownKind::SceneScope => {
            if let Some(scene_scope) = STORY_QUERY_SCENE_SCOPES.get(value).copied() {
                sheet.scene_scope = scene_scope;
            }
        }
        StoryQueryDropdownKind::PrimaryCharacter => sheet.character_a_index = value,
        StoryQueryDropdownKind::SecondaryCharacter => sheet.character_b_index = value,
        StoryQueryDropdownKind::Entity => sheet.entity_index = value,
        StoryQueryDropdownKind::Scene => sheet.scene_index = value,
        StoryQueryDropdownKind::Category(slot_index) => {
            if let Some(category_index) = sheet.category_indices.get_mut(slot_index) {
                *category_index = value;
            }
        }
    }
    clamp_story_query_dependencies(sheet);
}

pub(crate) fn step_open_story_query_dropdown(state: &mut EditorState, direction: isize) {
    let Some(kind) = state.story_query_sheet.open_dropdown else {
        return;
    };
    let choices = story_query_dropdown_choices(state, kind);
    if choices.is_empty() {
        return;
    }
    let current = story_query_dropdown_current_value(&state.story_query_sheet, kind);
    let current_index = choices
        .iter()
        .position(|choice| choice.value == current)
        .unwrap_or(0);
    let next_index = (current_index as isize + direction).rem_euclid(choices.len() as isize);
    apply_story_query_dropdown_choice(
        &mut state.story_query_sheet,
        kind,
        choices[next_index as usize].value,
    );
    state.story_query_sheet.reset_result_scroll();
    state.story_query_sheet.result_status = "Selection changed.".to_string();
}

pub(crate) fn story_query_horizontal_scroll_bounds(
    layout: &ProcessedPageLayout,
    panel_size: Vec2,
) -> (f32, f32) {
    if panel_size.x <= 1.0 {
        return (0.0, 0.0);
    }

    let base_left = layout.geometry.paper_left;
    let base_right = layout.geometry.paper_left + layout.geometry.paper_width;
    let overflow_left = (-base_left).max(0.0);
    let overflow_right = (base_right - panel_size.x).max(0.0);
    let overscroll = (panel_size.x * PROCESSED_HORIZONTAL_OVERSCROLL_FACTOR)
        .max(PROCESSED_HORIZONTAL_OVERSCROLL_MIN);
    (-(overflow_left + overscroll), overflow_right + overscroll)
}

pub(crate) fn apply_story_query_horizontal_scroll(
    state: &mut EditorState,
    result_panel_size: Vec2,
    horizontal_delta_px: f32,
) -> bool {
    if horizontal_delta_px.abs() <= f32::EPSILON {
        return false;
    }

    let document_format = story_query_document_format(state.story_query_sheet.result_format);
    let layout = processed_page_layout_for_format(result_panel_size, state, document_format, 0.0);
    let (min_scroll, max_scroll) = story_query_horizontal_scroll_bounds(&layout, result_panel_size);
    let next_scroll = (state.story_query_sheet.result_horizontal_scroll + horizontal_delta_px)
        .clamp(min_scroll, max_scroll);
    let changed =
        (next_scroll - state.story_query_sheet.result_horizontal_scroll).abs() > f32::EPSILON;
    state.story_query_sheet.result_horizontal_scroll = next_scroll;
    changed
}

pub(crate) fn apply_story_query_vertical_scroll(
    state: &mut EditorState,
    result_panel_size: Vec2,
    delta_lines: f32,
) -> bool {
    if delta_lines.abs() <= f32::EPSILON {
        return false;
    }

    let document_format = story_query_document_format(state.story_query_sheet.result_format);
    let layout = processed_page_layout_for_format(result_panel_size, state, document_format, 0.0);
    let layout_signature = StoryQueryLayoutSignature {
        output_format: state.story_query_sheet.result_format,
        wrap_columns: layout.wrap_columns,
        lines_per_page: layout.lines_per_page,
        spacer_lines: layout.spacer_lines,
        dialogue_double_space_newline: state.story_query_sheet.dialogue_double_space_newline,
        non_dialogue_double_space_newline: state
            .story_query_sheet
            .non_dialogue_double_space_newline,
    };
    state
        .story_query_sheet
        .ensure_visual_lines_for_layout(layout_signature);

    if state.story_query_sheet.visual_lines.is_empty() {
        state.story_query_sheet.result_scroll_visual = 0;
        state.story_query_sheet.result_scroll_anchor_bias_px = 0.0;
        return false;
    }

    let line_height = scaled_line_height(state).max(1.0);
    let requested_whole_lines = delta_lines.trunc() as isize;
    let max_visual = state.story_query_sheet.visual_lines.len().saturating_sub(1) as isize;
    let current_visual = state
        .story_query_sheet
        .result_scroll_visual
        .min(max_visual as usize) as isize;
    let next_visual = (current_visual + requested_whole_lines).clamp(0, max_visual);
    let actual_whole_lines = next_visual - current_visual;
    state.story_query_sheet.result_scroll_visual = next_visual as usize;

    let leftover_px = (delta_lines - actual_whole_lines as f32) * line_height;
    state.story_query_sheet.result_scroll_anchor_bias_px -= leftover_px;

    while state.story_query_sheet.result_scroll_anchor_bias_px <= -line_height
        && state.story_query_sheet.result_scroll_visual
            < state.story_query_sheet.visual_lines.len().saturating_sub(1)
    {
        state.story_query_sheet.result_scroll_anchor_bias_px += line_height;
        state.story_query_sheet.result_scroll_visual = state
            .story_query_sheet
            .result_scroll_visual
            .saturating_add(1);
    }
    while state.story_query_sheet.result_scroll_anchor_bias_px >= line_height
        && state.story_query_sheet.result_scroll_visual > 0
    {
        state.story_query_sheet.result_scroll_anchor_bias_px -= line_height;
        state.story_query_sheet.result_scroll_visual = state
            .story_query_sheet
            .result_scroll_visual
            .saturating_sub(1);
    }

    state.story_query_sheet.result_scroll_anchor_bias_px = state
        .story_query_sheet
        .result_scroll_anchor_bias_px
        .clamp(-line_height, line_height);

    actual_whole_lines != 0 || leftover_px.abs() > f32::EPSILON
}

pub(crate) fn story_query_page_label(
    sheet: &StoryQuerySheet,
    first_visible_page: usize,
    page_step_lines: usize,
) -> String {
    let format = match sheet.result_format {
        StoryQueryOutputFormat::Fountain => "Fountain",
        StoryQueryOutputFormat::Markdown => "Markdown",
    };
    let page_step_lines = page_step_lines.max(1);
    let total_pages = sheet
        .visual_lines
        .len()
        .saturating_add(page_step_lines.saturating_sub(1))
        / page_step_lines;
    let total_pages = total_pages.max(1);
    let first_page = first_visible_page.saturating_add(1).min(total_pages);
    let last_page = first_visible_page
        .saturating_add(PROCESSED_PAPER_CAPACITY)
        .min(total_pages);

    if first_page == last_page {
        format!("{format} page {first_page} of {total_pages}")
    } else {
        format!("{format} pages {first_page}-{last_page} of {total_pages}")
    }
}

impl StoryQuerySheet {
    pub(crate) fn reset_result_scroll(&mut self) {
        self.result_scroll_visual = 0;
        self.result_scroll_anchor_bias_px = 0.0;
        self.result_horizontal_scroll = 0.0;
    }

    pub(crate) fn ensure_visual_lines_for_layout(&mut self, signature: StoryQueryLayoutSignature) {
        if self.visual_layout_signature == Some(signature) && !self.visual_lines.is_empty() {
            return;
        }

        self.visual_lines = story_query_visual_lines(
            &self.result_text,
            self.result_format,
            self.dialogue_double_space_newline,
            self.non_dialogue_double_space_newline,
            signature.wrap_columns,
            signature.lines_per_page,
            signature.spacer_lines,
        );
        self.visual_layout_signature = Some(signature);
        self.result_scroll_visual = self
            .result_scroll_visual
            .min(self.visual_lines.len().saturating_sub(1));
    }

    pub(crate) fn set_output(
        &mut self,
        output: StoryQueryRunOutput,
        dialogue_double_space_newline: bool,
        non_dialogue_double_space_newline: bool,
    ) {
        self.result_title = output.title;
        self.result_status = output.status;
        self.result_format = output.format;
        self.result_text = output.text;
        self.dialogue_double_space_newline = dialogue_double_space_newline;
        self.non_dialogue_double_space_newline = non_dialogue_double_space_newline;
        self.visual_layout_signature = None;
        self.visual_lines.clear();
        self.source_targets = output.source_targets;
        self.reset_result_scroll();
    }

    pub(crate) fn set_error(&mut self, title: &str, message: String) {
        self.set_output(
            StoryQueryRunOutput {
                title: title.to_string(),
                status: message.clone(),
                format: StoryQueryOutputFormat::Markdown,
                text: format!("# {title}\n\n{message}"),
                source_targets: Vec::new(),
            },
            false,
            false,
        );
    }
}

impl EditorState {
    pub(crate) fn open_story_query_sheet(&mut self) {
        self.close_link_autocomplete();
        self.story_query_sheet.open = true;
        self.command_menu = None;
        self.workspace_focused = false;
        match self.refresh_story_query_sheet_options() {
            Ok(()) => {
                let entity_error_count = self
                    .story_index
                    .as_ref()
                    .map(|index| index.entity_error_count)
                    .unwrap_or(0);
                let taxonomy_status = self
                    .story_query_sheet
                    .taxonomy_notice
                    .clone()
                    .unwrap_or_else(|| {
                        format!(
                            "{} taxonomy categories loaded.",
                            self.story_query_sheet.taxonomy.categories.len()
                        )
                    });
                self.story_query_sheet.result_status = format!(
                    "{} characters, {} entities, {} entity errors, {} scenes indexed. {}",
                    self.story_query_sheet.characters.len(),
                    self.story_query_sheet.entities.len(),
                    entity_error_count,
                    self.story_query_sheet.scenes.len(),
                    taxonomy_status
                );
                self.status_message = "Opened story query sheet.".to_string();
            }
            Err(error) => {
                self.story_query_sheet
                    .set_error("Story Query Sheet", error.clone());
                self.status_message = error;
            }
        }
    }

    pub(crate) fn refresh_story_query_sheet_options(&mut self) -> Result<(), String> {
        let Some(workspace_root) = self.workspace_root.clone() else {
            return Err("Open a workspace before using the story query sheet.".to_string());
        };

        let taxonomy_load = load_story_taxonomy();
        self.story_query_sheet.taxonomy = taxonomy_load.taxonomy;
        self.story_query_sheet.taxonomy_notice = taxonomy_load.notice;

        if let Some(message) = self.refresh_story_index_for_workspace() {
            self.status_message = message;
        }

        let database = basscript_core::StoryIndexDatabase::open_workspace(&workspace_root)
            .map_err(|error| format!("Story index unavailable: {error}"))?
            .database;
        let mut characters = database
            .entities_of_type("character")
            .map_err(|error| format!("Character query failed: {error}"))?;
        let fallback_characters = database
            .character_appearance_entities()
            .map_err(|error| format!("Script character query failed: {error}"))?;
        merge_story_query_entities(&mut characters, fallback_characters.clone());
        self.story_query_sheet.characters = characters;

        let mut entities = database
            .all_entities()
            .map_err(|error| format!("Entity query failed: {error}"))?;
        merge_story_query_entities(&mut entities, fallback_characters);
        self.story_query_sheet.entities = entities;
        self.story_query_sheet.scenes = database
            .all_scenes()
            .map_err(|error| format!("Scene query failed: {error}"))?;
        self.clamp_story_query_sheet_selection();
        Ok(())
    }

    pub(crate) fn clamp_story_query_sheet_selection(&mut self) {
        clamp_story_query_index(
            &mut self.story_query_sheet.character_a_index,
            self.story_query_sheet.characters.len(),
        );
        clamp_story_query_index(
            &mut self.story_query_sheet.character_b_index,
            self.story_query_sheet.characters.len(),
        );
        if self.story_query_sheet.characters.len() > 1
            && self.story_query_sheet.character_a_index == self.story_query_sheet.character_b_index
        {
            self.story_query_sheet.character_b_index = (self.story_query_sheet.character_a_index
                + 1)
                % self.story_query_sheet.characters.len();
        }
        clamp_story_query_index(
            &mut self.story_query_sheet.entity_index,
            self.story_query_sheet.entities.len(),
        );
        clamp_story_query_dependencies(&mut self.story_query_sheet);
    }

    pub(crate) fn run_story_query_sheet(&mut self) {
        if let Err(error) = self.refresh_story_query_sheet_options() {
            self.story_query_sheet.set_error("Story Query Sheet", error);
            return;
        }

        let Some(workspace_root) = self.workspace_root.clone() else {
            self.story_query_sheet.set_error(
                "Story Query Sheet",
                "Open a workspace before running a story query.".to_string(),
            );
            return;
        };
        let database = match basscript_core::StoryIndexDatabase::open_workspace(&workspace_root) {
            Ok(report) => report.database,
            Err(error) => {
                self.story_query_sheet.set_error(
                    "Story Query Sheet",
                    format!("Story index unavailable: {error}"),
                );
                return;
            }
        };

        let output = match self.story_query_sheet.query_kind {
            StoryQueryKind::DialogueByCharacter => {
                let Some(character) = selected_character(
                    &self.story_query_sheet,
                    self.story_query_sheet.character_a_index,
                ) else {
                    self.story_query_sheet
                        .set_error("Dialogue", "No character entities are indexed.".to_string());
                    return;
                };
                build_dialogue_by_character_output(&database, character)
            }
            StoryQueryKind::DialogueBetweenCharacters => {
                let Some(character_a) = selected_character(
                    &self.story_query_sheet,
                    self.story_query_sheet.character_a_index,
                ) else {
                    self.story_query_sheet
                        .set_error("Dialogue", "No character entities are indexed.".to_string());
                    return;
                };
                let Some(character_b) = selected_character(
                    &self.story_query_sheet,
                    self.story_query_sheet.character_b_index,
                ) else {
                    self.story_query_sheet
                        .set_error("Dialogue", "Choose a second indexed character.".to_string());
                    return;
                };
                build_dialogue_between_output(&database, character_a, character_b, None)
            }
            StoryQueryKind::DialogueBetweenAtScene => {
                let Some(character_a) = selected_character(
                    &self.story_query_sheet,
                    self.story_query_sheet.character_a_index,
                ) else {
                    self.story_query_sheet
                        .set_error("Dialogue", "No character entities are indexed.".to_string());
                    return;
                };
                let Some(character_b) = selected_character(
                    &self.story_query_sheet,
                    self.story_query_sheet.character_b_index,
                ) else {
                    self.story_query_sheet
                        .set_error("Dialogue", "Choose a second indexed character.".to_string());
                    return;
                };
                let scene_filter = match self.selected_story_query_scene(&database) {
                    Ok(scene) => scene,
                    Err(error) => {
                        self.story_query_sheet.set_error("Dialogue", error);
                        return;
                    }
                };
                if self.story_query_sheet.scene_scope != StoryQuerySceneScope::All
                    && scene_filter.is_none()
                {
                    self.story_query_sheet.set_error(
                        "Dialogue",
                        "No current or selected scene is available.".to_string(),
                    );
                    return;
                }
                build_dialogue_between_output(&database, character_a, character_b, scene_filter)
            }
            StoryQueryKind::CategoriesInScene => {
                let selected_category_indices = selected_category_indices(&self.story_query_sheet);
                if selected_category_indices.is_empty() {
                    self.story_query_sheet.set_error(
                        "Categories",
                        "No taxonomy categories are available.".to_string(),
                    );
                    return;
                }
                match self.story_query_sheet.scene_scope {
                    StoryQuerySceneScope::All => {
                        let scenes = match database.all_scenes() {
                            Ok(scenes) => scenes,
                            Err(error) => {
                                self.story_query_sheet.set_error(
                                    "Entities by category",
                                    format!("Scene query failed: {error}"),
                                );
                                return;
                            }
                        };
                        build_category_scenes_output(
                            &database,
                            &self.story_query_sheet.taxonomy,
                            &selected_category_indices,
                            &scenes,
                            "All scenes",
                            true,
                        )
                    }
                    StoryQuerySceneScope::Current | StoryQuerySceneScope::Selected => {
                        let scene = match self.selected_story_query_scene(&database) {
                            Ok(Some(scene)) => scene,
                            Ok(None) => {
                                self.story_query_sheet.set_error(
                                    "Entities by category",
                                    "No current or selected scene is available.".to_string(),
                                );
                                return;
                            }
                            Err(error) => {
                                self.story_query_sheet
                                    .set_error("Entities by category", error);
                                return;
                            }
                        };
                        let scope_label = if self.story_query_sheet.scene_scope
                            == StoryQuerySceneScope::Current
                        {
                            "Current scene".to_string()
                        } else {
                            scene.heading_text.clone()
                        };
                        let scenes = vec![scene];
                        build_category_scenes_output(
                            &database,
                            &self.story_query_sheet.taxonomy,
                            &selected_category_indices,
                            &scenes,
                            &scope_label,
                            false,
                        )
                    }
                }
            }
            StoryQueryKind::AppearancesOfEntity => {
                let Some(entity) =
                    selected_entity(&self.story_query_sheet, self.story_query_sheet.entity_index)
                else {
                    self.story_query_sheet
                        .set_error("Appearances", "No entities are indexed.".to_string());
                    return;
                };
                build_appearances_output(&database, entity)
            }
        };

        match output {
            Ok(output) => {
                self.status_message = output.status.clone();
                self.story_query_sheet.set_output(
                    output,
                    self.dialogue_double_space_newline,
                    self.non_dialogue_double_space_newline,
                );
            }
            Err(error) => self.story_query_sheet.set_error("Story Query Sheet", error),
        }
    }

    pub(crate) fn open_first_story_query_source(&mut self) {
        let Some(target) = self.story_query_sheet.source_targets.first().cloned() else {
            self.status_message = "No source target in current story sheet.".to_string();
            return;
        };

        if !self.navigate_to_path(target.path.clone()) {
            return;
        }
        let line = target
            .line
            .min(self.document.line_count().saturating_sub(1));
        self.set_cursor(Position { line, column: 0 }, true);
        self.top_line = line.saturating_sub(3);
        self.processed_top_line = self.top_line;
        self.processed_top_visual = self.top_line;
        self.story_query_sheet.open = false;
        self.status_message = format!("Opened story query source at line {}.", line + 1);
    }

    pub(crate) fn open_story_query_link_target(&mut self, target: String) {
        match self.resolve_script_target_path(&target) {
            Ok(path) => {
                let metadata_warning = basscript_core::EntityDocument::load(&path).err();
                if !self.navigate_to_path(path.clone()) {
                    return;
                }
                self.story_query_sheet.open = false;
                if let Some(error) = metadata_warning {
                    self.status_message = format!(
                        "Loaded {} with metadata warning: {error}",
                        status_path_label(&path)
                    );
                } else {
                    self.status_message = format!("Opened linked entity `{target}`.");
                }
            }
            Err(message) => {
                self.status_message = message;
            }
        }
    }

    pub(crate) fn selected_story_query_scene(
        &self,
        database: &basscript_core::StoryIndexDatabase,
    ) -> Result<Option<StoryIndexSceneRecord>, String> {
        match self.story_query_sheet.scene_scope {
            StoryQuerySceneScope::Current => database
                .scene_at_line(&self.paths.save_path, self.cursor.position.line)
                .map_err(|error| format!("Current scene lookup failed: {error}")),
            StoryQuerySceneScope::Selected => {
                if self.story_query_sheet.scene_index == 0 {
                    return Ok(None);
                }

                Ok(self
                    .story_query_sheet
                    .scenes
                    .get(self.story_query_sheet.scene_index - 1)
                    .cloned())
            }
            StoryQuerySceneScope::All => Ok(None),
        }
    }
}

pub(crate) fn build_dialogue_by_character_output(
    database: &basscript_core::StoryIndexDatabase,
    character: &StoryIndexEntityRecord,
) -> Result<StoryQueryRunOutput, String> {
    let scenes = database
        .scenes_containing_entity(&character.target)
        .map_err(|error| format!("Dialogue scene query failed: {error}"))?;
    let extract = fountain_dialogue_extract(&scenes, &[character.target.clone()])?;
    Ok(StoryQueryRunOutput {
        title: format!("Dialogue: {}", character.name),
        status: format!("{} scenes matched.", scenes.len()),
        format: StoryQueryOutputFormat::Fountain,
        text: extract.text,
        source_targets: extract.source_targets,
    })
}

pub(crate) fn build_dialogue_between_output(
    database: &basscript_core::StoryIndexDatabase,
    character_a: &StoryIndexEntityRecord,
    character_b: &StoryIndexEntityRecord,
    scene_filter: Option<StoryIndexSceneRecord>,
) -> Result<StoryQueryRunOutput, String> {
    let mut scenes = database
        .scenes_containing_all_entities([character_a.target.as_str(), character_b.target.as_str()])
        .map_err(|error| format!("Dialogue co-appearance query failed: {error}"))?;
    let location_label = scene_filter
        .as_ref()
        .and_then(|scene| scene.location_text.as_deref())
        .map(|location| location.to_ascii_lowercase());
    if let Some(filter) = scene_filter.as_ref() {
        scenes.retain(|scene| {
            if let Some(location_label) = location_label.as_deref() {
                scene
                    .location_text
                    .as_deref()
                    .map(|location| location.eq_ignore_ascii_case(location_label))
                    .unwrap_or(false)
            } else {
                scene.scene_key == filter.scene_key
            }
        });
    }

    let extract = fountain_dialogue_extract(
        &scenes,
        &[character_a.target.clone(), character_b.target.clone()],
    )?;
    let filter_label = scene_filter
        .as_ref()
        .map(|scene| {
            scene
                .location_text
                .clone()
                .unwrap_or_else(|| scene.heading_text.clone())
        })
        .map(|label| format!(" at {label}"))
        .unwrap_or_default();
    Ok(StoryQueryRunOutput {
        title: format!(
            "Dialogue: {} / {}{}",
            character_a.name, character_b.name, filter_label
        ),
        status: format!("{} scenes matched.", scenes.len()),
        format: StoryQueryOutputFormat::Fountain,
        text: extract.text,
        source_targets: extract.source_targets,
    })
}

pub(crate) fn build_category_scenes_output(
    database: &basscript_core::StoryIndexDatabase,
    taxonomy: &StoryTaxonomyConfig,
    selected_category_indices: &[usize],
    scenes: &[StoryIndexSceneRecord],
    scope_label: &str,
    scene_first: bool,
) -> Result<StoryQueryRunOutput, String> {
    let mut source_targets = Vec::<StoryQuerySourceTarget>::new();
    let mut scene_groups = Vec::<(&StoryIndexSceneRecord, StoryQueryCategoryGroup)>::new();
    for scene in scenes {
        let mut grouped = StoryQueryCategoryGroup::new();
        let appearances = database
            .appearances_in_scene(&scene.scene_key)
            .map_err(|error| format!("Scene appearance query failed: {error}"))?
            .into_iter();
        for appearance in appearances {
            let Some(category_index) = appearance.entity_type.as_deref().and_then(|entity_type| {
                story_taxonomy_category_for_type(taxonomy, selected_category_indices, entity_type)
            }) else {
                continue;
            };
            source_targets.push(StoryQuerySourceTarget {
                path: appearance.source_path.clone(),
                line: appearance.line,
            });
            grouped
                .entry(category_index)
                .or_default()
                .entry(appearance.target.clone())
                .or_default()
                .push(appearance);
        }
        if !grouped.is_empty() {
            scene_groups.push((scene, grouped));
        }
    }

    let selected_label = selected_category_indices
        .iter()
        .filter_map(|index| taxonomy.categories.get(*index))
        .map(|category| category.display_label())
        .collect::<Vec<_>>()
        .join(" + ");
    let mut text = String::new();
    text.push_str(&format!("Scope: {scope_label}\n"));
    text.push_str(&format!("Categories: {selected_label}\n\n"));
    if scenes.is_empty() {
        text.push_str("No indexed scenes were found.\n");
    } else if scene_groups.is_empty() {
        text.push_str("No linked entities matched the selected categories in this scope.\n");
    }

    let mut occurrence_count = 0usize;
    if !scene_first && scene_groups.len() == 1 {
        let (scene, grouped) = &scene_groups[0];
        write_category_scene_heading(&mut text, scene, false);
        occurrence_count += write_category_group(&mut text, taxonomy, grouped);
    } else {
        for (scene, grouped) in &scene_groups {
            write_category_scene_heading(&mut text, scene, true);
            occurrence_count += write_category_group(&mut text, taxonomy, grouped);
        }
    }

    Ok(StoryQueryRunOutput {
        title: format!("Entities by category: {scope_label}"),
        status: format!("{occurrence_count} categorized appearances matched."),
        format: StoryQueryOutputFormat::Markdown,
        text,
        source_targets,
    })
}

pub(crate) fn write_category_scene_heading(
    text: &mut String,
    scene: &StoryIndexSceneRecord,
    include_separator: bool,
) {
    if include_separator && !text.ends_with("\n\n") {
        text.push('\n');
    }
    text.push_str(&scene.heading_text);
    text.push('\n');
    text.push_str(&format!(
        "Source: {}:{}\n\n",
        scene.relative_path,
        scene.start_line + 1
    ));
}

pub(crate) fn write_category_group(
    text: &mut String,
    taxonomy: &StoryTaxonomyConfig,
    grouped: &StoryQueryCategoryGroup,
) -> usize {
    let mut occurrence_count = 0usize;
    for (category_index, entities) in grouped {
        let category_label = taxonomy
            .categories
            .get(*category_index)
            .map(|category| category.display_label())
            .unwrap_or("Category");
        text.push_str(&format!("{category_label}\n"));
        for (target, appearances) in entities {
            let name = appearances
                .first()
                .and_then(|appearance| appearance.entity_name.as_deref())
                .unwrap_or(target);
            let entity_type = appearances
                .first()
                .and_then(|appearance| appearance.entity_type.as_deref())
                .unwrap_or("entity");
            let linked_name = story_query_entity_link(name, target);
            text.push_str(&format!("  {linked_name} ({entity_type})\n"));
            for appearance in appearances {
                occurrence_count += 1;
                text.push_str(&format!(
                    "    line {} - {} - {}\n",
                    appearance.line + 1,
                    human_story_query_role(&appearance.role),
                    trim_category_result_snippet(&appearance.raw_snippet)
                ));
            }
            text.push('\n');
        }
    }
    occurrence_count
}

pub(crate) fn human_story_query_role(role: &basscript_core::StoryIndexAppearanceRole) -> String {
    role.as_database_value().replace('_', " ")
}

pub(crate) fn build_appearances_output(
    database: &basscript_core::StoryIndexDatabase,
    entity: &StoryIndexEntityRecord,
) -> Result<StoryQueryRunOutput, String> {
    let appearances = database
        .appearances_of_entity(&entity.target)
        .map_err(|error| format!("Appearance query failed: {error}"))?;
    let mut text = String::new();
    text.push_str(&format!(
        "# Appearances: {}\n\n",
        story_query_entity_link(&entity.name, &entity.target)
    ));
    text.push_str(&format!(
        "Target: `{}`\nType: `{}`\n\n",
        entity.target, entity.entity_type
    ));
    if appearances.is_empty() {
        text.push_str("No appearances were found.\n");
    }
    let mut source_targets = Vec::<StoryQuerySourceTarget>::new();
    for appearance in &appearances {
        source_targets.push(StoryQuerySourceTarget {
            path: appearance.source_path.clone(),
            line: appearance.line,
        });
        text.push_str(&format!(
            "- {}:{} - {} - {}\n",
            appearance.relative_path,
            appearance.line + 1,
            appearance.role.as_database_value(),
            trim_result_snippet(&appearance.raw_snippet)
        ));
    }

    Ok(StoryQueryRunOutput {
        title: format!("Appearances: {}", entity.name),
        status: format!("{} appearances matched.", appearances.len()),
        format: StoryQueryOutputFormat::Markdown,
        text,
        source_targets,
    })
}

pub(crate) struct FountainDialogueExtract {
    pub(crate) text: String,
    pub(crate) source_targets: Vec<StoryQuerySourceTarget>,
}

pub(crate) fn fountain_dialogue_extract(
    scenes: &[StoryIndexSceneRecord],
    targets: &[String],
) -> Result<FountainDialogueExtract, String> {
    if scenes.is_empty() {
        return Ok(FountainDialogueExtract {
            text: "No matching dialogue scenes were found.".to_string(),
            source_targets: Vec::new(),
        });
    }

    let mut text = String::new();
    let mut source_targets = Vec::<StoryQuerySourceTarget>::new();
    let mut block_count = 0usize;
    for scene in scenes {
        let source = fs::read_to_string(&scene.source_path).map_err(|error| {
            format!(
                "Could not read {} for dialogue extraction: {error}",
                scene.source_path.display()
            )
        })?;
        let document = Document::from_text(&source);
        let parsed = parse_document_with_format(&document, DocumentFormat::Fountain);
        let mut scene_text = String::new();
        let mut line = scene.start_line.saturating_add(1);
        while line <= scene.end_line && line < parsed.len() {
            let parsed_line = &parsed[line];
            if parsed_line.kind != LineKind::Character {
                line += 1;
                continue;
            }

            let Some(target) = linked_character_target(parsed_line) else {
                line += 1;
                continue;
            };
            if !targets.iter().any(|candidate| candidate == &target) {
                line += 1;
                continue;
            }

            source_targets.push(StoryQuerySourceTarget {
                path: scene.source_path.clone(),
                line,
            });
            scene_text.push_str(&format!("Source: {}:{}\n", scene.relative_path, line + 1));
            scene_text.push_str(&raw_fountain_query_line(parsed_line));
            scene_text.push('\n');
            line += 1;
            while line <= scene.end_line && line < parsed.len() {
                let next = &parsed[line];
                if !matches!(next.kind, LineKind::Parenthetical | LineKind::Dialogue) {
                    break;
                }
                scene_text.push_str(&raw_fountain_query_line(next));
                scene_text.push('\n');
                line += 1;
            }
            scene_text.push('\n');
            block_count += 1;
        }

        if !scene_text.is_empty() {
            text.push_str(&scene.heading_text);
            text.push('\n');
            text.push_str(&format!(
                "Source: {}:{}\n\n",
                scene.relative_path,
                scene.start_line + 1
            ));
            text.push_str(&scene_text);
        }
    }

    if block_count == 0 {
        Ok(FountainDialogueExtract {
            text: "No matching dialogue blocks were found.".to_string(),
            source_targets,
        })
    } else {
        Ok(FountainDialogueExtract {
            text,
            source_targets,
        })
    }
}

pub(crate) fn linked_character_target(parsed_line: &ParsedLine) -> Option<String> {
    let [link] = parsed_line.script_links.as_slice() else {
        return None;
    };
    Some(link.target.clone())
}

pub(crate) fn raw_fountain_query_line(parsed_line: &ParsedLine) -> String {
    parsed_line.raw.clone()
}

pub(crate) fn load_story_taxonomy() -> StoryTaxonomyLoad {
    let path = PathBuf::from(STORY_TAXONOMY_SETTINGS_PATH);
    match fs::read_to_string(&path) {
        Ok(contents) => match StoryTaxonomyConfig::from_ron(&contents) {
            Ok(taxonomy) => StoryTaxonomyLoad {
                taxonomy,
                notice: None,
            },
            Err(error) => {
                warn!(
                    "[story_taxonomy] Failed parsing {}: {}; using defaults",
                    path.display(),
                    error
                );
                StoryTaxonomyLoad {
                    taxonomy: StoryTaxonomyConfig::default(),
                    notice: Some(format!("Taxonomy invalid; using defaults. {error}")),
                }
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut notice = None;
            if let Some(parent) = path.parent() {
                if let Err(create_error) = fs::create_dir_all(parent) {
                    warn!(
                        "[story_taxonomy] Failed creating {}: {}; using defaults",
                        parent.display(),
                        create_error
                    );
                    notice = Some(format!(
                        "Taxonomy defaults active; could not create {}.",
                        path.display()
                    ));
                }
            }
            if notice.is_none() {
                match fs::write(&path, DEFAULT_STORY_TAXONOMY_RON) {
                    Ok(()) => {
                        info!(
                            "[story_taxonomy] Created default taxonomy at {}",
                            path.display()
                        );
                        notice = Some(format!("Created default taxonomy at {}.", path.display()));
                    }
                    Err(write_error) => {
                        warn!(
                            "[story_taxonomy] Failed writing {}: {}; using defaults",
                            path.display(),
                            write_error
                        );
                        notice = Some(format!(
                            "Taxonomy defaults active; could not write {}.",
                            path.display()
                        ));
                    }
                }
            }
            StoryTaxonomyLoad {
                taxonomy: StoryTaxonomyConfig::default(),
                notice,
            }
        }
        Err(error) => {
            warn!(
                "[story_taxonomy] Failed reading {}: {}; using defaults",
                path.display(),
                error
            );
            StoryTaxonomyLoad {
                taxonomy: StoryTaxonomyConfig::default(),
                notice: Some(format!("Taxonomy unreadable; using defaults. {error}")),
            }
        }
    }
}

pub(crate) fn normalize_story_taxonomy_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub(crate) fn story_taxonomy_category_for_type(
    taxonomy: &StoryTaxonomyConfig,
    selected_category_indices: &[usize],
    entity_type: &str,
) -> Option<usize> {
    let entity_type = normalize_story_taxonomy_key(entity_type);
    selected_category_indices.iter().copied().find(|index| {
        taxonomy
            .categories
            .get(*index)
            .map(|category| {
                category
                    .types
                    .iter()
                    .any(|candidate| candidate == &entity_type)
            })
            .unwrap_or(false)
    })
}

pub(crate) fn selected_category_indices(sheet: &StoryQuerySheet) -> Vec<usize> {
    let mut selected = Vec::<usize>::new();
    for category_index in &sheet.category_indices {
        if *category_index < sheet.taxonomy.categories.len() && !selected.contains(category_index) {
            selected.push(*category_index);
        }
    }
    selected
}

pub(crate) fn selected_category_label(sheet: &StoryQuerySheet, index: usize) -> String {
    sheet
        .taxonomy
        .categories
        .get(index)
        .map(|category| category.display_label().to_string())
        .unwrap_or_else(|| "none".to_string())
}

pub(crate) fn selected_character(
    sheet: &StoryQuerySheet,
    index: usize,
) -> Option<&StoryIndexEntityRecord> {
    sheet.characters.get(index)
}

pub(crate) fn selected_entity(
    sheet: &StoryQuerySheet,
    index: usize,
) -> Option<&StoryIndexEntityRecord> {
    sheet.entities.get(index)
}

pub(crate) fn selected_character_label(sheet: &StoryQuerySheet, index: usize) -> String {
    selected_character(sheet, index)
        .map(entity_label)
        .unwrap_or_else(|| "none".to_string())
}

pub(crate) fn selected_entity_label(sheet: &StoryQuerySheet, index: usize) -> String {
    selected_entity(sheet, index)
        .map(entity_label)
        .unwrap_or_else(|| "none".to_string())
}

pub(crate) fn selected_scene_label(state: &EditorState) -> String {
    let sheet = &state.story_query_sheet;
    if sheet.scene_index == 0 {
        return "none".to_string();
    }

    sheet
        .scenes
        .get(sheet.scene_index - 1)
        .map(scene_label)
        .unwrap_or_else(|| "none".to_string())
}

pub(crate) fn scene_label(scene: &StoryIndexSceneRecord) -> String {
    scene
        .location_text
        .as_ref()
        .map(|location| format!("{} ({})", scene.heading_text, location))
        .unwrap_or_else(|| scene.heading_text.clone())
}

pub(crate) fn entity_label(entity: &StoryIndexEntityRecord) -> String {
    format!("{} ({})", entity.name, entity.target)
}

pub(crate) fn compact_story_query_label(label: &str) -> String {
    const LIMIT: usize = 34;
    if label.chars().count() <= LIMIT {
        return label.to_string();
    }

    let mut out = label
        .chars()
        .take(LIMIT.saturating_sub(3))
        .collect::<String>();
    out.push_str("...");
    out
}

pub(crate) fn merge_story_query_entities(
    entities: &mut Vec<StoryIndexEntityRecord>,
    incoming: Vec<StoryIndexEntityRecord>,
) {
    let mut seen = entities
        .iter()
        .map(|entity| entity.target.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for entity in incoming {
        if seen.insert(entity.target.to_ascii_lowercase()) {
            entities.push(entity);
        }
    }
    entities.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.target.cmp(&right.target))
    });
}

pub(crate) fn trim_result_snippet(snippet: &str) -> String {
    let trimmed = snippet.trim();
    if trimmed.chars().count() <= 120 {
        return trimmed.to_string();
    }

    let mut out = trimmed.chars().take(117).collect::<String>();
    out.push_str("...");
    out
}

pub(crate) fn trim_category_result_snippet(snippet: &str) -> String {
    let trimmed = snippet.trim();
    if trimmed.chars().count() <= 72 {
        return trimmed.to_string();
    }

    let mut out = trimmed.chars().take(69).collect::<String>();
    out.push_str("...");
    out
}

pub(crate) fn story_query_document_format(format: StoryQueryOutputFormat) -> DocumentFormat {
    match format {
        StoryQueryOutputFormat::Fountain => DocumentFormat::Fountain,
        StoryQueryOutputFormat::Markdown => DocumentFormat::Markdown,
    }
}

pub(crate) fn story_query_visual_lines(
    text: &str,
    output_format: StoryQueryOutputFormat,
    dialogue_double_space_newline: bool,
    non_dialogue_double_space_newline: bool,
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
) -> Vec<ProcessedVisualLine> {
    let document_format = story_query_document_format(output_format);
    let document = Document::from_text(text);
    let parsed = parse_document_with_format(&document, document_format);
    let wrap_columns = wrap_columns.max(1);
    let lines_per_page = lines_per_page.max(1);
    let mut visual_lines = Vec::<ProcessedVisualLine>::new();
    let mut page_fill = ProcessedPageFill::default();

    for (source_line, parsed_line) in parsed.iter().enumerate() {
        let visual_override = if document_format == DocumentFormat::Markdown {
            markdown_render_override_for_raw(&parsed_line.raw)
        } else {
            None
        };
        let effective_kind = visual_override
            .as_ref()
            .map(|override_style| &override_style.kind)
            .unwrap_or(&parsed_line.kind);
        let markdown_heading_level = visual_override
            .as_ref()
            .and_then(|override_style| override_style.markdown_heading_level)
            .or(parsed_line.markdown_heading_level);
        let indent_width = parsed_line.indent_width();
        let uppercase = matches!(
            effective_kind,
            LineKind::SceneHeading | LineKind::Transition | LineKind::Character
        );
        let (prepared_text, checklist_state) =
            prepare_processed_line_text(parsed_line, false, visual_override.as_ref());
        let mut wrapped = Vec::<ProcessedVisualLine>::new();

        if story_query_should_split_on_double_space(
            effective_kind,
            dialogue_double_space_newline,
            non_dialogue_double_space_newline,
        ) {
            for (segment_start, segment_end) in double_space_segments(&prepared_text.text) {
                push_wrapped_visual_lines(
                    &mut wrapped,
                    source_line,
                    indent_width,
                    uppercase,
                    &prepared_text,
                    segment_start,
                    segment_end,
                    wrap_columns,
                );
            }
        } else {
            push_wrapped_visual_lines(
                &mut wrapped,
                source_line,
                indent_width,
                uppercase,
                &prepared_text,
                0,
                prepared_text.text.chars().count(),
                wrap_columns,
            );
        }

        if let Some(checked) = checklist_state {
            if let Some(first_wrapped) = wrapped.first_mut() {
                first_wrapped.markdown_checklist_checked = Some(checked);
            }
        }

        let style_override = ProcessedLineRenderOverride {
            kind: effective_kind.clone(),
            markdown_heading_level,
        };
        for visual_line in &mut wrapped {
            visual_line.render_override = Some(style_override.clone());
        }

        let line_height_units = processed_line_style_for_kind(
            &style_override.kind,
            style_override.markdown_heading_level,
        )
        .line_height_scale;

        for visual_line in wrapped {
            if page_fill.entries > 0
                && page_fill.height_units + line_height_units > lines_per_page as f32 + 0.001
            {
                finish_processed_page(
                    &mut visual_lines,
                    source_line,
                    &mut page_fill,
                    lines_per_page,
                    spacer_lines,
                );
            }

            visual_lines.push(visual_line);
            page_fill.entries = page_fill.entries.saturating_add(1);
            page_fill.height_units += line_height_units;

            if page_fill.height_units >= lines_per_page as f32 - 0.001 {
                finish_processed_page(
                    &mut visual_lines,
                    source_line,
                    &mut page_fill,
                    lines_per_page,
                    spacer_lines,
                );
            }
        }
    }

    if visual_lines.is_empty() {
        visual_lines.push(story_query_blank_visual_line());
    }

    visual_lines
}

pub(crate) fn story_query_should_split_on_double_space(
    kind: &LineKind,
    dialogue_double_space_newline: bool,
    non_dialogue_double_space_newline: bool,
) -> bool {
    if matches!(
        kind,
        LineKind::MarkdownHeading
            | LineKind::MarkdownListItem
            | LineKind::MarkdownQuote
            | LineKind::MarkdownCodeFence
            | LineKind::MarkdownCode
            | LineKind::MarkdownRule
            | LineKind::MarkdownParagraph
    ) {
        return false;
    }

    match kind {
        LineKind::Dialogue => dialogue_double_space_newline,
        _ => non_dialogue_double_space_newline,
    }
}

pub(crate) fn story_query_blank_visual_line() -> ProcessedVisualLine {
    ProcessedVisualLine {
        source_line: 0,
        text: " ".to_string(),
        fragments: vec![ProcessedVisualFragment {
            text: " ".to_string(),
            is_link: false,
            link_target: None,
            inline_style: InlineTextStyle::default(),
        }],
        display_to_raw: vec![0, 0],
        raw_start_column: 0,
        raw_end_column: 0,
        markdown_checklist_checked: None,
        image_block: None,
        render_override: Some(ProcessedLineRenderOverride {
            kind: LineKind::Action,
            markdown_heading_level: None,
        }),
        is_spacer: false,
    }
}

pub(crate) fn story_query_link_color_for_target(
    sheet: &StoryQuerySheet,
    state: &EditorState,
    target: Option<&str>,
) -> Color {
    let Some(target) = target else {
        return state.processed_link_color_for_target(None);
    };
    if let Some(entity_type) = state.script_link_target_types.get(target) {
        return color_from_rgba(state.link_rgba_for_type(entity_type));
    }

    sheet
        .entities
        .iter()
        .chain(sheet.characters.iter())
        .find(|entity| entity.target == target)
        .map(|entity| color_from_rgba(state.link_rgba_for_type(&entity.entity_type)))
        .unwrap_or_else(|| state.processed_link_color_for_target(Some(target)))
}

pub(crate) fn story_query_link_target_at_position(
    sheet: &StoryQuerySheet,
    text_block: &ComputedTextBlock,
    inverse_scale: f32,
    local_x: f32,
    local_y: f32,
    slot: usize,
    first_visible_page: usize,
    page_step_lines: usize,
    lines_per_page: usize,
    document_format: DocumentFormat,
    zoom: f32,
) -> Option<String> {
    let fallback_char_width =
        (default_char_width_for_format(document_format) * zoom.max(f32::EPSILON)).max(1.0);
    let line_height = (LINE_HEIGHT * zoom.max(f32::EPSILON)).max(1.0);
    let page_step_lines = page_step_lines.max(1);
    let lines_per_page = lines_per_page.max(1).min(page_step_lines);
    let fallback_line =
        ((local_y / line_height).floor().max(0.0) as usize).min(lines_per_page.saturating_sub(1));
    let line_offset = line_index_from_layout_y(text_block, local_y, lines_per_page, inverse_scale)
        .unwrap_or(fallback_line)
        .min(lines_per_page.saturating_sub(1));
    let page_index = first_visible_page.saturating_add(slot);
    let global_index = page_index
        .saturating_mul(page_step_lines)
        .saturating_add(line_offset);
    let visual_line = sheet.visual_lines.get(global_index)?;
    let fallback_column = (local_x / fallback_char_width).round().max(0.0) as usize;
    let display_column = column_from_layout_x(
        text_block,
        line_offset,
        local_x,
        &visual_line.text,
        inverse_scale,
        fallback_char_width,
    )
    .unwrap_or(fallback_column);

    story_query_link_target_at_column(visual_line, display_column)
}

pub(crate) fn story_query_link_target_at_column(
    visual_line: &ProcessedVisualLine,
    display_column: usize,
) -> Option<String> {
    let mut cursor = 0usize;
    for fragment in &visual_line.fragments {
        let next = cursor.saturating_add(fragment.text.chars().count());
        if display_column >= cursor && display_column < next {
            return fragment.link_target.clone();
        }
        cursor = next;
    }

    visual_line
        .fragments
        .last()
        .filter(|_| display_column == cursor)
        .and_then(|fragment| fragment.link_target.clone())
}

pub(crate) fn story_query_entity_link(label: &str, target: &str) -> String {
    if !basscript_core::is_valid_target_key(target) {
        return label.to_string();
    }
    let label = label.replace('[', "(").replace(']', ")").replace('\n', " ");
    format!("[{label}]({target})")
}

pub(crate) fn clamp_story_query_index(index: &mut usize, len: usize) {
    if len == 0 {
        *index = 0;
    } else {
        *index = (*index).min(len - 1);
    }
}

pub(crate) fn story_query_can_add_category(sheet: &StoryQuerySheet) -> bool {
    if sheet.query_kind != StoryQueryKind::CategoriesInScene {
        return false;
    }
    let category_count = sheet.taxonomy.categories.len();
    let max_rows = STORY_QUERY_MAX_CATEGORY_ROWS.min(category_count);
    sheet.category_indices.len() < max_rows
}

pub(crate) fn add_story_query_category(sheet: &mut StoryQuerySheet) {
    if !story_query_can_add_category(sheet) {
        return;
    }

    let selected = sheet
        .category_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let next = (0..sheet.taxonomy.categories.len())
        .find(|index| !selected.contains(index))
        .unwrap_or(0);
    sheet.category_indices.push(next);
    clamp_story_query_dependencies(sheet);
}

pub(crate) fn clamp_story_query_categories(sheet: &mut StoryQuerySheet) {
    let category_count = sheet.taxonomy.categories.len();
    if category_count == 0 {
        sheet.category_indices.clear();
        return;
    }

    let max_rows = STORY_QUERY_MAX_CATEGORY_ROWS.min(category_count);
    if sheet.category_indices.is_empty() {
        sheet.category_indices.push(0);
    }
    sheet.category_indices.truncate(max_rows);

    let mut used = BTreeSet::<usize>::new();
    let mut normalized = Vec::<usize>::new();
    for selected in &sheet.category_indices {
        let mut category_index = (*selected).min(category_count - 1);
        if used.contains(&category_index) {
            if let Some(replacement) = (0..category_count).find(|index| !used.contains(index)) {
                category_index = replacement;
            } else {
                continue;
            }
        }
        used.insert(category_index);
        normalized.push(category_index);
    }

    if normalized.is_empty() {
        normalized.push(0);
    }
    sheet.category_indices = normalized;
}

pub(crate) fn clamp_story_query_dependencies(sheet: &mut StoryQuerySheet) {
    if sheet.scene_index > sheet.scenes.len() {
        sheet.scene_index = sheet.scenes.len();
    }
    if sheet.scene_scope == StoryQuerySceneScope::Selected
        && sheet.scene_index == 0
        && !sheet.scenes.is_empty()
    {
        sheet.scene_index = 1;
    }
    clamp_story_query_categories(sheet);
    if sheet
        .open_dropdown
        .map(|kind| !story_query_control_visible(sheet, story_query_dropdown_slot(kind)))
        .unwrap_or(false)
    {
        sheet.open_dropdown = None;
    }
}
#[allow(unused_imports)]
use super::*;
