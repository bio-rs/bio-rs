# Changelog

All notable public behavior changes for bio-rs are recorded here.

## Unreleased

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
