#!/usr/bin/env python3
"""Record Python binding benchmark artifacts for release regression guards."""

from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import platform
import statistics
import sys
import time
from datetime import UTC, datetime
from pathlib import Path

from render_python_benchmark_report import render_report

SCHEMA_VERSION = "biors.benchmark.python_bindings.v1"
RESULT_PATH = Path("benchmarks/python_bindings.json")
REPORT_PATH = Path("benchmarks/python_bindings.md")
PYTHON_PACKAGE_PATH = Path("packages/rust/biors-python/python")
ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
DNA_ALPHABET = "ACGT"
RNA_ALPHABET = "ACGU"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loops", type=int, default=25)
    parser.add_argument("--pythonpath", type=Path, default=PYTHON_PACKAGE_PATH)
    return parser.parse_args()


def sequence(seed: int, length: int, alphabet: str = ALPHABET) -> str:
    chars = []
    value = seed
    for _ in range(length):
        value = (value * 6364136223846793005 + 1) & ((1 << 64) - 1)
        chars.append(alphabet[(value >> 32) % len(alphabet)])
    return "".join(chars)


def fasta(records: int, length: int, alphabet: str = ALPHABET) -> str:
    parts = []
    for index in range(records):
        seq = sequence(index, length, alphabet)
        parts.append(f">seq_{index}\n")
        parts.extend(f"{seq[offset:offset + 60]}\n" for offset in range(0, len(seq), 60))
    return "".join(parts)


def sha256_text(value: str) -> str:
    return f"sha256:{hashlib.sha256(value.encode()).hexdigest()}"


def sha256_json(value: object) -> str:
    canonical = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return f"sha256:{hashlib.sha256(canonical).hexdigest()}"


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


def load_biors(pythonpath: Path):
    sys.path.insert(0, str(pythonpath))
    return importlib.import_module("biors")


def environment(biors_module) -> dict[str, str | None]:
    return {
        "os": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor() or None,
        "python": platform.python_version(),
        "biors_python": cargo_package_version("biors-python"),
        "biors_module": str(Path(biors_module.__file__).relative_to(Path.cwd())),
        "git_commit": command_output(["git", "rev-parse", "HEAD"]),
    }


