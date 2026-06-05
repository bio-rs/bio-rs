from __future__ import annotations

import urllib.request
from pathlib import Path

from Bio import SeqIO
from benchmark_fasta_biopython import AMBIGUOUS_SET, TOKEN_SET
from benchmark_support import raw_sha256_file

REFERENCE_PROTEOME_RELEASE = "QfO_release_2025_04"
REFERENCE_PROTEOME_SOURCE_DATE = "2025-10-25"
REFERENCE_PROTEOME_FASTA_URL = (
    "https://ftp.ebi.ac.uk/pub/databases/reference_proteomes/QfO/"
    "Eukaryota/UP000005640_9606.fasta"
)


def download_reference_human_proteome(destination_fasta: Path) -> dict[str, str]:
    urllib.request.urlretrieve(REFERENCE_PROTEOME_FASTA_URL, destination_fasta)
    return {
        "source": "EBI QfO reference proteomes",
        "proteome_id": "UP000005640",
        "taxonomy_id": "9606",
        "source_release": REFERENCE_PROTEOME_RELEASE,
        "source_date": REFERENCE_PROTEOME_SOURCE_DATE,
        "download_url": REFERENCE_PROTEOME_FASTA_URL,
        "downloaded_fasta_sha256": raw_sha256_file(destination_fasta),
    }


def ensure_large_fasta(
    source_fasta: Path, destination_fasta: Path, min_mb: int
) -> dict[str, int | str]:
    min_bytes = min_mb * 1024 * 1024
    copied = 0
    repeats = 0
    source_bytes = source_fasta.read_bytes()
    with destination_fasta.open("wb") as handle:
        while copied < min_bytes:
            handle.write(source_bytes)
            copied += len(source_bytes)
            repeats += 1

    return {
        "source": "repeated_uniprot_human_proteome",
        "base_proteome_id": "UP000005640",
        "repeat_count": repeats,
        "min_target_mb": min_mb,
    }


def read_residue_stream(source_fasta: Path) -> str:
    residues: list[str] = []
    with source_fasta.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            residues.append(str(record.seq).upper())
    return "".join(residues)


def ensure_many_short_fasta(
    source_fasta: Path,
    destination_fasta: Path,
    *,
    records: int,
    record_length: int,
) -> dict[str, int | str]:
    residues = read_residue_stream(source_fasta)
    required = records * record_length
    repeated = (residues * ((required // len(residues)) + 1))[:required]

    with destination_fasta.open("w", encoding="utf-8") as handle:
        for index in range(records):
            start = index * record_length
            end = start + record_length
            handle.write(f">short_{index}\n")
            handle.write(repeated[start:end])
            handle.write("\n")

    return {
        "source": "synthetic_many_short_records_from_uniprot_human_proteome",
        "base_proteome_id": "UP000005640",
        "shape_profile": "many_short_records",
        "record_count": records,
        "record_length": record_length,
    }


def ensure_single_long_fasta(
    source_fasta: Path,
    destination_fasta: Path,
    *,
    min_residues: int,
) -> dict[str, int | str]:
    residues = read_residue_stream(source_fasta)
    sequence = (residues * ((min_residues // len(residues)) + 1))[:min_residues]

    with destination_fasta.open("w", encoding="utf-8") as handle:
        handle.write(">single_long\n")
        for start in range(0, len(sequence), 80):
            handle.write(sequence[start : start + 80])
            handle.write("\n")

    return {
        "source": "synthetic_single_long_sequence_from_uniprot_human_proteome",
        "base_proteome_id": "UP000005640",
        "shape_profile": "single_long_sequence",
        "target_residues": min_residues,
    }


def dataset_stats(fasta_path: Path) -> dict[str, int]:
    records = 0
    total_residues = 0
    canonical_residues = 0
    ambiguous_residues = 0
    invalid_residues = 0

    with fasta_path.open("r", encoding="utf-8") as handle:
        for record in SeqIO.parse(handle, "fasta"):
            records += 1
            sequence = str(record.seq).upper()
            total_residues += len(sequence)
            for residue in sequence:
                if residue in TOKEN_SET:
                    canonical_residues += 1
                elif residue in AMBIGUOUS_SET:
                    ambiguous_residues += 1
                else:
                    invalid_residues += 1

    return {
        "records": records,
        "total_residues": total_residues,
        "canonical_residues": canonical_residues,
        "ambiguous_residues": ambiguous_residues,
        "invalid_residues": invalid_residues,
        "file_size_bytes": fasta_path.stat().st_size,
    }


def recorded_dataset_path(fasta_path: Path, provenance: dict) -> str:
    if provenance.get("source") == "user-provided FASTA":
        return str(fasta_path)
    return fasta_path.name


def add_base_provenance(
    provenance: dict[str, int | str],
    base_provenance: dict[str, str],
    base_fasta: Path,
) -> None:
    provenance["base_source_release"] = base_provenance.get("source_release", "user_provided")
    if "source_date" in base_provenance:
        provenance["base_source_date"] = base_provenance["source_date"]
    provenance["base_fasta_sha256"] = raw_sha256_file(base_fasta)
