fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));

    let fonts = EditorFonts {
        regular: asset_server.load(FONT_PATH),
        bold: asset_server.load(FONT_BOLD_PATH),
        italic: asset_server.load(FONT_ITALIC_PATH),
        bold_italic: asset_server.load(FONT_BOLD_ITALIC_PATH),
    };
    let font = fonts.regular.clone();
    commands.insert_resource(fonts);

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
                        children![
                            (
                                Text::new(
                                    "Cmd/Ctrl+O load | Cmd/Ctrl+S save | Cmd/Ctrl +/- or Ctrl+scroll zoom | arrows/home/end/page move cursor | mouse wheel scroll | click to place cursor",
                                ),
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
                                    toolbar_button(font.clone(), "Load", ToolbarAction::Load),
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
                            padding: UiRect::axes(px(12.0), px(0.0)),
                            ..default()
                        },
                        Text::new("Load opens a native file picker. Save As opens a native save dialog."),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MUTED),
                    ),
                    (
                        Node {
                            width: percent(100.0),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Row,
                            column_gap: px(10.0),
                            padding: UiRect::axes(px(10.0), px(8.0)),
                            ..default()
                        },
                        children![
                            panel_bundle(font.clone(), PanelKind::Plain, "Plain"),
                            panel_bundle(font.clone(), PanelKind::Processed, "Processed"),
                        ],
                    ),
                    (
                        Node {
                            width: percent(100.0),
                            padding: UiRect::axes(px(12.0), px(8.0)),
                            ..default()
                        },
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_MAIN),
                        StatusText,
                    )
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
                    settings_action_button(font, "Back to editor", SettingsAction::BackToEditor),
                ],
            ));
        });
}

fn setup_processed_papers(
    mut commands: Commands,
    canvas_query: Query<(Entity, &PanelCanvas)>,
    paper_query: Query<(Entity, &PanelPaper)>,
    text_query: Query<(Entity, &PanelText)>,
    fonts: Res<EditorFonts>,
) {
    let regular_font = fonts.regular.clone();
    let span_capacity = processed_page_step_lines().max(1);

    for (entity, panel_canvas) in canvas_query.iter() {
        if panel_canvas.kind != PanelKind::Processed {
            continue;
        }

        let regular_font = regular_font.clone();
        commands.entity(entity).with_children(|parent| {
            for slot in 1..PROCESSED_PAPER_CAPACITY {
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

fn panel_bundle(font: Handle<Font>, kind: PanelKind, title: &str) -> impl Bundle {
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
                    padding: UiRect::axes(px(14.0), px(6.0)),
                    ..default()
                },
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_MAIN),
            ),
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
    mut state: ResMut<EditorState>,
    mut dialogs: ResMut<DialogState>,
    mut next_screen_state: ResMut<NextState<UiScreenState>>,
) {
    let parent_handle = primary_window_query.iter().next();

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
            ToolbarAction::Load => open_load_dialog(&mut state, &mut dialogs, parent_handle),
            ToolbarAction::SaveAs => open_save_dialog(&mut state, &mut dialogs, parent_handle),
            ToolbarAction::ZoomOut => {
                let next_zoom = state.zoom - ZOOM_STEP;
                state.set_zoom(next_zoom);
                state.status_message = format!("Zoom: {}%", state.zoom_percent());
            }
            ToolbarAction::ZoomIn => {
                let next_zoom = state.zoom + ZOOM_STEP;
                state.set_zoom(next_zoom);
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
            SettingsAction::BackToEditor => {
                next_screen_state.set(UiScreenState::Editor);
                state.status_message = "Closed settings.".to_string();
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

fn sync_settings_ui(
    state: Res<EditorState>,
    screen_state: Res<State<UiScreenState>>,
    mut editor_root_query: Query<&mut Node, (With<EditorScreenRoot>, Without<SettingsScreenRoot>)>,
    mut settings_root_query: Query<
        &mut Node,
        (With<SettingsScreenRoot>, Without<EditorScreenRoot>),
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
