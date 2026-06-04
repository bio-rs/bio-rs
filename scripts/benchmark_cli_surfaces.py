#!/usr/bin/env python3
"""Record small CLI surface benchmark artifacts for release regression guards."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shutil
import statistics
import subprocess
import time
from datetime import UTC, datetime
from pathlib import Path

from render_cli_benchmark_report import render_report

SCHEMA_VERSION = "biors.benchmark.cli_surfaces.v1"
RESULT_PATH = Path("benchmarks/cli_surfaces.json")
REPORT_PATH = Path("benchmarks/cli_surfaces.md")
WORK_DIR = Path(".benchmark-cli-surfaces")
ALPHABET = b"ACDEFGHIKLMNPQRSTVWY"
DNA_ALPHABET = b"ACGT"
RNA_ALPHABET = b"ACGU"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loops", type=int, default=7)
    parser.add_argument("--bin", type=Path, default=Path("target/release/biors"))
    parser.add_argument("--no-build", action="store_true")
    return parser.parse_args()


def command_output(command: list[str]) -> str | None:
    try:
        completed = subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    return completed.stdout.strip()


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


def cargo_package_version(package_name: str) -> str | None:
    output = command_output(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    if output is None:
        return None
    metadata = json.loads(output)
    for package in metadata.get("packages", []):
        if package.get("name") == package_name:
            return str(package.get("version"))
    return None


def ensure_binary(binary: Path, no_build: bool) -> Path:
    if not no_build:
        subprocess.run(["cargo", "build", "--release", "-p", "biors"], check=True)
    if not binary.exists():
        raise FileNotFoundError(f"biors binary not found: {binary}")
    return binary


def sequence(seed: int, length: int, alphabet: bytes = ALPHABET) -> bytes:
    result = bytearray()
    value = seed
    for _ in range(length):
        value = (value * 6364136223846793005 + 1) & ((1 << 64) - 1)
        result.append(alphabet[(value >> 32) % len(alphabet)])
    return bytes(result)


def write_fasta(
    path: Path,
    *,
    records: int,
    length: int,
    alphabet: bytes = ALPHABET,
) -> dict[str, int | str]:
    residues = 0
    with path.open("wb") as handle:
        for index in range(records):
            handle.write(f">seq_{index}\n".encode())
            seq = sequence(index, length, alphabet)
            residues += len(seq)
            for offset in range(0, len(seq), 60):
                handle.write(seq[offset : offset + 60])
                handle.write(b"\n")
    return {
        "path": path.name,
        "records": records,
        "total_residues": residues,
        "file_size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def write_many_file_dataset(root: Path, *, files: int, records_per_file: int, length: int) -> dict:
    root.mkdir()
    total_bytes = 0
    total_records = 0
    total_residues = 0
    hashes = []
    for file_index in range(files):
        path = root / f"sample_{file_index:03}.fasta"
        info = write_fasta(path, records=records_per_file, length=length)
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


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return f"sha256:{hasher.hexdigest()}"


def sha256_text(value: str) -> str:
    return f"sha256:{hashlib.sha256(value.encode()).hexdigest()}"


def sha256_json_bytes(bytes_: bytes) -> str:
    value = json.loads(bytes_)
    canonical = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return f"sha256:{hashlib.sha256(canonical).hexdigest()}"


def file_input(path: Path, kind: str) -> dict[str, str | int]:
    return {
        "kind": kind,
        "path": str(path),
        "file_size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def timed_command(command: list[str], loops: int) -> dict:
    warmup = subprocess.run(
        command,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    seconds = []
    stdout_bytes = 0
    for _ in range(loops):
        start = time.perf_counter()
        completed = subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        seconds.append(time.perf_counter() - start)
        stdout_bytes = len(completed.stdout)
    return {
        "command": command,
        "output_hash": sha256_json_bytes(warmup.stdout),
        "seconds": seconds,
        "summary": {
            "mean_s": statistics.mean(seconds),
            "median_s": statistics.median(seconds),
            "min_s": min(seconds),
            "max_s": max(seconds),
            "stdout_bytes": stdout_bytes,
        },
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
        workflow_input = write_fasta(workflow_fasta, records=256, length=128)
        dna_fasta = WORK_DIR / "dna.fasta"
        dna_input = write_fasta(dna_fasta, records=256, length=128, alphabet=DNA_ALPHABET)
        rna_fasta = WORK_DIR / "rna.fasta"
        rna_input = write_fasta(rna_fasta, records=256, length=128, alphabet=RNA_ALPHABET)
        dataset_dir = WORK_DIR / "dataset"
        dataset_input = write_many_file_dataset(
            dataset_dir,
            files=32,
            records_per_file=8,
            length=96,
        )

        workloads = [
            workload(
                binary,
                args.loops,
                "cli_workflow_fixed_length",
                "cli_workflow",
                workflow_input,
                ["workflow", "--max-length", "160", str(workflow_fasta)],
            ),
            workload(
                binary,
                args.loops,
                "cli_seq_validate_dna",
                "nucleotide_validation",
                dna_input,
                ["seq", "validate", "--kind", "dna", str(dna_fasta)],
            ),
            workload(
                binary,
                args.loops,
                "cli_tokenize_dna_iupac",
                "nucleotide_tokenization",
                dna_input,
                ["tokenize", "--profile", "dna-iupac", str(dna_fasta)],
            ),
            workload(
                binary,
                args.loops,
                "cli_model_input_dna_iupac",
                "nucleotide_model_input",
                dna_input,
                [
                    "model-input",
                    "--profile",
                    "dna-iupac",
                    "--max-length",
                    "160",
                    str(dna_fasta),
                ],
            ),
            workload(
                binary,
                args.loops,
                "cli_workflow_dna_iupac",
                "nucleotide_workflow",
                dna_input,
                [
                    "workflow",
                    "--profile",
                    "dna-iupac",
                    "--max-length",
                    "160",
                    str(dna_fasta),
                ],
            ),
            workload(
                binary,
                args.loops,
                "cli_seq_validate_rna",
                "nucleotide_validation",
                rna_input,
                ["seq", "validate", "--kind", "rna", str(rna_fasta)],
            ),
            workload(
                binary,
                args.loops,
                "cli_tokenize_rna_iupac",
                "nucleotide_tokenization",
                rna_input,
                ["tokenize", "--profile", "rna-iupac", str(rna_fasta)],
            ),
            workload(
                binary,
                args.loops,
                "cli_model_input_rna_iupac",
                "nucleotide_model_input",
                rna_input,
                [
                    "model-input",
                    "--profile",
                    "rna-iupac",
                    "--max-length",
                    "160",
                    str(rna_fasta),
                ],
            ),
            workload(
                binary,
                args.loops,
                "cli_workflow_rna_iupac",
                "nucleotide_workflow",
                rna_input,
                [
                    "workflow",
                    "--profile",
                    "rna-iupac",
                    "--max-length",
                    "160",
                    str(rna_fasta),
                ],
            ),
            workload(
                binary,
                args.loops,
                "cli_dataset_inspect_many_file",
                "cli_dataset_inspect",
                dataset_input,
                [
                    "dataset",
                    "inspect",
                    "--source",
                    "synthetic",
                    "--version",
                    "benchmark",
                    "--split",
                    "train",
                    str(dataset_dir),
                ],
            ),
            workload(
                binary,
                args.loops,
                "cli_service_contract",
                "service_contract",
                {"kind": "no_input"},
                ["service", "contract"],
            ),
            workload(
                binary,
                args.loops,
                "cli_package_validate_example",
                "package_validation",
                file_input(package_manifest, "package_manifest"),
                ["package", "validate", str(package_manifest)],
            ),
            workload(
                binary,
                args.loops,
                "cli_package_bridge_example",
                "package_bridge",
                file_input(package_manifest, "package_manifest"),
                ["package", "bridge", str(package_manifest)],
            ),
        ]
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


def workload(
    binary: Path,
    loops: int,
    name: str,
    surface: str,
    input_: dict,
    args: list[str],
) -> dict:
    return {
        "name": name,
        "surface": surface,
        "input": input_,
        "result": timed_command([str(binary), *args], loops),
    }


if __name__ == "__main__":
    raise SystemExit(main())
