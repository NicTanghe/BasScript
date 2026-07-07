fn processed_page_step_lines() -> usize {
    ((A4_HEIGHT_POINTS + PAGE_GAP) / LINE_HEIGHT)
        .round()
        .max(1.0) as usize
}

fn processed_page_geometry(panel_size: Vec2, state: &EditorState) -> ProcessedPageGeometry {
    let zoom = state.zoom.max(f32::EPSILON);
    let paper_width = A4_WIDTH_POINTS * zoom;
    // Keep paper height on the same line grid used by processed pagination.
    let page_step_lines = processed_page_step_lines();
    let paper_height =
        ((page_step_lines as f32 * (LINE_HEIGHT * zoom)) - (PAGE_GAP * zoom)).max(1.0);
    let paper_left = if panel_size.x > paper_width {
        ((panel_size.x - paper_width) * 0.5).max(0.0).round()
    } else {
        PAGE_OUTER_MARGIN
    };
    let paper_top = PAGE_OUTER_MARGIN + markdown_metadata_header_offset(state);

    let margin_left = state.page_margin_left * zoom;
    let margin_right = state.page_margin_right * zoom;
    let margin_top = state.page_margin_top * zoom;
    let margin_bottom = state.page_margin_bottom * zoom;
    let text_left = paper_left + margin_left;
    let text_top = paper_top + margin_top;
    let text_width = (paper_width - margin_left - margin_right).max(1.0);
    let text_height = (paper_height - margin_top - margin_bottom).max(1.0);

    ProcessedPageGeometry {
        paper_left,
        paper_top,
        paper_width,
        paper_height,
        text_left,
        text_top,
        text_width,
        text_height,
    }
}

fn processed_page_layout(panel_size: Vec2, state: &EditorState) -> ProcessedPageLayout {
    let geometry = processed_page_geometry(panel_size, state);
    let page_step_lines = processed_page_step_lines();
    let base_paper_height = ((page_step_lines as f32 * LINE_HEIGHT) - PAGE_GAP).max(1.0);
    let base_text_width =
        (A4_WIDTH_POINTS - state.page_margin_left - state.page_margin_right).max(1.0);
    let base_text_height =
        (base_paper_height - state.page_margin_top - state.page_margin_bottom).max(1.0);
    let base_char_width = default_char_width_for_format(state.document_format).max(0.1);
    let wrap_columns = ((base_text_width / base_char_width) + 1e-4)
        .floor()
        .max(1.0) as usize;
    let lines_per_page = ((base_text_height / LINE_HEIGHT) + 1e-4).floor().max(1.0) as usize;
    let spacer_lines = page_step_lines.saturating_sub(lines_per_page);

    ProcessedPageLayout {
        geometry,
        wrap_columns,
        lines_per_page,
        spacer_lines,
        page_step_lines,
    }
}

fn processed_anchor_line_in_page(processed_view: &ProcessedView, page_step_lines: usize) -> usize {
    processed_view
        .anchor_index
        .saturating_sub(processed_view.start_index)
        .min(page_step_lines.max(1).saturating_sub(1))
}

fn processed_anchor_scroll_offset_px(anchor_line_in_page: usize, line_height: f32) -> f32 {
    anchor_line_in_page as f32 * line_height.max(1.0)
}

fn processed_page_step_px(geometry: &ProcessedPageGeometry, zoom: f32) -> f32 {
    (geometry.paper_height + PAGE_GAP * zoom.max(f32::EPSILON)).max(1.0)
}

fn processed_page_top_for_slot(
    geometry: &ProcessedPageGeometry,
    slot: usize,
    page_step_px: f32,
    anchor_scroll_offset_px: f32,
) -> f32 {
    geometry.paper_top + slot as f32 * page_step_px - anchor_scroll_offset_px
}

fn processed_text_top_for_slot(
    geometry: &ProcessedPageGeometry,
    slot: usize,
    page_step_px: f32,
    anchor_scroll_offset_px: f32,
) -> f32 {
    let page_top =
        processed_page_top_for_slot(geometry, slot, page_step_px, anchor_scroll_offset_px);
    page_top + (geometry.text_top - geometry.paper_top)
}

fn processed_anchor_page_top_for_state(
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
) -> Option<f32> {
    let panel_size = processed_panel_size?;
    let layout = processed_page_layout(panel_size, state);
    let step_lines = layout.page_step_lines.max(1);
    let step_px = processed_page_step_px(&layout.geometry, state.zoom);
    let view_capacity = step_lines.saturating_mul(PROCESSED_PAPER_CAPACITY).max(1);
    let processed_line_height = scaled_line_height(state).max(1.0);
    let all_lines = processed_display_lines(
        state,
        layout.wrap_columns,
        layout.lines_per_page,
        layout.spacer_lines,
    );
    if all_lines.is_empty() {
        return Some(layout.geometry.paper_top + state.processed_zoom_anchor_bias_px);
    }

    let view = build_processed_view(
        &all_lines,
        state.processed_top_visual,
        step_lines,
        view_capacity,
    );
    let anchor_line_in_page = processed_anchor_line_in_page(&view, step_lines);
    let anchor_offset_px =
        processed_anchor_scroll_offset_px(anchor_line_in_page, processed_line_height);
    let page_top = processed_page_top_for_slot(&layout.geometry, 0, step_px, anchor_offset_px)
        + state.processed_zoom_anchor_bias_px;
    Some(page_top)
}

fn set_zoom_preserving_processed_anchor(
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    next_zoom: f32,
) {
    let before_page_top = processed_anchor_page_top_for_state(state, processed_panel_size);
    state.set_zoom(next_zoom);
    let after_page_top = processed_anchor_page_top_for_state(state, processed_panel_size);
    if let (Some(before), Some(after)) = (before_page_top, after_page_top) {
        state.processed_zoom_anchor_bias_px += before - after;
    }
}

fn build_processed_view(
    all_lines: &[ProcessedVisualLine],
    anchor_index: usize,
    page_step_lines: usize,
    max_visible: usize,
) -> ProcessedView {
    let max_visible = max_visible.max(1);
    let page_step_lines = page_step_lines.max(1);
    let mut all_lines = all_lines.to_vec();
    if all_lines.is_empty() {
        return ProcessedView::default();
    }

    let anchor_index = anchor_index.min(all_lines.len().saturating_sub(1));
    let mut start_index = (anchor_index / page_step_lines) * page_step_lines;

    // Keep page-start anchoring near EOF by padding the view window.
    let required_len = start_index.saturating_add(max_visible);
    if all_lines.len() < required_len {
        let pad_source_line = all_lines
            .iter()
            .rfind(|line| !line.is_spacer)
            .map_or(0, |line| line.source_line);
        let missing = required_len.saturating_sub(all_lines.len());
        push_page_spacers(&mut all_lines, pad_source_line, missing);
    }

    let max_start = all_lines.len().saturating_sub(max_visible);
    start_index = start_index.min(max_start);
    let end_index = start_index.saturating_add(max_visible).min(all_lines.len());

    ProcessedView {
        start_index,
        anchor_index,
        lines: all_lines[start_index..end_index].to_vec(),
    }
}

fn processed_segment_ranges(state: &EditorState) -> Vec<(usize, usize, bool)> {
    let mut ranges = Vec::new();
    let mut segment_start = 0usize;

    for (line_index, parsed_line) in state.parsed.iter().enumerate() {
        if is_fountain_page_break_marker(&parsed_line.raw) {
            ranges.push((segment_start, line_index, true));
            segment_start = line_index.saturating_add(1);
        }
    }

    ranges.push((segment_start, state.parsed.len(), false));
    ranges
}

