#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

for checker in \
  scripts/benchmarks/check-cli-benchmark-artifact.py \
  scripts/benchmarks/check-python-benchmark-artifact.py \
  scripts/benchmarks/check-wasm-benchmark-artifact.py \
  scripts/benchmarks/check-backend-benchmark-artifact.py \
  scripts/benchmarks/check-mcp-benchmark-artifact.py
do
  python3 "$checker"
done
