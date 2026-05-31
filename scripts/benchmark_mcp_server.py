#!/usr/bin/env python3
"""Export the MCP server Criterion request benchmark as a release artifact."""

from __future__ import annotations

import argparse
import json
import platform
import subprocess
from datetime import UTC, datetime
from pathlib import Path

from render_mcp_benchmark_report import render_report

SCHEMA_VERSION = "biors.benchmark.mcp_server.v1"
RESULT_PATH = Path("benchmarks/mcp_server.json")
REPORT_PATH = Path("benchmarks/mcp_server.md")
ESTIMATES_PATH = Path("target/criterion/mcp_doctor_request_duplex/new/estimates.json")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sample-size", type=int, default=10)
    parser.add_argument("--no-run", action="store_true")
    return parser.parse_args()


def command_output(command: list[str]) -> str | None:
    try:
        completed = subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    return completed.stdout.strip()


def cargo_package_version(package_name: str) -> str | None:
    output = command_output(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    if output is None:
        return None
    for package in json.loads(output).get("packages", []):
        if package.get("name") == package_name:
            return str(package.get("version"))
    return None


def environment() -> dict[str, str | None]:
    return {
        "os": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor() or None,
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "biors_mcp_server": cargo_package_version("biors-mcp-server"),
        "git_commit": command_output(["git", "rev-parse", "HEAD"]),
    }


def main() -> int:
    args = parse_args()
    if not args.no_run:
        subprocess.run(
            [
                "cargo",
                "bench",
                "-p",
                "biors-mcp-server",
                "--bench",
                "mcp_request_overhead",
                "--",
                "--sample-size",
                str(args.sample_size),
            ],
            check=True,
        )
    estimates = json.loads(ESTIMATES_PATH.read_text())
    result = {
        "schema_version": SCHEMA_VERSION,
        "generated_at_utc": datetime.now(UTC).isoformat(),
        "methodology": {
            "scope": "MCP doctor tool request over rmcp client/server duplex transport",
            "criterion_estimates": str(ESTIMATES_PATH),
        },
        "environment": environment(),
        "workloads": [
            {
                "name": "mcp_doctor_request_duplex",
                "surface": "mcp_server_request_overhead",
                "criterion_estimates": estimates,
            }
        ],
    }
    RESULT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    REPORT_PATH.write_text(render_report(result))
    print(f"Wrote MCP benchmark results to {RESULT_PATH}")
    print(f"Wrote MCP benchmark report to {REPORT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
