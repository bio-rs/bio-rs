# FASTA parse+tokenize benchmark (biors v0.8.1 vs Biopython)

This repository should not make unverified performance claims.

This benchmark is intended as a reproducible baseline, not a final/permanent claim.

## Environment

- Date: 2026-04-25 (UTC)
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
- biors path: `target/release/biors tokenize <input>`
- Python path: Biopython `SeqIO.parse(..., "fasta")` + protein-20 membership loop

## Reproduce

```bash
cargo build --release -p biors
pip3 install biopython
python3 scripts/benchmark_fasta_vs_biopython.py
cat benchmarks/fasta_vs_biopython.json
```

## Latest recorded result

From `benchmarks/fasta_vs_biopython.json`:

- Biopython parse+tokenize mean: **1.086s**
- biors CLI tokenize mean: **1.093s**

Interpretation for v0.8.1:

- On the UniProt human proteome benchmark above, `biors tokenize` and the Biopython baseline are in a similar throughput range.
- This is still a single-machine run; avoid broad generalization until repeated runs and broader datasets are tracked.
- One `biors` run showed a visible outlier (max 1.266s), so median and min should also be considered.
