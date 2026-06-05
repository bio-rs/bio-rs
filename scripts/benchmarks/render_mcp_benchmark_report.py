#!/usr/bin/env python3
"""Render the committed MCP request benchmark report from JSON results."""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

RESULT_PATH = Path("benchmarks/mcp_server.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    print(render_report(result), end="")
    return 0


def render_report(result: dict) -> str:
    env = result["environment"]
    generated_at = datetime.fromisoformat(result["generated_at_utc"])
    workload = result["workloads"][0]
    estimates = workload["criterion_estimates"]
    lines = [
        "# MCP server benchmark",
        "",
        "This benchmark is a release regression guard for MCP request overhead over",
        "an in-process duplex transport. It is not a network throughput claim.",
        "",
        "## Environment",
        "",
        f"- Date: {generated_at.date().isoformat()} (UTC)",
        f"- OS: {env['os']}",
        f"- Machine: `{env['machine']}`",
        f"- Rust: `{env['rustc']}`",
        f"- Cargo: `{env['cargo']}`",
        f"- biors-mcp-server: `v{env['biors_mcp_server']}`",
        f"- Git commit: `{env['git_commit']}`",
        f"- Benchmark schema: `{result['schema_version']}`",
        "",
        "## Methodology",
        "",
        f"Scope: {result['methodology']['scope']}.",
        "",
        "The script runs the existing Criterion bench and exports the generated",
        "`target/criterion` estimates into a committed release-regression artifact.",
        "",
        "## Results",
        "",
        "| Workload | Surface | Mean | Median | Slope | 95% CI mean |",
        "| --- | --- | ---: | ---: | ---: | ---: |",
        "| "
        f"`{workload['name']}` | "
        f"`{workload['surface']}` | "
        f"{ns_to_us(estimates['mean']['point_estimate']):.2f} us | "
        f"{ns_to_us(estimates['median']['point_estimate']):.2f} us | "
        f"{ns_to_us(estimates['slope']['point_estimate']):.2f} us | "
        f"{ci_us(estimates['mean']['confidence_interval'])} |",
        "",
        "## Reproduce",
        "",
        "```bash",
        "python3 scripts/benchmarks/benchmark_mcp_server.py",
        "cat benchmarks/mcp_server.json",
        "```",
        "",
    ]
    return "\n".join(lines)


def ns_to_us(value: float) -> float:
    return value / 1_000.0


def ci_us(interval: dict) -> str:
    return (
        f"{ns_to_us(interval['lower_bound']):.2f}-"
        f"{ns_to_us(interval['upper_bound']):.2f} us"
    )


if __name__ == "__main__":
    raise SystemExit(main())
