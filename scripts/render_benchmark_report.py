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
    env = result["environment"]
    human = result["datasets"][0]
    large = result["datasets"][1]
    generated_at = datetime.fromisoformat(result["generated_at_utc"])
    generated_date = generated_at.date().isoformat()

    return "\n".join(
        [
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
            "### 1. Human reference proteome",
            "",
            "- Source: UniProt human reference proteome",
            f"- Proteome ID: `{human['dataset']['proteome_id']}`",
            f"- Taxonomy ID: `{human['dataset']['taxonomy_id']}` (`Homo sapiens`)",
            f"- URL: `{human['dataset']['download_url']}`",
            f"- Downloaded archive SHA256: `{human['dataset']['downloaded_gz_sha256']}`",
            f"- FASTA SHA256: `{human['dataset']['fasta_sha256']}`",
            f"- Shape: {shape(human)}",
            f"- Residue composition: {composition(human)}",
            "",
            "### 2. Large-scale FASTA",
            "",
            "- Source: repeated UniProt human reference proteome",
            f"- Construction: repeated the same real human proteome `{large['dataset']['repeat_count']}x` to exceed `{large['dataset']['min_target_mb']} MB`",
            f"- FASTA SHA256: `{large['dataset']['fasta_sha256']}`",
            f"- Shape: {shape(large)}",
            f"- Residue composition: {composition(large)}",
            "",
            "This second dataset is intentionally synthetic in scale, but it is built from a",
            "real proteome to isolate large-input throughput without introducing another",
            "dataset's annotation quirks.",
            "",
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
            "## Best-case matched results",
            "",
            "### Human proteome",
            "",
            results_table(human),
            "",
            "### Large-scale FASTA",
            "",
            results_table(large),
            "",
            "## Raw result scope",
            "",
            "The JSON artifact includes all matched workloads, including `pure_parse`. On",
            "this machine, Biopython remains faster on pure parse. The favorable result for",
            "bio-rs appears when the comparison includes the actual validation or",
            "tokenization work that the Rust engine is designed to do.",
            "",
            "That is the intended claim boundary:",
            "",
            "- reasonable claim: bio-rs is materially faster than Biopython on matched",
            "  protein FASTA validation and tokenization workloads in this benchmark",
            "- not a supported claim: bio-rs is universally faster than Biopython for every",
            "  FASTA-related task",
            "",
        ]
    )


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
