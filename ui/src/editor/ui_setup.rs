fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
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
        folder_closed: load_workspace_icon_image(
            &mut images,
            "assets/icons/folder-closed.svg",
            16,
        ),
        folder_open: load_workspace_icon_image(&mut images, "assets/icons/folder-open.svg", 16),
    };
    let checklist_icons = ChecklistIcons {
        unchecked: load_workspace_icon_image(
            &mut images,
            "assets/icons/checklist-unchecked.svg",
            16,
        ),
        checked: load_workspace_icon_image(
            &mut images,
            "assets/icons/checklist-checked.svg",
            16,
        ),
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
                ..default()
            },
            BackgroundColor(COLOR_APP_BG),
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
                            padding: UiRect::axes(px(12.0), px(8.0)),
                            ..default()
                        },
                        TopMenuSection,
                        children![
                            (
                                Text::new("BasScript"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(COLOR_TEXT_MAIN),
                            ),
                            (
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(8.0),
                                    ..default()
                                },
                                children![
                                    toolbar_button(
                                        font.clone(),
                                        "Open Folder",
                                        ToolbarAction::OpenWorkspace,
                                    ),
                                    toolbar_button(font.clone(), "Save As", ToolbarAction::SaveAs),
                                    toolbar_button(font.clone(), "Zoom -", ToolbarAction::ZoomOut),
                                    toolbar_button(font.clone(), "Zoom +", ToolbarAction::ZoomIn),
                                    toolbar_button(font.clone(), "Settings", ToolbarAction::Settings),
                                ],
                            )
                        ],
                    ),
                    (
                        Node {
                            width: percent(100.0),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Row,
                            column_gap: px(0.0),
                            padding: UiRect::all(px(0.0)),
                            ..default()
                        },
                        EditorBodyRow,
                        RelativeCursorPosition::default(),
                        children![
                            workspace_sidebar_bundle(font.clone()),
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
                                        children![panel_bundle(font.clone(), PanelKind::Plain)],
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
                                        children![panel_bundle(font.clone(), PanelKind::Processed)],
                                    ),
                                ],
                            )
                        ],
                    ),
                    status_line_bundle(font.clone())
                ],
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12.0),
                    padding: UiRect::axes(px(18.0), px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.86, 0.88, 0.90)),
                SettingsScreenRoot,
                children![
                    (
                        Text::new("Settings"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                    ),
                    (
                        Text::new("Processed page margins and formatting options."),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MUTED),
                    ),
                    settings_toggle_button(
                        font.clone(),
                        SettingsAction::DialogueDoubleSpaceNewline,
                    ),
                    settings_toggle_button(
                        font.clone(),
                        SettingsAction::NonDialogueDoubleSpaceNewline,
                    ),
                    settings_toggle_button(font.clone(), SettingsAction::ShowSystemTitlebar),
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
                    settings_action_button(font.clone(), "Theme", SettingsAction::OpenTheme),
                    settings_action_button(font.clone(), "Keybinds", SettingsAction::OpenKeybinds),
                    settings_action_button(font.clone(), "Back to editor", SettingsAction::BackToEditor),
                ],
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10.0),
                    padding: UiRect::axes(px(18.0), px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.86, 0.88, 0.90)),
                KeybindsScreenRoot,
                children![
                    (
                        Text::new("Keybinds"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                    ),
                    (
                        Text::new("Keyboard shortcuts and mouse controls."),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MUTED),
                    ),
                    (
                        Text::new("Keyboard"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                    ),
                    (
                        Text::new("Click a binding button, then press a key (Esc cancels)."),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MUTED),
                    ),
                    keybind_setting_row(font.clone(), ShortcutAction::OpenWorkspace),
                    keybind_setting_row(font.clone(), ShortcutAction::SaveAs),
                    keybind_setting_row(font.clone(), ShortcutAction::Undo),
                    keybind_setting_row(font.clone(), ShortcutAction::Redo),
                    keybind_setting_row(font.clone(), ShortcutAction::ZoomIn),
                    keybind_setting_row(font.clone(), ShortcutAction::ZoomOut),
                    keybind_row(font.clone(), "Cmd/Ctrl+Mouse wheel", "Zoom"),
                    keybind_setting_row(font.clone(), ShortcutAction::PlainView),
                    keybind_setting_row(font.clone(), ShortcutAction::ProcessedView),
                    keybind_setting_row(
                        font.clone(),
                        ShortcutAction::ProcessedRawCurrentLineView,
                    ),
                    keybind_setting_row(font.clone(), ShortcutAction::ToggleExplorer),
                    keybind_setting_row(font.clone(), ShortcutAction::ToggleTopMenu),
                    keybind_row(font.clone(), "Arrow keys", "Move cursor"),
                    keybind_row(font.clone(), "Home / End", "Move to line start/end"),
                    keybind_row(font.clone(), "Page Up / Page Down", "Move by viewport"),
                    keybind_row(font.clone(), "Escape", "Cancel middle-click autoscroll"),
                    (
                        Text::new("Mouse"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                    ),
                    keybind_row(font.clone(), "Mouse wheel", "Scroll active pane vertically"),
                    keybind_row(font.clone(), "Shift+Mouse wheel", "Scroll active pane horizontally"),
                    keybind_row(font.clone(), "Ctrl+Left drag", "Scroll active pane"),
                    keybind_row(
                        font.clone(),
                        "Middle click + hold",
                        "Autoscroll (release middle button to stop)",
                    ),
                    keybind_row(font.clone(), "Left click", "Place cursor"),
                    keybind_row(font.clone(), "Right click", "Cancel middle-click autoscroll"),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8.0),
                            margin: UiRect::top(px(8.0)),
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
                ],
            ));

            root.spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10.0),
                    padding: UiRect::axes(px(18.0), px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.86, 0.88, 0.90)),
                ThemeScreenRoot,
                children![
                    (
                        Text::new("Theme"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                    ),
                    (
                        Text::new("Adjust editor theme colors."),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MUTED),
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Start,
                            column_gap: px(12.0),
                            ..default()
                        },
                        children![
                            (
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8.0),
                                    ..default()
                                },
                                children![theme_selection_background_row(font.clone())],
                            ),
                            (
                                Node {
                                    display: Display::None,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8.0),
                                    padding: UiRect::all(px(10.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.82, 0.84, 0.86)),
                                ThemeColorPickerPanel,
                                children![
                                    theme_visual_picker(font.clone(), hue_sat_wheel.clone()),
                                    (
                                        Text::new(""),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(COLOR_TEXT_MAIN),
                                        ThemeSelectionRgbLabel,
                                    ),
                                    (
                                        Text::new(""),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(COLOR_TEXT_MAIN),
                                        ThemeSelectionHsvLabel,
                                    ),
                                    (
                                        Text::new(""),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(COLOR_TEXT_MAIN),
                                        ThemeSelectionHexLabel,
                                    ),
                                ],
                            ),
                        ],
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8.0),
                            margin: UiRect::top(px(8.0)),
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
                ],
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
                            font: font.clone(),
                            font_size: 34.0,
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
                            font: font.clone(),
                            font_size: 15.0,
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
                            font: font.clone(),
                            font_size: 11.0,
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
                            font: font.clone(),
                            font_size: 11.0,
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
                            font: font.clone(),
                            font_size: 11.0,
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
                            font: font.clone(),
                            font_size: 11.0,
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
        });
}

