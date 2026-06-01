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
- bio-rs WASM: `biors-wasm v0.47.4`
- Git commit: `58a7ce2e3fecd5335f2b606f4647bcd903a8d675`
- Benchmark schema: `biors.benchmark.wasm_bindings.v1`

## Methodology

Scope: Node.js WASM binding regression guard timings on deterministic synthetic FASTA input.

The script builds the WASM package with `wasm-pack --target nodejs`,
generates deterministic synthetic FASTA data, warms each exported API once,
and records timed in-process Node.js runs plus output hashes.

## Results

| Workload | Surface | Input shape | Mean | Median | Min | Max | Output size |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `wasm_parse_fasta` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.000334s | 0.000340s | 0.000311s | 0.000350s | 40,595 bytes |
| `wasm_validate_fasta` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.000639s | 0.000623s | 0.000605s | 0.000688s | 61,203 bytes |
| `wasm_tokenize` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.003131s | 0.003002s | 0.002637s | 0.003754s | 259,658 bytes |
| `wasm_run_workflow` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.010078s | 0.010086s | 0.009767s | 0.010381s | 364,030 bytes |
| `wasm_tokenize_dna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.001952s | 0.001992s | 0.001828s | 0.002035s | 226,392 bytes |
| `wasm_run_workflow_dna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.008578s | 0.008463s | 0.008373s | 0.008899s | 330,504 bytes |
| `wasm_tokenize_rna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.001767s | 0.001763s | 0.001755s | 0.001782s | 226,392 bytes |
| `wasm_run_workflow_rna_iupac` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.008435s | 0.008483s | 0.008260s | 0.008563s | 330,504 bytes |

## Reproduce

```bash
python3 scripts/benchmark_wasm_bindings.py
cat benchmarks/wasm_bindings.json
```
