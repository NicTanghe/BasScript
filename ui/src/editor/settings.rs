fn load_persistent_settings() -> PersistentSettings {
    let path = PathBuf::from(EDITOR_SETTINGS_PATH);
    let defaults = PersistentSettings::default();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if let Some(legacy) = load_legacy_persistent_settings_ron() {
                info!(
                    "[settings] Loaded legacy settings from {}; migrating to {}",
                    LEGACY_EDITOR_SETTINGS_PATH,
                    path.display()
                );
                let _ = save_persistent_settings(&legacy);
                return legacy;
            }
            if let Some(legacy) = load_legacy_toml_settings() {
                info!(
                    "[settings] Loaded legacy settings from {}; using as defaults",
                    LEGACY_SETTINGS_PATH
                );
                let _ = save_persistent_settings(&legacy);
                return legacy;
            }
            info!(
                "[settings] No settings file found at {}; using defaults",
                path.display()
            );
            let defaults = PersistentSettings::default();
            let _ = save_persistent_settings(&defaults);
            return defaults;
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

    info!("[settings] Loaded settings from {}", path.display());
    persistent_settings_from_ron(&contents, &defaults)
}

fn load_keybind_settings() -> KeybindSettings {
    let path = PathBuf::from(KEYBINDS_SETTINGS_PATH);
    let mut keybinds = KeybindSettings::default();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if let Some(legacy) = load_legacy_keybind_settings_ron() {
                info!(
                    "[keybinds] Loaded legacy keybinds from {}; migrating to {}",
                    LEGACY_KEYBINDS_SETTINGS_PATH,
                    path.display()
                );
                let _ = save_keybind_settings(&legacy);
                return legacy;
            }
            info!(
                "[keybinds] No keybind file found at {}; using defaults",
                path.display()
            );
            let _ = save_keybind_settings(&keybinds);
            return keybinds;
        }
        Err(error) => {
            warn!(
                "[keybinds] Failed reading {}: {}; using defaults",
                path.display(),
                error
            );
            return keybinds;
        }
    };

    apply_keybind_settings_from_ron(&contents, &mut keybinds);

    info!("[keybinds] Loaded keybinds from {}", path.display());
    keybinds
}

fn load_persistent_ui_state() -> PersistentUiState {
    let path = PathBuf::from(UI_STATE_PATH);
    let defaults = PersistentUiState::default();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            info!(
                "[state] No UI state file found at {}; using defaults",
                path.display()
            );
            let defaults = PersistentUiState::default();
            let _ = save_persistent_ui_state(&defaults);
            return defaults;
        }
        Err(error) => {
            warn!(
                "[state] Failed reading {}: {}; using defaults",
                path.display(),
                error
            );
            return PersistentUiState::default();
        }
    };

    info!("[state] Loaded UI state from {}", path.display());
    persistent_ui_state_from_ron(&contents, &defaults)
}

fn load_theme_settings() -> ThemeSettings {
    let path = PathBuf::from(THEME_SETTINGS_PATH);
    let defaults = ThemeSettings::default();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            info!(
                "[theme] No theme file found at {}; using defaults",
                path.display()
            );
            let _ = save_theme_settings(&defaults);
            return defaults;
        }
        Err(error) => {
            warn!(
                "[theme] Failed reading {}: {}; using defaults",
                path.display(),
                error
            );
            return defaults;
        }
    };

    let used_legacy_fields = parse_ron_vec4(&contents, "selection_background").is_none()
        && (parse_ron_f32(&contents, "selection_background_r").is_some()
            || parse_ron_f32(&contents, "selection_background_g").is_some()
            || parse_ron_f32(&contents, "selection_background_b").is_some()
            || parse_ron_f32(&contents, "selection_background_a").is_some());

    let theme = theme_settings_from_ron(&contents, &defaults);
    if used_legacy_fields {
        if let Err(error) = save_theme_settings(&theme) {
            warn!(
                "[theme] Failed migrating legacy theme format at {}: {}",
                path.display(),
                error
            );
        } else {
            info!(
                "[theme] Migrated legacy theme format to vector4 at {}",
                path.display()
            );
        }
    }

    info!("[theme] Loaded theme from {}", path.display());
    theme
}

