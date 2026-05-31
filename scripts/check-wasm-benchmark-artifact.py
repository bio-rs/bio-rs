#!/usr/bin/env python3
"""Validate the committed WASM binding benchmark JSON artifact."""

from __future__ import annotations

import json
from pathlib import Path

RESULT_PATH = Path("benchmarks/wasm_bindings.json")
REQUIRED_WORKLOADS = {
    "wasm_parse_fasta": "wasm_bindings",
    "wasm_validate_fasta": "wasm_bindings",
    "wasm_tokenize": "wasm_bindings",
    "wasm_run_workflow": "wasm_bindings",
}


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    if result.get("schema_version") != "biors.benchmark.wasm_bindings.v1":
        raise AssertionError("WASM benchmark artifact must use schema v1")
    for field in ["generated_at_utc", "loops", "methodology", "environment", "workloads"]:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")
    workloads = {workload["name"]: workload for workload in result["workloads"]}
    missing = sorted(set(REQUIRED_WORKLOADS) - set(workloads))
    if missing:
        raise AssertionError(f"missing WASM benchmark workload(s): {missing}")
    for name, surface in REQUIRED_WORKLOADS.items():
        workload = workloads[name]
        if workload.get("surface") != surface:
            raise AssertionError(f"{name} must cover {surface}")
        validate_input(workload.get("input"))
        validate_summary(workload.get("summary"))
    return 0


def validate_input(input_: object) -> None:
    if not isinstance(input_, dict):
        raise AssertionError("workload input must be an object")
    for field in ["records", "total_residues", "file_size_bytes", "sha256"]:
        if field not in input_:
            raise AssertionError(f"workload input missing {field}")
    if not str(input_["sha256"]).startswith("sha256:"):
        raise AssertionError("workload input sha256 must use sha256:<hex>")


def validate_summary(summary: object) -> None:
    if not isinstance(summary, dict):
        raise AssertionError("workload summary must be an object")
    for field in ["mean_s", "median_s", "min_s", "max_s", "output_hash", "output_bytes"]:
        if field not in summary:
            raise AssertionError(f"workload summary missing {field}")
    if not str(summary["output_hash"]).startswith("sha256:"):
        raise AssertionError("workload output hash must use sha256:<hex>")


if __name__ == "__main__":
    raise SystemExit(main())
