# CLI surface benchmark

This benchmark is a release regression guard for CLI overhead on deterministic
synthetic inputs and package fixtures. It is not a public throughput claim.

## Environment

- Date: 2026-05-31 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- bio-rs CLI: `biors v0.47.4`
- Python: `3.14.4`
- Git commit: `aea31b6124004627e52dcab252e8723991f1dcb3`
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
| `cli_workflow_fixed_length` | `cli_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0816s | 0.0795s | 0.0781s | 0.0864s | 1,930,154 bytes |
| `cli_dataset_inspect_many_file` | `cli_dataset_inspect` | 32 files, 256 records, 24,576 residues, 26,880 bytes | 0.0083s | 0.0082s | 0.0081s | 0.0087s | 110,161 bytes |
| `cli_service_contract` | `service_contract` | no input | 0.0046s | 0.0046s | 0.0045s | 0.0047s | 4,786 bytes |
| `cli_package_validate_example` | `package_validation` | 3,504 bytes | 0.0047s | 0.0046s | 0.0045s | 0.0051s | 128 bytes |
| `cli_package_bridge_example` | `package_bridge` | 3,504 bytes | 0.0047s | 0.0048s | 0.0043s | 0.0050s | 1,508 bytes |

## Reproduce

```bash
python3 scripts/benchmark_cli_surfaces.py
cat benchmarks/cli_surfaces.json
```
