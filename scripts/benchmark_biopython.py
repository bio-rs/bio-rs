#!/usr/bin/env python3
"""Biopython FASTA benchmark using pyperf for rigorous cross-language comparison.

This script produces a pyperf JSON file that can be compared against
criterion JSON from `cargo bench`. Both use their ecosystem's standard
benchmarking library for fair comparison.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

import Bio
from Bio import SeqIO

ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
TOKEN_SET = set(ALPHABET)
AMBIGUOUS_SET = set("XBZJUO")
UNKNOWN_TOKEN_ID = 20
UNIPROT_HUMAN_PROTEOME_GZ_URL = (
    "https://ftp.uniprot.org/pub/databases/uniprot/current_release/"
    "knowledgebase/reference_proteomes/Eukaryota/UP000005640/UP000005640_9606.fasta.gz"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=None)
    parser.add_argument("--output", type=Path, default=Path("benchmarks/biopython.json"))
    parser.add_argument("--loops", type=int, default=25)
    parser.add_argument("--warmup", type=int, default=3)
    return parser.parse_args()


def download_uniprot_human_proteome(destination: Path) -> None:
    gz_path = destination.with_suffix(destination.suffix + ".gz")
    urllib.request.urlretrieve(UNIPROT_HUMAN_PROTEOME_GZ_URL, gz_path)
    with gzip.open(gz_path, "rb") as source, destination.open("wb") as target:
        shutil.copyfileobj(source, target)


def biopython_parse_only(fasta_path: Path) -> None:
    with fasta_path.open("r", encoding="utf-8") as handle:
        for _ in SeqIO.parse(handle, "fasta"):
            pass


def biopython_parse_validate(fasta_path: Path) -> None:
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            sequence = str(record.seq).upper()
            for residue in sequence:
                if residue in TOKEN_SET:
                    pass
                elif residue in AMBIGUOUS_SET:
                    pass


def biopython_parse_tokenize(fasta_path: Path) -> None:
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            sequence = str(record.seq).upper()
            for residue in sequence:
                if residue in TOKEN_SET:
                    pass
                elif residue in AMBIGUOUS_SET:
                    pass


def run_pyperf(name: str, func, loops: int, warmup: int) -> dict:
    try:
        import pyperf
    except ImportError:
        print("ERROR: pyperf not installed. Run: pip install pyperf", file=sys.stderr)
        sys.exit(1)

    runner = pyperf.Runner(loops=loops, warmup=warmup)
    bench = runner.bench_func(name, func)
    
    if bench is None:
        return {"error": "benchmark failed"}
    
    runs = bench.get_runs()
    values = [run.values for run in runs]
    flat_values = [v for sublist in values for v in sublist]
    
    import statistics
    mean_s = statistics.mean(flat_values) if flat_values else 0
    median_s = statistics.median(flat_values) if flat_values else 0
    
    return {
        "name": name,
        "mean_s": mean_s,
        "median_s": median_s,
        "min_s": min(flat_values) if flat_values else 0,
        "max_s": max(flat_values) if flat_values else 0,
        "values": flat_values,
        "loops": len(flat_values),
    }


def main() -> int:
    args = parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        if args.input is None:
            fasta = tmp_path / "human.fasta"
            download_uniprot_human_proteome(fasta)
        else:
            fasta = args.input

        results = {}
        for workload_name, func in [
            ("pure_parse", biopython_parse_only),
            ("parse_plus_validation", biopython_parse_validate),
            ("parse_plus_tokenization", biopython_parse_tokenize),
        ]:
            results[workload_name] = run_pyperf(
                f"biopython_{workload_name}",
                lambda: func(fasta),
                loops=args.loops,
                warmup=args.warmup,
            )

        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open("w") as handle:
            json.dump(results, handle, indent=2)
        print(f"Wrote Biopython benchmark results to {args.output}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
