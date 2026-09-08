#!/usr/bin/env bash
# Generated installer: this header and its embedded payload ship together.
set -euo pipefail
umask 077
usage() {
  cat <<'USAGE'
Kaku2Okur — self-contained, ad-hoc signed macOS installer (not notarized)
Usage: bash INSTALLER [--user|--system|--prefix DIRECTORY] --allow-unsigned [--no-open]
       bash INSTALLER --verify-only

--user             Install in ~/Applications (default).
--system           Install in /Applications; requires an already writable directory.
--prefix DIRECTORY Install in a custom directory; no sudo is used.
--allow-unsigned   Explicitly allow this unnotarized build and quarantine removal
                   on the newly installed app only. Trust the source before using.
--no-open          Do not launch the app after installation.
--verify-only      Verify the embedded ZIP, app signature, ID and architecture;
                   do not install, change quarantine attributes or launch anything.
Existing installations are moved into a timestamped backup directory.
USAGE
}
DEST_PARENT="$HOME/Applications"
ALLOW_UNSIGNED=false
SHOULD_OPEN=true
VERIFY_ONLY=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --user) DEST_PARENT="$HOME/Applications" ;;
    --system) DEST_PARENT=/Applications ;;
    --prefix)
      [[ $# -ge 2 && -n "$2" && "$2" != --* ]] || { echo "error: --prefix needs a directory" >&2; exit 2; }
      DEST_PARENT="$2"; shift ;;
    --allow-unsigned) ALLOW_UNSIGNED=true ;;
    --no-open) SHOULD_OPEN=false ;;
    --verify-only) VERIFY_ONLY=true ;;
    *) usage >&2; exit 2 ;;
  esac
  shift
done
[[ "$(uname -s)" == Darwin ]] || { echo "error: macOS is required" >&2; exit 1; }
[[ "$(uname -m)" == '@ARCH@' ]] || { echo "error: this installer requires @ARCH@; use the installer for your Mac's architecture" >&2; exit 1; }
OS_VERSION="$(/usr/bin/sw_vers -productVersion)"
[[ "${OS_VERSION%%.*}" -ge 14 ]] || { echo "error: macOS 14 or later is required" >&2; exit 1; }
if ! $VERIFY_ONLY && ! $ALLOW_UNSIGNED; then
  echo "error: this build is not Developer ID signed or notarized; review its source and pass --allow-unsigned to install" >&2
  exit 2
fi
WORK="$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/kaku2okur-install.XXXXXX")"
STAGE=""
cleanup() {
  /bin/rm -rf "$WORK"
  if [[ -n "$STAGE" ]]; then /bin/rm -rf "$STAGE"; fi
}
trap cleanup EXIT
trap 'exit 130' HUP INT TERM
PAYLOAD_LINE="$(/usr/bin/awk '/^__KAKU2OKUR_PAYLOAD__$/ { print NR + 1; exit }' "$0")"
[[ -n "$PAYLOAD_LINE" ]] || { echo "error: missing installer payload" >&2; exit 1; }
/usr/bin/tail -n +"$PAYLOAD_LINE" "$0" | /usr/bin/base64 -D > "$WORK/app.zip"
ACTUAL="$(/usr/bin/shasum -a 256 "$WORK/app.zip" | /usr/bin/awk '{print $1}')"
[[ "$ACTUAL" == '@PAYLOAD_SHA256@' ]] || { echo "error: embedded ZIP checksum mismatch" >&2; exit 1; }
/usr/bin/ditto -x -k "$WORK/app.zip" "$WORK/unpacked"
APP="$WORK/unpacked/Kaku2Okur.app"
[[ -d "$APP" && ! -L "$APP" ]] || { echo "error: missing app bundle" >&2; exit 1; }
[[ "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$APP/Contents/Info.plist")" == app.kaku2okur.native ]] || { echo "error: unexpected bundle ID" >&2; exit 1; }
/usr/bin/lipo "$APP/Contents/MacOS/Kaku2Okur" -verify_arch '@ARCH@'
/usr/bin/codesign --verify --deep --strict "$APP"
if $VERIFY_ONLY; then echo "Verified: ZIP checksum, app signature, bundle ID and @ARCH@ architecture"; exit 0; fi
if /usr/bin/pgrep -x Kaku2Okur >/dev/null; then
  echo "error: quit Kaku2Okur from its menu bar menu before installing" >&2
  exit 1
fi
mkdir -p "$DEST_PARENT"
[[ -w "$DEST_PARENT" ]] || { echo "error: directory is not writable; try --user" >&2; exit 1; }
DEST_PARENT="$(cd "$DEST_PARENT" && pwd)"
DEST="$DEST_PARENT/Kaku2Okur.app"
[[ ! -L "$DEST" ]] || { echo "error: destination is a symbolic link" >&2; exit 1; }
STAGE="$(/usr/bin/mktemp -d "$DEST_PARENT/.kaku2okur-stage.XXXXXX")"
/usr/bin/ditto "$APP" "$STAGE/Kaku2Okur.app"
# Only this explicitly approved, newly staged app loses quarantine.
/usr/bin/xattr -r -d com.apple.quarantine "$STAGE/Kaku2Okur.app"
/usr/bin/codesign --verify --deep --strict "$STAGE/Kaku2Okur.app"
BACKUP=""
if [[ -e "$DEST" ]]; then
  BACKUP="$(/usr/bin/mktemp -d "$DEST_PARENT/Kaku2Okur-backup-$(date +%Y%m%d-%H%M%S).XXXXXX")"
  /bin/mv "$DEST" "$BACKUP/Kaku2Okur.app"
  echo "Previous app backed up to: $BACKUP/Kaku2Okur.app"
fi
if ! /bin/mv "$STAGE/Kaku2Okur.app" "$DEST"; then
  if [[ -n "$BACKUP" ]]; then /bin/mv "$BACKUP/Kaku2Okur.app" "$DEST"; fi
  echo "error: installation failed" >&2
  exit 1
fi
echo "Installed: $DEST"
if $SHOULD_OPEN; then /usr/bin/open "$DEST" --args --show; fi
exit 0
__KAKU2OKUR_PAYLOAD__
