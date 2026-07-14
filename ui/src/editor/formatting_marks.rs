use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FormattingMarkKey {
    Plain {
        line: usize,
        column: usize,
    },
    Processed {
        slot: usize,
        line_offset: usize,
        column: usize,
    },
    PageBreak {
        slot: usize,
    },
}

#[derive(Component)]
pub(crate) struct FormattingMarkGlyph;

#[derive(Component)]
pub(crate) struct FormattingPageBreakMark {
    pub(crate) source_line: usize,
}

#[derive(Clone)]
struct FormattingMarkSpec {
    parent: Entity,
    text: &'static str,
    left: f32,
    top: f32,
    width: Option<f32>,
    font_size: f32,
    line_height: f32,
    page_break_source_line: Option<usize>,
}

const PAGE_BREAK_LABEL: &str = "……………… Page Break ………………";
const FORMATTING_MARK_COLOR: Color = Color::srgba(0.24, 0.35, 0.55, 0.58);

pub(crate) fn handle_formatting_page_break_click(
    interaction_query: Query<(&Interaction, &FormattingPageBreakMark), Changed<Interaction>>,
    mut state: ResMut<EditorState>,
) {
    for (interaction, page_break) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let target_line = page_break
            .source_line
            .saturating_add(1)
            .min(state.document.line_count().saturating_sub(1));
        state.focused_panel = PanelKind::Processed;
        state.set_cursor(
            Position {
                line: target_line,
                column: 0,
            },
            true,
        );
        state.status_message = "Page break selected; Backspace removes it".to_string();
    }
}

pub(crate) fn sync_formatting_mark_overlays(
    mut commands: Commands,
    mut state: ResMut<EditorState>,
    fonts: Res<EditorFonts>,
    resolved_widths: Res<ResolvedPanelWidths>,
    body_query: Query<(&PanelBody, &ComputedNode)>,
    canvas_query: Query<(Entity, &PanelCanvas)>,
    paper_query: Query<(Entity, &PanelPaper)>,
    plain_layout_query: Query<(&PanelText, &ComputedTextBlock, &ComputedNode)>,
    processed_layout_query: Query<(&ProcessedPaperText, &ComputedTextBlock, &ComputedNode)>,
    mut mark_query: Query<
        (
            &mut Text,
            &mut TextLayout,
            &mut TextFont,
            &mut LineHeight,
            &mut Node,
        ),
        With<FormattingMarkGlyph>,
    >,
    mut entities: Local<BTreeMap<FormattingMarkKey, Entity>>,
) {
    if !state.formatting_marks_visible || state.document_format == DocumentFormat::Canvas {
        for (_, entity) in std::mem::take(&mut *entities) {
            commands.entity(entity).despawn();
        }
        return;
    }

    let mut desired = BTreeMap::<FormattingMarkKey, FormattingMarkSpec>::new();
    collect_plain_formatting_marks(
        &state,
        &body_query,
        &canvas_query,
        &plain_layout_query,
        &mut desired,
    );
    collect_processed_formatting_marks(
        &mut state,
        &resolved_widths,
        &body_query,
        &paper_query,
        &processed_layout_query,
        &mut desired,
    );

    entities.retain(|key, entity| {
        if desired.contains_key(key) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });

    let font = match state.document_format {
        DocumentFormat::Markdown => fonts.markdown_regular.clone(),
        DocumentFormat::Fountain | DocumentFormat::Canvas => fonts.regular.clone(),
    };
    for (key, spec) in desired {
        if let Some(entity) = entities.get(&key).copied() {
            if let Ok((mut text, mut layout, mut text_font, mut line_height, mut node)) =
                mark_query.get_mut(entity)
            {
                **text = spec.text.to_string();
                *layout = formatting_mark_layout(spec.width.is_some());
                text_font.font = font.clone().into();
                text_font.font_size = FontSize::Px(spec.font_size);
                *line_height = LineHeight::Px(spec.line_height);
                apply_formatting_mark_node(&mut node, &spec);
                if let Some(source_line) = spec.page_break_source_line {
                    commands.entity(entity).insert((
                        Interaction::None,
                        Pickable::default(),
                        FormattingPageBreakMark { source_line },
                    ));
                }
                continue;
            }
        }

        let entity = commands
            .spawn((
                Text::new(spec.text),
                formatting_mark_layout(spec.width.is_some()),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(spec.font_size),
                    ..default()
                },
                LineHeight::Px(spec.line_height),
                TextColor(FORMATTING_MARK_COLOR),
                formatting_mark_node(&spec),
                ZIndex(8),
                GlobalZIndex(8),
                FormattingMarkGlyph,
            ))
            .id();
        if let Some(source_line) = spec.page_break_source_line {
            commands.entity(entity).insert((
                Interaction::None,
                Pickable::default(),
                FormattingPageBreakMark { source_line },
            ));
        } else {
            commands.entity(entity).insert(Pickable::IGNORE);
        }
        commands.entity(spec.parent).add_child(entity);
        entities.insert(key, entity);
    }
}

