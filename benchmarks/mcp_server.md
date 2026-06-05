# MCP server benchmark

This benchmark is a release regression guard for MCP request overhead over
an in-process duplex transport. It is not a network throughput claim.

## Environment

- Date: 2026-06-01 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.88.0 (6b00bc388 2025-06-23)`
- Cargo: `cargo 1.88.0 (873a06493 2025-05-10)`
- biors-mcp-server: `v0.47.8`
- Git commit: `753307b44d8cb08ef6878f733835b3b0d4954a09`
- Benchmark schema: `biors.benchmark.mcp_server.v1`

## Methodology

Scope: MCP doctor tool request over rmcp client/server duplex transport.

The script runs the existing Criterion bench and exports the generated
`target/criterion` estimates into a committed release-regression artifact.

## Results

| Workload | Surface | Mean | Median | Slope | 95% CI mean |
| --- | --- | ---: | ---: | ---: | ---: |
| `mcp_doctor_request_duplex` | `mcp_server_request_overhead` | 60.14 us | 61.09 us | 60.55 us | 58.57-61.56 us |

## Reproduce

```bash
python3 scripts/benchmarks/benchmark_mcp_server.py
cat benchmarks/mcp_server.json
```
