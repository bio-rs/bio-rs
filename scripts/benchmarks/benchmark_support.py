from __future__ import annotations

import hashlib
import json
import statistics
import subprocess
import time
from datetime import timezone
from pathlib import Path

UTC = timezone.utc
PROTEIN_ALPHABET_BYTES = b"ACDEFGHIKLMNPQRSTVWY"
DNA_ALPHABET_BYTES = b"ACGT"
RNA_ALPHABET_BYTES = b"ACGU"
PROTEIN_ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
DNA_ALPHABET = "ACGT"
RNA_ALPHABET = "ACGU"


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


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return f"sha256:{hasher.hexdigest()}"


def sha256_text(value: str) -> str:
    return f"sha256:{hashlib.sha256(value.encode()).hexdigest()}"


def sha256_json(value: object) -> str:
    canonical = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return f"sha256:{hashlib.sha256(canonical).hexdigest()}"


def sha256_json_bytes(bytes_: bytes) -> str:
    value = json.loads(bytes_)
    return sha256_json(value)


def deterministic_bytes(seed: int, length: int, alphabet: bytes = PROTEIN_ALPHABET_BYTES) -> bytes:
    result = bytearray()
    value = seed
    for _ in range(length):
        value = (value * 6364136223846793005 + 1) & ((1 << 64) - 1)
        result.append(alphabet[(value >> 32) % len(alphabet)])
    return bytes(result)


def deterministic_text(seed: int, length: int, alphabet: str = PROTEIN_ALPHABET) -> str:
    chars = []
    value = seed
    for _ in range(length):
        value = (value * 6364136223846793005 + 1) & ((1 << 64) - 1)
        chars.append(alphabet[(value >> 32) % len(alphabet)])
    return "".join(chars)


def write_fasta_bytes(
    path: Path,
    *,
    records: int,
    length: int,
    alphabet: bytes = PROTEIN_ALPHABET_BYTES,
    wrap: int | None = None,
) -> dict[str, int | str]:
    residues = 0
    with path.open("wb") as handle:
        for index in range(records):
            handle.write(f">seq_{index}\n".encode())
            seq = deterministic_bytes(index, length, alphabet)
            residues += len(seq)
            if wrap is None:
                handle.write(seq)
                handle.write(b"\n")
            else:
                for offset in range(0, len(seq), wrap):
                    handle.write(seq[offset : offset + wrap])
                    handle.write(b"\n")
    return {
        "path": path.name,
        "records": records,
        "total_residues": residues,
        "file_size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def fasta_text(records: int, length: int, alphabet: str = PROTEIN_ALPHABET) -> str:
    parts = []
    for index in range(records):
        seq = deterministic_text(index, length, alphabet)
        parts.append(f">seq_{index}\n")
        parts.extend(f"{seq[offset:offset + 60]}\n" for offset in range(0, len(seq), 60))
    return "".join(parts)


def timed_case(name: str, fn, loops: int) -> dict:
    warmup = fn()
    seconds = []
    for _ in range(loops):
        start = time.perf_counter()
        fn()
        seconds.append(time.perf_counter() - start)
    return {
        "name": name,
        "output_hash": sha256_json(warmup),
        "warmup_summary": warmup,
        "seconds": seconds,
        "summary": {
            "mean_s": statistics.mean(seconds),
            "median_s": statistics.median(seconds),
            "min_s": min(seconds),
            "max_s": max(seconds),
        },
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
