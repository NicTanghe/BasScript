use basscript_core::{
    StoryIndexAppearanceRecord, StoryIndexEntityRecord, StoryIndexSceneRecord,
};
use serde::Deserialize;

const STORY_QUERY_RESULT_WIDTH_PERCENT: f32 = 61.803;
const STORY_QUERY_MENU_WIDTH_PERCENT: f32 = 38.197;
const STORY_QUERY_A4_SCALE: f32 = 0.86;
const STORY_QUERY_A4_PAGE_WIDTH: f32 = A4_WIDTH_POINTS * STORY_QUERY_A4_SCALE;
const STORY_QUERY_A4_PAGE_HEIGHT: f32 = A4_HEIGHT_POINTS * STORY_QUERY_A4_SCALE;
const STORY_QUERY_RESULT_FONT_SIZE: f32 = FONT_SIZE * STORY_QUERY_A4_SCALE;
const STORY_QUERY_RESULT_LINE_HEIGHT: f32 = LINE_HEIGHT * STORY_QUERY_A4_SCALE;
const STORY_QUERY_PAGE_LINE_LIMIT: usize = 56;
const STORY_QUERY_DROPDOWN_VISIBLE_OPTIONS: usize = 8;
const STORY_QUERY_MAX_CATEGORY_ROWS: usize = 8;
const DEFAULT_STORY_TAXONOMY_RON: &str = r#"(
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
enum StoryQueryKind {
    DialogueByCharacter,
    DialogueBetweenCharacters,
    DialogueBetweenAtScene,
    CategoriesInScene,
    AppearancesOfEntity,
}

const STORY_QUERY_KINDS: [StoryQueryKind; 5] = [
    StoryQueryKind::DialogueByCharacter,
    StoryQueryKind::DialogueBetweenCharacters,
    StoryQueryKind::DialogueBetweenAtScene,
    StoryQueryKind::CategoriesInScene,
    StoryQueryKind::AppearancesOfEntity,
];