fn save_persistent_settings(settings: &PersistentSettings) -> io::Result<()> {
    let path = PathBuf::from(EDITOR_SETTINGS_PATH);
    let workspace_root_path = settings
        .workspace_root_path
        .as_deref()
        .unwrap_or("")
        .replace('\\', "/");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = format!(
        "(\n\
         \tdialogue_double_space_newline: {},\n\
         \tnon_dialogue_double_space_newline: {},\n\
         \tshow_system_titlebar: {},\n\
         \tpage_margin_left: {:.3},\n\
         \tpage_margin_right: {:.3},\n\
         \tpage_margin_top: {:.3},\n\
         \tpage_margin_bottom: {:.3},\n\
         \tworkspace_root_path: \"{}\",\n\
         )\n",
        settings.dialogue_double_space_newline,
        settings.non_dialogue_double_space_newline,
        settings.show_system_titlebar,
        settings.page_margin_left,
        settings.page_margin_right,
        settings.page_margin_top,
        settings.page_margin_bottom,
        workspace_root_path,
    );

    fs::write(&path, contents)?;
    info!("[settings] Saved settings to {}", path.display());
    Ok(())
}

fn save_keybind_settings(keybinds: &KeybindSettings) -> io::Result<()> {
    let path = PathBuf::from(KEYBINDS_SETTINGS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut rows = Vec::new();
    for action in SHORTCUT_ACTIONS {
        rows.push(format!(
            "\t{}: \"{}\",",
            shortcut_action_settings_key(action),
            binding_spec(keybinds.binding(action))
        ));
    }

    let contents = format!("(\n{}\n)\n", rows.join("\n"));
    fs::write(&path, contents)?;
    info!("[keybinds] Saved keybinds to {}", path.display());
    Ok(())
}

fn save_persistent_ui_state(ui_state: &PersistentUiState) -> io::Result<()> {
    let path = PathBuf::from(UI_STATE_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = format!(
        "(\n\
         \tworkspace_sidebar_visible: {},\n\
         \ttop_menu_collapsed: {},\n\
         )\n",
        ui_state.workspace_sidebar_visible,
        ui_state.top_menu_collapsed
    );

    fs::write(&path, contents)?;
    info!("[state] Saved UI state to {}", path.display());
    Ok(())
}

fn save_theme_settings(theme: &ThemeSettings) -> io::Result<()> {
    let path = PathBuf::from(THEME_SETTINGS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let selection_background = theme.selection_background_clamped();
    let link_fallback = theme.link_fallback_clamped();
    let link_prop = theme.link_prop_clamped();
    let link_place = theme.link_place_clamped();
    let link_character = theme.link_character_clamped();
    let link_faction = theme.link_faction_clamped();
    let link_concept = theme.link_concept_clamped();
    let contents = format!(
        "(\n\
         \tselection_background: ({:.3}, {:.3}, {:.3}, {:.3}),\n\
         \tlink_fallback: ({:.3}, {:.3}, {:.3}, {:.3}),\n\
         \tlink_prop: ({:.3}, {:.3}, {:.3}, {:.3}),\n\
         \tlink_place: ({:.3}, {:.3}, {:.3}, {:.3}),\n\
         \tlink_character: ({:.3}, {:.3}, {:.3}, {:.3}),\n\
         \tlink_faction: ({:.3}, {:.3}, {:.3}, {:.3}),\n\
         \tlink_concept: ({:.3}, {:.3}, {:.3}, {:.3}),\n\
         )\n",
        selection_background.x,
        selection_background.y,
        selection_background.z,
        selection_background.w,
        link_fallback.x,
        link_fallback.y,
        link_fallback.z,
        link_fallback.w,
        link_prop.x,
        link_prop.y,
        link_prop.z,
        link_prop.w,
        link_place.x,
        link_place.y,
        link_place.z,
        link_place.w,
        link_character.x,
        link_character.y,
        link_character.z,
        link_character.w,
        link_faction.x,
        link_faction.y,
        link_faction.z,
        link_faction.w,
        link_concept.x,
        link_concept.y,
        link_concept.z,
        link_concept.w
    );

    fs::write(&path, contents)?;
    info!("[theme] Saved theme to {}", path.display());
    Ok(())
}

fn parse_ron_value(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with("//") || line == "(" || line == ")" {
            continue;
        }

        let Some((lhs, rhs)) = line.split_once(':') else {
            continue;
        };
        if lhs.trim() != key {
            continue;
        }

        return Some(rhs.trim().trim_end_matches(',').trim().to_string());
    }

    None
}

fn parse_ron_bool(contents: &str, key: &str) -> Option<bool> {
    match parse_ron_value(contents, key)?.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
}

fn parse_ron_f32(contents: &str, key: &str) -> Option<f32> {
    parse_ron_value(contents, key)?.parse::<f32>().ok()
}

fn parse_ron_vec4(contents: &str, key: &str) -> Option<Vec4> {
    let raw = parse_ron_value(contents, key)?;
    parse_ron_vec4_value(&raw)
}

fn parse_ron_vec4_value(raw: &str) -> Option<Vec4> {
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix("Vec4(")
        .and_then(|value| value.strip_suffix(')'))
        .or_else(|| {
            trimmed
                .strip_prefix("vec4(")
                .and_then(|value| value.strip_suffix(')'))
        })
        .unwrap_or(trimmed);

    let inner = stripped
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .or_else(|| {
            stripped
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
        })
        .unwrap_or(stripped);

    if inner.contains(':') {
        let x = parse_named_vec_component(inner, "x").or_else(|| parse_named_vec_component(inner, "r"))?;
        let y = parse_named_vec_component(inner, "y").or_else(|| parse_named_vec_component(inner, "g"))?;
        let z = parse_named_vec_component(inner, "z").or_else(|| parse_named_vec_component(inner, "b"))?;
        let w = parse_named_vec_component(inner, "w").or_else(|| parse_named_vec_component(inner, "a"))?;
        return Some(Vec4::new(x, y, z, w));
    }

    let mut values = [0.0_f32; 4];
    let mut index = 0usize;
    for token in inner.split(',').map(str::trim).filter(|token| !token.is_empty()) {
        if index >= values.len() {
            return None;
        }
        values[index] = token.parse::<f32>().ok()?;
        index += 1;
    }

    if index == 4 {
        Some(Vec4::new(values[0], values[1], values[2], values[3]))
    } else {
        None
    }
}

fn parse_named_vec_component(raw: &str, key: &str) -> Option<f32> {
    for entry in raw.split(',').map(str::trim).filter(|entry| !entry.is_empty()) {
        let (lhs, rhs) = entry.split_once(':')?;
        if lhs.trim() != key {
            continue;
        }
        return rhs.trim().parse::<f32>().ok();
    }
    None
}

fn parse_ron_string(contents: &str, key: &str) -> Option<String> {
    let value = parse_ron_value(contents, key)?;
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return Some(value[1..value.len().saturating_sub(1)].to_string());
    }
    None
}

fn persistent_settings_from_ron(contents: &str, defaults: &PersistentSettings) -> PersistentSettings {
    let dialogue_value = parse_ron_bool(contents, "dialogue_double_space_newline")
        .unwrap_or(defaults.dialogue_double_space_newline);
    let non_dialogue_value = parse_ron_bool(contents, "non_dialogue_double_space_newline")
        .unwrap_or(defaults.non_dialogue_double_space_newline);
    let show_system_titlebar =
        parse_ron_bool(contents, "show_system_titlebar").unwrap_or(defaults.show_system_titlebar);
    let page_margin_left = parse_ron_f32(contents, "page_margin_left").unwrap_or(defaults.page_margin_left);
    let page_margin_right =
        parse_ron_f32(contents, "page_margin_right").unwrap_or(defaults.page_margin_right);
    let page_margin_top = parse_ron_f32(contents, "page_margin_top").unwrap_or(defaults.page_margin_top);
    let page_margin_bottom =
        parse_ron_f32(contents, "page_margin_bottom").unwrap_or(defaults.page_margin_bottom);
    let workspace_root_path = parse_ron_string(contents, "workspace_root_path")
        .and_then(|value| if value.trim().is_empty() { None } else { Some(value) })
        .or_else(|| defaults.workspace_root_path.clone());

    PersistentSettings {
        dialogue_double_space_newline: dialogue_value,
        non_dialogue_double_space_newline: non_dialogue_value,
        show_system_titlebar,
        page_margin_left,
        page_margin_right,
        page_margin_top,
        page_margin_bottom,
        workspace_root_path,
    }
}

fn persistent_ui_state_from_ron(
    contents: &str,
    defaults: &PersistentUiState,
) -> PersistentUiState {
    let workspace_sidebar_visible = parse_ron_bool(contents, "workspace_sidebar_visible")
        .unwrap_or(defaults.workspace_sidebar_visible);
    let top_menu_collapsed =
        parse_ron_bool(contents, "top_menu_collapsed").unwrap_or(defaults.top_menu_collapsed);

    PersistentUiState {
        workspace_sidebar_visible,
        top_menu_collapsed,
    }
}

fn theme_settings_from_ron(contents: &str, defaults: &ThemeSettings) -> ThemeSettings {
    let selection_background = parse_ron_vec4(contents, "selection_background").unwrap_or_else(|| {
        Vec4::new(
            parse_ron_f32(contents, "selection_background_r")
                .unwrap_or(defaults.selection_background.x),
            parse_ron_f32(contents, "selection_background_g")
                .unwrap_or(defaults.selection_background.y),
            parse_ron_f32(contents, "selection_background_b")
                .unwrap_or(defaults.selection_background.z),
            parse_ron_f32(contents, "selection_background_a")
                .unwrap_or(defaults.selection_background.w),
        )
    });
    let legacy_processed_link = parse_ron_vec4(contents, "processed_link")
        .unwrap_or(defaults.link_fallback);
    let link_fallback = parse_ron_vec4(contents, "link_fallback").unwrap_or(legacy_processed_link);
    let link_prop = parse_ron_vec4(contents, "link_prop").unwrap_or(defaults.link_prop);
    let link_place = parse_ron_vec4(contents, "link_place").unwrap_or(defaults.link_place);
    let link_character =
        parse_ron_vec4(contents, "link_character").unwrap_or(defaults.link_character);
    let link_faction = parse_ron_vec4(contents, "link_faction").unwrap_or(defaults.link_faction);
    let link_concept = parse_ron_vec4(contents, "link_concept").unwrap_or(defaults.link_concept);

    ThemeSettings {
        selection_background: Vec4::new(
            selection_background.x.clamp(0.0, 1.0),
            selection_background.y.clamp(0.0, 1.0),
            selection_background.z.clamp(0.0, 1.0),
            selection_background.w.clamp(0.0, 1.0),
        ),
        link_fallback: clamp_vec4_rgba(link_fallback),
        link_prop: clamp_vec4_rgba(link_prop),
        link_place: clamp_vec4_rgba(link_place),
        link_character: clamp_vec4_rgba(link_character),
        link_faction: clamp_vec4_rgba(link_faction),
        link_concept: clamp_vec4_rgba(link_concept),
    }
}

fn apply_keybind_settings_from_ron(contents: &str, keybinds: &mut KeybindSettings) {
    for action in SHORTCUT_ACTIONS {
        let key = shortcut_action_settings_key(action);
        let Some(raw) = parse_ron_string(contents, key) else {
            continue;
        };
        let Some(binding) = parse_binding_spec(&raw) else {
            warn!("[keybinds] Invalid binding for {key}: {raw}");
            continue;
        };
        keybinds.set_binding(action, binding);
    }
}

fn load_legacy_persistent_settings_ron() -> Option<PersistentSettings> {
    let path = PathBuf::from(LEGACY_EDITOR_SETTINGS_PATH);
    let contents = fs::read_to_string(path).ok()?;
    let defaults = PersistentSettings::default();
    Some(persistent_settings_from_ron(&contents, &defaults))
}

fn load_legacy_keybind_settings_ron() -> Option<KeybindSettings> {
    let path = PathBuf::from(LEGACY_KEYBINDS_SETTINGS_PATH);
    let contents = fs::read_to_string(path).ok()?;
    let mut keybinds = KeybindSettings::default();
    apply_keybind_settings_from_ron(&contents, &mut keybinds);
    Some(keybinds)
}

fn load_legacy_toml_settings() -> Option<PersistentSettings> {
    let path = PathBuf::from(LEGACY_SETTINGS_PATH);
    let contents = fs::read_to_string(&path).ok()?;
    let defaults = PersistentSettings::default();
    Some(PersistentSettings {
        dialogue_double_space_newline: parse_toml_bool(&contents, "dialogue_double_space_newline")
            .or_else(|| parse_toml_bool(&contents, "parenthetical_double_space_newline"))
            .unwrap_or(defaults.dialogue_double_space_newline),
        non_dialogue_double_space_newline: parse_toml_bool(
            &contents,
            "non_dialogue_double_space_newline",
        )
        .unwrap_or(defaults.non_dialogue_double_space_newline),
        show_system_titlebar: parse_toml_bool(&contents, "show_system_titlebar")
            .unwrap_or(defaults.show_system_titlebar),
        page_margin_left: parse_toml_f32(&contents, "page_margin_left")
            .unwrap_or(defaults.page_margin_left),
        page_margin_right: parse_toml_f32(&contents, "page_margin_right")
            .unwrap_or(defaults.page_margin_right),
        page_margin_top: parse_toml_f32(&contents, "page_margin_top")
            .unwrap_or(defaults.page_margin_top),
        page_margin_bottom: parse_toml_f32(&contents, "page_margin_bottom")
            .unwrap_or(defaults.page_margin_bottom),
        workspace_root_path: None,
    })
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
        if lhs.trim() == key {
            return match rhs.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
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
        if lhs.trim() == key {
            return rhs.trim().parse::<f32>().ok();
        }
    }
    None
}

fn persistent_settings_from_state(state: &EditorState) -> PersistentSettings {
    PersistentSettings {
        dialogue_double_space_newline: state.dialogue_double_space_newline,
        non_dialogue_double_space_newline: state.non_dialogue_double_space_newline,
        show_system_titlebar: state.show_system_titlebar,
        page_margin_left: state.page_margin_left,
        page_margin_right: state.page_margin_right,
        page_margin_top: state.page_margin_top,
        page_margin_bottom: state.page_margin_bottom,
        workspace_root_path: state
            .workspace_root
            .as_ref()
            .map(|path| path.to_string_lossy().replace('\\', "/")),
    }
}

fn persistent_ui_state_from_state(state: &EditorState) -> PersistentUiState {
    PersistentUiState {
        workspace_sidebar_visible: state.workspace_sidebar_visible,
        top_menu_collapsed: state.top_menu_collapsed,
    }
}

fn theme_settings_from_state(state: &EditorState) -> ThemeSettings {
    ThemeSettings {
        selection_background: Vec4::new(
            state.selection_bg_rgba.x.clamp(0.0, 1.0),
            state.selection_bg_rgba.y.clamp(0.0, 1.0),
            state.selection_bg_rgba.z.clamp(0.0, 1.0),
            state.selection_bg_rgba.w.clamp(0.0, 1.0),
        ),
        link_fallback: clamp_vec4_rgba(state.link_fallback_rgba),
        link_prop: clamp_vec4_rgba(state.link_prop_rgba),
        link_place: clamp_vec4_rgba(state.link_place_rgba),
        link_character: clamp_vec4_rgba(state.link_character_rgba),
        link_faction: clamp_vec4_rgba(state.link_faction_rgba),
        link_concept: clamp_vec4_rgba(state.link_concept_rgba),
    }
}

fn sync_theme_colors(state: &mut EditorState) {
    state.selection_bg_rgba = Vec4::new(
        state.selection_bg_rgba.x.clamp(0.0, 1.0),
        state.selection_bg_rgba.y.clamp(0.0, 1.0),
        state.selection_bg_rgba.z.clamp(0.0, 1.0),
        state.selection_bg_rgba.w.clamp(0.0, 1.0),
    );
    state.selection_bg_color = Color::srgba(
        state.selection_bg_rgba.x,
        state.selection_bg_rgba.y,
        state.selection_bg_rgba.z,
        state.selection_bg_rgba.w,
    );
    state.link_fallback_rgba = clamp_vec4_rgba(state.link_fallback_rgba);
    state.link_fallback_color = color_from_rgba(state.link_fallback_rgba);
    state.link_prop_rgba = clamp_vec4_rgba(state.link_prop_rgba);
    state.link_prop_color = color_from_rgba(state.link_prop_rgba);
    state.link_place_rgba = clamp_vec4_rgba(state.link_place_rgba);
    state.link_place_color = color_from_rgba(state.link_place_rgba);
    state.link_character_rgba = clamp_vec4_rgba(state.link_character_rgba);
    state.link_character_color = color_from_rgba(state.link_character_rgba);
    state.link_faction_rgba = clamp_vec4_rgba(state.link_faction_rgba);
    state.link_faction_color = color_from_rgba(state.link_faction_rgba);
    state.link_concept_rgba = clamp_vec4_rgba(state.link_concept_rgba);
    state.link_concept_color = color_from_rgba(state.link_concept_rgba);
}

fn active_theme_rgba(state: &EditorState) -> Vec4 {
    theme_rgba_for_target(state, state.theme_color_target)
}

fn theme_rgba_for_target(state: &EditorState, target: ThemeColorTarget) -> Vec4 {
    match target {
        ThemeColorTarget::SelectionBackground => state.selection_bg_rgba,
        ThemeColorTarget::LinkFallback => state.link_fallback_rgba,
        ThemeColorTarget::LinkProp => state.link_prop_rgba,
        ThemeColorTarget::LinkPlace => state.link_place_rgba,
        ThemeColorTarget::LinkCharacter => state.link_character_rgba,
        ThemeColorTarget::LinkFaction => state.link_faction_rgba,
        ThemeColorTarget::LinkConcept => state.link_concept_rgba,
    }
}

fn theme_color_for_target(state: &EditorState, target: ThemeColorTarget) -> Color {
    match target {
        ThemeColorTarget::SelectionBackground => state.selection_bg_color,
        ThemeColorTarget::LinkFallback => state.link_fallback_color,
        ThemeColorTarget::LinkProp => state.link_prop_color,
        ThemeColorTarget::LinkPlace => state.link_place_color,
        ThemeColorTarget::LinkCharacter => state.link_character_color,
        ThemeColorTarget::LinkFaction => state.link_faction_color,
        ThemeColorTarget::LinkConcept => state.link_concept_color,
    }
}

fn set_active_theme_rgba(state: &mut EditorState, rgba: Vec4) {
    match state.theme_color_target {
        ThemeColorTarget::SelectionBackground => state.selection_bg_rgba = rgba,
        ThemeColorTarget::LinkFallback => state.link_fallback_rgba = rgba,
        ThemeColorTarget::LinkProp => state.link_prop_rgba = rgba,
        ThemeColorTarget::LinkPlace => state.link_place_rgba = rgba,
        ThemeColorTarget::LinkCharacter => state.link_character_rgba = rgba,
        ThemeColorTarget::LinkFaction => state.link_faction_rgba = rgba,
        ThemeColorTarget::LinkConcept => state.link_concept_rgba = rgba,
    }
    sync_theme_colors(state);
}

impl EditorState {
    fn processed_link_color_for_target(&self, target: Option<&str>) -> Color {
        let Some(target) = target else {
            return self.link_fallback_color;
        };
        let Some(entity_type) = self.script_link_target_types.get(target) else {
            return self.link_fallback_color;
        };

        self.link_color_for_type(entity_type)
    }

    fn link_color_for_type(&self, entity_type: &str) -> Color {
        match link_color_target_for_entity_type(entity_type) {
            ThemeColorTarget::LinkFallback => self.link_fallback_color,
            ThemeColorTarget::LinkProp => self.link_prop_color,
            ThemeColorTarget::LinkPlace => self.link_place_color,
            ThemeColorTarget::LinkCharacter => self.link_character_color,
            ThemeColorTarget::LinkFaction => self.link_faction_color,
            ThemeColorTarget::LinkConcept => self.link_concept_color,
            ThemeColorTarget::SelectionBackground => self.link_fallback_color,
        }
    }
}

fn link_color_target_for_entity_type(entity_type: &str) -> ThemeColorTarget {
    match normalize_entity_type_key(entity_type).as_str() {
        "" | "unknown" => ThemeColorTarget::LinkFallback,
        "prop" | "object" | "item" | "vehicle" | "costume" | "wardrobe" => {
            ThemeColorTarget::LinkProp
        }
        "location" | "place" | "setting" | "room" => ThemeColorTarget::LinkPlace,
        "character" | "person" | "cast" | "npc" => ThemeColorTarget::LinkCharacter,
        "group" | "faction" | "organization" | "org" => ThemeColorTarget::LinkFaction,
        "note" | "concept" | "theme" | "event" | "beat" | "moment" => {
            ThemeColorTarget::LinkConcept
        }
        _ => ThemeColorTarget::LinkFallback,
    }
}

fn normalize_entity_type_key(entity_type: &str) -> String {
    entity_type
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn clamp_vec4_rgba(value: Vec4) -> Vec4 {
    Vec4::new(
        value.x.clamp(0.0, 1.0),
        value.y.clamp(0.0, 1.0),
        value.z.clamp(0.0, 1.0),
        value.w.clamp(0.0, 1.0),
    )
}

fn color_from_rgba(value: Vec4) -> Color {
    Color::srgba(value.x, value.y, value.z, value.w)
}

fn rgb_to_hsv(rgb: Vec3) -> (f32, f32, f32) {
    let r = rgb.x.clamp(0.0, 1.0);
    let g = rgb.y.clamp(0.0, 1.0);
    let b = rgb.z.clamp(0.0, 1.0);

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let value = max;
    let saturation = if max <= f32::EPSILON { 0.0 } else { delta / max };

    let hue = if delta <= f32::EPSILON {
        0.0
    } else if (max - r).abs() <= f32::EPSILON {
        ((g - b) / delta).rem_euclid(6.0) / 6.0
    } else if (max - g).abs() <= f32::EPSILON {
        (((b - r) / delta) + 2.0) / 6.0
    } else {
        (((r - g) / delta) + 4.0) / 6.0
    };

    (hue.rem_euclid(1.0), saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0))
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    let h = hue.rem_euclid(1.0);
    let s = saturation.clamp(0.0, 1.0);
    let v = value.clamp(0.0, 1.0);

    if s <= f32::EPSILON {
        return Vec3::new(v, v, v);
    }

    let scaled = h * 6.0;
    let sector = scaled.floor() as i32;
    let fraction = scaled - sector as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * fraction);
    let t = v * (1.0 - s * (1.0 - fraction));

    match sector.rem_euclid(6) {
        0 => Vec3::new(v, t, p),
        1 => Vec3::new(q, v, p),
        2 => Vec3::new(p, v, t),
        3 => Vec3::new(p, q, v),
        4 => Vec3::new(t, p, v),
        _ => Vec3::new(v, p, q),
    }
}

fn save_editor_ui_state(state: &EditorState) -> io::Result<()> {
    save_persistent_ui_state(&persistent_ui_state_from_state(state))
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

fn default_char_width_for_format(format: DocumentFormat) -> f32 {
    match format {
        DocumentFormat::Markdown => DEFAULT_MARKDOWN_CHAR_WIDTH,
        DocumentFormat::Fountain => DEFAULT_CHAR_WIDTH,
    }
}

fn scaled_char_width(state: &EditorState) -> f32 {
    default_char_width_for_format(state.document_format) * state.zoom
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
    processed_horizontal_scroll_bounds_with_overscroll(state, processed_panel_size)
}
