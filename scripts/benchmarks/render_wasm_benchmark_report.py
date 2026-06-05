#!/usr/bin/env python3
"""Render the committed WASM binding benchmark report from JSON results."""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

RESULT_PATH = Path("benchmarks/wasm_bindings.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    print(render_report(result), end="")
    return 0


def render_report(result: dict) -> str:
    env = result["environment"]
    generated_at = datetime.fromisoformat(result["generated_at_utc"])
    lines = [
        "# WASM binding benchmark",
        "",
        "This benchmark is a release regression guard for Node.js-loaded WASM",
        "bindings. It is not a browser or public throughput claim.",
        "",
        "## Environment",
        "",
        f"- Date: {generated_at.date().isoformat()} (UTC)",
        f"- OS: {env['os']}",
        f"- Machine: `{env['machine']}`",
        f"- Rust: `{env['rustc']}`",
        f"- Cargo: `{env['cargo']}`",
        f"- wasm-pack: `{env['wasm_pack']}`",
        f"- Node.js: `{env['node']}`",
        f"- bio-rs WASM: `biors-wasm v{env['biors_wasm']}`",
        f"- Git commit: `{env['git_commit']}`",
        f"- Benchmark schema: `{result['schema_version']}`",
        "",
        "## Methodology",
        "",
        f"Scope: {result['methodology']['scope']}.",
        "",
        "The script builds the WASM package with `wasm-pack --target nodejs`,",
        "generates deterministic synthetic FASTA data, warms each exported API once,",
        "and records timed in-process Node.js runs plus output hashes.",
        "",
        "## Results",
        "",
        "| Workload | Surface | Input shape | Mean | Median | Min | Max | Output size |",
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |",
    ]

    for workload in result["workloads"]:
        summary = workload["summary"]
        input_ = workload["input"]
        lines.append(
            "| "
            f"`{workload['name']}` | "
            f"`{workload['surface']}` | "
            f"{input_['records']:,} records, {input_['total_residues']:,} residues, "
            f"{input_['file_size_bytes']:,} bytes | "
            f"{summary['mean_s']:.6f}s | "
            f"{summary['median_s']:.6f}s | "
            f"{summary['min_s']:.6f}s | "
            f"{summary['max_s']:.6f}s | "
            f"{summary['output_bytes']:,} bytes |"
        )

    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "python3 scripts/benchmarks/benchmark_wasm_bindings.py",
            "cat benchmarks/wasm_bindings.json",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


if __name__ == "__main__":
    raise SystemExit(main())