impl StoryQueryKind {
    fn label(self) -> &'static str {
        match self {
            Self::DialogueByCharacter => "All dialogue by character",
            Self::DialogueBetweenCharacters => "Dialogue between characters",
            Self::DialogueBetweenAtScene => "Dialogue at scene/location",
            Self::CategoriesInScene => "Entities by category",
            Self::AppearancesOfEntity => "Appearances of entity",
        }
    }

    fn index(self) -> usize {
        STORY_QUERY_KINDS
            .iter()
            .position(|kind| *kind == self)
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoryQuerySceneScope {
    Current,
    Selected,
    All,
}

const STORY_QUERY_SCENE_SCOPES: [StoryQuerySceneScope; 3] = [
    StoryQuerySceneScope::Current,
    StoryQuerySceneScope::Selected,
    StoryQuerySceneScope::All,
];

impl StoryQuerySceneScope {
    fn label(self) -> &'static str {
        match self {
            Self::Current => "Current scene",
            Self::Selected => "Selected scene",
            Self::All => "All scenes",
        }
    }

    fn index(self) -> usize {
        STORY_QUERY_SCENE_SCOPES
            .iter()
            .position(|scope| *scope == self)
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct StoryTaxonomyConfig {
    #[serde(default)]
    categories: Vec<StoryTaxonomyCategory>,
}

#[derive(Clone, Debug, Deserialize)]
struct StoryTaxonomyCategory {
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    types: Vec<String>,
}

struct StoryTaxonomyLoad {
    taxonomy: StoryTaxonomyConfig,
    notice: Option<String>,
}

impl Default for StoryTaxonomyConfig {
    fn default() -> Self {
        Self::from_ron(DEFAULT_STORY_TAXONOMY_RON)
            .expect("DEFAULT_STORY_TAXONOMY_RON must be valid")
    }
}

impl StoryTaxonomyConfig {
    fn from_ron(contents: &str) -> Result<Self, String> {
        let mut taxonomy = ron::from_str::<Self>(contents)
            .map_err(|error| format!("Could not parse taxonomy RON: {error}"))?;
        taxonomy.normalize();
        if taxonomy.categories.is_empty() {
            return Err("Taxonomy must define at least one non-empty category.".to_string());
        }
        Ok(taxonomy)
    }

    fn normalize(&mut self) {
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
    fn display_label(&self) -> &str {
        if self.label.is_empty() {
            &self.id
        } else {
            &self.label
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoryQueryOutputFormat {
    Fountain,
    Markdown,
}

#[derive(Clone, Debug)]
struct StoryQuerySheet {
    open: bool,
    query_kind: StoryQueryKind,
    characters: Vec<StoryIndexEntityRecord>,
    entities: Vec<StoryIndexEntityRecord>,
    scenes: Vec<StoryIndexSceneRecord>,
    character_a_index: usize,
    character_b_index: usize,
    entity_index: usize,
    scene_index: usize,
    scene_scope: StoryQuerySceneScope,
    category_indices: Vec<usize>,
    taxonomy: StoryTaxonomyConfig,
    taxonomy_notice: Option<String>,
    open_dropdown: Option<StoryQueryDropdownKind>,
    page_index: usize,
    visual_pages: Vec<Vec<ProcessedVisualLine>>,
    result_title: String,
    result_status: String,
    result_format: StoryQueryOutputFormat,
    source_targets: Vec<StoryQuerySourceTarget>,
}

#[derive(Clone, Debug)]
struct StoryQuerySourceTarget {
    path: PathBuf,
    line: usize,
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
            page_index: 0,
            visual_pages: story_query_visual_pages(
                "No result yet.",
                StoryQueryOutputFormat::Markdown,
                false,
                false,
            ),
            result_title: "Story Query Sheet".to_string(),
            result_status: "Ready.".to_string(),
            result_format: StoryQueryOutputFormat::Fountain,
            source_targets: Vec::new(),
        }
    }
}

#[derive(Component)]
struct StoryQuerySheetRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum StoryQuerySheetTextSlot {
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
enum StoryQueryDropdownKind {
    QueryKind,
    SceneScope,
    PrimaryCharacter,
    SecondaryCharacter,
    Entity,
    Scene,
    Category(usize),
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum StoryQuerySheetAction {
    ToggleDropdown(StoryQueryDropdownKind),
    SelectDropdownOption {
        kind: StoryQueryDropdownKind,
        slot_index: usize,
    },
    Run,
    OpenFirstSource,
    PreviousPage,
    NextPage,
    AddCategory,
    Close,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct StoryQueryControlRow {
    slot: StoryQuerySheetTextSlot,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct StoryQueryDropdownOptionsRoot {
    kind: StoryQueryDropdownKind,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct StoryQueryDropdownOptionNode {
    kind: StoryQueryDropdownKind,
    slot_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct StoryQueryDropdownOptionText {
    kind: StoryQueryDropdownKind,
    slot_index: usize,
}

#[derive(Component)]
struct StoryQueryRenderedText;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct StoryQueryRenderedLineSpan {
    line_offset: usize,
    part_index: usize,
}

struct StoryQueryRunOutput {
    title: String,
    status: String,
    format: StoryQueryOutputFormat,
    text: String,
    source_targets: Vec<StoryQuerySourceTarget>,
}

type StoryQueryCategoryGroup = BTreeMap<usize, BTreeMap<String, Vec<StoryIndexAppearanceRecord>>>;

fn story_query_sheet_bundle(font: Handle<Font>) -> impl Bundle {
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

fn story_query_sheet_page_bundle(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: percent(STORY_QUERY_RESULT_WIDTH_PERCENT),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(PAGE_OUTER_MARGIN)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(COLOR_PANEL_BODY_PROCESSED),
        children![(
            Node {
                width: px(STORY_QUERY_A4_PAGE_WIDTH),
                height: px(STORY_QUERY_A4_PAGE_HEIGHT),
                flex_direction: FlexDirection::Column,
                row_gap: px(8.0),
                padding: UiRect::new(
                    px(PAGE_TEXT_MARGIN_LEFT * STORY_QUERY_A4_SCALE),
                    px(PAGE_TEXT_MARGIN_RIGHT * STORY_QUERY_A4_SCALE),
                    px(PAGE_TEXT_MARGIN_TOP * STORY_QUERY_A4_SCALE),
                    px(PAGE_TEXT_MARGIN_BOTTOM * STORY_QUERY_A4_SCALE),
                ),
                overflow: Overflow::clip(),
                border: UiRect::all(px(1.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.12)),
            BackgroundColor(COLOR_PAPER),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.22),
                px(0.0),
                px(12.0),
                px(24.0),
                px(0.0),
            ),
            children![
                story_query_text(
                    font.clone(),
                    "",
                    15.0,
                    COLOR_TEXT_MAIN,
                    StoryQuerySheetTextSlot::Title,
                ),
                (
                    Node {
                        width: percent(100.0),
                        flex_grow: 1.0,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    children![(
                        Text::new(""),
                        TextLayout::new_with_no_wrap(),
                        TextFont {
                            font: font.clone(),
                            font_size: STORY_QUERY_RESULT_FONT_SIZE,
                            ..default()
                        },
                        LineHeight::Px(STORY_QUERY_RESULT_LINE_HEIGHT),
                        TextColor(COLOR_ACTION),
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            overflow: Overflow::visible(),
                            ..default()
                        },
                        RelativeCursorPosition::default(),
                        StoryQueryRenderedText,
                    )],
                ),
                story_query_text(
                    font,
                    "",
                    10.0,
                    COLOR_TEXT_MUTED,
                    StoryQuerySheetTextSlot::Page,
                ),
            ],
        )],
    )
}

fn story_query_sheet_menu_bundle(font: Handle<Font>) -> impl Bundle {
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
                    padding: UiRect::bottom(px(3.0)),
                    ..default()
                },
                children![story_query_text(
                    font.clone(),
                    "Story Query Sheet",
                    16.0,
                    COLOR_TEXT_MAIN,
                    StoryQuerySheetTextSlot::Status,
                )],
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
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8.0),
                    ..default()
                },
                children![
                    story_query_static_button(
                        font.clone(),
                        "Prev page",
                        StoryQuerySheetAction::PreviousPage,
                    ),
                    story_query_static_button(font, "Next page", StoryQuerySheetAction::NextPage),
                ],
            ),
        ],
    )
}

fn setup_story_query_sheet_result_spans(
    mut commands: Commands,
    text_query: Query<Entity, With<StoryQueryRenderedText>>,
    fonts: Res<EditorFonts>,
) {
    for entity in text_query.iter() {
        commands.entity(entity).with_children(|text_parent| {
            for line_offset in 0..STORY_QUERY_PAGE_LINE_LIMIT {
                for part_index in 0..PROCESSED_LINE_SPAN_PARTS {
                    text_parent.spawn((
                        TextSpan::new(""),
                        TextFont {
                            font: fonts.regular.clone(),
                            font_size: STORY_QUERY_RESULT_FONT_SIZE,
                            ..default()
                        },
                        LineHeight::Px(STORY_QUERY_RESULT_LINE_HEIGHT),
                        TextColor(COLOR_ACTION),
                        StoryQueryRenderedLineSpan {
                            line_offset,
                            part_index,
                        },
                    ));
                }
            }
        });
    }
}

fn story_query_dropdown_row(
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

fn story_query_add_category_button_row(font: Handle<Font>) -> impl Bundle {
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

fn story_query_dropdown_options_bundle(
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

fn story_query_dropdown_option_button(
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
            TextLayout::new_with_no_wrap(),
            TextFont {
                font,
                font_size: 10.0,
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
            StoryQueryDropdownOptionText { kind, slot_index },
        )],
    )
}

fn story_query_control_button(
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
            TextLayout::new_with_no_wrap(),
            TextFont {
                font,
                font_size: 11.0,
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
            slot,
        )],
    )
}

fn story_query_static_button(
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
            TextLayout::new_with_no_wrap(),
            TextFont {
                font,
                font_size: 11.0,
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
        )],
    )
}

