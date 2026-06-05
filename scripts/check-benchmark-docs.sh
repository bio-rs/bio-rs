#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

for checker in \
  scripts/benchmarks/check-cli-benchmark-artifact.py \
  scripts/benchmarks/check-python-benchmark-artifact.py \
  scripts/benchmarks/check-wasm-benchmark-artifact.py \
  scripts/benchmarks/check-backend-benchmark-artifact.py \
  scripts/benchmarks/check-mcp-benchmark-artifact.py
do
  python3 "$checker"
done

while read -r renderer expected
do
  python3 "$renderer" >"$tmp_file"
  diff -u "$expected" "$tmp_file"
done <<'REPORTS'
scripts/benchmarks/render_cli_benchmark_report.py benchmarks/cli_surfaces.md
scripts/benchmarks/render_python_benchmark_report.py benchmarks/python_bindings.md
scripts/benchmarks/render_wasm_benchmark_report.py benchmarks/wasm_bindings.md
scripts/benchmarks/render_backend_benchmark_report.py benchmarks/backend_smoke.md
scripts/benchmarks/render_mcp_benchmark_report.py benchmarks/mcp_server.md
REPORTS
