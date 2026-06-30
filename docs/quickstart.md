# Quickstart

This guide uses the repository test data so each command is reproducible from a
fresh checkout.

The quickstart exercises the researcher-callable CLI side of the AI-ready
biological data I/O, validation, and tokenization engine: local validation,
model-ready input preparation, package checks, and reproducible JSON outputs
that research agents can also call through MCP.

## Install

```bash
cargo install biors --version 0.58.0
biors --version
biors doctor
```

When working inside a source checkout, replace `biors` with
`cargo run -p biors --`.

`biors doctor` reports local platform, Rust/Cargo availability, optional WASM
target readiness, and whether the committed CLI transcript data and package fixtures are present.

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

The demo uses `testdata/sequences/launch-demo.fasta`, then shows the public contract
surfaces that make bio-rs useful outside a notebook:

- `doctor` readiness diagnostics for the local toolchain and release fixtures
- kind-aware validation with stable sequence diagnostics
- tokenization and model-input JSON with reproducibility hashes
- reproducible Markdown/shareable JSON reports from CLI output
- package fixture verification with checksums and observed outputs

## Validate FASTA

```bash
biors fasta validate testdata/sequences/protein.fasta
```

Use this for protein-first FASTA validation. It defaults to the `protein-20`
policy for compatibility and accepts `--kind protein|dna|rna|auto` when you
want a specific policy.

## Validate Biological Sequences

```bash
biors seq validate testdata/sequences/protein.fasta
```

Use this for mixed biological FASTA. It defaults to `--kind auto`, assigns
Protein, DNA, or RNA per record, and reports `kind_counts` plus kind-specific
warnings and errors. Short or alphabet-overlapping sequences such as `ACGT`
can be valid under more than one kind; auto-detected records include
`auto_detection.candidate_kinds` and `auto_detection.ambiguous` so you can
rerun with explicit `--kind protein|dna|rna` when the biological context is
known.

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
biors tokenize testdata/sequences/protein.fasta
biors tokenize \
  --config testdata/model-input-contract/protein-20-special.config.json \
  testdata/model-input-contract/protein.fasta
printf '>dna\nACGT\n' | biors tokenize --profile dna-iupac -
printf '>rna\nACGUN\n' | biors tokenize --profile rna-iupac-special -
```

`tokenize` emits stable token IDs for explicit protein, DNA, and RNA profiles.
Ambiguous or invalid residues keep positional alignment by using the profile
unknown token ID. DNA/RNA tokenization now supports direct `model-input` and
CLI `workflow` generation when an explicit nucleotide profile is selected.

Inspect tokenizer profiles and special token policy:

```bash
biors tokenizer inspect --profile protein-20-special
biors tokenizer convert-hf tokenizer_config.json --output tokenizers/protein-20-special.json
```

The `protein-20-special` profile keeps the protein-20 residue IDs stable and
adds `UNK=20`, `PAD=21`, `CLS=22`, `SEP=23`, and `MASK=24` for model-input
contract tests and Python preprocessing parity fixtures.
`tokenizer convert-hf` maps a Hugging Face `tokenizer_config.json` into the
small bio-rs protein tokenizer config as a preview. It does not read the
Hugging Face vocab, token IDs, normalizer, or pre-tokenizer, so validate
fixture parity before copying the preview fragments into a package. DNA/RNA
model-input and workflow paths use explicit bio-rs nucleotide profiles; they
are not promoted through arbitrary Hugging Face tokenizer conversion. The
command keeps `output_path` as the host write destination and emits
`preview_tokenizer_asset.path` as a package-relative `tokenizers/...` path.

## Build Model Input

```bash
biors model-input --max-length 8 testdata/sequences/protein.fasta
printf '>dna\nACGT\n' | biors model-input --profile dna-iupac --max-length 128 -
```

`model-input` emits `input_ids`, `attention_mask`, and truncation metadata. It
rejects sequences with unresolved residue warnings or errors. Use `--profile`
for explicit protein, DNA, or RNA tokenizer profiles.

## Run The Stable Workflow

```bash
biors workflow --max-length 8 testdata/sequences/protein.fasta
printf '>rna\nACGU\n' | biors workflow --profile rna-iupac --max-length 128 -
```

`workflow` runs the FASTA preparation path end to end: validation,
deterministic profile tokenization, model-input generation, readiness issues,
and reproducibility provenance. If a sequence has unresolved warnings or errors,
the command keeps validation and tokenization context in the JSON payload and
sets `model_ready` to `false` instead of silently producing partial model input.
Provenance also records the resolved CLI invocation, tokenizer vocabulary
SHA-256, and output-content SHA-256 for repeatable runs.

## Compose A CLI Pipeline

```bash
biors pipeline --max-length 8 testdata/sequences/protein.fasta
```

`pipeline` uses the same no-config preprocessing defaults as `workflow`, but
adds explicit validate, tokenize, and export step statuses for scripts that
chain command output into downstream jobs.

## Inspect A Problem Sequence

```bash
biors debug --max-length 8 testdata/sequences/protein.fasta
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

