use super::*;
use bevy::input::keyboard::KeyboardInput;

#[test]
fn open_theme_from_settings_activates_overlay_and_editor_screen() {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<UiScreenState>();
    app.init_resource::<EditorState>();
    app.add_systems(Update, handle_settings_buttons);

    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.display_mode = DisplayMode::Plain;
        state.right_buttons_visible = false;
        state.theme_overlay_open = false;
    }

    app.world_mut().spawn((
        Button,
        SettingsAction::OpenTheme,
        Interaction::Pressed,
    ));

    app.update();

    let state = app.world().resource::<EditorState>();
    assert!(state.theme_overlay_open);
    assert_eq!(state.theme_category, ThemeCategory::Theme);
    assert_eq!(state.theme_color_target, ThemeColorTarget::AppBackground);
    assert_eq!(state.display_mode, DisplayMode::Split);
    assert!(state.right_buttons_visible);

    let screen_state = app.world().resource::<State<UiScreenState>>();
    assert_eq!(*screen_state.get(), UiScreenState::Editor);
}

#[test]
fn open_link_colors_activates_overlay_and_links_category() {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<UiScreenState>();
    app.init_resource::<EditorState>();
    app.add_systems(Update, handle_settings_buttons);

    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.display_mode = DisplayMode::Plain;
        state.right_buttons_visible = false;
        state.theme_overlay_open = false;
    }

    app.world_mut().spawn((
        Button,
        SettingsAction::OpenLinkColors,
        Interaction::Pressed,
    ));

    app.update();

    let state = app.world().resource::<EditorState>();
    assert!(state.theme_overlay_open);
    assert_eq!(state.theme_category, ThemeCategory::Links);
    assert_eq!(state.theme_color_target, ThemeColorTarget::LinkFallback);
    assert_eq!(state.display_mode, DisplayMode::Split);
    assert!(state.right_buttons_visible);

    let screen_state = app.world().resource::<State<UiScreenState>>();
    assert_eq!(*screen_state.get(), UiScreenState::Editor);
}

#[test]
fn theme_overlay_ok_button_closes_overlay() {
    let mut app = App::new();
    app.init_resource::<EditorState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_message::<KeyboardInput>();
    app.add_systems(Update, handle_theme_overlay_buttons);

    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.theme_overlay_open = true;
        state.theme_color_picker_open = true;
    }

    app.world_mut().spawn((
        Button,
        ThemeOverlayOkButton,
        Interaction::Pressed,
    ));

    app.update();

    let state = app.world().resource::<EditorState>();
    assert!(!state.theme_overlay_open);
    assert!(!state.theme_color_picker_open);
    cleanup_test_settings();
}

#[test]
fn theme_overlay_tab_switch_updates_category_and_target() {
    let mut app = App::new();
    app.init_resource::<EditorState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_message::<KeyboardInput>();
    app.add_systems(Update, handle_theme_overlay_buttons);

    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.theme_overlay_open = true;
        state.theme_category = ThemeCategory::Theme;
        state.theme_color_target = ThemeColorTarget::AppBackground;
    }

    app.world_mut().spawn((
        Button,
        ThemeOverlayTabButton(ThemeCategory::Links),
        Interaction::Pressed,
    ));

    app.update();

    let state = app.world().resource::<EditorState>();
    assert_eq!(state.theme_category, ThemeCategory::Links);
    assert_eq!(state.theme_color_target, ThemeColorTarget::LinkFallback);
}

#[test]
fn theme_overlay_sync_controls_container_and_sections_visibility() {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<UiScreenState>();
    app.init_resource::<EditorState>();
    app.add_systems(Update, sync_theme_picker_ui);

    let container = app.world_mut().spawn((
        ThemeOverlayContainer,
        Node::default(),
    )).id();

    let theme_section = app.world_mut().spawn((
        ThemeCategorySection(ThemeCategory::Theme),
        Node::default(),
    )).id();

    let links_section = app.world_mut().spawn((
        ThemeCategorySection(ThemeCategory::Links),
        Node::default(),
    )).id();

    let picker_panel = app.world_mut().spawn((
        ThemeColorPickerPanel,
        Node::default(),
    )).id();

    // With theme_overlay_open = false
    app.update();

    assert_eq!(app.world().get::<Node>(container).unwrap().display, Display::None);

    // Open overlay in editor
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.theme_overlay_open = true;
        state.theme_category = ThemeCategory::Theme;
        state.theme_color_picker_open = true;
    }

    app.update();

    assert_eq!(app.world().get::<Node>(container).unwrap().display, Display::Flex);
    assert_eq!(app.world().get::<Node>(theme_section).unwrap().display, Display::Flex);
    assert_eq!(app.world().get::<Node>(links_section).unwrap().display, Display::None);
    assert_eq!(app.world().get::<Node>(picker_panel).unwrap().display, Display::Flex);
}

