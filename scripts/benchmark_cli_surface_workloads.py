from __future__ import annotations

from pathlib import Path

from benchmark_support import sha256_file, timed_command


def build_workloads(
    *,
    binary: Path,
    loops: int,
    workflow_fasta: Path,
    workflow_input: dict,
    dna_fasta: Path,
    dna_input: dict,
    rna_fasta: Path,
    rna_input: dict,
    dataset_dir: Path,
    dataset_input: dict,
    package_manifest: Path,
) -> list[dict]:
    package_input = file_input(package_manifest, "package_manifest")
    return [
        workload(
            binary,
            loops,
            "cli_workflow_fixed_length",
            "cli_workflow",
            workflow_input,
            ["workflow", "--max-length", "160", str(workflow_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_seq_validate_dna",
            "nucleotide_validation",
            dna_input,
            ["seq", "validate", "--kind", "dna", str(dna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_tokenize_dna_iupac",
            "nucleotide_tokenization",
            dna_input,
            ["tokenize", "--profile", "dna-iupac", str(dna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_model_input_dna_iupac",
            "nucleotide_model_input",
            dna_input,
            ["model-input", "--profile", "dna-iupac", "--max-length", "160", str(dna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_workflow_dna_iupac",
            "nucleotide_workflow",
            dna_input,
            ["workflow", "--profile", "dna-iupac", "--max-length", "160", str(dna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_seq_validate_rna",
            "nucleotide_validation",
            rna_input,
            ["seq", "validate", "--kind", "rna", str(rna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_tokenize_rna_iupac",
            "nucleotide_tokenization",
            rna_input,
            ["tokenize", "--profile", "rna-iupac", str(rna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_model_input_rna_iupac",
            "nucleotide_model_input",
            rna_input,
            ["model-input", "--profile", "rna-iupac", "--max-length", "160", str(rna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_workflow_rna_iupac",
            "nucleotide_workflow",
            rna_input,
            ["workflow", "--profile", "rna-iupac", "--max-length", "160", str(rna_fasta)],
        ),
        workload(
            binary,
            loops,
            "cli_dataset_inspect_many_file",
            "cli_dataset_inspect",
            dataset_input,
            [
                "dataset",
                "inspect",
                "--source",
                "synthetic",
                "--version",
                "benchmark",
                "--split",
                "train",
                str(dataset_dir),
            ],
        ),
        workload(
            binary,
            loops,
            "cli_service_contract",
            "service_contract",
            {"kind": "no_input"},
            ["service", "contract"],
        ),
        workload(
            binary,
            loops,
            "cli_package_validate_example",
            "package_validation",
            package_input,
            ["package", "validate", str(package_manifest)],
        ),
        workload(
            binary,
            loops,
            "cli_package_bridge_example",
            "package_bridge",
            package_input,
            ["package", "bridge", str(package_manifest)],
        ),
    ]


def file_input(path: Path, kind: str) -> dict[str, str | int]:
    return {
        "kind": kind,
        "path": str(path),
        "file_size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def workload(
    binary: Path,
    loops: int,
    name: str,
    surface: str,
    input_: dict,
    args: list[str],
) -> dict:
    return {
        "name": name,
        "surface": surface,
        "input": input_,
        "result": timed_command([str(binary), *args], loops),
    }
