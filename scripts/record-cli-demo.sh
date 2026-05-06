#!/bin/sh
set -eu

BIN="${BIORS_BIN:-biors}"
DATASET="${BIORS_DEMO_DATASET:-examples/launch-demo.fasta}"

if [ "${1:-}" = "--cargo" ]; then
  BIN="cargo run -q -p biors --"
fi

run_step() {
  title="$1"
  shift

  printf '\n## %s\n' "$title"
  printf '$ %s %s\n' "$BIN" "$*"
  # shellcheck disable=SC2086
  $BIN "$@"
}

printf '# bio-rs CLI demo\n'
printf '# Input: %s\n' "$DATASET"

run_step "Confirm the exact binary" --version
run_step "Check local launch readiness" doctor
run_step "Validate mixed biological FASTA" seq validate "$DATASET"
run_step "Tokenize FASTA into stable protein IDs" tokenize "$DATASET"
run_step "Build model-ready JSON records" model-input --max-length 32 "$DATASET"
run_step "Verify a portable package fixture" package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
