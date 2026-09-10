pub(crate) fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    state: Res<EditorState>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));

    let fonts = EditorFonts {
        regular: asset_server.load(FONT_PATH),
        bold: asset_server.load(FONT_BOLD_PATH),
        italic: asset_server.load(FONT_ITALIC_PATH),
        bold_italic: asset_server.load(FONT_BOLD_ITALIC_PATH),
        markdown_regular: load_font_with_fallback(
            &asset_server,
            FONT_MARKDOWN_PATH,
            FONT_ITALIC_PATH,
        ),
        markdown_bold: load_font_with_fallback(
            &asset_server,
            FONT_MARKDOWN_BOLD_PATH,
            FONT_BOLD_ITALIC_PATH,
        ),
        markdown_italic: load_font_with_fallback(
            &asset_server,
            FONT_MARKDOWN_ITALIC_PATH,
            FONT_ITALIC_PATH,
        ),
        markdown_bold_italic: load_font_with_fallback(
            &asset_server,
            FONT_MARKDOWN_BOLD_ITALIC_PATH,
            FONT_BOLD_ITALIC_PATH,
        ),
    };
    let workspace_icons = WorkspaceIcons {
        folder_closed: load_workspace_icon_image(&mut images, "assets/icons/folder-closed.svg", 16),
        folder_open: load_workspace_icon_image(&mut images, "assets/icons/folder-open.svg", 16),
    };
    let checklist_icons = ChecklistIcons {
        unchecked: load_workspace_icon_image(
            &mut images,
            "assets/icons/checklist-unchecked.svg",
            16,
        ),
        checked: load_workspace_icon_image(&mut images, "assets/icons/checklist-checked.svg", 16),
    };
    let theme_picker_assets = ThemePickerAssets {
        hue_sat_wheel: generate_theme_color_wheel_image(&mut images, THEME_COLOR_WHEEL_SIZE_PX),
    };
    let hue_sat_wheel = theme_picker_assets.hue_sat_wheel.clone();
    let font = fonts.regular.clone();
    commands.insert_resource(fonts);
    commands.insert_resource(workspace_icons);
    commands.insert_resource(checklist_icons);
    commands.insert_resource(theme_picker_assets);

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                overflow: window_surface_overflow(state.show_system_titlebar),
                border_radius: window_surface_border_radius(state.show_system_titlebar),
                ..default()
            },
            BackgroundColor(state.app_bg_color),
            WindowSurfaceRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                EditorScreenRoot,
                children![
                    (
                        Node {
                            width: percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            column_gap: px(20.0),
                            padding: UiRect::axes(px(16.0), px(10.0)),
                            ..default()
                        },
                        BackgroundColor(state.top_menu_bg_color),
                        TopMenuSection,
                        children![
                            (
                                Text::new("BasScript"),
                                Node {
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                TextFont {
                                    font: font.clone().into(),
                                    font_size: FontSize::Px(15.0),
                                    ..default()
                                },
                                ThemedText::Main,
                            ),
                            (
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    flex_wrap: FlexWrap::NoWrap,
                                    flex_grow: 1.0,
                                    flex_basis: px(0.0),
                                    min_width: px(0.0),
                                    justify_content: JustifyContent::End,
                                    column_gap: px(6.0),
                                    ..default()
                                },
                                ToolbarControls,
                                children![
                                    toolbar_button(
                                        font.clone(),
                                        "Open Folder",
                                        ToolbarAction::OpenWorkspace,
                                    ),
                                    toolbar_button(font.clone(), "Save", ToolbarAction::Save),
                                    toolbar_button(font.clone(), "Save As", ToolbarAction::SaveAs),
                                    toolbar_button(
                                        font.clone(),
                                        "Export PDF",
                                        ToolbarAction::ExportPdf,
                                    ),
                                    toolbar_button(
                                        font.clone(),
                                        "Story Sheet",
                                        ToolbarAction::StoryQuerySheet,
                                    ),
                                    toolbar_button(font.clone(), "Zoom -", ToolbarAction::ZoomOut),
                                    toolbar_button(font.clone(), "Zoom +", ToolbarAction::ZoomIn),
                                    toolbar_button(
                                        font.clone(),
                                        "Settings",
                                        ToolbarAction::Settings
                                    ),
                                ],
                            )
                        ],
                    ),
                    (
                        Node {
                            width: percent(100.0),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            flex_basis: px(0.0),
                            min_height: px(0.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: px(0.0),
                            padding: UiRect::all(px(0.0)),
                            ..default()
                        },
                        EditorBodyRow,
                        RelativeCursorPosition::default(),
                        children![
                            workspace_sidebar_bundle(font.clone(), state.explorer_bg_color),
                            panel_splitter_bundle(PanelSplitter::Workspace),
                            (
                                Node {
                                    flex_grow: 1.0,
                                    height: percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(0.0),
                                    ..default()
                                },
                                EditorPanelsContainer,
                                children![
                                    (
                                        Node {
                                            height: percent(100.0),
                                            ..default()
                                        },
                                        PanelPaneSlot {
                                            kind: PanelKind::Plain,
                                        },
                                        children![panel_bundle(font.clone(), hue_sat_wheel.clone(), PanelKind::Plain)],
                                    ),
                                    panel_splitter_bundle(PanelSplitter::Panels),
                                    (
                                        Node {
                                            height: percent(100.0),
                                            ..default()
                                        },
                                        PanelPaneSlot {
                                            kind: PanelKind::Processed,
                                        },
                                        children![panel_bundle(font.clone(), hue_sat_wheel.clone(), PanelKind::Processed)],
                                    ),
                                ],
                            ),
                            story_query_sheet_bundle(font.clone()),
                        ],
                    ),
                    status_line_bundle(font.clone(), state.app_bg_color, state.status_line_visible,)
                ],
            ));

            spawn_link_autocomplete_menu(root, font.clone());

            root.spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(state.app_bg_color),
                SettingsScreenRoot,
                children![(
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    RelativeCursorPosition::default(),
                    SettingsMenuScrollArea {
                        screen: UiScreenState::Settings,
                    },
                    children![(
                        Node {
                            width: percent(100.0),
                            max_width: px(760.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12.0),
                            padding: UiRect::axes(px(18.0), px(16.0)),
                            ..default()
                        },
                        SettingsMenuScrollContent {
                            screen: UiScreenState::Settings,
                        },
                        children![
                            settings_screen_heading(
                                font.clone(),
                                "Settings",
                                "Processed page margins and formatting options.",
                            ),
                            settings_toggle_button(
                                font.clone(),
                                SettingsAction::DialogueDoubleSpaceNewline,
                            ),
                            settings_toggle_button(
                                font.clone(),
                                SettingsAction::NonDialogueDoubleSpaceNewline,
                            ),
                            settings_toggle_button(
                                font.clone(),
                                SettingsAction::ToggleProcessedPagination,
                            ),
                            settings_toggle_button(
                                font.clone(),
                                SettingsAction::CycleDefaultViewMode,
                            ),
                            settings_toggle_button(font.clone(), SettingsAction::ToggleVimMode),
                            settings_toggle_button(
                                font.clone(),
                                SettingsAction::ShowSystemTitlebar
                            ),
                            margin_setting_row(
                                font.clone(),
                                "Left margin (pt)",
                                MarginEdge::Left,
                                SettingsAction::MarginLeftDecrease,
                                SettingsAction::MarginLeftIncrease,
                            ),
                            margin_setting_row(
                                font.clone(),
                                "Right margin (pt)",
                                MarginEdge::Right,
                                SettingsAction::MarginRightDecrease,
                                SettingsAction::MarginRightIncrease,
                            ),
                            margin_setting_row(
                                font.clone(),
                                "Top margin (pt)",
                                MarginEdge::Top,
                                SettingsAction::MarginTopDecrease,
                                SettingsAction::MarginTopIncrease,
                            ),
                            margin_setting_row(
                                font.clone(),
                                "Bottom margin (pt)",
                                MarginEdge::Bottom,
                                SettingsAction::MarginBottomDecrease,
                                SettingsAction::MarginBottomIncrease,
                            ),
                            settings_action_button(
                                font.clone(),
                                "Theme",
                                SettingsAction::OpenTheme
                            ),
                            settings_action_button(
                                font.clone(),
                                "Link colors",
                                SettingsAction::OpenLinkColors,
                            ),
                            settings_action_button(
                                font.clone(),
                                "Keybinds",
                                SettingsAction::OpenKeybinds
                            ),
                            settings_action_button(
                                font.clone(),
                                "Back to editor",
                                SettingsAction::BackToEditor
                            ),
                        ],
                    )],
                )],
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(state.app_bg_color),
                KeybindsScreenRoot,
                children![(
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    RelativeCursorPosition::default(),
                    SettingsMenuScrollArea {
                        screen: UiScreenState::Keybinds,
                    },
                    children![(
                        Node {
                            width: percent(100.0),
                            max_width: px(760.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8.0),
                            padding: UiRect::new(px(18.0), px(18.0), px(16.0), px(24.0)),
                            ..default()
                        },
                        SettingsMenuScrollContent {
                            screen: UiScreenState::Keybinds,
                        },
                        children![
                            settings_screen_heading(
                                font.clone(),
                                "Keybinds",
                                "Customize shortcuts and review editor controls.",
                            ),
                            (
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(8.0),
                                    margin: UiRect::bottom(px(8.0)),
                                    ..default()
                                },
                                children![
                                    settings_action_button(
                                        font.clone(),
                                        "Back to settings",
                                        SettingsAction::BackToSettings,
                                    ),
                                    settings_action_button(
                                        font.clone(),
                                        "Back to editor",
                                        SettingsAction::BackToEditor,
                                    ),
                                ],
                            ),
                            keybind_help_card(font.clone()),
                            keybind_section_heading(font.clone(), "Editable shortcuts"),
                            keybind_setting_row(font.clone(), ShortcutAction::NavigateForward),
                            keybind_setting_row(font.clone(), ShortcutAction::OpenWorkspace),
                            keybind_setting_row(font.clone(), ShortcutAction::Save),
                            keybind_setting_row(font.clone(), ShortcutAction::SaveAs),
                            keybind_setting_row(font.clone(), ShortcutAction::Undo),
                            keybind_setting_row(font.clone(), ShortcutAction::Redo),
                            keybind_setting_row(font.clone(), ShortcutAction::ZoomIn),
                            keybind_setting_row(font.clone(), ShortcutAction::ZoomOut),
                            keybind_setting_row(font.clone(), ShortcutAction::PlainView),
                            keybind_setting_row(font.clone(), ShortcutAction::ProcessedView),
                            keybind_setting_row(
                                font.clone(),
                                ShortcutAction::ProcessedRawCurrentLineView,
                            ),
                            keybind_setting_row(font.clone(), ShortcutAction::SplitView),
                            keybind_setting_row(font.clone(), ShortcutAction::ToggleExplorer),
                            keybind_setting_row(font.clone(), ShortcutAction::ToggleTopMenu),
                            keybind_setting_row(font.clone(), ShortcutAction::ToggleRightButtons),
                            keybind_section_heading(font.clone(), "Editor navigation"),
                            keybind_row(font.clone(), "Cmd/Ctrl + mouse wheel", "Zoom"),
                            keybind_row(font.clone(), "Arrow keys", "Move cursor"),
                            keybind_row(font.clone(), "Home / End", "Move to line start/end"),
                            keybind_row(font.clone(), "Page Up / Page Down", "Move by viewport"),
                            keybind_row(font.clone(), "Escape", "Cancel autoscroll"),
                            keybind_section_heading(font.clone(), "Explorer"),
                            keybind_row(font.clone(), "j / k", "Move selected row"),
                            keybind_row(font.clone(), "Enter / o / l", "Open selected row"),
                            keybind_row(font.clone(), "n", "New file or folder"),
                            keybind_row(font.clone(), "r", "Rename selected row"),
                            keybind_row(font.clone(), "d", "Delete selected row"),
                            keybind_row(font.clone(), "Shift+R", "Refresh workspace tree"),
                            keybind_row(font.clone(), "q / Esc", "Leave explorer focus"),
                            keybind_section_heading(font.clone(), "Vim mode"),
                            keybind_row(font.clone(), "Esc / i / a", "Normal / insert / append"),
                            keybind_row(font.clone(), "h j k l", "Move left / down / up / right"),
                            keybind_row(font.clone(), "v / V", "Visual / visual line mode"),
                            keybind_row(font.clone(), "yy / y", "Yank line / visual selection"),
                            keybind_row(font.clone(), "p", "Paste Vim register"),
                            keybind_row(font.clone(), "dd / d", "Delete line / visual selection"),
                            keybind_row(font.clone(), ":", "Open command menu"),
                            keybind_section_heading(font.clone(), "Mouse"),
                            keybind_row(
                                font.clone(),
                                "Mouse wheel",
                                "Scroll active pane vertically"
                            ),
                            keybind_row(
                                font.clone(),
                                "Shift + wheel",
                                "Scroll active pane horizontally"
                            ),
                            keybind_row(
                                font.clone(),
                                "Ctrl + left drag",
                                "Scroll text; move canvas card"
                            ),
                            keybind_row(font.clone(), "Space + left drag", "Pan canvas"),
                            keybind_row(
                                font.clone(),
                                "Middle drag",
                                "Pan canvas; autoscroll text panes"
                            ),
                            keybind_row(font.clone(), "Left click", "Place cursor"),
                            keybind_row(
                                font.clone(),
                                "Right click",
                                "Cancel middle-click autoscroll"
                            ),
                        ],
                    )],
                )],
            ));

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    width: px(40.0),
                    height: px(40.0),
                    display: Display::None,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    overflow: Overflow::visible(),
                    ..default()
                },
                ZIndex(48),
                MiddleAutoscrollIndicator,
                children![
                    (
                        Text::new("◯"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(34.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.10, 0.12, 0.15, 0.82)),
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(-2.0),
                            left: px(7.0),
                            ..default()
                        },
                    ),
                    (
                        Text::new("•"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.10, 0.12, 0.15, 0.95)),
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(8.0),
                            left: px(16.0),
                            ..default()
                        },
                    ),
                    (
                        Text::new("↑"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.96, 0.97, 0.99, 0.95)),
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(-14.0),
                            left: px(15.0),
                            ..default()
                        },
                    ),
                    (
                        Text::new("↓"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.96, 0.97, 0.99, 0.95)),
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(42.0),
                            left: px(15.0),
                            ..default()
                        },
                    ),
                    (
                        Text::new("←"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.96, 0.97, 0.99, 0.95)),
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(14.0),
                            left: px(-12.0),
                            ..default()
                        },
                    ),
                    (
                        Text::new("→"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.96, 0.97, 0.99, 0.95)),
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(14.0),
                            left: px(43.0),
                            ..default()
                        },
                    ),
                ],
            ));

            root.spawn(workspace_prompt_bundle(font.clone()));
            root.spawn(command_menu_bundle(font.clone()));
        });
}

pub(crate) fn load_workspace_icon_image(
    images: &mut Assets<Image>,
    icon_path: &str,
    max_side_px: u32,
) -> Handle<Image> {
    match rasterize_svg_to_image(images, icon_path, max_side_px) {
        Ok(handle) => handle,
        Err(error) => {
            warn!("Failed to rasterize {icon_path}: {error}");
            images.add(Image::new_fill(
                bevy::render::render_resource::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                &[255, 255, 255, 0],
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                bevy::asset::RenderAssetUsages::default(),
            ))
        }
    }
}

