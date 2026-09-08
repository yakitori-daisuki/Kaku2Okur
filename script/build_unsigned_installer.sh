#!/usr/bin/env bash
# Build a self-contained macOS installer. No signing account is required.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
case "${1:-}" in
  --help|-h) echo "usage: $0 (builds an ad-hoc signed installer for this Mac's architecture)"; exit 0 ;;
  '') ;;
  *) echo "usage: $0" >&2; exit 2 ;;
esac
[[ $# -le 1 ]] || exit 2
bash "$ROOT_DIR/script/build_and_run.sh" --build
APP="$ROOT_DIR/dist/Kaku2Okur.app"
ARCH="$(/usr/bin/uname -m)"
case "$ARCH" in arm64|x86_64) ;; *) echo "error: unsupported architecture: $ARCH" >&2; exit 1 ;; esac
/usr/bin/lipo "$APP/Contents/MacOS/Kaku2Okur" -verify_arch "$ARCH"
/usr/bin/codesign --verify --deep --strict "$APP"
WORK="$(mktemp -d "$ROOT_DIR/target/installer.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
/usr/bin/ditto -c -k --keepParent "$APP" "$WORK/app.zip"
CHECKSUM="$(/usr/bin/shasum -a 256 "$WORK/app.zip" | /usr/bin/awk '{print $1}')"
NAME="Kaku2Okur-install-unsigned-$ARCH.sh"
OUTPUT="$ROOT_DIR/dist/$NAME"
/usr/bin/sed -e "s/@PAYLOAD_SHA256@/$CHECKSUM/g" -e "s/@ARCH@/$ARCH/g" \
  "$ROOT_DIR/script/unsigned_installer.template.sh" > "$WORK/$NAME"
/usr/bin/base64 -i "$WORK/app.zip" >> "$WORK/$NAME"
chmod +x "$WORK/$NAME"
mv "$WORK/$NAME" "$OUTPUT"
(cd "$ROOT_DIR/dist" && /usr/bin/shasum -a 256 "$NAME" > "$NAME.sha256")
echo "Built $OUTPUT"
echo "Checksum: $OUTPUT.sha256"