fn story_query_text(
    font: Handle<Font>,
    text: &str,
    font_size: f32,
    color: Color,
    slot: StoryQuerySheetTextSlot,
) -> impl Bundle {
    (
        Text::new(text),
        TextLayout::new_with_no_wrap(),
        TextFont {
            font,
            font_size,
            ..default()
        },
        LineHeight::Px(font_size + 2.0),
        TextColor(color),
        slot,
    )
}

fn sync_story_query_sheet_ui(
    state: Res<EditorState>,
    fonts: Res<EditorFonts>,
    mut root_query: Query<
        &mut Node,
        (
            With<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionsRoot>,
            Without<StoryQueryDropdownOptionNode>,
        ),
    >,
    mut row_query: Query<
        (&StoryQueryControlRow, &mut Node),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryDropdownOptionsRoot>,
            Without<StoryQueryDropdownOptionNode>,
        ),
    >,
    mut dropdown_root_query: Query<
        (&StoryQueryDropdownOptionsRoot, &mut Node),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionNode>,
        ),
    >,
    mut option_node_query: Query<
        (&StoryQueryDropdownOptionNode, &mut Node, &mut BackgroundColor),
        (
            Without<StoryQuerySheetRoot>,
            Without<StoryQueryControlRow>,
            Without<StoryQueryDropdownOptionsRoot>,
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
    let sheet = &state.story_query_sheet;
    if let Ok(mut root) = root_query.single_mut() {
        root.display = if sheet.open {
            Display::Flex
        } else {
            Display::None
        };
    }

    if !sheet.open {
        return;
    }

    for (row, mut node) in row_query.iter_mut() {
        node.display = if story_query_control_visible(sheet, row.slot) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (root, mut node) in dropdown_root_query.iter_mut() {
        let slot = story_query_dropdown_slot(root.kind);
        node.display = if sheet.open_dropdown == Some(root.kind)
            && story_query_control_visible(sheet, slot)
        {
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
            StoryQuerySheetTextSlot::Page => sheet.page_label(),
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

    apply_story_query_rendered_page_styles(&mut rendered_span_query, sheet, &state, &fonts);
}

fn apply_story_query_rendered_page_styles(
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
) {
    let document_format = story_query_document_format(sheet.result_format);
    let page = sheet
        .visual_pages
        .get(sheet.page_index)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    for (span, mut text_span, mut text_font, mut text_line_height, mut text_color) in
        rendered_span_query.iter_mut()
    {
        let Some(visual_line) = page.get(span.line_offset) else {
            **text_span = String::new();
            text_font.font = font_for_variant_with_format(fonts, FontVariant::Regular, document_format);
            text_font.font_size = STORY_QUERY_RESULT_FONT_SIZE;
            *text_line_height = LineHeight::Px(STORY_QUERY_RESULT_LINE_HEIGHT);
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
            text_font.font = font_for_variant_with_format(fonts, FontVariant::Regular, document_format);
            text_font.font_size = STORY_QUERY_RESULT_FONT_SIZE;
            *text_line_height = LineHeight::Px(STORY_QUERY_RESULT_LINE_HEIGHT);
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        };

        if span.part_index + 1 == used_fragment_count
            && span.line_offset + 1 < STORY_QUERY_PAGE_LINE_LIMIT
        {
            fragment.text.push('\n');
        }

        let effective_variant =
            font_variant_for_processed_fragment(style.font_variant, &fragment, document_format);
        text_font.font = font_for_variant_with_format(fonts, effective_variant, document_format);
        text_font.font_size = STORY_QUERY_RESULT_FONT_SIZE * style.font_scale;
        *text_line_height = LineHeight::Px(STORY_QUERY_RESULT_LINE_HEIGHT * style.line_height_scale);
        **text_span = fragment.text;
        text_color.0 = if allow_link_color && fragment.is_link {
            story_query_link_color_for_target(sheet, state, fragment.link_target.as_deref())
        } else {
            style.color
        };
    }
}

fn handle_story_query_sheet_buttons(
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
                    apply_story_query_dropdown_choice(&mut state.story_query_sheet, *kind, choice.value);
                    state.story_query_sheet.open_dropdown = None;
                    state.story_query_sheet.page_index = 0;
                    state.story_query_sheet.result_status = "Selection changed.".to_string();
                }
            }
            StoryQuerySheetAction::Run => state.run_story_query_sheet(),
            StoryQuerySheetAction::OpenFirstSource => state.open_first_story_query_source(),
            StoryQuerySheetAction::PreviousPage => state.story_query_sheet.previous_page(),
            StoryQuerySheetAction::NextPage => state.story_query_sheet.next_page(),
            StoryQuerySheetAction::AddCategory => {
                add_story_query_category(&mut state.story_query_sheet);
                state.story_query_sheet.open_dropdown = None;
                state.story_query_sheet.page_index = 0;
                state.story_query_sheet.result_status = "Category field added.".to_string();
            }
            StoryQuerySheetAction::Close => state.story_query_sheet.open = false,
        }
    }
}

