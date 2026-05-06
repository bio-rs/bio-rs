# Quickstart

This guide uses the repository examples so each command is reproducible from a
fresh checkout.

## Install

```bash
cargo install biors --version 0.16.0
biors --version
biors doctor
```

When working inside a source checkout, replace `biors` with
`cargo run -p biors --`.

`biors doctor` reports local platform, Rust/Cargo availability, optional WASM
target readiness, and whether the committed demo/package fixtures are present.

## Validate FASTA

```bash
biors fasta validate examples/protein.fasta
```

Use this for protein-first FASTA validation. It defaults to the `protein-20`
policy for compatibility and accepts `--kind protein|dna|rna|auto` when you
want a specific policy.

## Validate Biological Sequences

```bash
biors seq validate examples/protein.fasta
```

Use this for mixed biological FASTA. It defaults to `--kind auto`, assigns
Protein, DNA, or RNA per record, and reports `kind_counts` plus kind-specific
warnings and errors.

## Run The Launch Demo

```bash
sh scripts/launch-demo.sh
```

From a source checkout before installing:

```bash
sh scripts/launch-demo.sh --cargo
```

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
