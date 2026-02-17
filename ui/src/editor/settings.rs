fn load_persistent_settings() -> PersistentSettings {
    let path = PathBuf::from(SETTINGS_PATH);
    let defaults = PersistentSettings::default();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            info!(
                "[settings] No settings file found at {}; using defaults",
                path.display()
            );
            return PersistentSettings::default();
        }
        Err(error) => {
            warn!(
                "[settings] Failed reading {}: {}; using defaults",
                path.display(),
                error
            );
            return PersistentSettings::default();
        }
    };

    let dialogue_value = parse_toml_bool(&contents, "dialogue_double_space_newline")
        .or_else(|| parse_toml_bool(&contents, "parenthetical_double_space_newline"))
        .unwrap_or(defaults.dialogue_double_space_newline);
    let non_dialogue_value = parse_toml_bool(&contents, "non_dialogue_double_space_newline")
        .unwrap_or(defaults.non_dialogue_double_space_newline);
    let page_margin_left =
        parse_toml_f32(&contents, "page_margin_left").unwrap_or(defaults.page_margin_left);
    let page_margin_right =
        parse_toml_f32(&contents, "page_margin_right").unwrap_or(defaults.page_margin_right);
    let page_margin_top =
        parse_toml_f32(&contents, "page_margin_top").unwrap_or(defaults.page_margin_top);
    let page_margin_bottom =
        parse_toml_f32(&contents, "page_margin_bottom").unwrap_or(defaults.page_margin_bottom);

    info!("[settings] Loaded settings from {}", path.display());
    PersistentSettings {
        dialogue_double_space_newline: dialogue_value,
        non_dialogue_double_space_newline: non_dialogue_value,
        page_margin_left,
        page_margin_right,
        page_margin_top,
        page_margin_bottom,
    }
}

fn save_persistent_settings(settings: &PersistentSettings) -> io::Result<()> {
    let path = PathBuf::from(SETTINGS_PATH);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = format!(
        "# BasScript settings\n\
         # true: processed pane renders dialogue double spaces as new lines\n\
         dialogue_double_space_newline = {}\n\
         # true: processed pane renders non-dialogue double spaces as new lines\n\
         non_dialogue_double_space_newline = {}\n\
         # processed page margins in typographic points\n\
         page_margin_left = {}\n\
         page_margin_right = {}\n\
         page_margin_top = {}\n\
         page_margin_bottom = {}\n",
        settings.dialogue_double_space_newline,
        settings.non_dialogue_double_space_newline,
        settings.page_margin_left,
        settings.page_margin_right,
        settings.page_margin_top,
        settings.page_margin_bottom,
    );

    fs::write(&path, contents)?;
    info!("[settings] Saved settings to {}", path.display());
    Ok(())
}

fn parse_toml_bool(contents: &str, key: &str) -> Option<bool> {
    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((lhs, rhs)) = line.split_once('=') else {
            continue;
        };
        if lhs.trim() != key {
            continue;
        }

        return match rhs.trim() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        };
    }

    None
}

fn parse_toml_f32(contents: &str, key: &str) -> Option<f32> {
    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((lhs, rhs)) = line.split_once('=') else {
            continue;
        };
        if lhs.trim() != key {
            continue;
        }

        return rhs.trim().parse::<f32>().ok();
    }

    None
}

fn persistent_settings_from_state(state: &EditorState) -> PersistentSettings {
    PersistentSettings {
        dialogue_double_space_newline: state.dialogue_double_space_newline,
        non_dialogue_double_space_newline: state.non_dialogue_double_space_newline,
        page_margin_left: state.page_margin_left,
        page_margin_right: state.page_margin_right,
        page_margin_top: state.page_margin_top,
        page_margin_bottom: state.page_margin_bottom,
    }
}

