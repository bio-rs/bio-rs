#!/usr/bin/env python3
"""Reproducible FASTA benchmark (biors vs Biopython).

By default, this script benchmarks against the UniProt human reference proteome
(UP000005640, taxonomy 9606) downloaded from UniProt FTP.

Usage:
  python3 scripts/benchmark_fasta_vs_biopython.py
  python3 scripts/benchmark_fasta_vs_biopython.py --input /path/to/input.fasta
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import statistics
import subprocess
import tempfile
import time
import urllib.request
from datetime import UTC, datetime
from pathlib import Path

from Bio import SeqIO

ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
TOKEN_SET = set(ALPHABET)
AMBIGUOUS_SET = set("XBZJUO")
UNIPROT_HUMAN_PROTEOME_GZ_URL = (
    "https://ftp.uniprot.org/pub/databases/uniprot/current_release/"
    "knowledgebase/reference_proteomes/Eukaryota/UP000005640/UP000005640_9606.fasta.gz"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input",
        type=Path,
        default=None,
        help="Existing FASTA file to benchmark. If omitted, downloads UniProt human proteome.",
    )
    parser.add_argument(
        "--loops",
        type=int,
        default=7,
        help="Number of timed iterations per implementation.",
    )
    return parser.parse_args()


def sha256_of_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download_uniprot_human_proteome(destination_fasta: Path) -> dict[str, str]:
    gz_path = destination_fasta.with_suffix(destination_fasta.suffix + ".gz")
    urllib.request.urlretrieve(UNIPROT_HUMAN_PROTEOME_GZ_URL, gz_path)

    with gzip.open(gz_path, "rb") as source, destination_fasta.open("wb") as target:
        target.write(source.read())

    return {
        "source": "UniProt reference proteome",
        "proteome_id": "UP000005640",
        "taxonomy_id": "9606",
        "download_url": UNIPROT_HUMAN_PROTEOME_GZ_URL,
        "downloaded_gz_sha256": sha256_of_file(gz_path),
    }


def dataset_stats(fasta_path: Path) -> dict[str, int]:
    records = 0
    total_residues = 0
    canonical_residues = 0
    ambiguous_residues = 0
    invalid_residues = 0

    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            records += 1
            sequence = str(record.seq).upper()
            total_residues += len(sequence)
            for residue in sequence:
                if residue in TOKEN_SET:
                    canonical_residues += 1
                elif residue in AMBIGUOUS_SET:
                    ambiguous_residues += 1
                else:
                    invalid_residues += 1

    return {
        "records": records,
        "total_residues": total_residues,
        "canonical_residues": canonical_residues,
        "ambiguous_residues": ambiguous_residues,
        "invalid_residues": invalid_residues,
    }


def run_biopython_parse_only(fasta_path: Path) -> tuple[int, int]:
    records = 0
    residues = 0
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            records += 1
            residues += len(record.seq)
    return records, residues


def run_biopython_parse_token_count(fasta_path: Path) -> int:
    tokens = 0
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            for residue in str(record.seq).upper():
                if residue in TOKEN_SET:
                    tokens += 1
    return tokens


def ensure_biors_release_binary() -> Path:
    binary = Path("target") / "release" / "biors"
    if not binary.exists():
        subprocess.run(
            ["cargo", "build", "--release", "-p", "biors"],
            check=True,
        )
    return binary


def run_biors_cli(binary: Path, command: str, fasta_path: Path) -> None:
    subprocess.run(
        [str(binary), command, str(fasta_path)],
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


def benchmark_case(name: str, fn, loops: int) -> dict:
    # Run once before timing so process startup, imports, and filesystem caches settle.
    warmup_result = fn()
    seconds = timed_runs(fn, loops=loops)
    return {
        "name": name,
        "warmup_result": warmup_result,
        "seconds": seconds,
        "summary": summarize(seconds),
    }


def main() -> int:
    args = parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        if args.input is None:
            fasta_path = tmp_path / "UP000005640_9606.fasta"
            provenance = download_uniprot_human_proteome(fasta_path)
        else:
            fasta_path = args.input
            provenance = {
                "source": "user-provided FASTA",
                "path": str(fasta_path),
            }

        if not fasta_path.exists():
            raise FileNotFoundError(f"FASTA not found: {fasta_path}")

        stats = dataset_stats(fasta_path)
        provenance["fasta_sha256"] = sha256_of_file(fasta_path)
        binary = ensure_biors_release_binary()

        benchmarks = {
            "biopython_parse_only": benchmark_case(
                "Biopython parse only",
                lambda: run_biopython_parse_only(fasta_path),
                loops=args.loops,
            ),
            "biopython_parse_token_count": benchmark_case(
                "Biopython parse + protein-20 token/count loop",
                lambda: run_biopython_parse_token_count(fasta_path),
                loops=args.loops,
            ),
            "biors_cli_inspect_summary": benchmark_case(
                "biors CLI inspect summary output",
                lambda: run_biors_cli(binary, "inspect", fasta_path),
                loops=args.loops,
            ),
            "biors_cli_tokenize_json": benchmark_case(
                "biors CLI tokenize JSON output",
                lambda: run_biors_cli(binary, "tokenize", fasta_path),
                loops=args.loops,
            ),
        }

    result = {
        "dataset": {
            **provenance,
            **stats,
        },
        "generated_at_utc": datetime.now(UTC).replace(microsecond=0).isoformat(),
        "loops": args.loops,
        "benchmarks": benchmarks,
    }

    output_path = Path("benchmarks") / "fasta_vs_biopython.json"
    output_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")

    print(f"Wrote benchmark results to {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
