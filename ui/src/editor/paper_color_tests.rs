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

fn pixels(value: Val) -> f32 {
    match value {
        Val::Px(value) => value,
        other => panic!("expected pixels, got {other:?}"),
    }
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.02,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn continuous_rendering_keeps_headings_caret_and_selection_on_paper() {
    let mut app = setup();
    app.add_systems(Update, render_editor);
    let step = processed_page_step_lines();
    let text = (0..step * 2 + 5)
        .map(|i| {
            if i % 5 == 0 {
                "# Summary Paragraph"
            } else {
                "A character's description."
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.document = Document::from_text(&text);
        state.processed_paginated = false;
        state.cursor.position = Position {
            line: step + 1,
            column: 10,
        };
        state.selection_anchor = Some(Position {
            line: step + 1,
            column: 0,
        });
        state.reparse();
    }
    let papers = (0..PROCESSED_PAPER_CAPACITY)
        .map(|slot| {
            spawn_paper(
                &mut app,
                PanelPaper {
                    kind: PanelKind::Processed,
                    slot,
                },
            )
        })
        .collect::<Vec<_>>();
    let rows = (0..step * PROCESSED_PAPER_CAPACITY)
        .map(|index| {
            app.world_mut()
                .spawn((
                    ProcessedPaperText {
                        slot: index / step,
                        line_offset: index % step,
                    },
                    Node::default(),
                    UiTransform::default(),
                ))
                .id()
        })
        .collect::<Vec<_>>();
    let caret = app
        .world_mut()
        .spawn((
            PanelCaret {
                kind: PanelKind::Processed,
            },
            Node::default(),
            UiTransform::default(),
            Visibility::Hidden,
            BackgroundColor(Color::BLACK),
        ))
        .id();
    let selection = app
        .world_mut()
        .spawn((
            PanelSelectionRect {
                kind: PanelKind::Processed,
                index: 0,
            },
            Node::default(),
            Visibility::Hidden,
            BackgroundColor(Color::NONE),
        ))
        .id();

    for mode in [DisplayMode::Processed, DisplayMode::ProcessedRawCurrentLine] {
        for zoom in [0.8, 1.2, 2.0] {
            for top in [0, step - 1, step + 1] {
                {
                    let mut state = app.world_mut().resource_mut::<EditorState>();
                    state.display_mode = mode;
                    state.set_zoom(zoom);
                    state.processed_top_visual = top;
                    state.processed_top_line = top;
                    state.processed_zoom_anchor_bias_px = -3.0;
                }
                app.update();
                let first = (top / step) * step;
                let count = text.lines().count() - first;
                let mut previous_bottom = None;
                for index in 0..count {
                    let paper = app.world().get::<Node>(papers[index / step]).unwrap();
                    let row = app.world().get::<Node>(rows[index]).unwrap();
                    let row_top = pixels(paper.top) + pixels(row.top);
                    if let Some(bottom) = previous_bottom {
                        // Includes the chunk boundaries where headings used to overlap.
                        assert_close(row_top, bottom);
                    }
                    previous_bottom = Some(row_top + pixels(row.height));
                    if first + index == step + 1 {
                        assert_eq!(
                            *app.world().get::<Visibility>(caret).unwrap(),
                            Visibility::Visible
                        );
                        assert_eq!(
                            *app.world().get::<Visibility>(selection).unwrap(),
                            Visibility::Visible
                        );
                        assert_close(pixels(app.world().get::<Node>(caret).unwrap().top), row_top);
                        assert_close(
                            pixels(app.world().get::<Node>(selection).unwrap().top),
                            row_top,
                        );
                    }
                }
                let paper = app.world().get::<Node>(papers[0]).unwrap();
                let state = app.world().resource::<EditorState>();
                assert_close(
                    pixels(paper.top) + pixels(paper.height),
                    previous_bottom.unwrap() + state.page_margin_bottom * state.zoom,
                );
            }
        }
    }
}

#[test]
fn caret_colors_update_for_each_panels_background_and_vim_mode() {
    let mut app = setup();
    app.add_systems(Update, render_editor);
    let carets = [PanelKind::Plain, PanelKind::Processed].map(|kind| {
        app.world_mut()
            .spawn((
                PanelCaret { kind },
                Node::default(),
                UiTransform::default(),
                Visibility::Hidden,
                BackgroundColor(Color::NONE),
            ))
            .id()
    });
    for (plain, paper, expected) in [
        (Vec4::ONE, Color::BLACK, [Color::BLACK, Color::WHITE]),
        (
            Vec4::new(0.12, 0.13, 0.15, 1.0),
            Color::WHITE,
            [Color::WHITE, Color::BLACK],
        ),
    ] {
        {
            let mut state = app.world_mut().resource_mut::<EditorState>();
            state.ui_colors.set(UiColor::PlainBackground, plain);
            state.paper_bg_color = paper;
            state.vim_enabled = false;
        }
        app.update();
        for (caret, color) in carets.into_iter().zip(expected) {
            assert_eq!(app.world().get::<BackgroundColor>(caret).unwrap().0, color);
        }
        {
            let mut state = app.world_mut().resource_mut::<EditorState>();
            state.vim_enabled = true;
            state.vim_mode = VimMode::Normal;
        }
        app.update();
        for (caret, color) in carets.into_iter().zip(expected) {
            assert_eq!(
                app.world().get::<BackgroundColor>(caret).unwrap().0,
                color.with_alpha(0.5)
            );
        }
    }
}
