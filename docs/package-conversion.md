# Package Conversion

This guide covers the package conversion path for Python and Hugging Face based
projects.

The conversion layer builds portable bio-rs package structure and records
checksums. It does not infer optional model artifact metadata such as model
name, version, architecture, task, source, or description. Package authors can
add `model.metadata` after conversion when they want inspect and bridge reports
to carry that context.

## Hugging Face Tokenizer Config

```bash
biors tokenizer convert-hf ./tokenizer_config.json \
  --output ./protein-package/tokenizers/protein-20-special.json
```

The command reads a Hugging Face `tokenizer_config.json`, maps supported
special-token metadata to the closest built-in bio-rs protein tokenizer config,
and emits:

- converted bio-rs tokenizer config
- `conversion_status` showing the result is a preview, not package-ready
- preview tokenizer asset and preprocessing step fragments
- conversion assumptions and warnings
- SHA-256 of the converted config

The conversion does not read Hugging Face vocab files, token IDs, normalizer
rules, or pre-tokenizer rules. Treat the fragments as scaffolding only until a
fixture parity check proves the converted bio-rs tokens match the source
tokenizer for representative protein inputs.

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
config, writes a static pipeline config, and records byte-for-byte SHA-256
checksums for all manifest-referenced files.

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

- one `.onnx` model artifact
- optional `tokenizer_config.json`

Generated and cache directories such as `.venv`, `.git`, `.cache`,
`__pycache__`, notebook checkpoints, `target`, `build`, and `dist` are skipped
by default. Pass `--include-generated` only when the intended model or tokenizer
config really lives in one of those directories. If multiple ONNX candidates are
found, the command returns their sorted paths and requires `--model`.

## Verify

```bash
biors package validate ./protein-package/manifest.json
biors package verify ./protein-package/manifest.json ./protein-package/observations.json
```

Validation checks portable paths, v1 metadata, declared layout placement, and
checksums. Verification compares package fixture expected outputs against
observed output artifacts supplied by the caller.
