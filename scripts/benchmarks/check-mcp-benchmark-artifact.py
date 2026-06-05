#!/usr/bin/env python3
"""Validate the committed MCP server benchmark JSON artifact."""

from __future__ import annotations

import json
from pathlib import Path

from benchmark_release_status import validate_release_status

RESULT_PATH = Path("benchmarks/mcp_server.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    if result.get("schema_version") != "biors.benchmark.mcp_server.v1":
        raise AssertionError("MCP benchmark artifact must use schema v1")
    for field in ["generated_at_utc", "methodology", "environment", "workloads"]:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")
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
        validate_estimate(estimates.get(estimate), estimate)
    return 0


def validate_estimate(estimate: object, name: str) -> None:
    if not isinstance(estimate, dict):
        raise AssertionError(f"{name} estimate must be an object")
    for field in ["point_estimate", "confidence_interval", "standard_error"]:
        if field not in estimate:
            raise AssertionError(f"{name} estimate missing {field}")


if __name__ == "__main__":
    raise SystemExit(main())
