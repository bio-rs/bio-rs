#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

python3 scripts/check-benchmark-artifact.py
python3 scripts/render_benchmark_report.py >"$tmp_file"
diff -u benchmarks/fasta_vs_biopython.md "$tmp_file"
