# Pipeline Config

Pipeline config v0 makes the static FASTA preprocessing path reproducible from
a checked-in config file.
The JSON shape is described by `schemas/pipeline-config.v0.json`.
Package manifests can reference pipeline config artifacts from preprocessing
steps; see [Package Format](package-format.md).

Supported formats:

- TOML: `examples/pipeline/protein.toml`
- YAML: `examples/pipeline/protein.yaml`
- JSON: `examples/pipeline/protein.json`

## Run

```bash
biors pipeline --config examples/pipeline/protein.toml
```

`--config` cannot be combined with legacy `--max-length` or a positional input
path. Config-relative input paths are resolved from the config file directory.

## Dry Run

```bash
biors pipeline --config examples/pipeline/protein.yaml --dry-run
```

Dry-run validates the config and emits the planned stages without reading the
input FASTA file.

## Explain Plan

```bash
biors pipeline --config examples/pipeline/protein.json --explain-plan
```

`--explain-plan` includes the static execution plan together with the normal
workflow result.

## Config Shape

```toml
schema_version = "biors.pipeline.v0"
name = "protein-fixture-pipeline"

[input]
format = "fasta"
path = "../protein.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 8
pad_token_id = 0
padding = "fixed_length"
```

The MVP intentionally supports one static workflow:

1. parse FASTA input
2. normalize sequence records with `strip_ascii_whitespace_uppercase`
3. validate protein records
4. tokenize with `protein-20`
5. export model-ready JSON

Unknown fields and unsupported values fail with `pipeline.invalid_config`.

## Crate Split Review

`biors-pipeline` remains deferred in v0.33.0.

Rationale:

- The config MVP is a thin CLI orchestration layer over the existing workflow.
- The reusable policy surface is already in `biors_core::versioning`.
- A new crate would add release coordination before the pipeline model has more
  than one workflow backend.

Revisit the split when pipeline configs have multiple engines, reusable
builders, or non-CLI consumers.
