#!/usr/bin/env python3
"""Record Python binding benchmark artifacts for release regression guards."""

from __future__ import annotations

import argparse
import importlib
import json
import platform
import sys
from datetime import datetime
from pathlib import Path

from benchmark_support import (
    DNA_ALPHABET,
    RNA_ALPHABET,
    UTC,
    cargo_package_version,
    command_output,
    fasta_text,
    sha256_text,
    timed_case,
)
from render_python_benchmark_report import render_report

SCHEMA_VERSION = "biors.benchmark.python_bindings.v1"
RESULT_PATH = Path("benchmarks/python_bindings.json")
REPORT_PATH = Path("benchmarks/python_bindings.md")
PYTHON_PACKAGE_PATH = Path("crates/biors-python/python")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loops", type=int, default=25)
    parser.add_argument("--pythonpath", type=Path, default=PYTHON_PACKAGE_PATH)
    return parser.parse_args()


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


def main() -> int:
    args = parse_args()
    biors = load_biors(args.pythonpath)
    input_fasta = fasta_text(records=512, length=128)
    dna_fasta = fasta_text(records=512, length=128, alphabet=DNA_ALPHABET)
    rna_fasta = fasta_text(records=512, length=128, alphabet=RNA_ALPHABET)
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
