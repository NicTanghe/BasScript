#!/usr/bin/env bash
set -euo pipefail

APP_NAME="BassScript"
APP_VERSION="@VERSION@"
INSTALL_DIR="/opt/basscript"
COMMAND_LINK="/usr/local/bin/basscript"
DESKTOP_FILE="/usr/local/share/applications/basscript.desktop"
PAYLOAD_MARKER="__BASSCRIPT_PAYLOAD_BELOW__"
TMP_DIR=""

die() {
    echo "error: $*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

require_root() {
    if [ "$(id -u)" -ne 0 ]; then
        die "run this installer with sudo: sudo $0"
    fi
}

cleanup() {
    if [ -n "${TMP_DIR:-}" ]; then
        rm -rf "$TMP_DIR"
    fi
}

resolve_install_owner() {
    if [ -n "${BASSCRIPT_USER:-}" ]; then
        INSTALL_USER="$BASSCRIPT_USER"
    elif [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != "root" ]; then
        INSTALL_USER="$SUDO_USER"
    else
        die "run with sudo from the target user account, or set BASSCRIPT_USER=<user>"
    fi

    [ "$INSTALL_USER" != "root" ] || die "refusing to chown $INSTALL_DIR to root; set BASSCRIPT_USER=<user>"
    id "$INSTALL_USER" >/dev/null 2>&1 || die "install user does not exist: $INSTALL_USER"
    INSTALL_GROUP="$(id -gn "$INSTALL_USER")"
}

extract_payload() {
    local installer_path="$1"
    local output_dir="$2"
    local archive_line

    archive_line="$(awk "/^${PAYLOAD_MARKER}\$/ { print NR + 1; exit }" "$installer_path")"
    [ -n "$archive_line" ] || die "payload marker not found"

    tail -n +"$archive_line" "$installer_path" | tar -xzf - -C "$output_dir"
    [ -d "$output_dir/payload" ] || die "payload archive did not contain payload/"
}

copy_tree_overwrite() {
    local payload_dir="$1"
    local name="$2"

    [ -d "$payload_dir/$name" ] || return 0
    install -d -m 0755 "$INSTALL_DIR/$name"
    cp -a "$payload_dir/$name/." "$INSTALL_DIR/$name/"
}

copy_tree_missing() {
    local payload_dir="$1"
    local name="$2"

    [ -d "$payload_dir/$name" ] || return 0
    install -d -m 0755 "$INSTALL_DIR/$name"
    cp -a -n "$payload_dir/$name/." "$INSTALL_DIR/$name/"
}

install_payload() {
    local payload_dir="$1"

    [ -x "$payload_dir/app/basscript-app" ] || die "payload is missing app/basscript-app"
    [ -x "$payload_dir/basscript" ] || die "payload is missing basscript launcher"

    install -d -m 0755 "$INSTALL_DIR" "$INSTALL_DIR/app"
    install -m 0755 "$payload_dir/app/basscript-app" "$INSTALL_DIR/app/basscript-app"
    install -m 0755 "$payload_dir/basscript" "$INSTALL_DIR/basscript"

    copy_tree_overwrite "$payload_dir" "assets"
    copy_tree_overwrite "$payload_dir" "fonts"
    copy_tree_overwrite "$payload_dir" "docs"
    copy_tree_missing "$payload_dir" "settings"
    copy_tree_missing "$payload_dir" "scripts"

    if [ -f "$payload_dir/LICENSE" ]; then
        install -m 0644 "$payload_dir/LICENSE" "$INSTALL_DIR/LICENSE"
    fi

    chown -R "$INSTALL_USER:$INSTALL_GROUP" "$INSTALL_DIR"
}

install_command_link() {
    local link_dir
    link_dir="$(dirname "$COMMAND_LINK")"
    install -d -m 0755 "$link_dir"

    if [ -L "$COMMAND_LINK" ] || [ ! -e "$COMMAND_LINK" ]; then
        ln -sfn "$INSTALL_DIR/basscript" "$COMMAND_LINK"
    else
        echo "Skipped command link because $COMMAND_LINK already exists and is not a symlink."
        echo "Run with: $INSTALL_DIR/basscript"
    fi
}

install_desktop_entry() {
    local desktop_dir
    desktop_dir="$(dirname "$DESKTOP_FILE")"
    install -d -m 0755 "$desktop_dir"

    cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=BassScript
Comment=Script writing editor
Exec=/usr/bin/env BASSCRIPT_WINDOW_TRANSPARENT=0 $COMMAND_LINK
Path=$INSTALL_DIR
Icon=$INSTALL_DIR/assets/icons/logo/logo2.svg
Terminal=false
Categories=Office;TextEditor;Development;
StartupWMClass=basscript
StartupNotify=false
EOF

    chmod 0644 "$DESKTOP_FILE"

    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$desktop_dir" >/dev/null 2>&1 || true
    fi
}

main() {
    require_command awk
    require_command cat
    require_command chmod
    require_command chown
    require_command cp
    require_command dirname
    require_command id
    require_command install
    require_command ln
    require_command mktemp
    require_command rm
    require_command tail
    require_command tar

    require_root
    resolve_install_owner

    TMP_DIR="$(mktemp -d)"
    trap cleanup EXIT

    extract_payload "$0" "$TMP_DIR"
    install_payload "$TMP_DIR/payload"
    install_command_link
    install_desktop_entry

    echo "$APP_NAME $APP_VERSION installed to $INSTALL_DIR"
    echo "Owner: $INSTALL_USER:$INSTALL_GROUP"
    echo "Run: basscript"
    echo "Desktop entry: $DESKTOP_FILE"
}

main "$@"
exit 0
__BASSCRIPT_PAYLOAD_BELOW__
