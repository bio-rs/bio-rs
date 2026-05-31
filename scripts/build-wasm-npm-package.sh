#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="$ROOT_DIR/packages/rust/biors-wasm"
PKG_DIR="$CRATE_DIR/pkg"
TEMP_LICENSE_APACHE=0
TEMP_LICENSE_MIT=0

cleanup_temp_licenses() {
  if [ "$TEMP_LICENSE_APACHE" -eq 1 ]; then
    rm -f "$CRATE_DIR/LICENSE-APACHE"
  fi
  if [ "$TEMP_LICENSE_MIT" -eq 1 ]; then
    rm -f "$CRATE_DIR/LICENSE-MIT"
  fi
}
trap cleanup_temp_licenses EXIT

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required. Install with: cargo install wasm-pack --locked" >&2
  exit 1
fi

for license_name in LICENSE-APACHE LICENSE-MIT; do
  if [ ! -e "$CRATE_DIR/$license_name" ]; then
    cp "$ROOT_DIR/$license_name" "$CRATE_DIR/$license_name"
    case "$license_name" in
      LICENSE-APACHE) TEMP_LICENSE_APACHE=1 ;;
      LICENSE-MIT) TEMP_LICENSE_MIT=1 ;;
    esac
  fi
done

wasm-pack build "$CRATE_DIR" --target bundler --out-dir pkg --scope bio-rs

cp "$CRATE_DIR/README.md" "$PKG_DIR/README.md"
cp "$CRATE_DIR/index.d.ts" "$PKG_DIR/index.d.ts"
cp "$ROOT_DIR/LICENSE-APACHE" "$PKG_DIR/LICENSE-APACHE"
cp "$ROOT_DIR/LICENSE-MIT" "$PKG_DIR/LICENSE-MIT"

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

pack_output="$(npm pack "$PKG_DIR" --dry-run --json)"
printf '%s\n' "$pack_output"

node - "$pack_output" <<'NODE'
const pack = JSON.parse(process.argv[2]);
const files = new Set(pack[0].files.map((file) => file.path));
const required = ["LICENSE-APACHE", "LICENSE-MIT"];
const missing = required.filter((file) => !files.has(file));

if (missing.length > 0) {
  console.error(`npm package dry-run is missing required file(s): ${missing.join(", ")}`);
  process.exit(1);
}
NODE
