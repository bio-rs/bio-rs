#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

. scripts/release-tool-versions.env

printf 'maturin=%s\n' "$BIORS_RELEASE_MATURIN_VERSION"
printf 'wasm-pack=%s\n' "$BIORS_RELEASE_WASM_PACK_VERSION"
printf 'node=%s\n' "$BIORS_RELEASE_NODE_VERSION"
