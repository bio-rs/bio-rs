#!/usr/bin/env python3
"""Validate the committed CLI surface benchmark JSON artifact."""

from __future__ import annotations

from pathlib import Path

from artifact_validation import (
    JsonValue,
    load_json_object,
    require_fields,
    require_object,
    require_sha256,
    require_timed_iterations,
    require_top_level_fields,
    validate_schema_version,
)
from benchmark_release_status import validate_release_status

RESULT_PATH = Path("benchmarks/cli_surfaces.json")
REQUIRED_WORKLOADS = {
    "cli_workflow_fixed_length": "cli_workflow",
    "cli_seq_validate_dna": "nucleotide_validation",
    "cli_tokenize_dna_iupac": "nucleotide_tokenization",
    "cli_model_input_dna_iupac": "nucleotide_model_input",
    "cli_workflow_dna_iupac": "nucleotide_workflow",
    "cli_seq_validate_rna": "nucleotide_validation",
    "cli_tokenize_rna_iupac": "nucleotide_tokenization",
    "cli_model_input_rna_iupac": "nucleotide_model_input",
    "cli_workflow_rna_iupac": "nucleotide_workflow",
    "cli_dataset_inspect_many_file": "cli_dataset_inspect",
    "cli_service_contract": "service_contract",
    "cli_package_validate_example": "package_validation",
    "cli_package_bridge_example": "package_bridge",
}


def main() -> int:
    result = load_json_object(RESULT_PATH)
    validate_schema_version(
        result,
        "biors.benchmark.cli_surfaces.v1",
        "CLI benchmark artifact must use schema v1",
    )
    require_top_level_fields(
        result,
        ["generated_at_utc", "loops", "methodology", "environment", "workloads"],
    )
    validate_release_status(result, environment_key="biors", package_name="biors")
    workloads = {workload["name"]: workload for workload in result["workloads"]}
    missing = sorted(set(REQUIRED_WORKLOADS) - set(workloads))
    if missing:
        raise AssertionError(f"missing CLI benchmark workload(s): {missing}")
    for name, surface in REQUIRED_WORKLOADS.items():
        workload = workloads[name]
        if workload.get("surface") != surface:
            raise AssertionError(f"{name} must cover {surface}")
        validate_input(workload.get("input"))
        validate_result(workload.get("result"))
    return 0


def validate_input(input_: JsonValue) -> None:
    input_ = require_object(input_, "workload input must be an object")
    if input_.get("kind") == "no_input":
        return
    if "sha256" not in input_:
        raise AssertionError("workload input sha256 must use sha256:<hex>")
    require_sha256(input_["sha256"], "workload input sha256 must use sha256:<hex>")
    if "file_size_bytes" not in input_:
        raise AssertionError("workload input missing file_size_bytes")
    if "records" in input_ and "total_residues" not in input_:
        raise AssertionError("FASTA workload input missing total_residues")


def validate_result(result: JsonValue) -> None:
    result = require_fields(
        result,
        ["command", "output_hash", "seconds", "summary"],
        "workload result",
    )
    require_sha256(result["output_hash"], "workload output hash must use sha256:<hex>")
    require_timed_iterations(result["seconds"], "workload must include timed iterations")
    require_fields(
        result["summary"],
        ["mean_s", "median_s", "min_s", "max_s", "stdout_bytes"],
        "workload summary",
    )


if __name__ == "__main__":
    raise SystemExit(main())
