#!/bin/sh
set -eu

BIN="${BIORS_BIN:-biors}"
DATASET="${BIORS_DEMO_DATASET:-examples/launch-demo.fasta}"

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
$BIN model-input --max-length 32 "$DATASET"

echo "==> verify portable package fixture"
$BIN package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
