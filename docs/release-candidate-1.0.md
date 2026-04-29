# 1.0 Release Candidate Path

0.12.0 completes the Phase 2 stabilization pass. The next step is to use the
documented contract surfaces in real package-validation workflows before
declaring 1.0.

## 0.12.0 Stabilization Done

- FASTA and tokenizer golden fixtures are committed.
- Invalid FASTA and tokenizer invariant tests cover core edge cases.
- Package verification reports expose structured mismatch details.
- Benchmarks include provenance, input/output hashes, speed, and memory
  metadata.
- Quickstart, CLI contract, API/schema review, MSRV draft, license, citation,
  and release-history surfaces are linked from the README.

## Before 1.0

- Confirm the public Rust API list in `docs/public-contract-1.0-candidates.md`.
- Confirm all JSON schema names and `$id` values.
- Decide whether legacy compatibility fields should remain through 1.0.
- Run `scripts/check.sh` on the final release-candidate commit.
- Publish only after tag and Crates.io release approval.

## Not In Scope For 1.0

- Hosted web workflows.
- Python bindings.
- Model inference backends.
- Package registry or plugin ecosystem.
