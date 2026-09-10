use super::*;

fn setup() -> App {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<UiScreenState>()
        .init_resource::<EditorState>()
        .init_resource::<ResolvedPanelWidths>()
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(theme_overlay_container_bundle(default(), default()));
            commands.spawn(toolbar_button(default(), "Save", ToolbarAction::Save));
            commands.spawn((
                PanelBody {
                    kind: PanelKind::Processed,
                },
                ComputedNode {
                    size: Vec2::new(900.0, 900.0),
                    inverse_scale_factor: 1.0,
                    ..default()
                },
            ));
            commands.spawn(Node::default()).with_children(|parent| {
                spawn_markdown_metadata_controls(parent, default());
            });
            commands.spawn((
                Button,
                WorkspaceFileButton { index: 0 },
                BackgroundColor::default(),
                children![(Text::new("Notes.md"), ThemedText::Main)],
            ));
        })
        .add_systems(
            Update,
            (sync_theme_picker_ui, sync_markdown_metadata_controls_ui),
        )
        .add_systems(PostUpdate, (sync_theme_widgets, style_workspace_rows));
    let mut state = app.world_mut().resource_mut::<EditorState>();
    state.document = Document::from_text("---\nid: notes\ntype: document\n---\nNotes");
    state.document_format = DocumentFormat::Markdown;
    state.display_mode = DisplayMode::Processed;
    state.theme_overlay_open = true;
    state.reparse();
    app
}

#[test]
fn theme_popup_is_opaque_and_stacks_above_document_content() {
    let mut app = setup();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::state::app::StatesPlugin>()
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::render::RenderPlugin>()
            .disable::<bevy::core_pipeline::CorePipelinePlugin>()
            .disable::<bevy::sprite_render::SpriteRenderPlugin>()
            .disable::<bevy::ui_render::UiRenderPlugin>()
            .disable::<bevy::log::LogPlugin>(),
    );
    app.world_mut()
        .resource_mut::<EditorState>()
        .ui_colors
        .set(UiColor::PanelBackground, Vec4::new(0.16, 0.17, 0.19, 0.25));
    let document = app.world_mut().spawn((Node::default(), ZIndex(1000))).id();
    let underlays = [1, 8, 12].map(|layer| {
        app.world_mut()
            .spawn((Node::default(), GlobalZIndex(layer), ChildOf(document)))
            .id()
    });
    // Canvas nodes use local layers, including high indices in large boards.
    let canvas_node = app
        .world_mut()
        .spawn((Node::default(), ZIndex(10_000), ChildOf(document)))
        .id();
    app.update();
    let world = app.world_mut();
    let (popup_index, background) = world.query_filtered::<
        (&bevy::ui::ComputedStackIndex, &BackgroundColor), With<ThemeOverlayContainer>
    >().single(world).unwrap();
    assert_eq!(background.0, Color::srgb(0.16, 0.17, 0.19));
    let popup_index = popup_index.0;
    for underlay in underlays.into_iter().chain([canvas_node]) {
        assert!(
            world
                .get::<bevy::ui::ComputedStackIndex>(underlay)
                .unwrap()
                .0
                < popup_index
        );
    }
    let button_index = world
        .query_filtered::<&bevy::ui::ComputedStackIndex, With<ThemeOverlayOkButton>>()
        .single(world)
        .unwrap()
        .0;
    assert!(button_index > popup_index);
}

#[test]
fn existing_widgets_and_explorer_follow_theme_changes() {
    let mut app = setup();
    for theme in [
        ThemeSettings::dark(),
        ThemeSettings::default(),
        ThemeSettings::dark(),
    ] {
        apply_theme_to_state(&mut app.world_mut().resource_mut::<EditorState>(), &theme);
        app.update();
        let world = app.world_mut();
        for (role, color) in world.query::<(&ThemedText, &TextColor)>().iter(world) {
            assert_eq!(
                color.0,
                match role {
                    ThemedText::Main => theme.text_main_color(),
                    ThemedText::Muted => theme.text_muted_color(),
                }
            );
        }
        let mut surfaces = world.query::<(&ThemedBackground, &BackgroundColor)>();
        assert!(surfaces.iter(world).count() >= 4);
        for (role, color) in surfaces.iter(world) {
            assert_eq!(color.0, theme.ui_colors.color(role.0));
        }
        let mut fields =
            world.query_filtered::<&BackgroundColor, With<MarkdownMetadataFieldButton>>();
        assert_eq!(fields.iter(world).count(), 6);
        for color in fields.iter(world) {
            assert_eq!(color.0, theme.ui_colors.color(UiColor::InputBackground));
        }
        for color in world
            .query_filtered::<&BorderColor, With<ThemedBorder>>()
            .iter(world)
        {
            assert_eq!(
                *color,
                BorderColor::all(theme.ui_colors.color(UiColor::Border))
            );
        }
    }

    app.world_mut()
        .resource_mut::<EditorState>()
        .workspace_active_file = Some(0);
    app.update();
    let expected = app
        .world()
        .resource::<EditorState>()
        .ui_colors
        .color(UiColor::ActiveBackground);
    let world = app.world_mut();
    assert_eq!(
        world
            .query_filtered::<&BackgroundColor, With<WorkspaceFileButton>>()
            .single(world)
            .unwrap()
            .0,
        expected
    );
}

