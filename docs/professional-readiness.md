# Professional Readiness

This document records the current readiness boundary for using bio-rs in
day-to-day biological sequence preprocessing workflows.

## Researcher-Ready Scope

bio-rs is ready for local and CI use when the workflow is:

- protein FASTA parsing and normalization
- protein-20 validation with explicit ambiguous-residue warnings
- stable token ID generation with positional unknown-token preservation
- deterministic model-input JSON for fixed-length or unpadded model inputs
- package manifest inspection and portable asset validation
- package fixture verification against observed output artifacts
- reproducible FASTA validation/tokenization benchmarking
- installed CLI version verification with `biors --version`

The strongest fit today is preprocessing and verification around biological AI
model inputs. The project should be presented as input-contract infrastructure,
not as a model inference engine or broad bioinformatics suite.

## Phase 1 Coverage

| Version | Planned area | Current status |
|---|---|---|
| 0.1.0 | workspace, core/CLI split, FASTA skeleton, fixtures, CI, security policy | Implemented through the Rust workspace, `biors-core`, `biors`, test fixtures, CI, and `SECURITY.md`. |
| 0.2.0 | FASTA parser MVP with headers, records, empty lines, CRLF, invalid errors, line/record diagnostics | Implemented and covered by parser contract tests and fixtures. |
| 0.3.0 | protein validation, lowercase/whitespace normalization, ambiguous residue policy, validation report | Implemented in `sequence` and FASTA validation contracts. |
| 0.4.0 | tokenizer trait, protein tokenizer, vocab loading, unknown-token policy, fixtures | Implemented in `tokenizer` with tokenizer golden fixtures. |
| 0.5.0 | model-ready input, attention mask, padding/truncation, deterministic JSON schemas | Implemented with model-input builders, CLI output, and schema contract tests. |
| 0.6.0 | CLI stabilization draft, JSON mode, exit codes, human errors, landing docs linkage | Implemented for repo-local CLI/docs; hosted website remains out of scope for this checkout. |
| 0.7.0 | provenance metadata, input hash, version in output, verification harness, mismatch report | Implemented through success envelopes, input hashes, and package verification. |
| 0.8.0 | package manifest draft, schema validation, checksums, runtime/input/output contract fields | Implemented with typed manifest enums, checksum validation, and package CLI commands. |

## Phase 2 Coverage

| Version | Planned area | Current status |
|---|---|---|
| 0.9.0 | CLI and JSON contract freeze, error taxonomy, snapshots, contract candidates | Implemented in CLI/schema/error docs and public behavior tests. |
| 0.10.0 | fixture and verification hardening, invalid FASTA invariants, tokenizer invariants | Implemented with expanded fixtures, deterministic invalid FASTA tests, and structured mismatch reports. |
| 0.11.0 | benchmark and reproducibility pass, Biopython comparison, speed/memory proof assets | Implemented with reproducible benchmark JSON/Markdown and artifact validation. |
| 0.12.0 | documentation and 1.0 release candidate, full workflow e2e, policies, release notes | Implemented with e2e CLI workflow tests, quickstart, API/schema review, MSRV, citation, and README release history. |

## Phase 3 Coverage

| Version | Planned area | Current status |
|---|---|---|
| 0.13.0 | DNA/RNA validation draft, sequence kind enum, unified alphabet policy | Implemented with `SequenceKind`, DNA/RNA IUPAC policies, auto-detection, and stable sequence diagnostics. |
| 0.14.0 | multi-alphabet FASTA support, per-record kind assignment, `--kind` CLI flag | Implemented with kind-aware FASTA reader validation, mixed-kind summaries, schema coverage, and explicit override support. |
| 0.15.0 | biological sequence UX polish, `seq validate`, kind-specific messages | Implemented with `biors seq validate`, auto-detect-by-default validation, kind-specific issue messages, and e2e coverage. |

## Refactor And Performance Review

- FASTA parsing now uses a shared scanner for string and reader paths, reducing
  duplicated state-machine logic.
- Reader-based FASTA parsing uses a byte-buffered scanner on ASCII inputs,
  while retaining UTF-8 Unicode fallback behavior and invalid UTF-8 read
  failure classification.
- Residue validation and tokenization use static ASCII lookup tables for
  canonical and ambiguous residues instead of repeated branch-heavy matches on
  the hot path.
- Tokenization-only FASTA APIs avoid materializing normalized sequence strings
  when callers only need token IDs, reducing parse + tokenization time and peak
  memory on large FASTA inputs.
- Validation APIs use a dedicated reader sink for full validation reports, so
  `biors fasta validate` no longer routes through token-vector construction.
- `biors inspect` uses a summary-only reader path so large FASTA inspection no
  longer materializes token vectors just to count records, residues, warnings,
  and errors.
- The published CLI exposes `biors --version`, which makes installed binary
  provenance explicit in lab notebooks, CI logs, and benchmark runs.
- Vocabulary token definitions are static; callers that need only the canonical
  token list can use `protein_20_vocab_tokens()` without rebuilding a `Vec`.
- Benchmark proof assets now cover the human reference proteome, 100MB+
  repeated-proteome input, many short records, and a single long sequence so
  throughput claims are less dependent on one FASTA shape.

## Known Limits

- Protein, DNA, and RNA FASTA validation are supported; tokenization and
  model-ready input remain protein-only through the `protein-20` tokenizer.
- Structure tooling, chemistry tooling, and Python bindings are not implemented.
- Package verification compares local artifacts; bio-rs does not run model
  inference backends.
- Benchmark claims are limited to the committed FASTA validation/tokenization
  workloads and should not be generalized to every Biopython use case.
- The project is still pre-1.0, so public contracts are stabilization
  candidates rather than final stable guarantees.