fn collect_plain_formatting_marks(
    state: &EditorState,
    body_query: &Query<(&PanelBody, &ComputedNode)>,
    canvas_query: &Query<(Entity, &PanelCanvas)>,
    layout_query: &Query<(&PanelText, &ComputedTextBlock, &ComputedNode)>,
    desired: &mut BTreeMap<FormattingMarkKey, FormattingMarkSpec>,
) {
    if !state.display_mode.panel_visible(PanelKind::Plain) {
        return;
    }
    let Some(parent) = canvas_query
        .iter()
        .find(|(_, canvas)| canvas.kind == PanelKind::Plain)
        .map(|(entity, _)| entity)
    else {
        return;
    };
    let Some((_, layout, computed)) = layout_query
        .iter()
        .find(|(panel, _, _)| panel.kind == PanelKind::Plain)
    else {
        return;
    };
    let visible_lines = viewport_lines(
        body_query,
        state.display_mode,
        state.measured_line_step,
        scaled_text_padding_y(state),
    );
    let lines = visible_plain_lines(state, visible_lines);
    let inverse_scale = computed.inverse_scale_factor();
    let origin_x = scaled_text_padding_x(state) - state.plain_horizontal_scroll;
    let origin_y = scaled_text_padding_y(state);
    let font_size = scaled_font_size(state);
    let line_height = state.measured_line_step.max(1.0);
    let fallback_width = scaled_char_width(state).max(1.0);

    for (line_offset, line) in lines.iter().enumerate() {
        let boundaries = line_boundaries(layout, line_offset, line, inverse_scale, fallback_width);
        let line_top = line_top_from_layout(layout, line_offset, inverse_scale)
            .unwrap_or(line_offset as f32 * line_height);
        for (column, (byte_index, ch)) in line.char_indices().enumerate() {
            let Some(mark) = whitespace_mark(ch) else {
                continue;
            };
            let left = boundary_x(&boundaries, byte_index);
            desired.insert(
                FormattingMarkKey::Plain {
                    line: line_offset,
                    column,
                },
                FormattingMarkSpec {
                    parent,
                    text: mark,
                    left: origin_x + left,
                    top: origin_y + line_top,
                    width: None,
                    font_size,
                    line_height,
                    page_break_source_line: None,
                },
            );
        }

        let source_line = state.top_line.saturating_add(line_offset);
        let raw = state.document.line(source_line).unwrap_or(line);
        desired.insert(
            FormattingMarkKey::Plain {
                line: line_offset,
                column: line.chars().count(),
            },
            FormattingMarkSpec {
                parent,
                text: line_ending_mark(raw),
                left: origin_x + boundary_x(&boundaries, line.len()),
                top: origin_y + line_top,
                width: None,
                font_size,
                line_height,
                page_break_source_line: None,
            },
        );
    }
}

