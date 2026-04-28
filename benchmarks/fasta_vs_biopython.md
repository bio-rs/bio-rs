# FASTA core-throughput benchmark

This repository should not make unverified performance claims.

This benchmark measures the Rust core library directly. It excludes `biors` CLI
startup and JSON serialization overhead so the result reflects the engine's raw
FASTA throughput.

## Environment

- Date: 2026-04-28 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- CPU: Apple M1 Pro
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- bio-rs: `biors-core v0.12.0`
- Python: `3.14.3`
- Biopython: `1.87`
- Git commit: `9acf7a1cdd09180da97d62f4ee46c018b661b475`
- Benchmark schema: `biors.benchmark.fasta_vs_biopython.v1`

## Datasets

### 1. Human reference proteome

- Source: UniProt human reference proteome
- Proteome ID: `UP000005640`
- Taxonomy ID: `9606` (`Homo sapiens`)
- URL: `https://ftp.uniprot.org/pub/databases/uniprot/current_release/knowledgebase/reference_proteomes/Eukaryota/UP000005640/UP000005640_9606.fasta.gz`
- Downloaded archive SHA256: `cfaa8ce64eb832a549be794ab86127d49574456708adb756907415949ca2cf58`
- FASTA SHA256: `7272526c282498e7229eefedeb34173a52e9d3c19a046102d93c02c72d20dbef`
- Shape: 20,659 records, 11,456,702 residues, 13,735,476 bytes
- Residue composition: 11,456,623 canonical, 79 ambiguous, 0 invalid

### 2. Large-scale FASTA

- Source: repeated UniProt human reference proteome
- Construction: repeated the same real human proteome `9x` to exceed `110 MB`
- FASTA SHA256: `1aee653542929e4d8052600c48fe11863584cfb93bfbbaee3de8e7b0231bb410`
- Shape: 185,931 records, 103,110,318 residues, 123,619,284 bytes
- Residue composition: 103,109,607 canonical, 711 ambiguous, 0 invalid

This second dataset is intentionally synthetic in scale, but it is built from a
real proteome to isolate large-input throughput without introducing another
dataset's annotation quirks.

## Workload matching

Scope: core library FASTA throughput, excluding CLI startup and success-envelope JSON serialization.

The benchmark compares the same work on both sides:

- Pure Parse: read FASTA records and count records/residues
- Parse + Validation: parse and classify canonical / ambiguous / invalid residues
- Parse + Tokenization: parse and produce position-preserving token IDs with an
  explicit unknown-token path for ambiguous or invalid residues

For bio-rs, the script rebuilds and invokes the release benchmark example:

```bash
cargo build --release -p biors-core --example benchmark_fasta
target/release/examples/benchmark_fasta <mode> <input.fasta>
```

The benchmark example uses `biors-core` buffered reader APIs, not the `biors`
CLI. It excludes CLI startup and success-envelope JSON serialization.

For Biopython, the script performs matched Python loops over `SeqIO.parse(...)`.

Each benchmark case records the input FASTA SHA-256, an output hash of the
warmup result, timed iterations, throughput, and best-effort memory metadata.
On macOS, memory uses `/usr/bin/time -l` peak RSS for separate bio-rs
and Biopython subprocesses. Treat memory as run metadata, not a universal
memory-efficiency claim across every FASTA workload.

## Reproduce

```bash
cargo build --release -p biors-core --example benchmark_fasta
python3 -m venv .venv-bench
. .venv-bench/bin/activate
pip install biopython
python scripts/benchmark_fasta_vs_biopython.py
cat benchmarks/fasta_vs_biopython.json
```

## Best-case matched results

### Human proteome

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.191s** | 0.471s | **2.46x** | **59.8M** | **68.4** | 56.1 MB | 43.8 MB |
| Parse + tokenization | **0.186s** | 0.471s | **2.54x** | **61.7M** | **70.6** | 40.0 MB | 43.8 MB |

### Large-scale FASTA

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **1.712s** | 4.232s | **2.47x** | **60.2M** | **68.9** | 476.6 MB | 43.8 MB |
| Parse + tokenization | **1.659s** | 4.238s | **2.55x** | **62.1M** | **71.1** | 334.4 MB | 43.8 MB |

## Raw result scope

The JSON artifact includes all matched workloads, including `pure_parse`. On
this machine, Biopython remains faster on pure parse. The favorable result for
bio-rs appears when the comparison includes the actual validation or
tokenization work that the Rust engine is designed to do.

That is the intended claim boundary:

- reasonable claim: bio-rs is materially faster than Biopython on matched
  protein FASTA validation and tokenization workloads in this benchmark
- not a supported claim: bio-rs is universally faster than Biopython for every
  FASTA-related task