pub(crate) fn generate_theme_color_wheel_image(
    images: &mut Assets<Image>,
    diameter: u32,
) -> Handle<Image> {
    let size = diameter.max(2);
    let half = size as f32 * 0.5;
    let mut data = vec![0_u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let fx = ((x as f32 + 0.5) - half) / half;
            let fy = (half - (y as f32 + 0.5)) / half;
            let radius = (fx * fx + fy * fy).sqrt();
            let pixel = ((y * size + x) * 4) as usize;
            if radius > 1.0 {
                data[pixel + 0] = 0;
                data[pixel + 1] = 0;
                data[pixel + 2] = 0;
                data[pixel + 3] = 0;
                continue;
            }

            let mut hue = fy.atan2(fx) / std::f32::consts::TAU;
            if hue < 0.0 {
                hue += 1.0;
            }
            let saturation = radius.clamp(0.0, 1.0);
            let rgb = hsv_to_rgb(hue, saturation, 1.0);
            data[pixel + 0] = (rgb.x * 255.0).round().clamp(0.0, 255.0) as u8;
            data[pixel + 1] = (rgb.y * 255.0).round().clamp(0.0, 255.0) as u8;
            data[pixel + 2] = (rgb.z * 255.0).round().clamp(0.0, 255.0) as u8;
            data[pixel + 3] = 255;
        }
    }

    images.add(Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    ))
}

pub(crate) fn load_font_with_fallback(
    asset_server: &AssetServer,
    preferred_path: &str,
    fallback_path: &str,
) -> Handle<Font> {
    if resolve_workspace_asset_path(preferred_path).is_some() {
        return asset_server.load(preferred_path.to_owned());
    }

    warn!(
        "Preferred font {} not found. Falling back to {}.",
        preferred_path, fallback_path
    );
    asset_server.load(fallback_path.to_owned())
}

pub(crate) fn rasterize_svg_to_image(
    images: &mut Assets<Image>,
    icon_path: &str,
    max_side_px: u32,
) -> Result<Handle<Image>, String> {
    let icon_fs_path = resolve_workspace_asset_path(icon_path)
        .ok_or_else(|| format!("cannot resolve icon path {}", icon_path))?;
    let svg_data = fs::read(&icon_fs_path)
        .map_err(|error| format!("cannot read icon file {}: {error}", icon_fs_path.display()))?;
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &options)
        .map_err(|error| format!("invalid svg {}: {error}", icon_fs_path.display()))?;
    let source_size = tree.size().to_int_size();
    let source_width = source_size.width().max(1);
    let source_height = source_size.height().max(1);
    let scale = max_side_px.max(1) as f32 / source_width.max(source_height) as f32;
    let target_width = (source_width as f32 * scale).round().max(1.0) as u32;
    let target_height = (source_height as f32 * scale).round().max(1.0) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_width, target_height)
        .ok_or_else(|| "failed to allocate icon pixmap".to_string())?;
    let transform = resvg::tiny_skia::Transform::from_scale(
        target_width as f32 / source_width as f32,
        target_height as f32 / source_height as f32,
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: target_width,
            height: target_height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        pixmap.take(),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    Ok(images.add(image))
}

pub(crate) fn resolve_workspace_asset_path(path: &str) -> Option<PathBuf> {
    let direct = PathBuf::from(path);
    if direct.exists() {
        return Some(direct);
    }

    let from_ui_crate = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path);
    if from_ui_crate.exists() {
        return Some(from_ui_crate);
    }

    None
}

pub(crate) fn setup_processed_papers(
    mut commands: Commands,
    canvas_query: Query<(Entity, &PanelCanvas)>,
    paper_query: Query<(Entity, &PanelPaper)>,
    text_query: Query<(Entity, &PanelText)>,
    fonts: Res<EditorFonts>,
    checklist_icons: Res<ChecklistIcons>,
) {
    let regular_font = fonts.regular.clone();
    let unchecked_icon = checklist_icons.unchecked.clone();
    let span_capacity = processed_page_step_lines().max(1);

    for (entity, panel_canvas) in canvas_query.iter() {
        if panel_canvas.kind != PanelKind::Processed {
            continue;
        }

        let regular_font = regular_font.clone();
        let unchecked_icon = unchecked_icon.clone();
        commands.entity(entity).with_children(|parent| {
            spawn_markdown_metadata_controls(parent, regular_font.clone());

            for slot in 1..PROCESSED_PAPER_CAPACITY {
                let slot_font = regular_font.clone();
                let slot_unchecked_icon = unchecked_icon.clone();
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
                        PanelPaper {
                            kind: PanelKind::Processed,
                            slot,
                        },
                    ))
                    .with_children(|paper| {
                        for line_offset in 0..span_capacity {
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
                                        height: px(LINE_HEIGHT),
                                        overflow: Overflow::visible(),
                                        ..default()
                                    },
                                    UiTransform::default(),
                                    ZIndex(1),
                                    GlobalZIndex(1),
                                    ProcessedPaperText { slot, line_offset },
                                ))
                                .with_children(|text_parent| {
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
                                            ProcessedPaperLineSpan {
                                                slot,
                                                line_offset,
                                                part_index,
                                            },
                                        ));
                                    }
                                });
                        }

                        for line_offset in 0..span_capacity {
                            paper.spawn((
                                ImageNode::solid_color(COLOR_IMAGE_PLACEHOLDER),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(PAGE_TEXT_MARGIN_LEFT),
                                    top: px(PAGE_TEXT_MARGIN_TOP + line_offset as f32 * LINE_HEIGHT),
                                    width: px(10.0),
                                    height: px(10.0),
                                    ..default()
                                },
                                Visibility::Hidden,
                                ZIndex(0),
                                GlobalZIndex(1),
                                ProcessedImageBlockNode { slot, line_offset },
                            ));

                            paper.spawn((
                                ImageNode::new(slot_unchecked_icon.clone()),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(PAGE_TEXT_MARGIN_LEFT),
                                    top: px(PAGE_TEXT_MARGIN_TOP + line_offset as f32 * LINE_HEIGHT),
                                    width: px(10.0),
                                    height: px(10.0),
                                    ..default()
                                },
                                Visibility::Hidden,
                                ZIndex(3),
                                GlobalZIndex(1),
                                ProcessedChecklistIcon { slot, line_offset },
                            ));
                        }
                    });
            }
        });
    }

    for (entity, panel_paper) in paper_query.iter() {
        if panel_paper.kind != PanelKind::Processed || panel_paper.slot != 0 {
            continue;
        }

        let slot = panel_paper.slot;
        let regular_font = regular_font.clone();
        let unchecked_icon = unchecked_icon.clone();
        commands.entity(entity).with_children(|paper| {
            for line_offset in 0..span_capacity {
                paper
                    .spawn((
                        Text::new(""),
                        TextLayout::no_wrap(),
                        TextFont {
                            font: regular_font.clone().into(),
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
                            height: px(LINE_HEIGHT),
                            overflow: Overflow::visible(),
                            ..default()
                        },
                        UiTransform::default(),
                        ZIndex(1),
                        GlobalZIndex(1),
                        ProcessedPaperText { slot, line_offset },
                    ))
                    .with_children(|text_parent| {
                        for part_index in 0..PROCESSED_LINE_SPAN_PARTS {
                            text_parent.spawn((
                                TextSpan::new(""),
                                TextFont {
                                    font: regular_font.clone().into(),
                                    font_size: FontSize::Px(FONT_SIZE),
                                    ..default()
                                },
                                LineHeight::Px(LINE_HEIGHT),
                                TextColor(COLOR_ACTION),
                                ProcessedPaperLineSpan {
                                    slot,
                                    line_offset,
                                    part_index,
                                },
                            ));
                        }
                    });
            }

            for line_offset in 0..span_capacity {
                paper.spawn((
                    ImageNode::solid_color(COLOR_IMAGE_PLACEHOLDER),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(PAGE_TEXT_MARGIN_LEFT),
                        top: px(PAGE_TEXT_MARGIN_TOP + line_offset as f32 * LINE_HEIGHT),
                        width: px(10.0),
                        height: px(10.0),
                        ..default()
                    },
                    Visibility::Hidden,
                    ZIndex(0),
                    GlobalZIndex(1),
                    ProcessedImageBlockNode { slot, line_offset },
                ));

                paper.spawn((
                    ImageNode::new(unchecked_icon.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(PAGE_TEXT_MARGIN_LEFT),
                        top: px(PAGE_TEXT_MARGIN_TOP + line_offset as f32 * LINE_HEIGHT),
                        width: px(10.0),
                        height: px(10.0),
                        ..default()
                    },
                    Visibility::Hidden,
                    ZIndex(3),
                    GlobalZIndex(1),
                    ProcessedChecklistIcon { slot, line_offset },
                ));
            }
        });
    }

    for (entity, panel_text) in text_query.iter() {
        if panel_text.kind == PanelKind::Processed {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub(crate) struct ToolbarControls;

#[derive(Component)]
pub(crate) struct ToolbarButtonLabel {
    action: ToolbarAction,
    full: String,
}

pub(crate) fn toolbar_button(
    font: Handle<Font>,
    label: &str,
    action: ToolbarAction,
) -> impl Bundle {
    (
        Button,
        action,
        Node {
            min_height: px(32.0),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(12.0), px(7.0)),
            border_radius: BorderRadius::all(px(UI_CONTROL_CORNER_RADIUS)),
            ..default()
        },
        ThemedButton,
        children![(
            Text::new(label),
            ToolbarButtonLabel {
                action,
                full: label.to_owned(),
            },
            TextLayout::no_wrap(),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(12.5),
                ..default()
            },
            ThemedText::Main,
        )],
    )
}

pub(crate) fn settings_toggle_button(font: Handle<Font>, action: SettingsAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            min_height: px(32.0),
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(12.0), px(7.0)),
            border_radius: BorderRadius::all(px(UI_CONTROL_CORNER_RADIUS)),
            ..default()
        },
        ThemedButton,
        children![(
            Text::new(""),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(13.0),
                ..default()
            },
            ThemedText::Main,
            SettingToggleLabel { action },
        )],
    )
}


pub(crate) fn settings_action_button(
    font: Handle<Font>,
    label: &str,
    action: SettingsAction,
) -> impl Bundle {
    (
        Button,
        action,
        Node {
            min_height: px(32.0),
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(12.0), px(7.0)),
            border_radius: BorderRadius::all(px(UI_CONTROL_CORNER_RADIUS)),
            ..default()
        },
        ThemedButton,
        children![(
            Text::new(label),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(13.0),
                ..default()
            },
            ThemedText::Main,
        )],
    )
}

pub(crate) fn keybind_setting_row(font: Handle<Font>, action: ShortcutAction) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(10.0),
            padding: UiRect::axes(px(10.0), px(7.0)),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        ThemedBackground(UiColor::PanelBackground),
        children![
            (
                Text::new(shortcut_action_description(action)),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                ThemedText::Muted,
                Node {
                    flex_grow: 1.0,
                    min_width: px(180.0),
                    ..default()
                },
            ),
            (
                Button,
                KeybindRebindButton { action },
                Node {
                    padding: UiRect::axes(px(12.0), px(6.0)),
                    min_width: px(170.0),
                    ..default()
                },
                ThemedButton,
                children![(
                    Text::new(""),
                    TextFont {
                        font: font.into(),
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    ThemedText::Main,
                    KeybindBindingLabel { action },
                )],
            ),
        ],
    )
}

pub(crate) fn settings_screen_heading(
    font: Handle<Font>,
    title: &str,
    description: &str,
) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            margin: UiRect::bottom(px(6.0)),
            ..default()
        },
        children![
            (
                Text::new(title),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                ThemedText::Main,
            ),
            (
                Text::new(description),
                TextFont {
                    font: font.into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                ThemedText::Muted,
            ),
        ],
    )
}

pub(crate) fn keybind_row(font: Handle<Font>, binding: &str, description: &str) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            padding: UiRect::axes(px(10.0), px(4.0)),
            ..default()
        },
        children![
            (
                Text::new(binding),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                ThemedText::Main,
                Node {
                    width: px(210.0),
                    ..default()
                },
            ),
            (
                Text::new(description),
                TextFont {
                    font: font.into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                ThemedText::Muted,
            ),
        ],
    )
}

pub(crate) fn keybind_section_heading(font: Handle<Font>, label: &str) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(16.0),
            ..default()
        },
        ThemedText::Main,
        Node {
            width: percent(100.0),
            margin: UiRect::new(px(0.0), px(0.0), px(12.0), px(2.0)),
            ..default()
        },
    )
}

pub(crate) fn keybind_help_card(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            padding: UiRect::all(px(12.0)),
            margin: UiRect::bottom(px(8.0)),
            border_radius: BorderRadius::all(px(5.0)),
            ..default()
        },
        ThemedBackground(UiColor::PanelBackground),
        children![
            (
                Text::new("Changing a shortcut"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                ThemedText::Main,
            ),
            (
                Text::new(
                    "Click a binding, then press a key. Hold Ctrl, Alt, Super/Cmd, or Space to choose its modifier. Esc cancels capture or closes this screen."
                ),
                TextFont {
                    font: font.into(),
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                ThemedText::Muted,
            ),
        ],
    )
}

pub(crate) fn margin_setting_row(
    font: Handle<Font>,
    label: &str,
    edge: MarginEdge,
    decrease_action: SettingsAction,
    increase_action: SettingsAction,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8.0),
            ..default()
        },
        children![
            (
                Text::new(label),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                ThemedText::Main,
            ),
            settings_action_button(font.clone(), "-", decrease_action),
            (
                Text::new(""),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                ThemedText::Main,
                SettingMarginLabel { edge },
            ),
            settings_action_button(font, "+", increase_action),
        ],
    )
}

pub(crate) fn theme_overlay_tab_button(
    font: Handle<Font>,
    label: &str,
    category: ThemeCategory,
) -> impl Bundle {
    (
        Button,
        ThemeOverlayTabButton(category),
        Node {
            flex_grow: 1.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(6.0), px(4.0)),
            border_radius: BorderRadius::all(px(3.0)),
            ..default()
        },
        ThemedButton,
        children![(
            Text::new(label),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            ThemedText::Main,
        )],
    )
}

pub(crate) fn theme_preset_button(font: Handle<Font>, name: &str) -> impl Bundle {
    (
        Button,
        ThemePresetButton(name.to_string()),
        Node {
            padding: UiRect::axes(px(8.0), px(4.0)),
            border_radius: BorderRadius::all(px(3.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ThemedButton,
        children![(
            Text::new(name),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            ThemedText::Main,
        )],
    )
}

pub(crate) fn theme_color_row(font: Handle<Font>, target: ThemeColorTarget) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::axes(px(4.0), px(2.0)),
            ..default()
        },
        ThemeColorRow { target },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8.0),
                    flex_grow: 1.0,
                    ..default()
                },
                children![
                    (
                        Node {
                            width: px(14.0),
                            height: px(14.0),
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(2.0)),
                            ..default()
                        },
                        ThemedBorder,
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ThemeColorPreviewSwatch { target },
                    ),
                    (
                        Text::new(""),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        ThemedText::Main,
                        ThemeColorNameLabel { target },
                    ),
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(6.0),
                    ..default()
                },
                children![
                    (
                        Text::new(""),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        ThemedText::Muted,
                        ThemeColorValueLabel { target },
                    ),
                    (
                        Button,
                        ThemeColorPickerButton { target },
                        Node {
                            padding: UiRect::axes(px(8.0), px(3.0)),
                            border_radius: BorderRadius::all(px(3.0)),
                            ..default()
                        },
                        ThemedButton,
                        children![(
                            Text::new("Pick"),
                            TextFont {
                                font: font.into(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            ThemedText::Main,
                        )],
                    ),
                ],
            ),
        ],
    )
}

