# CLI surface benchmark

This benchmark is a release regression guard for CLI overhead on deterministic
synthetic inputs and package fixtures. It is not a public throughput claim.

## Environment

- Date: 2026-06-01 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.88.0 (6b00bc388 2025-06-23)`
- Cargo: `cargo 1.88.0 (873a06493 2025-05-10)`
- bio-rs CLI: `biors v0.47.7`
- Python: `3.14.4`
- Git commit: `753307b44d8cb08ef6878f733835b3b0d4954a09`
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
| `cli_workflow_fixed_length` | `cli_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0758s | 0.0755s | 0.0755s | 0.0763s | 1,930,201 bytes |
| `cli_seq_validate_dna` | `nucleotide_validation` | 256 records, 32,768 residues, 35,730 bytes | 0.0059s | 0.0059s | 0.0056s | 0.0063s | 81,602 bytes |
| `cli_tokenize_dna_iupac` | `nucleotide_tokenization` | 256 records, 32,768 residues, 35,730 bytes | 0.0249s | 0.0228s | 0.0221s | 0.0299s | 405,500 bytes |
| `cli_model_input_dna_iupac` | `nucleotide_model_input` | 256 records, 32,768 residues, 35,730 bytes | 0.0478s | 0.0473s | 0.0468s | 0.0495s | 1,100,924 bytes |
| `cli_workflow_dna_iupac` | `nucleotide_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0754s | 0.0755s | 0.0748s | 0.0759s | 1,896,670 bytes |
| `cli_seq_validate_rna` | `nucleotide_validation` | 256 records, 32,768 residues, 35,730 bytes | 0.0065s | 0.0060s | 0.0057s | 0.0078s | 81,602 bytes |
| `cli_tokenize_rna_iupac` | `nucleotide_tokenization` | 256 records, 32,768 residues, 35,730 bytes | 0.0226s | 0.0226s | 0.0221s | 0.0230s | 405,500 bytes |
| `cli_model_input_rna_iupac` | `nucleotide_model_input` | 256 records, 32,768 residues, 35,730 bytes | 0.0485s | 0.0485s | 0.0484s | 0.0486s | 1,100,924 bytes |
| `cli_workflow_rna_iupac` | `nucleotide_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0745s | 0.0745s | 0.0738s | 0.0751s | 1,896,670 bytes |
| `cli_dataset_inspect_many_file` | `cli_dataset_inspect` | 32 files, 256 records, 24,576 residues, 26,880 bytes | 0.0080s | 0.0080s | 0.0079s | 0.0080s | 110,264 bytes |
| `cli_service_contract` | `service_contract` | no input | 0.0042s | 0.0041s | 0.0040s | 0.0044s | 4,786 bytes |
| `cli_package_validate_example` | `package_validation` | 3,540 bytes | 0.0049s | 0.0049s | 0.0046s | 0.0052s | 128 bytes |
| `cli_package_bridge_example` | `package_bridge` | 3,540 bytes | 0.0049s | 0.0049s | 0.0048s | 0.0051s | 1,810 bytes |

## Reproduce

```bash
python3 scripts/benchmark_cli_surfaces.py
cat benchmarks/cli_surfaces.json
```
