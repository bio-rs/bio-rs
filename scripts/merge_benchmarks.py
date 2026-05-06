#!/usr/bin/env python3
"""Extract throughput metrics from criterion JSON and merge with Biopython pyperf results."""

from __future__ import annotations

import json
import statistics
from pathlib import Path

CRITERION_DIR = Path("target/criterion")
BIOPYTHON_JSON = Path("benchmarks/biopython.json")
OUTPUT_JSON = Path("benchmarks/fasta_vs_biopython.json")
OUTPUT_MD = Path("benchmarks/fasta_vs_biopython.md")


def read_criterion_estimate(group: str, bench_id: str) -> dict | None:
    path = CRITERION_DIR / group / bench_id / "estimates.json"
    if not path.exists():
        return None
    data = json.loads(path.read_text())
    mean_ns = data["mean"]["point_estimate"]
    return {
        "mean_s": mean_ns / 1e9,
        "stddev_s": data["std_dev"]["point_estimate"] / 1e9,
    }


def extract_criterion() -> dict:
    results = {}
    workloads = [
        ("parse_human_proteome", "parse/20659_records", "human_proteome", "pure_parse"),
        ("validate_human_proteome", "validate/20659_records", "human_proteome", "parse_plus_validation"),
        ("tokenize_human_proteome", "tokenize/20659_records", "human_proteome", "parse_plus_tokenization"),
        ("parse_large_fasta", "parse/100MB_plus", "large_scale", "pure_parse"),
        ("validate_large_fasta", "validate/100MB_plus", "large_scale", "parse_plus_validation"),
        ("tokenize_large_fasta", "tokenize/100MB_plus", "large_scale", "parse_plus_tokenization"),
        ("parse_many_short_records", "parse/20000_x_48", "many_short", "pure_parse"),
        ("validate_many_short_records", "validate/20000_x_48", "many_short", "parse_plus_validation"),
        ("tokenize_many_short_records", "tokenize/20000_x_48", "many_short", "parse_plus_tokenization"),
    ]
    
    for group, bench_id, dataset, workload_type in workloads:
        est = read_criterion_estimate(group, bench_id)
        if est:
            if dataset not in results:
                results[dataset] = {}
            if workload_type not in results[dataset]:
                results[dataset][workload_type] = {}
            results[dataset][workload_type]["biors_core"] = est
    return results


def extract_biopython() -> dict:
    if not BIOPYTHON_JSON.exists():
        return {}
    raw = json.loads(BIOPYTHON_JSON.read_text())
    mapping = {
        "pure_parse": "pure_parse",
        "parse_plus_validation": "parse_plus_validation",
        "parse_plus_tokenization": "parse_plus_tokenization",
    }
    
    results = {}
    results["human_proteome"] = {}
    for pyperf_key, workload_type in mapping.items():
        if pyperf_key in raw:
            data = raw[pyperf_key]
            results["human_proteome"][workload_type] = {
                "biopython": {
                    "mean_s": data.get("mean_s", 0),
                    "median_s": data.get("median_s", 0),
                }
            }
    return results


def merge_results(criterion: dict, biopython: dict) -> dict:
    merged = {}
    for dataset in set(criterion.keys()) | set(biopython.keys()):
        merged[dataset] = {}
        c_workloads = criterion.get(dataset, {})
        b_workloads = biopython.get(dataset, {})
        for workload in set(c_workloads.keys()) | set(b_workloads.keys()):
            merged[dataset][workload] = {
                **c_workloads.get(workload, {}),
                **b_workloads.get(workload, {}),
            }
    return merged


def render_markdown(data: dict) -> str:
    lines = [
        "# FASTA core-throughput benchmark",
        "",
        "Cross-language comparison using ecosystem-standard tools:",
        "- **bio-rs**: Criterion (`cargo bench`)",
        "- **Biopython**: pyperf",
        "",
        "Both tools handle warmup, outlier detection, and statistical rigor natively.",
        "Results are normalized to throughput for fair comparison.",
        "",
        "## Results",
        "",
    ]
    
    for dataset, workloads in sorted(data.items()):
        lines.append(f"### {dataset.replace('_', ' ').title()}")
        lines.append("")
        lines.append("| Workload | bio-rs mean | Biopython mean | speedup |")
        lines.append("| --- | ---: | ---: | ---: |")
        for workload, impls in sorted(workloads.items()):
            biors = impls.get("biors_core", {}).get("mean_s", 0)
            bio = impls.get("biopython", {}).get("mean_s", 0)
            speedup = f"**{bio/biors:.1f}x**" if biors > 0 and bio > 0 else "N/A"
            lines.append(
                f"| {workload} | {biors:.4f}s | {bio:.4f}s | {speedup} |"
            )
        lines.append("")
    
    return "\n".join(lines)


def main() -> int:
    criterion_data = extract_criterion()
    biopython_data = extract_biopython()
    merged = merge_results(criterion_data, biopython_data)
    
    with OUTPUT_JSON.open("w") as handle:
        json.dump(merged, handle, indent=2)
    print(f"Wrote merged JSON to {OUTPUT_JSON}")
    
    md = render_markdown(merged)
    with OUTPUT_MD.open("w") as handle:
        handle.write(md)
    print(f"Wrote Markdown report to {OUTPUT_MD}")
    
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
