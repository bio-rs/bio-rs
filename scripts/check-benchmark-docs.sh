#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

python3 scripts/check-benchmark-artifact.py
python3 scripts/check-cli-benchmark-artifact.py
python3 scripts/check-python-benchmark-artifact.py
python3 scripts/check-wasm-benchmark-artifact.py
python3 scripts/check-backend-benchmark-artifact.py
python3 scripts/check-mcp-benchmark-artifact.py
python3 scripts/render_benchmark_report.py >"$tmp_file"
diff -u benchmarks/fasta_vs_biopython.md "$tmp_file"
python3 scripts/render_cli_benchmark_report.py >"$tmp_file"
diff -u benchmarks/cli_surfaces.md "$tmp_file"
python3 scripts/render_python_benchmark_report.py >"$tmp_file"
diff -u benchmarks/python_bindings.md "$tmp_file"
python3 scripts/render_wasm_benchmark_report.py >"$tmp_file"
diff -u benchmarks/wasm_bindings.md "$tmp_file"
python3 scripts/render_backend_benchmark_report.py >"$tmp_file"
diff -u benchmarks/backend_smoke.md "$tmp_file"
python3 scripts/render_mcp_benchmark_report.py >"$tmp_file"
diff -u benchmarks/mcp_server.md "$tmp_file"
