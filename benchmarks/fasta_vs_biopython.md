# FASTA parse+tokenize benchmark (biors v0.8.0 vs Biopython)

This repository should not make unverified performance claims.

This benchmark is intended as a reproducible baseline, not a final/permanent claim.

## Environment

- Date: 2026-04-25 (UTC)
- Input: synthetic protein FASTA
- Dataset shape: 5,000 records × 600 residues = 3,000,000 residues
- biors path: `target/release/biors tokenize <input>`
- Python path: Biopython `SeqIO.parse(..., "fasta")` + protein-20 token mapping loop

## Reproduce

```bash
cargo build --release -p biors
pip3 install biopython
python3 scripts/benchmark_fasta_vs_biopython.py
cat benchmarks/fasta_vs_biopython.json
```

## Latest recorded result

From `benchmarks/fasta_vs_biopython.json`:

- Biopython parse+tokenize mean: **0.205s**
- biors CLI tokenize mean: **0.206s**

Interpretation for v0.8.0:

- Current CLI tokenize performance is in the same range as the Biopython baseline on this synthetic workload.
- This is a single synthetic benchmark; avoid broad generalization until larger datasets and repeated CI runs are added.
- For future releases, expand dataset diversity and add repeated CI-published benchmark runs.
