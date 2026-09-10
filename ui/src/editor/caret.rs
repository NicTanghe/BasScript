pub(crate) const CARET_WIDTH: f32 = 2.0;
pub(crate) const VIM_NORMAL_CARET_WIDTH: f32 = 8.0;
pub(crate) const CARET_X_OFFSET: f32 = -1.0;
// Parley's line metrics already provide the line box top. The old glyph
// metadata was center-based, which needed an upward compensation here.
pub(crate) const CARET_VERTICAL_OFFSET_LINES: f32 = 0.0;

#[derive(Component)]
pub(crate) struct PanelCaret {
    pub(crate) kind: PanelKind,
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct CaretBlinkState {
    pub(crate) timer: Timer,
    pub(crate) visible: bool,
}

impl Default for CaretBlinkState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            visible: true,
        }
    }
}

pub(crate) fn blink_caret(
    // Virtual time clamps long idle gaps to 250 ms by default. Caret blinking
    // must follow wall-clock time when the event loop sleeps between frames.
    time: Res<Time<Real>>,
    mut blink: ResMut<CaretBlinkState>,
    state: Res<EditorState>,
    windows: Query<&Window>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
    mut last_tick: Local<Duration>,
) {
    let focused = windows.is_empty() || windows.iter().any(|window| window.focused);
    if !focused {
        return;
    }

    // The render thread's timestamp can be a frame behind after sleeping. Count
    // the idle gap now, rather than blinking twice when that timestamp catches up.
    let now = time.elapsed().saturating_add(
        time.last_update()
            .map_or(Duration::ZERO, |last_update| last_update.elapsed()),
    );
    let delta = now.saturating_sub(*last_tick);
    *last_tick = now;
    if state.is_changed() {
        blink.timer.reset();
        blink.visible = true;
    } else if blink.timer.tick(delta).times_finished_this_tick() % 2 == 1 {
        blink.visible = !blink.visible;
        redraw.write(bevy::window::RequestRedraw);
    }
}

pub(crate) fn caret_vertical_offset(line_height: f32) -> f32 {
    CARET_VERTICAL_OFFSET_LINES * line_height
}

pub(crate) fn caret_width_for_state(state: &EditorState, char_width: f32) -> f32 {
    if state.vim_enabled && state.vim_mode != VimMode::Insert {
        char_width.max(VIM_NORMAL_CARET_WIDTH).max(CARET_WIDTH)
    } else {
        CARET_WIDTH.max(1.0)
    }
}

pub(crate) fn caret_x_offset_for_state(state: &EditorState) -> f32 {
    if state.vim_enabled && state.vim_mode != VimMode::Insert {
        0.0
    } else {
        CARET_X_OFFSET
    }
}

pub(crate) fn caret_color_for_state(state: &EditorState, panel: PanelKind) -> Color {
    let background = match panel {
        PanelKind::Plain => state.ui_colors.color(UiColor::PlainBackground),
        PanelKind::Processed => state.paper_bg_color,
    }
    .to_linear();
    let luminance = 0.2126 * background.red + 0.7152 * background.green + 0.0722 * background.blue;
    let color = if luminance > 0.179 {
        Color::BLACK
    } else {
        Color::WHITE
    };
    if state.vim_enabled && state.vim_mode != VimMode::Insert {
        color.with_alpha(0.5)
    } else {
        color
    }
}

pub(crate) fn processed_caret_line_height(
    state: &EditorState,
    visual_line: &ProcessedVisualLine,
    base_line_height: f32,
) -> f32 {
    let (style, _) = processed_visual_line_style_for_state(state, visual_line);
    (base_line_height * style.line_height_scale).max(1.0)
}