fn load_workspace_icon_image(
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

fn generate_theme_color_wheel_image(images: &mut Assets<Image>, diameter: u32) -> Handle<Image> {
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

fn load_font_with_fallback(
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

fn rasterize_svg_to_image(
    images: &mut Assets<Image>,
    icon_path: &str,
    max_side_px: u32,
) -> Result<Handle<Image>, String> {
    let icon_fs_path = resolve_workspace_asset_path(icon_path)
        .ok_or_else(|| format!("cannot resolve icon path {}", icon_path))?;
    let svg_data = fs::read(&icon_fs_path).map_err(|error| {
        format!(
            "cannot read icon file {}: {error}",
            icon_fs_path.display()
        )
    })?;
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

fn resolve_workspace_asset_path(path: &str) -> Option<PathBuf> {
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

fn setup_processed_papers(
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
                        paper
                            .spawn((
                                Text::new(""),
                                TextLayout::new_with_no_wrap(),
                                TextFont {
                                    font: slot_font.clone(),
                                    font_size: FONT_SIZE,
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
                                GlobalZIndex(1),
                                ProcessedPaperText { slot },
                            ))
                            .with_children(|text_parent| {
                                for line_offset in 0..span_capacity {
                                    text_parent.spawn((
                                        TextSpan::new(""),
                                        TextFont {
                                            font: slot_font.clone(),
                                            font_size: FONT_SIZE,
                                            ..default()
                                        },
                                        LineHeight::Px(LINE_HEIGHT),
                                        TextColor(COLOR_ACTION),
                                        ProcessedPaperLineSpan { slot, line_offset },
                                    ));
                                }
                            });

                        for line_offset in 0..span_capacity {
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
            paper
                .spawn((
                    Text::new(""),
                    TextLayout::new_with_no_wrap(),
                    TextFont {
                        font: regular_font.clone(),
                        font_size: FONT_SIZE,
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
                    GlobalZIndex(1),
                    ProcessedPaperText { slot },
                ))
                .with_children(|text_parent| {
                    for line_offset in 0..span_capacity {
                        text_parent.spawn((
                            TextSpan::new(""),
                            TextFont {
                                font: regular_font.clone(),
                                font_size: FONT_SIZE,
                                ..default()
                            },
                            LineHeight::Px(LINE_HEIGHT),
                            TextColor(COLOR_ACTION),
                            ProcessedPaperLineSpan { slot, line_offset },
                        ));
                        }
                    });

            for line_offset in 0..span_capacity {
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

fn toolbar_button(font: Handle<Font>, label: &str, action: ToolbarAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            padding: UiRect::axes(px(12.0), px(6.0)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(label),
            TextFont {
                font,
                font_size: 13.0,
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
        )],
    )
}

fn settings_toggle_button(font: Handle<Font>, action: SettingsAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            padding: UiRect::axes(px(12.0), px(6.0)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(""),
            TextFont {
                font,
                font_size: 13.0,
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
            SettingToggleLabel { action },
        )],
    )
}

fn settings_action_button(font: Handle<Font>, label: &str, action: SettingsAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            padding: UiRect::axes(px(12.0), px(6.0)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        children![(
            Text::new(label),
            TextFont {
                font,
                font_size: 13.0,
                ..default()
            },
            TextColor(COLOR_TEXT_MAIN),
        )],
    )
}

fn keybind_setting_row(font: Handle<Font>, action: ShortcutAction) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            ..default()
        },
        children![
            (
                Text::new(shortcut_action_description(action)),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
                Node {
                    width: px(260.0),
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
                BackgroundColor(BUTTON_NORMAL),
                children![(
                    Text::new(""),
                    TextFont {
                        font,
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MAIN),
                    KeybindBindingLabel { action },
                )],
            ),
        ],
    )
}

fn keybind_row(font: Handle<Font>, binding: &str, description: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(10.0),
            ..default()
        },
        children![
            (
                Text::new(binding),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
                Node {
                    width: px(220.0),
                    ..default()
                },
            ),
            (
                Text::new(description),
                TextFont {
                    font,
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
            ),
        ],
    )
}

fn margin_setting_row(
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
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
            ),
            settings_action_button(font.clone(), "-", decrease_action),
            (
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
                SettingMarginLabel { edge },
            ),
            settings_action_button(font, "+", increase_action),
        ],
    )
}

fn theme_selection_background_row(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            ..default()
        },
        children![
            (
                Text::new("Selection background"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
                Node {
                    width: px(170.0),
                    ..default()
                },
            ),
            (
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
                ThemeSelectionBackgroundValueLabel,
                Node {
                    width: px(220.0),
                    ..default()
                },
            ),
            (
                Button,
                SettingsAction::ToggleThemeColorPicker,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8.0),
                    padding: UiRect::axes(px(10.0), px(6.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_NORMAL),
                children![
                    (
                        Node {
                            width: px(14.0),
                            height: px(14.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ThemeColorPreviewSwatch,
                    ),
                    (
                        Text::new("Pick"),
                        TextFont {
                            font,
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                    ),
                ],
            ),
        ],
    )
}

fn theme_visual_picker(font: Handle<Font>, hue_sat_wheel: Handle<Image>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Start,
            column_gap: px(12.0),
            ..default()
        },
        children![
            (
                Node {
                    width: px(THEME_COLOR_WHEEL_SIZE),
                    height: px(THEME_COLOR_WHEEL_SIZE),
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
                        left: px((THEME_COLOR_WHEEL_SIZE - 10.0) * 0.5),
                        top: px((THEME_COLOR_WHEEL_SIZE - 10.0) * 0.5),
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
                    row_gap: px(6.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                children![
                    theme_color_slider_row(font.clone(), "Hue", ThemeSliderChannel::Hue),
                    theme_color_slider_row(
                        font.clone(),
                        "Sat",
                        ThemeSliderChannel::Saturation,
                    ),
                    theme_color_slider_row(font.clone(), "Value", ThemeSliderChannel::Value),
                    theme_color_slider_row(font.clone(), "Red", ThemeSliderChannel::Red),
                    theme_color_slider_row(font.clone(), "Green", ThemeSliderChannel::Green),
                    theme_color_slider_row(font.clone(), "Blue", ThemeSliderChannel::Blue),
                    theme_color_slider_row(font.clone(), "Alpha", ThemeSliderChannel::Alpha),
                ],
            )
        ],
    )
}

fn theme_color_slider_row(
    font: Handle<Font>,
    label: &str,
    channel: ThemeSliderChannel,
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
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
                Node {
                    width: px(40.0),
                    ..default()
                },
            ),
            (
                Node {
                    width: px(THEME_COLOR_SLIDER_WIDTH),
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
                    font,
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
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
                    width: px(44.0),
                    ..default()
                },
            ),
        ],
    )
}

fn panel_splitter_bundle(kind: PanelSplitter) -> impl Bundle {
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

fn panel_bundle(font: Handle<Font>, kind: PanelKind) -> impl Bundle {
    let body_color = match kind {
        PanelKind::Plain => COLOR_PANEL_BODY_PLAIN,
        PanelKind::Processed => COLOR_PANEL_BODY_PROCESSED,
    };

    (
        Node {
            flex_grow: 1.0,
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        PanelRoot { kind },
        BackgroundColor(COLOR_PANEL_BG),
        children![
            (
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
                            BackgroundColor(Color::srgba(0.12, 0.12, 0.13, 0.35)),
                            Visibility::Hidden,
                            ZIndex(2),
                            PanelCaret { kind },
                        ),
                        (
                            Text::new(""),
                            TextLayout::new_with_no_wrap(),
                            TextFont {
                                font: font.clone(),
                                font_size: FONT_SIZE,
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
                )],
            )
        ],
    )
}

fn handle_toolbar_buttons(
    _dialog_main_thread: NonSend<DialogMainThreadMarker>,
    interaction_query: Query<(&Interaction, &ToolbarAction), (Changed<Interaction>, With<Button>)>,
    primary_window_query: Query<&RawHandleWrapper, With<PrimaryWindow>>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    mut state: ResMut<EditorState>,
    mut dialogs: ResMut<DialogState>,
    mut next_screen_state: ResMut<NextState<UiScreenState>>,
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
                open_workspace_dialog(&mut state, &mut dialogs, parent_handle)
            }
            ToolbarAction::SaveAs => open_save_dialog(&mut state, &mut dialogs, parent_handle),
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
                next_screen_state.set(UiScreenState::Settings);
                state.status_message = "Opened settings.".to_string();
            }
        }
    }
}

