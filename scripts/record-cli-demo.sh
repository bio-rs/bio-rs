#!/bin/sh
set -eu

BIN="${BIORS_BIN:-biors}"
DATASET="${BIORS_DEMO_DATASET:-examples/launch-demo.fasta}"
OUT_DIR="${BIORS_DEMO_OUT_DIR:-target/biors-demo}"

if [ "${1:-}" = "--cargo" ]; then
  BIN="cargo run -p biors --"
fi

mkdir -p "$OUT_DIR"

printf '$ %s --version\n' "$BIN"
$BIN --version

printf '\n$ %s doctor\n' "$BIN"
$BIN doctor >"$OUT_DIR/doctor.json"
cat "$OUT_DIR/doctor.json"

printf '\n$ %s seq validate %s\n' "$BIN" "$DATASET"
$BIN seq validate "$DATASET" >"$OUT_DIR/validate.json"
cat "$OUT_DIR/validate.json"

printf '\n$ %s model-input --max-length 32 %s\n' "$BIN" "$DATASET"
$BIN model-input --max-length 32 "$DATASET" >"$OUT_DIR/model-input.json"
cat "$OUT_DIR/model-input.json"

printf '\n$ %s package verify examples/protein-package/manifest.json examples/protein-package/observations.json\n' "$BIN"
$BIN package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json \
  >"$OUT_DIR/package-verify.json"
cat "$OUT_DIR/package-verify.json"

printf '\nDemo artifacts written to %s\n' "$OUT_DIR"
