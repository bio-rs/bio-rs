#!/usr/bin/env python3
"""Render the committed Python binding benchmark report."""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

RESULT_PATH = Path("benchmarks/python_bindings.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    print(render_report(result), end="")
    return 0


def render_report(result: dict) -> str:
    env = result["environment"]
    generated_at = datetime.fromisoformat(result["generated_at_utc"])
    input_ = result["input"]
    lines = [
        "# Python binding benchmark",
        "",
        "This benchmark is a release regression guard for Python binding overhead on",
        "deterministic synthetic FASTA input. It is not a public throughput claim.",
        "",
        "## Environment",
        "",
        f"- Date: {generated_at.date().isoformat()} (UTC)",
        f"- OS: {env['os']}",
        f"- Machine: `{env['machine']}`",
        f"- Python: `{env['python']}`",
        f"- Module: `{env['biors_module']}`",
        f"- Git commit: `{env['git_commit']}`",
        f"- Benchmark schema: `{result['schema_version']}`",
        "",
        "## Input",
        "",
        f"- Records: {input_['records']:,}",
        f"- Record length: {input_['record_length']:,}",
        f"- Total residues: {input_['total_residues']:,}",
        f"- FASTA bytes: {input_['fasta_bytes']:,}",
        f"- FASTA SHA256: `{input_['sha256']}`",
        "",
        "## Results",
        "",
        "| Workload | Mean | Median | Min | Max | Output hash |",
        "| --- | ---: | ---: | ---: | ---: | --- |",
    ]
    for workload in result["workloads"]:
        summary = workload["summary"]
        lines.append(
            "| "
            f"`{workload['name']}` | "
            f"{summary['mean_s']:.4f}s | "
            f"{summary['median_s']:.4f}s | "
            f"{summary['min_s']:.4f}s | "
            f"{summary['max_s']:.4f}s | "
            f"`{workload['output_hash']}` |"
        )
    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "PYTHONPATH=packages/rust/biors-python/python python3 scripts/benchmark_python_bindings.py",
            "cat benchmarks/python_bindings.json",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


if __name__ == "__main__":
    raise SystemExit(main())
