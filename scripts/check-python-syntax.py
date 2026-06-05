#!/usr/bin/env python3
"""Check Python source syntax without writing __pycache__ artifacts."""

from pathlib import Path
import sys


DEFAULT_FILES = [
    "scripts/benchmark_fasta_vs_biopython.py",
    "scripts/benchmark_feature_coverage.py",
    "scripts/benchmark_release_status.py",
    "scripts/benchmark_cli_surfaces.py",
    "scripts/benchmark_python_bindings.py",
    "scripts/compare-benchmark-artifacts.py",
    "scripts/check-benchmark-artifact.py",
    "scripts/check-cli-benchmark-artifact.py",
    "scripts/check-python-benchmark-artifact.py",
    "scripts/check-wasm-benchmark-artifact.py",
    "scripts/check-backend-benchmark-artifact.py",
    "scripts/check-mcp-benchmark-artifact.py",
    "scripts/check-registry-versions.py",
    "scripts/check-dependency-policy.py",
    "scripts/check-github-actions-pinning.py",
    "scripts/check-module-size.py",
    "scripts/check-release-artifact-contents.py",
    "scripts/check-release-workflow.py",
    "scripts/check-rust-version-policy.py",
    "scripts/check-sequence-kind-support-docs.py",
    "scripts/test-python-wheel.py",
    "scripts/write-release-checksums.py",
    "scripts/benchmark_large_file_streaming.py",
    "scripts/render_benchmark_report.py",
    "scripts/render_cli_benchmark_report.py",
    "scripts/render_python_benchmark_report.py",
    "scripts/check-python-syntax.py",
    "integrations/python/esm_from_biors_json.py",
    "integrations/python/pandas_numpy_friendly.py",
    "integrations/python/protbert_from_biors_json.py",
    "integrations/python/reference_preprocess.py",
]


def main() -> int:
    failed = False
    for file_name in sys.argv[1:] or DEFAULT_FILES:
        path = Path(file_name)
        try:
            compile(path.read_text(encoding="utf-8"), str(path), "exec")
        except SyntaxError as error:
            print(f"{path}: {error}", file=sys.stderr)
            failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
