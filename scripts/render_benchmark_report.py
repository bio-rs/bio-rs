#!/usr/bin/env python3
"""Render the committed FASTA benchmark report from JSON results."""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

RESULT_PATH = Path("benchmarks/fasta_vs_biopython.json")


def main() -> int:
    result = json.loads(RESULT_PATH.read_text())
    print(render_report(result), end="")
    return 0


def render_report(result: dict) -> str:
    if "environment" not in result:
        return render_hyperfine_report(result)

    env = result["environment"]
    generated_at = datetime.fromisoformat(result["generated_at_utc"])
    generated_date = generated_at.date().isoformat()

    lines = [
        "# FASTA core-throughput benchmark",
        "",
        "This repository should not make unverified performance claims.",
        "",
        "This benchmark measures the Rust core library directly. It excludes `biors` CLI",
        "startup and JSON serialization overhead so the result reflects the engine's raw",
        "FASTA throughput.",
        "",
        "## Environment",
        "",
        f"- Date: {generated_date} (UTC)",
        f"- OS: {env['os']}",
        f"- CPU: {env['cpu_brand']}",
        f"- Rust: `{env['rustc']}`",
        f"- Cargo: `{env['cargo']}`",
        f"- bio-rs: `biors-core v{env['biors_core']}`",
        f"- Python: `{env['python']}`",
        f"- Biopython: `{env['biopython']}`",
        f"- Git commit: `{env['git_commit']}`",
        f"- Benchmark schema: `{result['schema_version']}`",
        "",
        "## Datasets",
        "",
    ]

    for index, dataset in enumerate(result["datasets"], start=1):
        lines.extend(dataset_section(index, dataset))

    lines.extend(
        [
            "## Workload matching",
            "",
            f"Scope: {result['methodology']['scope']}.",
            "",
            "The benchmark compares the same work on both sides:",
            "",
            "- Pure Parse: read FASTA records and count records/residues",
            "- Parse + Validation: parse and classify canonical / ambiguous / invalid residues",
            "- Parse + Tokenization: parse and produce position-preserving token IDs with an",
            "  explicit unknown-token path for ambiguous or invalid residues",
            "",
            "For bio-rs, the script rebuilds and invokes the release benchmark example:",
            "",
            "```bash",
            "cargo build --release -p biors-core --example benchmark_fasta",
            "target/release/examples/benchmark_fasta <mode> <input.fasta>",
            "```",
            "",
            "The benchmark example uses `biors-core` buffered reader APIs, not the `biors`",
            "CLI. It excludes CLI startup and success-envelope JSON serialization.",
            "",
            "For Biopython, the script performs matched Python loops over `SeqIO.parse(...)`.",
            "",
            "Each benchmark case records the input FASTA SHA-256, an output hash of the",
            "warmup result, timed iterations, throughput, and best-effort memory metadata.",
            "On macOS, memory uses `/usr/bin/time -l` peak RSS for separate bio-rs",
            "and Biopython subprocesses. Treat memory as run metadata, not a universal",
            "memory-efficiency claim across every FASTA workload.",
            "",
            "## Matched results",
            "",
        ]
    )

    for dataset in result["datasets"]:
        lines.extend(
            [
                f"### {dataset_title(dataset)}",
                "",
                results_table(dataset),
                "",
            ]
        )

    lines.extend(
        [
            "## Reproduce",
            "",
            "```bash",
            "cargo build --release -p biors-core --example benchmark_fasta",
            "python3 -m venv .venv-bench",
            ". .venv-bench/bin/activate",
            "pip install biopython",
            "python scripts/benchmark_fasta_vs_biopython.py",
            "cat benchmarks/fasta_vs_biopython.json",
            "```",
            "",
            "## Raw result scope",
            "",
            "The JSON artifact includes all matched workloads, including `pure_parse`.",
            "The intended claim boundary is workload-specific:",
            "",
            "- reasonable claim: bio-rs is materially faster than Biopython on matched",
            "  protein FASTA validation and tokenization workloads in this benchmark",
            "- not a supported claim: bio-rs is universally faster than Biopython for every",
            "  FASTA-related task",
            "",
        ]
    )

    return "\n".join(lines)


