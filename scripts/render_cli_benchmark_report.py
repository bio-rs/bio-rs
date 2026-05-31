#!/usr/bin/env python3
"""Render the committed CLI surface benchmark report from JSON results."""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

RESULT_PATH = Path("benchmarks/cli_surfaces.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    print(render_report(result), end="")
    return 0


def render_report(result: dict) -> str:
    env = result["environment"]
    generated_at = datetime.fromisoformat(result["generated_at_utc"])
    lines = [
        "# CLI surface benchmark",
        "",
        "This benchmark is a release regression guard for CLI overhead on deterministic",
        "synthetic inputs and package fixtures. It is not a public throughput claim.",
        "",
        "## Environment",
        "",
        f"- Date: {generated_at.date().isoformat()} (UTC)",
        f"- OS: {env['os']}",
        f"- Machine: `{env['machine']}`",
        f"- Rust: `{env['rustc']}`",
        f"- Cargo: `{env['cargo']}`",
        f"- bio-rs CLI: `biors v{env['biors']}`",
        f"- Python: `{env['python']}`",
        f"- Git commit: `{env['git_commit']}`",
        f"- Benchmark schema: `{result['schema_version']}`",
        "",
        "## Methodology",
        "",
        f"Scope: {result['methodology']['scope']}.",
        "",
        "The script builds the release CLI binary, generates deterministic synthetic",
        "FASTA inputs in a temporary directory, reuses the committed package fixture,",
        "warms each command once, and records timed subprocess runs plus canonical JSON",
        "output hashes.",
        "",
        "## Results",
        "",
        "| Workload | Surface | Input shape | Mean | Median | Min | Max | Output size |",
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |",
    ]

    for workload in result["workloads"]:
        summary = workload["result"]["summary"]
        lines.append(
            "| "
            f"`{workload['name']}` | "
            f"`{workload['surface']}` | "
            f"{input_shape(workload['input'])} | "
            f"{summary['mean_s']:.4f}s | "
            f"{summary['median_s']:.4f}s | "
            f"{summary['min_s']:.4f}s | "
            f"{summary['max_s']:.4f}s | "
            f"{summary['stdout_bytes']:,} bytes |"
        )

    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "python3 scripts/benchmark_cli_surfaces.py",
            "cat benchmarks/cli_surfaces.json",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


def input_shape(input_: dict) -> str:
    if input_.get("kind") == "no_input":
        return "no input"
    fields = []
    if "files" in input_:
        fields.append(f"{input_['files']:,} files")
    if "records" in input_:
        fields.append(f"{input_['records']:,} records")
    if "total_residues" in input_:
        fields.append(f"{input_['total_residues']:,} residues")
    fields.append(f"{input_['file_size_bytes']:,} bytes")
    return ", ".join(fields)


if __name__ == "__main__":
    raise SystemExit(main())
