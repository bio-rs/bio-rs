# Changelog

All notable pre-1.0 changes are summarized here. GitHub Releases remain the
source for exact commit lists after tags are published.

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