struct PreparedProcessedText {
    text: String,
    display_to_raw: Vec<usize>,
    link_targets: Vec<Option<String>>,
    inline_styles: Vec<InlineTextStyle>,
}

fn identity_link_display_text(input: &str) -> LinkDisplayText {
    let char_count = input.chars().count();
    LinkDisplayText {
        text: input.to_owned(),
        display_to_raw: (0..=char_count).collect(),
    }
}

fn build_link_targets(
    display_to_raw: &[usize],
    script_links: &[ScriptLink],
) -> Vec<Option<String>> {
    let ranges = script_links
        .iter()
        .map(|link| {
            let visible = basscript_core::script_link_visible_column_range(link);
            (
                *visible.start()..visible.end().saturating_add(1),
                link.target.clone(),
            )
        })
        .collect::<Vec<_>>();

    (0..display_to_raw.len().saturating_sub(1))
        .map(|index| {
            let raw_start = display_to_raw[index];
            let raw_end = display_to_raw[index + 1];
            ranges
                .iter()
                .find(|(range, _)| raw_start < range.end && raw_end > range.start)
                .map(|(_, target)| target.clone())
        })
        .collect()
}

fn prepare_processed_line_text(
    parsed_line: &ParsedLine,
    raw_override_active: bool,
    render_override: Option<&ProcessedLineRenderOverride>,
) -> (PreparedProcessedText, Option<bool>) {
    if !raw_override_active && !parsed_line.image_embeds.is_empty() {
        return (prepare_image_embed_line_text(parsed_line), None);
    }

    let (raw_column_base, rendered_raw, checklist_state) = if raw_override_active {
        (0, parsed_line.raw.clone(), None)
    } else {
        render_override
            .and_then(|override_style| {
                markdown_visual_text_for_kind(
                    &parsed_line.raw,
                    &override_style.kind,
                    override_style.markdown_heading_level,
                )
            })
            .or_else(|| markdown_visual_text(parsed_line))
            .unwrap_or_else(|| (0, parsed_line.raw.clone(), None))
    };
    let rendered = if raw_override_active {
        identity_link_display_text(&rendered_raw)
    } else {
        basscript_core::render_script_link_text(&rendered_raw)
    };
    let display_to_raw = rendered
        .display_to_raw
        .iter()
        .map(|column| raw_column_base.saturating_add(*column))
        .collect::<Vec<_>>();
    let link_targets = if raw_override_active {
        vec![None; rendered.text.chars().count()]
    } else {
        build_link_targets(&display_to_raw, &parsed_line.script_links)
    };
    let effective_kind = render_override
        .map(|override_style| &override_style.kind)
        .unwrap_or(&parsed_line.kind);
    let prepared = PreparedProcessedText {
        inline_styles: vec![InlineTextStyle::default(); rendered.text.chars().count()],
        text: rendered.text,
        display_to_raw,
        link_targets,
    };
    let prepared = if !raw_override_active && markdown_inline_emphasis_allowed(effective_kind) {
        apply_markdown_inline_emphasis(prepared)
    } else {
        prepared
    };

    (prepared, checklist_state)
}

fn prepare_image_embed_line_text(parsed_line: &ParsedLine) -> PreparedProcessedText {
    let chars = parsed_line.raw.chars().collect::<Vec<_>>();
    let mut filtered = String::new();
    let mut filtered_to_raw = vec![0usize];
    let mut image_index = 0usize;
    let mut index = 0usize;

    while index < chars.len() {
        let next_embed = parsed_line.image_embeds.get(image_index);
        if let Some(embed) = next_embed {
            if index == embed.raw_start_column {
                index = embed.raw_end_column.min(chars.len());
                image_index += 1;
                continue;
            }
            if index > embed.raw_start_column {
                image_index += 1;
                continue;
            }
        }

        filtered.push(chars[index]);
        filtered_to_raw.push(index + 1);
        index += 1;
    }

    let rendered = basscript_core::render_script_link_text(&filtered);
    let display_to_raw = rendered
        .display_to_raw
        .iter()
        .map(|column| {
            filtered_to_raw
                .get(*column)
                .copied()
                .unwrap_or_else(|| *filtered_to_raw.last().unwrap_or(&0))
        })
        .collect::<Vec<_>>();
    let link_targets = build_link_targets(&display_to_raw, &parsed_line.script_links);

    PreparedProcessedText {
        inline_styles: vec![InlineTextStyle::default(); rendered.text.chars().count()],
        text: rendered.text,
        display_to_raw,
        link_targets,
    }
}

#[derive(Clone, Copy, Debug)]
struct MarkdownOpenDelimiter {
    marker: char,
    start: usize,
    used: usize,
    remaining: usize,
}

fn markdown_inline_emphasis_allowed(kind: &LineKind) -> bool {
    matches!(
        kind,
        LineKind::MarkdownHeading
            | LineKind::MarkdownListItem
            | LineKind::MarkdownQuote
            | LineKind::MarkdownParagraph
    )
}

fn apply_markdown_inline_emphasis(prepared: PreparedProcessedText) -> PreparedProcessedText {
    let chars = prepared.text.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return prepared;
    }

    let code_span_mask = markdown_code_span_mask(&chars);
    let mut remove = vec![false; chars.len()];
    let mut styles = vec![InlineTextStyle::default(); chars.len()];
    let mut stack = Vec::<MarkdownOpenDelimiter>::new();
    let mut index = 0usize;

    while index < chars.len() {
        let marker = chars[index];
        if code_span_mask[index]
            || !matches!(marker, '*' | '_')
            || markdown_delimiter_is_escaped(&chars, index)
        {
            index += 1;
            continue;
        }

        let mut len = 1usize;
        while chars.get(index + len).is_some_and(|ch| *ch == marker) {
            len += 1;
        }

        let (can_open, can_close) = markdown_delimiter_flanking(&chars, index, len, marker);
        let mut consumed_as_close = 0usize;
        if can_close {
            consumed_as_close = close_markdown_emphasis_run(
                &mut stack,
                &mut remove,
                &mut styles,
                marker,
                index,
                len,
            );
        }

        if can_open && consumed_as_close < len {
            stack.push(MarkdownOpenDelimiter {
                marker,
                start: index + consumed_as_close,
                used: 0,
                remaining: len - consumed_as_close,
            });
        }

        index += len;
    }

    build_emphasized_processed_text(prepared, &chars, &remove, &styles)
}

fn close_markdown_emphasis_run(
    stack: &mut Vec<MarkdownOpenDelimiter>,
    remove: &mut [bool],
    styles: &mut [InlineTextStyle],
    marker: char,
    close_start: usize,
    len: usize,
) -> usize {
    let mut close_used = 0usize;
    let mut close_remaining = len;

    while close_remaining > 0 {
        let Some(open_index) = stack
            .iter()
            .rposition(|open| open.marker == marker && open.remaining > 0)
        else {
            break;
        };

        let use_len = markdown_emphasis_pair_len(stack[open_index].remaining, close_remaining);
        let open_remove_start = stack[open_index].start + stack[open_index].used;
        let close_remove_start = close_start + close_used;

        for position in open_remove_start..open_remove_start.saturating_add(use_len) {
            if let Some(slot) = remove.get_mut(position) {
                *slot = true;
            }
        }
        for position in close_remove_start..close_remove_start.saturating_add(use_len) {
            if let Some(slot) = remove.get_mut(position) {
                *slot = true;
            }
        }

        let inline_style = markdown_emphasis_style_for_len(use_len);
        let style_start = open_remove_start.saturating_add(use_len);
        let styles_len = styles.len();
        let style_start = style_start.min(styles_len);
        let style_end = close_start.min(styles_len);
        for style in styles[style_start..style_end].iter_mut() {
            style.bold |= inline_style.bold;
            style.italic |= inline_style.italic;
        }

        stack[open_index].used += use_len;
        stack[open_index].remaining = stack[open_index].remaining.saturating_sub(use_len);
        if stack[open_index].remaining == 0 {
            stack.remove(open_index);
        }

        close_used += use_len;
        close_remaining -= use_len;
    }

    close_used
}

