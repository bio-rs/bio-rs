#!/usr/bin/env python3
"""Record small CLI surface benchmark artifacts for release regression guards."""

from __future__ import annotations

import argparse
import json
import platform
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

from benchmark_support import (
    DNA_ALPHABET_BYTES,
    RNA_ALPHABET_BYTES,
    UTC,
    cargo_package_version,
    command_output,
    sha256_file,
    sha256_text,
    write_fasta_bytes,
)
from benchmark_cli_surface_workloads import build_workloads
from render_cli_benchmark_report import render_report

SCHEMA_VERSION = "biors.benchmark.cli_surfaces.v1"
RESULT_PATH = Path("benchmarks/cli_surfaces.json")
REPORT_PATH = Path("benchmarks/cli_surfaces.md")
WORK_DIR = Path(".benchmark-cli-surfaces")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loops", type=int, default=7)
    parser.add_argument("--bin", type=Path, default=Path("target/release/biors"))
    parser.add_argument("--no-build", action="store_true")
    return parser.parse_args()


def environment() -> dict[str, str | None]:
    return {
        "os": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor() or None,
        "python": platform.python_version(),
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "biors": cargo_package_version("biors"),
        "git_commit": command_output(["git", "rev-parse", "HEAD"]),
    }


def ensure_binary(binary: Path, no_build: bool) -> Path:
    if not no_build:
        subprocess.run(["cargo", "build", "--release", "-p", "biors"], check=True)
    if not binary.exists():
        raise FileNotFoundError(f"biors binary not found: {binary}")
    return binary


def write_many_file_dataset(root: Path, *, files: int, records_per_file: int, length: int) -> dict:
    root.mkdir()
    total_bytes = 0
    total_records = 0
    total_residues = 0
    hashes = []
    for file_index in range(files):
        path = root / f"sample_{file_index:03}.fasta"
        info = write_fasta_bytes(path, records=records_per_file, length=length, wrap=60)
        total_bytes += int(info["file_size_bytes"])
        total_records += int(info["records"])
        total_residues += int(info["total_residues"])
        hashes.append(f"{path.name}:{info['sha256']}")
    return {
        "path": root.name,
        "files": files,
        "records": total_records,
        "total_residues": total_residues,
        "file_size_bytes": total_bytes,
        "sha256": sha256_text("\n".join(hashes)),
    }


def main() -> int:
    args = parse_args()
    binary = ensure_binary(args.bin, args.no_build)
    package_manifest = Path("examples/protein-package/manifest.json")

    if WORK_DIR.exists():
        shutil.rmtree(WORK_DIR)
    WORK_DIR.mkdir()
    try:
        workflow_fasta = WORK_DIR / "workflow.fasta"
        workflow_input = write_fasta_bytes(workflow_fasta, records=256, length=128, wrap=60)
        dna_fasta = WORK_DIR / "dna.fasta"
        dna_input = write_fasta_bytes(
            dna_fasta,
            records=256,
            length=128,
            alphabet=DNA_ALPHABET_BYTES,
            wrap=60,
        )
        rna_fasta = WORK_DIR / "rna.fasta"
        rna_input = write_fasta_bytes(
            rna_fasta,
            records=256,
            length=128,
            alphabet=RNA_ALPHABET_BYTES,
            wrap=60,
        )
        dataset_dir = WORK_DIR / "dataset"
        dataset_input = write_many_file_dataset(
            dataset_dir,
            files=32,
            records_per_file=8,
            length=96,
        )

        workloads = build_workloads(
            binary=binary,
            loops=args.loops,
            workflow_fasta=workflow_fasta,
            workflow_input=workflow_input,
            dna_fasta=dna_fasta,
            dna_input=dna_input,
            rna_fasta=rna_fasta,
            rna_input=rna_input,
            dataset_dir=dataset_dir,
            dataset_input=dataset_input,
            package_manifest=package_manifest,
        )
    finally:
        shutil.rmtree(WORK_DIR, ignore_errors=True)

    result = {
        "schema_version": SCHEMA_VERSION,
        "generated_at_utc": datetime.now(UTC).isoformat(),
        "loops": args.loops,
        "methodology": {
            "scope": "CLI regression guard timings on deterministic synthetic inputs and package fixtures; not a public throughput claim",
            "surfaces": [
                "cli_workflow",
                "cli_dataset_inspect",
                "nucleotide_validation",
                "nucleotide_tokenization",
                "nucleotide_model_input",
                "nucleotide_workflow",
                "service_contract",
                "package_validation",
                "package_bridge",
            ],
            "binary": str(binary),
        },
        "environment": environment(),
        "release_status": {
            "status": "current",
            "claim_policy": "Regression guard timings only; not a public throughput claim.",
        },
        "workloads": workloads,
    }

    RESULT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    REPORT_PATH.write_text(render_report(result))
    print(f"Wrote CLI benchmark results to {RESULT_PATH}")
    print(f"Wrote CLI benchmark report to {REPORT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
