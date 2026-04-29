# Changelog

All notable pre-1.0 changes are summarized here. GitHub Releases remain the
source for exact commit lists after tags are published.

## 0.12.2

- Added `biors --version` so installed CLI binaries can be tied back to the
  exact published package version in notebooks, CI logs, and benchmark records.
- Documented version verification in the quickstart, CLI contract, and
  professional-readiness guide.

## 0.12.1

- Added a release workflow invariant check and fixed crates.io publish ordering
  so `biors-core` is published before `biors` dry-run validation.
- Added a professional-readiness audit document covering Phase 1 and Phase 2
  implementation status, researcher-ready scope, and known limits.
- Switched `biors inspect` to a summary-only FASTA reader path to avoid
  materializing token vectors for large-input summaries.
- Updated quickstart documentation for the published CLI install path.

## 0.12.0

- Added full CLI workflow coverage for FASTA validation, tokenization,
  model-input generation, package validation, and package verification.
- Added release-candidate documentation for quickstart, API/schema review,
  MSRV policy, citation policy, and the 1.0 stabilization path.
- Linked public docs from the README so researchers can find the current
  contract surfaces from one entrypoint.

## 0.11.0

- Added benchmark artifact validation for schema version, methodology,
  environment provenance, input/output hashes, and memory metadata.
- Regenerated FASTA vs Biopython proof assets with speed and memory tables.
- Added generated benchmark report checks to local and CI validation.

## 0.10.0

- Added FASTA, tokenizer, and manifest fixture corpora.
- Added structured package verification issue codes and first-difference
  mismatch reports.
- Refactored FASTA parsing and tokenization through a shared byte-aware scanner
  with Unicode fallback.
