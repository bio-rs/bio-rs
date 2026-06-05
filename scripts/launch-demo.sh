#!/bin/sh
set -eu

BIN="${BIORS_BIN:-biors}"
DATASET="${BIORS_DEMO_DATASET:-testdata/sequences/launch-demo.fasta}"
OUT_DIR="${BIORS_DEMO_OUT_DIR:-target/biors-demo}"

if [ "${1:-}" = "--cargo" ]; then
  BIN="cargo run -p biors --"
fi

echo "==> version"
$BIN --version

echo "==> doctor"
$BIN doctor

echo "==> validate mixed launch FASTA"
$BIN seq validate "$DATASET"

echo "==> tokenize launch FASTA"
$BIN tokenize "$DATASET"

echo "==> model-ready records"
mkdir -p "$OUT_DIR"
$BIN model-input --max-length 32 "$DATASET" | tee "$OUT_DIR/model-input.json"

echo "==> reproducible report export"
$BIN report generate "$OUT_DIR/model-input.json" \
  --output "$OUT_DIR/model-input-report.md" \
  --shareable-json "$OUT_DIR/model-input-report.json"

echo "==> verify portable package fixture"
$BIN package verify \
  testdata/protein-package/manifest.json \
  testdata/protein-package/observations.json
