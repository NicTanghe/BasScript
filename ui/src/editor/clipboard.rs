fn paste_shortcut_just_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    shortcut_just_pressed(
        keys,
        ShortcutBinding::platform(KeyCode::KeyV, false),
    )
}

fn read_system_clipboard_text() -> Option<String> {
    platform_clipboard_text().and_then(normalize_clipboard_text)
}

fn write_system_clipboard_text(text: &str) -> bool {
    platform_set_clipboard_text(text)
}

fn normalize_clipboard_text(text: String) -> Option<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let filtered: String = normalized
        .chars()
        .filter(|&ch| ch == '\n' || ch == '\t' || is_printable_char(ch))
        .collect();

    (!filtered.is_empty()).then_some(filtered)
}

impl VimRegister {
    fn text(&self) -> &str {
        match self {
            Self::Characterwise(text) | Self::Linewise(text) => text,
        }
    }
}

fn set_vim_register_and_clipboard(state: &mut EditorState, register: VimRegister) {
    let clipboard_text = register.text().to_string();
    state.vim_register = Some(register);
    write_system_clipboard_text(&clipboard_text);
}

fn current_vim_register(state: &mut EditorState) -> Option<VimRegister> {
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

#[cfg(windows)]
fn platform_clipboard_text() -> Option<String> {
    use std::slice;

    use windows_sys::Win32::System::{
        DataExchange::{
            CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        },
        Memory::{GlobalLock, GlobalUnlock},
        Ole::CF_UNICODETEXT,
    };

    struct ClipboardGuard;

    impl Drop for ClipboardGuard {
        fn drop(&mut self) {
            unsafe {
                CloseClipboard();
            }
        }
    }

    struct GlobalLockGuard(*mut core::ffi::c_void);

    impl Drop for GlobalLockGuard {
        fn drop(&mut self) {
            unsafe {
                GlobalUnlock(self.0);
            }
        }
    }

    unsafe {
        let format = u32::from(CF_UNICODETEXT);
        if IsClipboardFormatAvailable(format) == 0 {
            return None;
        }
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }
        let _clipboard_guard = ClipboardGuard;

        let handle = GetClipboardData(format);
        if handle.is_null() {
            return None;
        }

        let locked = GlobalLock(handle);
        if locked.is_null() {
            return None;
        }
        let _lock_guard = GlobalLockGuard(handle);

        let wide = locked.cast::<u16>();
        let mut len = 0;
        while *wide.add(len) != 0 {
            len += 1;
        }

        Some(String::from_utf16_lossy(slice::from_raw_parts(wide, len)))
    }
}

#[cfg(windows)]
fn platform_set_clipboard_text(text: &str) -> bool {
    use windows_sys::Win32::{
        Foundation::GlobalFree,
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
            Ole::CF_UNICODETEXT,
        },
    };

    struct ClipboardGuard;

    impl Drop for ClipboardGuard {
        fn drop(&mut self) {
            unsafe {
                CloseClipboard();
            }
        }
    }

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return false;
        }
        let _clipboard_guard = ClipboardGuard;

        let mut wide: Vec<u16> = text.replace('\n', "\r\n").encode_utf16().collect();
        wide.push(0);
        let bytes = wide.len() * std::mem::size_of::<u16>();
        let handle = GlobalAlloc(GMEM_MOVEABLE, bytes);
        if handle.is_null() {
            return false;
        }

        let locked = GlobalLock(handle);
        if locked.is_null() {
            GlobalFree(handle);
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), locked.cast::<u16>(), wide.len());
        GlobalUnlock(handle);

        if EmptyClipboard() == 0 {
            GlobalFree(handle);
            return false;
        }

        if SetClipboardData(u32::from(CF_UNICODETEXT), handle).is_null() {
            GlobalFree(handle);
            return false;
        }

        true
    }
}

#[cfg(not(windows))]
fn platform_clipboard_text() -> Option<String> {
    None
}

#[cfg(not(windows))]
fn platform_set_clipboard_text(_text: &str) -> bool {
    false
}

fn paste_clipboard_into_document(state: &mut EditorState) -> Option<usize> {
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
}