pub(crate) fn theme_link_hover_setting_row(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::axes(px(4.0), px(2.0)),
            ..default()
        },
        ThemeLinkHoverSettingRow,
        children![
            (
                Text::new("Hover HSV adjustment"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                ThemedText::Main,
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(6.0),
                    ..default()
                },
                children![
                    (
                        Button,
                        SettingsAction::LinkHoverHsvValueDecrease,
                        Node {
                            padding: UiRect::axes(px(7.0), px(2.0)),
                            border_radius: BorderRadius::all(px(3.0)),
                            ..default()
                        },
                        ThemedButton,
                        children![(
                            Text::new("-"),
                            TextFont {
                                font: font.clone().into(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            ThemedText::Main,
                        )],
                    ),
                    (
                        Text::new(""),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        ThemedText::Main,
                        ThemeLinkHoverValueLabel,
                    ),
                    (
                        Button,
                        SettingsAction::LinkHoverHsvValueIncrease,
                        Node {
                            padding: UiRect::axes(px(7.0), px(2.0)),
                            border_radius: BorderRadius::all(px(3.0)),
                            ..default()
                        },
                        ThemedButton,
                        children![(
                            Text::new("+"),
                            TextFont {
                                font: font.into(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            ThemedText::Main,
                        )],
                    ),
                ],
            ),
        ],
    )
}

pub(crate) fn format_hsv_value_adjustment_label(value: f32) -> String {
    format!("{:+.1}%", value * 100.0)
}

pub(crate) fn theme_glass_toggle_button(font: Handle<Font>, action: SettingsAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            width: percent(100.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(8.0), px(5.0)),
            border_radius: BorderRadius::all(px(3.0)),
            ..default()
        },
        ThemedButton,
        children![(
            Text::new(""),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(12.0),
                ..default()
            },
            ThemedText::Main,
            SettingToggleLabel { action },
        )],
    )
}

pub(crate) fn theme_visual_picker(font: Handle<Font>, hue_sat_wheel: Handle<Image>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Start,
            column_gap: px(8.0),
            ..default()
        },
        children![
            (
                Node {
                    width: px(THEME_OVERLAY_WHEEL_SIZE),
                    height: px(THEME_OVERLAY_WHEEL_SIZE),
                    position_type: PositionType::Relative,
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::new(hue_sat_wheel),
                RelativeCursorPosition::default(),
                ThemeHueSatWheel,
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(10.0),
                        height: px(10.0),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::MAX,
                        left: px((THEME_OVERLAY_WHEEL_SIZE - 10.0) * 0.5),
                        top: px((THEME_OVERLAY_WHEEL_SIZE - 10.0) * 0.5),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    BorderColor::all(Color::srgb(1.0, 1.0, 1.0)),
                    ThemeHueSatCursor,
                )],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                children![
                    theme_color_slider_row(font.clone(), "Hue", ThemeSliderChannel::Hue),
                    theme_color_slider_row(font.clone(), "Sat", ThemeSliderChannel::Saturation,),
                    theme_color_slider_row(font.clone(), "Val", ThemeSliderChannel::Value),
                    theme_color_slider_row(font.clone(), "Red", ThemeSliderChannel::Red),
                    theme_color_slider_row(font.clone(), "Grn", ThemeSliderChannel::Green),
                    theme_color_slider_row(font.clone(), "Blu", ThemeSliderChannel::Blue),
                    theme_color_slider_row(font.clone(), "Alf", ThemeSliderChannel::Alpha),
                ],
            )
        ],
    )
}

pub(crate) fn theme_color_slider_row(
    font: Handle<Font>,
    label: &str,
    channel: ThemeSliderChannel,
) -> impl Bundle {
    (
        ThemeColorSliderRow(channel),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(6.0),
            ..default()
        },
        children![
            (
                Text::new(label),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                ThemedText::Main,
                Node {
                    width: px(26.0),
                    ..default()
                },
            ),
            (
                Node {
                    width: px(THEME_OVERLAY_SLIDER_WIDTH),
                    height: px(THEME_COLOR_SLIDER_HEIGHT),
                    position_type: PositionType::Relative,
                    flex_shrink: 0.0,
                    ..default()
                },
                RelativeCursorPosition::default(),
                BackgroundColor(Color::srgb(0.45, 0.46, 0.48)),
                ThemeColorSlider { channel },
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: px(THEME_COLOR_SLIDER_KNOB_WIDTH),
                        height: px(THEME_COLOR_SLIDER_HEIGHT),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.94, 0.95, 0.97)),
                    ThemeColorSliderKnob { channel },
                )],
            ),
            (
                Text::new(""),
                TextFont {
                    font: font.into(),
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                ThemedText::Main,
                ThemeColorLabel {
                    channel: match channel {
                        ThemeSliderChannel::Hue => ThemeColorChannel::Hue,
                        ThemeSliderChannel::Saturation => ThemeColorChannel::Saturation,
                        ThemeSliderChannel::Red => ThemeColorChannel::Red,
                        ThemeSliderChannel::Green => ThemeColorChannel::Green,
                        ThemeSliderChannel::Blue => ThemeColorChannel::Blue,
                        ThemeSliderChannel::Value => ThemeColorChannel::Value,
                        ThemeSliderChannel::Alpha => ThemeColorChannel::Alpha,
                    },
                },
                Node {
                    width: px(34.0),
                    ..default()
                },
            ),
        ],
    )
}