fn style_toolbar_buttons(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(
                With<ToolbarAction>,
                With<SettingsAction>,
                With<KeybindRebindButton>,
            )>,
        ),
    >,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        color.0 = match *interaction {
            Interaction::Pressed => BUTTON_PRESSED,
            Interaction::Hovered => BUTTON_HOVER,
            Interaction::None => BUTTON_NORMAL,
        };
    }
}

fn handle_settings_buttons(
    interaction_query: Query<(&Interaction, &SettingsAction), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<EditorState>,
    mut next_screen_state: ResMut<NextState<UiScreenState>>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut settings_changed = false;
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
            SettingsAction::ShowSystemTitlebar => {
                state.show_system_titlebar = !state.show_system_titlebar;
                settings_changed = true;
                state.status_message = format!(
                    "System titlebar: {}",
                    if state.show_system_titlebar { "ON" } else { "OFF" }
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
            SettingsAction::OpenTheme => {
                next_screen_state.set(UiScreenState::Theme);
                state.status_message = "Opened theme.".to_string();
            }
            SettingsAction::ToggleThemeColorPicker => {
                state.theme_color_picker_open = !state.theme_color_picker_open;
                state.status_message = if state.theme_color_picker_open {
                    "Opened color picker.".to_string()
                } else {
                    "Closed color picker.".to_string()
                };
            }
            SettingsAction::OpenKeybinds => {
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
    }
}

fn handle_theme_color_picker_input(
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
        let current_rgb = Vec3::new(
            state.selection_bg_rgba.x,
            state.selection_bg_rgba.y,
            state.selection_bg_rgba.z,
        );
        let (_, _, value) = rgb_to_hsv(current_rgb);
        let next_rgb = hsv_to_rgb(hue, saturation, value);
        state.selection_bg_rgba.x = next_rgb.x;
        state.selection_bg_rgba.y = next_rgb.y;
        state.selection_bg_rgba.z = next_rgb.z;
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
        let next = ((cursor_x - THEME_COLOR_SLIDER_KNOB_WIDTH * 0.5) / usable_width).clamp(0.0, 1.0);
        match slider.channel {
            ThemeSliderChannel::Hue => {
                let current_rgb = Vec3::new(
                    state.selection_bg_rgba.x,
                    state.selection_bg_rgba.y,
                    state.selection_bg_rgba.z,
                );
                let (_, saturation, value) = rgb_to_hsv(current_rgb);
                let next_rgb = hsv_to_rgb(next, saturation, value);
                state.selection_bg_rgba.x = next_rgb.x;
                state.selection_bg_rgba.y = next_rgb.y;
                state.selection_bg_rgba.z = next_rgb.z;
            }
            ThemeSliderChannel::Saturation => {
                let current_rgb = Vec3::new(
                    state.selection_bg_rgba.x,
                    state.selection_bg_rgba.y,
                    state.selection_bg_rgba.z,
                );
                let (hue, _, value) = rgb_to_hsv(current_rgb);
                let next_rgb = hsv_to_rgb(hue, next, value);
                state.selection_bg_rgba.x = next_rgb.x;
                state.selection_bg_rgba.y = next_rgb.y;
                state.selection_bg_rgba.z = next_rgb.z;
            }
            ThemeSliderChannel::Red => state.selection_bg_rgba.x = next,
            ThemeSliderChannel::Green => state.selection_bg_rgba.y = next,
            ThemeSliderChannel::Blue => state.selection_bg_rgba.z = next,
            ThemeSliderChannel::Alpha => state.selection_bg_rgba.w = next,
            ThemeSliderChannel::Value => {
                let current_rgb = Vec3::new(
                    state.selection_bg_rgba.x,
                    state.selection_bg_rgba.y,
                    state.selection_bg_rgba.z,
                );
                let (hue, saturation, _) = rgb_to_hsv(current_rgb);
                let next_rgb = hsv_to_rgb(hue, saturation, next);
                state.selection_bg_rgba.x = next_rgb.x;
                state.selection_bg_rgba.y = next_rgb.y;
                state.selection_bg_rgba.z = next_rgb.z;
            }
        }
        theme_changed = true;
    }

    if theme_changed {
        sync_selection_background_color(&mut state);
        let theme = theme_settings_from_state(&state);
        if let Err(error) = save_theme_settings(&theme) {
            state.status_message = format!("Theme save failed: {error}");
        }
    }
}

fn handle_keybind_buttons(
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
            "Press a key for {} (Esc to cancel).",
            shortcut_action_label(button.action)
        );
    }
}