fn collect_processed_formatting_marks(
    state: &mut EditorState,
    resolved_widths: &ResolvedPanelWidths,
    body_query: &Query<(&PanelBody, &ComputedNode)>,
    paper_query: &Query<(Entity, &PanelPaper)>,
    layout_query: &Query<(&ProcessedPaperText, &ComputedTextBlock, &ComputedNode)>,
    desired: &mut BTreeMap<FormattingMarkKey, FormattingMarkSpec>,
) {
    if !state.display_mode.panel_visible(PanelKind::Processed) {
        return;
    }
    let panel_size = body_query
        .iter()
        .find(|(panel, _)| panel.kind == PanelKind::Processed)
        .map(|(panel, computed)| resolved_widths.panel_size(panel.kind, computed))
        .unwrap_or(Vec2::ZERO);
    let page_layout = processed_page_layout(panel_size, state);
    let page_step = page_layout.page_step_lines.max(1);
    let all_lines = processed_display_lines(
        state,
        page_layout.wrap_columns,
        page_layout.lines_per_page,
        page_layout.spacer_lines,
    );
    if all_lines.is_empty() {
        return;
    }
    let first_page = (state.processed_top_visual.min(all_lines.len() - 1) / page_step)
        .min(processed_page_count_for_lines(&all_lines, page_step).saturating_sub(1));
    let paper_entities = paper_query
        .iter()
        .filter(|(_, paper)| paper.kind == PanelKind::Processed)
        .map(|(entity, paper)| (paper.slot, entity))
        .collect::<BTreeMap<_, _>>();
    let text_left = page_layout.geometry.text_left - page_layout.geometry.paper_left;
    let text_top = page_layout.geometry.text_top - page_layout.geometry.paper_top;
    let font_size = scaled_font_size(state);
    let base_line_height = scaled_line_height(state).max(1.0);
    let fallback_width = scaled_char_width(state).max(1.0);

    for (paper_text, layout, computed) in layout_query.iter() {
        let Some(parent) = paper_entities.get(&paper_text.slot).copied() else {
            continue;
        };
        if paper_text.line_offset >= page_layout.lines_per_page {
            continue;
        }
        let page_index = first_page.saturating_add(paper_text.slot);
        let page_start = page_index.saturating_mul(page_step);
        let global_index = page_start.saturating_add(paper_text.line_offset);
        let Some(visual_line) = all_lines.get(global_index) else {
            continue;
        };
        if visual_line.is_spacer || visual_line.image_block.is_some() {
            continue;
        }
        let boundaries = line_boundaries(
            layout,
            0,
            &visual_line.text,
            computed.inverse_scale_factor(),
            fallback_width,
        );
        let top = text_top
            + processed_visual_line_top_units(
                state,
                &all_lines,
                page_start,
                paper_text.line_offset,
            ) * base_line_height;
        let (style, _) = processed_visual_line_style_for_state(state, visual_line);
        let mark_font_size = font_size * style.font_scale;
        let mark_line_height = base_line_height * style.line_height_scale;
        let raw_chars = state
            .document
            .line(visual_line.source_line)
            .unwrap_or("")
            .chars()
            .collect::<Vec<_>>();

        for (column, (byte_index, ch)) in visual_line.text.char_indices().enumerate() {
            let Some(mark) = whitespace_mark(ch) else {
                continue;
            };
            let raw_start = visual_line.display_to_raw.get(column).copied().unwrap_or(0);
            let raw_end = visual_line
                .display_to_raw
                .get(column + 1)
                .copied()
                .unwrap_or(raw_start);
            if raw_start == raw_end
                || !raw_chars
                    .get(raw_start)
                    .is_some_and(|raw| raw.is_whitespace())
            {
                continue;
            }
            desired.insert(
                FormattingMarkKey::Processed {
                    slot: paper_text.slot,
                    line_offset: paper_text.line_offset,
                    column,
                },
                FormattingMarkSpec {
                    parent,
                    text: mark,
                    left: text_left + boundary_x(&boundaries, byte_index),
                    top,
                    width: None,
                    font_size: mark_font_size,
                    line_height: mark_line_height,
                    page_break_source_line: None,
                },
            );
        }

        let is_last_visual_for_source = all_lines
            .iter()
            .skip(global_index + 1)
            .find(|line| !line.is_spacer)
            .is_none_or(|line| line.source_line != visual_line.source_line);
        if is_last_visual_for_source {
            let raw = state.document.line(visual_line.source_line).unwrap_or("");
            desired.insert(
                FormattingMarkKey::Processed {
                    slot: paper_text.slot,
                    line_offset: paper_text.line_offset,
                    column: visual_line.text.chars().count(),
                },
                FormattingMarkSpec {
                    parent,
                    text: line_ending_mark(raw),
                    left: text_left + boundary_x(&boundaries, visual_line.text.len()),
                    top,
                    width: None,
                    font_size: mark_font_size,
                    line_height: mark_line_height,
                    page_break_source_line: None,
                },
            );
        }
    }

    if state.processed_paginated {
        for slot in 0..PROCESSED_PAPER_CAPACITY {
            let page_index = first_page.saturating_add(slot);
            let page_start = page_index.saturating_mul(page_step);
            let page_end = page_start.saturating_add(page_step).min(all_lines.len());
            let page_lines = &all_lines[page_start.min(all_lines.len())..page_end];
            let page_last_source = page_lines
                .iter()
                .filter(|line| !line.is_spacer)
                .map(|line| line.source_line)
                .max();
            let next_source = all_lines
                .iter()
                .skip(page_end)
                .find(|line| !line.is_spacer)
                .map(|line| line.source_line);
            let explicit_break_source = page_lines
                .iter()
                .find(|line| {
                    line.is_spacer
                        && state
                            .document
                            .line(line.source_line)
                            .is_some_and(is_fountain_page_break_marker)
                })
                .map(|line| line.source_line)
                .or_else(|| {
                    page_last_source.and_then(|last_source| {
                        state
                            .document
                            .lines()
                            .iter()
                            .enumerate()
                            .find(|(source, raw)| {
                                *source > last_source
                                    && next_source.is_none_or(|next| *source < next)
                                    && is_fountain_page_break_marker(raw)
                            })
                            .map(|(source, _)| source)
                    })
                });
            let (Some(source_line), Some(parent)) =
                (explicit_break_source, paper_entities.get(&slot).copied())
            else {
                continue;
            };
            desired.insert(
                FormattingMarkKey::PageBreak { slot },
                FormattingMarkSpec {
                    parent,
                    text: PAGE_BREAK_LABEL,
                    left: 0.0,
                    top: page_layout.geometry.paper_height
                        - (state.page_margin_bottom * state.zoom * 0.62),
                    width: Some(page_layout.geometry.paper_width),
                    font_size: font_size * 0.85,
                    line_height: base_line_height,
                    page_break_source_line: Some(source_line),
                },
            );
        }
    }
}

