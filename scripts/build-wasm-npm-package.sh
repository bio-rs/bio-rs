#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="$ROOT_DIR/packages/rust/biors-wasm"
PKG_DIR="$CRATE_DIR/pkg"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required. Install with: cargo install wasm-pack --locked" >&2
  exit 1
fi

wasm-pack build "$CRATE_DIR" --target bundler --out-dir pkg --scope bio-rs

cp "$CRATE_DIR/README.md" "$PKG_DIR/README.md"
cp "$CRATE_DIR/index.d.ts" "$PKG_DIR/index.d.ts"

node - "$CRATE_DIR/package.json" "$PKG_DIR/package.json" <<'NODE'
const fs = require("fs");
const [sourcePath, generatedPath] = process.argv.slice(2);
const source = JSON.parse(fs.readFileSync(sourcePath, "utf8"));
const generated = JSON.parse(fs.readFileSync(generatedPath, "utf8"));

const merged = {
  ...generated,
  name: source.name,
  version: source.version,
  description: source.description,
  license: source.license,
  repository: source.repository,
  files: source.files,
  module: source.module,
  types: source.types,
  sideEffects: source.sideEffects,
  keywords: source.keywords,
};

fs.writeFileSync(generatedPath, `${JSON.stringify(merged, null, 2)}\n`);
NODE

npm pack "$PKG_DIR" --dry-run
