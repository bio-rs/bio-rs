# Python binding benchmark

This benchmark is a release regression guard for Python binding overhead on
deterministic synthetic FASTA input. It is not a public throughput claim.

## Environment

- Date: 2026-05-31 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Python: `3.14.4`
- Module: `packages/rust/biors-python/python/biors/__init__.py`
- Git commit: `6f0f6afdcd1414421cbfd5b24dd78799c1a0b217`
- Benchmark schema: `biors.benchmark.python_bindings.v1`

## Input

- Records: 512
- Record length: 128
- Total residues: 65,536
- FASTA bytes: 71,570
- FASTA SHA256: `sha256:2846469758800181ebdca708d0ce68a9036c111acb9a7418f68906112f03c95a`

## Results

| Workload | Mean | Median | Min | Max | Output hash |
| --- | ---: | ---: | ---: | ---: | --- |
| `python_parse_fasta_records` | 0.0003s | 0.0003s | 0.0002s | 0.0003s | `sha256:ea3d5c0845c2ec49b52c6f8f533e0e29c5bd2c74e2e1b344832f4872c358757c` |
| `python_tokenize_fasta_records` | 0.0007s | 0.0007s | 0.0007s | 0.0008s | `sha256:b999deadad2dccae12383a6ce9daa38a363520c24c2788ce5c70dca56a7541e1` |
| `python_build_model_inputs_checked` | 0.0013s | 0.0012s | 0.0012s | 0.0014s | `sha256:c8fb829e7ae93a64c519ee2d93844aa6b38582db9f7ffa4e52fcd7b0594c25df` |
| `python_prepare_workflow_from_fasta` | 0.0123s | 0.0122s | 0.0120s | 0.0129s | `sha256:93e2bb13201f0ecbf3b2d5001ea171234bf7fd13318cd45de399e1343a199a59` |

## Reproduce

```bash
PYTHONPATH=packages/rust/biors-python/python python3 scripts/benchmark_python_bindings.py
cat benchmarks/python_bindings.json
```
