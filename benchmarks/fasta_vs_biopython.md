# FASTA core-throughput benchmark

This repository should not make unverified performance claims.

This benchmark measures the Rust core library directly. It excludes `biors` CLI
startup and JSON serialization overhead so the result reflects the engine's raw
FASTA throughput.

## Environment

- Date: 2026-05-06 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit
- CPU: Apple M1 Pro
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- bio-rs: `biors-core v0.20.0`
- Python: `3.12.12`
- Biopython: `1.87`
- Git commit: `a4aee2c2af1d0e23c72f73ef5f6dd48b48cf8252`
- Benchmark schema: `biors.benchmark.fasta_vs_biopython.v1`

## Datasets

### 1. Human Reference Proteome

- Source: user-provided FASTA
- Shape profile: `human_reference_proteome`
- FASTA SHA256: `bf4860fdb87c9a96d3576675bdb0b3c889ac727696e73a07410c7e965aa1fc36`
- Shape: 20,659 records, 11,456,034 residues, 13,735,094 bytes
- Residue composition: 11,455,955 canonical, 79 ambiguous, 0 invalid

### 2. Large Scale Fasta

- Source: repeated_uniprot_human_proteome
- Shape profile: `large_repeated_proteome`
- Construction: repeated the same real human proteome `9x` to exceed `110 MB`
- FASTA SHA256: `c44228388807ec9ac03dbc66516a8e459302336bf2a3391c6b10e75692c9774f`
- Shape: 185,931 records, 103,104,306 residues, 123,615,846 bytes
- Residue composition: 103,103,595 canonical, 711 ambiguous, 0 invalid

### 3. Many Short Records

- Source: synthetic_many_short_records_from_uniprot_human_proteome
- Shape profile: `many_short_records`
- Construction: `20,000` records of `48` residues
- FASTA SHA256: `cb8049118f95a518cc63a4d6c9e0b07e04ab989013f4dfb436368c2108e7ce61`
- Shape: 20,000 records, 960,000 residues, 1,228,890 bytes
- Residue composition: 959,998 canonical, 2 ambiguous, 0 invalid

### 4. Single Long Sequence

- Source: synthetic_single_long_sequence_from_uniprot_human_proteome
- Shape profile: `single_long_sequence`
- Construction: one sequence with `960,000` residues
- FASTA SHA256: `da1455be67cb58e7a60ad883bf38d8523cdeb777ca10bf4c45f1ceb986ed264e`
- Shape: 1 records, 960,000 residues, 972,013 bytes
- Residue composition: 959,998 canonical, 2 ambiguous, 0 invalid

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

## Matched results

### Human Reference Proteome

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.036s** | 0.584s | **16.09x** | **315.4M** | **360.6** | 1.5 MB | 40.9 MB |
| Parse + tokenization | **0.061s** | 0.587s | **9.68x** | **189.0M** | **216.1** | 23.4 MB | 40.5 MB |

### Large Scale Fasta

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.294s** | 3.994s | **13.59x** | **350.8M** | **401.1** | 1.5 MB | 40.6 MB |
| Parse + tokenization | **0.492s** | 4.040s | **8.22x** | **209.7M** | **239.8** | 187.1 MB | 40.7 MB |

### Many Short Records

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.007s** | 0.204s | **28.35x** | **133.5M** | **163.0** | 1.5 MB | 40.5 MB |
| Parse + tokenization | **0.010s** | 0.205s | **20.54x** | **96.2M** | **117.5** | 6.9 MB | 40.5 MB |

### Single Long Sequence

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.005s** | 0.176s | **34.48x** | **188.5M** | **182.0** | 1.5 MB | 43.8 MB |
| Parse + tokenization | **0.007s** | 0.177s | **26.67x** | **144.8M** | **139.8** | 2.5 MB | 44.0 MB |

## Reproduce

```bash
cargo build --release -p biors-core --example benchmark_fasta
python3 -m venv .venv-bench
. .venv-bench/bin/activate
pip install biopython
python scripts/benchmark_fasta_vs_biopython.py
cat benchmarks/fasta_vs_biopython.json
```

## Raw result scope

The JSON artifact includes all matched workloads, including `pure_parse`.
The intended claim boundary is workload-specific:

- reasonable claim: bio-rs is materially faster than Biopython on matched
  protein FASTA validation and tokenization workloads in this benchmark
- not a supported claim: bio-rs is universally faster than Biopython for every
  FASTA-related task
