# Quickstart

This guide uses the repository examples so each command is reproducible from a
fresh checkout.

## Install

```bash
cargo install biors --version 0.14.0
biors --version
```

When working inside a source checkout, replace `biors` with
`cargo run -p biors --`.

## Validate FASTA

```bash
biors fasta validate examples/protein.fasta
```

Use this first when you need structured diagnostics for record counts,
ambiguous residues, and invalid residues.

## Tokenize FASTA

```bash
biors tokenize examples/protein.fasta
```

`tokenize` emits stable `protein-20` token IDs. Ambiguous or invalid residues
keep positional alignment by using the explicit unknown token ID.

## Build Model Input

```bash
biors model-input --max-length 8 examples/protein.fasta
```

`model-input` emits `input_ids`, `attention_mask`, and truncation metadata. It
rejects sequences with unresolved residue warnings or errors.

## Verify Package Fixtures

```bash
biors package validate examples/protein-package/manifest.json
biors package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

Package commands validate portable manifest assets and compare expected fixture
outputs against observed output artifacts.
