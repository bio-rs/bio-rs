# MCP server benchmark

This benchmark is a release regression guard for MCP request overhead over
an in-process duplex transport. It is not a network throughput claim.

## Environment

- Date: 2026-05-31 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- biors-mcp-server: `v0.47.4`
- Git commit: `64d4cd4f56a95c2053303755238ea78406b05a1f`
- Benchmark schema: `biors.benchmark.mcp_server.v1`

## Methodology

Scope: MCP doctor tool request over rmcp client/server duplex transport.

The script runs the existing Criterion bench and exports the generated
`target/criterion` estimates into a committed release-regression artifact.

## Results

| Workload | Surface | Mean | Median | Slope | 95% CI mean |
| --- | --- | ---: | ---: | ---: | ---: |
| `mcp_doctor_request_duplex` | `mcp_server_request_overhead` | 54.21 us | 54.06 us | 54.24 us | 53.88-54.60 us |

## Reproduce

```bash
python3 scripts/benchmark_mcp_server.py
cat benchmarks/mcp_server.json
```
