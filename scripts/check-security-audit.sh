#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

if ! command -v cargo-deny >/dev/null 2>&1; then
  echo "cargo-deny is required for the release security audit." >&2
  echo "Install with: cargo install --locked cargo-deny" >&2
  exit 1
fi

cargo deny check advisories bans licenses sources
