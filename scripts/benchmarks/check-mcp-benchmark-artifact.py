#!/usr/bin/env python3
"""Validate the committed MCP server benchmark JSON artifact."""

from __future__ import annotations

from pathlib import Path

from artifact_validation import (
    load_json_object,
    require_top_level_fields,
    validate_criterion_estimate,
    validate_schema_version,
)
from benchmark_release_status import validate_release_status

RESULT_PATH = Path("benchmarks/mcp_server.json")


def main() -> int:
    result = load_json_object(RESULT_PATH)
    validate_schema_version(
        result,
        "biors.benchmark.mcp_server.v1",
        "MCP benchmark artifact must use schema v1",
    )
    require_top_level_fields(
        result,
        ["generated_at_utc", "methodology", "environment", "workloads"],
    )
    validate_release_status(
        result,
        environment_key="biors_mcp_server",
        package_name="biors-mcp-server",
    )
    workloads = result["workloads"]
    if len(workloads) != 1 or workloads[0].get("name") != "mcp_doctor_request_duplex":
        raise AssertionError("MCP artifact must cover mcp_doctor_request_duplex")
    if workloads[0].get("surface") != "mcp_server_request_overhead":
        raise AssertionError("MCP artifact must use the MCP request overhead surface")
    estimates = workloads[0].get("criterion_estimates")
    if not isinstance(estimates, dict):
        raise AssertionError("MCP workload must include Criterion estimates")
    for estimate in ["mean", "median", "slope"]:
        validate_criterion_estimate(
            estimates.get(estimate),
            estimate,
            require_confidence_interval_fields=False,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
