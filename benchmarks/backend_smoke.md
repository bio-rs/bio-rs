# Backend smoke benchmark

This benchmark is a release regression guard for the optional Candle CPU
backend smoke path. It is not a broad model-serving throughput claim.

## Environment

- Date: 2026-05-31 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- biors-backend-candle: `v0.47.4`
- Git commit: `ad33d9fc8a5ce5b69d2df63b06b93114b6eee598`
- Benchmark schema: `biors.benchmark.backend_smoke.v1`

## Methodology

Scope: Optional Candle CPU backend smoke execution on a synthetic 32x128 model-input payload.

The script runs the existing Criterion bench and exports the generated
`target/criterion` estimates into a committed release-regression artifact.

## Results

| Workload | Surface | Mean | Median | Slope | 95% CI mean |
| --- | --- | ---: | ---: | ---: | ---: |
| `candle_linear_probe_32x128_cpu` | `candle_backend_cpu_smoke` | 233.82 us | 232.15 us | 232.18 us | 231.73-237.46 us |

## Reproduce

```bash
python3 scripts/benchmark_backend_smoke.py
cat benchmarks/backend_smoke.json
```