fn markdown_emphasis_pair_len(open_remaining: usize, close_remaining: usize) -> usize {
    if open_remaining >= 3 && close_remaining >= 3 {
        3
    } else if open_remaining >= 2 && close_remaining >= 2 {
        2
    } else {
        1
    }
}

fn markdown_emphasis_style_for_len(len: usize) -> InlineTextStyle {
    InlineTextStyle {
        bold: len >= 2,
        italic: len == 1 || len >= 3,
    }
}

fn build_emphasized_processed_text(
    prepared: PreparedProcessedText,
    chars: &[char],
    remove: &[bool],
    styles: &[InlineTextStyle],
) -> PreparedProcessedText {
    let mut text = String::new();
    let mut display_to_raw = vec![prepared.display_to_raw.first().copied().unwrap_or(0)];
    let mut link_targets = Vec::<Option<String>>::new();
    let mut inline_styles = Vec::<InlineTextStyle>::new();

    for (index, ch) in chars.iter().enumerate() {
        let raw_boundary = prepared
            .display_to_raw
            .get(index + 1)
            .copied()
            .unwrap_or_else(|| *display_to_raw.last().unwrap_or(&0));

        if remove.get(index).copied().unwrap_or(false) {
            if let Some(last) = display_to_raw.last_mut() {
                *last = raw_boundary;
            }
            continue;
        }

        text.push(*ch);
        display_to_raw.push(raw_boundary);
        link_targets.push(prepared.link_targets.get(index).cloned().unwrap_or(None));
        inline_styles.push(styles.get(index).copied().unwrap_or_default());
    }

    PreparedProcessedText {
        text,
        display_to_raw,
        link_targets,
        inline_styles,
    }
}

fn markdown_code_span_mask(chars: &[char]) -> Vec<bool> {
    let mut mask = vec![false; chars.len()];
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '`' {
            index += 1;
            continue;
        }

        let run_len = markdown_same_char_run_len(chars, index, '`');
        let mut search = index + run_len;
        let mut closing = None;
        while search < chars.len() {
            if chars[search] == '`' && markdown_same_char_run_len(chars, search, '`') == run_len {
                closing = Some(search);
                break;
            }
            search += 1;
        }

        if let Some(closing) = closing {
            for slot in &mut mask[index..closing.saturating_add(run_len).min(chars.len())] {
                *slot = true;
            }
            index = closing + run_len;
        } else {
            index += run_len;
        }
    }

    mask
}

fn markdown_same_char_run_len(chars: &[char], start: usize, marker: char) -> usize {
    let mut len = 0usize;
    while chars.get(start + len).is_some_and(|ch| *ch == marker) {
        len += 1;
    }
    len
}

fn markdown_delimiter_is_escaped(chars: &[char], index: usize) -> bool {
    let mut slash_count = 0usize;
    let mut cursor = index;
    while cursor > 0 && chars[cursor - 1] == '\\' {
        slash_count += 1;
        cursor -= 1;
    }
    slash_count % 2 == 1
}

fn markdown_delimiter_flanking(
    chars: &[char],
    start: usize,
    len: usize,
    marker: char,
) -> (bool, bool) {
    let before = start.checked_sub(1).and_then(|index| chars.get(index)).copied();
    let after = chars.get(start + len).copied();
    let before_whitespace = before.map_or(true, char::is_whitespace);
    let after_whitespace = after.map_or(true, char::is_whitespace);
    let before_punctuation = before.is_some_and(|ch| ch.is_ascii_punctuation());
    let after_punctuation = after.is_some_and(|ch| ch.is_ascii_punctuation());

    let left_flanking =
        !after_whitespace && (!after_punctuation || before_whitespace || before_punctuation);
    let right_flanking =
        !before_whitespace && (!before_punctuation || after_whitespace || after_punctuation);

    if marker == '_' {
        (
            left_flanking && (!right_flanking || before_punctuation),
            right_flanking && (!left_flanking || after_punctuation),
        )
    } else {
        (left_flanking, right_flanking)
    }
}

fn build_processed_segment_lines(
    state: &EditorState,
    start_line: usize,
    end_line_exclusive: usize,
    ends_with_hard_break: bool,
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
    raw_override_line: Option<usize>,
) -> Vec<ProcessedVisualLine> {
    let lines_per_page = lines_per_page.max(1);
    let mut paged_lines = Vec::<ProcessedVisualLine>::new();
    let mut lines_in_page = 0usize;
    let markdown_front_matter = markdown_front_matter_display(&state.document);

    for source_line in start_line..end_line_exclusive {
        let Some(parsed_line) = state.parsed.get(source_line) else {
            continue;
        };

        let raw_override_active = raw_override_line == Some(source_line);
        if markdown_front_matter
            .as_ref()
            .is_some_and(|front_matter| source_line <= front_matter.closing_line_index)
        {
            continue;
        }

        let markdown_render_override = (!raw_override_active)
            .then(|| markdown_render_override_for_raw(&parsed_line.raw))
            .flatten();
        let render_override = markdown_render_override;
        let effective_kind = render_override
            .as_ref()
            .map(|override_style| &override_style.kind)
            .unwrap_or(&parsed_line.kind);
        let indent_width = if raw_override_active {
            0
        } else {
            parsed_line.indent_width()
        };
        let uppercase = if raw_override_active {
            false
        } else {
            matches!(
                effective_kind,
                LineKind::SceneHeading | LineKind::Transition | LineKind::Character
            )
        };
        let (prepared_text, checklist_state) =
            prepare_processed_line_text(parsed_line, raw_override_active, render_override.as_ref());
        let mut wrapped = Vec::<ProcessedVisualLine>::new();

        let image_embed_line = !raw_override_active && !parsed_line.image_embeds.is_empty();
        let should_render_text_for_line =
            !image_embed_line || !prepared_text.text.trim().is_empty();

        if !should_render_text_for_line {
            // Image-only lines use the image block itself as the visible processed row.
        } else if should_split_on_double_space(state, effective_kind) {
            for (segment_start, segment_end) in double_space_segments(&prepared_text.text) {
                push_wrapped_visual_lines(
                    &mut wrapped,
                    source_line,
                    indent_width,
                    uppercase,
                    &prepared_text,
                    segment_start,
                    segment_end,
                    wrap_columns,
                );
            }
        } else {
            push_wrapped_visual_lines(
                &mut wrapped,
                source_line,
                indent_width,
                uppercase,
                &prepared_text,
                0,
                prepared_text.text.chars().count(),
                wrap_columns,
            );
        }

        if let Some(checked) = checklist_state {
            if let Some(first_wrapped) = wrapped.first_mut() {
                first_wrapped.markdown_checklist_checked = Some(checked);
            }
        }

        if let Some(render_override) = render_override.clone() {
            for visual_line in &mut wrapped {
                visual_line.render_override = Some(render_override.clone());
            }
        }

        if image_embed_line {
            push_image_embed_visual_lines(&mut wrapped, source_line, &parsed_line.image_embeds);
        }

        push_paged_visual_lines(
            &mut paged_lines,
            wrapped,
            source_line,
            &mut lines_in_page,
            lines_per_page,
            spacer_lines,
        );
    }

    if ends_with_hard_break && lines_in_page > 0 {
        let remaining_content = lines_per_page.saturating_sub(lines_in_page);
        let spacer_total = remaining_content.saturating_add(spacer_lines);
        push_page_spacers(&mut paged_lines, end_line_exclusive, spacer_total);
    }

    paged_lines
}

