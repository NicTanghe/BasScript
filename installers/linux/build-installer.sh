#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="$(awk -F '"' '/^version =/ { print $2; exit }' "$ROOT_DIR/app/Cargo.toml")"
TARGET_DIR="$ROOT_DIR/target/installers"
STAGE_DIR="$ROOT_DIR/target/installer-stage"
PAYLOAD_DIR="$STAGE_DIR/payload"
INSTALLER_PATH="$TARGET_DIR/basscript-${VERSION}-linux-x86_64.run"

require_command() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "error: required command not found: $1" >&2
        exit 1
    }
}

require_command awk
require_command cargo
require_command cat
require_command chmod
require_command cp
require_command find
require_command install
require_command mkdir
require_command rm
require_command sed
require_command tar

[ -n "$VERSION" ] || {
    echo "error: could not read version from app/Cargo.toml" >&2
    exit 1
}

cd "$ROOT_DIR"
cargo build --release --bin basscript-app

rm -rf "$STAGE_DIR"
mkdir -p "$PAYLOAD_DIR/app" "$TARGET_DIR"

install -m 0755 "$ROOT_DIR/target/release/basscript-app" "$PAYLOAD_DIR/app/basscript-app"
install -m 0755 "$SCRIPT_DIR/launcher.sh" "$PAYLOAD_DIR/basscript"

for dir in assets fonts settings scripts docs; do
    if [ -d "$ROOT_DIR/$dir" ]; then
        mkdir -p "$PAYLOAD_DIR/$dir"
        cp -a "$ROOT_DIR/$dir/." "$PAYLOAD_DIR/$dir/"
    fi
done

find "$PAYLOAD_DIR" -name .DS_Store -delete

if [ -f "$ROOT_DIR/LICENSE" ]; then
    install -m 0644 "$ROOT_DIR/LICENSE" "$PAYLOAD_DIR/LICENSE"
fi

sed "s/@VERSION@/$VERSION/g" "$SCRIPT_DIR/installer.sh" > "$STAGE_DIR/installer.sh"
tar -C "$STAGE_DIR" -czf "$STAGE_DIR/payload.tar.gz" payload
cat "$STAGE_DIR/installer.sh" "$STAGE_DIR/payload.tar.gz" > "$INSTALLER_PATH"
chmod 0755 "$INSTALLER_PATH"

echo "Created $INSTALLER_PATH"
