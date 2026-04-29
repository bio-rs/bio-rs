# API and Schema Review

This document records the 0.12.x review state for the 1.0 release-candidate
path. It is not a promise that every listed surface is stable today.

## Rust API

Current candidate surfaces are listed in
[`public-contract-1.0-candidates.md`](public-contract-1.0-candidates.md).

Reviewed for 0.12.0:

- FASTA parse and reader APIs keep line and record-index diagnostics.
- Tokenization preserves sequence length with explicit unknown-token IDs.
- FASTA inspect summaries can be produced from reader input without
  materializing token vectors.
- Model-input builders separate checked and unchecked paths.
- Package verification reports include stable issue codes plus content diff
  metadata for mismatches.

## JSON Schemas

Schemas under [`../schemas`](../schemas) are validated by
`packages/rust/biors/tests/schema_contract.rs`.

Reviewed for 0.12.0:

- CLI success and error envelopes.
- FASTA validation, inspect, tokenize, and model-input payloads.
- Package manifest, inspect, validate, bridge, and verify payloads.

## Breaking-Change Cleanup

Before 1.0, review whether to keep or adjust:

- FASTA `fnv1a64:` input hashes for backward compatibility.
- Legacy string `issues` alongside typed `structured_issues`.
- Pre-1.0 `v0` schema filenames and `$id` values.
- Lockstep `biors-core` and `biors` versioning.
