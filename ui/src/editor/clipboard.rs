pub(crate) fn copy_shortcut_just_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    shortcut_just_pressed(keys, ShortcutBinding::platform(KeyCode::KeyC, false))
}

pub(crate) fn cut_shortcut_just_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    shortcut_just_pressed(keys, ShortcutBinding::platform(KeyCode::KeyX, false))
}

pub(crate) fn paste_shortcut_just_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    shortcut_just_pressed(keys, ShortcutBinding::platform(KeyCode::KeyV, false))
}

pub(crate) fn clipboard_shortcut_matches(keys: &ButtonInput<KeyCode>, key_code: KeyCode) -> bool {
    [KeyCode::KeyC, KeyCode::KeyX, KeyCode::KeyV]
        .into_iter()
        .any(|key| {
            key_combination_matches_binding(keys, key_code, ShortcutBinding::platform(key, false))
        })
}

pub(crate) fn clipboard_input_matches(
    keys: &ButtonInput<KeyCode>,
    input: &KeyboardInput,
    key: KeyCode,
) -> bool {
    input.state.is_pressed()
        && key_combination_matches_binding(
            keys,
            input.key_code,
            ShortcutBinding::platform(key, false),
        )
}

pub(crate) fn read_system_clipboard_text() -> Option<String> {
    platform_clipboard_text().and_then(normalize_clipboard_text)
}

pub(crate) fn write_system_clipboard_text(text: &str) -> bool {
    platform_set_clipboard_text(text)
}

pub(crate) fn normalize_clipboard_text(text: String) -> Option<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let filtered: String = normalized
        .chars()
        .filter(|&ch| ch == '\n' || ch == '\t' || is_printable_char(ch))
        .collect();

    (!filtered.is_empty()).then_some(filtered)
}

impl VimRegister {
    pub(crate) fn text(&self) -> &str {
        match self {
            Self::Characterwise(text) | Self::Linewise(text) => text,
        }
    }
}

pub(crate) fn set_vim_register_and_clipboard(state: &mut EditorState, register: VimRegister) {
    let clipboard_text = register.text().to_string();
    state.vim_register = Some(register);
    write_system_clipboard_text(&clipboard_text);
}

pub(crate) fn current_vim_register(state: &mut EditorState) -> Option<VimRegister> {
    let Some(clipboard_text) = read_system_clipboard_text() else {
        return state.vim_register.clone();
    };

    let register = state
        .vim_register
        .as_ref()
        .filter(|register| register.text() == clipboard_text)
        .cloned()
        .unwrap_or(VimRegister::Characterwise(clipboard_text));
    state.vim_register = Some(register.clone());
    Some(register)
}

#[cfg(any(target_os = "linux", windows))]
thread_local! {
    static ARBOARD_CLIPBOARD: std::cell::RefCell<Option<arboard::Clipboard>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(any(target_os = "linux", windows))]
pub(crate) fn with_arboard_clipboard<R>(
    mut operation: impl FnMut(&mut arboard::Clipboard) -> Result<R, arboard::Error>,
) -> Option<R> {
    ARBOARD_CLIPBOARD.with(|slot| {
        let mut clipboard = slot.borrow_mut();
        if clipboard.is_none() {
            *clipboard = arboard::Clipboard::new().ok();
        }

        if let Some(result) = clipboard
            .as_mut()
            .and_then(|clipboard| operation(clipboard).ok())
        {
            return Some(result);
        }

        *clipboard = arboard::Clipboard::new().ok();
        clipboard
            .as_mut()
            .and_then(|clipboard| operation(clipboard).ok())
    })
}

#[cfg(any(target_os = "linux", windows))]
pub(crate) fn platform_clipboard_text() -> Option<String> {
    with_arboard_clipboard(|clipboard| clipboard.get_text())
}

#[cfg(any(target_os = "linux", windows))]
pub(crate) fn platform_set_clipboard_text(text: &str) -> bool {
    with_arboard_clipboard(|clipboard| clipboard.set_text(text.to_owned())).is_some()
}

#[cfg(not(any(target_os = "linux", windows)))]
pub(crate) fn platform_clipboard_text() -> Option<String> {
    None
}

#[cfg(not(any(target_os = "linux", windows)))]
pub(crate) fn platform_set_clipboard_text(_text: &str) -> bool {
    false
}

pub(crate) fn copy_document_selection_to_clipboard(state: &mut EditorState) -> bool {
    let Some((start, end)) = state.selection_bounds() else {
        state.status_message = "Nothing selected.".to_string();
        return false;
    };

    let text = document_text_range(&state.document, start, end);
    if text.is_empty() {
        state.status_message = "Nothing selected.".to_string();
        return false;
    }

    if !write_system_clipboard_text(&text) {
        state.status_message = "Clipboard is unavailable.".to_string();
        return false;
    }

    state.vim_register = Some(VimRegister::Characterwise(text));
    state.status_message = "Copied selection.".to_string();
    true
}

pub(crate) fn cut_document_selection_to_clipboard(state: &mut EditorState) -> Option<usize> {
    let Some((start, end)) = state.selection_bounds() else {
        state.status_message = "Nothing selected.".to_string();
        return None;
    };

    let text = document_text_range(&state.document, start, end);
    if text.is_empty() {
        state.status_message = "Nothing selected.".to_string();
        return None;
    }

    if !write_system_clipboard_text(&text) {
        state.status_message = "Clipboard is unavailable.".to_string();
        return None;
    }

    let snapshot = state.history_snapshot();
    let next = state.document.delete_range(start, end);
    state.set_cursor(next, true);
    state.vim_register = Some(VimRegister::Characterwise(text));
    state.push_undo_snapshot(snapshot);
    state.status_message = "Cut selection.".to_string();

    Some(start.line)
}