fn handle_story_query_sheet_link_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    rendered_text_query: Query<
        (&RelativeCursorPosition, &ComputedNode, &TextLayoutInfo),
        With<StoryQueryRenderedText>,
    >,
    mut state: ResMut<EditorState>,
) {
    if !state.story_query_sheet.open || !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let target = rendered_text_query
        .iter()
        .find_map(|(relative_cursor, computed, layout)| {
            if !relative_cursor.cursor_over() {
                return None;
            }
            let normalized = relative_cursor.normalized?;
            let size = computed.size() * computed.inverse_scale_factor();
            let local_x = (normalized.x + 0.5) * size.x;
            let local_y = (normalized.y + 0.5) * size.y;
            story_query_link_target_at_position(
                &state.story_query_sheet,
                layout,
                computed.inverse_scale_factor(),
                local_x,
                local_y,
            )
        });

    if let Some(target) = target {
        state.open_story_query_link_target(target);
    }
}

fn handle_story_query_sheet_keyboard(
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
    } else if keys.just_pressed(KeyCode::PageUp) {
        state.story_query_sheet.previous_page();
    } else if keys.just_pressed(KeyCode::PageDown) {
        state.story_query_sheet.next_page();
    }
}

#[derive(Clone, Debug)]
struct StoryQueryDropdownChoice {
    value: usize,
    label: String,
}

