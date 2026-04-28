#!/usr/bin/env python3
"""Validate the committed benchmark JSON has reproducibility metadata."""

from __future__ import annotations

import json
from pathlib import Path

RESULT_PATH = Path("benchmarks/fasta_vs_biopython.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    require(result, "schema_version")
    require(result, "methodology")
    require(result, "generated_at_utc")
    require(result, "loops")
    require(result, "environment")
    require(result, "datasets")

    for field in [
        "os",
        "machine",
        "cpu_brand",
        "python",
        "biopython",
        "rustc",
        "cargo",
        "biors_core",
        "git_commit",
    ]:
        require(result["environment"], field)

    for dataset in result["datasets"]:
        require(dataset, "label")
        require(dataset, "dataset")
        require(dataset, "benchmarks")
        require(dataset["dataset"], "fasta_sha256")
        require(dataset["dataset"], "file_size_bytes")
        for workload in dataset["benchmarks"].values():
            for implementation in workload.values():
                require(implementation, "name")
                require(implementation, "input_hash")
                require(implementation, "output_hash")
                require(implementation, "warmup_result")
                require(implementation, "summary")
                require(implementation["summary"], "mean_s")
                require(implementation["summary"], "peak_memory_bytes")
                assert implementation["input_hash"].startswith("sha256:")
                assert implementation["output_hash"].startswith("sha256:")

    return 0


def require(value: dict, key: str) -> None:
    if key not in value:
        raise AssertionError(f"missing benchmark artifact field: {key}")


if __name__ == "__main__":
    raise SystemExit(main())