pub(crate) fn handle_document_clipboard_key_shortcut(
    keys: &ButtonInput<KeyCode>,
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    visible_lines: usize,
) -> bool {
    if copy_shortcut_just_pressed(keys) {
        copy_document_selection_to_clipboard(state);
        return true;
    }

    if cut_shortcut_just_pressed(keys) {
        if let Some(dirty_line) = cut_document_selection_to_clipboard(state) {
            state.reparse_with_dirty_hint(dirty_line);
            apply_cursor_follow_scroll_policy(state, processed_panel_size, visible_lines);
        }
        return true;
    }

    if paste_shortcut_just_pressed(keys) {
        if let Some(dirty_line) = paste_clipboard_into_document(state) {
            state.reparse_with_dirty_hint(dirty_line);
            apply_cursor_follow_scroll_policy(state, processed_panel_size, visible_lines);
        }
        return true;
    }

    false
}

pub(crate) fn handle_document_clipboard_input_shortcut(
    keys: &ButtonInput<KeyCode>,
    input: &KeyboardInput,
    state: &mut EditorState,
    processed_panel_size: Option<Vec2>,
    visible_lines: usize,
) -> bool {
    if clipboard_input_matches(keys, input, KeyCode::KeyC) {
        copy_document_selection_to_clipboard(state);
        return true;
    }

    if clipboard_input_matches(keys, input, KeyCode::KeyX) {
        if let Some(dirty_line) = cut_document_selection_to_clipboard(state) {
            state.reparse_with_dirty_hint(dirty_line);
            apply_cursor_follow_scroll_policy(state, processed_panel_size, visible_lines);
        }
        return true;
    }

    if clipboard_input_matches(keys, input, KeyCode::KeyV) {
        if let Some(dirty_line) = paste_clipboard_into_document(state) {
            state.reparse_with_dirty_hint(dirty_line);
            apply_cursor_follow_scroll_policy(state, processed_panel_size, visible_lines);
        }
        return true;
    }

    false
}

pub(crate) fn paste_clipboard_into_document(state: &mut EditorState) -> Option<usize> {
    let Some(text) = read_system_clipboard_text() else {
        state.status_message = "Clipboard is empty or unavailable.".to_string();
        return None;
    };

    let snapshot = state.history_snapshot();
    let cursor_pos = state.cursor.position;
    let mut dirty_line = cursor_pos.line;
    if let Some(next) = state.delete_selection() {
        dirty_line = dirty_line.min(next.line);
    }

    let next = state.document.insert_text(state.cursor.position, &text);
    state.set_cursor(next, true);
    state.vim_register = Some(VimRegister::Characterwise(text));
    state.push_undo_snapshot(snapshot);
    state.status_message = "Pasted clipboard.".to_string();

    Some(dirty_line)
}

pub(crate) fn copy_canvas_text_selection_to_clipboard(
    state: &mut EditorState,
    document: &Document,
) -> bool {
    let Some((start, end)) = state.canvas_text_selection_bounds(document) else {
        state.status_message = "Nothing selected.".to_string();
        return false;
    };

    let text = document_text_range(document, start, end);
    if text.is_empty() {
        state.status_message = "Nothing selected.".to_string();
        return false;
    }

    if !write_system_clipboard_text(&text) {
        state.status_message = "Clipboard is unavailable.".to_string();
        return false;
    }

    state.vim_register = Some(VimRegister::Characterwise(text));
    state.status_message = "Copied canvas text.".to_string();
    true
}

pub(crate) fn cut_canvas_text_selection_to_clipboard(
    state: &mut EditorState,
    document: &mut Document,
) -> bool {
    let Some((start, end)) = state.canvas_text_selection_bounds(document) else {
        state.status_message = "Nothing selected.".to_string();
        return false;
    };

    let text = document_text_range(document, start, end);
    if text.is_empty() {
        state.status_message = "Nothing selected.".to_string();
        return false;
    }

    if !write_system_clipboard_text(&text) {
        state.status_message = "Clipboard is unavailable.".to_string();
        return false;
    }

    let next = document.delete_range(start, end);
    state.canvas_text_cursor.set_position(next);
    state.canvas_text_selection_anchor = None;
    state.vim_register = Some(VimRegister::Characterwise(text));
    true
}

#[cfg(test)]
mod clipboard_tests {
    use super::*;

    #[test]
    fn normalizes_windows_newlines_for_document_storage() {
        assert_eq!(
            normalize_clipboard_text("a\r\nb\rc".to_string()).as_deref(),
            Some("a\nb\nc")
        );
    }

    #[test]
    fn paste_shortcut_uses_platform_modifier() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ControlLeft);
        keys.press(KeyCode::KeyV);

        assert!(paste_shortcut_just_pressed(&keys));
    }

    #[test]
    fn clipboard_shortcuts_use_platform_modifier() {
        let mut copy = ButtonInput::<KeyCode>::default();
        copy.press(KeyCode::ControlLeft);
        copy.press(KeyCode::KeyC);
        assert!(copy_shortcut_just_pressed(&copy));
        assert!(clipboard_shortcut_matches(&copy, KeyCode::KeyC));

        let mut cut = ButtonInput::<KeyCode>::default();
        cut.press(KeyCode::ControlLeft);
        cut.press(KeyCode::KeyX);
        assert!(cut_shortcut_just_pressed(&cut));
        assert!(clipboard_shortcut_matches(&cut, KeyCode::KeyX));
    }
}
#[allow(unused_imports)]
use super::*;
