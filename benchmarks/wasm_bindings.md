# WASM binding benchmark

This benchmark is a release regression guard for Node.js-loaded WASM
bindings. It is not a browser or public throughput claim.

## Environment

- Date: 2026-05-31 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- wasm-pack: `wasm-pack 0.15.0`
- Node.js: `v25.2.1`
- bio-rs WASM: `biors-wasm v0.47.4`
- Git commit: `984af679e4e13eb2e90fce876c20a4c2b4945516`
- Benchmark schema: `biors.benchmark.wasm_bindings.v1`

## Methodology

Scope: Node.js WASM binding regression guard timings on deterministic synthetic FASTA input.

The script builds the WASM package with `wasm-pack --target nodejs`,
generates deterministic synthetic FASTA data, warms each exported API once,
and records timed in-process Node.js runs plus output hashes.

## Results

| Workload | Surface | Input shape | Mean | Median | Min | Max | Output size |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `wasm_parse_fasta` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.000335s | 0.000333s | 0.000286s | 0.000414s | 40,595 bytes |
| `wasm_validate_fasta` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.000657s | 0.000680s | 0.000576s | 0.000710s | 61,203 bytes |
| `wasm_tokenize` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.002428s | 0.002432s | 0.002316s | 0.002571s | 259,658 bytes |
| `wasm_run_workflow` | `wasm_bindings` | 256 records, 32,768 residues, 35,218 bytes | 0.009224s | 0.009086s | 0.009029s | 0.009588s | 363,996 bytes |

## Reproduce

```bash
python3 scripts/benchmark_wasm_bindings.py
cat benchmarks/wasm_bindings.json
```
