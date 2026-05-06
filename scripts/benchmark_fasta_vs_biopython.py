#!/usr/bin/env python3
"""Reproducible core FASTA benchmark (biors-core vs Biopython).

This benchmark compares matched workloads:
  - Pure Parse
  - Parse + Validation
  - Parse + Tokenization

For bio-rs it intentionally excludes CLI startup and pretty JSON serialization by
invoking a small `biors-core` benchmark example binary.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
import urllib.request
from datetime import UTC, datetime
from pathlib import Path

import Bio
from Bio import SeqIO
from render_benchmark_report import render_report

ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
TOKEN_SET = set(ALPHABET)
AMBIGUOUS_SET = set("XBZJUO")
UNKNOWN_TOKEN_ID = 20
BENCHMARK_SCHEMA_VERSION = "biors.benchmark.fasta_vs_biopython.v1"
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
        help="Existing FASTA file for the human-proteome benchmark. Defaults to UniProt human proteome.",
    )
    parser.add_argument(
        "--large-input",
        type=Path,
        default=None,
        help="Existing FASTA file for the large-scale benchmark. Defaults to an auto-generated 100MB+ repeated human proteome.",
    )
    parser.add_argument(
        "--loops",
        type=int,
        default=15,
        help="Number of timed iterations per implementation.",
    )
    parser.add_argument(
        "--large-min-mb",
        type=int,
        default=110,
        help="Minimum size in MB for the generated large FASTA when --large-input is omitted.",
    )
    parser.add_argument(
        "--shape-profile-records",
        type=int,
        default=20000,
        help="Number of synthetic records for record-shape stress datasets.",
    )
    parser.add_argument(
        "--short-record-length",
        type=int,
        default=48,
        help="Residues per record for the many-short-records benchmark dataset.",
    )
    return parser.parse_args()


def sha256_of_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sha256_of_json(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return f"sha256:{hashlib.sha256(payload).hexdigest()}"


def download_uniprot_human_proteome(destination_fasta: Path) -> dict[str, str]:
    gz_path = destination_fasta.with_suffix(destination_fasta.suffix + ".gz")
    urllib.request.urlretrieve(UNIPROT_HUMAN_PROTEOME_GZ_URL, gz_path)

    with gzip.open(gz_path, "rb") as source, destination_fasta.open("wb") as target:
        shutil.copyfileobj(source, target)

    return {
        "source": "UniProt reference proteome",
        "proteome_id": "UP000005640",
        "taxonomy_id": "9606",
        "download_url": UNIPROT_HUMAN_PROTEOME_GZ_URL,
        "downloaded_gz_sha256": sha256_of_file(gz_path),
    }


def ensure_large_fasta(source_fasta: Path, destination_fasta: Path, min_mb: int) -> dict[str, int | str]:
    min_bytes = min_mb * 1024 * 1024
    copied = 0
    repeats = 0
    source_bytes = source_fasta.read_bytes()
    with destination_fasta.open("wb") as handle:
        while copied < min_bytes:
            handle.write(source_bytes)
            copied += len(source_bytes)
            repeats += 1

    return {
        "source": "repeated_uniprot_human_proteome",
        "base_proteome_id": "UP000005640",
        "repeat_count": repeats,
        "min_target_mb": min_mb,
    }


def read_residue_stream(source_fasta: Path) -> str:
    residues: list[str] = []
    with source_fasta.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            residues.append(str(record.seq).upper())
    return "".join(residues)


def ensure_many_short_fasta(
    source_fasta: Path,
    destination_fasta: Path,
    *,
    records: int,
    record_length: int,
) -> dict[str, int | str]:
    residues = read_residue_stream(source_fasta)
    required = records * record_length
    repeated = (residues * ((required // len(residues)) + 1))[:required]

    with destination_fasta.open("w", encoding="utf-8") as handle:
        for index in range(records):
            start = index * record_length
            end = start + record_length
            handle.write(f">short_{index}\n")
            handle.write(repeated[start:end])
            handle.write("\n")

    return {
        "source": "synthetic_many_short_records_from_uniprot_human_proteome",
        "base_proteome_id": "UP000005640",
        "shape_profile": "many_short_records",
        "record_count": records,
        "record_length": record_length,
    }


def ensure_single_long_fasta(
    source_fasta: Path,
    destination_fasta: Path,
    *,
    min_residues: int,
) -> dict[str, int | str]:
    residues = read_residue_stream(source_fasta)
    sequence = (residues * ((min_residues // len(residues)) + 1))[:min_residues]

    with destination_fasta.open("w", encoding="utf-8") as handle:
        handle.write(">single_long\n")
        for start in range(0, len(sequence), 80):
            handle.write(sequence[start : start + 80])
            handle.write("\n")

    return {
        "source": "synthetic_single_long_sequence_from_uniprot_human_proteome",
        "base_proteome_id": "UP000005640",
        "shape_profile": "single_long_sequence",
        "target_residues": min_residues,
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
        "file_size_bytes": fasta_path.stat().st_size,
    }


def biopython_parse_only(fasta_path: Path) -> dict[str, int]:
    records = 0
    residues = 0
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            records += 1
            residues += len(record.seq)
    return {"records": records, "residues": residues}


def biopython_parse_validate(fasta_path: Path) -> dict[str, int]:
    records = 0
    residues = 0
    canonical = 0
    warnings = 0
    errors = 0
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            records += 1
            sequence = str(record.seq).upper()
            residues += len(sequence)
            for residue in sequence:
                if residue in TOKEN_SET:
                    canonical += 1
                elif residue in AMBIGUOUS_SET:
                    warnings += 1
                else:
                    errors += 1
    return {
        "records": records,
        "residues": residues,
        "canonical_tokens": canonical,
        "unknown_tokens": warnings + errors,
        "warning_count": warnings,
        "error_count": errors,
    }


def biopython_parse_tokenize(fasta_path: Path) -> dict[str, int]:
    records = 0
    residues = 0
    canonical = 0
    unknown = 0
    warnings = 0
    errors = 0
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            records += 1
            sequence = str(record.seq).upper()
            residues += len(sequence)
            for residue in sequence:
                if residue in TOKEN_SET:
                    canonical += 1
                elif residue in AMBIGUOUS_SET:
                    unknown += 1
                    warnings += 1
                else:
                    unknown += 1
                    errors += 1
    return {
        "records": records,
        "residues": residues,
        "canonical_tokens": canonical,
        "unknown_tokens": unknown,
        "warning_count": warnings,
        "error_count": errors,
        "unknown_token_id": UNKNOWN_TOKEN_ID,
    }


def command_output(command: list[str]) -> str | None:
    try:
        completed = subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    return completed.stdout.strip()


def cargo_package_version(package_name: str) -> str | None:
    output = command_output(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    if output is None:
        return None
    try:
        metadata = json.loads(output)
    except json.JSONDecodeError:
        return None
    for package in metadata.get("packages", []):
        if package.get("name") == package_name:
            return str(package.get("version"))
    return None


def benchmark_environment() -> dict[str, str | None]:
    return {
        "os": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor() or None,
        "cpu_brand": command_output(["sysctl", "-n", "machdep.cpu.brand_string"]),
        "python": platform.python_version(),
        "biopython": Bio.__version__,
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "biors_core": cargo_package_version("biors-core"),
        "git_commit": command_output(["git", "rev-parse", "HEAD"]),
    }


def ensure_benchmark_harness() -> Path:
    binary = Path("target") / "release" / "examples" / "benchmark_fasta"
    subprocess.run(
        ["cargo", "build", "--release", "-p", "biors-core", "--example", "benchmark_fasta"],
        check=True,
    )
    return binary


def biors_core_benchmark(binary: Path, mode: str, fasta_path: Path) -> dict[str, int | str]:
    completed = subprocess.run(
        [str(binary), mode, str(fasta_path)],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return json.loads(completed.stdout)


def peak_memory_bytes(command: list[str]) -> int | None:
    time_binary = Path("/usr/bin/time")
    if not time_binary.exists():
        return None

    if platform.system() == "Darwin":
        completed = subprocess.run(
            [str(time_binary), "-l", *command],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        for line in completed.stderr.splitlines():
            if "maximum resident set size" in line:
                return int(line.split()[0])
        return None

    completed = subprocess.run(
        [str(time_binary), "-f", "%M", *command],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    last_line = completed.stderr.splitlines()[-1] if completed.stderr.splitlines() else ""
    if last_line.isdigit():
        return int(last_line) * 1024
    return None


def biors_core_peak_memory_bytes(binary: Path, mode: str, fasta_path: Path) -> int | None:
    return peak_memory_bytes([str(binary), mode, str(fasta_path)])


def biopython_peak_memory_bytes(function_name: str, fasta_path: Path) -> int | None:
    code = (
        "import sys; "
        "from pathlib import Path; "
        "sys.path.insert(0, 'scripts'); "
        "import benchmark_fasta_vs_biopython as b; "
        f"b.{function_name}(Path(sys.argv[1]))"
    )
    return peak_memory_bytes([sys.executable, "-c", code, str(fasta_path)])


def biopython_subprocess_run(function_name: str, fasta_path: Path):
    """Run a Biopython benchmark function in a fresh subprocess to match Rust's subprocess model."""
    code = (
        "import sys, json; "
        "from pathlib import Path; "
        "sys.path.insert(0, 'scripts'); "
        "import benchmark_fasta_vs_biopython as b; "
        "result = b.{}(Path(sys.argv[1])); "
        "print(json.dumps(result, sort_keys=True, separators=(',', ':')))"
    ).format(function_name)
    completed = subprocess.run(
        [sys.executable, "-c", code, str(fasta_path)],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return json.loads(completed.stdout)


def timed_runs(fn, loops: int, warmup: int = 3) -> list[float]:
    for _ in range(warmup):
        fn()
    values = []
    for _ in range(loops):
        start = time.perf_counter()
        fn()
        values.append(time.perf_counter() - start)
    return values


def summarize(
    seconds: list[float],
    *,
    residues: int,
    file_size_bytes: int,
    peak_memory_bytes: int | None,
) -> dict[str, float | int | None]:
    mean_s = statistics.mean(seconds)
    return {
        "mean_s": mean_s,
        "median_s": statistics.median(seconds),
        "min_s": min(seconds),
        "max_s": max(seconds),
        "residues_per_sec": residues / mean_s,
        "mb_per_sec": (file_size_bytes / (1024 * 1024)) / mean_s,
        "peak_memory_bytes": peak_memory_bytes,
    }


def benchmark_case(
    name: str,
    fn,
    loops: int,
    *,
    residues: int,
    file_size_bytes: int,
    input_hash: str,
    memory_fn=None,
) -> dict:
    warmup_result = fn()
    seconds = timed_runs(fn, loops=loops)
    peak_memory_bytes = memory_fn() if memory_fn is not None else None
    return {
        "name": name,
        "input_hash": input_hash,
        "output_hash": sha256_of_json(warmup_result),
        "warmup_result": warmup_result,
        "seconds": seconds,
        "summary": summarize(
            seconds,
            residues=residues,
            file_size_bytes=file_size_bytes,
            peak_memory_bytes=peak_memory_bytes,
        ),
    }


def dataset_report(label: str, fasta_path: Path, provenance: dict, loops: int, harness: Path) -> dict:
    stats = dataset_stats(fasta_path)
    size_bytes = stats["file_size_bytes"]
    residues = stats["total_residues"]
    fasta_sha256 = sha256_of_file(fasta_path)
    input_hash = f"sha256:{fasta_sha256}"

    benchmarks = {
        "pure_parse": {
            "biopython": benchmark_case(
                "Biopython pure parse",
                lambda: biopython_parse_only(fasta_path),
                loops=loops,
                residues=residues,
                file_size_bytes=size_bytes,
                input_hash=input_hash,
                memory_fn=lambda: biopython_peak_memory_bytes("biopython_parse_only", fasta_path),
            ),
            "biors_core": benchmark_case(
                "biors-core pure parse",
                lambda: biors_core_benchmark(harness, "parse", fasta_path),
                loops=loops,
                residues=residues,
                file_size_bytes=size_bytes,
                input_hash=input_hash,
                memory_fn=lambda: biors_core_peak_memory_bytes(harness, "parse", fasta_path),
            ),
        },
        "parse_plus_validation": {
            "biopython": benchmark_case(
                "Biopython parse plus validation",
                lambda: biopython_parse_validate(fasta_path),
                loops=loops,
                residues=residues,
                file_size_bytes=size_bytes,
                input_hash=input_hash,
                memory_fn=lambda: biopython_peak_memory_bytes(
                    "biopython_parse_validate", fasta_path
                ),
            ),
            "biors_core": benchmark_case(
                "biors-core parse plus validation",
                lambda: biors_core_benchmark(harness, "validate", fasta_path),
                loops=loops,
                residues=residues,
                file_size_bytes=size_bytes,
                input_hash=input_hash,
                memory_fn=lambda: biors_core_peak_memory_bytes(harness, "validate", fasta_path),
            ),
        },
        "parse_plus_tokenization": {
            "biopython": benchmark_case(
                "Biopython parse plus tokenization",
                lambda: biopython_parse_tokenize(fasta_path),
                loops=loops,
                residues=residues,
                file_size_bytes=size_bytes,
                input_hash=input_hash,
                memory_fn=lambda: biopython_peak_memory_bytes(
                    "biopython_parse_tokenize", fasta_path
                ),
            ),
            "biors_core": benchmark_case(
                "biors-core parse plus tokenization",
                lambda: biors_core_benchmark(harness, "tokenize", fasta_path),
                loops=loops,
                residues=residues,
                file_size_bytes=size_bytes,
                input_hash=input_hash,
                memory_fn=lambda: biors_core_peak_memory_bytes(harness, "tokenize", fasta_path),
            ),
        },
    }

    return {
        "label": label,
        "dataset": {
            **provenance,
            "shape_profile": provenance.get("shape_profile", label),
            **stats,
            "fasta_sha256": fasta_sha256,
            "path": recorded_dataset_path(fasta_path, provenance),
        },
        "benchmarks": benchmarks,
    }


def recorded_dataset_path(fasta_path: Path, provenance: dict) -> str:
    if provenance.get("source") == "user-provided FASTA":
        return str(fasta_path)
    return fasta_path.name


def main() -> int:
    args = parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)

        if args.input is None:
            human_fasta = tmp_path / "UP000005640_9606.fasta"
            human_provenance = download_uniprot_human_proteome(human_fasta)
        else:
            human_fasta = args.input
            human_provenance = {
                "source": "user-provided FASTA",
                "path_hint": str(human_fasta),
            }

        if not human_fasta.exists():
            raise FileNotFoundError(f"FASTA not found: {human_fasta}")

        if args.large_input is None:
            large_fasta = tmp_path / f"human_proteome_x{args.large_min_mb}.fasta"
            large_provenance = ensure_large_fasta(human_fasta, large_fasta, args.large_min_mb)
            large_provenance["shape_profile"] = "large_repeated_proteome"
        else:
            large_fasta = args.large_input
            large_provenance = {
                "source": "user-provided FASTA",
                "path_hint": str(large_fasta),
                "shape_profile": "user_provided_large",
            }

        if not large_fasta.exists():
            raise FileNotFoundError(f"Large FASTA not found: {large_fasta}")

        many_short_fasta = tmp_path / "many_short_records.fasta"
        many_short_provenance = ensure_many_short_fasta(
            human_fasta,
            many_short_fasta,
            records=args.shape_profile_records,
            record_length=args.short_record_length,
        )

        single_long_fasta = tmp_path / "single_long_sequence.fasta"
        single_long_provenance = ensure_single_long_fasta(
            human_fasta,
            single_long_fasta,
            min_residues=args.shape_profile_records * args.short_record_length,
        )

        harness = ensure_benchmark_harness()

        result = {
            "schema_version": BENCHMARK_SCHEMA_VERSION,
            "generated_at_utc": datetime.now(UTC).isoformat(),
            "loops": args.loops,
            "methodology": {
                "scope": "core library FASTA throughput, excluding CLI startup and success-envelope JSON serialization",
                "workloads": [
                    "pure_parse",
                    "parse_plus_validation",
                    "parse_plus_tokenization",
                ],
                "shape_profiles": [
                    "human_reference_proteome",
                    "large_repeated_proteome",
                    "many_short_records",
                    "single_long_sequence",
                ],
                "memory": "best-effort peak RSS from /usr/bin/time for biors-core and Biopython subprocesses",
            },
            "environment": benchmark_environment(),
            "datasets": [
                dataset_report(
                    "human_reference_proteome",
                    human_fasta,
                    human_provenance,
                    args.loops,
                    harness,
                ),
                dataset_report(
                    "large_scale_fasta",
                    large_fasta,
                    large_provenance,
                    args.loops,
                    harness,
                ),
                dataset_report(
                    "many_short_records",
                    many_short_fasta,
                    many_short_provenance,
                    args.loops,
                    harness,
                ),
                dataset_report(
                    "single_long_sequence",
                    single_long_fasta,
                    single_long_provenance,
                    args.loops,
                    harness,
                ),
            ],
        }

    output_path = Path("benchmarks") / "fasta_vs_biopython.json"
    report_path = Path("benchmarks") / "fasta_vs_biopython.md"
    output_path.write_text(json.dumps(result, indent=2))
    report_path.write_text(render_report(result))
    print(f"Wrote benchmark results to {output_path}")
    print(f"Wrote benchmark report to {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
