#!/usr/bin/env python3
"""Record WASM binding benchmark artifacts for release regression guards."""

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
    PROTEIN_ALPHABET_BYTES,
    RNA_ALPHABET_BYTES,
    UTC,
    cargo_package_version,
    command_output,
    write_fasta_bytes,
)

SCHEMA_VERSION = "biors.benchmark.wasm_bindings.v1"
RESULT_PATH = Path("benchmarks/wasm_bindings.json")
WORK_DIR = Path(".benchmark-wasm")
PKG_DIR = WORK_DIR / "pkg"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loops", type=int, default=7)
    parser.add_argument("--no-build", action="store_true")
    return parser.parse_args()


def environment() -> dict[str, str | None]:
    return {
        "os": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor() or None,
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "wasm_pack": command_output(["wasm-pack", "--version"]),
        "node": command_output(["node", "--version"]),
        "biors_wasm": cargo_package_version("biors-wasm"),
        "git_commit": command_output(["git", "rev-parse", "HEAD"]),
    }


def ensure_wasm_package(no_build: bool) -> Path:
    if not no_build:
        shutil.rmtree(WORK_DIR, ignore_errors=True)
        WORK_DIR.mkdir()
        subprocess.run(
            [
                "wasm-pack",
                "build",
                "crates/biors-wasm",
                "--target",
                "nodejs",
                "--out-dir",
                "../../../.benchmark-wasm/pkg",
                "--scope",
                "bio-rs",
                "--no-typescript",
            ],
            check=True,
        )
    if not (PKG_DIR / "biors_wasm.js").exists():
        raise FileNotFoundError(f"WASM package not found: {PKG_DIR}")
    return PKG_DIR


def write_wasm_fasta(
    path: Path,
    *,
    records: int,
    length: int,
    alphabet: bytes,
) -> dict[str, int | str]:
    info = write_fasta_bytes(path, records=records, length=length, alphabet=alphabet)
    info["path"] = str(path)
    return info


def run_node_benchmark(
    pkg_dir: Path,
    fasta_path: Path,
    dna_fasta_path: Path,
    rna_fasta_path: Path,
    input_info: dict,
    dna_input_info: dict,
    rna_input_info: dict,
    loops: int,
) -> list[dict]:
    script_path = Path(__file__).with_name("benchmark_wasm_runner.cjs")
    completed = subprocess.run(
        [
            "node",
            str(script_path),
            str((pkg_dir / "biors_wasm.js").resolve()),
            str(fasta_path.resolve()),
            str(dna_fasta_path.resolve()),
            str(rna_fasta_path.resolve()),
            str(loops),
            json.dumps(input_info),
            json.dumps(dna_input_info),
            json.dumps(rna_input_info),
        ],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return json.loads(completed.stdout)


def main() -> int:
    args = parse_args()
    pkg_dir = ensure_wasm_package(args.no_build)
    fasta_path = WORK_DIR / "wasm.fasta"
    dna_fasta_path = WORK_DIR / "wasm-dna.fasta"
    rna_fasta_path = WORK_DIR / "wasm-rna.fasta"
    input_info = write_wasm_fasta(
        fasta_path,
        records=256,
        length=128,
        alphabet=PROTEIN_ALPHABET_BYTES,
    )
    dna_input_info = write_wasm_fasta(
        dna_fasta_path,
        records=256,
        length=128,
        alphabet=DNA_ALPHABET_BYTES,
    )
    rna_input_info = write_wasm_fasta(
        rna_fasta_path,
        records=256,
        length=128,
        alphabet=RNA_ALPHABET_BYTES,
    )
    workloads = run_node_benchmark(
        pkg_dir,
        fasta_path,
        dna_fasta_path,
        rna_fasta_path,
        input_info,
        dna_input_info,
        rna_input_info,
        args.loops,
    )
    result = {
        "schema_version": SCHEMA_VERSION,
        "generated_at_utc": datetime.now(UTC).isoformat(),
        "loops": args.loops,
        "methodology": {
            "scope": "Node.js WASM binding regression guard timings on deterministic synthetic FASTA input",
            "surfaces": ["wasm_bindings"],
            "package": str(pkg_dir),
        },
        "environment": environment(),
        "release_status": {
            "status": "current",
            "claim_policy": "Regression guard timings only; not a public throughput claim.",
        },
        "workloads": workloads,
    }
    RESULT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    print(f"Wrote WASM benchmark results to {RESULT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
