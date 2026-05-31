#!/usr/bin/env python3
"""Validate the committed Python binding benchmark JSON artifact."""

from __future__ import annotations

import json
from pathlib import Path

RESULT_PATH = Path("benchmarks/python_bindings.json")
REQUIRED_WORKLOADS = {
    "python_parse_fasta_records",
    "python_tokenize_fasta_records",
    "python_build_model_inputs_checked",
    "python_prepare_workflow_from_fasta",
}


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    if result.get("schema_version") != "biors.benchmark.python_bindings.v1":
        raise AssertionError("Python benchmark artifact must use schema v1")
    for field in ["generated_at_utc", "loops", "methodology", "environment", "input", "workloads"]:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")
    observed = {workload["name"]: workload for workload in result["workloads"]}
    missing = sorted(REQUIRED_WORKLOADS - set(observed))
    if missing:
        raise AssertionError(f"missing Python benchmark workload(s): {missing}")
    validate_input(result["input"])
    for workload in observed.values():
        validate_workload(workload)
    return 0


def validate_input(input_: object) -> None:
    if not isinstance(input_, dict):
        raise AssertionError("input must be an object")
    for field in ["records", "record_length", "total_residues", "fasta_bytes", "sha256"]:
        if field not in input_:
            raise AssertionError(f"input missing {field}")
    if not str(input_["sha256"]).startswith("sha256:"):
        raise AssertionError("input sha256 must use sha256:<hex>")


def validate_workload(workload: dict) -> None:
    for field in ["name", "output_hash", "warmup_summary", "seconds", "summary"]:
        if field not in workload:
            raise AssertionError(f"workload missing {field}")
    if not str(workload["output_hash"]).startswith("sha256:"):
        raise AssertionError("workload output hash must use sha256:<hex>")
    if not workload["seconds"]:
        raise AssertionError("workload must include timed iterations")
    for field in ["mean_s", "median_s", "min_s", "max_s"]:
        if field not in workload["summary"]:
            raise AssertionError(f"workload summary missing {field}")


if __name__ == "__main__":
    raise SystemExit(main())