#[test]
fn test_classic_theme_preset_matches_pre_commit_defaults() {
    let classic = ThemeSettings::classic();
    assert_eq!(classic.name, "Classic");
    assert_eq!(classic.app_background, Vec4::new(0.885, 0.901, 0.898, 1.0));
    assert_eq!(classic.top_menu_background, Vec4::new(0.891, 0.907, 0.904, 1.0));
    assert_eq!(classic.explorer_background, Vec4::new(0.860, 0.870, 0.890, 1.0));
    assert_eq!(classic.processed_background, Vec4::new(0.536, 0.545, 0.570, 1.0));
    assert_eq!(classic.selection_background, Vec4::new(0.517, 0.680, 1.0, 0.558));
    assert_eq!(classic.link_fallback, Vec4::new(0.100, 0.380, 0.720, 1.0));
    assert_eq!(classic.link_prop, Vec4::new(0.680, 0.400, 0.100, 1.0));
    assert_eq!(classic.link_place, Vec4::new(0.120, 0.500, 0.340, 1.0));
    assert_eq!(classic.link_character, Vec4::new(0.569, 0.058, 0.822, 1.0));
    assert_eq!(classic.link_faction, Vec4::new(0.680, 0.176, 0.209, 1.0));
    assert_eq!(classic.link_concept, Vec4::new(0.560, 0.280, 0.140, 1.0));
    assert_eq!(classic.link_hover_hsv_value_adjustment, 0.380);
    assert_eq!(classic.paper_background, Vec4::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(classic.text_main, Vec4::new(0.18, 0.19, 0.20, 1.0));
    assert_eq!(classic.text_muted, Vec4::new(0.34, 0.36, 0.39, 1.0));
    assert_eq!(classic.text_scene_heading, Vec4::new(0.10, 0.10, 0.12, 1.0));
    assert_eq!(classic.text_action, Vec4::new(0.12, 0.13, 0.15, 1.0));
    assert_eq!(classic.text_character, Vec4::new(0.20, 0.16, 0.12, 1.0));
    assert_eq!(classic.text_dialogue, Vec4::new(0.11, 0.12, 0.13, 1.0));
    assert_eq!(classic.text_parenthetical, Vec4::new(0.24, 0.28, 0.32, 1.0));
    assert_eq!(classic.text_transition, Vec4::new(0.15, 0.23, 0.31, 1.0));
    assert_eq!(classic.text_markdown_heading, Vec4::new(0.18, 0.24, 0.40, 1.0));
    assert_eq!(classic.text_markdown_quote, Vec4::new(0.22, 0.29, 0.26, 1.0));
    assert_eq!(classic.text_markdown_code, Vec4::new(0.29, 0.17, 0.18, 1.0));
    assert_eq!(classic.text_markdown_rule, Vec4::new(0.35, 0.35, 0.38, 1.0));
}

#[test]
fn test_dark_theme_preset_values() {
    let dark = ThemeSettings::dark();
    assert_eq!(dark.name, "Dark");
    assert_eq!(dark.app_background, Vec4::new(0.14, 0.15, 0.17, 1.0));
    assert_eq!(dark.paper_background, Vec4::new(0.12, 0.12, 0.14, 1.0));
    assert_eq!(dark.text_main, Vec4::new(0.88, 0.89, 0.91, 1.0));
    assert_eq!(dark.text_character, Vec4::new(0.95, 0.85, 0.65, 1.0));
}

#[test]
fn test_apply_theme_to_state_updates_all_colors_and_name() {
    let mut state = EditorState::from_world(&mut World::new());
    let classic = ThemeSettings::classic();
    apply_theme_to_state(&mut state, &classic);

    assert_eq!(state.current_theme_name, "Classic");
    assert_eq!(state.app_bg_rgba, Vec4::new(0.885, 0.901, 0.898, 1.0));
    assert_eq!(state.paper_bg_rgba, Vec4::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(state.paper_bg_color, Color::srgba(1.0, 1.0, 1.0, 1.0));
    assert_eq!(state.text_character_rgba, Vec4::new(0.20, 0.16, 0.12, 1.0));
    assert_eq!(state.text_character_color, Color::srgba(0.20, 0.16, 0.12, 1.0));

    let dark = ThemeSettings::dark();
    apply_theme_to_state(&mut state, &dark);

    assert_eq!(state.current_theme_name, "Dark");
    assert_eq!(state.app_bg_rgba, Vec4::new(0.14, 0.15, 0.17, 1.0));
    assert_eq!(state.paper_bg_rgba, Vec4::new(0.12, 0.12, 0.14, 1.0));
    assert_eq!(state.paper_bg_color, Color::srgba(0.12, 0.12, 0.14, 1.0));
    assert_eq!(state.text_character_rgba, Vec4::new(0.95, 0.85, 0.65, 1.0));
    assert_eq!(state.text_character_color, Color::srgba(0.95, 0.85, 0.65, 1.0));
}

#[test]
fn test_ron_serialization_and_deserialization_roundtrip() {
    let mut theme = ThemeSettings::classic();
    theme.name = "MyCustomTheme".to_string();
    theme.paper_background = Vec4::new(0.95, 0.94, 0.92, 1.0);
    theme.text_character = Vec4::new(0.2, 0.3, 0.4, 1.0);

    let ron_str = ron_string_from_theme(&theme);
    let parsed = theme_settings_from_ron(&ron_str, &ThemeSettings::default());

    assert_eq!(parsed.name, "MyCustomTheme");
    assert_eq!(parsed.paper_background, Vec4::new(0.95, 0.94, 0.92, 1.0));
    assert_eq!(parsed.text_character, Vec4::new(0.2, 0.3, 0.4, 1.0));
    assert_eq!(parsed.app_background, theme.app_background);
}

#[test]
fn test_ron_backward_compatibility_defaults() {
    let old_ron = r#"ThemeSettings(
    app_background: (0.8, 0.8, 0.8, 1.0),
    top_menu: (0.85, 0.85, 0.85, 1.0),
    explorer: (0.8, 0.8, 0.8, 1.0),
    processed: (0.5, 0.5, 0.5, 1.0),
    selection: (0.5, 0.6, 1.0, 0.5),
    link_fallback: (0.1, 0.3, 0.7, 1.0),
    link_prop: (0.6, 0.4, 0.1, 1.0),
    link_place: (0.1, 0.5, 0.3, 1.0),
    link_character: (0.5, 0.05, 0.8, 1.0),
    link_faction: (0.6, 0.1, 0.2, 1.0),
    link_concept: (0.5, 0.2, 0.1, 1.0),
    link_hover_hsv_value_adjustment: 0.38,
)"#;
    let parsed = theme_settings_from_ron(old_ron, &ThemeSettings::default());
    assert_eq!(parsed.name, "Default");
    assert_eq!(parsed.app_background, Vec4::new(0.8, 0.8, 0.8, 1.0));
    assert_eq!(parsed.paper_background, Vec4::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(parsed.text_main, Vec4::new(0.18, 0.19, 0.20, 1.0));
}

#[test]
fn test_save_and_load_named_theme() {
    let test_name = "UnitTestNamedTheme_Unique";
    let mut theme = ThemeSettings::classic();
    theme.name = test_name.to_string();
    theme.paper_background = Vec4::new(0.11, 0.22, 0.33, 1.0);

    let save_res = save_named_theme(test_name, &theme);
    assert!(save_res.is_ok());

    let available = list_available_themes();
    assert!(available.contains(&test_name.to_string()));

    let loaded = load_named_theme(test_name);
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.name, test_name);
    assert_eq!(loaded.paper_background, Vec4::new(0.11, 0.22, 0.33, 1.0));

    // Clean up test file
    let path = std::path::Path::new(THEMES_DIR).join(format!("{test_name}.ron"));
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    cleanup_test_settings();
}

fn cleanup_test_settings() {
    if std::path::Path::new("../Cargo.toml").exists() && std::path::Path::new("settings").exists() {
        let _ = std::fs::remove_dir_all("settings");
    }
}

#[test]
fn test_theme_overlay_cycle_prev_and_next_buttons() {
    let mut app = App::new();
    app.init_resource::<EditorState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_message::<KeyboardInput>();
    app.add_systems(Update, handle_theme_overlay_buttons);

    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.theme_overlay_open = true;
        state.available_themes = vec!["Default".to_string(), "Classic".to_string(), "Dark".to_string()];
        state.current_theme_name = "Default".to_string();
    }

    // Press Next
    let next_btn = app.world_mut().spawn((
        Button,
        ThemeSwitchNextButton,
        Interaction::Pressed,
    )).id();

    app.update();

    {
        let state = app.world().resource::<EditorState>();
        assert_eq!(state.current_theme_name, "Classic");
        assert_eq!(state.app_bg_rgba, ThemeSettings::classic().app_background);
    }

    // Reset interaction on next_btn, simulate Next again
    app.world_mut().entity_mut(next_btn).insert(Interaction::None);
    app.update();

    app.world_mut().entity_mut(next_btn).insert(Interaction::Pressed);
    app.update();

    {
        let state = app.world().resource::<EditorState>();
        assert_eq!(state.current_theme_name, "Dark");
        assert_eq!(state.app_bg_rgba, ThemeSettings::dark().app_background);
    }

    // Press Prev
    app.world_mut().entity_mut(next_btn).insert(Interaction::None);
    let _prev_btn = app.world_mut().spawn((
        Button,
        ThemeSwitchPrevButton,
        Interaction::Pressed,
    )).id();
    app.update();

    {
        let state = app.world().resource::<EditorState>();
        assert_eq!(state.current_theme_name, "Classic");
    }
    cleanup_test_settings();
}

