#!/usr/bin/env python3
"""Validate the committed benchmark JSON has the current reproducible structure."""

from __future__ import annotations

import json
from pathlib import Path

RESULT_PATH = Path("benchmarks/fasta_vs_biopython.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())

    if result.get("schema_version") != "biors.benchmark.fasta_vs_biopython.v1":
        raise AssertionError("benchmark artifact must use schema v1")

    for field in ["generated_at_utc", "loops", "methodology", "environment", "datasets"]:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")

    if not isinstance(result["datasets"], list) or len(result["datasets"]) != 4:
        raise AssertionError("benchmark artifact must contain the four recorded shape profiles")

    required_workloads = [
        "pure_parse",
        "parse_plus_validation",
        "parse_plus_tokenization",
    ]
    required_dataset_fields = [
        "shape_profile",
        "records",
        "total_residues",
        "canonical_residues",
        "ambiguous_residues",
        "invalid_residues",
        "file_size_bytes",
        "fasta_sha256",
    ]

    for dataset in result["datasets"]:
        if "label" not in dataset or "dataset" not in dataset or "benchmarks" not in dataset:
            raise AssertionError("each dataset must include label, dataset metadata, and benchmarks")
        metadata = dataset["dataset"]
        for field in required_dataset_fields:
            if field not in metadata:
                raise AssertionError(f"missing dataset field {field} in {dataset.get('label')}")
        for workload in required_workloads:
            if workload not in dataset["benchmarks"]:
                raise AssertionError(f"missing workload {workload} in {dataset['label']}")
            workload_result = dataset["benchmarks"][workload]
            for implementation in ["biors_core", "biopython"]:
                if implementation not in workload_result:
                    raise AssertionError(
                        f"missing {implementation} result for {workload} in {dataset['label']}"
                    )
                case = workload_result[implementation]
                for field in ["input_hash", "output_hash", "seconds", "summary"]:
                    if field not in case:
                        raise AssertionError(
                            f"missing {field} in {implementation} {workload} result"
                        )
                summary = case["summary"]
                for field in ["mean_s", "residues_per_sec", "mb_per_sec"]:
                    if field not in summary:
                        raise AssertionError(
                            f"missing summary field {field} in {implementation} {workload}"
                        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