#[test]
fn ok_and_save_use_the_same_live_palette_as_toolbar_buttons() {
    let mut app = setup();
    apply_theme_to_state(
        &mut app.world_mut().resource_mut::<EditorState>(),
        &ThemeSettings::dark(),
    );
    app.update();
    for interaction in [
        Interaction::None,
        Interaction::Hovered,
        Interaction::Pressed,
    ] {
        let world = app.world_mut();
        let buttons: Vec<Entity> = world
            .query_filtered::<Entity, Or<(
                With<ToolbarAction>,
                With<ThemeOverlayOkButton>,
                With<ThemeSaveButton>,
                With<ThemeSaveNewButton>,
            )>>()
            .iter(world)
            .collect();
        assert_eq!(buttons.len(), 4);
        for &button in &buttons {
            *world.get_mut::<Interaction>(button).unwrap() = interaction;
        }
        app.update();
        let expected = themed_button_color(app.world().resource::<EditorState>(), interaction);
        for &button in &buttons {
            assert_eq!(
                app.world().get::<BackgroundColor>(button).unwrap().0,
                expected
            );
        }

        // Changing a color must update buttons even without a new pointer event.
        let role = match interaction {
            Interaction::None => UiColor::ButtonBackground,
            Interaction::Hovered => UiColor::ButtonHover,
            Interaction::Pressed => UiColor::ButtonPressed,
        };
        {
            let mut state = app.world_mut().resource_mut::<EditorState>();
            state.theme_color_target = ThemeColorTarget::Ui(role);
            set_active_theme_rgba(&mut state, Vec4::new(0.3, 0.4, 0.5, 1.0));
        }
        app.update();
        for button in buttons {
            assert_eq!(
                app.world().get::<BackgroundColor>(button).unwrap().0,
                Color::srgb(0.3, 0.4, 0.5)
            );
        }
    }
}

#[test]
fn ui_colors_round_trip_and_legacy_dark_themes_get_dark_controls() {
    let mut state = EditorState::from_world(&mut World::new());
    apply_theme_to_state(&mut state, &ThemeSettings::dark());
    for (index, role) in UiColor::ALL.into_iter().enumerate() {
        state.theme_color_target = ThemeColorTarget::Ui(role);
        set_active_theme_rgba(
            &mut state,
            Vec4::new(0.1 + index as f32 * 0.05, 0.2, 0.3, 0.8),
        );
    }
    let saved = ron_string_from_theme(&theme_settings_from_state(&state));
    let parsed = theme_settings_from_ron(&saved, &ThemeSettings::default());
    for role in UiColor::ALL {
        assert!(
            (parsed.ui_colors.rgba(role) - state.ui_colors.rgba(role))
                .abs()
                .max_element()
                < 0.001
        );
    }
    apply_theme_to_state(&mut state, &parsed);
    assert_eq!(
        theme_settings_from_state(&state).ui_colors,
        parsed.ui_colors
    );

    let legacy = theme_settings_from_ron(
        "(\nname: \"Dark\",\napp_background: (0.14, 0.15, 0.17, 1.0),\n)",
        &ThemeSettings::default(),
    );
    assert_eq!(legacy.ui_colors, UiPalette::dark());
    let light = theme_settings_from_ron("()", &ThemeSettings::default());
    assert_eq!(light.ui_colors, UiPalette::default());
}

#[test]
fn paper_opacity_is_fixed_in_saved_themes_and_the_picker() {
    let mut app = setup();
    let mut theme = ThemeSettings::dark();
    theme.paper_background.w = 0.2;
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        apply_theme_to_state(&mut state, &theme);
        assert_eq!(state.paper_bg_color.alpha(), 1.0);
        state.theme_color_target = ThemeColorTarget::PaperBackground;
        set_active_theme_rgba(&mut state, Vec4::new(0.1, 0.2, 0.3, 0.0));
        assert_eq!(state.paper_bg_rgba.w, 1.0);
        let saved = ron_string_from_theme(&theme_settings_from_state(&state));
        assert_eq!(parse_ron_vec4(&saved, "paper_background").unwrap().w, 1.0);
    }
    for target in [
        ThemeColorTarget::PaperBackground,
        ThemeColorTarget::ProcessedBackground,
    ] {
        app.world_mut()
            .resource_mut::<EditorState>()
            .theme_color_target = target;
        app.update();
        let world = app.world_mut();
        for (row, node) in world.query::<(&ThemeColorSliderRow, &Node)>().iter(world) {
            if matches!(row.0, ThemeSliderChannel::Alpha) {
                assert_eq!(
                    node.display,
                    if target == ThemeColorTarget::PaperBackground {
                        Display::None
                    } else {
                        Display::Flex
                    }
                );
            }
        }
    }
}
