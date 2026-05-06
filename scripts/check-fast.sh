#!/bin/sh
set -eu

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

echo "==> shell syntax"
find scripts .githooks -type f -print | while IFS= read -r file; do
  case "$file" in
    *.sh|.githooks/*) sh -n "$file" ;;
  esac
done

echo "==> python syntax"
python3 -m py_compile \
  scripts/benchmark_fasta_vs_biopython.py \
  scripts/check-benchmark-artifact.py \
  scripts/check-module-size.py \
  scripts/check-release-workflow.py \
  scripts/benchmark_large_file_streaming.py \
  scripts/render_benchmark_report.py \
  examples/python/esm_from_biors_json.py \
  examples/python/pandas_numpy_friendly.py \
  examples/python/protbert_from_biors_json.py \
  examples/python/reference_preprocess.py

echo "==> module size"
python3 scripts/check-module-size.py

echo "==> benchmark docs"
scripts/check-benchmark-docs.sh

echo "==> release workflow"
python3 scripts/check-release-workflow.py

echo "==> cargo fmt --check"
cargo fmt --all --check

echo "==> cargo check --workspace --all-targets --all-features"
cargo check --locked --workspace --all-targets --all-features
