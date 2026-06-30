#!/usr/bin/env sh
set -eu

INSTALL_DIR="/opt/basscript"
APP_DIR="$INSTALL_DIR/app"

unset CARGO_MANIFEST_DIR
export BEVY_ASSET_ROOT="$APP_DIR"

USER_NAME="$(id -un 2>/dev/null || true)"
USER_ID="$(id -u 2>/dev/null || true)"

if [ -z "${SHELL:-}" ] && [ -n "$USER_NAME" ] && command -v getent >/dev/null 2>&1; then
    PASSWD_ENTRY="$(getent passwd "$USER_NAME" 2>/dev/null || true)"
    USER_SHELL="${PASSWD_ENTRY##*:}"
    if [ -n "$USER_SHELL" ]; then
        export SHELL="$USER_SHELL"
    fi
fi

if [ -z "${XDG_RUNTIME_DIR:-}" ] && [ -n "$USER_ID" ] && [ -d "/run/user/$USER_ID" ]; then
    export XDG_RUNTIME_DIR="/run/user/$USER_ID"
fi

if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ] \
    && [ -n "${XDG_RUNTIME_DIR:-}" ] \
    && [ -S "$XDG_RUNTIME_DIR/bus" ]; then
    export DBUS_SESSION_BUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/bus"
fi

if [ -z "${XAUTHORITY:-}" ] && [ -n "${HOME:-}" ] && [ -f "$HOME/.Xauthority" ]; then
    export XAUTHORITY="$HOME/.Xauthority"
fi

SCALE_SOURCE="environment"
if [ -n "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ] && [ -z "${WINIT_X11_SCALE_FACTOR:-}" ]; then
    SCALE_SOURCE="default"
    LOGIN_SHELL_SCALE=""
    if [ -n "$USER_NAME" ] && command -v getent >/dev/null 2>&1; then
        PASSWD_ENTRY="$(getent passwd "$USER_NAME" 2>/dev/null || true)"
        USER_SHELL="${PASSWD_ENTRY##*:}"
        if [ -n "$USER_SHELL" ] && [ -x "$USER_SHELL" ]; then
            LOGIN_SHELL_SCALE="$("$USER_SHELL" -lc 'printf "%s" "$WINIT_X11_SCALE_FACTOR"' 2>/dev/null || true)"
        fi
    fi

    case "$LOGIN_SHELL_SCALE" in
        ""|*[!0123456789.]*)
            export WINIT_X11_SCALE_FACTOR="${BASSCRIPT_DEFAULT_X11_SCALE_FACTOR:-1.2}"
            SCALE_SOURCE="fallback"
            ;;
        *)
            export WINIT_X11_SCALE_FACTOR="$LOGIN_SHELL_SCALE"
            SCALE_SOURCE="login-shell"
            ;;
    esac
fi

if [ ! -t 2 ] || [ "${BASSCRIPT_FORCE_LOG:-}" = "1" ]; then
    LOG_DIR="${BASSCRIPT_LOG_DIR:-${XDG_STATE_HOME:-${HOME:-$INSTALL_DIR/.state}/.local/state}/basscript}"
    LOG_FILE="${BASSCRIPT_LOG:-$LOG_DIR/basscript.log}"

    mkdir -p "$LOG_DIR" 2>/dev/null || true
    if [ -d "$LOG_DIR" ]; then
        {
            printf '\n=== BassScript launch ===\n'
            date -Is 2>/dev/null || date
            printf 'launcher=%s\n' "$0"
            printf 'user=%s\n' "$(id -un 2>/dev/null || printf unknown)"
            printf 'cwd_before=%s\n' "$(pwd)"
            printf 'HOME=%s\n' "${HOME:-}"
            printf 'SHELL=%s\n' "${SHELL:-}"
            printf 'PATH=%s\n' "${PATH:-}"
            printf 'LANG=%s\n' "${LANG:-}"
            printf 'LC_ALL=%s\n' "${LC_ALL:-}"
            printf 'XDG_STATE_HOME=%s\n' "${XDG_STATE_HOME:-}"
            printf 'XDG_RUNTIME_DIR=%s\n' "${XDG_RUNTIME_DIR:-}"
            printf 'XDG_SESSION_TYPE=%s\n' "${XDG_SESSION_TYPE:-}"
            printf 'XDG_CURRENT_DESKTOP=%s\n' "${XDG_CURRENT_DESKTOP:-}"
            printf 'XDG_SESSION_DESKTOP=%s\n' "${XDG_SESSION_DESKTOP:-}"
            printf 'WAYLAND_DISPLAY=%s\n' "${WAYLAND_DISPLAY:-}"
            printf 'DISPLAY=%s\n' "${DISPLAY:-}"
            printf 'XAUTHORITY=%s\n' "${XAUTHORITY:-}"
            printf 'DBUS_SESSION_BUS_ADDRESS=%s\n' "${DBUS_SESSION_BUS_ADDRESS:-}"
            printf 'DESKTOP_STARTUP_ID=%s\n' "${DESKTOP_STARTUP_ID:-}"
            printf 'BASSCRIPT_WINDOW_TRANSPARENT=%s\n' "${BASSCRIPT_WINDOW_TRANSPARENT:-}"
            printf 'BASSCRIPT_CAPTURE_APP_LOG=%s\n' "${BASSCRIPT_CAPTURE_APP_LOG:-}"
            printf 'GDK_BACKEND=%s\n' "${GDK_BACKEND:-}"
            printf 'WINIT_UNIX_BACKEND=%s\n' "${WINIT_UNIX_BACKEND:-}"
            printf 'WINIT_X11_SCALE_FACTOR=%s\n' "${WINIT_X11_SCALE_FACTOR:-}"
            printf 'WINIT_X11_SCALE_FACTOR_SOURCE=%s\n' "$SCALE_SOURCE"
            printf 'WGPU_BACKEND=%s\n' "${WGPU_BACKEND:-}"
            printf 'BEVY_ASSET_ROOT=%s\n' "$BEVY_ASSET_ROOT"
            printf 'RUST_BACKTRACE=%s\n' "${RUST_BACKTRACE:-}"
            printf 'RUST_LOG=%s\n' "${RUST_LOG:-}"
            printf 'binary=%s\n' "$APP_DIR/basscript-app"
            if command -v ps >/dev/null 2>&1; then
                ps -o pid,ppid,comm,args -p "$$" -p "${PPID:-}" 2>/dev/null || true
            fi
        } >> "$LOG_FILE" 2>&1 || true
    fi
fi

cd "$INSTALL_DIR"
if [ -n "${LOG_FILE:-}" ] && [ -d "${LOG_DIR:-}" ]; then
    printf 'cwd_after=%s\n' "$(pwd)" >> "$LOG_FILE" 2>&1 || true
fi

if [ "${BASSCRIPT_CAPTURE_APP_LOG:-}" = "1" ] && [ -n "${LOG_FILE:-}" ] && [ -d "${LOG_DIR:-}" ]; then
    exec >> "$LOG_FILE" 2>&1
fi

exec "$APP_DIR/basscript-app" "$@"
