# CLI surface benchmark

This benchmark is a release regression guard for CLI overhead on deterministic
synthetic FASTA inputs. It is not a public throughput claim.

## Environment

- Date: 2026-05-31 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- bio-rs CLI: `biors v0.47.4`
- Python: `3.14.4`
- Git commit: `59e7f7a53544dd6237c75b7f2277c06c837a451c`
- Benchmark schema: `biors.benchmark.cli_surfaces.v1`

## Methodology

Scope: CLI regression guard timings on deterministic synthetic FASTA inputs; not a public throughput claim.

The script builds the release CLI binary, generates deterministic synthetic
FASTA inputs in a temporary directory, warms each command once, and records
timed subprocess runs plus canonical JSON output hashes.

## Results

| Workload | Surface | Input shape | Mean | Median | Min | Max | Output size |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `cli_workflow_fixed_length` | `cli_workflow` | 256 records, 32,768 residues, 35,730 bytes | 0.0795s | 0.0764s | 0.0756s | 0.0928s | 1,930,154 bytes |
| `cli_dataset_inspect_many_file` | `cli_dataset_inspect` | 32 files, 256 records, 24,576 residues, 26,880 bytes | 0.0081s | 0.0081s | 0.0076s | 0.0085s | 110,161 bytes |

## Reproduce

```bash
python3 scripts/benchmark_cli_surfaces.py
cat benchmarks/cli_surfaces.json
```
