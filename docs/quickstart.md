# Quickstart

This guide uses the repository examples so each command is reproducible from a
fresh checkout.

## Install

```bash
cargo install biors --version 0.34.0
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
biors tokenize \
  --config examples/model-input-contract/protein-20-special.config.json \
  examples/model-input-contract/protein.fasta
```

`tokenize` emits stable `protein-20` token IDs. Ambiguous or invalid residues
keep positional alignment by using the explicit unknown token ID.

Inspect tokenizer profiles and special token policy:

```bash
biors tokenizer inspect --profile protein-20-special
```

The `protein-20-special` profile keeps the protein-20 residue IDs stable and
adds `UNK=20`, `PAD=21`, `CLS=22`, `SEP=23`, and `MASK=24` for model-input
contract tests and Python preprocessing parity fixtures.

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
model input. Provenance also records the resolved CLI invocation, tokenizer
vocabulary SHA-256, and output-content SHA-256 for repeatable runs.

## Compose A CLI Pipeline

```bash
biors pipeline --max-length 8 examples/protein.fasta
```

`pipeline` uses the same no-config preprocessing defaults as `workflow`, but
adds explicit validate, tokenize, and export step statuses for scripts that
chain command output into downstream jobs.

## Inspect A Problem Sequence

```bash
biors debug --max-length 8 examples/protein.fasta
```

`debug` shows each normalized residue, emitted token ID, model-input record
when available, and compact `W`/`E` markers for residues that need review.

## Compare Outputs

```bash
biors diff expected.json observed.json
```

`diff` compares canonical JSON when both files are JSON and raw bytes
otherwise. It reports SHA-256 hashes, whether the outputs match, and
first-difference metadata for mismatches.

## Validate Batches

```bash
biors batch validate --kind auto examples/
biors batch validate --kind auto "examples/*.fasta"
```

`batch validate` accepts multiple files, recursive directories, and quoted glob
patterns. Directory inputs include common FASTA extensions and skip unrelated
files. Empty glob expansion returns `batch.no_inputs`. The command emits one
batch summary plus per-file validation summaries without retaining per-record
validation payloads.

## Run A Pipeline Config

```bash
biors pipeline --config examples/pipeline/protein.toml --explain-plan
biors pipeline --config examples/pipeline/protein.yaml --dry-run
biors pipeline \
  --config examples/protein-package/pipelines/protein.toml \
  --package examples/protein-package/manifest.json \
  --write-lock examples/pipeline/pipeline.lock
```

Pipeline configs support TOML, YAML, and JSON. The static MVP runs parse,
normalize, validate, tokenize, and export stages. `--write-lock` records a
reproducible execution lock with config, input, vocabulary, output, model, and
backend pins when package context is supplied. See
[Pipeline Config](pipeline-config.md).

## Verify Package Fixtures

```bash
biors package validate examples/protein-package/manifest.json
biors package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

Package commands validate portable manifest assets, v1 layout and research
metadata, SHA-256 checksums, and expected fixture outputs against observed
output artifacts. See [Package Format](package-format.md) for the directory
layout and manifest rules.
