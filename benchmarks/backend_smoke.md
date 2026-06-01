# Backend smoke benchmark

This benchmark is a release regression guard for the optional Candle CPU
backend smoke path. It is not a broad model-serving throughput claim.

## Environment

- Date: 2026-06-01 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.88.0 (6b00bc388 2025-06-23)`
- Cargo: `cargo 1.88.0 (873a06493 2025-05-10)`
- biors-backend-candle: `v0.47.5`
- Git commit: `753307b44d8cb08ef6878f733835b3b0d4954a09`
- Benchmark schema: `biors.benchmark.backend_smoke.v1`

## Methodology

Scope: Optional Candle CPU backend smoke execution on a synthetic 32x128 model-input payload.

The script runs the existing Criterion bench and exports the generated
`target/criterion` estimates into a committed release-regression artifact.

## Results

| Workload | Surface | Mean | Median | Slope | 95% CI mean |
| --- | --- | ---: | ---: | ---: | ---: |
| `candle_linear_probe_32x128_cpu` | `candle_backend_cpu_smoke` | 229.22 us | 229.10 us | 229.50 us | 228.84-229.64 us |

## Reproduce

```bash
python3 scripts/benchmark_backend_smoke.py
cat benchmarks/backend_smoke.json
```
