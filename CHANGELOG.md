# Changelog

All notable public behavior changes for bio-rs are recorded here.

## Unreleased

### Added

- Added `protein-20-special` tokenizer profile with explicit UNK/PAD/CLS/SEP/MASK
  token policy.
- Added tokenizer JSON config loading, `tokenize --config <json>`, and
  `biors tokenizer inspect` for machine-readable token/profile inspection.
- Added draft model-input contract fixtures plus a dependency-free reference
  Python preprocessing parity fixture.
- Added workflow provenance for resolved CLI invocation arguments, tokenizer
  vocabulary SHA-256, and workflow output-content SHA-256.
- Added `biors diff <expected> <observed>` for canonical JSON/raw output hash
  comparison and first-difference reports.
- Added `biors pipeline --max-length <N> <path|->` for no-config
  validate -> tokenize -> export workflow composition.
- Added `biors debug --max-length <N> <path|->` for sequence -> token ->
  model-input step inspection and compact residue error visualization.
- Added Python interop examples for ESM-style batches, ProtBERT-style sequence
  adaptation, and pandas/NumPy-friendly JSON conventions without PyO3.
- Hardened core sequence validation and tokenization APIs so invalid direct
  byte input returns structured reports instead of panicking.
- Added `biors batch validate [--kind auto|protein|dna|rna] <path|directory|glob>...`
  for multiple input files, recursive directory expansion, quoted glob input,
  empty-glob `batch.no_inputs` errors, and memory-bounded validation summaries.
- Added `schemas/batch-validation-output.v0.json` for the batch validation data
  payload and `schemas/tokenizer-inspect-output.v0.json` for tokenizer
  inspection output.

## 0.21.0 - 2026-05-06

### Added

- Added `biors workflow --max-length <N> <path|->`, a stable protein FASTA
  preparation workflow that emits validation, deterministic protein-20
  tokenization, model-ready input, readiness issues, and reproducibility
  provenance in one machine-readable JSON payload.
- Added `schemas/sequence-workflow-output.v0.json` for the workflow data
  payload and regression tests that validate live CLI output against it.

### Notes

- Website and static browser demo work are intentionally excluded from this
  phase per maintainer direction; repo docs, CLI usage, examples, schemas, and
  benchmark artifacts remain the documentation surface.
