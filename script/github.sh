#!/usr/bin/env bash
# Keep GitHub CLI account selection local to this checkout.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v gh >/dev/null || { echo "error: install GitHub CLI from https://cli.github.com" >&2; exit 1; }
umask 077
export GH_CONFIG_DIR="$ROOT_DIR/.github-local/gh"
export GIT_CONFIG_GLOBAL="$ROOT_DIR/.github-local/gitconfig"
mkdir -p "$GH_CONFIG_DIR"
chmod 700 "$ROOT_DIR/.github-local" "$GH_CONFIG_DIR"
# Environment tokens would override the account selected in the local config.
unset GH_TOKEN GITHUB_TOKEN GH_ENTERPRISE_TOKEN GITHUB_ENTERPRISE_TOKEN
cd "$ROOT_DIR"
exec gh "$@"
