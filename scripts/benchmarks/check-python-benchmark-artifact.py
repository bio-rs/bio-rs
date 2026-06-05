#!/usr/bin/env python3
"""Validate the committed Python binding benchmark JSON artifact."""

from __future__ import annotations

from pathlib import Path

from artifact_validation import (
    load_json_object,
    require_fields,
    require_sha256,
    require_timed_iterations,
    require_top_level_fields,
    validate_schema_version,
)
from benchmark_release_status import validate_release_status

RESULT_PATH = Path("benchmarks/python_bindings.json")
REQUIRED_WORKLOADS = {
    "python_parse_fasta_records",
    "python_tokenize_fasta_records",
    "python_build_model_inputs_checked",
    "python_prepare_workflow_from_fasta",
    "python_tokenize_fasta_records_dna_iupac",
    "python_build_model_inputs_checked_dna_iupac",
    "python_prepare_workflow_from_fasta_dna_iupac",
    "python_tokenize_fasta_records_rna_iupac",
    "python_build_model_inputs_checked_rna_iupac",
    "python_prepare_workflow_from_fasta_rna_iupac",
}


def main() -> int:
    result = load_json_object(RESULT_PATH)
    validate_schema_version(
        result,
        "biors.benchmark.python_bindings.v1",
        "Python benchmark artifact must use schema v1",
    )
    require_top_level_fields(
        result,
        ["generated_at_utc", "loops", "methodology", "environment", "input", "workloads"],
    )
    validate_release_status(
        result,
        environment_key="biors_python",
        package_name="biors-python",
    )
    observed = {workload["name"]: workload for workload in result["workloads"]}
    missing = sorted(REQUIRED_WORKLOADS - set(observed))
    if missing:
        raise AssertionError(f"missing Python benchmark workload(s): {missing}")
    validate_input(result["input"])
    for workload in observed.values():
        validate_workload(workload)
    return 0


def validate_input(input_: object) -> None:
    input_ = require_fields(
        input_,
        ["records", "record_length", "total_residues", "fasta_bytes", "sha256"],
        "input",
    )
    require_sha256(input_["sha256"], "input sha256 must use sha256:<hex>")


def validate_workload(workload: dict) -> None:
    require_fields(
        workload,
        ["name", "output_hash", "warmup_summary", "seconds", "summary"],
        "workload",
    )
    require_sha256(workload["output_hash"], "workload output hash must use sha256:<hex>")
    require_timed_iterations(workload["seconds"], "workload must include timed iterations")
    require_fields(
        workload["summary"],
        ["mean_s", "median_s", "min_s", "max_s"],
        "workload summary",
    )


if __name__ == "__main__":
    raise SystemExit(main())
