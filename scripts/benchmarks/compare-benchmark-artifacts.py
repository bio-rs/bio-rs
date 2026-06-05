#!/usr/bin/env python3
"""Compare two committed FASTA benchmark JSON artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--allow-missing",
        action="store_true",
        help="report removed benchmark coverage without failing",
    )
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    args = parser.parse_args()

    before = json.loads(args.before.read_text())
    after = json.loads(args.after.read_text())

    rows, coverage_changes = collect_comparison(before, after)
    print_table(rows)
    print_coverage_changes(coverage_changes)
    if any(change["status"] == "removed" for change in coverage_changes):
        return 0 if args.allow_missing else 1
    return 0


def collect_comparison(before: dict, after: dict) -> tuple[list[dict], list[dict]]:
    before_datasets = {dataset["label"]: dataset for dataset in before["datasets"]}
    after_datasets = {dataset["label"]: dataset for dataset in after["datasets"]}
    rows = []
    coverage_changes = []

    coverage_changes.extend(
        coverage_delta("dataset", (), before_datasets.keys(), after_datasets.keys())
    )

    for dataset_label in sorted(before_datasets.keys() & after_datasets.keys()):
        before_workloads = before_datasets[dataset_label]["benchmarks"]
        after_workloads = after_datasets[dataset_label]["benchmarks"]
        coverage_changes.extend(
            coverage_delta(
                "workload",
                (dataset_label,),
                before_workloads.keys(),
                after_workloads.keys(),
            )
        )
        for workload in sorted(before_workloads.keys() & after_workloads.keys()):
            coverage_changes.extend(
                coverage_delta(
                    "implementation",
                    (dataset_label, workload),
                    before_workloads[workload].keys(),
                    after_workloads[workload].keys(),
                )
            )
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

    return rows, coverage_changes


def coverage_delta(
    kind: str, parents: tuple[str, ...], before_keys, after_keys
) -> list[dict]:
    before = set(before_keys)
    after = set(after_keys)
    changes = []
    for name in sorted(before - after):
        changes.append(
            {
                "status": "removed",
                "kind": kind,
                "path": coverage_path(parents, name),
            }
        )
    for name in sorted(after - before):
        changes.append(
            {
                "status": "added",
                "kind": kind,
                "path": coverage_path(parents, name),
            }
        )
    return changes


def coverage_path(parents: tuple[str, ...], name: str) -> str:
    return " / ".join((*parents, name))


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


def print_coverage_changes(changes: list[dict]) -> None:
    if not changes:
        return

    print()
    print("| Status | Kind | Benchmark coverage |")
    print("| --- | --- | --- |")
    for change in changes:
        print(f"| {change['status']} | {change['kind']} | {change['path']} |")


def format_percent(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:+.1f}%"


if __name__ == "__main__":
    raise SystemExit(main())
