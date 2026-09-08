#!/usr/bin/env bash
# Download, verify, and run the installer from the latest Kaku2Okur release.
set -euo pipefail
case "${1:-}" in
  --allow-unsigned|--verify-only) mode="$1" ;;
  *) echo "usage: $0 --allow-unsigned | --verify-only" >&2; exit 2 ;;
esac
[[ $# -eq 1 ]] || { echo "error: unexpected arguments" >&2; exit 2; }
[[ "$(uname -s)" == Darwin ]] || { echo "error: macOS is required" >&2; exit 1; }
arch="$(uname -m)"
case "$arch" in arm64|x86_64) ;; *) echo "error: unsupported architecture: $arch" >&2; exit 1 ;; esac
umask 077
base="https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download"
name="Kaku2Okur-install-unsigned-$arch.sh"
temp_root="${TMPDIR:-/tmp}"
work="$(/usr/bin/mktemp -d "${temp_root%/}/kaku2okur-download.XXXXXX")"
cleanup() { /bin/rm -rf "$work"; }
trap cleanup EXIT
trap "exit 130" HUP INT TERM
cd "$work"
fetch() {
  /usr/bin/curl --disable --fail --silent --show-error --location \
    --max-redirs 10 --proto "=https" --proto-redir "=https" \
    --connect-timeout 20 --max-time 600 --retry 3 --output "$1" "$2"
}
fetch "$name" "$base/$name"
fetch "$name.sha256" "$base/$name.sha256"
read -r expected listed extra < "$name.sha256"
[[ -z "${extra:-}" && "$listed" == "$name" && ${#expected} -eq 64 && "$expected" != *[!0-9a-f]* ]] || { echo "error: invalid checksum manifest" >&2; exit 1; }
[[ "$(/usr/bin/awk "END { print NR }" "$name.sha256")" == "1" ]] || { echo "error: checksum manifest must contain exactly one line" >&2; exit 1; }
/usr/bin/shasum -a 256 -c "$name.sha256"
if [[ -d /Applications && -w /Applications ]]; then scope="--system"; else scope="--user"; fi
/bin/bash "$name" "$scope" "$mode"