fn normalize_page_margins(state: &mut EditorState) {
    state.page_margin_left = state.page_margin_left.max(0.0);
    state.page_margin_right = state.page_margin_right.max(0.0);
    state.page_margin_top = state.page_margin_top.max(0.0);
    state.page_margin_bottom = state.page_margin_bottom.max(0.0);

    let max_horizontal = (A4_WIDTH_POINTS - MIN_TEXT_BOX_WIDTH).max(0.0);
    if state.page_margin_left + state.page_margin_right > max_horizontal {
        let overflow = state.page_margin_left + state.page_margin_right - max_horizontal;
        state.page_margin_right = (state.page_margin_right - overflow).max(0.0);
    }

    let max_vertical = (A4_HEIGHT_POINTS - MIN_TEXT_BOX_HEIGHT).max(0.0);
    if state.page_margin_top + state.page_margin_bottom > max_vertical {
        let overflow = state.page_margin_top + state.page_margin_bottom - max_vertical;
        state.page_margin_bottom = (state.page_margin_bottom - overflow).max(0.0);
    }
}

fn adjust_page_margin(state: &mut EditorState, edge: MarginEdge, delta: f32) {
    match edge {
        MarginEdge::Left => {
            let max_left =
                (A4_WIDTH_POINTS - MIN_TEXT_BOX_WIDTH - state.page_margin_right).max(0.0);
            state.page_margin_left = (state.page_margin_left + delta).clamp(0.0, max_left);
        }
        MarginEdge::Right => {
            let max_right =
                (A4_WIDTH_POINTS - MIN_TEXT_BOX_WIDTH - state.page_margin_left).max(0.0);
            state.page_margin_right = (state.page_margin_right + delta).clamp(0.0, max_right);
        }
        MarginEdge::Top => {
            let max_top =
                (A4_HEIGHT_POINTS - MIN_TEXT_BOX_HEIGHT - state.page_margin_bottom).max(0.0);
            state.page_margin_top = (state.page_margin_top + delta).clamp(0.0, max_top);
        }
        MarginEdge::Bottom => {
            let max_bottom =
                (A4_HEIGHT_POINTS - MIN_TEXT_BOX_HEIGHT - state.page_margin_top).max(0.0);
            state.page_margin_bottom = (state.page_margin_bottom + delta).clamp(0.0, max_bottom);
        }
    }

    normalize_page_margins(state);
}

fn scaled_font_size(state: &EditorState) -> f32 {
    FONT_SIZE * state.zoom
}

fn scaled_line_height(state: &EditorState) -> f32 {
    LINE_HEIGHT * state.zoom
}

fn scaled_char_width(state: &EditorState) -> f32 {
    DEFAULT_CHAR_WIDTH * state.zoom
}

fn scaled_text_padding_x(state: &EditorState) -> f32 {
    TEXT_PADDING_X * state.zoom
}

fn scaled_text_padding_y(state: &EditorState) -> f32 {
    TEXT_PADDING_Y * state.zoom
}

fn plain_horizontal_scroll_max(state: &EditorState, plain_panel_size: Option<Vec2>) -> f32 {
    let Some(panel_size) = plain_panel_size else {
        return 0.0;
    };

    let char_width = scaled_char_width(state).max(1.0);
    let max_line_chars = state
        .document
        .lines()
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as f32;
    let content_width =
        scaled_text_padding_x(state) + max_line_chars * char_width + scaled_text_padding_x(state);
    (content_width - panel_size.x).max(0.0)
}

fn processed_horizontal_scroll_bounds(
    state: &EditorState,
    processed_panel_size: Option<Vec2>,
) -> (f32, f32) {
    let Some(panel_size) = processed_panel_size else {
        return (0.0, 0.0);
    };

    let geometry = processed_page_geometry(panel_size, state);
    let base_left = geometry.paper_left;
    let base_right = geometry.paper_left + geometry.paper_width;
    let overflow_left = (-base_left).max(0.0);
    let overflow_right = (base_right - panel_size.x).max(0.0);
    (-overflow_left, overflow_right)
}

