#!/usr/bin/env python3
"""Run hyperfine-based cross-language benchmark and generate report.

This is the industry-standard approach for fair cross-language benchmarking.
Hyperfine handles warmup, statistical analysis, and outlier detection.
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

UNIPROT_HUMAN_PROTEOME_GZ_URL = (
    "https://ftp.uniprot.org/pub/databases/uniprot/current_release/"
    "knowledgebase/reference_proteomes/Eukaryota/UP000005640/UP000005640_9606.fasta.gz"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=None)
    parser.add_argument("--loops", type=int, default=15)
    parser.add_argument("--warmup", type=int, default=3)
    parser.add_argument("--output-json", type=Path, default=Path("benchmarks/fasta_vs_biopython.json"))
    parser.add_argument("--output-md", type=Path, default=Path("benchmarks/fasta_vs_biopython.md"))
    return parser.parse_args()


def download_uniprot_human_proteome(destination: Path) -> None:
    gz_path = destination.with_suffix(destination.suffix + ".gz")
    urllib.request.urlretrieve(UNIPROT_HUMAN_PROTEOME_GZ_URL, gz_path)
    with gzip.open(gz_path, "rb") as source, destination.open("wb") as target:
        shutil.copyfileobj(source, target)


def ensure_large_fasta(source_fasta: Path, destination_fasta: Path, min_mb: int) -> None:
    min_bytes = min_mb * 1024 * 1024
    copied = 0
    source_bytes = source_fasta.read_bytes()
    with destination_fasta.open("wb") as handle:
        while copied < min_bytes:
            handle.write(source_bytes)
            copied += len(source_bytes)


def sha256_of_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run_hyperfine(rust_cmd: str, python_cmd: str, loops: int, warmup: int) -> dict:
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False, mode="w") as tmp:
        tmp_path = Path(tmp.name)

    result = subprocess.run(
        [
            "hyperfine",
            "--warmup", str(warmup),
            "--runs", str(loops),
            "--export-json", str(tmp_path),
            rust_cmd,
            python_cmd,
        ],
        check=True,
        capture_output=True,
        text=True,
    )

    data = json.loads(tmp_path.read_text())
    tmp_path.unlink()
    return data


def extract_mean_ms(data: dict, command_idx: int) -> float:
    return data["results"][command_idx]["mean"] * 1000


def format_time(ms: float) -> str:
    if ms < 1:
        return f"{ms*1000:.0f}μs"
    elif ms < 1000:
        return f"{ms:.2f}ms"
    else:
        return f"{ms/1000:.3f}s"


def main() -> int:
    args = parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)

        if args.input is None:
            human_fasta = tmp_path / "human.fasta"
            download_uniprot_human_proteome(human_fasta)
        else:
            human_fasta = args.input

        large_fasta = tmp_path / "large.fasta"
        ensure_large_fasta(human_fasta, large_fasta, 110)

        rust_bin = "target/release/examples/benchmark_fasta"
        py_script = "scripts/biopython_bench.py"

        results = {}
        for workload, mode in [("parse", "parse"), ("validate", "validate"), ("tokenize", "tokenize")]:
            print(f"Benchmarking {workload} on human proteome...")
            data = run_hyperfine(
                f"{rust_bin} {mode} {human_fasta}",
                f"python {py_script} {mode} {human_fasta}",
                args.loops,
                args.warmup,
            )
            rust_ms = extract_mean_ms(data, 0)
            py_ms = extract_mean_ms(data, 1)
            results[workload] = {
                "rust_ms": rust_ms,
                "python_ms": py_ms,
                "speedup": py_ms / rust_ms if rust_ms > 0 else 0,
            }

        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        with args.output_json.open("w") as handle:
            json.dump(results, handle, indent=2)

        with args.output_md.open("w") as handle:
            handle.write("# FASTA Benchmark: bio-rs vs Biopython\n\n")
            handle.write("Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine).\n\n")
            handle.write("| Workload | bio-rs | Biopython | Speedup |\n")
            handle.write("| --- | --- | --- | --- |\n")
            for workload, data in results.items():
                handle.write(
                    f"| {workload} | {format_time(data['rust_ms'])} | "
                    f"{format_time(data['python_ms'])} | **{data['speedup']:.1f}x** |\n"
                )

        print(f"Wrote {args.output_json} and {args.output_md}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