fn build_processed_cache(
    state: &EditorState,
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
) -> ProcessedCache {
    let mut segments = Vec::<ProcessedSegment>::new();
    let mut lines = Vec::<ProcessedVisualLine>::new();

    for (start_line, end_line_exclusive, ends_with_hard_break) in processed_segment_ranges(state) {
        let segment_lines = build_processed_segment_lines(
            state,
            start_line,
            end_line_exclusive,
            ends_with_hard_break,
            wrap_columns,
            lines_per_page,
            spacer_lines,
            None,
        );
        lines.extend(segment_lines.iter().cloned());
        segments.push(ProcessedSegment {
            start_line,
            end_line_exclusive,
            ends_with_hard_break,
            lines: segment_lines,
        });
    }

    ProcessedCache {
        wrap_columns,
        lines_per_page,
        spacer_lines,
        segments,
        lines,
        source_line_count: state.parsed.len(),
    }
}

fn rebuild_processed_cache_segment(
    state: &EditorState,
    cache: &mut ProcessedCache,
    dirty_line: usize,
) -> bool {
    let Some(segment_index) = cache.segments.iter().position(|segment| {
        dirty_line >= segment.start_line
            && dirty_line < segment.end_line_exclusive.max(segment.start_line + 1)
    }) else {
        return false;
    };

    let segment = &cache.segments[segment_index];
    let updated_lines = build_processed_segment_lines(
        state,
        segment.start_line,
        segment.end_line_exclusive,
        segment.ends_with_hard_break,
        cache.wrap_columns,
        cache.lines_per_page,
        cache.spacer_lines,
        None,
    );
    cache.segments[segment_index].lines = updated_lines;
    cache.lines.clear();
    for segment in &cache.segments {
        cache.lines.extend(segment.lines.iter().cloned());
    }
    true
}

fn ensure_processed_cache(
    state: &mut EditorState,
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
) {
    let requires_full_rebuild = state.processed_cache.as_ref().map_or(true, |cache| {
        cache.wrap_columns != wrap_columns
            || cache.lines_per_page != lines_per_page
            || cache.spacer_lines != spacer_lines
            || cache.source_line_count != state.parsed.len()
    });

    if requires_full_rebuild {
        state.processed_cache = Some(build_processed_cache(
            state,
            wrap_columns,
            lines_per_page,
            spacer_lines,
        ));
        state.processed_cache_dirty_from_line = None;
        return;
    }

    let Some(dirty_line) = state.processed_cache_dirty_from_line.take() else {
        return;
    };

    let marker_near_dirty = state
        .parsed
        .get(dirty_line)
        .is_some_and(|line| is_fountain_page_break_marker(&line.raw))
        || dirty_line
            .checked_sub(1)
            .and_then(|line| state.parsed.get(line))
            .is_some_and(|line| is_fountain_page_break_marker(&line.raw))
        || state
            .parsed
            .get(dirty_line.saturating_add(1))
            .is_some_and(|line| is_fountain_page_break_marker(&line.raw));

    if marker_near_dirty {
        state.processed_cache = Some(build_processed_cache(
            state,
            wrap_columns,
            lines_per_page,
            spacer_lines,
        ));
        return;
    }

    if let Some(mut cache) = state.processed_cache.take() {
        let updated = rebuild_processed_cache_segment(state, &mut cache, dirty_line);
        if updated {
            state.processed_cache = Some(cache);
        } else {
            state.processed_cache = Some(build_processed_cache(
                state,
                wrap_columns,
                lines_per_page,
                spacer_lines,
            ));
        }
    }
}

fn processed_cache_lines<'a>(
    state: &'a mut EditorState,
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
) -> &'a [ProcessedVisualLine] {
    state.ensure_current_script_link_targets_cached();
    ensure_processed_cache(state, wrap_columns, lines_per_page, spacer_lines);
    state
        .processed_cache
        .as_ref()
        .map_or(&[], |cache| cache.lines.as_slice())
}

fn processed_display_lines(
    state: &mut EditorState,
    wrap_columns: usize,
    lines_per_page: usize,
    spacer_lines: usize,
) -> Vec<ProcessedVisualLine> {
    if state.display_mode != DisplayMode::ProcessedRawCurrentLine {
        return processed_cache_lines(state, wrap_columns, lines_per_page, spacer_lines).to_vec();
    }

    state.ensure_current_script_link_targets_cached();
    let raw_override_line = Some(
        state
            .cursor
            .position
            .line
            .min(state.parsed.len().saturating_sub(1)),
    );
    let mut lines = Vec::<ProcessedVisualLine>::new();
    for (start_line, end_line_exclusive, ends_with_hard_break) in processed_segment_ranges(state) {
        let segment_lines = build_processed_segment_lines(
            state,
            start_line,
            end_line_exclusive,
            ends_with_hard_break,
            wrap_columns,
            lines_per_page,
            spacer_lines,
            raw_override_line,
        );
        lines.extend(segment_lines);
    }
    lines
}

fn push_processed_fragment(
    fragments: &mut Vec<ProcessedVisualFragment>,
    text: String,
    is_link: bool,
    link_target: Option<String>,
    inline_style: InlineTextStyle,
) {
    if text.is_empty() {
        return;
    }

    if let Some(previous) = fragments.last_mut() {
        if previous.is_link == is_link
            && previous.link_target == link_target
            && previous.inline_style == inline_style
        {
            previous.text.push_str(&text);
            return;
        }
    }

    fragments.push(ProcessedVisualFragment {
        text,
        is_link,
        link_target,
        inline_style,
    });
}

fn uppercase_processed_text(input: &str, uppercase: bool) -> String {
    if uppercase {
        input.to_ascii_uppercase()
    } else {
        input.to_owned()
    }
}

