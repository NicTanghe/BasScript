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
    let paper_top = PAGE_OUTER_MARGIN;

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
    let wrap_columns = ((base_text_width / DEFAULT_CHAR_WIDTH) + 1e-4)
        .floor()
        .max(1.0) as usize;
    let lines_per_page = ((base_text_height / LINE_HEIGHT) + 1e-4)
        .floor()
        .max(1.0) as usize;
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

fn processed_anchor_scroll_offset_px(
    anchor_line_in_page: usize,
    line_height: f32,
) -> f32 {
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
    let page_top = processed_page_top_for_slot(geometry, slot, page_step_px, anchor_scroll_offset_px);
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
    let anchor_offset_px = processed_anchor_scroll_offset_px(anchor_line_in_page, processed_line_height);
    let page_top =
        processed_page_top_for_slot(&layout.geometry, 0, step_px, anchor_offset_px)
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

    for source_line in start_line..end_line_exclusive {
        let Some(parsed_line) = state.parsed.get(source_line) else {
            continue;
        };

        let raw_override_active = raw_override_line == Some(source_line);
        let indent_width = if raw_override_active {
            0
        } else {
            parsed_line.indent_width()
        };
        let uppercase = if raw_override_active {
            false
        } else {
            matches!(
                parsed_line.kind,
                LineKind::SceneHeading | LineKind::Transition | LineKind::Character
            )
        };
        let (raw_column_base, rendered_raw, checklist_state) = if raw_override_active {
            (0, parsed_line.raw.clone(), None)
        } else {
            markdown_visual_text(parsed_line).unwrap_or_else(|| (0, parsed_line.raw.clone(), None))
        };
        let mut wrapped = Vec::<ProcessedVisualLine>::new();

        if should_split_on_double_space(state, &parsed_line.kind) {
            for (raw_start_column, raw_segment) in double_space_segments(&rendered_raw) {
                push_wrapped_visual_lines(
                    &mut wrapped,
                    source_line,
                    indent_width,
                    uppercase,
                    raw_column_base.saturating_add(raw_start_column),
                    &raw_segment,
                    wrap_columns,
                );
            }
        } else {
            push_wrapped_visual_lines(
                &mut wrapped,
                source_line,
                indent_width,
                uppercase,
                raw_column_base,
                &rendered_raw,
                wrap_columns,
            );
        }

        if let Some(checked) = checklist_state {
            if let Some(first_wrapped) = wrapped.first_mut() {
                first_wrapped.markdown_checklist_checked = Some(checked);
            }
        }

        for visual_line in wrapped {
            if lines_in_page >= lines_per_page {
                push_page_spacers(&mut paged_lines, source_line, spacer_lines);
                lines_in_page = 0;
            }

            paged_lines.push(visual_line);
            lines_in_page = lines_in_page.saturating_add(1);
        }
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

fn push_wrapped_visual_lines(
    out: &mut Vec<ProcessedVisualLine>,
    source_line: usize,
    indent_width: usize,
    uppercase: bool,
    raw_start_column: usize,
    raw_segment: &str,
    wrap_columns: usize,
) {
    let chars = raw_segment.chars().collect::<Vec<_>>();
    let max_content_columns = wrap_columns.saturating_sub(indent_width).max(1);

    if chars.is_empty() {
        // Keep an actual glyph cell on empty lines so their line box stays stable under zoom.
        let blank_columns = indent_width.max(1);
        out.push(ProcessedVisualLine {
            source_line,
            text: " ".repeat(blank_columns),
            display_indent_width: indent_width,
            raw_start_column,
            raw_end_column: raw_start_column,
            markdown_checklist_checked: None,
            is_spacer: false,
        });
        return;
    }

    let mut start = 0usize;
    while start < chars.len() {
        let max_end = (start + max_content_columns).min(chars.len());
        let mut split = max_end;

        if max_end < chars.len() {
            if let Some(space_index) = (start + 1..max_end).rev().find(|&idx| chars[idx] == ' ') {
                split = space_index;
            }
        }

        if split <= start {
            split = max_end;
        }

        let chunk = chars[start..split].iter().collect::<String>();
        let display_chunk = if uppercase {
            chunk.to_uppercase()
        } else {
            chunk
        };
        out.push(ProcessedVisualLine {
            source_line,
            text: format!("{}{}", " ".repeat(indent_width), display_chunk),
            display_indent_width: indent_width,
            raw_start_column: raw_start_column.saturating_add(start),
            raw_end_column: raw_start_column.saturating_add(split),
            markdown_checklist_checked: None,
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
            display_indent_width: 0,
            raw_start_column: 0,
            raw_end_column: 0,
            markdown_checklist_checked: None,
            is_spacer: true,
        });
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

fn markdown_visual_text(parsed_line: &ParsedLine) -> Option<(usize, String, Option<bool>)> {
    match parsed_line.kind {
        LineKind::MarkdownHeading => {
            let (consumed, rendered) = markdown_heading_visual(&parsed_line.raw);
            Some((consumed, rendered, None))
        }
        LineKind::MarkdownListItem => Some(markdown_list_item_visual(&parsed_line.raw)),
        LineKind::MarkdownQuote => {
            let (consumed, rendered) = markdown_quote_visual(&parsed_line.raw);
            Some((consumed, rendered, None))
        }
        LineKind::MarkdownRule => Some((0, "────────────────────────".to_string(), None)),
        LineKind::MarkdownCodeFence => Some((0, "```".to_string(), None)),
        _ => None,
    }
}

fn markdown_heading_visual(raw: &str) -> (usize, String) {
    let leading = leading_markdown_whitespace(raw);
    let trimmed = raw.chars().skip(leading).collect::<Vec<_>>();
    let mut hashes = 0usize;
    while trimmed.get(hashes).is_some_and(|ch| *ch == '#') {
        hashes += 1;
    }

    let mut consumed = hashes;
    if trimmed.get(consumed).is_some_and(|ch| *ch == ' ') {
        consumed += 1;
    }

    let text = trimmed[consumed..].iter().collect::<String>();
    (leading.saturating_add(consumed), text)
}

fn markdown_quote_visual(raw: &str) -> (usize, String) {
    let leading = leading_markdown_whitespace(raw);
    let trimmed = raw.chars().skip(leading).collect::<Vec<_>>();
    let mut consumed = 0usize;
    while trimmed.get(consumed).is_some_and(|ch| *ch == '>') {
        consumed += 1;
        if trimmed.get(consumed).is_some_and(|ch| *ch == ' ') {
            consumed += 1;
        }
    }
    let text = trimmed[consumed..].iter().collect::<String>();
    (leading.saturating_add(consumed), text)
}

fn markdown_list_item_visual(raw: &str) -> (usize, String, Option<bool>) {
    let leading = leading_markdown_whitespace(raw);
    let trimmed = raw.chars().skip(leading).collect::<Vec<_>>();
    if trimmed.is_empty() {
        return (leading, String::new(), None);
    }

    if let Some(marker_end) = unordered_list_content_start(&trimmed) {
        if let Some((consumed, checked, content_start)) =
            markdown_checklist_marker(&trimmed, marker_end)
        {
            let text = trimmed[content_start..].iter().collect::<String>();
            return (leading.saturating_add(consumed), text, Some(checked));
        }

        let text = trimmed[marker_end..].iter().collect::<String>();
        return (leading.saturating_add(marker_end), format!("• {text}"), None);
    }

    if let Some((prefix, content_start)) = ordered_list_content_start(&trimmed) {
        if let Some((consumed, checked, checklist_content_start)) =
            markdown_checklist_marker(&trimmed, content_start)
        {
            let text = trimmed[checklist_content_start..].iter().collect::<String>();
            return (
                leading.saturating_add(consumed),
                format!("{prefix} {text}"),
                Some(checked),
            );
        }

        let text = trimmed[content_start..].iter().collect::<String>();
        return (
            leading.saturating_add(content_start),
            format!("{prefix} {text}"),
            None,
        );
    }

    (0, raw.to_string(), None)
}

fn markdown_checklist_marker(
    chars: &[char],
    start: usize,
) -> Option<(usize, bool, usize)> {
    let checked_char = *chars.get(start + 1)?;
    let checked = matches!(checked_char, 'x' | 'X');
    if chars.get(start).is_some_and(|ch| *ch == '[')
        && matches!(checked_char, 'x' | 'X' | ' ')
        && chars.get(start + 2).is_some_and(|ch| *ch == ']')
    {
        let mut content_start = start + 3;
        if chars.get(content_start).is_some_and(|ch| *ch == ' ') {
            content_start += 1;
        }
        return Some((content_start, checked, content_start));
    }

    None
}

fn leading_markdown_whitespace(raw: &str) -> usize {
    raw.chars()
        .take_while(|ch| matches!(*ch, ' ' | '\t'))
        .count()
}

fn unordered_list_content_start(chars: &[char]) -> Option<usize> {
    if chars.is_empty() || !matches!(chars[0], '-' | '*' | '+') {
        return None;
    }

    let mut index = 1usize;
    let mut saw_whitespace = false;
    while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
        saw_whitespace = true;
        index += 1;
    }

    saw_whitespace.then_some(index)
}

fn ordered_list_content_start(chars: &[char]) -> Option<(String, usize)> {
    let mut digits = 0usize;
    while chars.get(digits).is_some_and(|ch| ch.is_ascii_digit()) {
        digits += 1;
    }
    if digits == 0 || chars.get(digits) != Some(&'.') {
        return None;
    }

    let mut content_start = digits + 1;
    let mut saw_whitespace = false;
    while chars
        .get(content_start)
        .is_some_and(|ch| ch.is_whitespace())
    {
        saw_whitespace = true;
        content_start += 1;
    }
    if !saw_whitespace {
        return None;
    }

    let prefix = chars[..=digits].iter().collect::<String>();
    Some((prefix, content_start))
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

fn double_space_segments(input: &str) -> Vec<(usize, String)> {
    let chars = input.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return vec![(0, String::new())];
    }

    let mut segments = Vec::<(usize, String)>::new();
    let mut start = 0usize;
    let mut index = 0usize;

    while index + 1 < chars.len() {
        if chars[index] == ' ' && chars[index + 1] == ' ' {
            let segment = chars[start..index].iter().collect::<String>();
            segments.push((start, segment));
            index += 2;
            start = index;
            continue;
        }

        index += 1;
    }

    let tail = chars[start..].iter().collect::<String>();
    segments.push((start, tail));

    segments
}

fn processed_raw_column_from_display(
    visual_line: &ProcessedVisualLine,
    display_column: usize,
) -> usize {
    let segment_len = visual_line
        .raw_end_column
        .saturating_sub(visual_line.raw_start_column);
    let local_column = display_column
        .saturating_sub(visual_line.display_indent_width)
        .min(segment_len);
    visual_line.raw_start_column.saturating_add(local_column)
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
        let segment_len = visual_line
            .raw_end_column
            .saturating_sub(visual_line.raw_start_column);
        let local_column = raw_column
            .saturating_sub(visual_line.raw_start_column)
            .min(segment_len);

        if raw_column <= visual_line.raw_end_column
            || next_start.is_some_and(|start| raw_column < start)
            || entry_index + 1 == relevant.len()
        {
            return Some((
                *visual_index,
                visual_line.display_indent_width.saturating_add(local_column),
                &visual_line.text,
            ));
        }
    }

    let default_len = default_line
        .raw_end_column
        .saturating_sub(default_line.raw_start_column);
    Some((
        default_index,
        default_line.display_indent_width.saturating_add(default_len),
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
            **text_span = if line_offset + 1 < lines_per_page {
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

        let mut line_text = visual_line.text.clone();
        if line_offset + 1 < lines_per_page {
            line_text.push('\n');
        }

        **text_span = line_text;

        text_font.font_size = font_size;
        *text_line_height = LineHeight::Px(line_height);
        if visual_line.is_spacer {
            text_font.font =
                font_for_variant_with_format(fonts, FontVariant::Regular, state.document_format);
            text_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            continue;
        }

        let Some(parsed_line) = state.parsed.get(visual_line.source_line) else {
            text_font.font =
                font_for_variant_with_format(fonts, FontVariant::Regular, state.document_format);
            text_color.0 = COLOR_ACTION;
            continue;
        };

        let raw_current_line_mode_active =
            state.display_mode == DisplayMode::ProcessedRawCurrentLine
                && visual_line.source_line == state.cursor.position.line;
        if raw_current_line_mode_active {
            text_font.font =
                font_for_variant_with_format(fonts, FontVariant::Regular, state.document_format);
            text_font.font_size = font_size;
            *text_line_height = LineHeight::Px(line_height);
            text_color.0 = COLOR_ACTION;
            continue;
        }

        let (font_variant, color, font_scale, line_height_scale) =
            style_for_line_kind(&parsed_line.kind);
        text_font.font = font_for_variant_with_format(fonts, font_variant, state.document_format);
        text_font.font_size = font_size * font_scale;
        *text_line_height = LineHeight::Px(line_height * line_height_scale);
        text_color.0 = color;
    }
}

fn style_for_line_kind(kind: &LineKind) -> (FontVariant, Color, f32, f32) {
    match kind {
        LineKind::SceneHeading => (FontVariant::Bold, COLOR_SCENE, 1.0, 1.0),
        LineKind::Action => (FontVariant::Regular, COLOR_ACTION, 1.0, 1.0),
        LineKind::Character => (FontVariant::Bold, COLOR_CHARACTER, 1.0, 1.0),
        LineKind::Dialogue => (FontVariant::Regular, COLOR_DIALOGUE, 1.0, 1.0),
        LineKind::Parenthetical => (FontVariant::Italic, COLOR_PARENTHETICAL, 1.0, 1.0),
        LineKind::Transition => (FontVariant::BoldItalic, COLOR_TRANSITION, 1.0, 1.0),
        // Markdown headings render visibly stronger than body content.
        LineKind::MarkdownHeading => (FontVariant::Bold, COLOR_MARKDOWN_HEADING, 1.35, 1.25),
        LineKind::MarkdownListItem => (FontVariant::Regular, COLOR_MARKDOWN_LIST, 1.0, 1.0),
        LineKind::MarkdownQuote => (FontVariant::Italic, COLOR_MARKDOWN_QUOTE, 1.0, 1.0),
        LineKind::MarkdownCodeFence => (FontVariant::Bold, COLOR_MARKDOWN_CODE, 1.0, 1.0),
        LineKind::MarkdownCode => (FontVariant::Regular, COLOR_MARKDOWN_CODE, 1.0, 1.0),
        LineKind::MarkdownRule => (FontVariant::Bold, COLOR_MARKDOWN_RULE, 1.0, 1.0),
        LineKind::MarkdownParagraph => (FontVariant::Regular, COLOR_ACTION, 1.0, 1.0),
        LineKind::Empty => (FontVariant::Regular, COLOR_ACTION, 1.0, 1.0),
    }
}

fn font_for_variant_with_format(
    fonts: &EditorFonts,
    variant: FontVariant,
    format: DocumentFormat,
) -> Handle<Font> {
    match format {
        DocumentFormat::Markdown => match variant {
            FontVariant::Regular => fonts.markdown_regular.clone(),
            FontVariant::Bold => fonts.markdown_bold.clone(),
            FontVariant::Italic => fonts.markdown_italic.clone(),
            FontVariant::BoldItalic => fonts.markdown_bold_italic.clone(),
        },
        DocumentFormat::Fountain => match variant {
            FontVariant::Regular => fonts.regular.clone(),
            FontVariant::Bold => fonts.bold.clone(),
            FontVariant::Italic => fonts.italic.clone(),
            FontVariant::BoldItalic => fonts.bold_italic.clone(),
        },
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