pub(crate) fn render_panel_carets(
    caret_query: &mut Query<
        (
            &PanelCaret,
            &mut Node,
            &mut Visibility,
            &mut UiTransform,
            &mut BackgroundColor,
        ),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCanvas>,
            Without<ProcessedChecklistIcon>,
            Without<ProcessedImageBlockNode>,
        ),
    >,
    state: &EditorState,
    caret_visible: bool,
    visible_lines: usize,
    plain_lines: &[String],
    plain_layout: Option<&ComputedTextBlock>,
    plain_inverse_scale: f32,
    plain_origin_x: f32,
    plain_origin_y: f32,
    plain_char_width: f32,
    plain_line_height: f32,
    processed_view: &ProcessedView,
    first_visible_page: usize,
    processed_page_step_lines: usize,
    processed_lines_per_page: usize,
    processed_text_layout_query: &Query<
        (&ProcessedPaperText, &ComputedTextBlock, &ComputedNode),
        (
            Without<PanelText>,
            Without<PanelPaper>,
            Without<PanelCaret>,
            Without<PanelCanvas>,
        ),
    >,
    processed_geometry: &ProcessedPageGeometry,
    processed_pages: &[ProcessedPagePlacement],
    processed_char_width: f32,
    processed_line_height: f32,
) {
    for (panel_caret, mut node, mut visibility, mut transform, mut color) in caret_query.iter_mut()
    {
        color.set_if_neq(BackgroundColor(caret_color_for_state(
            state,
            panel_caret.kind,
        )));
        if !caret_visible {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }

        let (
            line_offset,
            display_column,
            line_text,
            panel_layout,
            panel_inverse_scale,
            origin_x,
            origin_y,
            panel_char_width,
            panel_line_height,
            panel_caret_x_offset,
            panel_caret_width,
            fallback_caret_height,
            clamp_display_column,
            clamp_local_position_to_origin,
        ) = match panel_caret.kind {
            PanelKind::Plain => {
                let in_view = state.cursor.position.line >= state.top_line
                    && state.cursor.position.line < state.top_line + visible_lines;
                if !in_view {
                    *visibility = Visibility::Hidden;
                    continue;
                }

                let line_offset = state.cursor.position.line - state.top_line;
                let line_text = plain_lines
                    .get(line_offset)
                    .map_or("", |line| line.as_str());
                (
                    line_offset,
                    state.cursor.position.column,
                    line_text,
                    plain_layout,
                    plain_inverse_scale,
                    plain_origin_x,
                    plain_origin_y,
                    plain_char_width,
                    plain_line_height,
                    caret_x_offset_for_state(state),
                    caret_width_for_state(state, plain_char_width),
                    plain_line_height,
                    true,
                    true,
                )
            }
            PanelKind::Processed => {
                let Some((visual_index, display_column, visual_line)) =
                    processed_caret_visual(state, processed_view)
                else {
                    *visibility = Visibility::Hidden;
                    continue;
                };

                let global_index = processed_view.start_index.saturating_add(visual_index);
                let page_index = global_index / processed_page_step_lines;
                let line_in_page = global_index % processed_page_step_lines;
                if line_in_page >= processed_lines_per_page {
                    *visibility = Visibility::Hidden;
                    continue;
                }
                let slot = page_index.saturating_sub(first_visible_page);
                let (processed_layout, processed_inverse_scale) = processed_text_layout_query
                    .iter()
                    .find(|(paper_text, _, _)| {
                        paper_text.slot == slot && paper_text.line_offset == line_in_page
                    })
                    .map_or((None, 1.0), |(_, text_block, computed)| {
                        (Some(text_block), computed.inverse_scale_factor())
                    });

                let Some(page) = processed_pages.get(slot) else {
                    *visibility = Visibility::Hidden;
                    continue;
                };
                let page_text_top = page.text_top;
                let page_start_in_view = slot.saturating_mul(processed_page_step_lines);
                let line_top = processed_visual_line_top_units(
                    state,
                    &processed_view.lines,
                    page_start_in_view,
                    line_in_page,
                ) * processed_line_height;
                let (processed_origin_x, processed_origin_y) = (
                    processed_geometry.text_left - state.processed_horizontal_scroll,
                    page_text_top + line_top,
                );
                (
                    0,
                    display_column,
                    visual_line.text.as_str(),
                    processed_layout,
                    processed_inverse_scale,
                    processed_origin_x,
                    processed_origin_y,
                    processed_char_width,
                    processed_line_height,
                    caret_x_offset_for_state(state),
                    caret_width_for_state(state, processed_char_width),
                    processed_caret_line_height(state, visual_line, processed_line_height),
                    true,
                    true,
                )
            }
        };

        let display_column = if clamp_display_column {
            display_column.min(line_text.chars().count())
        } else {
            display_column
        };
        let byte_index = char_to_byte_index(line_text, display_column);
        let caret_x = panel_layout
            .and_then(|text_block| {
                caret_x_from_layout(
                    text_block,
                    line_offset,
                    line_text,
                    byte_index,
                    panel_inverse_scale,
                    panel_char_width,
                )
            })
            .unwrap_or(display_column as f32 * panel_char_width);
        let caret_top = panel_layout
            .and_then(|text_block| {
                line_top_from_layout(text_block, line_offset, panel_inverse_scale)
            })
            .unwrap_or(line_offset as f32 * panel_line_height);
        let caret_height = panel_layout
            .and_then(|text_block| {
                line_height_from_layout(text_block, line_offset, panel_inverse_scale)
            })
            .unwrap_or(fallback_caret_height)
            .max(1.0);

        let local_caret_left = if clamp_local_position_to_origin {
            (caret_x + panel_caret_x_offset).max(0.0)
        } else {
            caret_x + panel_caret_x_offset
        };
        let caret_left = origin_x + local_caret_left;
        let caret_y_offset = caret_vertical_offset(caret_height);
        let local_caret_top = if clamp_local_position_to_origin {
            (caret_top + caret_y_offset).max(0.0)
        } else {
            caret_top + caret_y_offset
        };
        let caret_top = origin_y + local_caret_top;
        let left = px(caret_left);
        let top = px(caret_top);
        let width = px(panel_caret_width);
        let height = px(caret_height);
        if node.left != left {
            node.left = left;
        }
        if node.top != top {
            node.top = top;
        }
        if node.width != width {
            node.width = width;
        }
        if node.height != height {
            node.height = height;
        }
        if transform.scale != Vec2::ONE {
            transform.scale = Vec2::ONE;
        }
        if transform.translation != Val2::ZERO {
            transform.translation = Val2::ZERO;
        }
        visibility.set_if_neq(Visibility::Visible);
    }
}
#[allow(unused_imports)]
use super::*;