fn capture_keybind_input(
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

        if binding_key_name(key_code).is_none() {
            state.status_message = format!(
                "Unsupported key for {}. Use letters, digits, '=' or '-'.",
                shortcut_action_label(action)
            );
            continue;
        }

        let binding = ShortcutBinding {
            key: key_code,
            shift: shift_modifier_pressed(&keys),
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

fn sync_top_menu_visibility(
    state: Res<EditorState>,
    mut top_menu_query: Query<&mut Node, With<TopMenuSection>>,
) {
    let display = if state.top_menu_collapsed {
        Display::None
    } else {
        Display::Flex
    };

    for mut node in top_menu_query.iter_mut() {
        node.display = display;
    }
}

fn sync_panel_display_mode(
    state: Res<EditorState>,
    mut panel_root_query: Query<(&PanelRoot, &mut Node)>,
) {
    for (panel_root, mut node) in panel_root_query.iter_mut() {
        node.display = if state.panel_visible(panel_root.kind) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn sync_settings_ui(
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
            Without<ThemeSelectionBackgroundValueLabel>,
        ),
    >,
    mut margin_label_query: Query<
        (&SettingMarginLabel, &mut Text),
        (
            Without<SettingToggleLabel>,
            Without<KeybindBindingLabel>,
            Without<ThemeColorLabel>,
            Without<ThemeSelectionBackgroundValueLabel>,
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
        editor_root.display = if *screen_state.get() == UiScreenState::Editor {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut settings_root) = settings_root_query.single_mut() {
        settings_root.display = if *screen_state.get() == UiScreenState::Settings {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut keybinds_root) = keybinds_root_query.single_mut() {
        keybinds_root.display = if *screen_state.get() == UiScreenState::Keybinds {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut theme_root) = theme_root_query.single_mut() {
        theme_root.display = if *screen_state.get() == UiScreenState::Theme {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (label, mut text) in toggle_label_query.iter_mut() {
        **text = match label.action {
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
            SettingsAction::ShowSystemTitlebar => format!(
                "Show system titlebar: {}",
                if state.show_system_titlebar {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            _ => String::new(),
        };
    }

    for (label, mut text) in margin_label_query.iter_mut() {
        let value = match label.edge {
            MarginEdge::Left => state.page_margin_left,
            MarginEdge::Right => state.page_margin_right,
            MarginEdge::Top => state.page_margin_top,
            MarginEdge::Bottom => state.page_margin_bottom,
        };
        **text = format!("{value:.1} pt");
    }

    for (label, mut text) in keybind_label_query.iter_mut() {
        **text = if state.pending_keybind_capture == Some(label.action) {
            "Press key...".to_string()
        } else {
            binding_display(state.keybinds.binding(label.action))
        };
    }
}

fn sync_theme_picker_ui(
    state: Res<EditorState>,
    screen_state: Res<State<UiScreenState>>,
    mut node_queries: ParamSet<(
        Query<&mut Node, With<ThemeColorPickerPanel>>,
        Query<
            (
                &mut Node,
                Option<&ThemeColorSliderKnob>,
                Option<&ThemeHueSatCursor>,
            ),
            Or<(With<ThemeColorSliderKnob>, With<ThemeHueSatCursor>)>,
        >,
    )>,
    mut color_queries: ParamSet<(
        Query<&mut BackgroundColor, With<ThemeColorPreviewSwatch>>,
        Query<(&ThemeColorSlider, &mut BackgroundColor)>,
    )>,
    mut text_query: Query<
        (
            &mut Text,
            Option<&ThemeSelectionBackgroundValueLabel>,
            Option<&ThemeColorLabel>,
            Option<&ThemeSelectionRgbLabel>,
            Option<&ThemeSelectionHsvLabel>,
            Option<&ThemeSelectionHexLabel>,
        ),
        Or<(
            With<ThemeSelectionBackgroundValueLabel>,
            With<ThemeColorLabel>,
            With<ThemeSelectionRgbLabel>,
            With<ThemeSelectionHsvLabel>,
            With<ThemeSelectionHexLabel>,
        )>,
    >,
    wheel_size_query: Query<&ComputedNode, With<ThemeHueSatWheel>>,
) {
    if let Ok(mut picker_panel) = node_queries.p0().single_mut() {
        picker_panel.display =
            if state.theme_color_picker_open && *screen_state.get() == UiScreenState::Theme {
                Display::Flex
            } else {
                Display::None
            };
    }

    for mut swatch in color_queries.p0().iter_mut() {
        swatch.0 = state.selection_bg_color;
    }

    let rgb = Vec3::new(
        state.selection_bg_rgba.x,
        state.selection_bg_rgba.y,
        state.selection_bg_rgba.z,
    );
    let (hue, saturation, value) = rgb_to_hsv(rgb);
    let rgb_255 = (
        (state.selection_bg_rgba.x * 255.0).round().clamp(0.0, 255.0) as u8,
        (state.selection_bg_rgba.y * 255.0).round().clamp(0.0, 255.0) as u8,
        (state.selection_bg_rgba.z * 255.0).round().clamp(0.0, 255.0) as u8,
        (state.selection_bg_rgba.w * 255.0).round().clamp(0.0, 255.0) as u8,
    );

    for (mut text, value_label, channel_label, rgb_label, hsv_label, hex_label) in
        text_query.iter_mut()
    {
        if value_label.is_some() {
            **text = format!(
                "({:.3}, {:.3}, {:.3}, {:.3})",
                state.selection_bg_rgba.x,
                state.selection_bg_rgba.y,
                state.selection_bg_rgba.z,
                state.selection_bg_rgba.w
            );
            continue;
        }

        if let Some(channel_label) = channel_label {
            let channel_value = match channel_label.channel {
                ThemeColorChannel::Hue => hue,
                ThemeColorChannel::Saturation => saturation,
                ThemeColorChannel::Red => state.selection_bg_rgba.x,
                ThemeColorChannel::Green => state.selection_bg_rgba.y,
                ThemeColorChannel::Blue => state.selection_bg_rgba.z,
                ThemeColorChannel::Alpha => state.selection_bg_rgba.w,
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
        .unwrap_or(Vec2::splat(THEME_COLOR_WHEEL_SIZE));
    let wheel_radius = wheel_size.x.min(wheel_size.y) * 0.5;
    let cursor_half = 5.0;
    let angle = hue * std::f32::consts::TAU;
    let cursor_x = angle.cos() * saturation;
    let cursor_y = angle.sin() * saturation;
    for (mut node, slider_knob, wheel_cursor) in node_queries.p1().iter_mut() {
        if let Some(knob) = slider_knob {
            let t = match knob.channel {
                ThemeSliderChannel::Hue => hue,
                ThemeSliderChannel::Saturation => saturation,
                ThemeSliderChannel::Red => state.selection_bg_rgba.x,
                ThemeSliderChannel::Green => state.selection_bg_rgba.y,
                ThemeSliderChannel::Blue => state.selection_bg_rgba.z,
                ThemeSliderChannel::Alpha => state.selection_bg_rgba.w,
                ThemeSliderChannel::Value => value,
            }
            .clamp(0.0, 1.0);
            node.left = px((THEME_COLOR_SLIDER_WIDTH - THEME_COLOR_SLIDER_KNOB_WIDTH) * t);
            node.top = px(0.0);
            continue;
        }

        if wheel_cursor.is_some() {
            node.left = px(wheel_size.x * 0.5 + cursor_x * wheel_radius - cursor_half);
            node.top = px(wheel_size.y * 0.5 - cursor_y * wheel_radius - cursor_half);
        }
    }
}