fn story_query_dropdown_slot(kind: StoryQueryDropdownKind) -> StoryQuerySheetTextSlot {
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

fn story_query_control_visible(sheet: &StoryQuerySheet, slot: StoryQuerySheetTextSlot) -> bool {
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
        StoryQuerySheetTextSlot::Entity => {
            sheet.query_kind == StoryQueryKind::AppearancesOfEntity
        }
        StoryQuerySheetTextSlot::Category(slot_index) => {
            sheet.query_kind == StoryQueryKind::CategoriesInScene
                && slot_index < sheet.category_indices.len()
        }
        StoryQuerySheetTextSlot::AddCategory => story_query_can_add_category(sheet),
    }
}

fn story_query_dropdown_choices(
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
                sheet.category_indices
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

fn story_query_dropdown_current_value(
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
        StoryQueryDropdownKind::Category(slot_index) => sheet
            .category_indices
            .get(slot_index)
            .copied()
            .unwrap_or(0),
    }
}

fn story_query_dropdown_window_start(
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
    selected
        .saturating_sub(3)
        .min(choices.len().saturating_sub(STORY_QUERY_DROPDOWN_VISIBLE_OPTIONS))
}

fn apply_story_query_dropdown_choice(
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

fn step_open_story_query_dropdown(state: &mut EditorState, direction: isize) {
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
    state.story_query_sheet.page_index = 0;
    state.story_query_sheet.result_status = "Selection changed.".to_string();
}

impl StoryQuerySheet {
    fn page_label(&self) -> String {
        let format = match self.result_format {
            StoryQueryOutputFormat::Fountain => "Fountain",
            StoryQueryOutputFormat::Markdown => "Markdown",
        };
        format!(
            "{} page {} of {}",
            format,
            self.page_index.saturating_add(1),
            self.visual_pages.len().max(1)
        )
    }

    fn previous_page(&mut self) {
        self.page_index = self.page_index.saturating_sub(1);
    }

    fn next_page(&mut self) {
        if self.page_index + 1 < self.visual_pages.len() {
            self.page_index += 1;
        }
    }

    fn set_output(
        &mut self,
        output: StoryQueryRunOutput,
        dialogue_double_space_newline: bool,
        non_dialogue_double_space_newline: bool,
    ) {
        self.result_title = output.title;
        self.result_status = output.status;
        self.result_format = output.format;
        self.visual_pages = story_query_visual_pages(
            &output.text,
            self.result_format,
            dialogue_double_space_newline,
            non_dialogue_double_space_newline,
        );
        self.source_targets = output.source_targets;
        self.page_index = 0;
    }

    fn set_error(&mut self, title: &str, message: String) {
        self.set_output(StoryQueryRunOutput {
            title: title.to_string(),
            status: message.clone(),
            format: StoryQueryOutputFormat::Markdown,
            text: format!("# {title}\n\n{message}"),
            source_targets: Vec::new(),
        }, false, false);
    }
}

impl EditorState {
    fn open_story_query_sheet(&mut self) {
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

    fn refresh_story_query_sheet_options(&mut self) -> Result<(), String> {
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

    fn clamp_story_query_sheet_selection(&mut self) {
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
            self.story_query_sheet.character_b_index =
                (self.story_query_sheet.character_a_index + 1)
                    % self.story_query_sheet.characters.len();
        }
        clamp_story_query_index(
            &mut self.story_query_sheet.entity_index,
            self.story_query_sheet.entities.len(),
        );
        clamp_story_query_dependencies(&mut self.story_query_sheet);
    }

    fn run_story_query_sheet(&mut self) {
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
                self.story_query_sheet
                    .set_error("Story Query Sheet", format!("Story index unavailable: {error}"));
                return;
            }
        };

        let output = match self.story_query_sheet.query_kind {
            StoryQueryKind::DialogueByCharacter => {
                let Some(character) =
                    selected_character(&self.story_query_sheet, self.story_query_sheet.character_a_index)
                else {
                    self.story_query_sheet.set_error(
                        "Dialogue",
                        "No character entities are indexed.".to_string(),
                    );
                    return;
                };
                build_dialogue_by_character_output(&database, character)
            }
            StoryQueryKind::DialogueBetweenCharacters => {
                let Some(character_a) =
                    selected_character(&self.story_query_sheet, self.story_query_sheet.character_a_index)
                else {
                    self.story_query_sheet.set_error(
                        "Dialogue",
                        "No character entities are indexed.".to_string(),
                    );
                    return;
                };
                let Some(character_b) =
                    selected_character(&self.story_query_sheet, self.story_query_sheet.character_b_index)
                else {
                    self.story_query_sheet.set_error(
                        "Dialogue",
                        "Choose a second indexed character.".to_string(),
                    );
                    return;
                };
                build_dialogue_between_output(&database, character_a, character_b, None)
            }
            StoryQueryKind::DialogueBetweenAtScene => {
                let Some(character_a) =
                    selected_character(&self.story_query_sheet, self.story_query_sheet.character_a_index)
                else {
                    self.story_query_sheet.set_error(
                        "Dialogue",
                        "No character entities are indexed.".to_string(),
                    );
                    return;
                };
                let Some(character_b) =
                    selected_character(&self.story_query_sheet, self.story_query_sheet.character_b_index)
                else {
                    self.story_query_sheet.set_error(
                        "Dialogue",
                        "Choose a second indexed character.".to_string(),
                    );
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
                    self.story_query_sheet.set_error(
                        "Appearances",
                        "No entities are indexed.".to_string(),
                    );
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

    fn open_first_story_query_source(&mut self) {
        let Some(target) = self.story_query_sheet.source_targets.first().cloned() else {
            self.status_message = "No source target in current story sheet.".to_string();
            return;
        };

        self.load_from_path(target.path.clone());
        let line = target.line.min(self.document.line_count().saturating_sub(1));
        self.set_cursor(Position { line, column: 0 }, true);
        self.top_line = line.saturating_sub(3);
        self.processed_top_line = self.top_line;
        self.processed_top_visual = self.top_line;
        self.story_query_sheet.open = false;
        self.status_message = format!("Opened story query source at line {}.", line + 1);
    }

    fn open_story_query_link_target(&mut self, target: String) {
        match self.resolve_script_target_path(&target) {
            Ok(path) => {
                let metadata_warning = basscript_core::EntityDocument::load(&path).err();
                self.load_from_path(path.clone());
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

    fn selected_story_query_scene(
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

fn build_dialogue_by_character_output(
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

fn build_dialogue_between_output(
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
                scene.location_text
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
            scene.location_text
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

fn build_category_scenes_output(
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

fn write_category_scene_heading(
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

fn write_category_group(
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

fn human_story_query_role(role: &basscript_core::StoryIndexAppearanceRole) -> String {
    role.as_database_value().replace('_', " ")
}

fn build_appearances_output(
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

struct FountainDialogueExtract {
    text: String,
    source_targets: Vec<StoryQuerySourceTarget>,
}

fn fountain_dialogue_extract(
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

fn linked_character_target(parsed_line: &ParsedLine) -> Option<String> {
    let [link] = parsed_line.script_links.as_slice() else {
        return None;
    };
    Some(link.target.clone())
}

fn raw_fountain_query_line(parsed_line: &ParsedLine) -> String {
    parsed_line.raw.clone()
}

fn load_story_taxonomy() -> StoryTaxonomyLoad {
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

fn normalize_story_taxonomy_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn story_taxonomy_category_for_type(
    taxonomy: &StoryTaxonomyConfig,
    selected_category_indices: &[usize],
    entity_type: &str,
) -> Option<usize> {
    let entity_type = normalize_story_taxonomy_key(entity_type);
    selected_category_indices.iter().copied().find(|index| {
        taxonomy
            .categories
            .get(*index)
            .map(|category| category.types.iter().any(|candidate| candidate == &entity_type))
            .unwrap_or(false)
    })
}

fn selected_category_indices(sheet: &StoryQuerySheet) -> Vec<usize> {
    let mut selected = Vec::<usize>::new();
    for category_index in &sheet.category_indices {
        if *category_index < sheet.taxonomy.categories.len()
            && !selected.contains(category_index)
        {
            selected.push(*category_index);
        }
    }
    selected
}

fn selected_category_label(sheet: &StoryQuerySheet, index: usize) -> String {
    sheet
        .taxonomy
        .categories
        .get(index)
        .map(|category| category.display_label().to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn selected_character(
    sheet: &StoryQuerySheet,
    index: usize,
) -> Option<&StoryIndexEntityRecord> {
    sheet.characters.get(index)
}

fn selected_entity(sheet: &StoryQuerySheet, index: usize) -> Option<&StoryIndexEntityRecord> {
    sheet.entities.get(index)
}

fn selected_character_label(sheet: &StoryQuerySheet, index: usize) -> String {
    selected_character(sheet, index)
        .map(entity_label)
        .unwrap_or_else(|| "none".to_string())
}

fn selected_entity_label(sheet: &StoryQuerySheet, index: usize) -> String {
    selected_entity(sheet, index)
        .map(entity_label)
        .unwrap_or_else(|| "none".to_string())
}

fn selected_scene_label(state: &EditorState) -> String {
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

fn scene_label(scene: &StoryIndexSceneRecord) -> String {
    scene
        .location_text
        .as_ref()
        .map(|location| format!("{} ({})", scene.heading_text, location))
        .unwrap_or_else(|| scene.heading_text.clone())
}

fn entity_label(entity: &StoryIndexEntityRecord) -> String {
    format!("{} ({})", entity.name, entity.target)
}

fn compact_story_query_label(label: &str) -> String {
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

fn merge_story_query_entities(
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

fn trim_result_snippet(snippet: &str) -> String {
    let trimmed = snippet.trim();
    if trimmed.chars().count() <= 120 {
        return trimmed.to_string();
    }

    let mut out = trimmed.chars().take(117).collect::<String>();
    out.push_str("...");
    out
}

fn trim_category_result_snippet(snippet: &str) -> String {
    let trimmed = snippet.trim();
    if trimmed.chars().count() <= 72 {
        return trimmed.to_string();
    }

    let mut out = trimmed.chars().take(69).collect::<String>();
    out.push_str("...");
    out
}

fn story_query_document_format(format: StoryQueryOutputFormat) -> DocumentFormat {
    match format {
        StoryQueryOutputFormat::Fountain => DocumentFormat::Fountain,
        StoryQueryOutputFormat::Markdown => DocumentFormat::Markdown,
    }
}

fn story_query_visual_pages(
    text: &str,
    output_format: StoryQueryOutputFormat,
    dialogue_double_space_newline: bool,
    non_dialogue_double_space_newline: bool,
) -> Vec<Vec<ProcessedVisualLine>> {
    let document_format = story_query_document_format(output_format);
    let document = Document::from_text(text);
    let parsed = parse_document_with_format(&document, document_format);
    let wrap_columns = story_query_wrap_columns(document_format);
    let mut visual_lines = Vec::<ProcessedVisualLine>::new();

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

        visual_lines.extend(wrapped);
    }

    if visual_lines.is_empty() {
        visual_lines.push(story_query_blank_visual_line());
    }

    visual_lines
        .chunks(STORY_QUERY_PAGE_LINE_LIMIT)
        .map(|chunk| chunk.to_vec())
        .collect()
}

fn story_query_wrap_columns(document_format: DocumentFormat) -> usize {
    let text_width = (STORY_QUERY_A4_PAGE_WIDTH
        - (PAGE_TEXT_MARGIN_LEFT + PAGE_TEXT_MARGIN_RIGHT) * STORY_QUERY_A4_SCALE)
        .max(1.0);
    let char_width = (default_char_width_for_format(document_format) * STORY_QUERY_A4_SCALE)
        .max(0.1);
    ((text_width / char_width) + 1e-4).floor().max(1.0) as usize
}

fn story_query_should_split_on_double_space(
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

fn story_query_blank_visual_line() -> ProcessedVisualLine {
    ProcessedVisualLine {
        source_line: 0,
        text: " ".to_string(),
        fragments: vec![ProcessedVisualFragment {
            text: " ".to_string(),
            is_link: false,
            link_target: None,
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

fn story_query_link_color_for_target(
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

fn story_query_link_target_at_position(
    sheet: &StoryQuerySheet,
    layout: &TextLayoutInfo,
    inverse_scale: f32,
    local_x: f32,
    local_y: f32,
) -> Option<String> {
    let page = sheet.visual_pages.get(sheet.page_index)?;
    let document_format = story_query_document_format(sheet.result_format);
    let fallback_char_width =
        (default_char_width_for_format(document_format) * STORY_QUERY_A4_SCALE).max(1.0);
    let fallback_line = ((local_y / STORY_QUERY_RESULT_LINE_HEIGHT)
        .floor()
        .max(0.0) as usize)
        .min(STORY_QUERY_PAGE_LINE_LIMIT.saturating_sub(1));
    let line_offset = line_index_from_layout_y(
        layout,
        local_y,
        STORY_QUERY_PAGE_LINE_LIMIT,
        inverse_scale,
    )
    .unwrap_or(fallback_line)
    .min(STORY_QUERY_PAGE_LINE_LIMIT.saturating_sub(1));
    let visual_line = page.get(line_offset)?;
    let fallback_column = (local_x / fallback_char_width).round().max(0.0) as usize;
    let display_column = column_from_layout_x(
        layout,
        line_offset,
        local_x,
        &visual_line.text,
        inverse_scale,
        fallback_char_width,
    )
    .unwrap_or(fallback_column);

    story_query_link_target_at_column(visual_line, display_column)
}

fn story_query_link_target_at_column(
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

fn story_query_entity_link(label: &str, target: &str) -> String {
    if !basscript_core::is_valid_target_key(target) {
        return label.to_string();
    }
    let label = label
        .replace('[', "(")
        .replace(']', ")")
        .replace('\n', " ");
    format!("[{label}]({target})")
}

fn clamp_story_query_index(index: &mut usize, len: usize) {
    if len == 0 {
        *index = 0;
    } else {
        *index = (*index).min(len - 1);
    }
}

fn story_query_can_add_category(sheet: &StoryQuerySheet) -> bool {
    if sheet.query_kind != StoryQueryKind::CategoriesInScene {
        return false;
    }
    let category_count = sheet.taxonomy.categories.len();
    let max_rows = STORY_QUERY_MAX_CATEGORY_ROWS.min(category_count);
    sheet.category_indices.len() < max_rows
}

fn add_story_query_category(sheet: &mut StoryQuerySheet) {
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

fn clamp_story_query_categories(sheet: &mut StoryQuerySheet) {
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

fn clamp_story_query_dependencies(sheet: &mut StoryQuerySheet) {
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
