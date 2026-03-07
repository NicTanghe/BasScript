fn consume_script_link_click(
    state: &mut EditorState,
    mouse_selection: &mut MouseSelectionState,
    keys: &ButtonInput<KeyCode>,
    is_start: bool,
    position: Position,
) -> bool {
    if !is_start || shift_modifier_pressed(keys) || !state.open_script_link_at(position) {
        return false;
    }

    mouse_selection.active = false;
    mouse_selection.extend_from_existing = false;
    mouse_selection.dragged = false;
    true
}
