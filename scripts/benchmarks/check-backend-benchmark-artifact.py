#!/usr/bin/env python3
"""Validate the committed backend smoke benchmark JSON artifact."""

from __future__ import annotations

from pathlib import Path

from artifact_validation import (
    load_json_object,
    require_top_level_fields,
    validate_criterion_estimate,
    validate_schema_version,
)
from benchmark_release_status import validate_release_status

RESULT_PATH = Path("benchmarks/backend_smoke.json")


def main() -> int:
    result = load_json_object(RESULT_PATH)
    validate_schema_version(
        result,
        "biors.benchmark.backend_smoke.v1",
        "backend smoke benchmark artifact must use schema v1",
    )
    require_top_level_fields(
        result,
        ["generated_at_utc", "methodology", "environment", "workloads"],
    )
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
        validate_criterion_estimate(
            estimates.get(estimate),
            estimate,
            require_confidence_interval_fields=True,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