#[test]
fn test_theme_overlay_preset_button_click() {
    let mut app = App::new();
    app.init_resource::<EditorState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_message::<KeyboardInput>();
    app.add_systems(Update, handle_theme_overlay_buttons);

    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.theme_overlay_open = true;
        state.current_theme_name = "Default".to_string();
    }

    app.world_mut().spawn((
        Button,
        ThemePresetButton("Classic".to_string()),
        Interaction::Pressed,
    ));

    app.update();

    let state = app.world().resource::<EditorState>();
    assert_eq!(state.current_theme_name, "Classic");
    assert_eq!(state.app_bg_rgba, ThemeSettings::classic().app_background);
    cleanup_test_settings();
}

#[test]
fn test_fountain_and_markdown_text_styles_reflect_state_colors() {
    let mut state = EditorState::from_world(&mut World::new());
    state.text_character_color = Color::srgb(0.9, 0.1, 0.2);
    state.text_dialogue_color = Color::srgb(0.8, 0.2, 0.3);
    state.text_scene_heading_color = Color::srgb(0.7, 0.3, 0.4);
    state.text_markdown_heading_color = Color::srgb(0.6, 0.4, 0.5);

    let char_style = fountain_line_style_for_state(&state, &LineKind::Character).unwrap();
    assert_eq!(char_style.color, Color::srgb(0.9, 0.1, 0.2));

    let dial_style = fountain_line_style_for_state(&state, &LineKind::Dialogue).unwrap();
    assert_eq!(dial_style.color, Color::srgb(0.8, 0.2, 0.3));

    let scene_style = fountain_line_style_for_state(&state, &LineKind::SceneHeading).unwrap();
    assert_eq!(scene_style.color, Color::srgb(0.7, 0.3, 0.4));

    let md_style = markdown_line_style_for_state(&state, &LineKind::MarkdownHeading, Some(1)).unwrap();
    assert_eq!(md_style.color, Color::srgb(0.6, 0.4, 0.5));
}

#[test]
fn test_sync_glass_surfaces_updates_panel_paper_color() {
    let mut app = App::new();
    app.init_resource::<EditorState>();
    app.init_resource::<NativeGlassState>();
    app.add_systems(Update, sync_glass_surfaces);

    let paper_entity = app.world_mut().spawn((
        PanelPaper { kind: PanelKind::Processed, slot: 0 },
        BackgroundColor(Color::WHITE),
    )).id();

    let custom_paper_color = Color::srgba(0.85, 0.88, 0.92, 1.0);
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.paper_bg_color = custom_paper_color;
    }

    app.update();

    let bg = app.world().get::<BackgroundColor>(paper_entity).unwrap();
    assert_eq!(bg.0, custom_paper_color);
}

