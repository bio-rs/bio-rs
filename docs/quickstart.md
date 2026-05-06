# Quickstart

This guide uses the repository examples so each command is reproducible from a
fresh checkout.

## Install

```bash
cargo install biors --version 0.21.0
biors --version
biors doctor
```

When working inside a source checkout, replace `biors` with
`cargo run -p biors --`.

`biors doctor` reports local platform, Rust/Cargo availability, optional WASM
target readiness, and whether the committed demo/package fixtures are present.

Generate shell completions from the installed binary:

```bash
biors completions zsh > _biors
```

## First 60 Seconds

Run the first-impression workflow before trying individual commands:

```bash
sh scripts/launch-demo.sh
```

From a source checkout before installing:

```bash
sh scripts/launch-demo.sh --cargo
```

For terminal recording tools such as `asciinema`, use the deterministic CLI
transcript script:

```bash
sh scripts/record-cli-demo.sh --cargo
```

The demo uses `examples/launch-demo.fasta`, then shows local diagnostics,
kind-aware validation, tokenization, model-input JSON, and package fixture
verification.

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

For a recording-friendly transcript:

```bash
sh scripts/record-cli-demo.sh
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

## Run The Stable Workflow

```bash
biors workflow --max-length 8 examples/protein.fasta
```

`workflow` runs the protein FASTA preparation path end to end: validation,
deterministic `protein-20` tokenization, model-input generation, readiness
issues, and reproducibility provenance. If a sequence has unresolved warnings
or errors, the command keeps validation and tokenization context in the JSON
payload and sets `model_ready` to `false` instead of silently producing partial
model input.

## Validate Batches

```bash
biors batch validate --kind auto examples/
biors batch validate --kind auto "examples/*.fasta"
```

`batch validate` accepts multiple files, directories, and quoted glob patterns.
Directory inputs include common FASTA extensions and skip unrelated files. The
command emits one batch summary plus per-file validation summaries without
retaining per-record validation payloads.

## Verify Package Fixtures

```bash
biors package validate examples/protein-package/manifest.json
biors package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

Package commands validate portable manifest assets and compare expected fixture
outputs against observed output artifacts.
