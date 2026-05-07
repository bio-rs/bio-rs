# Changelog

All notable public behavior changes for bio-rs are recorded here.

## 0.37.0 - 2026-05-07

### Added

- Added `biors dataset inspect <path|directory|glob>...` for deterministic
  FASTA input resolution before validation or pipeline execution.
- Added a shared input dataset resolver used by both `dataset inspect` and
  `batch validate`, preserving the existing batch glob error contract.
- Added `schemas/dataset-inspect-output.v0.json` and CLI/schema coverage for
  dataset inspection.

## 0.36.0 - 2026-05-07

### Added

- Added `biors package convert <manifest|-> --to biors.package.v1` for
  converting supported v0 package manifests to v1 manifests with explicit
  author-supplied research metadata.
- Added optional converted manifest file writing with `--output`, manifest
  SHA-256 provenance, and inferred v1 package layout directories.
- Added `schemas/package-conversion-output.v0.json` plus CLI and schema tests
  for conversion success and missing metadata errors.

## 0.35.0 - 2026-05-07

### Added

- Added `biors package migrate <manifest|-> --to <schema>` for inspectable
  package manifest schema migration plans.
- Added `biors package compatibility <left-manifest> <right-manifest>` for
  schema compatibility and migration-required checks.
- Added `biors package diff <left-manifest> <right-manifest>` for canonical
  package manifest content diffs with schema context.
- Added `schemas/package-migration-output.v0.json`,
  `schemas/package-compatibility-output.v0.json`, and
  `schemas/package-diff-output.v0.json`.

## 0.34.0 - 2026-05-07

### Added

- Added package manifest support for preprocessing/postprocessing steps that
  reference checked `biors.pipeline.v0` config artifacts.
- Added `package_layout.pipelines` and package inspect output for declared
  pipeline config paths.
- Extended package artifact validation to check pipeline config paths,
  checksums, and declared layout placement.
- Added `biors pipeline --config ... --write-lock <pipeline.lock>` for
  reproducible lockfiles that pin bio-rs versions, config SHA-256, input hash,
  vocabulary hash, output-content hash, and Python baseline parity strategy.
- Added optional package context for pipeline lockfiles so package model
  checksum, runtime backend, target, and backend version can be pinned.
- Added `schemas/pipeline-lock.v0.json` and a committed
  `examples/pipeline/pipeline.lock` fixture.

## 0.33.0 - 2026-05-07

### Added

- Added `biors pipeline --config <toml|yaml|json>` for config-driven static
  FASTA preprocessing over the existing validate -> tokenize -> export path.
- Added `schemas/pipeline-config.v0.json`, config examples, and documentation
  for config-relative inputs, dry runs, and explain-plan output.
- Extended pipeline output schema coverage for `config_pipeline.v0` dry-run and
  plan payloads.

## 0.32.0 - 2026-05-07

### Added

- Added `biors_core::versioning` policy APIs for package manifest and pipeline
  config schema lifecycles.
- Defined deprecation, breaking-change, backward-compatibility, and migration
  policies for schema-bearing artifacts.
- Added a v0 to v1 package manifest migration plan and reserved
  `biors.pipeline.v0` for the upcoming pipeline config MVP.
- Documented schema versioning rules in `docs/schema-versioning.md`.

## 0.31.0 - 2026-05-07

### Added

- Added `biors.package.v1` manifests with declared package directory layout,
  license/citation/model-card metadata, and schema coverage in
  `schemas/package-manifest.v1.json`.
- Extended package validation to check v1 layout membership, package-relative
  metadata document paths, and SHA-256 checksums for license files, citation
  files, and model cards.
- Added package-format documentation and upgraded the committed protein package
  fixture to manifest v1.

### Notes

- `biors-manifest` remains deferred as a separate crate; the manifest contract
  stays isolated under `biors-core::package` until independent consumers need a
  smaller parsing-only crate.

## 0.30.0 - 2026-05-06

### Added

- Added `biors workflow --max-length <N> <path|->`, a stable protein FASTA
  preparation workflow that emits validation, deterministic protein-20
  tokenization, model-ready input, readiness issues, and reproducibility
  provenance in one machine-readable JSON payload.
- Added `biors batch validate [--kind auto|protein|dna|rna] <path|directory|glob>...`
  for multiple input files, recursive directory expansion, quoted glob input,
  empty-glob `batch.no_inputs` errors, and memory-bounded validation summaries.
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
- Added package manifest layout summary output to improve package validation UX.
- Added Python interop examples for ESM-style batches, ProtBERT-style sequence
  adaptation, and pandas/NumPy-friendly JSON conventions without PyO3.
- Added WASM readiness documentation and compile-level `biors-core`
  `wasm32-unknown-unknown` verification.
- Added `schemas/sequence-workflow-output.v0.json`,
  `schemas/batch-validation-output.v0.json`,
  `schemas/tokenizer-inspect-output.v0.json`, `schemas/pipeline-output.v0.json`,
  `schemas/sequence-debug-output.v0.json`, and `schemas/output-diff.v0.json`
  for the new Phase 5 CLI payloads.

### Fixed

- Hardened core sequence validation and tokenization APIs so invalid direct byte
  input returns structured reports instead of panicking.

### Notes

- Website and static browser demo work are intentionally excluded from this
  phase per maintainer direction; repo docs, CLI usage, examples, schemas, and
  benchmark artifacts remain the documentation surface.