fn push_wrapped_visual_lines(
    out: &mut Vec<ProcessedVisualLine>,
    source_line: usize,
    indent_width: usize,
    uppercase: bool,
    prepared_text: &PreparedProcessedText,
    segment_start: usize,
    segment_end: usize,
    wrap_columns: usize,
) {
    let chars = prepared_text.text.chars().collect::<Vec<_>>();
    let max_content_columns = wrap_columns.saturating_sub(indent_width).max(1);
    let segment_start = segment_start.min(chars.len());
    let segment_end = segment_end.min(chars.len());

    if segment_start >= segment_end {
        // Keep an actual glyph cell on empty lines so their line box stays stable under zoom.
        let blank_columns = indent_width.max(1);
        let raw_column = prepared_text
            .display_to_raw
            .get(segment_start)
            .copied()
            .unwrap_or(0);
        out.push(ProcessedVisualLine {
            source_line,
            text: " ".repeat(blank_columns),
            fragments: vec![ProcessedVisualFragment {
                text: " ".repeat(blank_columns),
                is_link: false,
                link_target: None,
                inline_style: InlineTextStyle::default(),
            }],
            display_to_raw: vec![raw_column; blank_columns.saturating_add(1)],
            raw_start_column: raw_column,
            raw_end_column: raw_column,
            markdown_checklist_checked: None,
            image_block: None,
            render_override: None,
            is_spacer: false,
        });
        return;
    }

    let mut start = segment_start;
    while start < segment_end {
        let max_end = (start + max_content_columns).min(segment_end);
        let mut split = max_end;

        if max_end < segment_end {
            if let Some(space_index) = (start + 1..max_end).rev().find(|&idx| chars[idx] == ' ') {
                split = space_index;
            }
        }

        if split <= start {
            split = max_end;
        }

        let mut fragments = Vec::<ProcessedVisualFragment>::new();
        if indent_width > 0 {
            push_processed_fragment(
                &mut fragments,
                " ".repeat(indent_width),
                false,
                None,
                InlineTextStyle::default(),
            );
        }

        let mut index = start;
        while index < split {
            let link_target = prepared_text
                .link_targets
                .get(index)
                .cloned()
                .unwrap_or(None);
            let is_link = link_target.is_some();
            let inline_style = prepared_text
                .inline_styles
                .get(index)
                .copied()
                .unwrap_or_default();
            let fragment_start = index;
            index += 1;
            while index < split
                && prepared_text
                    .link_targets
                    .get(index)
                    .cloned()
                    .unwrap_or(None)
                    == link_target
                && prepared_text
                    .inline_styles
                    .get(index)
                    .copied()
                    .unwrap_or_default()
                    == inline_style
            {
                index += 1;
            }

            let fragment_text = chars[fragment_start..index].iter().collect::<String>();
            push_processed_fragment(
                &mut fragments,
                uppercase_processed_text(&fragment_text, uppercase),
                is_link,
                link_target,
                inline_style,
            );
        }

        let line_text = fragments
            .iter()
            .map(|fragment| fragment.text.as_str())
            .collect::<String>();
        let raw_start_column = prepared_text
            .display_to_raw
            .get(start)
            .copied()
            .unwrap_or(0);
        let raw_end_column = prepared_text
            .display_to_raw
            .get(split)
            .copied()
            .unwrap_or(raw_start_column);
        let mut display_to_raw = vec![raw_start_column; indent_width.saturating_add(1)];
        display_to_raw.extend(
            prepared_text.display_to_raw[start.saturating_add(1)..=split]
                .iter()
                .copied(),
        );

        out.push(ProcessedVisualLine {
            source_line,
            text: line_text,
            fragments,
            display_to_raw,
            raw_start_column,
            raw_end_column,
            markdown_checklist_checked: None,
            image_block: None,
            render_override: None,
            is_spacer: false,
        });

        // Skip one wrapping space at the split boundary for word-wrapped output.
        start = split;
        if start < chars.len() && chars[start] == ' ' {
            start += 1;
        }
    }
}

fn push_page_spacers(out: &mut Vec<ProcessedVisualLine>, source_line: usize, count: usize) {
    for _ in 0..count {
        out.push(ProcessedVisualLine {
            source_line,
            text: " ".to_owned(),
            fragments: vec![ProcessedVisualFragment {
                text: " ".to_owned(),
                is_link: false,
                link_target: None,
                inline_style: InlineTextStyle::default(),
            }],
            display_to_raw: vec![0, 0],
            raw_start_column: 0,
            raw_end_column: 0,
            markdown_checklist_checked: None,
            image_block: None,
            render_override: None,
            is_spacer: true,
        });
    }
}

fn push_image_embed_visual_lines(
    out: &mut Vec<ProcessedVisualLine>,
    source_line: usize,
    image_embeds: &[ImageEmbed],
) {
    for embed in image_embeds {
        let reserved_lines = PROCESSED_IMAGE_BLOCK_LINES.max(1);
        out.push(ProcessedVisualLine {
            source_line,
            text: " ".to_owned(),
            fragments: vec![ProcessedVisualFragment {
                text: " ".to_owned(),
                is_link: false,
                link_target: None,
                inline_style: InlineTextStyle::default(),
            }],
            display_to_raw: vec![embed.raw_start_column, embed.raw_end_column],
            raw_start_column: embed.raw_start_column,
            raw_end_column: embed.raw_end_column,
            markdown_checklist_checked: None,
            image_block: Some(ProcessedImageBlock {
                target: embed.target.clone(),
                reserved_lines,
            }),
            render_override: None,
            is_spacer: false,
        });

        for _ in 1..reserved_lines {
            out.push(ProcessedVisualLine {
                source_line,
                text: " ".to_owned(),
                fragments: vec![ProcessedVisualFragment {
                    text: " ".to_owned(),
                    is_link: false,
                    link_target: None,
                    inline_style: InlineTextStyle::default(),
                }],
                display_to_raw: vec![embed.raw_start_column, embed.raw_end_column],
                raw_start_column: embed.raw_start_column,
                raw_end_column: embed.raw_end_column,
                markdown_checklist_checked: None,
                image_block: None,
                render_override: None,
                is_spacer: true,
            });
        }
    }
}

fn push_paged_visual_lines(
    paged_lines: &mut Vec<ProcessedVisualLine>,
    lines: Vec<ProcessedVisualLine>,
    source_line: usize,
    lines_in_page: &mut usize,
    lines_per_page: usize,
    spacer_lines: usize,
) {
    let mut index = 0usize;
    while index < lines.len() {
        let visual_line = lines[index].clone();
        if let Some(image_block) = visual_line.image_block.as_ref() {
            let block_lines = image_block.reserved_lines.max(1);
            if *lines_in_page > 0
                && (*lines_in_page).saturating_add(block_lines) > lines_per_page
            {
                let remaining_content = lines_per_page.saturating_sub(*lines_in_page);
                push_page_spacers(
                    paged_lines,
                    source_line,
                    remaining_content.saturating_add(spacer_lines),
                );
                *lines_in_page = 0;
            }
        }

        if *lines_in_page >= lines_per_page {
            push_page_spacers(paged_lines, source_line, spacer_lines);
            *lines_in_page = 0;
        }

        paged_lines.push(visual_line);
        *lines_in_page = lines_in_page.saturating_add(1);
        index += 1;
    }
}

fn is_fountain_page_break_marker(raw: &str) -> bool {
    let trimmed = raw.trim();
    trimmed.chars().count() >= 3 && trimmed.chars().all(|ch| ch == '=')
}

fn should_split_on_double_space(state: &EditorState, kind: &LineKind) -> bool {
    if matches!(
        kind,
        LineKind::MarkdownHeading
            | LineKind::MarkdownListItem
            | LineKind::MarkdownQuote
            | LineKind::MarkdownCodeFence
            | LineKind::MarkdownCode
            | LineKind::MarkdownRule
            | LineKind::MarkdownParagraph
    ) {
        return false;
    }

    match kind {
        LineKind::Dialogue => state.dialogue_double_space_newline,
        _ => state.non_dialogue_double_space_newline,
    }
}

fn first_visual_index_for_source_line(
    lines: &[ProcessedVisualLine],
    source_line: usize,
) -> Option<usize> {
    lines
        .iter()
        .position(|line| !line.is_spacer && line.source_line >= source_line)
        .or_else(|| {
            lines
                .iter()
                .rposition(|line| !line.is_spacer && line.source_line <= source_line)
        })
}

fn double_space_segments(input: &str) -> Vec<(usize, usize)> {
    let chars = input.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return vec![(0, 0)];
    }

    let mut segments = Vec::<(usize, usize)>::new();
    let mut start = 0usize;
    let mut index = 0usize;

    while index + 1 < chars.len() {
        if chars[index] == ' ' && chars[index + 1] == ' ' {
            segments.push((start, index));
            index += 2;
            start = index;
            continue;
        }

        index += 1;
    }

    segments.push((start, chars.len()));

    segments
}

fn processed_raw_column_from_display(
    visual_line: &ProcessedVisualLine,
    display_column: usize,
) -> usize {
    let last_index = visual_line.display_to_raw.len().saturating_sub(1);
    let display_column = display_column.min(last_index);
    visual_line
        .display_to_raw
        .get(display_column)
        .copied()
        .unwrap_or(visual_line.raw_end_column)
}

