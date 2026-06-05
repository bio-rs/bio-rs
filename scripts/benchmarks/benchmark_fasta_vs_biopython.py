#!/usr/bin/env python3
"""Reproducible core FASTA benchmark (biors-core vs Biopython)."""

from __future__ import annotations

import argparse
import json
import tempfile
from datetime import datetime
from pathlib import Path

from benchmark_fasta_inputs import (
    add_base_provenance,
    download_reference_human_proteome,
    ensure_large_fasta,
    ensure_many_short_fasta,
    ensure_single_long_fasta,
)
from benchmark_fasta_runner import (
    benchmark_environment,
    dataset_report,
    ensure_benchmark_harness,
)
from benchmark_feature_coverage import FEATURE_COVERAGE
from benchmark_support import UTC
from render_benchmark_report import render_report

BENCHMARK_SCHEMA_VERSION = "biors.benchmark.fasta_vs_biopython.v1"


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
    parser.add_argument("--loops", type=int, default=15)
    parser.add_argument("--large-min-mb", type=int, default=110)
    parser.add_argument("--shape-profile-records", type=int, default=20000)
    parser.add_argument("--short-record-length", type=int, default=48)
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        human_fasta, human_provenance = prepare_reference_input(args, tmp_path)
        large_fasta, large_provenance = prepare_large_input(args, tmp_path, human_fasta, human_provenance)
        many_short_fasta, many_short_provenance = prepare_many_short_input(args, tmp_path, human_fasta, human_provenance)
        single_long_fasta, single_long_provenance = prepare_single_long_input(args, tmp_path, human_fasta, human_provenance)
        harness = ensure_benchmark_harness()

        result = {
            "schema_version": BENCHMARK_SCHEMA_VERSION,
            "generated_at_utc": datetime.now(UTC).isoformat(),
            "loops": args.loops,
            "methodology": methodology(),
            "environment": benchmark_environment(),
            "feature_coverage": FEATURE_COVERAGE,
            "datasets": [
                dataset_report("human_reference_proteome", human_fasta, human_provenance, args.loops, harness),
                dataset_report("large_scale_fasta", large_fasta, large_provenance, args.loops, harness),
                dataset_report("many_short_records", many_short_fasta, many_short_provenance, args.loops, harness),
                dataset_report("single_long_sequence", single_long_fasta, single_long_provenance, args.loops, harness),
            ],
        }

    output_path = Path("benchmarks") / "fasta_vs_biopython.json"
    report_path = Path("benchmarks") / "fasta_vs_biopython.md"
    output_path.write_text(json.dumps(result, indent=2))
    report_path.write_text(render_report(result))
    print(f"Wrote benchmark results to {output_path}")
    print(f"Wrote benchmark report to {report_path}")
    return 0


def prepare_reference_input(args: argparse.Namespace, tmp_path: Path) -> tuple[Path, dict]:
    if args.input is None:
        human_fasta = tmp_path / "UP000005640_9606.fasta"
        human_provenance = download_reference_human_proteome(human_fasta)
    else:
        human_fasta = args.input
        human_provenance = {"source": "user-provided FASTA", "path_hint": str(human_fasta)}
    require_existing_fasta(human_fasta, "FASTA")
    return human_fasta, human_provenance


def prepare_large_input(
    args: argparse.Namespace,
    tmp_path: Path,
    human_fasta: Path,
    human_provenance: dict,
) -> tuple[Path, dict]:
    if args.large_input is None:
        large_fasta = tmp_path / f"human_proteome_x{args.large_min_mb}.fasta"
        provenance = ensure_large_fasta(human_fasta, large_fasta, args.large_min_mb)
        provenance["shape_profile"] = "large_repeated_proteome"
        add_base_provenance(provenance, human_provenance, human_fasta)
    else:
        large_fasta = args.large_input
        provenance = {
            "source": "user-provided FASTA",
            "path_hint": str(large_fasta),
            "shape_profile": "user_provided_large",
        }
    require_existing_fasta(large_fasta, "Large FASTA")
    return large_fasta, provenance


def prepare_many_short_input(
    args: argparse.Namespace,
    tmp_path: Path,
    human_fasta: Path,
    human_provenance: dict,
) -> tuple[Path, dict]:
    fasta = tmp_path / "many_short_records.fasta"
    provenance = ensure_many_short_fasta(
        human_fasta,
        fasta,
        records=args.shape_profile_records,
        record_length=args.short_record_length,
    )
    add_base_provenance(provenance, human_provenance, human_fasta)
    return fasta, provenance


def prepare_single_long_input(
    args: argparse.Namespace,
    tmp_path: Path,
    human_fasta: Path,
    human_provenance: dict,
) -> tuple[Path, dict]:
    fasta = tmp_path / "single_long_sequence.fasta"
    provenance = ensure_single_long_fasta(
        human_fasta,
        fasta,
        min_residues=args.shape_profile_records * args.short_record_length,
    )
    add_base_provenance(provenance, human_provenance, human_fasta)
    return fasta, provenance


def require_existing_fasta(path: Path, label: str) -> None:
    if not path.exists():
        raise FileNotFoundError(f"{label} not found: {path}")


def methodology() -> dict:
    return {
        "scope": "core library FASTA throughput, excluding CLI startup and success-envelope JSON serialization",
        "workloads": ["pure_parse", "parse_plus_validation", "parse_plus_tokenization"],
        "shape_profiles": [
            "human_reference_proteome",
            "large_repeated_proteome",
            "many_short_records",
            "single_long_sequence",
        ],
        "memory": "best-effort peak RSS from /usr/bin/time for biors-core and Biopython subprocesses",
    }


if __name__ == "__main__":
    raise SystemExit(main())
