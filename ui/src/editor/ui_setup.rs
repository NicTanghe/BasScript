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
    let font = fonts.regular.clone();
    commands.insert_resource(fonts);
    commands.insert_resource(workspace_icons);
    commands.insert_resource(checklist_icons);

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
                        children![
                            workspace_sidebar_bundle(font.clone()),
                            (
                                Node {
                                    flex_grow: 1.0,
                                    height: percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(0.0),
                                    ..default()
                                },
                                children![
                                    panel_bundle(font.clone(), PanelKind::Plain),
                                    panel_bundle(font.clone(), PanelKind::Processed),
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
                    keybind_row(font.clone(), "Cmd/Ctrl+O", "Open workspace folder"),
                    keybind_row(font.clone(), "Cmd/Ctrl+S", "Save As dialog"),
                    keybind_row(font.clone(), "Cmd/Ctrl+Z", "Undo"),
                    keybind_row(font.clone(), "Cmd/Ctrl+Shift+Z", "Redo"),
                    keybind_row(font.clone(), "Cmd/Ctrl+=", "Zoom in"),
                    keybind_row(font.clone(), "Cmd/Ctrl+-", "Zoom out"),
                    keybind_row(font.clone(), "Cmd/Ctrl+Mouse wheel", "Zoom"),
                    keybind_row(font.clone(), "Cmd/Ctrl+B", "Toggle top menu"),
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

fn workspace_sidebar_bundle(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: px(280.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            padding: UiRect::axes(px(10.0), px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.86, 0.87, 0.89)),
        children![
            (
                Text::new("No workspace opened."),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
                WorkspaceRootLabel,
            ),
            (
                Node {
                    width: percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                WorkspaceFileList,
            ),
        ],
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
                                left: px(TEXT_PADDING_X),
                                top: px(TEXT_PADDING_Y),
                                width: px(CARET_WIDTH),
                                height: px(LINE_HEIGHT),
                                ..default()
                            },
                            UiTransform::default(),
                            BackgroundColor(Color::srgba(0.12, 0.12, 0.13, 0.35)),
                            Visibility::Hidden,
                            ZIndex(1),
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
                            ZIndex(2),
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

fn handle_workspace_file_buttons(
    interaction_query: Query<
        (&Interaction, &WorkspaceFileButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    for (interaction, file_button) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.open_workspace_file(file_button.index);
    }
}

fn handle_workspace_folder_buttons(
    interaction_query: Query<
        (&Interaction, &WorkspaceFolderToggleButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<EditorState>,
) {
    for (interaction, folder_button) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.toggle_workspace_folder(&folder_button.folder_key);
    }
}

fn sync_workspace_sidebar(
    mut commands: Commands,
    fonts: Res<EditorFonts>,
    workspace_icons: Res<WorkspaceIcons>,
    mut state: ResMut<EditorState>,
    mut root_label_query: Query<&mut Text, With<WorkspaceRootLabel>>,
    list_query: Query<(Entity, Option<&Children>), With<WorkspaceFileList>>,
) {
    if !state.workspace_ui_dirty {
        return;
    }

    if let Ok(mut root_label) = root_label_query.single_mut() {
        **root_label = state.workspace_root.as_ref().map_or_else(
            || "No workspace opened.".to_string(),
            |root| format!("Root: {}", root.display()),
        );
    }

    let Ok((file_list_entity, children)) = list_query.single() else {
        state.workspace_ui_dirty = false;
        return;
    };

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let rows = workspace_sidebar_rows(&state);

    commands.entity(file_list_entity).with_children(|parent| {
        if rows.is_empty() {
            parent.spawn((
                Text::new("No .fountain/.md/.txt files found."),
                TextFont {
                    font: fonts.regular.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MUTED),
            ));
            return;
        }

        for row in rows {
            match row {
                WorkspaceSidebarRow::Folder {
                    folder_key,
                    folder_name,
                    depth,
                    expanded,
                } => {
                    let icon_handle = if expanded {
                        workspace_icons.folder_open.clone()
                    } else {
                        workspace_icons.folder_closed.clone()
                    };
                    let left_indent = 6.0 + depth as f32 * 14.0;
                    let fallback_marker = if expanded { "▾" } else { "▸" };

                    parent.spawn((
                        Button,
                        WorkspaceFolderToggleButton { folder_key },
                        Node {
                            width: percent(100.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(6.0),
                            padding: UiRect::axes(px(left_indent), px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        children![
                            (
                                Text::new(fallback_marker),
                                TextFont {
                                    font: fonts.regular.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(COLOR_TEXT_MUTED),
                            ),
                            (
                                Node {
                                    width: px(18.0),
                                    height: px(18.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                children![(
                                    ImageNode::new(icon_handle),
                                    Node {
                                        width: px(14.0),
                                        height: px(14.0),
                                        ..default()
                                    },
                                )],
                            ),
                            (
                                Text::new(folder_name),
                                TextFont {
                                    font: fonts.regular.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(COLOR_TEXT_MAIN),
                            )
                        ],
                    ));
                }
                WorkspaceSidebarRow::File {
                    file_index,
                    file_name,
                    depth,
                } => {
                    let left_indent = 28.0 + depth as f32 * 14.0;
                    let text_color = if state.workspace_selected == Some(file_index) {
                        COLOR_WORKSPACE_FILE_SELECTED
                    } else {
                        COLOR_WORKSPACE_FILE
                    };

                    parent.spawn((
                        Button,
                        WorkspaceFileButton { index: file_index },
                        Node {
                            width: percent(100.0),
                            padding: UiRect::new(
                                px(left_indent),
                                px(8.0),
                                px(4.0),
                                px(4.0),
                            ),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        children![(
                            Text::new(file_name),
                            TextFont {
                                font: fonts.regular.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(text_color),
                        )],
                    ));
                }
            }
        }
    });

    state.workspace_ui_dirty = false;
}

fn style_workspace_file_entry_text(
    state: Res<EditorState>,
    mut file_button_query: Query<
        (&Interaction, &WorkspaceFileButton, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_color_query: Query<&mut TextColor>,
) {
    for (interaction, workspace_file_button, children) in file_button_query.iter_mut() {
        let color = match *interaction {
            Interaction::Hovered | Interaction::Pressed => COLOR_WORKSPACE_FILE_HOVER,
            Interaction::None => {
                if state.workspace_selected == Some(workspace_file_button.index) {
                    COLOR_WORKSPACE_FILE_SELECTED
                } else {
                    COLOR_WORKSPACE_FILE
                }
            }
        };

        for child in children.iter() {
            if let Ok(mut text_color) = text_color_query.get_mut(child) {
                text_color.0 = color;
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
            Or<(With<ToolbarAction>, With<SettingsAction>)>,
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
                    "Dialogue double-space newline in processed pane: {}",
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
                    "Non-dialogue double-space newline in processed pane: {}",
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
            SettingsAction::OpenKeybinds => {
                next_screen_state.set(UiScreenState::Keybinds);
                state.status_message = "Opened keybinds.".to_string();
            }
            SettingsAction::BackToSettings => {
                next_screen_state.set(UiScreenState::Settings);
                state.status_message = "Opened settings.".to_string();
            }
            SettingsAction::BackToEditor => {
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

fn sync_settings_ui(
    state: Res<EditorState>,
    screen_state: Res<State<UiScreenState>>,
    mut editor_root_query: Query<
        &mut Node,
        (
            With<EditorScreenRoot>,
            Without<SettingsScreenRoot>,
            Without<KeybindsScreenRoot>,
        ),
    >,
    mut settings_root_query: Query<
        &mut Node,
        (
            With<SettingsScreenRoot>,
            Without<EditorScreenRoot>,
            Without<KeybindsScreenRoot>,
        ),
    >,
    mut keybinds_root_query: Query<
        &mut Node,
        (
            With<KeybindsScreenRoot>,
            Without<EditorScreenRoot>,
            Without<SettingsScreenRoot>,
        ),
    >,
    mut toggle_label_query: Query<(&SettingToggleLabel, &mut Text), Without<SettingMarginLabel>>,
    mut margin_label_query: Query<(&SettingMarginLabel, &mut Text), Without<SettingToggleLabel>>,
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

    for (label, mut text) in toggle_label_query.iter_mut() {
        **text = match label.action {
            SettingsAction::DialogueDoubleSpaceNewline => format!(
                "Double space as newline in dialogue (processed pane): {}",
                if state.dialogue_double_space_newline {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            SettingsAction::NonDialogueDoubleSpaceNewline => format!(
                "Double space as newline in non-dialogue (processed pane): {}",
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
}
