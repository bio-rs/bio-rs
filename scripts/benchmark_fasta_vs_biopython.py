#!/usr/bin/env python3
"""Reproducible FASTA parse+tokenize benchmark (biors vs Biopython).

Usage:
  python3 scripts/benchmark_fasta_vs_biopython.py
"""

from __future__ import annotations

import json
import random
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

from Bio import SeqIO

ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
TOKEN_MAP = {aa: i for i, aa in enumerate(ALPHABET)}


def make_fasta(path: Path, records: int, length: int, seed: int) -> None:
    rng = random.Random(seed)
    with path.open("w", encoding="utf-8") as handle:
        for idx in range(records):
            sequence = "".join(rng.choice(ALPHABET) for _ in range(length))
            handle.write(f">seq{idx}\n{sequence}\n")


def run_biopython(fasta_path: Path) -> int:
    tokens = 0
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            for residue in str(record.seq).upper():
                _ = TOKEN_MAP[residue]
                tokens += 1
    return tokens


def run_biors_cli(fasta_path: Path) -> None:
    subprocess.run(
        ["target/release/biors", "tokenize", str(fasta_path)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def timed_runs(fn, loops: int):
    values = []
    for _ in range(loops):
        start = time.perf_counter()
        fn()
        values.append(time.perf_counter() - start)
    return values


def summarize(seconds):
    return {
        "mean_s": statistics.mean(seconds),
        "median_s": statistics.median(seconds),
        "min_s": min(seconds),
        "max_s": max(seconds),
    }


def main() -> int:
    records = 5000
    sequence_length = 600
    loops = 7
    seed = 7

    with tempfile.TemporaryDirectory() as tmp:
        fasta_path = Path(tmp) / "synthetic.fasta"
        make_fasta(fasta_path, records=records, length=sequence_length, seed=seed)

        # Warm up both paths.
        run_biopython(fasta_path)
        run_biors_cli(fasta_path)

        biopython_seconds = timed_runs(lambda: run_biopython(fasta_path), loops=loops)
        biors_seconds = timed_runs(lambda: run_biors_cli(fasta_path), loops=loops)

        result = {
            "dataset": {
                "records": records,
                "sequence_length": sequence_length,
                "total_residues": records * sequence_length,
                "seed": seed,
            },
            "loops": loops,
            "biopython_parse_tokenize": {
                "seconds": biopython_seconds,
                "summary": summarize(biopython_seconds),
            },
            "biors_cli_tokenize": {
                "seconds": biors_seconds,
                "summary": summarize(biors_seconds),
            },
        }

    output_path = Path("benchmarks") / "fasta_vs_biopython.json"
    output_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")

    print(f"Wrote benchmark results to {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
