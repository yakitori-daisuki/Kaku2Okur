#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-run}"
usage() {
  echo "usage: $0 [run|--build|--test|--debug|--hidden|--settings|--about|--cycle|--verify|--verify-toggle|--verify-permission-recheck|--logs|--measure]"
}
case "$MODE" in
  --help|-h) usage; exit 0 ;;
  run|--build|build|--test|test|--debug|debug|--hidden|hidden|--settings|settings|--about|about|--cycle|cycle|--verify|verify|--verify-toggle|--verify-permission-recheck|--logs|logs|--measure|measure) ;;
  *) usage >&2; exit 2 ;;
esac
if [[ $# -gt 1 ]]; then usage >&2; exit 2; fi
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Kaku2Okur"
PROFILE="release"
BUILD_ARGS=(--release)

if [[ "$MODE" == "--debug" || "$MODE" == "debug" ]]; then
  PROFILE="debug"
  BUILD_ARGS=()
fi

APP_BUNDLE="$ROOT_DIR/dist/$APP_NAME.app"
EXECUTABLE="$ROOT_DIR/target/$PROFILE/$APP_NAME"

cd "$ROOT_DIR"
export CARGO_TARGET_DIR="$ROOT_DIR/target"
export MACOSX_DEPLOYMENT_TARGET=14.0
if [[ "$(uname -s)" != Darwin ]]; then
  echo "error: this script requires macOS; use script/build_windows.ps1 on Windows" >&2
  exit 1
fi
command -v cargo >/dev/null || { echo "error: install Rust 1.90 or newer from https://rustup.rs" >&2; exit 1; }
if [[ "$MODE" == --test || "$MODE" == test ]]; then
  cargo test --locked -p kaku2okur-native
  exit 0
fi
xcrun --find actool >/dev/null || { echo "error: select a full Xcode 26 or newer installation with xcode-select" >&2; exit 1; }
pkill -x "$APP_NAME" >/dev/null 2>&1 || true
cargo build --locked -p kaku2okur-native "${BUILD_ARGS[@]}"
bash "$ROOT_DIR/script/build_icon.sh"

mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
cp "$EXECUTABLE" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
cp "$ROOT_DIR/apps/native/macos/Info.plist" "$APP_BUNDLE/Contents/Info.plist"
cp "$ROOT_DIR/target/app-icon/Kaku2Okur-Envelope.icns" "$APP_BUNDLE/Contents/Resources/Kaku2Okur.icns"
cp "$ROOT_DIR/target/app-icon/Kaku2Okur-Envelope.icns" "$APP_BUNDLE/Contents/Resources/icon.icns"
cp "$ROOT_DIR/target/app-icon/Assets.car" "$APP_BUNDLE/Contents/Resources/Assets.car"
cp "$ROOT_DIR/THIRD_PARTY_NOTICES.md" "$APP_BUNDLE/Contents/Resources/THIRD_PARTY_NOTICES.md"
cp "$ROOT_DIR/apps/native/assets/icons/LICENSE" "$APP_BUNDLE/Contents/Resources/Lucide-LICENSE.txt"
/usr/bin/codesign --force --deep --sign - "$APP_BUNDLE"
/usr/bin/codesign --verify --deep --strict "$APP_BUNDLE"
touch "$APP_BUNDLE"

case "$MODE" in
  --build|build)
    echo "Built $APP_BUNDLE"
    ;;
  run|--debug|debug)
    /usr/bin/open -n "$APP_BUNDLE" --args --show
    ;;
  --hidden|hidden)
    /usr/bin/open -n "$APP_BUNDLE"
    ;;
  --settings|settings)
    /usr/bin/open -n "$APP_BUNDLE" --args --show-settings
    ;;
  --about|about)
    /usr/bin/open -n "$APP_BUNDLE" --args --show-about
    ;;
  --cycle|cycle)
    /usr/bin/open -n "$APP_BUNDLE" --args --measure-cycle
    ;;
  --verify|verify)
    /usr/bin/open -n "$APP_BUNDLE" --args --show
    sleep 1
    pgrep -x "$APP_NAME" >/dev/null
    ;;
  --verify-toggle)
    RESULT="$(mktemp -t kaku2okur-toggle)"
    trap 'rm -f "$RESULT"' EXIT
    /usr/bin/open -n -W "$APP_BUNDLE" --args --verify-toggle "$RESULT"
    cp "$RESULT" "$ROOT_DIR/target/toggle-verification.log"
    rg -q '^PASS: show/hide/show/hide and pending text preserved$' "$RESULT"
    ;;
  --verify-permission-recheck)
    RESULT="$(mktemp -t kaku2okur-permission)"
    trap 'rm -f "$RESULT"' EXIT
    /usr/bin/open -n -W "$APP_BUNDLE" --args --verify-permission-recheck "$RESULT"
    cp "$RESULT" "$ROOT_DIR/target/permission-verification.log"
    rg -q '^PASS: permission recheck busy state, result, duplicate click, and dismissal$' "$RESULT"
    ;;
  --logs|logs)
    /usr/bin/open -n "$APP_BUNDLE" --args --show
    /usr/bin/log stream --info --style compact --predicate "process == \"$APP_NAME\""
    ;;
  --measure|measure)
    /usr/bin/open -n "$APP_BUNDLE"
    sleep 2
    PID="$(pgrep -x "$APP_NAME" | head -1)"
    ps -o pid=,rss=,%cpu=,etime=,command= -p "$PID"
    du -sh "$APP_BUNDLE"
    ;;
esac