## Export A Shareable Report

```bash
biors workflow --max-length 8 testdata/sequences/protein.fasta > workflow.json
biors report generate workflow.json \
  --output workflow-report.md \
  --shareable-json workflow-report.json
```

`report generate` turns bio-rs JSON into a deterministic Markdown report and
`biors.report.v0` JSON export. The report includes provenance, raw input
SHA-256, canonical JSON SHA-256, and a Markdown SHA-256 so reviewers can trace
what was summarized without uploading biological data.

## Validate Batches

```bash
biors dataset inspect --source local --version unversioned --split testdata testdata/sequences/
biors batch validate --kind auto testdata/sequences/
biors batch validate --kind auto "testdata/sequences/*.fasta"
```

`dataset inspect` and `batch validate` accept multiple files, recursive
directories, and quoted glob patterns. Directory inputs include common FASTA
extensions and skip unrelated files. Empty glob expansion returns
`dataset.no_inputs` or `batch.no_inputs`, depending on the command. Batch
validation emits one summary plus per-file validation summaries without
retaining per-record validation payloads.
Dataset inspection emits a descriptor (`source`, `version`, `split`), optional
`--metadata key=value` pairs, file SHA-256 values, a portable dataset content
hash, a local mapping hash, and a dataset-to-sample mapping built from FASTA
record IDs.

## Start The Local Service

```bash
biors service contract
biors serve --host 127.0.0.1 --port 8787
```

In another terminal:

```bash
curl -s http://127.0.0.1:8787/health
curl -s http://127.0.0.1:8787/v0/batch/sequence/validate \
  -H 'content-type: application/json' \
  -d '{"kind":"auto","inputs":[{"id":"sample1","fasta_text":">seq1\nACDE\n"}]}'
```

`biors serve` is local-first. It does not upload biological data, call external
services, run model inference, or persist request bodies.

## Run A Pipeline Config

```bash
biors pipeline --config testdata/pipeline/protein.toml --explain-plan
biors pipeline --config testdata/pipeline/protein.json --dry-run
biors pipeline \
  --config testdata/protein-package/pipelines/protein.toml \
  --package testdata/protein-package/manifest.json \
  --write-lock testdata/pipeline/pipeline.lock
```

Pipeline configs support TOML and JSON. The static workflow runs parse,
normalize, validate, tokenize, and export stages. `--write-lock` records a
reproducible execution lock with config, input, vocabulary, output, model, and
backend pins when package context is supplied. See
[Pipeline Config](pipeline-config.md).

## Verify Package Fixtures

```bash
biors package validate testdata/protein-package/manifest.json
biors package verify \
  testdata/protein-package/manifest.json \
  testdata/protein-package/observations.json
```

Package commands validate portable manifest assets, v1 layout and research
metadata, SHA-256 checksums, and expected fixture outputs against observed
output artifacts. See [Package Format](package-format.md) for the directory
layout and manifest rules.

## Convert A Python Project Into A Package Skeleton

```bash
biors package convert-project ./python-project \
  --output ./protein-package \
  --name protein-package \
  --fixture-input testdata/protein-package/fixtures/tiny.fasta \
  --fixture-output testdata/protein-package/fixtures/tiny.output.json \
  --license CC0-1.0 \
  --citation "Your package citation" \
  --model-card-summary "What this package is intended to do." \
  --intended-use "Local preprocessing parity checks" \
  --limitation "Review before scientific inference"
biors package validate ./protein-package/manifest.json
```

`package convert-project` scans for one ONNX model and an optional
`tokenizer_config.json`, skipping generated/cache directories by default. If
multiple ONNX or tokenizer config candidates are found, pass `--model` or
`--tokenizer-config` explicitly. Its Hugging Face tokenizer conversion path is
a protein-tokenizer preview: it creates package docs, writes a pipeline config,
records checksums, and leaves optional model artifact metadata unset for the
package author to fill in. DNA/RNA package manifest validation and explicit
`package init --tokenizer-config` skeletons are supported, but arbitrary
Python/Hugging Face project conversion is not promoted for nucleotide packages.
