# CLI surface benchmark

This benchmark is a release regression guard for CLI overhead on deterministic
synthetic inputs and package fixtures. It is not a public throughput claim.

## Environment

- Date: 2026-06-01 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.88.0 (6b00bc388 2025-06-23)`
- Cargo: `cargo 1.88.0 (873a06493 2025-05-10)`
- bio-rs CLI: `biors v0.47.4`
- Python: `3.14.4`
- Git commit: `58a7ce2e3fecd5335f2b606f4647bcd903a8d675`
- Benchmark schema: `biors.benchmark.cli_surfaces.v1`

## Methodology

Scope: CLI regression guard timings on deterministic synthetic inputs and package fixtures; not a public throughput claim.

The script builds the release CLI binary, generates deterministic synthetic
FASTA inputs in a temporary directory, reuses the committed package fixture,
warms each command once, and records timed subprocess runs plus canonical JSON
output hashes.

## Results

| Workload | Surface | Input shape | Mean | Median | Min | Max | Output size |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `cli_workflow_fixed_length` | `cli_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0950s | 0.0930s | 0.0856s | 0.1064s | 1,930,201 bytes |
| `cli_seq_validate_dna` | `nucleotide_validation` | 256 records, 32,768 residues, 35,730 bytes | 0.0059s | 0.0058s | 0.0057s | 0.0062s | 81,602 bytes |
| `cli_tokenize_dna_iupac` | `nucleotide_tokenization` | 256 records, 32,768 residues, 35,730 bytes | 0.0234s | 0.0225s | 0.0224s | 0.0255s | 405,500 bytes |
| `cli_model_input_dna_iupac` | `nucleotide_model_input` | 256 records, 32,768 residues, 35,730 bytes | 0.0504s | 0.0511s | 0.0486s | 0.0514s | 1,100,924 bytes |
| `cli_workflow_dna_iupac` | `nucleotide_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0774s | 0.0772s | 0.0760s | 0.0791s | 1,896,670 bytes |
| `cli_seq_validate_rna` | `nucleotide_validation` | 256 records, 32,768 residues, 35,730 bytes | 0.0063s | 0.0062s | 0.0060s | 0.0066s | 81,602 bytes |
| `cli_tokenize_rna_iupac` | `nucleotide_tokenization` | 256 records, 32,768 residues, 35,730 bytes | 0.0231s | 0.0230s | 0.0227s | 0.0234s | 405,500 bytes |
| `cli_model_input_rna_iupac` | `nucleotide_model_input` | 256 records, 32,768 residues, 35,730 bytes | 0.0498s | 0.0500s | 0.0484s | 0.0509s | 1,100,924 bytes |
| `cli_workflow_rna_iupac` | `nucleotide_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0763s | 0.0761s | 0.0756s | 0.0774s | 1,896,670 bytes |
| `cli_dataset_inspect_many_file` | `cli_dataset_inspect` | 32 files, 256 records, 24,576 residues, 26,880 bytes | 0.0087s | 0.0087s | 0.0086s | 0.0087s | 110,264 bytes |
| `cli_service_contract` | `service_contract` | no input | 0.0046s | 0.0046s | 0.0046s | 0.0047s | 4,786 bytes |
| `cli_package_validate_example` | `package_validation` | 3,504 bytes | 0.0054s | 0.0055s | 0.0051s | 0.0057s | 128 bytes |
| `cli_package_bridge_example` | `package_bridge` | 3,504 bytes | 0.0052s | 0.0052s | 0.0051s | 0.0054s | 1,810 bytes |

## Reproduce

```bash
python3 scripts/benchmark_cli_surfaces.py
cat benchmarks/cli_surfaces.json
```
