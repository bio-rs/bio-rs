from __future__ import annotations

import json
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path

import Bio
from benchmark_fasta_inputs import dataset_stats, recorded_dataset_path
from benchmark_support import (
    cargo_package_version,
    command_output,
    raw_sha256_file,
    sha256_json,
)


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
    binary = Path("target") / "release" / "biors-core-benchmark-fasta"
    subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "-p",
            "biors-core",
            "--features",
            "benchmark-tools",
            "--bin",
            "biors-core-benchmark-fasta",
        ],
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
        "import benchmark_fasta_biopython as b; "
        f"b.{function_name}(Path(sys.argv[1]))"
    )
    return peak_memory_bytes([sys.executable, "-c", code, str(fasta_path)])


def biopython_subprocess_run(function_name: str, fasta_path: Path):
    code = (
        "import sys, json; "
        "from pathlib import Path; "
        "sys.path.insert(0, 'scripts'); "
        "import benchmark_fasta_biopython as b; "
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
        "output_hash": sha256_json(warmup_result),
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
    fasta_sha256 = raw_sha256_file(fasta_path)
    input_hash = f"sha256:{fasta_sha256}"

    benchmarks = {
        "pure_parse": benchmark_pair(
            "pure parse",
            "biopython_parse_only",
            "parse",
            fasta_path,
            harness,
            loops,
            residues,
            size_bytes,
            input_hash,
        ),
        "parse_plus_validation": benchmark_pair(
            "parse plus validation",
            "biopython_parse_validate",
            "validate",
            fasta_path,
            harness,
            loops,
            residues,
            size_bytes,
            input_hash,
        ),
        "parse_plus_tokenization": benchmark_pair(
            "parse plus tokenization",
            "biopython_parse_tokenize",
            "tokenize",
            fasta_path,
            harness,
            loops,
            residues,
            size_bytes,
            input_hash,
        ),
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


def benchmark_pair(
    label: str,
    biopython_function: str,
    biors_mode: str,
    fasta_path: Path,
    harness: Path,
    loops: int,
    residues: int,
    size_bytes: int,
    input_hash: str,
) -> dict[str, dict]:
    return {
        "biopython": benchmark_case(
            f"Biopython {label}",
            lambda: biopython_subprocess_run(biopython_function, fasta_path),
            loops=loops,
            residues=residues,
            file_size_bytes=size_bytes,
            input_hash=input_hash,
            memory_fn=lambda: biopython_peak_memory_bytes(biopython_function, fasta_path),
        ),
        "biors_core": benchmark_case(
            f"biors-core {label}",
            lambda: biors_core_benchmark(harness, biors_mode, fasta_path),
            loops=loops,
            residues=residues,
            file_size_bytes=size_bytes,
            input_hash=input_hash,
            memory_fn=lambda: biors_core_peak_memory_bytes(harness, biors_mode, fasta_path),
        ),
    }
