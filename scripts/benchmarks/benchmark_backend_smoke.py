#!/usr/bin/env python3
"""Export the Candle backend Criterion smoke benchmark as a release artifact."""

from __future__ import annotations

import argparse
import json
import platform
import subprocess
from datetime import UTC, datetime
from pathlib import Path

from benchmark_support import command_output

SCHEMA_VERSION = "biors.benchmark.backend_smoke.v1"
RESULT_PATH = Path("benchmarks/backend_smoke.json")
ESTIMATES_PATH = Path("target/criterion/candle_linear_probe_32x128_cpu/new/estimates.json")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sample-size", type=int, default=10)
    parser.add_argument("--no-run", action="store_true")
    return parser.parse_args()


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
        "biors_backend_candle": cargo_package_version("biors-backend-candle"),
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
                "biors-backend-candle",
                "--bench",
                "candle_linear_probe",
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
            "scope": "Optional Candle CPU backend smoke execution on a synthetic 32x128 model-input payload",
            "criterion_estimates": str(ESTIMATES_PATH),
        },
        "environment": environment(),
        "release_status": {
            "status": "current",
            "claim_policy": "Regression guard timings only; not a public throughput claim.",
        },
        "workloads": [
            {
                "name": "candle_linear_probe_32x128_cpu",
                "surface": "candle_backend_cpu_smoke",
                "criterion_estimates": estimates,
            }
        ],
    }
    RESULT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    print(f"Wrote backend benchmark results to {RESULT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
