use super::*;

fn setup() -> App {
    let mut app = App::new();
    app.init_resource::<EditorState>()
        .init_resource::<NativeGlassState>()
        .init_resource::<ResolvedPanelWidths>()
        .init_resource::<CaretBlinkState>()
        .insert_resource(EditorFonts {
            regular: default(),
            bold: default(),
            italic: default(),
            bold_italic: default(),
            markdown_regular: default(),
            markdown_bold: default(),
            markdown_italic: default(),
            markdown_bold_italic: default(),
        })
        .insert_resource(ChecklistIcons {
            unchecked: default(),
            checked: default(),
        });
    let mut state = app.world_mut().resource_mut::<EditorState>();
    state.document = Document::from_text(&"Paper color regression.\n".repeat(200));
    state.document_format = DocumentFormat::Markdown;
    state.display_mode = DisplayMode::Processed;
    state.processed_paginated = true;
    state.processed_glass = false;
    state.cursor = Cursor::default();
    state.reparse();
    app
}

fn spawn_paper(app: &mut App, marker: impl Component) -> Entity {
    app.world_mut()
        .spawn((
            marker,
            Node::default(),
            UiTransform::default(),
            Visibility::Hidden,
            BackgroundColor(COLOR_PAPER),
        ))
        .id()
}

fn set_paper_color(app: &mut App, rgba: Vec4) -> Color {
    let mut state = app.world_mut().resource_mut::<EditorState>();
    state.theme_color_target = ThemeColorTarget::PaperBackground;
    set_active_theme_rgba(&mut state, rgba);
    Color::srgba(rgba.x, rgba.y, rgba.z, 1.0)
}

fn assert_paper_color(app: &App, paper: Entity, expected: Color) {
    assert_eq!(
        *app.world().get::<Visibility>(paper).unwrap(),
        Visibility::Visible
    );
    assert_eq!(
        app.world().get::<BackgroundColor>(paper).unwrap().0,
        expected
    );
}

#[test]
fn paper_color_survives_redraws_in_either_system_order() {
    for sync_first in [false, true] {
        let mut app = setup();
        if sync_first {
            app.add_systems(Update, (sync_glass_surfaces, render_editor).chain());
        } else {
            app.add_systems(Update, (render_editor, sync_glass_surfaces).chain());
        }
        let paper = spawn_paper(
            &mut app,
            PanelPaper {
                kind: PanelKind::Processed,
                slot: 0,
            },
        );

        for rgba in [
            Vec4::new(0.075, 0.075, 0.075, 1.0),
            Vec4::new(0.2, 0.4, 0.6, 0.5),
        ] {
            let expected = set_paper_color(&mut app, rgba);
            for _ in 0..3 {
                app.update();
                assert_paper_color(&app, paper, expected);
            }
        }

        // A newly created page must inherit the theme even when state is unchanged.
        let next_page = spawn_paper(
            &mut app,
            PanelPaper {
                kind: PanelKind::Processed,
                slot: 1,
            },
        );
        app.update();
        assert_paper_color(&app, next_page, Color::srgba(0.2, 0.4, 0.6, 1.0));
    }
}

#[test]
fn paper_stays_opaque_with_glass_in_both_page_modes() {
    let mut app = setup();
    app.add_systems(Update, (render_editor, sync_glass_surfaces).chain());
    let papers = [0, 1].map(|slot| {
        spawn_paper(
            &mut app,
            PanelPaper {
                kind: PanelKind::Processed,
                slot,
            },
        )
    });
    let paper_color = set_paper_color(&mut app, Vec4::new(0.12, 0.14, 0.16, 1.0));

    for paginated in [true, false, true] {
        for (enabled, available) in [(false, true), (true, true), (true, false), (false, true)] {
            {
                let mut state = app.world_mut().resource_mut::<EditorState>();
                state.processed_paginated = paginated;
                state.processed_glass = enabled;
            }
            app.world_mut().resource_mut::<NativeGlassState>().active = available;
            for _ in 0..3 {
                app.update();
                assert_paper_color(&app, papers[0], paper_color);
                assert_paper_color(
                    &app,
                    papers[1],
                    if !paginated { Color::NONE } else { paper_color },
                );
            }
        }
    }
}

#[test]
fn story_query_paper_uses_the_theme_in_both_page_modes() {
    let mut app = setup();
    app.add_systems(Update, sync_story_query_sheet_ui);
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.story_query_sheet.open = true;
        state.story_query_sheet.result_text = "Paper color regression.\n".repeat(200);
    }
    let papers = [0, 1].map(|slot| spawn_paper(&mut app, StoryQueryPaper { slot }));

    for rgba in [
        Vec4::new(0.075, 0.075, 0.075, 1.0),
        Vec4::new(0.2, 0.4, 0.6, 0.5),
    ] {
        let paper_color = set_paper_color(&mut app, rgba);
        for paginated in [true, false, true] {
            app.world_mut()
                .resource_mut::<EditorState>()
                .processed_paginated = paginated;
            for _ in 0..3 {
                app.update();
                assert_paper_color(&app, papers[0], paper_color);
                assert_paper_color(
                    &app,
                    papers[1],
                    if paginated { paper_color } else { Color::NONE },
                );
            }
        }
    }
}
