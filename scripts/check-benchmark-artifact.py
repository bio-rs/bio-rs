#!/usr/bin/env python3
"""Validate the committed benchmark JSON has the current reproducible structure."""

from __future__ import annotations

import json
from pathlib import Path

from benchmark_feature_coverage import REQUIRED_FEATURE_COVERAGE

RESULT_PATH = Path("benchmarks/fasta_vs_biopython.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())

    if result.get("schema_version") != "biors.benchmark.fasta_vs_biopython.v1":
        raise AssertionError("benchmark artifact must use schema v1")

    for field in [
        "generated_at_utc",
        "loops",
        "methodology",
        "environment",
        "datasets",
        "feature_coverage",
    ]:
        if field not in result:
            raise AssertionError(f"missing top-level field: {field}")

    if not isinstance(result["datasets"], list) or len(result["datasets"]) != 4:
        raise AssertionError("benchmark artifact must contain the four recorded shape profiles")

    validate_feature_coverage(result["feature_coverage"])

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
        validate_dataset_provenance(metadata, dataset["label"])
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


def validate_feature_coverage(feature_coverage: object) -> None:
    if not isinstance(feature_coverage, list):
        raise AssertionError("feature_coverage must be a list")

    observed: dict[str, dict] = {}
    for entry in feature_coverage:
        if not isinstance(entry, dict):
            raise AssertionError("feature_coverage entries must be objects")
        for field in ["feature", "status", "claim_scope", "evidence"]:
            if field not in entry:
                raise AssertionError(f"feature_coverage entry missing {field}")
        observed[entry["feature"]] = entry

    missing = sorted(set(REQUIRED_FEATURE_COVERAGE) - set(observed))
    if missing:
        raise AssertionError(f"benchmark artifact missing feature coverage: {missing}")

    for feature, status in REQUIRED_FEATURE_COVERAGE.items():
        entry = observed[feature]
        if entry["status"] != status:
            raise AssertionError(
                f"{feature} must use status {status}, found {entry['status']}"
            )
        if not entry["claim_scope"]:
            raise AssertionError(f"{feature} must describe benchmark claim scope")
        if not entry["evidence"]:
            raise AssertionError(f"{feature} must list benchmark evidence or the gap")


def validate_dataset_provenance(metadata: dict, label: str) -> None:
    for key, value in metadata.items():
        if isinstance(value, str) and "current_release" in value:
            raise AssertionError(
                f"{label} metadata field {key} must not use an unpinned current_release URL"
            )

    source = metadata.get("source")
    if source == "EBI QfO reference proteomes":
        required = [
            "source_release",
            "source_date",
            "download_url",
            "downloaded_fasta_sha256",
        ]
        if metadata["downloaded_fasta_sha256"] != metadata["fasta_sha256"]:
            raise AssertionError(f"{label} downloaded FASTA SHA256 must match fasta_sha256")
    elif "base_proteome_id" in metadata:
        required = ["base_source_release", "base_fasta_sha256"]
    else:
        required = []

    for field in required:
        if not metadata.get(field):
            raise AssertionError(f"{label} missing benchmark provenance field {field}")


if __name__ == "__main__":
    raise SystemExit(main())
