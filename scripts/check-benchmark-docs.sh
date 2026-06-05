#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

check_rendered_report() {
  renderer="$1"
  expected="$2"

  python3 "$renderer" >"$tmp_file"
  diff -u "$expected" "$tmp_file"
}

python3 scripts/benchmarks/check-cli-benchmark-artifact.py
python3 scripts/benchmarks/check-python-benchmark-artifact.py
python3 scripts/benchmarks/check-wasm-benchmark-artifact.py
python3 scripts/benchmarks/check-backend-benchmark-artifact.py
python3 scripts/benchmarks/check-mcp-benchmark-artifact.py
check_rendered_report scripts/benchmarks/render_cli_benchmark_report.py benchmarks/cli_surfaces.md
check_rendered_report scripts/benchmarks/render_python_benchmark_report.py benchmarks/python_bindings.md
check_rendered_report scripts/benchmarks/render_wasm_benchmark_report.py benchmarks/wasm_bindings.md
check_rendered_report scripts/benchmarks/render_backend_benchmark_report.py benchmarks/backend_smoke.md
check_rendered_report scripts/benchmarks/render_mcp_benchmark_report.py benchmarks/mcp_server.md
