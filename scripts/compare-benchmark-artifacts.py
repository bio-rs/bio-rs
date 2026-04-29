#!/usr/bin/env python3
"""Compare two committed FASTA benchmark JSON artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    args = parser.parse_args()

    before = json.loads(args.before.read_text())
    after = json.loads(args.after.read_text())

    rows = collect_rows(before, after)
    print_table(rows)
    return 0


def collect_rows(before: dict, after: dict) -> list[dict]:
    before_datasets = {dataset["label"]: dataset for dataset in before["datasets"]}
    after_datasets = {dataset["label"]: dataset for dataset in after["datasets"]}
    rows = []

    for dataset_label in sorted(before_datasets.keys() & after_datasets.keys()):
        before_workloads = before_datasets[dataset_label]["benchmarks"]
        after_workloads = after_datasets[dataset_label]["benchmarks"]
        for workload in sorted(before_workloads.keys() & after_workloads.keys()):
            for implementation in sorted(
                before_workloads[workload].keys() & after_workloads[workload].keys()
            ):
                before_summary = before_workloads[workload][implementation]["summary"]
                after_summary = after_workloads[workload][implementation]["summary"]
                rows.append(
                    {
                        "dataset": dataset_label,
                        "workload": workload,
                        "implementation": implementation,
                        "before_s": before_summary["mean_s"],
                        "after_s": after_summary["mean_s"],
                        "seconds_delta_pct": percent_delta(
                            before_summary["mean_s"], after_summary["mean_s"]
                        ),
                        "residues_delta_pct": percent_delta(
                            before_summary["residues_per_sec"],
                            after_summary["residues_per_sec"],
                        ),
                        "memory_delta_pct": percent_delta(
                            before_summary.get("peak_memory_bytes"),
                            after_summary.get("peak_memory_bytes"),
                        ),
                    }
                )

    return rows


def percent_delta(before: float | int | None, after: float | int | None) -> float | None:
    if before in (None, 0) or after is None:
        return None
    return ((after - before) / before) * 100.0


def print_table(rows: list[dict]) -> None:
    print(
        "| Dataset | Workload | Implementation | Before mean | After mean | "
        "Mean delta | Residues/s delta | Memory delta |"
    )
    print("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for row in rows:
        print(
            f"| {row['dataset']} | {row['workload']} | {row['implementation']} | "
            f"{row['before_s']:.3f}s | {row['after_s']:.3f}s | "
            f"{format_percent(row['seconds_delta_pct'])} | "
            f"{format_percent(row['residues_delta_pct'])} | "
            f"{format_percent(row['memory_delta_pct'])} |"
        )


def format_percent(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:+.1f}%"


if __name__ == "__main__":
    raise SystemExit(main())
