#!/usr/bin/env bash
# Run after ./script/github.sh auth login; never changes global Git config.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[[ $# -eq 1 && "$1" != -* ]] || { echo "usage: $0 EXPECTED_GITHUB_LOGIN" >&2; exit 2; }
cd "$ROOT_DIR"
git rev-parse --git-dir >/dev/null
LOGIN="$("$ROOT_DIR/script/github.sh" api user --jq .login)"
[[ "$LOGIN" == "$1" ]] || { echo "error: authenticated as $LOGIN, expected $1; no Git configuration changed" >&2; exit 1; }
USER_ID="$("$ROOT_DIR/script/github.sh" api user --jq .id)"
[[ "$USER_ID" =~ ^[0-9]+$ ]] || { echo "error: invalid GitHub user ID" >&2; exit 1; }
git config --local user.name "$LOGIN"
git config --local user.email "$USER_ID+$LOGIN@users.noreply.github.com"
git config --local credential.https://github.com.username "$LOGIN"
git config --local --replace-all credential.https://github.com.helper ""
git config --local --add credential.https://github.com.helper '!f() { root_dir=$(git rev-parse --show-toplevel) || exit; "$root_dir/script/github.sh" auth git-credential "$@"; }; f'
echo "Configured this checkout for $LOGIN (GitHub noreply email; no global configuration changes)."
