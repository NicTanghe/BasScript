pub(crate) fn consume_script_link_click(
    state: &mut EditorState,
    mouse_selection: &mut MouseSelectionState,
    keys: &ButtonInput<KeyCode>,
    is_start: bool,
    position: Position,
    rendered_external_target: Option<&str>,
    allow_raw_external_hit: bool,
) -> bool {
    if !is_start
        || shift_modifier_pressed(keys)
        || !state.open_link_at(position, rendered_external_target, allow_raw_external_hit)
    {
        return false;
    }

    mouse_selection.active = false;
    mouse_selection.extend_from_existing = false;
    mouse_selection.dragged = false;
    true
}
#[allow(unused_imports)]
use super::*;