def cargo_package_version(package_name: str) -> str | None:
    output = command_output(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    if output is None:
        return None
    for package in json.loads(output).get("packages", []):
        if package.get("name") == package_name:
            return str(package.get("version"))
    return None


def command_output(command: list[str]) -> str | None:
    import subprocess

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


def main() -> int:
    args = parse_args()
    biors = load_biors(args.pythonpath)
    input_fasta = fasta(records=512, length=128)
    dna_fasta = fasta(records=512, length=128, alphabet=DNA_ALPHABET)
    rna_fasta = fasta(records=512, length=128, alphabet=RNA_ALPHABET)
    input_hash = sha256_text(input_fasta)
    dna_input_hash = sha256_text(dna_fasta)
    rna_input_hash = sha256_text(rna_fasta)
    tokenized = biors.tokenize_fasta_records(input_fasta)
    dna_tokenized = biors.tokenize_fasta_records(dna_fasta, profile="dna-iupac")
    rna_tokenized = biors.tokenize_fasta_records(rna_fasta, profile="rna-iupac")

    def parse_summary() -> dict:
        records = biors.parse_fasta_records(input_fasta)
        return {
            "records": len(records),
            "total_residues": sum(len(record.sequence) for record in records),
            "first_id": records[0].id,
            "last_id": records[-1].id,
        }

    def tokenize_summary() -> dict:
        records = biors.tokenize_fasta_records(input_fasta)
        return {
            "records": len(records),
            "total_tokens": sum(len(record.tokens) for record in records),
            "valid_records": sum(1 for record in records if record.valid),
        }

    def model_input_summary() -> dict:
        model_input = biors.build_model_inputs_checked(
            tokenized,
            max_length=160,
            padding="fixed_length",
            pad_token_id=21,
        )
        return {
            "records": len(model_input.records),
            "total_input_ids": sum(len(record.input_ids) for record in model_input.records),
            "truncated_records": sum(1 for record in model_input.records if record.truncated),
        }

    def workflow_summary() -> dict:
        workflow = biors.prepare_workflow_from_fasta(
            input_fasta,
            max_length=160,
            padding="fixed_length",
            pad_token_id=21,
        )
        return {
            "records": len(workflow.records),
            "model_ready": workflow.model_ready,
            "input_hash_prefix": workflow.input_hash.split(":", 1)[0],
            "total_input_ids": sum(len(record.input_ids) for record in workflow.records),
        }

    def nucleotide_tokenize_summary(input_fasta: str, profile: str) -> dict:
        records = biors.tokenize_fasta_records(input_fasta, profile=profile)
        return {
            "records": len(records),
            "total_tokens": sum(len(record.tokens) for record in records),
            "valid_records": sum(1 for record in records if record.valid),
            "profile": profile,
        }

    def nucleotide_model_input_summary(tokenized_records: list, profile: str) -> dict:
        model_input = biors.build_model_inputs_checked(
            tokenized_records,
            max_length=160,
            padding="fixed_length",
            pad_token_id=0,
        )
        return {
            "records": len(model_input.records),
            "total_input_ids": sum(len(record.input_ids) for record in model_input.records),
            "truncated_records": sum(1 for record in model_input.records if record.truncated),
            "profile": profile,
        }

    def nucleotide_workflow_summary(input_fasta: str, profile: str) -> dict:
        workflow = biors.prepare_workflow_from_fasta(
            input_fasta,
            max_length=160,
            padding="fixed_length",
            pad_token_id=0,
            profile=profile,
        )
        return {
            "records": len(workflow.records),
            "model_ready": workflow.model_ready,
            "input_hash_prefix": workflow.input_hash.split(":", 1)[0],
            "total_input_ids": sum(len(record.input_ids) for record in workflow.records),
            "profile": profile,
        }

    result = {
        "schema_version": SCHEMA_VERSION,
        "generated_at_utc": datetime.now(UTC).isoformat(),
        "loops": args.loops,
        "methodology": {
            "scope": "Python binding regression guard timings on deterministic synthetic FASTA input; not a public throughput claim",
            "surfaces": [
                "parse_fasta_records",
                "tokenize_fasta_records",
                "build_model_inputs_checked",
                "prepare_workflow_from_fasta",
                "tokenize_fasta_records dna-iupac",
                "build_model_inputs_checked dna-iupac",
                "prepare_workflow_from_fasta dna-iupac",
                "tokenize_fasta_records rna-iupac",
                "build_model_inputs_checked rna-iupac",
                "prepare_workflow_from_fasta rna-iupac",
            ],
        },
        "environment": environment(biors),
        "release_status": {
            "status": "current",
            "claim_policy": "Regression guard timings only; not a public throughput claim.",
        },
        "input": {
            "records": 512,
            "record_length": 128,
            "total_residues": 512 * 128,
            "fasta_bytes": len(input_fasta.encode()),
            "sha256": input_hash,
            "nucleotide_sha256": dna_input_hash,
            "rna_nucleotide_sha256": rna_input_hash,
        },
        "workloads": [
            timed_case("python_parse_fasta_records", parse_summary, args.loops),
            timed_case("python_tokenize_fasta_records", tokenize_summary, args.loops),
            timed_case("python_build_model_inputs_checked", model_input_summary, args.loops),
            timed_case("python_prepare_workflow_from_fasta", workflow_summary, args.loops),
            timed_case(
                "python_tokenize_fasta_records_dna_iupac",
                lambda: nucleotide_tokenize_summary(dna_fasta, "dna-iupac"),
                args.loops,
            ),
            timed_case(
                "python_build_model_inputs_checked_dna_iupac",
                lambda: nucleotide_model_input_summary(dna_tokenized, "dna-iupac"),
                args.loops,
            ),
            timed_case(
                "python_prepare_workflow_from_fasta_dna_iupac",
                lambda: nucleotide_workflow_summary(dna_fasta, "dna-iupac"),
                args.loops,
            ),
            timed_case(
                "python_tokenize_fasta_records_rna_iupac",
                lambda: nucleotide_tokenize_summary(rna_fasta, "rna-iupac"),
                args.loops,
            ),
            timed_case(
                "python_build_model_inputs_checked_rna_iupac",
                lambda: nucleotide_model_input_summary(rna_tokenized, "rna-iupac"),
                args.loops,
            ),
            timed_case(
                "python_prepare_workflow_from_fasta_rna_iupac",
                lambda: nucleotide_workflow_summary(rna_fasta, "rna-iupac"),
                args.loops,
            ),
        ],
    }

    RESULT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    REPORT_PATH.write_text(render_report(result))
    print(f"Wrote Python benchmark results to {RESULT_PATH}")
    print(f"Wrote Python benchmark report to {REPORT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
