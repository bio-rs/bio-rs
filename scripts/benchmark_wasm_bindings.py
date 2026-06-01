#!/usr/bin/env python3
"""Record WASM binding benchmark artifacts for release regression guards."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shutil
import subprocess
from datetime import UTC, datetime
from pathlib import Path

from render_wasm_benchmark_report import render_report

SCHEMA_VERSION = "biors.benchmark.wasm_bindings.v1"
RESULT_PATH = Path("benchmarks/wasm_bindings.json")
REPORT_PATH = Path("benchmarks/wasm_bindings.md")
WORK_DIR = Path(".benchmark-wasm")
PKG_DIR = WORK_DIR / "pkg"
ALPHABET = b"ACDEFGHIKLMNPQRSTVWY"
DNA_ALPHABET = b"ACGT"
RNA_ALPHABET = b"ACGU"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loops", type=int, default=7)
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


def cargo_package_version(package_name: str) -> str | None:
    output = command_output(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    if output is None:
        return None
    for package in json.loads(output).get("packages", []):
        if package.get("name") == package_name:
            return str(package.get("version"))
    return None


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
                "packages/rust/biors-wasm",
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
            handle.write(seq)
            handle.write(b"\n")
    return {
        "path": str(path),
        "records": records,
        "total_residues": residues,
        "file_size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return f"sha256:{hasher.hexdigest()}"


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
    script_path = WORK_DIR / "runner.cjs"
    script_path.write_text(NODE_RUNNER)
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


NODE_RUNNER = r"""
const crypto = require("crypto");
const fs = require("fs");
const wasm = require(process.argv[2]);
const fasta = fs.readFileSync(process.argv[3]);
const dnaFasta = fs.readFileSync(process.argv[4]);
const rnaFasta = fs.readFileSync(process.argv[5]);
const loops = Number(process.argv[6]);
const input = JSON.parse(process.argv[7]);
const dnaInput = JSON.parse(process.argv[8]);
const rnaInput = JSON.parse(process.argv[9]);

const parsedRecords = wasm.parseFasta(fasta);
const tokenizedRecords = wasm.tokenize(parsedRecords, "protein-20");
const parsedDnaRecords = wasm.parseFasta(dnaFasta);
const parsedRnaRecords = wasm.parseFasta(rnaFasta);
const workflowConfig = {
  fastaBytes: Uint8Array.from(fasta),
  maxLength: 160,
  padding: "fixed_length",
  padTokenId: 0,
};
const dnaWorkflowConfig = {
  fastaBytes: Uint8Array.from(dnaFasta),
  kind: "dna",
  profile: "dna-iupac",
  maxLength: 160,
  padding: "fixed_length",
  padTokenId: 0,
};
const rnaWorkflowConfig = {
  fastaBytes: Uint8Array.from(rnaFasta),
  kind: "rna",
  profile: "rna-iupac",
  maxLength: 160,
  padding: "fixed_length",
  padTokenId: 0,
};

function hash(value) {
  return `sha256:${crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
}

function timed(name, fn, workloadInput = input) {
  fn();
  const seconds = [];
  let output;
  for (let index = 0; index < loops; index += 1) {
    const start = process.hrtime.bigint();
    output = fn();
    seconds.push(Number(process.hrtime.bigint() - start) / 1e9);
  }
  seconds.sort((a, b) => a - b);
  const mean = seconds.reduce((sum, value) => sum + value, 0) / seconds.length;
  return {
    name,
    surface: "wasm_bindings",
    input: workloadInput,
    summary: {
      mean_s: mean,
      median_s: seconds[Math.floor(seconds.length / 2)],
      min_s: seconds[0],
      max_s: seconds[seconds.length - 1],
      output_hash: hash(output),
      output_bytes: Buffer.byteLength(JSON.stringify(output)),
    },
  };
}

process.stdout.write(JSON.stringify([
  timed("wasm_parse_fasta", () => wasm.parseFasta(fasta)),
  timed("wasm_validate_fasta", () => wasm.validateFasta(fasta, "protein")),
  timed("wasm_tokenize", () => wasm.tokenize(parsedRecords, "protein-20")),
  timed("wasm_run_workflow", () => wasm.runWorkflow(workflowConfig)),
  timed("wasm_tokenize_dna_iupac", () => wasm.tokenize(parsedDnaRecords, "dna-iupac"), dnaInput),
  timed("wasm_run_workflow_dna_iupac", () => wasm.runWorkflow(dnaWorkflowConfig), dnaInput),
  timed("wasm_tokenize_rna_iupac", () => wasm.tokenize(parsedRnaRecords, "rna-iupac"), rnaInput),
  timed("wasm_run_workflow_rna_iupac", () => wasm.runWorkflow(rnaWorkflowConfig), rnaInput),
]));
"""


def main() -> int:
    args = parse_args()
    pkg_dir = ensure_wasm_package(args.no_build)
    fasta_path = WORK_DIR / "wasm.fasta"
    dna_fasta_path = WORK_DIR / "wasm-dna.fasta"
    rna_fasta_path = WORK_DIR / "wasm-rna.fasta"
    input_info = write_fasta(fasta_path, records=256, length=128)
    dna_input_info = write_fasta(
        dna_fasta_path,
        records=256,
        length=128,
        alphabet=DNA_ALPHABET,
    )
    rna_input_info = write_fasta(
        rna_fasta_path,
        records=256,
        length=128,
        alphabet=RNA_ALPHABET,
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
        "workloads": workloads,
    }
    RESULT_PATH.write_text(json.dumps(result, indent=2) + "\n")
    REPORT_PATH.write_text(render_report(result))
    print(f"Wrote WASM benchmark results to {RESULT_PATH}")
    print(f"Wrote WASM benchmark report to {REPORT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
