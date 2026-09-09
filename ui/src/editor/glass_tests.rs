use super::*;

struct Surfaces {
    root: Entity,
    menu: Entity,
    explorer: Entity,
    plain: Entity,
    processed: Entity,
    paper: Entity,
    settings: Entity,
    status: Entity,
}

fn setup() -> (App, Surfaces) {
    let mut app = App::new();
    app.init_resource::<EditorState>()
        .init_resource::<NativeGlassState>()
        .add_systems(Update, sync_glass_surfaces);
    let world = app.world_mut();
    {
        let mut state = world.resource_mut::<EditorState>();
        state.processed_glass = false;
        state.explorer_glass = false;
        state.settings_glass = false;
        state.document_format = DocumentFormat::Markdown;
        state.app_bg_color = Color::srgb(0.8, 0.8, 0.8);
        state.top_menu_bg_color = Color::srgb(0.9, 0.9, 0.9);
        state.explorer_bg_color = Color::srgb(0.7, 0.7, 0.7);
        state.processed_bg_color = Color::srgb(0.6, 0.6, 0.6);
    }
    let surfaces = Surfaces {
        root: world
            .spawn((WindowSurfaceRoot, BackgroundColor::default()))
            .id(),
        menu: world
            .spawn((TopMenuSection, BackgroundColor::default()))
            .id(),
        explorer: world
            .spawn((WorkspaceSidebarPane, BackgroundColor::default()))
            .id(),
        plain: world
            .spawn((
                PanelBody {
                    kind: PanelKind::Plain,
                },
                BackgroundColor::default(),
            ))
            .id(),
        processed: world
            .spawn((
                PanelBody {
                    kind: PanelKind::Processed,
                },
                BackgroundColor::default(),
            ))
            .id(),
        paper: world
            .spawn((
                PanelPaper {
                    kind: PanelKind::Processed,
                    slot: 0,
                },
                BackgroundColor::default(),
            ))
            .id(),
        settings: world
            .spawn((SettingsScreenRoot, BackgroundColor::default()))
            .id(),
        status: world
            .spawn((StatusLineRoot, BackgroundColor::default()))
            .id(),
    };
    (app, surfaces)
}

fn color(app: &App, entity: Entity) -> Color {
    app.world().get::<BackgroundColor>(entity).unwrap().0
}

#[test]
fn glass_toggles_open_the_root_and_only_the_selected_surfaces() {
    let (mut app, surfaces) = setup();
    app.world_mut().resource_mut::<NativeGlassState>().active = true;
    app.update();
    assert_eq!(color(&app, surfaces.root).alpha(), 1.0);

    app.world_mut().resource_mut::<EditorState>().explorer_glass = true;
    app.update();
    assert_eq!(color(&app, surfaces.root), Color::NONE);
    assert!(color(&app, surfaces.explorer).alpha() < 1.0);
    assert_eq!(color(&app, surfaces.processed).alpha(), 1.0);
    assert_eq!(color(&app, surfaces.paper), COLOR_PAPER);
    assert_eq!(color(&app, surfaces.menu).alpha(), 1.0);
    assert_eq!(color(&app, surfaces.settings).alpha(), 1.0);

    app.world_mut()
        .resource_mut::<EditorState>()
        .processed_glass = true;
    app.update();
    assert!(color(&app, surfaces.processed).alpha() < 1.0);
    assert_eq!(color(&app, surfaces.paper), COLOR_PAPER);
    assert_eq!(color(&app, surfaces.plain), COLOR_PANEL_BODY_PLAIN);

    app.world_mut().resource_mut::<EditorState>().settings_glass = true;
    app.update();
    assert!(color(&app, surfaces.menu).alpha() < 1.0);
    assert!(color(&app, surfaces.settings).alpha() < 1.0);
    assert_eq!(color(&app, surfaces.status).alpha(), 1.0);

    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.explorer_glass = false;
        state.processed_glass = false;
        state.settings_glass = false;
    }
    app.update();
    for entity in [
        surfaces.root,
        surfaces.menu,
        surfaces.explorer,
        surfaces.processed,
        surfaces.settings,
    ] {
        assert_eq!(color(&app, entity).alpha(), 1.0);
    }
    assert_eq!(color(&app, surfaces.paper), COLOR_PAPER);
}

#[test]
fn native_availability_restores_backgrounds_without_changing_preferences() {
    let (mut app, surfaces) = setup();
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.explorer_glass = true;
        state.processed_glass = true;
        state.settings_glass = true;
    }
    app.update();
    assert_eq!(color(&app, surfaces.root).alpha(), 1.0);
    assert_eq!(color(&app, surfaces.explorer).alpha(), 1.0);

    app.world_mut().resource_mut::<NativeGlassState>().active = true;
    app.update();
    assert_eq!(color(&app, surfaces.root), Color::NONE);
    assert!(color(&app, surfaces.explorer).alpha() < 1.0);

    app.world_mut().resource_mut::<NativeGlassState>().active = false;
    app.update();
    for entity in [
        surfaces.root,
        surfaces.menu,
        surfaces.explorer,
        surfaces.processed,
        surfaces.settings,
    ] {
        assert_eq!(color(&app, entity).alpha(), 1.0);
    }
    assert_eq!(color(&app, surfaces.paper), COLOR_PAPER);
    assert!(app.world().resource::<EditorState>().any_glass_enabled());
}

#[test]
fn glass_keeps_canvas_solid_and_preserves_a_lighter_linux_tint() {
    let (mut app, surfaces) = setup();
    app.world_mut().resource_mut::<NativeGlassState>().active = true;
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.processed_glass = true;
        state.explorer_glass = true;
        state.document_format = DocumentFormat::Canvas;
        state.explorer_bg_color = Color::srgba(0.2, 0.4, 0.6, 0.25);
    }
    app.update();
    assert_eq!(color(&app, surfaces.processed), COLOR_CANVAS_BG);
    assert_eq!(color(&app, surfaces.plain), COLOR_PANEL_BODY_PLAIN);
    #[cfg(target_os = "linux")]
    assert_eq!(
        color(&app, surfaces.explorer),
        Color::srgba(0.2, 0.4, 0.6, 0.25)
    );
}
