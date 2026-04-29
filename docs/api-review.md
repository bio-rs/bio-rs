# API and Schema Review

This document records the 0.12.x review state for the 1.0 release-candidate
path. It is not a promise that every listed surface is stable today.

## Rust API

Current candidate surfaces are listed in
[`public-contract-1.0-candidates.md`](public-contract-1.0-candidates.md).

Reviewed through 0.12.8:

- FASTA parse and reader APIs keep line and record-index diagnostics.
- Tokenization preserves sequence length with explicit unknown-token IDs.
- Protein-20 vocabulary can be borrowed through `protein_20_vocabulary` and
  `ProteinTokenizer::vocabulary_ref` without changing the owned vocabulary API.
- FASTA inspect summaries can be produced from reader input without
  materializing token vectors.
- Model-input builders separate checked and unchecked paths.
- Package verification reports include stable issue codes plus content diff
  metadata for mismatches.
- `biors-core` SRP review keeps `package`, `sequence`, `verification`, and
  FASTA scanner changes internal-only: public re-exports and JSON payloads are
  unchanged.
- Requested `pssm`, `fmindex`, and `from_fm_index_unchecked` audit items do
  not exist in `bio-rs` 0.12.x; they are external rust-bio concepts and were
  not added as new modules.

## JSON Schemas

Schemas under [`../schemas`](../schemas) are validated by
`packages/rust/biors/tests/schema_contract.rs`.

Reviewed through 0.12.8:

- CLI success and error envelopes.
- FASTA validation, inspect, tokenize, and model-input payloads.
- Package manifest, inspect, validate, bridge, and verify payloads.
- 0.12.8 changed internal module boundaries only; no JSON schema shape changed.

## Breaking-Change Cleanup

Before 1.0, review whether to keep or adjust:

- FASTA `fnv1a64:` input hashes for backward compatibility.
- Legacy string `issues` alongside typed `structured_issues`.
- Pre-1.0 `v0` schema filenames and `$id` values.
- Lockstep `biors-core` and `biors` versioning.
