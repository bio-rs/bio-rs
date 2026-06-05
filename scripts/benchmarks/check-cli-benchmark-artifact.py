#!/usr/bin/env python3
"""Validate the committed CLI surface benchmark JSON artifact."""

from __future__ import annotations

import json
from pathlib import Path

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
    result = json.loads(RESULT_PATH.read_text())
    if result.get("schema_version") != "biors.benchmark.cli_surfaces.v1":
        raise AssertionError("CLI benchmark artifact must use schema v1")
    for field in ["generated_at_utc", "loops", "methodology", "environment", "workloads"]:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")
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


def validate_input(input_: object) -> None:
    if not isinstance(input_, dict):
        raise AssertionError("workload input must be an object")
    if input_.get("kind") == "no_input":
        return
    if "sha256" not in input_ or not str(input_["sha256"]).startswith("sha256:"):
        raise AssertionError("workload input sha256 must use sha256:<hex>")
    if "file_size_bytes" not in input_:
        raise AssertionError("workload input missing file_size_bytes")
    if "records" in input_ and "total_residues" not in input_:
        raise AssertionError("FASTA workload input missing total_residues")


def validate_result(result: object) -> None:
    if not isinstance(result, dict):
        raise AssertionError("workload result must be an object")
    for field in ["command", "output_hash", "seconds", "summary"]:
        if field not in result:
            raise AssertionError(f"workload result missing {field}")
    if not str(result["output_hash"]).startswith("sha256:"):
        raise AssertionError("workload output hash must use sha256:<hex>")
    if not result["seconds"]:
        raise AssertionError("workload must include timed iterations")
    summary = result["summary"]
    for field in ["mean_s", "median_s", "min_s", "max_s", "stdout_bytes"]:
        if field not in summary:
            raise AssertionError(f"workload summary missing {field}")


if __name__ == "__main__":
    raise SystemExit(main())
