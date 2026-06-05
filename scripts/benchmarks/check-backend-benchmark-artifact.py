#!/usr/bin/env python3
"""Validate the committed backend smoke benchmark JSON artifact."""

from __future__ import annotations

import json
from pathlib import Path

from benchmark_release_status import validate_release_status

RESULT_PATH = Path("benchmarks/backend_smoke.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    if result.get("schema_version") != "biors.benchmark.backend_smoke.v1":
        raise AssertionError("backend smoke benchmark artifact must use schema v1")
    for field in ["generated_at_utc", "methodology", "environment", "workloads"]:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")
    validate_release_status(
        result,
        environment_key="biors_backend_candle",
        package_name="biors-backend-candle",
    )
    workloads = result["workloads"]
    if len(workloads) != 1 or workloads[0].get("name") != "candle_linear_probe_32x128_cpu":
        raise AssertionError("backend smoke artifact must cover candle_linear_probe_32x128_cpu")
    if workloads[0].get("surface") != "candle_backend_cpu_smoke":
        raise AssertionError("backend smoke artifact must use the backend smoke surface")
    estimates = workloads[0].get("criterion_estimates")
    if not isinstance(estimates, dict):
        raise AssertionError("backend smoke workload must include Criterion estimates")
    for estimate in ["mean", "median", "slope"]:
        validate_estimate(estimates.get(estimate), estimate)
    return 0


def validate_estimate(estimate: object, name: str) -> None:
    if not isinstance(estimate, dict):
        raise AssertionError(f"{name} estimate must be an object")
    for field in ["point_estimate", "confidence_interval", "standard_error"]:
        if field not in estimate:
            raise AssertionError(f"{name} estimate missing {field}")
    interval = estimate["confidence_interval"]
    for field in ["confidence_level", "lower_bound", "upper_bound"]:
        if field not in interval:
            raise AssertionError(f"{name} confidence interval missing {field}")


if __name__ == "__main__":
    raise SystemExit(main())