fn processed_display_column_from_raw(
    visual_line: &ProcessedVisualLine,
    raw_column: usize,
) -> usize {
    let mut display_column = 0usize;
    let clamped = raw_column.clamp(visual_line.raw_start_column, visual_line.raw_end_column);

    for (index, mapped_raw) in visual_line.display_to_raw.iter().enumerate() {
        if *mapped_raw <= clamped {
            display_column = index;
        } else {
            break;
        }
    }

    display_column.min(visual_line.text.chars().count())
}

fn processed_caret_visual<'a>(
    state: &EditorState,
    processed_view: &'a ProcessedView,
) -> Option<(usize, usize, &'a str)> {
    processed_cursor_visual_from_lines(state, &processed_view.lines)
}

fn processed_cursor_visual_from_lines<'a>(
    state: &EditorState,
    lines: &'a [ProcessedVisualLine],
) -> Option<(usize, usize, &'a str)> {
    let source_line = state.cursor.position.line;
    let raw_column = state.cursor.position.column;

    let relevant = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| !line.is_spacer && line.source_line == source_line)
        .collect::<Vec<_>>();

    let (default_index, default_line) = *relevant.last()?;

    for (entry_index, (visual_index, visual_line)) in relevant.iter().enumerate() {
        let next_start = relevant
            .get(entry_index + 1)
            .map(|(_, next_line)| next_line.raw_start_column);

        let is_last_entry = entry_index + 1 == relevant.len();
        let inside_visual_line = raw_column < visual_line.raw_end_column;
        let before_next_line = next_start.is_some_and(|start| raw_column < start);
        let at_final_line_end = is_last_entry && raw_column <= visual_line.raw_end_column;

        if inside_visual_line || before_next_line || at_final_line_end {
            return Some((
                *visual_index,
                processed_display_column_from_raw(visual_line, raw_column),
                &visual_line.text,
            ));
        }
    }

    Some((
        default_index,
        processed_display_column_from_raw(default_line, raw_column),
        &default_line.text,
    ))
}

fn nearest_non_spacer_visual_index(lines: &[ProcessedVisualLine], index: usize) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }
    if lines.get(index).is_some_and(|line| !line.is_spacer) {
        return Some(index);
    }

    for distance in 1..lines.len() {
        let forward = index.saturating_add(distance);
        if lines.get(forward).is_some_and(|line| !line.is_spacer) {
            return Some(forward);
        }

        let backward = index.saturating_sub(distance);
        if lines.get(backward).is_some_and(|line| !line.is_spacer) {
            return Some(backward);
        }
    }

    None
}

fn processed_visual_fragment_for_part(
    visual_line: &ProcessedVisualLine,
    part_index: usize,
) -> Option<ProcessedVisualFragment> {
    if part_index >= PROCESSED_LINE_SPAN_PARTS {
        return None;
    }

    if visual_line.fragments.len() <= PROCESSED_LINE_SPAN_PARTS {
        return visual_line.fragments.get(part_index).cloned();
    }

    if part_index + 1 < PROCESSED_LINE_SPAN_PARTS {
        return visual_line.fragments.get(part_index).cloned();
    }

    let start = PROCESSED_LINE_SPAN_PARTS.saturating_sub(1);
    let tail = visual_line.fragments.get(start..)?;
    let all_same_link_state = tail.iter().all(|fragment| {
        fragment.is_link == tail[0].is_link && fragment.link_target == tail[0].link_target
    });
    let all_same_inline_style = tail
        .iter()
        .all(|fragment| fragment.inline_style == tail[0].inline_style);

    Some(ProcessedVisualFragment {
        text: tail
            .iter()
            .map(|fragment| fragment.text.as_str())
            .collect::<String>(),
        is_link: all_same_link_state && tail[0].is_link,
        link_target: if all_same_link_state {
            tail[0].link_target.clone()
        } else {
            None
        },
        inline_style: if all_same_inline_style {
            tail[0].inline_style
        } else {
            InlineTextStyle::default()
        },
    })
}

fn processed_visual_fragment_raw_range(
    visual_line: &ProcessedVisualLine,
    part_index: usize,
) -> Option<(usize, usize)> {
    if part_index >= PROCESSED_LINE_SPAN_PARTS {
        return None;
    }

    let fragment_index = if visual_line.fragments.len() <= PROCESSED_LINE_SPAN_PARTS
        || part_index + 1 < PROCESSED_LINE_SPAN_PARTS
    {
        part_index
    } else {
        PROCESSED_LINE_SPAN_PARTS.saturating_sub(1)
    };
    let display_start = visual_line
        .fragments
        .iter()
        .take(fragment_index)
        .map(|fragment| fragment.text.chars().count())
        .sum::<usize>();
    let display_end = if fragment_index == PROCESSED_LINE_SPAN_PARTS.saturating_sub(1)
        && visual_line.fragments.len() > PROCESSED_LINE_SPAN_PARTS
    {
        visual_line.text.chars().count()
    } else {
        display_start.saturating_add(
            visual_line
                .fragments
                .get(fragment_index)?
                .text
                .chars()
                .count(),
        )
    };

    Some((
        visual_line
            .display_to_raw
            .get(display_start)
            .copied()
            .unwrap_or(visual_line.raw_start_column),
        visual_line
            .display_to_raw
            .get(display_end)
            .copied()
            .unwrap_or(visual_line.raw_end_column),
    ))
}

fn processed_visual_fragment_count(visual_line: &ProcessedVisualLine) -> usize {
    visual_line
        .fragments
        .len()
        .min(PROCESSED_LINE_SPAN_PARTS)
        .max(1)
}

#[cfg(test)]
mod processed_markdown_inline_tests {
    use super::*;

    fn style(bold: bool, italic: bool) -> InlineTextStyle {
        InlineTextStyle { bold, italic }
    }

    fn prepared_markdown(raw: &str) -> PreparedProcessedText {
        let document = Document::from_text(raw);
        let parsed = parse_document_with_format(&document, DocumentFormat::Markdown);
        prepare_processed_line_text(&parsed[0], false, None).0
    }

    fn style_for_text(prepared: &PreparedProcessedText, needle: &str) -> InlineTextStyle {
        let byte_start = prepared
            .text
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not found in {:?}", prepared.text));
        let char_start = prepared.text[..byte_start].chars().count();
        let char_len = needle.chars().count();
        let styles = &prepared.inline_styles[char_start..char_start + char_len];
        let first = styles[0];
        assert!(
            styles.iter().all(|style| *style == first),
            "{needle:?} should have one inline style"
        );
        first
    }

    #[test]
    fn parses_markdown_asterisk_emphasis_into_processed_styles() {
        let prepared = prepared_markdown("Plain *italic*, **bold**, and ***both***.");

        assert_eq!(prepared.text, "Plain italic, bold, and both.");
        assert_eq!(style_for_text(&prepared, "Plain"), InlineTextStyle::default());
        assert_eq!(style_for_text(&prepared, "italic"), style(false, true));
        assert_eq!(style_for_text(&prepared, "bold"), style(true, false));
        assert_eq!(style_for_text(&prepared, "both"), style(true, true));

        let mut lines = Vec::<ProcessedVisualLine>::new();
        push_wrapped_visual_lines(
            &mut lines,
            0,
            0,
            false,
            &prepared,
            0,
            prepared.text.chars().count(),
            120,
        );
        let styled_fragments = lines[0]
            .fragments
            .iter()
            .filter(|fragment| fragment.inline_style != InlineTextStyle::default())
            .map(|fragment| (fragment.text.as_str(), fragment.inline_style))
            .collect::<Vec<_>>();
        assert_eq!(
            styled_fragments,
            vec![
                ("italic", style(false, true)),
                ("bold", style(true, false)),
                ("both", style(true, true)),
            ]
        );
    }