def render_hyperfine_report(result: dict) -> str:
    lines = [
        "# FASTA Benchmark: bio-rs vs Biopython",
        "",
        "Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine).",
        "",
        "| Workload | bio-rs | Biopython | Speedup |",
        "| --- | --- | --- | --- |",
    ]

    for workload in ["parse", "validate", "tokenize"]:
        data = result[workload]
        lines.append(
            "| "
            f"{workload} | "
            f"{data['rust_ms']:.2f}ms | "
            f"{data['python_ms']:.2f}ms | "
            f"**{data['speedup']:.1f}x** |"
        )

    lines.append("")
    return "\n".join(lines)


def dataset_section(index: int, dataset: dict) -> list[str]:
    info = dataset["dataset"]
    lines = [
        f"### {index}. {dataset_title(dataset)}",
        "",
        f"- Source: {info['source']}",
        f"- Shape profile: `{info['shape_profile']}`",
    ]
    if "proteome_id" in info:
        lines.append(f"- Proteome ID: `{info['proteome_id']}`")
    if "taxonomy_id" in info:
        lines.append(f"- Taxonomy ID: `{info['taxonomy_id']}` (`Homo sapiens`)")
    if "download_url" in info:
        lines.append(f"- URL: `{info['download_url']}`")
    if "downloaded_gz_sha256" in info:
        lines.append(f"- Downloaded archive SHA256: `{info['downloaded_gz_sha256']}`")
    if "repeat_count" in info:
        lines.append(
            f"- Construction: repeated the same real human proteome `{info['repeat_count']}x` "
            f"to exceed `{info['min_target_mb']} MB`"
        )
    if "record_count" in info:
        lines.append(
            f"- Construction: `{info['record_count']:,}` records of "
            f"`{info['record_length']}` residues"
        )
    if "target_residues" in info:
        lines.append(f"- Construction: one sequence with `{info['target_residues']:,}` residues")
    lines.extend(
        [
            f"- FASTA SHA256: `{info['fasta_sha256']}`",
            f"- Shape: {shape(dataset)}",
            f"- Residue composition: {composition(dataset)}",
            "",
        ]
    )
    return lines


def dataset_title(dataset: dict) -> str:
    return dataset["label"].replace("_", " ").title()


def shape(dataset: dict) -> str:
    info = dataset["dataset"]
    return (
        f"{info['records']:,} records, {info['total_residues']:,} residues, "
        f"{info['file_size_bytes']:,} bytes"
    )


def composition(dataset: dict) -> str:
    info = dataset["dataset"]
    return (
        f"{info['canonical_residues']:,} canonical, "
        f"{info['ambiguous_residues']:,} ambiguous, {info['invalid_residues']:,} invalid"
    )


def results_table(dataset: dict) -> str:
    rows = [
        "| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for key, label in [
        ("parse_plus_validation", "Parse + validation"),
        ("parse_plus_tokenization", "Parse + tokenization"),
    ]:
        benchmark = dataset["benchmarks"][key]
        biors = benchmark["biors_core"]["summary"]
        biopython = benchmark["biopython"]["summary"]
        speedup = biopython["mean_s"] / biors["mean_s"]
        rows.append(
            "| "
            f"{label} | "
            f"**{biors['mean_s']:.3f}s** | "
            f"{biopython['mean_s']:.3f}s | "
            f"**{speedup:.2f}x** | "
            f"**{biors['residues_per_sec'] / 1_000_000:.1f}M** | "
            f"**{biors['mb_per_sec']:.1f}** | "
            f"{memory(biors['peak_memory_bytes'])} | "
            f"{memory(biopython['peak_memory_bytes'])} |"
        )
    return "\n".join(rows)


def memory(value: int | None) -> str:
    if value is None:
        return "not captured"
    return f"{value / (1024 * 1024):.1f} MB"


if __name__ == "__main__":
    raise SystemExit(main())
