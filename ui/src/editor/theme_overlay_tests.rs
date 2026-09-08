use super::*;

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
}

#[test]
fn theme_overlay_tab_switch_updates_category_and_target() {
    let mut app = App::new();
    app.init_resource::<EditorState>();
    app.init_resource::<ButtonInput<KeyCode>>();
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
