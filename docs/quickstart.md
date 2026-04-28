# Quickstart

This guide uses the repository examples so each command is reproducible from a
fresh checkout.

## Validate FASTA

```bash
cargo run -p biors -- fasta validate examples/protein.fasta
```

Use this first when you need structured diagnostics for record counts,
ambiguous residues, and invalid residues.

## Tokenize FASTA

```bash
cargo run -p biors -- tokenize examples/protein.fasta
```

`tokenize` emits stable `protein-20` token IDs. Ambiguous or invalid residues
keep positional alignment by using the explicit unknown token ID.

## Build Model Input

```bash
cargo run -p biors -- model-input --max-length 8 examples/protein.fasta
```

`model-input` emits `input_ids`, `attention_mask`, and truncation metadata. It
rejects sequences with unresolved residue warnings or errors.

## Verify Package Fixtures

```bash
cargo run -p biors -- package validate examples/protein-package/manifest.json
cargo run -p biors -- package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

Package commands validate portable manifest assets and compare expected fixture
outputs against observed output artifacts.
