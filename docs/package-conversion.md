# Package Conversion

This guide covers the package conversion path for Python and Hugging Face based
projects.

The conversion layer builds portable bio-rs package structure and records
checksums. It does not inspect or import model artifact metadata beyond local
path and SHA-256; model artifact metadata belongs to the execution backend
layer.

## Hugging Face Tokenizer Config

```bash
biors tokenizer convert-hf ./tokenizer_config.json \
  --output ./protein-package/tokenizers/protein-20-special.json
```

The command reads a Hugging Face `tokenizer_config.json`, maps supported
special-token metadata to the closest built-in bio-rs protein tokenizer config,
and emits:

- converted bio-rs tokenizer config
- manifest `tokenizer` asset suggestion
- manifest preprocessing step suggestion
- conversion assumptions and warnings
- SHA-256 of the converted config

## Initialize A Package From Explicit Assets

```bash
biors package init ./protein-package \
  --name protein-package \
  --model ./model.onnx \
  --tokenizer-config ./tokenizer_config.json \
  --fixture-input ./fixtures/tiny.fasta \
  --fixture-output ./fixtures/tiny.output.json \
  --license CC0-1.0 \
  --citation "Your package citation" \
  --model-card-summary "Short model-card summary." \
  --intended-use "Local preprocessing parity checks" \
  --limitation "Review before scientific inference"
```

`package init` creates:

- `manifest.json`
- `models/`
- `tokenizers/`
- `pipelines/protein.toml`
- `fixtures/`
- `observed/`
- `docs/LICENSE.txt`
- `docs/CITATION.cff`
- `docs/model-card.md`

It copies the supplied model and fixture artifacts, writes a bio-rs tokenizer
config, writes a static pipeline config, and records SHA-256 checksums for all
manifest-referenced files.

## Convert A Python Project

```bash
biors package convert-project ./python-project \
  --output ./protein-package \
  --name protein-package \
  --fixture-input ./fixtures/tiny.fasta \
  --fixture-output ./fixtures/tiny.output.json \
  --license CC0-1.0 \
  --citation "Your package citation" \
  --model-card-summary "Short model-card summary." \
  --intended-use "Local preprocessing parity checks" \
  --limitation "Review before scientific inference"
```

`package convert-project` scans the project directory for:

- first `.onnx` model artifact
- `tokenizer_config.json`

Use `--model` or `--tokenizer-config` when the project has multiple candidates
or non-standard layout.

## Verify

```bash
biors package validate ./protein-package/manifest.json
biors package verify ./protein-package/manifest.json ./protein-package/observations.json
```

Validation checks portable paths, v1 metadata, declared layout placement, and
checksums. Verification compares package fixture expected outputs against
observed output artifacts supplied by the caller.
