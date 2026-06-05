#!/usr/bin/env python3
"""Validate the committed WASM binding benchmark JSON artifact."""

from __future__ import annotations

from pathlib import Path

from artifact_validation import (
    load_json_object,
    require_fields,
    require_sha256,
    require_top_level_fields,
    validate_schema_version,
)
from benchmark_release_status import validate_release_status

RESULT_PATH = Path("benchmarks/wasm_bindings.json")
REQUIRED_WORKLOADS = {
    "wasm_parse_fasta": "wasm_bindings",
    "wasm_validate_fasta": "wasm_bindings",
    "wasm_tokenize": "wasm_bindings",
    "wasm_run_workflow": "wasm_bindings",
    "wasm_tokenize_dna_iupac": "wasm_bindings",
    "wasm_run_workflow_dna_iupac": "wasm_bindings",
    "wasm_tokenize_rna_iupac": "wasm_bindings",
    "wasm_run_workflow_rna_iupac": "wasm_bindings",
}


def main() -> int:
    result = load_json_object(RESULT_PATH)
    validate_schema_version(
        result,
        "biors.benchmark.wasm_bindings.v1",
        "WASM benchmark artifact must use schema v1",
    )
    require_top_level_fields(
        result,
        ["generated_at_utc", "loops", "methodology", "environment", "workloads"],
    )
    validate_release_status(result, environment_key="biors_wasm", package_name="biors-wasm")
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
    input_ = require_fields(
        input_,
        ["records", "total_residues", "file_size_bytes", "sha256"],
        "workload input",
    )
    require_sha256(input_["sha256"], "workload input sha256 must use sha256:<hex>")


def validate_summary(summary: object) -> None:
    summary = require_fields(
        summary,
        ["mean_s", "median_s", "min_s", "max_s", "output_hash", "output_bytes"],
        "workload summary",
    )
    require_sha256(summary["output_hash"], "workload output hash must use sha256:<hex>")


if __name__ == "__main__":
    raise SystemExit(main())
