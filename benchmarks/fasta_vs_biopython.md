# FASTA benchmark baseline (biors v0.9.1 vs Biopython)

This repository should not make unverified performance claims.

This benchmark is intended as a reproducible baseline on a realistic proteome-scale
dataset, not a final/permanent speed claim.

## Environment

- Date: 2026-04-26 (UTC)
- OS: macOS-26.3.1-arm64-arm-64bit-Mach-O
- CPU: Apple M1 Pro
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Python: `3.14.3`
- Biopython: `1.87`
- Input: **UniProt human reference proteome**
  - Proteome ID: `UP000005640`
  - Taxonomy ID: `9606` (Homo sapiens)
  - Source URL: `https://ftp.uniprot.org/pub/databases/uniprot/current_release/knowledgebase/reference_proteomes/Eukaryota/UP000005640/UP000005640_9606.fasta.gz`
  - Downloaded archive SHA256: `cfaa8ce64eb832a549be794ab86127d49574456708adb756907415949ca2cf58`
  - FASTA SHA256 (decompressed): `7272526c282498e7229eefedeb34173a52e9d3c19a046102d93c02c72d20dbef`
- Dataset shape: 20,659 records, 11,456,702 residues
  - Canonical protein-20 residues: 11,456,623
  - Ambiguous residues (`X/B/Z/J/U/O`): 79
  - Invalid residues: 0
- Biopython paths:
  - parse only measures `SeqIO.parse(..., "fasta")` plus record/residue counts
  - parse + token/count adds a Python-level protein-20 membership loop
- biors CLI paths:
  - `target/release/biors inspect <input>` measures CLI startup, file read,
    FASTA parse, protein-20 tokenization/validation, and summary JSON output
  - `target/release/biors tokenize <input>` measures CLI startup, file read,
    FASTA parse, protein-20 tokenization/validation, and full pretty JSON output

## Reproduce

```bash
cargo build --release -p biors
pip3 install biopython
python3 scripts/benchmark_fasta_vs_biopython.py
cat benchmarks/fasta_vs_biopython.json
```

## Latest recorded result

From `benchmarks/fasta_vs_biopython.json`:

- Biopython parse only mean: **0.056s**
- Biopython parse + protein-20 token/count loop mean: **0.457s**
- biors CLI inspect summary output mean: **0.198s**
- biors CLI tokenize full JSON output mean: **0.385s**

Interpretation for v0.9.1:

- The baseline uses the UniProt human reference proteome, which is a realistic
  reference-proteome-scale dataset for researcher workflows.
- This is one realistic workload class, not a claim that researchers only run
  single-proteome FASTA jobs.
- The split matters: Biopython parse-only timing is not comparable to `biors
  tokenize`, because `biors tokenize` parses, tokenizes, launches a CLI process,
  and writes full pretty JSON.
- On this UniProt human proteome benchmark, `biors tokenize` completed in
  0.385s while producing full JSON output, compared with 0.457s for a Biopython
  parse + protein-20 token/count loop.
- On this single-machine run, `biors inspect` gives the closest CLI-level summary
  timing, while `biors tokenize` measures the additional cost of full JSON output.
- Treat these numbers as a reproducible bottleneck map for FASTA parsing,
  token/count work, CLI overhead, and JSON-output overhead. Do not use them as a
  broad claim that bio-rs is faster than Biopython across FASTA workloads.
