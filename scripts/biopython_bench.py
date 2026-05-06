#!/usr/bin/env python3
"""Biopython benchmark companion for hyperfine cross-language comparison.

Usage:
    python scripts/biopython_bench.py <parse|validate|tokenize> <input.fasta>

Outputs JSON to stdout with the same shape as benchmark_fasta.rs for fair comparison.
"""

import json
import sys
from pathlib import Path

from Bio import SeqIO

ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
TOKEN_SET = set(ALPHABET)
AMBIGUOUS_SET = set("XBZJUO")


def biopython_parse_only(fasta_path: Path) -> dict:
    records = 0
    residues = 0
    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            records += 1
            residues += len(record.seq)
    return {
        "mode": "parse",
        "records": records,
        "residues": residues,
        "canonical_tokens": 0,
        "unknown_tokens": 0,
        "warning_count": 0,
        "error_count": 0,
    }


def biopython_parse_validate(fasta_path: Path) -> dict:
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
        "mode": "validate",
        "records": records,
        "residues": residues,
        "canonical_tokens": canonical,
        "unknown_tokens": warnings + errors,
        "warning_count": warnings,
        "error_count": errors,
    }


def biopython_parse_tokenize(fasta_path: Path) -> dict:
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
        "mode": "tokenize",
        "records": records,
        "residues": residues,
        "canonical_tokens": canonical,
        "unknown_tokens": unknown,
        "warning_count": warnings,
        "error_count": errors,
    }


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: python biopython_bench.py <parse|validate|tokenize> <input.fasta>", file=sys.stderr)
        return 1

    mode = sys.argv[1]
    fasta_path = Path(sys.argv[2])

    result = {
        "parse": biopython_parse_only,
        "validate": biopython_parse_validate,
        "tokenize": biopython_parse_tokenize,
    }.get(mode)

    if result is None:
        print(f"unknown mode: {mode}", file=sys.stderr)
        return 1

    print(json.dumps(result(fasta_path), sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