pub(crate) fn theme_overlay_container_bundle(
    font: Handle<Font>,
    hue_sat_wheel: Handle<Image>,
) -> impl Bundle {
    (
        Node {
            width: px(380.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            padding: UiRect::all(px(10.0)),
            border_radius: BorderRadius::all(px(6.0)),
            border: UiRect::all(px(1.0)),
            ..default()
        },
        ThemedBorder,
        ThemedBackground(UiColor::PanelBackground),
        RelativeCursorPosition::default(),
        ThemeOverlayContainer,
        OpaqueThemedBackground,
        // Processed text and metadata use global layers; the popup must sit
        // above those layers as well as above its local canvas siblings.
        GlobalZIndex(25),
        children![
            (
                Node {
                    width: percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Text::new("Theme Options"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        ThemedText::Main,
                    ),
                    (
                        Button,
                        ThemeOverlayOkButton,
                        Node {
                            padding: UiRect::axes(px(12.0), px(4.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ThemedButton,
                        children![(
                            Text::new("OK"),
                            TextFont {
                                font: font.clone().into(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            ThemedText::Main,
                        )],
                    ),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(6.0), px(4.0)),
                    border_radius: BorderRadius::all(px(4.0)),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                ThemedBorder,
                ThemedBackground(UiColor::PanelBackground),
                children![
                    (
                        Button,
                        ThemeSwitchPrevButton,
                        Node {
                            padding: UiRect::axes(px(8.0), px(3.0)),
                            border_radius: BorderRadius::all(px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ThemedButton,
                        children![(
                            Text::new("< Prev"),
                            TextFont {
                                font: font.clone().into(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            ThemedText::Main,
                        )],
                    ),
                    (
                        Text::new("Theme: Default"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        ThemedText::Main,
                        ThemeCurrentNameLabel,
                    ),
                    (
                        Button,
                        ThemeSwitchNextButton,
                        Node {
                            padding: UiRect::axes(px(8.0), px(3.0)),
                            border_radius: BorderRadius::all(px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ThemedButton,
                        children![(
                            Text::new("Next >"),
                            TextFont {
                                font: font.clone().into(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            ThemedText::Main,
                        )],
                    ),
                    (
                        Button,
                        ThemeSaveButton,
                        Node {
                            padding: UiRect::axes(px(10.0), px(3.0)),
                            border_radius: BorderRadius::all(px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ThemedButton,
                        children![(
                            Text::new("Save"),
                            TextFont {
                                font: font.clone().into(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            ThemedText::Main,
                        )],
                    ),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4.0),
                    ..default()
                },
                children![
                    theme_overlay_tab_button(font.clone(), "Colors", ThemeCategory::Theme),
                    theme_overlay_tab_button(font.clone(), "UI", ThemeCategory::Controls),
                    theme_overlay_tab_button(font.clone(), "Text", ThemeCategory::Text),
                    theme_overlay_tab_button(font.clone(), "Links", ThemeCategory::Links),
                    theme_overlay_tab_button(font.clone(), "Glass", ThemeCategory::Glass),
                    theme_overlay_tab_button(font.clone(), "Themes", ThemeCategory::Themes),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    ..default()
                },
                ThemeCategorySection(ThemeCategory::Theme),
                children![
                    theme_color_row(font.clone(), ThemeColorTarget::AppBackground),
                    theme_color_row(font.clone(), ThemeColorTarget::TopMenuBackground),
                    theme_color_row(font.clone(), ThemeColorTarget::ExplorerBackground),
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::PlainBackground)),
                    theme_color_row(font.clone(), ThemeColorTarget::ProcessedBackground),
                    theme_color_row(font.clone(), ThemeColorTarget::PaperBackground),
                    theme_color_row(font.clone(), ThemeColorTarget::SelectionBackground),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    ..default()
                },
                ThemeCategorySection(ThemeCategory::Controls),
                children![
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::PanelBackground)),
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::InputBackground)),
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::ButtonBackground)),
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::ButtonHover)),
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::ButtonPressed)),
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::ActiveBackground)),
                    theme_color_row(font.clone(), ThemeColorTarget::Ui(UiColor::Border)),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    ..default()
                },
                ThemeCategorySection(ThemeCategory::Text),
                children![
                    theme_color_row(font.clone(), ThemeColorTarget::TextMain),
                    theme_color_row(font.clone(), ThemeColorTarget::TextMuted),
                    theme_color_row(font.clone(), ThemeColorTarget::TextSceneHeading),
                    theme_color_row(font.clone(), ThemeColorTarget::TextAction),
                    theme_color_row(font.clone(), ThemeColorTarget::TextCharacter),
                    theme_color_row(font.clone(), ThemeColorTarget::TextDialogue),
                    theme_color_row(font.clone(), ThemeColorTarget::TextParenthetical),
                    theme_color_row(font.clone(), ThemeColorTarget::TextTransition),
                    theme_color_row(font.clone(), ThemeColorTarget::TextMarkdownHeading),
                    theme_color_row(font.clone(), ThemeColorTarget::TextMarkdownQuote),
                    theme_color_row(font.clone(), ThemeColorTarget::TextMarkdownCode),
                    theme_color_row(font.clone(), ThemeColorTarget::TextMarkdownRule),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    ..default()
                },
                ThemeCategorySection(ThemeCategory::Links),
                children![
                    theme_color_row(font.clone(), ThemeColorTarget::LinkFallback),
                    theme_color_row(font.clone(), ThemeColorTarget::LinkCharacter),
                    theme_color_row(font.clone(), ThemeColorTarget::LinkPlace),
                    theme_color_row(font.clone(), ThemeColorTarget::LinkProp),
                    theme_color_row(font.clone(), ThemeColorTarget::LinkFaction),
                    theme_color_row(font.clone(), ThemeColorTarget::LinkConcept),
                    theme_link_hover_setting_row(font.clone()),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    ..default()
                },
                ThemeCategorySection(ThemeCategory::Glass),
                children![
                    theme_glass_toggle_button(font.clone(), SettingsAction::ToggleProcessedGlass),
                    theme_glass_toggle_button(font.clone(), SettingsAction::ToggleExplorerGlass),
                    theme_glass_toggle_button(font.clone(), SettingsAction::ToggleSettingsGlass),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8.0),
                    padding: UiRect::all(px(4.0)),
                    ..default()
                },
                ThemeCategorySection(ThemeCategory::Themes),
                children![
                    (
                        Text::new("Preset Themes:"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        ThemedText::Main,
                    ),
                    (
                        Node {
                            width: percent(100.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6.0),
                            ..default()
                        },
                        children![
                            theme_preset_button(font.clone(), "Default"),
                            theme_preset_button(font.clone(), "Classic"),
                            theme_preset_button(font.clone(), "Dark"),
                        ],
                    ),
                    (
                        Text::new("Save Current Theme As:"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        ThemedText::Main,
                    ),
                    (
                        Node {
                            width: percent(100.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Node {
                                    flex_grow: 1.0,
                                    padding: UiRect::axes(px(8.0), px(5.0)),
                                    border_radius: BorderRadius::all(px(3.0)),
                                    border: UiRect::all(px(1.0)),
                                    ..default()
                                },
                                ThemedBorder,
                                ThemedBackground(UiColor::InputBackground),
                                children![(
                                    Text::new(""),
                                    TextFont {
                                        font: font.clone().into(),
                                        font_size: FontSize::Px(11.0),
                                        ..default()
                                    },
                                    ThemedText::Main,
                                    ThemeNameInputText,
                                )],
                            ),
                            (
                                Button,
                                ThemeSaveNewButton,
                                Node {
                                    padding: UiRect::axes(px(10.0), px(5.0)),
                                    border_radius: BorderRadius::all(px(3.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ThemedButton,
                                children![(
                                    Text::new("Save As New"),
                                    TextFont {
                                        font: font.clone().into(),
                                        font_size: FontSize::Px(11.0),
                                        ..default()
                                    },
                                    ThemedText::Main,
                                )],
                            ),
                        ],
                    ),
                    (
                        Text::new("Tip: Type a name and click Save As New. Themes are stored in settings/themes/."),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        ThemedText::Muted,
                    ),
                ],
            ),
            (
                Node {
                    width: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6.0),
                    padding: UiRect::all(px(8.0)),
                    border_radius: BorderRadius::all(px(4.0)),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                ThemedBorder,
                ThemedBackground(UiColor::PanelBackground),
                ThemeColorPickerPanel,
                children![
                    (
                        Text::new(""),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        ThemedText::Main,
                        ThemeScreenTitleLabel,
                    ),
                    theme_visual_picker(font.clone(), hue_sat_wheel),
                    (
                        Node {
                            width: percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        children![
                            (
                                Text::new(""),
                                TextFont {
                                    font: font.clone().into(),
                                    font_size: FontSize::Px(11.0),
                                    ..default()
                                },
                                ThemedText::Muted,
                                ThemeSelectionRgbLabel,
                            ),
                            (
                                Text::new(""),
                                TextFont {
                                    font: font.clone().into(),
                                    font_size: FontSize::Px(11.0),
                                    ..default()
                                },
                                ThemedText::Muted,
                                ThemeSelectionHsvLabel,
                            ),
                            (
                                Text::new(""),
                                TextFont {
                                    font: font.clone().into(),
                                    font_size: FontSize::Px(11.0),
                                    ..default()
                                },
                                ThemedText::Main,
                                ThemeSelectionHexLabel,
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

pub(crate) fn handle_theme_overlay_buttons(
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    ok_query: Query<&Interaction, (Changed<Interaction>, With<ThemeOverlayOkButton>)>,
    tab_query: Query<(&Interaction, &ThemeOverlayTabButton), (Changed<Interaction>, With<Button>)>,
    prev_query: Query<&Interaction, (Changed<Interaction>, With<ThemeSwitchPrevButton>)>,
    next_query: Query<&Interaction, (Changed<Interaction>, With<ThemeSwitchNextButton>)>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<ThemeSaveButton>)>,
    save_new_query: Query<&Interaction, (Changed<Interaction>, With<ThemeSaveNewButton>)>,
    preset_query: Query<(&Interaction, &ThemePresetButton), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<EditorState>,
) {
    if keys.just_pressed(KeyCode::Escape) && state.theme_overlay_open {
        state.theme_overlay_open = false;
        state.theme_color_picker_open = false;
        let theme = theme_settings_from_state(&state);
        let _ = save_theme_settings(&theme);
        state.status_message = "Theme closed.".to_string();
        return;
    }

    for interaction in ok_query.iter() {
        if *interaction == Interaction::Pressed {
            state.theme_overlay_open = false;
            state.theme_color_picker_open = false;
            let theme = theme_settings_from_state(&state);
            if let Err(error) = save_theme_settings(&theme) {
                state.status_message = format!("Theme save failed: {error}");
            } else {
                state.status_message = "Theme saved.".to_string();
            }
        }
    }

    for (interaction, tab) in tab_query.iter() {
        if *interaction == Interaction::Pressed {
            state.theme_category = tab.0;
            match tab.0 {
                ThemeCategory::Theme => {
                    if state.theme_color_target.is_link_color()
                        || state.theme_color_target.is_text_color()
                        || matches!(state.theme_color_target, ThemeColorTarget::Ui(role) if role != UiColor::PlainBackground)
                    {
                        state.theme_color_target = ThemeColorTarget::AppBackground;
                    }
                }
                ThemeCategory::Controls => {
                    if !matches!(state.theme_color_target, ThemeColorTarget::Ui(role) if role != UiColor::PlainBackground) {
                        state.theme_color_target = ThemeColorTarget::Ui(UiColor::PanelBackground);
                    }
                }
                ThemeCategory::Text => {
                    if !state.theme_color_target.is_text_color() {
                        state.theme_color_target = ThemeColorTarget::TextMain;
                    }
                }
                ThemeCategory::Links => {
                    if !state.theme_color_target.is_link_color() {
                        state.theme_color_target = ThemeColorTarget::LinkFallback;
                    }
                }
                ThemeCategory::Glass => {
                    state.theme_color_picker_open = false;
                }
                ThemeCategory::Themes => {
                    state.theme_color_picker_open = false;
                    state.available_themes = list_available_themes();
                }
            }
        }
    }

    for interaction in prev_query.iter() {
        if *interaction == Interaction::Pressed {
            state.available_themes = list_available_themes();
            if !state.available_themes.is_empty() {
                let current_idx = state
                    .available_themes
                    .iter()
                    .position(|t| t.eq_ignore_ascii_case(&state.current_theme_name))
                    .unwrap_or(0);
                let prev_idx = if current_idx == 0 {
                    state.available_themes.len() - 1
                } else {
                    current_idx - 1
                };
                let theme_name = state.available_themes[prev_idx].clone();
                if let Some(loaded) = load_named_theme(&theme_name) {
                    apply_theme_to_state(&mut state, &loaded);
                    let _ = save_theme_settings(&theme_settings_from_state(&state));
                    state.status_message =
                        format!("Theme switched to: {}", state.current_theme_name);
                }
            }
        }
    }

    for interaction in next_query.iter() {
        if *interaction == Interaction::Pressed {
            state.available_themes = list_available_themes();
            if !state.available_themes.is_empty() {
                let current_idx = state
                    .available_themes
                    .iter()
                    .position(|t| t.eq_ignore_ascii_case(&state.current_theme_name))
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % state.available_themes.len();
                let theme_name = state.available_themes[next_idx].clone();
                if let Some(loaded) = load_named_theme(&theme_name) {
                    apply_theme_to_state(&mut state, &loaded);
                    let _ = save_theme_settings(&theme_settings_from_state(&state));
                    state.status_message =
                        format!("Theme switched to: {}", state.current_theme_name);
                }
            }
        }
    }

    for (interaction, preset) in preset_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(loaded) = load_named_theme(&preset.0) {
                apply_theme_to_state(&mut state, &loaded);
                let _ = save_theme_settings(&theme_settings_from_state(&state));
                state.status_message = format!("Theme switched to: {}", state.current_theme_name);
            }
        }
    }

    for interaction in save_query.iter() {
        if *interaction == Interaction::Pressed {
            let theme = theme_settings_from_state(&state);
            let _ = save_theme_settings(&theme);
            if !state.current_theme_name.is_empty()
                && !state.current_theme_name.eq_ignore_ascii_case("Default")
                && !state.current_theme_name.eq_ignore_ascii_case("Classic")
                && !state.current_theme_name.eq_ignore_ascii_case("Dark")
            {
                let _ = save_named_theme(&state.current_theme_name, &theme);
            }
            state.status_message = format!("Saved theme: {}", state.current_theme_name);
        }
    }

    for interaction in save_new_query.iter() {
        if *interaction == Interaction::Pressed {
            let trimmed = state.theme_name_input.trim().to_string();
            if trimmed.is_empty() {
                state.status_message = "Please enter a theme name first.".to_string();
            } else {
                let mut theme = theme_settings_from_state(&state);
                theme.name = trimmed.clone();
                if let Err(error) = save_named_theme(&trimmed, &theme) {
                    state.status_message = format!("Failed to save theme: {error}");
                } else {
                    apply_theme_to_state(&mut state, &theme);
                    let _ = save_theme_settings(&theme);
                    state.theme_name_input.clear();
                    state.status_message = format!("Saved theme: {trimmed}");
                }
            }
        }
    }

    if state.theme_overlay_open && state.theme_category == ThemeCategory::Themes {
        for key_input in keyboard_inputs.read() {
            if !key_input.state.is_pressed() {
                continue;
            }
            match &key_input.logical_key {
                Key::Enter => {
                    let trimmed = state.theme_name_input.trim().to_string();
                    if !trimmed.is_empty() {
                        let mut theme = theme_settings_from_state(&state);
                        theme.name = trimmed.clone();
                        let _ = save_named_theme(&trimmed, &theme);
                        apply_theme_to_state(&mut state, &theme);
                        let _ = save_theme_settings(&theme);
                        state.theme_name_input.clear();
                        state.status_message = format!("Saved theme: {trimmed}");
                    }
                }
                Key::Backspace => {
                    state.theme_name_input.pop();
                }
                _ => {
                    if let Some(inserted_text) = &key_input.text {
                        for ch in inserted_text.chars() {
                            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == ' ' {
                                state.theme_name_input.push(ch);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn panel_splitter_bundle(kind: PanelSplitter) -> impl Bundle {
    (
        Node {
            width: px(PANEL_SPLITTER_WIDTH),
            height: percent(100.0),
            ..default()
        },
        RelativeCursorPosition::default(),
        BackgroundColor(COLOR_SPLITTER_IDLE),
        kind,
    )
}

pub(crate) fn panel_bundle(
    font: Handle<Font>,
    hue_sat_wheel: Handle<Image>,
    kind: PanelKind,
) -> impl Bundle {
    let body_color = match kind {
        PanelKind::Plain => COLOR_PANEL_BODY_PLAIN,
        PanelKind::Processed => COLOR_PANEL_BODY_PROCESSED,
    };
    let root_color = match kind {
        PanelKind::Plain => COLOR_PANEL_BG,
        PanelKind::Processed => Color::NONE,
    };

    (
        Node {
            flex_grow: 1.0,
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        PanelRoot { kind },
        BackgroundColor(root_color),
        children![(
            Node {
                width: percent(100.0),
                flex_grow: 1.0,
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(body_color),
            RelativeCursorPosition::default(),
            PanelBody { kind },
            children![
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    UiTransform::default(),
                    ZIndex(1),
                    PanelCanvas { kind },
                    children![
                        (
                            Node {
                                position_type: PositionType::Absolute,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            UiTransform::default(),
                            BackgroundColor(COLOR_PAPER),
                            Visibility::Hidden,
                            ZIndex(0),
                            PanelPaper { kind, slot: 0 },
                        ),
                        (
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(0.0),
                                top: px(0.0),
                                width: percent(100.0),
                                height: percent(100.0),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            PanelSelectionLayer { kind },
                            ZIndex(1),
                        ),
                        (
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(TEXT_PADDING_X),
                                top: px(TEXT_PADDING_Y),
                                width: px(CARET_WIDTH),
                                height: px(LINE_HEIGHT),
                                ..default()
                            },
                            UiTransform::default(),
                            BackgroundColor(Color::BLACK),
                            Visibility::Hidden,
                            ZIndex(2),
                            PanelCaret { kind },
                        ),
                        (
                            Text::new(""),
                            TextLayout::no_wrap(),
                            TextFont {
                                font: font.clone().into(),
                                font_size: FontSize::Px(FONT_SIZE),
                                ..default()
                            },
                            LineHeight::Px(LINE_HEIGHT),
                            TextColor(COLOR_ACTION),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(TEXT_PADDING_X),
                                top: px(TEXT_PADDING_Y),
                                ..default()
                            },
                            UiTransform::default(),
                            ZIndex(3),
                            PanelText { kind },
                        )
                    ],
                ),
                processed_overlay_toggle_group_bundle(font.clone(), hue_sat_wheel, kind),
            ],
        )],
    )
}

pub(crate) fn processed_overlay_toggle_group_bundle(
    font: Handle<Font>,
    hue_sat_wheel: Handle<Image>,
    kind: PanelKind,
) -> impl Bundle {
    (
        ProcessedOverlayToggleGroup { kind },
        ProcessedOverlayToggleSpring::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(10.0),
            display: if kind == PanelKind::Processed {
                Display::Flex
            } else {
                Display::None
            },
            min_width: px(136.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: px(6.0),
            ..default()
        },
        ZIndex(20),
        children![
            theme_overlay_container_bundle(font.clone(), hue_sat_wheel),
            processed_overlay_toggle_button(
                font.clone(),
                "Links: colored",
                ProcessedLinkColorToggle,
                ProcessedLinkColorToggleLabel,
            ),
            processed_overlay_toggle_button(
                font.clone(),
                "Sheet: A4 pages",
                ProcessedPaginationToggle,
                ProcessedPaginationToggleLabel,
            ),
            processed_overlay_toggle_button(
                font.clone(),
                "Marks: hidden",
                FormattingMarksToggle,
                FormattingMarksToggleLabel,
            ),
            processed_overlay_toggle_button(
                font,
                "Status: shown",
                StatusLineToggle,
                StatusLineToggleLabel,
            ),
        ],
    )
}

pub(crate) fn processed_overlay_toggle_button<M: Component, L: Component>(
    font: Handle<Font>,
    label: &str,
    marker: M,
    label_marker: L,
) -> impl Bundle {
    (
        Button,
        marker,
        Node {
            width: percent(100.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(9.0), px(5.0)),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        ThemedButton,
        children![(
            Text::new(label),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(12.0),
                ..default()
            },
            ThemedText::Main,
            label_marker,
        )],
    )
}

pub(crate) fn handle_toolbar_buttons(
    _dialog_main_thread: NonSend<DialogMainThreadMarker>,
    interaction_query: Query<(&Interaction, &ToolbarAction), (Changed<Interaction>, With<Button>)>,
    primary_window_query: Query<&RawHandleWrapper, With<PrimaryWindow>>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut state: ResMut<EditorState>,
    mut dialogs: ResMut<DialogState>,
    mut next_screen_state: ResMut<NextState<UiScreenState>>,
    mut task_state: Option<ResMut<StoryIndexTask>>,
) {
    let parent_handle = primary_window_query.iter().next();
    let processed_panel_size = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor());

    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        info!(
            "[dialog] Toolbar {:?} pressed (parent_handle: {}, has_pending: {})",
            action,
            parent_handle.is_some(),
            dialogs.pending.is_some()
        );

        match action {
            ToolbarAction::OpenWorkspace => {
                state.close_link_autocomplete();
                open_workspace_dialog(&mut state, &mut dialogs, parent_handle)
            }
            ToolbarAction::Save => {
                state.save_current_with_task(task_state.as_deref_mut());
            }
            ToolbarAction::SaveAs => {
                state.close_link_autocomplete();
                open_save_dialog(&mut state, &mut dialogs, parent_handle)
            }
            ToolbarAction::ExportPdf => {
                state.close_link_autocomplete();
                open_pdf_export_dialog(&mut state, &mut dialogs, parent_handle)
            }
            ToolbarAction::StoryQuerySheet => state.open_story_query_sheet(),
            ToolbarAction::ZoomOut => {
                let next_zoom = state.zoom - ZOOM_STEP;
                set_zoom_preserving_processed_anchor(&mut state, processed_panel_size, next_zoom);
                state.status_message = format!("Zoom: {}%", state.zoom_percent());
            }
            ToolbarAction::ZoomIn => {
                let next_zoom = state.zoom + ZOOM_STEP;
                set_zoom_preserving_processed_anchor(&mut state, processed_panel_size, next_zoom);
                state.status_message = format!("Zoom: {}%", state.zoom_percent());
            }
            ToolbarAction::Settings => {
                state.close_link_autocomplete();
                next_screen_state.set(UiScreenState::Settings);
                state.status_message = "Opened settings.".to_string();
            }
        }
    }
}

pub(crate) fn handle_processed_link_color_toggle(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ProcessedLinkColorToggle>)>,
    mut state: ResMut<EditorState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.processed_link_color_mode = state.processed_link_color_mode.next();
        state.status_message = format!(
            "Rendered link colors: {}",
            state.processed_link_color_mode.label()
        );
        if let Err(error) = save_editor_ui_state(&state) {
            state.status_message = format!("UI state save failed: {error}");
        }
    }
}

pub(crate) fn handle_processed_pagination_toggle(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ProcessedPaginationToggle>)>,
    panel_query: Query<(&PanelBody, &ComputedNode)>,
    mut state: ResMut<EditorState>,
) {
    let processed_panel_size = panel_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor());
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        toggle_processed_pagination(&mut state, processed_panel_size);
        if let Err(error) = save_editor_ui_state(&state) {
            state.status_message = format!("UI state save failed: {error}");
        }
    }
}

pub(crate) fn handle_formatting_marks_toggle(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<FormattingMarksToggle>)>,
    mut state: ResMut<EditorState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.formatting_marks_visible = !state.formatting_marks_visible;
        state.status_message = format!(
            "Formatting marks: {}",
            if state.formatting_marks_visible {
                "shown"
            } else {
                "hidden"
            }
        );
        if let Err(error) = save_editor_ui_state(&state) {
            state.status_message = format!("UI state save failed: {error}");
        }
    }
}

pub(crate) fn handle_status_line_toggle(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<StatusLineToggle>)>,
    mut state: ResMut<EditorState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.status_line_visible = !state.status_line_visible;
        state.status_message = format!(
            "Status line: {}",
            if state.status_line_visible {
                "shown"
            } else {
                "hidden"
            }
        );
        if let Err(error) = save_editor_ui_state(&state) {
            state.status_message = format!("UI state save failed: {error}");
        }
    }
}

pub(crate) fn toggle_processed_pagination(
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
) {
    let caret_anchor = processed_panel_size
        .and_then(|panel_size| capture_processed_caret_viewport_anchor(state, panel_size));
    state.processed_paginated = !state.processed_paginated;
    state.mark_processed_cache_dirty_from(0);
    if let (Some(panel_size), Some(anchor)) = (processed_panel_size, caret_anchor) {
        restore_processed_caret_viewport_anchor(state, panel_size, anchor);
    }
    state.status_message = format!(
        "Rendered sheet: {}",
        if state.processed_paginated {
            "A4 pages"
        } else {
            "continuous"
        }
    );
}

pub(crate) fn sync_processed_overlay_toggle_group(
    time: Res<Time>,
    state: Res<EditorState>,
    resolved_widths: Res<ResolvedPanelWidths>,
    panel_query: Query<(&PanelBody, &ComputedNode)>,
    paper_query: Query<(&PanelPaper, &Node, &Visibility), Without<ProcessedOverlayToggleGroup>>,
    mut toggle_query: Query<
        (
            &ProcessedOverlayToggleGroup,
            &ComputedNode,
            &mut ProcessedOverlayToggleSpring,
            &mut Node,
            &mut ZIndex,
        ),
        Without<PanelPaper>,
    >,
    mut label_query: Query<&mut Text, With<ProcessedLinkColorToggleLabel>>,
    mut pagination_label_query: Query<
        &mut Text,
        (
            With<ProcessedPaginationToggleLabel>,
            Without<ProcessedLinkColorToggleLabel>,
        ),
    >,
    mut formatting_marks_label_query: Query<
        &mut Text,
        (
            With<FormattingMarksToggleLabel>,
            Without<ProcessedLinkColorToggleLabel>,
            Without<ProcessedPaginationToggleLabel>,
        ),
    >,
    mut status_line_label_query: Query<
        &mut Text,
        (
            With<StatusLineToggleLabel>,
            Without<ProcessedLinkColorToggleLabel>,
            Without<ProcessedPaginationToggleLabel>,
            Without<FormattingMarksToggleLabel>,
        ),
    >,
) {
    let processed_panel_size = panel_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(panel, computed)| resolved_widths.panel_size(panel.kind, computed));
    let dt = time.delta_secs().clamp(0.0, 1.0 / 30.0);

    for (toggle, computed, mut spring, mut node, mut z_index) in toggle_query.iter_mut() {
        let Some(panel_size) = processed_panel_size.filter(|_| {
            toggle.kind == PanelKind::Processed
                && (state.theme_overlay_open || state.document_format != DocumentFormat::Canvas)
                && state.right_buttons_visible
        }) else {
            node.display = Display::None;
            *z_index = ZIndex(20);
            *spring = ProcessedOverlayToggleSpring::default();
            continue;
        };

        let geometry = processed_page_geometry(panel_size, &state);
        let page_right =
            geometry.paper_left + geometry.paper_width - state.processed_horizontal_scroll;
        let toggle_width = (computed.size().x * computed.inverse_scale_factor())
            .max(if state.theme_overlay_open { 380.0 } else { 116.0 });
        let toggle_height = (computed.size().y * computed.inverse_scale_factor()).max(30.0);
        let base_x = (panel_size.x - toggle_width - 10.0).max(0.0);
        let page_border_offset = page_right - base_x;
        let touching_page = page_right >= base_x;
        let contact_started = spring.initialized && touching_page && !spring.touching_page;
        let impact_speed = if spring.initialized && dt > f32::EPSILON {
            ((page_right - spring.previous_page_right) / dt).max(0.0)
        } else {
            0.0
        };

        node.display = Display::Flex;
        if state.theme_overlay_open {
            spring.phase = ProcessedOverlayTogglePhase::Idle;
            spring.offset_x = 0.0;
            spring.velocity_x = 0.0;
            spring.velocity_y = 0.0;
            spring.touching_page = false;
            spring.initialized = true;
            spring.previous_page_right = page_right;
            node.left = px(base_x);
            node.top = px(10.0);
            *z_index = ZIndex(25);
            continue;
        }

        if !spring.initialized {
            spring.phase = if touching_page {
                ProcessedOverlayTogglePhase::ReturningUnderPage
            } else {
                ProcessedOverlayTogglePhase::Idle
            };
        } else if !touching_page {
            let returning_below_page =
                spring.phase == ProcessedOverlayTogglePhase::ReturningUnderPage;
            if !returning_below_page
                || link_toggle_has_cleared_page_border(spring.offset_x, page_border_offset)
            {
                spring.phase = ProcessedOverlayTogglePhase::Idle;
            }
        } else if contact_started {
            spring.compression_distance = link_toggle_compression_for_impact(impact_speed);
            spring.phase = ProcessedOverlayTogglePhase::Compressing;
            spring.velocity_x = -impact_speed.clamp(260.0, 900.0);
        }

        match spring.phase {
            ProcessedOverlayTogglePhase::Idle => {
                (spring.offset_x, spring.velocity_x) =
                    step_link_toggle_return(spring.offset_x, spring.velocity_x, dt);
            }
            ProcessedOverlayTogglePhase::Compressing => {
                let compression_target =
                    link_toggle_compression_target(page_border_offset, spring.compression_distance);
                (spring.offset_x, spring.velocity_x) = step_link_toggle_spring(
                    spring.offset_x,
                    spring.velocity_x,
                    compression_target,
                    240.0,
                    13.0,
                    dt,
                );
                // The paper boundary may continue moving during impact.
                // Never allow the visible overlap to exceed its bounded
                // compression distance.
                if spring.offset_x < compression_target {
                    spring.offset_x = compression_target;
                }
                if spring.offset_x <= compression_target + 0.5
                    || (spring.velocity_x >= 0.0 && spring.offset_x < page_border_offset)
                {
                    spring.velocity_x = (420.0 + impact_speed * 0.32).clamp(420.0, 1_000.0);
                    spring.phase = ProcessedOverlayTogglePhase::Rebounding;
                }
            }
            ProcessedOverlayTogglePhase::Rebounding => {
                let rebound_target =
                    link_toggle_rebound_target(page_border_offset, spring.compression_distance);
                let compression_limit =
                    link_toggle_compression_target(page_border_offset, spring.compression_distance);
                (spring.offset_x, spring.velocity_x) = step_link_toggle_spring(
                    spring.offset_x,
                    spring.velocity_x,
                    rebound_target,
                    190.0,
                    10.0,
                    dt,
                );
                if spring.offset_x < compression_limit {
                    spring.offset_x = compression_limit;
                    spring.velocity_x = spring.velocity_x.max(0.0);
                }
                if link_toggle_rebound_has_reached_apex(
                    spring.offset_x,
                    spring.velocity_x,
                    rebound_target,
                    page_border_offset,
                ) {
                    // Change layer at the outward rebound apex, preserving the
                    // remaining velocity for the under-page settling spring.
                    spring.phase = ProcessedOverlayTogglePhase::ReturningUnderPage;
                }
            }
            ProcessedOverlayTogglePhase::ReturningUnderPage => {
                (spring.offset_x, spring.velocity_x) =
                    step_link_toggle_return(spring.offset_x, spring.velocity_x, dt);
            }
        }
        if spring.phase == ProcessedOverlayTogglePhase::Idle && spring.offset_x < page_border_offset
        {
            // When scrolling back, keep the above-page control on the safe
            // side of the live paper border instead of letting its return
            // spring phase through the page.
            spring.offset_x = page_border_offset;
            if spring.velocity_x < 0.0 {
                spring.velocity_x *= -0.35;
            }
        }
        spring.touching_page = touching_page;
        spring.initialized = true;
        spring.previous_page_right = page_right;
        node.left = px(base_x + spring.offset_x);

        let current_top = match node.top {
            Val::Px(top) => top,
            _ => 10.0,
        };
        let target_top = if touching_page {
            link_toggle_vertical_target(
                10.0,
                toggle_height,
                paper_query
                    .iter()
                    .filter_map(|(paper, paper_node, paper_visibility)| {
                        if paper.kind != PanelKind::Processed
                            || *paper_visibility != Visibility::Visible
                        {
                            return None;
                        }
                        match (paper_node.top, paper_node.height) {
                            (Val::Px(top), Val::Px(height)) => Some((top, height)),
                            _ => None,
                        }
                    }),
            )
            .unwrap_or(10.0)
        } else {
            10.0
        };
        let (next_top, next_velocity_y) =
            step_link_toggle_vertical_spring(current_top, spring.velocity_y, target_top, dt);
        node.top = px(next_top);
        spring.velocity_y = next_velocity_y;

        *z_index = if spring.phase == ProcessedOverlayTogglePhase::ReturningUnderPage {
            // The layer changes at the bounded outward rebound apex, then the
            // control settles underneath the page.
            ZIndex(0)
        } else {
            ZIndex(20)
        };
    }

    let label = match state.processed_link_color_mode {
        ProcessedLinkColorMode::Colored => "Links: colored",
        ProcessedLinkColorMode::Hovered => "Links: hovered",
        ProcessedLinkColorMode::Plain => "Links: plain",
    };
    for mut text in label_query.iter_mut() {
        if text.as_str() != label {
            **text = label.to_string();
        }
    }

    let pagination_label = if state.processed_paginated {
        "Sheet: A4 pages"
    } else {
        "Sheet: continuous"
    };
    for mut text in pagination_label_query.iter_mut() {
        if text.as_str() != pagination_label {
            **text = pagination_label.to_string();
        }
    }

    let formatting_marks_label = if state.formatting_marks_visible {
        "Marks: shown"
    } else {
        "Marks: hidden"
    };
    for mut text in formatting_marks_label_query.iter_mut() {
        if text.as_str() != formatting_marks_label {
            **text = formatting_marks_label.to_string();
        }
    }

    let status_line_label = if state.status_line_visible {
        "Status: shown"
    } else {
        "Status: hidden"
    };
    for mut text in status_line_label_query.iter_mut() {
        if text.as_str() != status_line_label {
            **text = status_line_label.to_string();
        }
    }
}

pub(crate) fn link_toggle_compression_for_impact(impact_speed: f32) -> f32 {
    (8.0 + impact_speed.max(0.0) * 0.015).clamp(8.0, 20.0)
}

pub(crate) fn link_toggle_compression_target(border_offset: f32, compression_distance: f32) -> f32 {
    border_offset - compression_distance.clamp(0.0, 20.0)
}

pub(crate) fn link_toggle_rebound_target(border_offset: f32, compression_distance: f32) -> f32 {
    border_offset + (compression_distance.clamp(0.0, 20.0) * 2.0).min(40.0)
}

pub(crate) fn link_toggle_vertical_target(
    current_top: f32,
    toggle_height: f32,
    pages: impl Iterator<Item = (f32, f32)>,
) -> Option<f32> {
    let toggle_height = toggle_height.max(0.0);
    pages
        .filter_map(|(page_top, page_height)| {
            let page_bottom = page_top + page_height;
            if page_height < toggle_height {
                return None;
            }
            let target = if current_top < page_top {
                page_top
            } else if current_top + toggle_height > page_bottom {
                page_bottom - toggle_height
            } else {
                current_top
            };
            Some((target, (target - current_top).abs()))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(target, _)| target)
}

pub(crate) fn step_link_toggle_vertical_spring(
    top: f32,
    velocity: f32,
    target: f32,
    dt: f32,
) -> (f32, f32) {
    let (next_top, next_velocity) = step_link_toggle_spring(top, velocity, target, 115.0, 11.0, dt);
    if (next_top - target).abs() < 0.08 && next_velocity.abs() < 0.8 {
        (target, 0.0)
    } else {
        (next_top, next_velocity)
    }
}

pub(crate) fn link_toggle_rebound_has_reached_apex(
    offset: f32,
    velocity: f32,
    rebound_target: f32,
    border_offset: f32,
) -> bool {
    offset >= rebound_target - 0.5 || (velocity <= 0.0 && offset > border_offset)
}

pub(crate) fn link_toggle_has_cleared_page_border(offset: f32, border_offset: f32) -> bool {
    offset >= border_offset
}

pub(crate) fn step_link_toggle_spring(
    offset: f32,
    velocity: f32,
    target: f32,
    stiffness: f32,
    damping: f32,
    dt: f32,
) -> (f32, f32) {
    let dt = dt.clamp(0.0, 1.0 / 30.0);
    let acceleration = (target - offset) * stiffness - velocity * damping;
    let next_velocity = velocity + acceleration * dt;
    let next_offset = offset + next_velocity * dt;

    (next_offset, next_velocity)
}

pub(crate) fn step_link_toggle_return(offset: f32, velocity: f32, dt: f32) -> (f32, f32) {
    let (mut next_offset, mut next_velocity) =
        step_link_toggle_spring(offset, velocity, 0.0, 130.0, 8.0, dt);

    // Keep extreme frame spikes bounded without removing the spring rebound.
    if next_offset < -24.0 {
        next_offset = -24.0;
        if next_velocity < 0.0 {
            next_velocity *= -0.42;
        }
    }

    if next_offset.abs() < 0.08 && next_velocity.abs() < 0.8 {
        (0.0, 0.0)
    } else {
        (next_offset, next_velocity)
    }
}

pub(crate) fn handle_theme_color_picker_buttons(
    interaction_query: Query<
        (&Interaction, &ThemeColorPickerButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let same_target = state.theme_color_target == button.target;
        state.theme_color_target = button.target;
        state.theme_color_picker_open = !(same_target && state.theme_color_picker_open);
        state.status_message = if state.theme_color_picker_open {
            format!("Opened color picker for {}.", button.target.status_label())
        } else {
            format!("Closed color picker for {}.", button.target.status_label())
        };
    }
}

pub(crate) fn handle_settings_buttons(
    interaction_query: Query<(&Interaction, &SettingsAction), (Changed<Interaction>, With<Button>)>,
    panel_query: Query<(&PanelBody, &ComputedNode)>,
    mut state: ResMut<EditorState>,
    mut next_screen_state: ResMut<NextState<UiScreenState>>,
) {
    let processed_panel_size = panel_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(_, computed)| computed.size() * computed.inverse_scale_factor());
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut settings_changed = false;
        let mut ui_state_changed = false;
        let mut theme_changed = false;
        match action {
            SettingsAction::DialogueDoubleSpaceNewline => {
                state.dialogue_double_space_newline = !state.dialogue_double_space_newline;
                settings_changed = true;
                state.status_message = format!(
                    "Dialogue double-space newline in processed modes: {}",
                    if state.dialogue_double_space_newline {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            SettingsAction::NonDialogueDoubleSpaceNewline => {
                state.non_dialogue_double_space_newline = !state.non_dialogue_double_space_newline;
                settings_changed = true;
                state.status_message = format!(
                    "Non-dialogue double-space newline in processed modes: {}",
                    if state.non_dialogue_double_space_newline {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            SettingsAction::ToggleProcessedPagination => {
                toggle_processed_pagination(&mut state, processed_panel_size);
                ui_state_changed = true;
            }
            SettingsAction::ToggleVimMode => {
                state.close_link_autocomplete();
                state.vim_enabled = !state.vim_enabled;
                state.vim_mode = VimMode::Normal;
                state.vim_pending_operator = None;
                state.vim_visual_anchor = None;
                state.vim_visual_head = None;
                state.selection_anchor = None;
                settings_changed = true;
                state.status_message =
                    format!("Vim mode: {}", if state.vim_enabled { "ON" } else { "OFF" });
            }
            SettingsAction::CycleDefaultViewMode => {
                state.default_view_mode = state.default_view_mode.next_default();
                settings_changed = true;
                state.status_message = format!(
                    "Default view: {} (used on next launch)",
                    state.default_view_mode.default_settings_label()
                );
            }
            SettingsAction::ShowSystemTitlebar => {
                state.show_system_titlebar = !state.show_system_titlebar;
                settings_changed = true;
                state.status_message = format!(
                    "System titlebar: {}",
                    if state.show_system_titlebar {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            SettingsAction::ToggleProcessedGlass => {
                state.processed_glass = !state.processed_glass;
                theme_changed = true;
                state.status_message = format!(
                    "Processed background glass: {}",
                    if state.processed_glass { "ON" } else { "OFF" }
                );
            }
            SettingsAction::ToggleExplorerGlass => {
                state.explorer_glass = !state.explorer_glass;
                theme_changed = true;
                state.status_message = format!(
                    "Explorer glass: {}",
                    if state.explorer_glass { "ON" } else { "OFF" }
                );
            }
            SettingsAction::ToggleSettingsGlass => {
                state.settings_glass = !state.settings_glass;
                theme_changed = true;
                state.status_message = format!(
                    "Settings glass: {}",
                    if state.settings_glass { "ON" } else { "OFF" }
                );
            }
            SettingsAction::MarginLeftDecrease => {
                adjust_page_margin(&mut state, MarginEdge::Left, -PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::MarginLeftIncrease => {
                adjust_page_margin(&mut state, MarginEdge::Left, PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::MarginRightDecrease => {
                adjust_page_margin(&mut state, MarginEdge::Right, -PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::MarginRightIncrease => {
                adjust_page_margin(&mut state, MarginEdge::Right, PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::MarginTopDecrease => {
                adjust_page_margin(&mut state, MarginEdge::Top, -PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::MarginTopIncrease => {
                adjust_page_margin(&mut state, MarginEdge::Top, PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::MarginBottomDecrease => {
                adjust_page_margin(&mut state, MarginEdge::Bottom, -PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::MarginBottomIncrease => {
                adjust_page_margin(&mut state, MarginEdge::Bottom, PAGE_MARGIN_STEP);
                settings_changed = true;
            }
            SettingsAction::LinkHoverHsvValueDecrease => {
                state.link_hover_hsv_value_adjustment -= LINK_HOVER_HSV_VALUE_STEP;
                sync_theme_colors(&mut state);
                theme_changed = true;
                state.status_message = format!(
                    "Link hover HSV value adjustment: {}.",
                    format_hsv_value_adjustment_label(state.link_hover_hsv_value_adjustment)
                );
            }
            SettingsAction::LinkHoverHsvValueIncrease => {
                state.link_hover_hsv_value_adjustment += LINK_HOVER_HSV_VALUE_STEP;
                sync_theme_colors(&mut state);
                theme_changed = true;
                state.status_message = format!(
                    "Link hover HSV value adjustment: {}.",
                    format_hsv_value_adjustment_label(state.link_hover_hsv_value_adjustment)
                );
            }
            SettingsAction::OpenTheme => {
                state.close_link_autocomplete();
                state.theme_color_target = ThemeColorTarget::AppBackground;
                state.theme_color_picker_open = false;
                state.theme_overlay_open = true;
                state.theme_category = ThemeCategory::Theme;
                state.right_buttons_visible = true;
                if state.display_mode == DisplayMode::Plain {
                    state.display_mode = DisplayMode::Split;
                }
                next_screen_state.set(UiScreenState::Editor);
                state.status_message = "Opened theme options.".to_string();
            }
            SettingsAction::OpenLinkColors => {
                state.close_link_autocomplete();
                state.theme_color_target = ThemeColorTarget::LinkFallback;
                state.theme_color_picker_open = false;
                state.theme_overlay_open = true;
                state.theme_category = ThemeCategory::Links;
                state.right_buttons_visible = true;
                if state.display_mode == DisplayMode::Plain {
                    state.display_mode = DisplayMode::Split;
                }
                next_screen_state.set(UiScreenState::Editor);
                state.status_message = "Opened link color options.".to_string();
            }
            SettingsAction::OpenKeybinds => {
                state.close_link_autocomplete();
                state.pending_keybind_capture = None;
                next_screen_state.set(UiScreenState::Keybinds);
                state.status_message = "Opened keybinds.".to_string();
            }
            SettingsAction::BackToSettings => {
                state.pending_keybind_capture = None;
                next_screen_state.set(UiScreenState::Settings);
                state.status_message = "Opened settings.".to_string();
            }
            SettingsAction::BackToEditor => {
                state.pending_keybind_capture = None;
                next_screen_state.set(UiScreenState::Editor);
                state.status_message = "Returned to editor.".to_string();
            }
        }

        if settings_changed {
            state.mark_processed_cache_dirty_from(0);
            let persistent = persistent_settings_from_state(&state);
            if let Err(error) = save_persistent_settings(&persistent) {
                state.status_message = format!("Settings save failed: {error}");
            }
        }

        if ui_state_changed && let Err(error) = save_editor_ui_state(&state) {
            state.status_message = format!("UI state save failed: {error}");
        }

        if theme_changed {
            let theme = theme_settings_from_state(&state);
            if let Err(error) = save_theme_settings(&theme) {
                state.status_message = format!("Theme save failed: {error}");
            }
        }
    }
}

pub(crate) fn handle_theme_color_picker_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<EditorState>,
    wheel_query: Query<(&RelativeCursorPosition, &ComputedNode), With<ThemeHueSatWheel>>,
    slider_query: Query<(&ThemeColorSlider, &RelativeCursorPosition, &ComputedNode)>,
) {
    if !state.theme_color_picker_open || !mouse_buttons.pressed(MouseButton::Left) {
        return;
    }

    let mut theme_changed = false;
    for (relative_cursor, computed) in wheel_query.iter() {
        if !relative_cursor.cursor_over() {
            continue;
        }
        let Some(normalized) = relative_cursor.normalized else {
            continue;
        };

        let width = computed.size().x * computed.inverse_scale_factor();
        let height = computed.size().y * computed.inverse_scale_factor();
        if width <= 0.0 || height <= 0.0 {
            continue;
        }
        let local_x = (normalized.x + 0.5).clamp(0.0, 1.0) * width;
        let local_y = (normalized.y + 0.5).clamp(0.0, 1.0) * height;
        let radius_px = width.min(height) * 0.5;
        if radius_px <= 0.0 {
            continue;
        }

        let dx = (local_x - width * 0.5) / radius_px;
        let dy = (height * 0.5 - local_y) / radius_px;
        let radius = (dx * dx + dy * dy).sqrt();
        if radius > 1.0 {
            continue;
        }

        let mut hue = dy.atan2(dx) / std::f32::consts::TAU;
        if hue < 0.0 {
            hue += 1.0;
        }
        let saturation = radius.clamp(0.0, 1.0);
        let current_rgba = active_theme_rgba(&state);
        let current_rgb = Vec3::new(current_rgba.x, current_rgba.y, current_rgba.z);
        let (_, _, value) = rgb_to_hsv(current_rgb);
        let next_rgb = hsv_to_rgb(hue, saturation, value);
        let mut next_rgba = current_rgba;
        next_rgba.x = next_rgb.x;
        next_rgba.y = next_rgb.y;
        next_rgba.z = next_rgb.z;
        set_active_theme_rgba(&mut state, next_rgba);
        theme_changed = true;
    }

    for (slider, relative_cursor, computed) in slider_query.iter() {
        if !relative_cursor.cursor_over() {
            continue;
        }
        let Some(normalized) = relative_cursor.normalized else {
            continue;
        };
        let track_width = (computed.size().x * computed.inverse_scale_factor())
            .max(THEME_COLOR_SLIDER_KNOB_WIDTH + 1.0);
        let usable_width = (track_width - THEME_COLOR_SLIDER_KNOB_WIDTH).max(1.0);
        let cursor_x = (normalized.x + 0.5).clamp(0.0, 1.0) * track_width;
        let next =
            ((cursor_x - THEME_COLOR_SLIDER_KNOB_WIDTH * 0.5) / usable_width).clamp(0.0, 1.0);
        let mut next_rgba = active_theme_rgba(&state);
        match slider.channel {
            ThemeSliderChannel::Hue => {
                let current_rgb = Vec3::new(next_rgba.x, next_rgba.y, next_rgba.z);
                let (_, saturation, value) = rgb_to_hsv(current_rgb);
                let next_rgb = hsv_to_rgb(next, saturation, value);
                next_rgba.x = next_rgb.x;
                next_rgba.y = next_rgb.y;
                next_rgba.z = next_rgb.z;
            }
            ThemeSliderChannel::Saturation => {
                let current_rgb = Vec3::new(next_rgba.x, next_rgba.y, next_rgba.z);
                let (hue, _, value) = rgb_to_hsv(current_rgb);
                let next_rgb = hsv_to_rgb(hue, next, value);
                next_rgba.x = next_rgb.x;
                next_rgba.y = next_rgb.y;
                next_rgba.z = next_rgb.z;
            }
            ThemeSliderChannel::Red => next_rgba.x = next,
            ThemeSliderChannel::Green => next_rgba.y = next,
            ThemeSliderChannel::Blue => next_rgba.z = next,
            ThemeSliderChannel::Alpha => next_rgba.w = next,
            ThemeSliderChannel::Value => {
                let current_rgb = Vec3::new(next_rgba.x, next_rgba.y, next_rgba.z);
                let (hue, saturation, _) = rgb_to_hsv(current_rgb);
                let next_rgb = hsv_to_rgb(hue, saturation, next);
                next_rgba.x = next_rgb.x;
                next_rgba.y = next_rgb.y;
                next_rgba.z = next_rgb.z;
            }
        }
        set_active_theme_rgba(&mut state, next_rgba);
        theme_changed = true;
    }

    if theme_changed {
        let theme = theme_settings_from_state(&state);
        if let Err(error) = save_theme_settings(&theme) {
            state.status_message = format!("Theme save failed: {error}");
        }
    }
}

pub(crate) fn handle_keybind_buttons(
    interaction_query: Query<
        (&Interaction, &KeybindRebindButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.pending_keybind_capture = Some(button.action);
        state.status_message = format!(
            "Press a key for {}. Hold Ctrl, Alt, Super/Cmd, or Space to choose that modifier. Esc cancels.",
            shortcut_action_label(button.action)
        );
    }
}

pub(crate) fn settings_menu_max_scroll(viewport: &ComputedNode, content: &ComputedNode) -> f32 {
    let viewport_height = viewport.size().y * viewport.inverse_scale_factor();
    let content_height = content.size().y * content.inverse_scale_factor();
    (content_height - viewport_height).max(0.0)
}

pub(crate) fn reset_settings_menu_scroll_on_open(
    screen_state: Res<State<UiScreenState>>,
    mut previous_screen: Local<Option<UiScreenState>>,
    mut scroll_query: Query<(&SettingsMenuScrollArea, &mut ScrollPosition)>,
) {
    let screen = *screen_state.get();
    if *previous_screen == Some(screen) {
        return;
    }
    *previous_screen = Some(screen);

    for (area, mut scroll_position) in scroll_query.iter_mut() {
        if area.screen == screen {
            scroll_position.x = 0.0;
            scroll_position.y = 0.0;
        }
    }
}

pub(crate) fn handle_settings_menu_mouse_scroll(
    mut mouse_wheels: MessageReader<MouseWheel>,
    screen_state: Res<State<UiScreenState>>,
    mut scroll_query: Query<(
        &SettingsMenuScrollArea,
        &RelativeCursorPosition,
        &ComputedNode,
        &mut ScrollPosition,
    )>,
    content_query: Query<(&SettingsMenuScrollContent, &ComputedNode)>,
) {
    let screen = *screen_state.get();
    let Some((_, relative_cursor, viewport, mut scroll_position)) = scroll_query
        .iter_mut()
        .find(|(area, _, _, _)| area.screen == screen)
    else {
        return;
    };
    if !relative_cursor.cursor_over() {
        return;
    }
    let Some((_, content)) = content_query
        .iter()
        .find(|(content, _)| content.screen == screen)
    else {
        return;
    };

    let mut delta_y = 0.0_f32;
    for wheel in mouse_wheels.read() {
        delta_y += match wheel.unit {
            MouseScrollUnit::Line => -wheel.y * 36.0,
            MouseScrollUnit::Pixel => -wheel.y,
        };
    }
    if delta_y.abs() <= f32::EPSILON {
        return;
    }

    let max_scroll = settings_menu_max_scroll(viewport, content);
    scroll_position.y = (scroll_position.y + delta_y).clamp(0.0, max_scroll);
    scroll_position.x = 0.0;
}

pub(crate) fn handle_keybind_screen_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    mut next_screen_state: ResMut<NextState<UiScreenState>>,
    mut scroll_query: Query<(&SettingsMenuScrollArea, &ComputedNode, &mut ScrollPosition)>,
    content_query: Query<(&SettingsMenuScrollContent, &ComputedNode)>,
) {
    if state.pending_keybind_capture.is_some() {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        next_screen_state.set(UiScreenState::Settings);
        state.status_message = "Closed keybinds.".to_string();
        return;
    }

    let Some((_, viewport, mut scroll_position)) = scroll_query
        .iter_mut()
        .find(|(area, _, _)| area.screen == UiScreenState::Keybinds)
    else {
        return;
    };
    let Some((_, content)) = content_query
        .iter()
        .find(|(content, _)| content.screen == UiScreenState::Keybinds)
    else {
        return;
    };
    let max_scroll = settings_menu_max_scroll(viewport, content);
    let page_step = (viewport.size().y * viewport.inverse_scale_factor() * 0.85).max(80.0);

    let next_y = if keys.just_pressed(KeyCode::Home) {
        Some(0.0)
    } else if keys.just_pressed(KeyCode::End) {
        Some(max_scroll)
    } else if keys.just_pressed(KeyCode::PageUp) {
        Some(scroll_position.y - page_step)
    } else if keys.just_pressed(KeyCode::PageDown) {
        Some(scroll_position.y + page_step)
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        Some(scroll_position.y - 36.0)
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        Some(scroll_position.y + 36.0)
    } else {
        None
    };

    if let Some(next_y) = next_y {
        scroll_position.y = next_y.clamp(0.0, max_scroll);
        scroll_position.x = 0.0;
    }
}

pub(crate) fn handle_settings_screen_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    mut next_screen_state: ResMut<NextState<UiScreenState>>,
    mut scroll_query: Query<(&SettingsMenuScrollArea, &ComputedNode, &mut ScrollPosition)>,
    content_query: Query<(&SettingsMenuScrollContent, &ComputedNode)>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_screen_state.set(UiScreenState::Editor);
        state.status_message = "Closed settings.".to_string();
        return;
    }

    let Some((_, viewport, mut scroll_position)) = scroll_query
        .iter_mut()
        .find(|(area, _, _)| area.screen == UiScreenState::Settings)
    else {
        return;
    };
    let Some((_, content)) = content_query
        .iter()
        .find(|(content, _)| content.screen == UiScreenState::Settings)
    else {
        return;
    };
    let max_scroll = settings_menu_max_scroll(viewport, content);
    let page_step = (viewport.size().y * viewport.inverse_scale_factor() * 0.85).max(80.0);

    let next_y = if keys.just_pressed(KeyCode::Home) {
        Some(0.0)
    } else if keys.just_pressed(KeyCode::End) {
        Some(max_scroll)
    } else if keys.just_pressed(KeyCode::PageUp) {
        Some(scroll_position.y - page_step)
    } else if keys.just_pressed(KeyCode::PageDown) {
        Some(scroll_position.y + page_step)
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        Some(scroll_position.y - 36.0)
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        Some(scroll_position.y + 36.0)
    } else {
        None
    };

    if let Some(next_y) = next_y {
        scroll_position.y = next_y.clamp(0.0, max_scroll);
        scroll_position.x = 0.0;
    }
}

pub(crate) fn capture_keybind_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
) {
    let Some(action) = state.pending_keybind_capture else {
        return;
    };

    for input in keyboard_inputs.read() {
        if !input.state.is_pressed() {
            continue;
        }

        let key_code = input.key_code;
        if key_code == KeyCode::Escape {
            state.pending_keybind_capture = None;
            state.status_message = format!(
                "Canceled keybind change for {}.",
                shortcut_action_label(action)
            );
            return;
        }

        if matches!(
            key_code,
            KeyCode::ControlLeft
                | KeyCode::ControlRight
                | KeyCode::ShiftLeft
                | KeyCode::ShiftRight
                | KeyCode::AltLeft
                | KeyCode::AltRight
                | KeyCode::SuperLeft
                | KeyCode::SuperRight
        ) {
            continue;
        }
        if key_code == KeyCode::Space && !text_input_modifier_pressed(&keys) {
            continue;
        }

        if binding_key_name(key_code).is_none() {
            state.status_message = format!(
                "Unsupported key for {}. Use letters, digits, Space, '`', '=' or '-'.",
                shortcut_action_label(action)
            );
            continue;
        }

        let binding = ShortcutBinding {
            key: key_code,
            shift: shift_modifier_pressed(&keys),
            modifier: match capture_shortcut_modifier(&keys, key_code) {
                Ok(modifier) => modifier,
                Err(message) => {
                    state.status_message = message.to_string();
                    continue;
                }
            },
        };
        let conflict = SHORTCUT_ACTIONS.iter().copied().find(|candidate| {
            *candidate != action && state.keybinds.binding(*candidate) == binding
        });

        state.keybinds.set_binding(action, binding);
        state.pending_keybind_capture = None;
        if let Err(error) = save_keybind_settings(&state.keybinds) {
            state.status_message = format!("Keybind save failed: {error}");
            return;
        }

        state.status_message = if let Some(conflict_action) = conflict {
            format!(
                "Updated {} to {} (also used by {}).",
                shortcut_action_label(action),
                binding_display(binding),
                shortcut_action_label(conflict_action)
            )
        } else {
            format!(
                "Updated {} to {}.",
                shortcut_action_label(action),
                binding_display(binding)
            )
        };
        return;
    }
}

pub(crate) fn sync_glass_surfaces(
    state: Res<EditorState>,
    native_glass_state: Res<NativeGlassState>,
    mut color_queries: ParamSet<(
        Query<&mut BackgroundColor, With<WindowSurfaceRoot>>,
        Query<&mut BackgroundColor, With<TopMenuSection>>,
        Query<&mut BackgroundColor, With<WorkspaceSidebarPane>>,
        Query<(&PanelRoot, &mut BackgroundColor)>,
        Query<(&PanelBody, &mut BackgroundColor)>,
        Query<(&PanelPaper, &mut BackgroundColor)>,
        Query<&mut BackgroundColor, With<StatusLineRoot>>,
        Query<
            &mut BackgroundColor,
            Or<(
                With<SettingsScreenRoot>,
                With<KeybindsScreenRoot>,
                With<ThemeScreenRoot>,
            )>,
        >,
    )>,
) {
    let processed_glass_active = state.processed_glass && native_glass_state.active;

    // Keep paper colors owned by this system, independent of page layout/redraws.
    // Check every frame so newly created pages also inherit the current theme.
    for (panel_paper, mut color) in color_queries.p5().iter_mut() {
        let paper_color = match panel_paper.kind {
            PanelKind::Processed
                if !state.processed_paginated && panel_paper.slot != 0 =>
            {
                Color::NONE
            }
            _ => state.paper_bg_color.with_alpha(1.0),
        };
        color.set_if_neq(BackgroundColor(paper_color));
    }

    if !state.is_changed() && !native_glass_state.is_changed() {
        return;
    }

    let settings_glass_active = state.settings_glass && native_glass_state.active;

    if let Ok(mut color) = color_queries.p0().single_mut() {
        // Every ancestor must let alpha through or the desktop remains hidden.
        let next = if state.any_glass_enabled() && native_glass_state.active {
            Color::NONE
        } else {
            state.app_bg_color
        };
        color.set_if_neq(BackgroundColor(next));
    }

    if let Ok(mut color) = color_queries.p1().single_mut() {
        let next = if settings_glass_active {
            glass_surface_tint(state.top_menu_bg_color)
        } else {
            state.top_menu_bg_color
        };
        color.set_if_neq(BackgroundColor(next));
    }

    if let Ok(mut color) = color_queries.p2().single_mut() {
        let next = if state.explorer_glass && native_glass_state.active {
            glass_surface_tint(state.explorer_bg_color)
        } else {
            state.explorer_bg_color
        };
        color.set_if_neq(BackgroundColor(next));
    }

    if let Ok(mut color) = color_queries.p6().single_mut() {
        color.set_if_neq(BackgroundColor(state.app_bg_color));
    }

    for mut color in color_queries.p7().iter_mut() {
        let next = if settings_glass_active {
            glass_surface_tint(state.app_bg_color)
        } else {
            state.app_bg_color
        };
        color.set_if_neq(BackgroundColor(next));
    }

    for (panel_root, mut color) in color_queries.p3().iter_mut() {
        let next = match panel_root.kind {
            PanelKind::Plain => state.ui_colors.color(UiColor::PlainBackground),
            PanelKind::Processed => Color::NONE,
        };
        color.set_if_neq(BackgroundColor(next));
    }

    for (panel_body, mut color) in color_queries.p4().iter_mut() {
        let next = match panel_body.kind {
            PanelKind::Plain => state.ui_colors.color(UiColor::PlainBackground),
            PanelKind::Processed if processed_glass_active => {
                glass_surface_tint(state.processed_bg_color)
            }
            PanelKind::Processed => state.processed_bg_color,
        };
        color.set_if_neq(BackgroundColor(next));
    }
}

fn glass_surface_tint(_theme_color: Color) -> Color {
    // Windows acrylic and macOS vibrancy supply their own material. Linux
    // compositors supply blur only, so retain the surface's theme tint here.
    #[cfg(target_os = "linux")]
    {
        _theme_color.with_alpha(_theme_color.alpha().min(0.72))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Color::NONE
    }
}

pub(crate) fn sync_top_menu_visibility(
    state: Res<EditorState>,
    mut top_menu_query: Query<&mut Node, With<TopMenuSection>>,
) {
    let display = if state.top_menu_collapsed {
        Display::None
    } else {
        Display::Flex
    };

    for mut node in top_menu_query.iter_mut() {
        if node.display != display {
            node.display = display;
        }
    }
}

pub(crate) fn sync_toolbar_layout(
    mut controls_query: Query<(&ComputedNode, &mut Node), With<ToolbarControls>>,
    mut buttons_query: Query<&mut Node, (With<ToolbarAction>, Without<ToolbarControls>)>,
    mut labels_query: Query<(&ToolbarButtonLabel, &mut Text)>,
) {
    let Ok((computed, mut controls)) = controls_query.single_mut() else {
        return;
    };
    let available_width = computed.size().x * computed.inverse_scale_factor();
    if available_width <= 0.0 {
        return;
    }

    // Keep every action on one row. At half-screen widths, tighten the gaps;
    // use shorter labels only when the full labels would no longer fit.
    let roomy = available_width >= 720.0;
    let compact_labels = available_width < 640.0;
    let padding_x = if roomy {
        12.0
    } else if compact_labels {
        6.0
    } else {
        8.0
    };
    let gap = px(if roomy { 6.0 } else { 4.0 });
    if controls.column_gap != gap {
        controls.column_gap = gap;
    }
    let padding = UiRect::axes(px(padding_x), px(7.0));
    for mut button in &mut buttons_query {
        if button.padding != padding {
            button.padding = padding;
        }
    }
    for (label, mut text) in &mut labels_query {
        let next = if compact_labels {
            match label.action {
                ToolbarAction::OpenWorkspace => "Open",
                ToolbarAction::ExportPdf => "PDF",
                ToolbarAction::StoryQuerySheet => "Story",
                ToolbarAction::ZoomOut => "-",
                ToolbarAction::ZoomIn => "+",
                _ => &label.full,
            }
        } else {
            &label.full
        };
        if text.0 != next {
            **text = next.to_owned();
        }
    }
}

pub(crate) fn sync_rounded_window_surfaces(
    state: Res<EditorState>,
    screen_state: Res<State<UiScreenState>>,
    mut node_queries: ParamSet<(
        Query<
            &mut Node,
            (
                With<EditorScreenRoot>,
                Without<SettingsScreenRoot>,
                Without<KeybindsScreenRoot>,
                Without<ThemeScreenRoot>,
            ),
        >,
        Query<
            &mut Node,
            (
                With<SettingsScreenRoot>,
                Without<EditorScreenRoot>,
                Without<KeybindsScreenRoot>,
                Without<ThemeScreenRoot>,
            ),
        >,
        Query<
            &mut Node,
            (
                With<KeybindsScreenRoot>,
                Without<EditorScreenRoot>,
                Without<SettingsScreenRoot>,
                Without<ThemeScreenRoot>,
            ),
        >,
        Query<
            &mut Node,
            (
                With<ThemeScreenRoot>,
                Without<EditorScreenRoot>,
                Without<SettingsScreenRoot>,
                Without<KeybindsScreenRoot>,
            ),
        >,
        Query<&mut Node, With<TopMenuSection>>,
        Query<&mut Node, With<WorkspaceSidebarPane>>,
        Query<(&PanelRoot, &mut Node)>,
        Query<(&PanelBody, &mut Node)>,
    )>,
    mut status_line_query: Query<
        &mut Node,
        (
            With<StatusLineRoot>,
            Without<EditorScreenRoot>,
            Without<SettingsScreenRoot>,
            Without<KeybindsScreenRoot>,
            Without<ThemeScreenRoot>,
            Without<TopMenuSection>,
            Without<WorkspaceSidebarPane>,
            Without<PanelRoot>,
            Without<PanelBody>,
        ),
    >,
) {
    if !state.is_changed() && !screen_state.is_changed() {
        return;
    }

    let round_window = !state.show_system_titlebar;
    let editor_screen_active = *screen_state.get() == UiScreenState::Editor;
    let editor_top_radius_active = round_window && editor_screen_active && state.top_menu_collapsed;
    let clipped_overflow = if round_window {
        window_surface_overflow(false)
    } else {
        Overflow::visible()
    };

    if let Ok(mut editor_root) = node_queries.p0().single_mut() {
        editor_root.border_radius = if round_window {
            window_surface_border_radius(false)
        } else {
            BorderRadius::ZERO
        };
        editor_root.overflow = clipped_overflow;
    }

    if let Ok(mut settings_root) = node_queries.p1().single_mut() {
        settings_root.border_radius = if round_window {
            window_surface_border_radius(false)
        } else {
            BorderRadius::ZERO
        };
        settings_root.overflow = clipped_overflow;
    }

    if let Ok(mut keybinds_root) = node_queries.p2().single_mut() {
        keybinds_root.border_radius = if round_window {
            window_surface_border_radius(false)
        } else {
            BorderRadius::ZERO
        };
        keybinds_root.overflow = clipped_overflow;
    }

    if let Ok(mut theme_root) = node_queries.p3().single_mut() {
        theme_root.border_radius = if round_window {
            window_surface_border_radius(false)
        } else {
            BorderRadius::ZERO
        };
        theme_root.overflow = clipped_overflow;
    }

    if let Ok(mut top_menu) = node_queries.p4().single_mut() {
        top_menu.border_radius =
            if round_window && editor_screen_active && !state.top_menu_collapsed {
                window_surface_top_border_radius(true, true)
            } else {
                BorderRadius::ZERO
            };
        top_menu.overflow = clipped_overflow;
    }

    if let Ok(mut workspace_sidebar) = node_queries.p5().single_mut() {
        workspace_sidebar.border_radius =
            if editor_top_radius_active && state.workspace_sidebar_visible {
                window_surface_top_border_radius(true, false)
            } else {
                BorderRadius::ZERO
            };
    }

    let plain_visible = state.panel_visible(PanelKind::Plain);
    let processed_visible = state.panel_visible(PanelKind::Processed);
    for (panel_root, mut node) in node_queries.p6().iter_mut() {
        let (round_left, round_right) = if !editor_top_radius_active {
            (false, false)
        } else {
            match panel_root.kind {
                PanelKind::Plain => (!state.workspace_sidebar_visible, !processed_visible),
                PanelKind::Processed => (!plain_visible && !state.workspace_sidebar_visible, true),
            }
        };
        node.border_radius = window_surface_top_border_radius(round_left, round_right);
    }

    for (panel_body, mut node) in node_queries.p7().iter_mut() {
        let (round_left, round_right) = if !editor_top_radius_active {
            (false, false)
        } else {
            match panel_body.kind {
                PanelKind::Plain => (!state.workspace_sidebar_visible, !processed_visible),
                PanelKind::Processed => (!plain_visible && !state.workspace_sidebar_visible, true),
            }
        };
        node.border_radius = window_surface_top_border_radius(round_left, round_right);
    }

    if let Ok(mut status_line) = status_line_query.single_mut() {
        status_line.border_radius = if round_window && editor_screen_active {
            window_surface_bottom_border_radius(true, true)
        } else {
            BorderRadius::ZERO
        };
        status_line.overflow = clipped_overflow;
    }
}

pub(crate) fn sync_panel_display_mode(
    state: Res<EditorState>,
    mut panel_root_query: Query<(&PanelRoot, &mut Node)>,
) {
    for (panel_root, mut node) in panel_root_query.iter_mut() {
        let display = if state.panel_visible(panel_root.kind) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
}

pub(crate) fn sync_settings_ui(
    state: Res<EditorState>,
    screen_state: Res<State<UiScreenState>>,
    mut editor_root_query: Query<
        &mut Node,
        (
            With<EditorScreenRoot>,
            Without<SettingsScreenRoot>,
            Without<KeybindsScreenRoot>,
            Without<ThemeScreenRoot>,
        ),
    >,
    mut settings_root_query: Query<
        &mut Node,
        (
            With<SettingsScreenRoot>,
            Without<EditorScreenRoot>,
            Without<KeybindsScreenRoot>,
            Without<ThemeScreenRoot>,
        ),
    >,
    mut keybinds_root_query: Query<
        &mut Node,
        (
            With<KeybindsScreenRoot>,
            Without<EditorScreenRoot>,
            Without<SettingsScreenRoot>,
            Without<ThemeScreenRoot>,
        ),
    >,
    mut theme_root_query: Query<
        &mut Node,
        (
            With<ThemeScreenRoot>,
            Without<EditorScreenRoot>,
            Without<SettingsScreenRoot>,
            Without<KeybindsScreenRoot>,
        ),
    >,
    mut toggle_label_query: Query<
        (&SettingToggleLabel, &mut Text),
        (
            Without<SettingMarginLabel>,
            Without<KeybindBindingLabel>,
            Without<ThemeColorLabel>,
            Without<ThemeColorValueLabel>,
        ),
    >,
    mut margin_label_query: Query<
        (&SettingMarginLabel, &mut Text),
        (
            Without<SettingToggleLabel>,
            Without<KeybindBindingLabel>,
            Without<ThemeColorLabel>,
            Without<ThemeColorValueLabel>,
        ),
    >,
    mut keybind_label_query: Query<
        (&KeybindBindingLabel, &mut Text),
        (
            Without<SettingToggleLabel>,
            Without<SettingMarginLabel>,
            Without<ThemeColorLabel>,
        ),
    >,
) {
    if let Ok(mut editor_root) = editor_root_query.single_mut() {
        let target = if *screen_state.get() == UiScreenState::Editor {
            Display::Flex
        } else {
            Display::None
        };
        if editor_root.display != target {
            editor_root.display = target;
        }
    }

    if let Ok(mut settings_root) = settings_root_query.single_mut() {
        let target = if *screen_state.get() == UiScreenState::Settings {
            Display::Flex
        } else {
            Display::None
        };
        if settings_root.display != target {
            settings_root.display = target;
        }
    }

    if let Ok(mut keybinds_root) = keybinds_root_query.single_mut() {
        let target = if *screen_state.get() == UiScreenState::Keybinds {
            Display::Flex
        } else {
            Display::None
        };
        if keybinds_root.display != target {
            keybinds_root.display = target;
        }
    }

    if let Ok(mut theme_root) = theme_root_query.single_mut() {
        let target = if *screen_state.get() == UiScreenState::Theme {
            Display::Flex
        } else {
            Display::None
        };
        if theme_root.display != target {
            theme_root.display = target;
        }
    }

    // Skip updating all settings, margin, and keybind labels while in the editor.
    if *screen_state.get() == UiScreenState::Editor {
        return;
    }

    if !state.is_changed() && !screen_state.is_changed() {
        return;
    }

    for (label, mut text) in toggle_label_query.iter_mut() {
        let next = match label.action {
            SettingsAction::DialogueDoubleSpaceNewline => format!(
                "Double space as newline in dialogue (processed modes): {}",
                if state.dialogue_double_space_newline {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            SettingsAction::NonDialogueDoubleSpaceNewline => format!(
                "Double space as newline in non-dialogue (processed modes): {}",
                if state.non_dialogue_double_space_newline {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            SettingsAction::ToggleProcessedPagination => format!(
                "Rendered sheet: {}",
                if state.processed_paginated {
                    "A4 pages"
                } else {
                    "continuous"
                }
            ),
            SettingsAction::ToggleVimMode => {
                format!("Vim mode: {}", if state.vim_enabled { "ON" } else { "OFF" })
            }
            SettingsAction::CycleDefaultViewMode => format!(
                "Default view: {}",
                state.default_view_mode.default_settings_label()
            ),
            SettingsAction::ShowSystemTitlebar => format!(
                "Show system titlebar: {}",
                if state.show_system_titlebar {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            SettingsAction::ToggleProcessedGlass => format!(
                "Processed background glass: {}",
                if state.processed_glass { "ON" } else { "OFF" }
            ),
            SettingsAction::ToggleExplorerGlass => format!(
                "Explorer glass: {}",
                if state.explorer_glass { "ON" } else { "OFF" }
            ),
            SettingsAction::ToggleSettingsGlass => format!(
                "Settings glass: {}",
                if state.settings_glass { "ON" } else { "OFF" }
            ),
            _ => String::new(),
        };
        if text.0 != next {
            **text = next;
        }
    }

    for (label, mut text) in margin_label_query.iter_mut() {
        let value = match label.edge {
            MarginEdge::Left => state.page_margin_left,
            MarginEdge::Right => state.page_margin_right,
            MarginEdge::Top => state.page_margin_top,
            MarginEdge::Bottom => state.page_margin_bottom,
        };
        let next = format!("{value:.1} pt");
        if text.0 != next {
            **text = next;
        }
    }

    for (label, mut text) in keybind_label_query.iter_mut() {
        let next = if state.pending_keybind_capture == Some(label.action) {
            "Press key...".to_string()
        } else {
            binding_display(state.keybinds.binding(label.action))
        };
        if text.0 != next {
            **text = next;
        }
    }
}

pub(crate) fn sync_theme_picker_ui(
    state: Res<EditorState>,
    screen_state: Res<State<UiScreenState>>,
    mut node_queries: ParamSet<(
        Query<(&ThemeColorRow, &mut Node)>,
        Query<&mut Node, With<ThemeColorPickerPanel>>,
        Query<&mut Node, With<ThemeLinkHoverSettingRow>>,
        Query<
            (
                &mut Node,
                Option<&ThemeColorSliderKnob>,
                Option<&ThemeHueSatCursor>,
            ),
            Or<(With<ThemeColorSliderKnob>, With<ThemeHueSatCursor>)>,
        >,
        Query<&mut Node, With<ThemeOverlayContainer>>,
        Query<(&ThemeCategorySection, &mut Node)>,
        Query<(&ThemeColorSliderRow, &mut Node)>,
    )>,
    mut color_queries: ParamSet<(
        Query<(&ThemeColorPreviewSwatch, &mut BackgroundColor)>,
        Query<(&ThemeColorSlider, &mut BackgroundColor)>,
        Query<(&ThemeOverlayTabButton, &Interaction, &mut BackgroundColor)>,
    )>,
    mut text_query: Query<
        (
            &mut Text,
            Option<&ThemeScreenTitleLabel>,
            Option<&ThemeScreenDescriptionLabel>,
            Option<&ThemeColorNameLabel>,
            Option<&ThemeColorValueLabel>,
            Option<&ThemeColorLabel>,
            Option<&ThemeSelectionRgbLabel>,
            Option<&ThemeSelectionHsvLabel>,
            Option<&ThemeSelectionHexLabel>,
            Option<&ThemeLinkHoverValueLabel>,
            Option<&ThemeCurrentNameLabel>,
            Option<&ThemeNameInputText>,
        ),
        Or<(
            With<ThemeScreenTitleLabel>,
            With<ThemeScreenDescriptionLabel>,
            With<ThemeColorNameLabel>,
            With<ThemeColorValueLabel>,
            With<ThemeColorLabel>,
            With<ThemeSelectionRgbLabel>,
            With<ThemeSelectionHsvLabel>,
            With<ThemeSelectionHexLabel>,
            With<ThemeLinkHoverValueLabel>,
            With<ThemeCurrentNameLabel>,
            With<ThemeNameInputText>,
        )>,
    >,
    wheel_size_query: Query<&ComputedNode, With<ThemeHueSatWheel>>,
) {
    let theme_active = *screen_state.get() == UiScreenState::Theme || state.theme_overlay_open;
    for mut container in node_queries.p4().iter_mut() {
        let target = if state.theme_overlay_open && *screen_state.get() == UiScreenState::Editor {
            Display::Flex
        } else {
            Display::None
        };
        if container.display != target {
            container.display = target;
        }
    }

    for mut picker_panel in node_queries.p1().iter_mut() {
        let target = if state.theme_color_picker_open
            && state.theme_category != ThemeCategory::Glass
            && state.theme_category != ThemeCategory::Themes
            && theme_active
        {
            Display::Flex
        } else {
            Display::None
        };
        if picker_panel.display != target {
            picker_panel.display = target;
        }
    }

    if !theme_active {
        return;
    }

    if !state.is_changed() && !screen_state.is_changed() {
        return;
    }

    for (section, mut node) in node_queries.p5().iter_mut() {
        node.display = if state.theme_category == section.0 {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (row, mut node) in node_queries.p6().iter_mut() {
        node.display = if matches!(row.0, ThemeSliderChannel::Alpha)
            && state.theme_color_target == ThemeColorTarget::PaperBackground {
            Display::None
        } else {
            Display::Flex
        };
    }

    for (tab, interaction, mut bg) in color_queries.p2().iter_mut() {
        bg.0 = if tab.0 == state.theme_category && *interaction == Interaction::None {
            state.ui_colors.color(UiColor::ActiveBackground)
        } else {
            themed_button_color(&state, *interaction)
        };
    }

    for (_row, mut node) in node_queries.p0().iter_mut() {
        node.display = Display::Flex;
    }
    for mut node in node_queries.p2().iter_mut() {
        node.display = Display::Flex;
    }

    for (swatch, mut color) in color_queries.p0().iter_mut() {
        color.0 = theme_color_for_target(&state, swatch.target);
    }

    let active_target = state.theme_color_target;
    let active_rgba = active_theme_rgba(&state);

    let rgb = Vec3::new(active_rgba.x, active_rgba.y, active_rgba.z);
    let (hue, saturation, value) = rgb_to_hsv(rgb);
    let rgb_255 = (
        (active_rgba.x * 255.0).round().clamp(0.0, 255.0) as u8,
        (active_rgba.y * 255.0).round().clamp(0.0, 255.0) as u8,
        (active_rgba.z * 255.0).round().clamp(0.0, 255.0) as u8,
        (active_rgba.w * 255.0).round().clamp(0.0, 255.0) as u8,
    );

    for (
        mut text,
        title_label,
        description_label,
        name_label,
        value_label,
        channel_label,
        rgb_label,
        hsv_label,
        hex_label,
        hover_value_label,
        current_name_label,
        name_input_text,
    ) in text_query.iter_mut()
    {
        if title_label.is_some() {
            **text = active_target.screen_title().to_string();
            continue;
        }

        if description_label.is_some() {
            **text = active_target.screen_description().to_string();
            continue;
        }

        if let Some(name_label) = name_label {
            **text = name_label.target.color_label().to_string();
            continue;
        }

        if let Some(value_label) = value_label {
            let rgba = theme_rgba_for_target(&state, value_label.target);
            let r = (rgba.x * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = (rgba.y * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = (rgba.z * 255.0).round().clamp(0.0, 255.0) as u8;
            **text = format!("#{r:02X}{g:02X}{b:02X}");
            continue;
        }

        if let Some(channel_label) = channel_label {
            let channel_value = match channel_label.channel {
                ThemeColorChannel::Hue => hue,
                ThemeColorChannel::Saturation => saturation,
                ThemeColorChannel::Red => active_rgba.x,
                ThemeColorChannel::Green => active_rgba.y,
                ThemeColorChannel::Blue => active_rgba.z,
                ThemeColorChannel::Alpha => active_rgba.w,
                ThemeColorChannel::Value => value,
            };
            **text = format!("{channel_value:.3}");
            continue;
        }

        if rgb_label.is_some() {
            **text = format!(
                "RGBA: {} {} {} {}",
                rgb_255.0, rgb_255.1, rgb_255.2, rgb_255.3
            );
            continue;
        }

        if hsv_label.is_some() {
            **text = format!(
                "HSV: {:.1}° {:.1}% {:.1}%",
                hue * 360.0,
                saturation * 100.0,
                value * 100.0
            );
            continue;
        }

        if hex_label.is_some() {
            **text = format!(
                "HEX: #{:02X}{:02X}{:02X}{:02X}",
                rgb_255.0, rgb_255.1, rgb_255.2, rgb_255.3
            );
            continue;
        }

        if hover_value_label.is_some() {
            **text = format_hsv_value_adjustment_label(state.link_hover_hsv_value_adjustment);
            continue;
        }

        if current_name_label.is_some() {
            **text = format!("Theme: {}", state.current_theme_name);
            continue;
        }

        if name_input_text.is_some() {
            **text = if state.theme_name_input.is_empty() {
                "Type new name...".to_string()
            } else {
                state.theme_name_input.clone()
            };
            continue;
        }
    }

    for (slider, mut color) in color_queries.p1().iter_mut() {
        color.0 = match slider.channel {
            ThemeSliderChannel::Hue => Color::srgb(0.56, 0.56, 0.58),
            ThemeSliderChannel::Saturation => {
                let vibrant = hsv_to_rgb(hue, 1.0, value.max(0.2));
                Color::srgb(vibrant.x, vibrant.y, vibrant.z)
            }
            ThemeSliderChannel::Red => Color::srgb(0.74, 0.28, 0.28),
            ThemeSliderChannel::Green => Color::srgb(0.28, 0.68, 0.35),
            ThemeSliderChannel::Blue => Color::srgb(0.30, 0.44, 0.78),
            ThemeSliderChannel::Value => Color::srgb(0.42, 0.42, 0.45),
            ThemeSliderChannel::Alpha => Color::srgb(0.58, 0.58, 0.60),
        };
    }

    let wheel_size = wheel_size_query
        .iter()
        .next()
        .map(|computed| {
            Vec2::new(
                computed.size().x * computed.inverse_scale_factor(),
                computed.size().y * computed.inverse_scale_factor(),
            )
        })
        .unwrap_or(Vec2::splat(THEME_OVERLAY_WHEEL_SIZE));
    let wheel_radius = wheel_size.x.min(wheel_size.y) * 0.5;
    let cursor_half = 5.0;
    let angle = hue * std::f32::consts::TAU;
    let cursor_x = angle.cos() * saturation;
    let cursor_y = angle.sin() * saturation;
    for (mut node, slider_knob, wheel_cursor) in node_queries.p3().iter_mut() {
        if let Some(knob) = slider_knob {
            let t = match knob.channel {
                ThemeSliderChannel::Hue => hue,
                ThemeSliderChannel::Saturation => saturation,
                ThemeSliderChannel::Red => active_rgba.x,
                ThemeSliderChannel::Green => active_rgba.y,
                ThemeSliderChannel::Blue => active_rgba.z,
                ThemeSliderChannel::Alpha => active_rgba.w,
                ThemeSliderChannel::Value => value,
            }
            .clamp(0.0, 1.0);
            node.left = px((THEME_OVERLAY_SLIDER_WIDTH - THEME_COLOR_SLIDER_KNOB_WIDTH) * t);
            node.top = px(0.0);
            continue;
        }

        if wheel_cursor.is_some() {
            node.left = px(wheel_size.x * 0.5 + cursor_x * wheel_radius - cursor_half);
            node.top = px(wheel_size.y * 0.5 - cursor_y * wheel_radius - cursor_half);
        }
    }
}
#[allow(unused_imports)]
use super::*;
