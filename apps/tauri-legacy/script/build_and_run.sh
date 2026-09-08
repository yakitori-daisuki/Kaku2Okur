#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-run}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Kaku2Okur"
APP_BUNDLE="$ROOT_DIR/src-tauri/target/debug/bundle/macos/$APP_NAME.app"

cd "$ROOT_DIR"
pkill -x "$APP_NAME" >/dev/null 2>&1 || true

case "$MODE" in
  run)
    npm run tauri dev
    ;;
  --debug|debug)
    RUST_BACKTRACE=1 npm run tauri dev
    ;;
  --logs|logs)
    npm run tauri dev &
    /usr/bin/log stream --info --style compact --predicate "process == \"$APP_NAME\""
    ;;
  --telemetry|telemetry)
    npm run tauri dev &
    /usr/bin/log stream --info --style compact --predicate 'subsystem == "app.kaku2okur.desktop"'
    ;;
  --verify|verify)
    npm run tauri build -- --debug
    /usr/bin/open -n "$APP_BUNDLE"
    sleep 1
    pgrep -x "$APP_NAME" >/dev/null
    ;;
  *)
    echo "usage: $0 [run|--debug|--logs|--telemetry|--verify]" >&2
    exit 2
    ;;
esac