    #[test]
    fn parses_markdown_underscore_emphasis_without_touching_words() {
        let prepared = prepared_markdown("snake_case and _italic_ and __bold__");

        assert_eq!(prepared.text, "snake_case and italic and bold");
        assert_eq!(
            style_for_text(&prepared, "snake_case"),
            InlineTextStyle::default()
        );
        assert_eq!(style_for_text(&prepared, "italic"), style(false, true));
        assert_eq!(style_for_text(&prepared, "bold"), style(true, false));
    }

    #[test]
    fn skips_markdown_emphasis_inside_code_spans() {
        let prepared = prepared_markdown("`*literal*` and *italic*");

        assert_eq!(prepared.text, "`*literal*` and italic");
        assert_eq!(
            style_for_text(&prepared, "*literal*"),
            InlineTextStyle::default()
        );
        assert_eq!(style_for_text(&prepared, "italic"), style(false, true));
    }

    #[test]
    fn leaves_markdown_emphasis_raw_on_current_line_override() {
        let document = Document::from_text("Plain *italic*");
        let parsed = parse_document_with_format(&document, DocumentFormat::Markdown);
        let prepared = prepare_processed_line_text(&parsed[0], true, None).0;

        assert_eq!(prepared.text, "Plain *italic*");
        assert!(
            prepared
                .inline_styles
                .iter()
                .all(|style| *style == InlineTextStyle::default())
        );
    }

    #[test]
    fn combines_inline_markdown_style_with_base_font_variant() {
        let fragment = ProcessedVisualFragment {
            text: "heading italic".to_owned(),
            is_link: false,
            link_target: None,
            inline_style: style(false, true),
        };

        assert_eq!(
            font_variant_for_processed_fragment(
                FontVariant::Bold,
                &fragment,
                DocumentFormat::Markdown
            ),
            FontVariant::BoldItalic
        );
    }
}

fn apply_processed_styles(
    processed_span_query: &mut Query<
        (
            &ProcessedPaperLineSpan,
            &mut TextSpan,
            &mut TextFont,
            &mut LineHeight,
            &mut TextColor,
        ),
        Without<PanelText>,
    >,
    state: &EditorState,
    processed_lines: &[ProcessedVisualLine],
    first_visible_page: usize,
    page_step_lines: usize,
    lines_per_page: usize,
    fonts: &EditorFonts,
    font_size: f32,
    line_height: f32,
) {
    let page_step_lines = page_step_lines.max(1);
    let lines_per_page = lines_per_page.max(1).min(page_step_lines);

    for (processed_span, mut text_span, mut text_font, mut text_line_height, mut text_color) in
        processed_span_query.iter_mut()
    {
        let page_index = first_visible_page.saturating_add(processed_span.slot);
        let line_offset = processed_span
            .line_offset
            .min(page_step_lines.saturating_sub(1));
        let page_start = page_index.saturating_mul(page_step_lines);
        let global_index = page_start.saturating_add(line_offset);

        if line_offset >= lines_per_page {
            **text_span = String::new();
            text_font.font = fonts.regular.clone();
            text_font.font_size = font_size;
            *text_line_height = LineHeight::Px(line_height);
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        }

        let Some(visual_line) = processed_lines.get(global_index) else {
            **text_span = if processed_span.part_index == 0 && line_offset + 1 < lines_per_page {
                "\n".to_owned()
            } else {
                String::new()
            };
            text_font.font = fonts.regular.clone();
            text_font.font_size = font_size;
            *text_line_height = LineHeight::Px(line_height);
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        };
        let raw_current_line_mode_active = state.display_mode
            == DisplayMode::ProcessedRawCurrentLine
            && visual_line.source_line == state.cursor.position.line;
        let (style, allow_link_color) = if visual_line.is_spacer {
            (transparent_line_render_style(), false)
        } else if let Some(render_override) = visual_line.render_override.as_ref() {
            (
                processed_line_style_for_kind(
                    &render_override.kind,
                    render_override.markdown_heading_level,
                ),
                true,
            )
        } else if raw_current_line_mode_active {
            (default_line_render_style(), false)
        } else if let Some(parsed_line) = state.parsed.get(visual_line.source_line) {
            (processed_line_style(parsed_line), true)
        } else {
            (default_line_render_style(), false)
        };

        let used_fragment_count = processed_visual_fragment_count(visual_line);
        let Some(mut fragment) =
            processed_visual_fragment_for_part(visual_line, processed_span.part_index)
        else {
            **text_span = String::new();
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        };

        if processed_span.part_index + 1 == used_fragment_count && line_offset + 1 < lines_per_page
        {
            fragment.text.push('\n');
        }

        let effective_variant = font_variant_for_processed_fragment(
            style.font_variant,
            &fragment,
            state.document_format,
        );
        let fragment_raw_range =
            processed_visual_fragment_raw_range(visual_line, processed_span.part_index);
        text_font.font =
            font_for_variant_with_format(fonts, effective_variant, state.document_format);
        text_font.font_size = font_size * style.font_scale;
        *text_line_height = LineHeight::Px(line_height * style.line_height_scale);
        **text_span = fragment.text;
        text_color.0 = if allow_link_color && fragment.is_link {
            let hovered = state
                .hovered_processed_link
                .as_ref()
                .is_some_and(|hovered| {
                    fragment_raw_range.is_some_and(|(raw_start, raw_end)| {
                        hovered.source_line == visual_line.source_line
                            && raw_start < hovered.raw_end_column
                            && raw_end > hovered.raw_start_column
                    })
                });
            if hovered {
                state.hovered_processed_link_color_for_target(fragment.link_target.as_deref())
            } else {
                state.processed_link_color_for_target(fragment.link_target.as_deref())
            }
        } else {
            style.color
        };
    }
}

fn panel_layout_info<'a>(
    text_layout_query: &'a Query<(&PanelText, &TextLayoutInfo)>,
    kind: PanelKind,
) -> Option<&'a TextLayoutInfo> {
    text_layout_query
        .iter()
        .find(|(panel_text, _)| panel_text.kind == kind)
        .map(|(_, layout)| layout)
}

fn layout_line_bounds(layout: &TextLayoutInfo, inverse_scale: f32) -> Vec<(usize, f32, f32)> {
    let mut per_line = BTreeMap::<usize, (f32, f32)>::new();

    for glyph in &layout.glyphs {
        let top = glyph.position.y * inverse_scale;
        let bottom = (glyph.position.y + glyph.size.y) * inverse_scale;
        let entry = per_line.entry(glyph.line_index).or_insert((top, bottom));
        entry.0 = entry.0.min(top);
        entry.1 = entry.1.max(bottom);
    }

    per_line
        .into_iter()
        .map(|(line_index, (top, bottom))| (line_index, top, bottom))
        .collect()
}

fn median(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len().saturating_sub(1) / 2])
}

fn default_line_step(samples: &[(usize, f32)], fallback_height: f32) -> f32 {
    let mut steps = samples
        .windows(2)
        .filter_map(|window| {
            let left = window[0];
            let right = window[1];
            let index_delta = right.0.saturating_sub(left.0);
            if index_delta == 0 {
                return None;
            }

            let step = (right.1 - left.1) / index_delta as f32;
            (step.is_finite() && step.abs() > 0.1).then_some(step)
        })
        .collect::<Vec<_>>();

    median(&mut steps).unwrap_or(fallback_height.max(1.0))
}

