#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DOC="$ROOT/docs/researcher-workflows.md"
WORKDIR="${TMPDIR:-/tmp}/biors-researcher-workflows.$$"
BIORS_BIN="${BIORS_BIN:-$ROOT/target/debug/biors}"

RECIPES="validate-fasta-fastq
validate-sequence-kinds
protein-model-ready-workflow
invalid-workflow-recovery
molecule-structure-validation
package-validate-verify-bridge
local-report-json-output
mcp-agent-sequence"

cleanup() {
  rm -rf "$WORKDIR"
}
trap cleanup EXIT INT TERM

fail() {
  echo "error: $*" >&2
  exit 1
}

ensure_binary() {
  if [ ! -x "$BIORS_BIN" ]; then
    (cd "$ROOT" && cargo build -p biors)
  fi
}

biors() {
  "$BIORS_BIN" "$@"
}

assert_file_contains() {
  file=$1
  pattern=$2
  grep -Eq "$pattern" "$file" || fail "$file missing pattern: $pattern"
}

list_recipes() {
  printf '%s\n' "$RECIPES"
}

check_docs_inventory() {
  [ -f "$DOC" ] || fail "missing $DOC"
  for recipe in $RECIPES; do
    grep -q "### $recipe" "$DOC" || fail "docs missing recipe: $recipe"
  done
}

run_happy() {
  ensure_binary
  mkdir -p "$WORKDIR"
  biors fasta validate "$ROOT/testdata/researcher-workflows/protein.fasta" > "$WORKDIR/fasta.json"
  assert_file_contains "$WORKDIR/fasta.json" '"records"[[:space:]]*:[[:space:]]*1'
  biors formats validate --format fastq "$ROOT/testdata/researcher-workflows/reads.fastq" > "$WORKDIR/fastq.json"
  assert_file_contains "$WORKDIR/fastq.json" '"format"[[:space:]]*:[[:space:]]*"fastq"'
  biors seq validate --kind protein "$ROOT/testdata/researcher-workflows/protein.fasta" > "$WORKDIR/protein.json"
  biors seq validate --kind dna "$ROOT/testdata/researcher-workflows/dna.fasta" > "$WORKDIR/dna.json"
  biors seq validate --kind rna "$ROOT/testdata/researcher-workflows/rna.fasta" > "$WORKDIR/rna.json"
  biors tokenize --profile protein-20 "$ROOT/testdata/researcher-workflows/protein.fasta" > "$WORKDIR/tokens.json"
  biors model-input --max-length 16 "$ROOT/testdata/researcher-workflows/protein.fasta" > "$WORKDIR/model-input.json"
  assert_file_contains "$WORKDIR/model-input.json" '"records"'
  biors workflow --max-length 16 "$ROOT/testdata/researcher-workflows/protein.fasta" > "$WORKDIR/workflow.json"
  assert_file_contains "$WORKDIR/workflow.json" '"workflow"'
  biors molecule validate --format smiles "$ROOT/testdata/researcher-workflows/molecule.smi" > "$WORKDIR/molecule.json"
  assert_file_contains "$WORKDIR/molecule.json" '"format"[[:space:]]*:[[:space:]]*"smiles"'
  biors structure validate --format pdb "$ROOT/testdata/researcher-workflows/structure.pdb" > "$WORKDIR/structure.json"
  assert_file_contains "$WORKDIR/structure.json" '"format"[[:space:]]*:[[:space:]]*"pdb"'
  biors report generate "$WORKDIR/workflow.json" --output "$WORKDIR/workflow-report.md" --shareable-json "$WORKDIR/workflow-report.json"
  [ -s "$WORKDIR/workflow-report.md" ] || fail "missing workflow report markdown"
  [ -s "$WORKDIR/workflow-report.json" ] || fail "missing workflow report JSON"
  grep -q 'fn validate' "$ROOT/crates/biors-mcp-server/src/server.rs" || fail "missing MCP validate tool"
  grep -q 'fn workflow' "$ROOT/crates/biors-mcp-server/src/server.rs" || fail "missing MCP workflow tool"
  grep -q 'fn package_validate' "$ROOT/crates/biors-mcp-server/src/server.rs" || fail "missing MCP package_validate tool"
}

run_failure() {
  ensure_binary
  mkdir -p "$WORKDIR"
  if biors --json workflow --max-length 16 "$ROOT/testdata/researcher-workflows/invalid.fasta" > "$WORKDIR/invalid.json" 2> "$WORKDIR/invalid.err"; then
    fail "invalid workflow unexpectedly succeeded"
  fi
  assert_file_contains "$WORKDIR/invalid.json" '"error"'
  assert_file_contains "$WORKDIR/invalid.json" 'fasta\.missing_header'
  assert_file_contains "$WORKDIR/invalid.json" '"recovery_hint"'
}

run_package() {
  ensure_binary
  mkdir -p "$WORKDIR"
  biors package inspect "$ROOT/testdata/protein-package/manifest.json" > "$WORKDIR/package-inspect.json"
  biors package validate "$ROOT/testdata/protein-package/manifest.json" > "$WORKDIR/package-validate.json"
  biors package verify "$ROOT/testdata/protein-package/manifest.json" "$ROOT/testdata/protein-package/observations.json" > "$WORKDIR/package-verify.json"
  biors package bridge "$ROOT/testdata/protein-package/manifest.json" > "$WORKDIR/package-bridge.json"
  assert_file_contains "$WORKDIR/package-validate.json" '"valid"[[:space:]]*:[[:space:]]*true'
  assert_file_contains "$WORKDIR/package-verify.json" '"passed"[[:space:]]*:[[:space:]]*1'
  assert_file_contains "$WORKDIR/package-verify.json" '"failed"[[:space:]]*:[[:space:]]*0'
  assert_file_contains "$WORKDIR/package-bridge.json" '"ready"[[:space:]]*:[[:space:]]*true'
  assert_file_contains "$WORKDIR/package-bridge.json" '"contract_ready"[[:space:]]*:[[:space:]]*true'
}

check_local_only() {
  check_docs_inventory
  if grep -En 'curl|https?://|cargo install|pip install|npm install|npm publish|cargo publish|twine' "$DOC"; then
    fail "researcher workflows must stay local-only"
  fi
}

run_all() {
  check_docs_inventory
  run_happy
  run_failure
  run_package
  check_local_only
}

case "${1:---all}" in
  --list) list_recipes ;;
  --happy) check_docs_inventory; run_happy ;;
  --failure) check_docs_inventory; run_failure ;;
  --package) check_docs_inventory; run_package ;;
  --check-local-only) check_local_only ;;
  --all) run_all ;;
  *) fail "usage: $0 [--list|--happy|--failure|--package|--check-local-only|--all]" ;;
esac
