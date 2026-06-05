from __future__ import annotations

from pathlib import Path

from Bio import SeqIO

ALPHABET = "ACDEFGHIKLMNPQRSTVWY"
TOKEN_SET = set(ALPHABET)
AMBIGUOUS_SET = set("XBZJUO")
UNKNOWN_TOKEN_ID = 20


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