fn interpolate_line_value(samples: &[(usize, f32)], line_index: usize, step: f32) -> Option<f32> {
    if samples.is_empty() {
        return None;
    }

    match samples.binary_search_by_key(&line_index, |(index, _)| *index) {
        Ok(position) => Some(samples[position].1),
        Err(insert) if insert > 0 && insert < samples.len() => {
            let (left_index, left_value) = samples[insert - 1];
            let (right_index, right_value) = samples[insert];
            let index_span = right_index.saturating_sub(left_index).max(1);
            let t = line_index.saturating_sub(left_index) as f32 / index_span as f32;
            Some(left_value + (right_value - left_value) * t)
        }
        Err(0) => {
            let (first_index, first_value) = samples[0];
            Some(first_value - step * first_index.saturating_sub(line_index) as f32)
        }
        Err(_) => {
            let (last_index, last_value) = samples[samples.len().saturating_sub(1)];
            Some(last_value + step * line_index.saturating_sub(last_index) as f32)
        }
    }
}

fn line_top_from_layout(
    layout: &TextLayoutInfo,
    line_index: usize,
    inverse_scale: f32,
) -> Option<f32> {
    let bounds = layout_line_bounds(layout, inverse_scale);
    let mut heights = bounds
        .iter()
        .map(|(_, top, bottom)| (bottom - top).max(1.0))
        .collect::<Vec<_>>();
    let fallback_height = median(&mut heights).unwrap_or(LINE_HEIGHT);
    let top_samples = bounds
        .iter()
        .map(|(index, top, _)| (*index, *top))
        .collect::<Vec<_>>();
    let step = default_line_step(&top_samples, fallback_height);

    interpolate_line_value(&top_samples, line_index, step)
}

fn line_index_from_layout_y(
    layout: &TextLayoutInfo,
    y: f32,
    visible_lines: usize,
    inverse_scale: f32,
) -> Option<usize> {
    let bounds = layout_line_bounds(layout, inverse_scale);
    if bounds.is_empty() {
        return None;
    }

    let mut heights = bounds
        .iter()
        .map(|(_, top, bottom)| (bottom - top).max(1.0))
        .collect::<Vec<_>>();
    let fallback_height = median(&mut heights).unwrap_or(LINE_HEIGHT);

    let center_samples = bounds
        .iter()
        .map(|(index, top, bottom)| (*index, (*top + *bottom) * 0.5))
        .collect::<Vec<_>>();
    let center_step = default_line_step(&center_samples, fallback_height);

    let mut best_line = 0usize;
    let mut best_distance = f32::MAX;
    for line in 0..visible_lines.max(1) {
        let Some(center_y) = interpolate_line_value(&center_samples, line, center_step) else {
            continue;
        };

        let distance = (center_y - y).abs();
        if distance < best_distance {
            best_distance = distance;
            best_line = line;
        }
    }

    Some(best_line)
}

fn line_boundaries(
    layout: &TextLayoutInfo,
    line_index: usize,
    line_text: &str,
    inverse_scale: f32,
    fallback_char_width: f32,
) -> Vec<(usize, f32)> {
    let line_len = line_text.len();
    let mut glyphs = layout
        .glyphs
        .iter()
        .filter(|glyph| glyph.line_index == line_index)
        .collect::<Vec<_>>();

    if glyphs.is_empty() {
        let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
        for byte_index in 0..=line_len {
            boundaries.push((byte_index, byte_index as f32 * fallback_char_width));
        }
        return boundaries;
    }

    glyphs.sort_by_key(|glyph| (glyph.byte_index, glyph.byte_length));
    let mut step_candidates = glyphs
        .windows(2)
        .filter_map(|window| {
            let left = window[0];
            let right = window[1];
            let byte_gap = right.byte_index.saturating_sub(left.byte_index);
            if byte_gap == 0 {
                return None;
            }
            let step = (right.position.x - left.position.x) * inverse_scale / byte_gap as f32;
            (step.is_finite() && step.abs() > 0.1).then_some(step)
        })
        .collect::<Vec<_>>();

    step_candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let byte_step = step_candidates
        .get(step_candidates.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(fallback_char_width);

    let mut anchors = BTreeMap::<usize, Vec<f32>>::new();

    for glyph in glyphs {
        let start = glyph.byte_index.min(line_len);
        let end = glyph
            .byte_index
            .saturating_add(glyph.byte_length)
            .min(line_len);
        let span_bytes = end.saturating_sub(start).max(1);
        let half_width = byte_step * span_bytes as f32 * 0.5;
        let center_x = glyph.position.x * inverse_scale;
        let left = center_x - half_width;
        let right = center_x + half_width;

        anchors.entry(start).or_default().push(left);
        anchors.entry(end).or_default().push(right);
    }

    let mut known = anchors
        .into_iter()
        .map(|(byte_index, xs)| {
            let sum = xs.iter().copied().sum::<f32>();
            (byte_index, sum / xs.len() as f32)
        })
        .collect::<Vec<_>>();

    if known.is_empty() {
        let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
        for byte_index in 0..=line_len {
            boundaries.push((byte_index, byte_index as f32 * fallback_char_width));
        }
        return boundaries;
    }

    known.sort_by_key(|(byte_index, _)| *byte_index);

    let first = known[0];
    let last = known[known.len().saturating_sub(1)];
    let mut boundaries = Vec::with_capacity(line_len.saturating_add(1));
    let mut segment = 0usize;

    for byte_index in 0..=line_len {
        while segment + 1 < known.len() && known[segment + 1].0 <= byte_index {
            segment += 1;
        }

        let x = if byte_index <= first.0 {
            first.1 - (first.0 - byte_index) as f32 * byte_step
        } else if byte_index >= last.0 {
            last.1 + (byte_index - last.0) as f32 * byte_step
        } else {
            let (left_byte, left_x) = known[segment];
            let (right_byte, right_x) = known[segment + 1];
            let gap = right_byte.saturating_sub(left_byte).max(1);
            let t = byte_index.saturating_sub(left_byte) as f32 / gap as f32;
            left_x + (right_x - left_x) * t
        };

        boundaries.push((byte_index, x));
    }

    boundaries
}

fn caret_x_from_layout(
    layout: &TextLayoutInfo,
    line_index: usize,
    line_text: &str,
    byte_index: usize,
    inverse_scale: f32,
    fallback_char_width: f32,
) -> Option<f32> {
    let boundaries = line_boundaries(
        layout,
        line_index,
        line_text,
        inverse_scale,
        fallback_char_width,
    );
    boundaries
        .iter()
        .find(|(byte, _)| *byte >= byte_index)
        .map(|(_, x)| *x)
        .or_else(|| boundaries.last().map(|(_, x)| *x))
}

fn column_from_layout_x(
    layout: &TextLayoutInfo,
    line_index: usize,
    x: f32,
    line_text: &str,
    inverse_scale: f32,
    fallback_char_width: f32,
) -> Option<usize> {
    let boundaries = line_boundaries(
        layout,
        line_index,
        line_text,
        inverse_scale,
        fallback_char_width,
    );
    let (best_byte, _) = boundaries.iter().min_by(|(_, ax), (_, bx)| {
        (*ax - x)
            .abs()
            .partial_cmp(&(*bx - x).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;

    Some(byte_to_char_index(line_text, *best_byte))
}

fn char_to_byte_index(input: &str, column: usize) -> usize {
    if column == 0 {
        return 0;
    }

    input
        .char_indices()
        .map(|(byte, _)| byte)
        .nth(column)
        .unwrap_or(input.len())
}

fn byte_to_char_index(input: &str, byte_index: usize) -> usize {
    if byte_index == 0 {
        return 0;
    }

    input
        .char_indices()
        .take_while(|(byte, _)| *byte < byte_index)
        .count()
}

fn is_printable_char(chr: char) -> bool {
    let private_use = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !private_use && !chr.is_ascii_control()
}
