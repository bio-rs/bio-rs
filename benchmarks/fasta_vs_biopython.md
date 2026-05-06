# FASTA core-throughput benchmark

This repository should not make unverified performance claims.

This benchmark measures the Rust core library directly. It excludes `biors` CLI
startup and JSON serialization overhead so the result reflects the engine's raw
FASTA throughput.

## Environment

- Date: 2026-05-06 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- CPU: Apple M1 Pro
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- bio-rs: `biors-core v0.15.1`
- Python: `3.14.3`
- Biopython: `1.87`
- Git commit: `208ca73a09bdf32882213d9b9f5e69e6ffea0795`
- Benchmark schema: `biors.benchmark.fasta_vs_biopython.v1`

## Datasets

### 1. Human Reference Proteome

- Source: UniProt reference proteome
- Shape profile: `human_reference_proteome`
- Proteome ID: `UP000005640`
- Taxonomy ID: `9606` (`Homo sapiens`)
- URL: `https://ftp.uniprot.org/pub/databases/uniprot/current_release/knowledgebase/reference_proteomes/Eukaryota/UP000005640/UP000005640_9606.fasta.gz`
- Downloaded archive SHA256: `cfaa8ce64eb832a549be794ab86127d49574456708adb756907415949ca2cf58`
- FASTA SHA256: `7272526c282498e7229eefedeb34173a52e9d3c19a046102d93c02c72d20dbef`
- Shape: 20,659 records, 11,456,702 residues, 13,735,476 bytes
- Residue composition: 11,456,623 canonical, 79 ambiguous, 0 invalid

### 2. Large Scale Fasta

- Source: repeated_uniprot_human_proteome
- Shape profile: `large_repeated_proteome`
- Construction: repeated the same real human proteome `9x` to exceed `110 MB`
- FASTA SHA256: `1aee653542929e4d8052600c48fe11863584cfb93bfbbaee3de8e7b0231bb410`
- Shape: 185,931 records, 103,110,318 residues, 123,619,284 bytes
- Residue composition: 103,109,607 canonical, 711 ambiguous, 0 invalid

### 3. Many Short Records

- Source: synthetic_many_short_records_from_uniprot_human_proteome
- Shape profile: `many_short_records`
- Construction: `20,000` records of `48` residues
- FASTA SHA256: `2a8622814c7de2a912f9bb95d9f6c8ae90bc0882381453c4f6eee67cd716d64a`
- Shape: 20,000 records, 960,000 residues, 1,228,890 bytes
- Residue composition: 959,996 canonical, 4 ambiguous, 0 invalid

### 4. Single Long Sequence

- Source: synthetic_single_long_sequence_from_uniprot_human_proteome
- Shape profile: `single_long_sequence`
- Construction: one sequence with `960,000` residues
- FASTA SHA256: `be3d011cba58794fb1e2d9b567910a704472f415152f055804533304f8313599`
- Shape: 1 records, 960,000 residues, 972,013 bytes
- Residue composition: 959,996 canonical, 4 ambiguous, 0 invalid

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
| Parse + validation | **0.038s** | 0.446s | **11.79x** | **302.8M** | **346.3** | 1.5 MB | 44.2 MB |
| Parse + tokenization | **0.060s** | 0.446s | **7.46x** | **191.6M** | **219.1** | 23.4 MB | 44.1 MB |

### Large Scale Fasta

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.347s** | 4.110s | **11.84x** | **297.0M** | **339.6** | 1.5 MB | 43.9 MB |
| Parse + tokenization | **0.516s** | 4.197s | **8.13x** | **199.9M** | **228.5** | 187.4 MB | 44.1 MB |

### Many Short Records

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.007s** | 0.059s | **8.37x** | **135.8M** | **165.8** | 1.5 MB | 44.5 MB |
| Parse + tokenization | **0.010s** | 0.059s | **5.90x** | **96.2M** | **117.5** | 6.8 MB | 44.2 MB |

### Single Long Sequence

| Workload | bio-rs mean | Biopython mean | bio-rs speedup | bio-rs residues/s | bio-rs MB/s | bio-rs peak memory | Biopython peak memory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Parse + validation | **0.006s** | 0.036s | **5.93x** | **160.3M** | **154.8** | 1.5 MB | 48.0 MB |
| Parse + tokenization | **0.008s** | 0.035s | **4.45x** | **121.4M** | **117.3** | 2.5 MB | 48.3 MB |

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