fn whitespace_mark(ch: char) -> Option<&'static str> {
    match ch {
        ' ' => Some("·"),
        '\t' => Some("→"),
        _ => None,
    }
}

fn line_ending_mark(raw: &str) -> &'static str {
    if raw.ends_with("  ") { "↵" } else { "¶" }
}

fn boundary_x(boundaries: &[(usize, f32)], byte_index: usize) -> f32 {
    boundaries
        .iter()
        .find(|(byte, _)| *byte >= byte_index)
        .or_else(|| boundaries.last())
        .map_or(0.0, |(_, x)| *x)
}

fn formatting_mark_layout(centered: bool) -> TextLayout {
    if centered {
        TextLayout::no_wrap().with_justify(Justify::Center)
    } else {
        TextLayout::no_wrap()
    }
}

fn formatting_mark_node(spec: &FormattingMarkSpec) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };
    apply_formatting_mark_node(&mut node, spec);
    node
}

fn apply_formatting_mark_node(node: &mut Node, spec: &FormattingMarkSpec) {
    node.position_type = PositionType::Absolute;
    node.left = px(spec.left);
    node.top = px(spec.top);
    node.width = spec.width.map_or(Val::Auto, px);
    node.height = px(spec.line_height.max(1.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_paragraph_and_manual_line_breaks() {
        assert_eq!(line_ending_mark("paragraph"), "¶");
        assert_eq!(line_ending_mark("manual  "), "↵");
    }

    #[test]
    fn maps_only_space_and_tab_characters() {
        assert_eq!(whitespace_mark(' '), Some("·"));
        assert_eq!(whitespace_mark('\t'), Some("→"));
        assert_eq!(whitespace_mark('x'), None);
    }
}
