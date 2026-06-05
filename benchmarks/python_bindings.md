# Python binding benchmark

This benchmark is a release regression guard for Python binding overhead on
deterministic synthetic FASTA input. It is not a public throughput claim.

## Environment

- Date: 2026-06-01 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- Machine: `arm64`
- Python: `3.14.4`
- Module: `crates/biors-python/python/biors/__init__.py`
- Git commit: `58a7ce2e3fecd5335f2b606f4647bcd903a8d675`
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
| `python_parse_fasta_records` | 0.0023s | 0.0018s | 0.0017s | 0.0045s | `sha256:ea3d5c0845c2ec49b52c6f8f533e0e29c5bd2c74e2e1b344832f4872c358757c` |
| `python_tokenize_fasta_records` | 0.0095s | 0.0082s | 0.0075s | 0.0121s | `sha256:b999deadad2dccae12383a6ce9daa38a363520c24c2788ce5c70dca56a7541e1` |
| `python_build_model_inputs_checked` | 0.0099s | 0.0099s | 0.0097s | 0.0101s | `sha256:c8fb829e7ae93a64c519ee2d93844aa6b38582db9f7ffa4e52fcd7b0594c25df` |
| `python_prepare_workflow_from_fasta` | 0.1312s | 0.1291s | 0.1283s | 0.1405s | `sha256:93e2bb13201f0ecbf3b2d5001ea171234bf7fd13318cd45de399e1343a199a59` |
| `python_tokenize_fasta_records_dna_iupac` | 0.0072s | 0.0072s | 0.0071s | 0.0073s | `sha256:8083a02a3638ba6e318fae56a440fe6dd4093158ebeac60f53766262d637e136` |
| `python_build_model_inputs_checked_dna_iupac` | 0.0103s | 0.0103s | 0.0102s | 0.0103s | `sha256:d9410ec33e94deb2e59a83e21b47c242d49a42ac56b9afad295f640c0bd98d4c` |
| `python_prepare_workflow_from_fasta_dna_iupac` | 0.1210s | 0.1192s | 0.1184s | 0.1294s | `sha256:25f4e8315da92dd5575f4af1703ddc8f1b45500ad15a641baf76b1dd2bfc0190` |
| `python_tokenize_fasta_records_rna_iupac` | 0.0075s | 0.0075s | 0.0071s | 0.0080s | `sha256:91be069bda3254f22b8863738c88c3d3062255e3e2dd20172dd9a403fbb73205` |
| `python_build_model_inputs_checked_rna_iupac` | 0.0101s | 0.0101s | 0.0099s | 0.0103s | `sha256:5a3b76c6589f01a611815347659d232a449c8dcf70ca2ebd0ef2196312444b6e` |
| `python_prepare_workflow_from_fasta_rna_iupac` | 0.1238s | 0.1196s | 0.1187s | 0.1379s | `sha256:b04992ab78a61f00132bfb6d10dab885958ec6a9ce1f6c20e9bbbcfd14d79376` |

## Reproduce

```bash
PYTHONPATH=crates/biors-python/python python3 scripts/benchmarks/benchmark_python_bindings.py
cat benchmarks/python_bindings.json
```