#[cfg(test)]
mod caret_tests {
    use super::*;

    #[test]
    fn caret_blink_state_default() {
        let blink = CaretBlinkState::default();
        assert!(blink.visible);
        assert_eq!(blink.timer.duration(), Duration::from_millis(500));
        assert_eq!(blink.timer.mode(), TimerMode::Repeating);
    }

    #[test]
    fn blink_caret_toggles_and_idle_does_not_mutate_editor_state() {
        let mut app = App::new();
        app.init_resource::<EditorState>()
            .init_resource::<CaretBlinkState>()
            .init_resource::<Time<Real>>()
            .add_message::<bevy::window::RequestRedraw>()
            .add_systems(Update, blink_caret);

        // Run initial update frame so change detection settles
        app.update();

        // Advance by 250ms (halfway): should stay visible
        {
            let mut time = app.world_mut().resource_mut::<Time<Real>>();
            time.advance_by(Duration::from_millis(250));
        }
        app.update();
        assert!(app.world().resource::<CaretBlinkState>().visible);
        assert!(!app.world().resource_ref::<EditorState>().is_changed());

        // Advance by another 250ms (reaching 500ms): should toggle to false
        {
            let mut time = app.world_mut().resource_mut::<Time<Real>>();
            time.advance_by(Duration::from_millis(250));
        }
        app.update();
        assert!(!app.world().resource::<CaretBlinkState>().visible);
        assert!(!app.world().resource_ref::<EditorState>().is_changed());

        // Advance by another 500ms: should toggle back to true
        {
            let mut time = app.world_mut().resource_mut::<Time<Real>>();
            time.advance_by(Duration::from_millis(500));
        }
        app.update();
        assert!(app.world().resource::<CaretBlinkState>().visible);
        assert!(!app.world().resource_ref::<EditorState>().is_changed());
    }

    #[test]
    fn blink_caret_resets_on_editor_state_change() {
        let mut app = App::new();
        app.init_resource::<EditorState>()
            .init_resource::<CaretBlinkState>()
            .init_resource::<Time<Real>>()
            .add_message::<bevy::window::RequestRedraw>()
            .add_systems(Update, blink_caret);

        app.update();

        // Advance by 500ms: toggles visible to false
        {
            let mut time = app.world_mut().resource_mut::<Time<Real>>();
            time.advance_by(Duration::from_millis(500));
        }
        app.update();
        assert!(!app.world().resource::<CaretBlinkState>().visible);

        // Mutating EditorState must reset the blink timer and restore visibility
        {
            let mut state = app.world_mut().resource_mut::<EditorState>();
            state.cursor.position.column += 1;
        }
        app.update();
        let blink = app.world().resource::<CaretBlinkState>();
        assert!(blink.visible);
        assert_eq!(blink.timer.elapsed(), Duration::ZERO);
    }

    #[test]
    fn blink_uses_real_time_after_a_reactive_idle_wait() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_millis(500),
            ))
            .init_resource::<EditorState>()
            .init_resource::<CaretBlinkState>()
            .add_message::<bevy::window::RequestRedraw>()
            .add_systems(Update, blink_caret);

        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<Time>().delta(),
            Duration::from_millis(250)
        );
        assert_eq!(
            app.world().resource::<Time<Real>>().delta(),
            Duration::from_millis(500)
        );
        assert!(!app.world().resource::<CaretBlinkState>().visible);
    }

    #[test]
    fn blink_preserves_phase_when_an_idle_wait_spans_multiple_blinks() {
        let mut app = App::new();
        app.init_resource::<EditorState>()
            .init_resource::<CaretBlinkState>()
            .init_resource::<Time<Real>>()
            .add_message::<bevy::window::RequestRedraw>()
            .add_systems(Update, blink_caret);
        app.update();

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(1));
        app.update();
        assert!(app.world().resource::<CaretBlinkState>().visible);
        assert!(
            app.world()
                .resource::<Messages<bevy::window::RequestRedraw>>()
                .is_empty()
        );

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(500));
        app.update();
        assert!(!app.world().resource::<CaretBlinkState>().visible);
    }

    #[test]
    fn catching_up_render_timestamp_does_not_blink_twice() {
        let mut time = Time::<Real>::default();
        time.update_with_instant(bevy::platform::time::Instant::now() - Duration::from_millis(500));
        let mut app = App::new();
        app.init_resource::<EditorState>()
            .init_resource::<CaretBlinkState>()
            .insert_resource(time)
            .add_message::<bevy::window::RequestRedraw>()
            .add_systems(Update, blink_caret);
        app.update();

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(500));
        app.update();
        assert!(!app.world().resource::<CaretBlinkState>().visible);

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .update_with_instant(bevy::platform::time::Instant::now());
        app.update();
        assert!(!app.world().resource::<CaretBlinkState>().visible);
    }
}
