# WASM binding benchmark

This benchmark is a release regression guard for Node.js-loaded WASM
bindings. It is not a browser or public throughput claim.

## Environment

- Date: 2026-06-01 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.88.0 (6b00bc388 2025-06-23)`
- Cargo: `cargo 1.88.0 (873a06493 2025-05-10)`
- wasm-pack: `wasm-pack 0.15.0`
- Node.js: `v25.2.1`
- bio-rs WASM: `biors-wasm v0.47.8`
- Git commit: `753307b44d8cb08ef6878f733835b3b0d4954a09`
- Benchmark schema: `biors.benchmark.wasm_bindings.v1`

## Methodology

Scope: Node.js WASM binding regression guard timings on deterministic synthetic FASTA input.

The script builds the WASM package with `wasm-pack --target nodejs`,
generates deterministic synthetic FASTA data, warms each exported API once,
and records timed in-process Node.js runs plus output hashes.

## Results

| Workload | Surface | Input shape | Mean | Median | Min | Max | Output size |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `wasm_parse_fasta` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.000294s | 0.000293s | 0.000280s | 0.000308s | 40,595 bytes |
| `wasm_validate_fasta` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.000623s | 0.000611s | 0.000588s | 0.000669s | 61,203 bytes |
| `wasm_tokenize` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.002534s | 0.002592s | 0.002295s | 0.002716s | 259,658 bytes |
| `wasm_run_workflow` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.010132s | 0.010092s | 0.010037s | 0.010267s | 364,030 bytes |
| `wasm_tokenize_dna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.001871s | 0.001871s | 0.001867s | 0.001875s | 226,392 bytes |
| `wasm_run_workflow_dna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.008366s | 0.008341s | 0.008273s | 0.008484s | 330,504 bytes |
| `wasm_tokenize_rna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.001701s | 0.001700s | 0.001699s | 0.001706s | 226,392 bytes |
| `wasm_run_workflow_rna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.008596s | 0.008550s | 0.008529s | 0.008710s | 330,504 bytes |

## Reproduce

```bash
python3 scripts/benchmarks/benchmark_wasm_bindings.py
cat benchmarks/wasm_bindings.json
```
