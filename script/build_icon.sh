#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT="${1:-$ROOT_DIR/target/app-icon}"
mkdir -p "$OUTPUT"

xcrun actool "$ROOT_DIR/apps/native/assets/app-icon/Kaku2Okur-Envelope.icon" \
  --compile "$OUTPUT" \
  --platform macosx \
  --minimum-deployment-target 14.0 \
  --target-device mac \
  --app-icon Kaku2Okur-Envelope \
  --standalone-icon-behavior all \
  --output-partial-info-plist "$OUTPUT/icon-info.plist" \
  --output-format human-readable-text \
  --warnings --errors

test -s "$OUTPUT/Assets.car"
test -s "$OUTPUT/Kaku2Okur-Envelope.icns"
