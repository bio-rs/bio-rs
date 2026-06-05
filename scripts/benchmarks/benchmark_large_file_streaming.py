#!/usr/bin/env python3
"""Generate a large FASTA file and time streaming-oriented CLI paths."""

from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
import time
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--records", type=int, default=5000)
    parser.add_argument("--residues", type=int, default=400)
    parser.add_argument("--bin", default="target/release/biors")
    args = parser.parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        fasta = Path(tmp) / "large.fasta"
        write_fasta(fasta, args.records, args.residues)

        commands = {
            "inspect": [args.bin, "inspect", str(fasta)],
            "validate": [args.bin, "seq", "validate", str(fasta)],
            "tokenize": [args.bin, "tokenize", str(fasta)],
        }
        results = {
            name: run_timed(command)
            for name, command in commands.items()
        }
        print(json.dumps({
            "records": args.records,
            "residues_per_record": args.residues,
            "file_size_bytes": fasta.stat().st_size,
            "results": results,
        }, indent=2))

    return 0


def write_fasta(path: Path, records: int, residues: int) -> None:
    alphabet = "ACDEFGHIKLMNPQRSTVWY"
    sequence = (alphabet * ((residues // len(alphabet)) + 1))[:residues]
    with path.open("w", encoding="utf-8") as handle:
        for index in range(records):
            handle.write(f">seq{index}\n")
            handle.write(sequence)
            handle.write("\n")


def run_timed(command: list[str]) -> dict[str, float | int]:
    start = time.perf_counter()
    output = subprocess.run(
        command,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    elapsed = time.perf_counter() - start
    if output.returncode != 0:
        raise SystemExit(
            f"{command!r} failed with {output.returncode}: {output.stderr}"
        )
    return {
        "seconds": elapsed,
        "exit_code": output.returncode,
    }


if __name__ == "__main__":
    raise SystemExit(main())
