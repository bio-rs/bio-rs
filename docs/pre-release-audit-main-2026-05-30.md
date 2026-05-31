# Pre-release Audit Against Latest `main`

Audit date: 2026-05-30 KST
Last updated: 2026-05-31 KST
Baseline: `origin/main` at `06052d92c8ce606dcd989227942e91572320be07`
Mode: read-only code audit plus local verification. No commit or push.

## Running Verification

- `scripts/check.sh`: passed in isolated worktree `/tmp/bio-rs-main-audit.3R4Ee0`
- `cargo build --locked --release -p biors`: passed
- `BIORS_BIN=target/release/biors sh scripts/launch-demo.sh`: passed
- `wasm-pack test --node packages/rust/biors-wasm`: passed, 9 WASM tests
- `maturin build --release --manifest-path packages/rust/biors-python/Cargo.toml`: passed for local macOS arm64 `cp39-abi3` wheel
- Installed the locally built Python wheel into a temporary venv and ran `python -m pytest packages/rust/biors-python/tests -q`: passed, 6 tests
- `scripts/build-wasm-npm-package.sh`: passed npm pack dry-run, but emitted a missing LICENSE-file warning
- `cargo publish --locked --dry-run` for `biors-core`, `biors-mcp-server`, `biors-backend-candle`, and `biors`: package verification passed; crates.io reports `0.47.4` already exists
- Registry version checks: crates.io, PyPI `biors`, and npm `@bio-rs/biors-wasm` all report `0.47.4`
- Markdown local-link scan across root docs, `docs/`, and package READMEs: passed with zero missing local links
- Schema JSON parse scan across `schemas/*.json`: passed with zero invalid JSON files
- Local tool availability check: `cargo-audit`, `cargo-deny`, `cargo-machete`, and `cargo-udeps` are not installed/configured in this audit environment
- Dependency tree checks: `cargo tree --locked -p biors --duplicates` and `cargo tree --locked -p biors-core --duplicates` had nothing to print; `biors-backend-candle` still has duplicate transitive versions through Candle dependencies, and its normal dependency tree is much larger than the core CLI tree
- GitHub Actions tag existence check: workflow action refs such as `actions/checkout@v6`, `actions/setup-python@v6`, `actions/setup-node@v6`, `actions/upload-artifact@v7`, `actions/download-artifact@v8`, `Swatinem/rust-cache@v2`, and `pypa/gh-action-pypi-publish@release/v1` resolve, but they remain moving tag/branch refs rather than immutable SHAs
- Additional behavior checks in `/tmp`: reproduced invalid CLI provenance padding, clap `--json` parse-error behavior, Python constructor/export gaps, Python validation diagnostics loss, Python/MCP package validation missing artifact/checksum checks, path-sensitive dataset hashes for identical FASTA content, duplicate dataset metadata keys being overwritten, package validation accepting invalid pipeline config content, package validation accepting invalid tokenizer config content, package validation accepting a vocab file that the public vocab loader rejects, package validation accepting a package pipeline config that reads an absolute external FASTA input, package validation accepting a `package_layout.manifest` mismatch, pipeline lock accepting a package manifest unrelated to the config path, `package convert-project` selecting a `.venv` cached ONNX artifact and hidden tokenizer config, benchmark report drift passing the artifact-only check, cache clean broad-root acceptance, tokenizer conversion suggesting absolute manifest paths, `package init` overwrite behavior, `package verify` passing an invalid manifest, `package init` mislabeling `.safetensors` as ONNX/WebGPU, pipeline dry-run returning `ready: true` for a missing input, duplicate fixture names passing validation, generated citation/license file quality, npm pack excluding required WASM glue, documented WASM default import failure, package validation accepting empty contract identifiers, package validation accepting empty shape-dimension strings, core workflow accepting schema-invalid provenance input hashes, and pipeline nested workflow using a command value rejected by the direct workflow schema

## Release Verdict

Latest `origin/main` is not ready for a new public release/tag as-is.

The core Rust/CLI checks pass, but the binding/package/release surfaces still have correctness and packaging gaps that matter for real researchers. Treat these as blockers before promoting a new tag:

- Version/publication/package integrity: A-001, A-010, A-012, A-020, A-021, A-027, A-035, A-036, A-086, A-087, A-126
- Public contract and researcher-safety correctness: A-004, A-006, A-007, A-023, A-026, A-041, A-047, A-049, A-050, A-051, A-053, A-055, A-057, A-058, A-060, A-069, A-070, A-074, A-077, A-078, A-079, A-083, A-084, A-088, A-089, A-091, A-092, A-093, A-094, A-096, A-098, A-099, A-100, A-101, A-102, A-103, A-104, A-105, A-107, A-108, A-109, A-111, A-112, A-113, A-114, A-117, A-118, A-119, A-121, A-122, A-123, A-124
- Destructive-operation safety: A-095
- Benchmark/security/reproducibility/dependency gates: A-029, A-030, A-064, A-065, A-066, A-090, A-097, A-106, A-110, A-115, A-116, A-120, A-125

## Cleanup And Deletion Queue

Delete or rewrite these before release:

- `biors-wasm-design-v0.45.0.md`: complete in follow-up A-009; stale root-level WASM design doc removed.
- `scripts/benchmark_hyperfine.py` and `scripts/biopython_bench.py`: old benchmark path that can overwrite current benchmark artifacts with the wrong schema.
- `packages/rust/biors-core/src/package/checksum.rs`: duplicate checksum re-export module unless intentionally kept as public API.
- `pub mod fasta_scan`: make it private or expose a deliberate public scanner API.
- `packages/rust/biors-python/python/biors/py.typed`: remove until stubs exist, or add real `.pyi` coverage.
- `.github/release_template.md`: either wire it into release creation or delete it as unused release documentation.
- `packages/rust/biors-wasm/package.json` generated `files` policy: rewrite before npm release because it currently excludes required `biors_wasm_bg.js`.
- `packages/rust/biors-wasm/index.d.ts`, `README.md`, and `docs/wasm-api.md`: rewrite the `default init` contract or build a package artifact that actually exports it.
- ignored local build/cache artifacts listed in A-028: clean before release prep, but do not commit.

## Code-Level Coverage Completed

Read and cross-checked these surfaces against schemas, docs, scripts, and local behavior:

- Core FASTA, sequence detection/validation/normalization, tokenizer, model-input, workflow provenance, package manifest/artifact/layout/runtime/tooling/report modules, runtime external-process contracts, service contract, verification/diff/hash modules, and versioning.
- CLI args, handlers, input/output/errors, batch, cache, dataset, debug, workflow, pipeline config/lock/output, package init/convert/verify/bridge/diff/migrate, package skeleton file generation, and tokenizer conversion.
- Python binding implementation, Python package metadata, Python README/API docs, local wheel build and pytest behavior.
- WASM binding implementation, TypeScript/package metadata, generated `pkg/` output, npm pack contents, and local wasm-pack node tests.
- MCP server implementation/tests, Candle backend implementation/docs, root/package Cargo manifests, `.github` workflows, issue/PR/release templates, `scripts/check*.sh`, release/benchmark/module-size scripts, benchmark artifacts, schemas, examples, and public docs.

## Findings

### A-001: `CITATION.cff` version is stale

Severity: release blocker before any public package/tag release.

Status: Complete in follow-up (2026-05-31 KST). `CITATION.cff` now matches the workspace package version, and `cargo test --locked -p biors --test release_readiness citation_version_matches_workspace_package_version` covers the release-readiness invariant.

Evidence:

- Workspace package version is `0.47.4` in `Cargo.toml`.
- README, quickstart, install docs, demo docs, Rust API docs, and WASM package metadata use `0.47.4`.
- `CITATION.cff` still says `version: "0.43.0"`.

Why this matters:

Research users are explicitly asked to cite the repository and version. A stale citation version makes public release metadata inconsistent and can cause researchers to cite the wrong artifact.

Required fix:

- Update `CITATION.cff` to `0.47.4`.
- Add a release-readiness test or script check that compares `CITATION.cff` against the workspace version.

### A-002: Candle backend crate has a module-splitting gap

Severity: refactor before release if this crate is part of the promoted surface.

Status: Complete in follow-up (2026-05-31 KST). `packages/rust/biors-backend-candle/src/lib.rs` was split into focused `backend`, `config`, `error`, `output`, and `tensor` modules while preserving the crate's public re-exports, and `scripts/check-module-size.py` now scans every `packages/rust/*/src` tree.

Evidence:

- `packages/rust/biors-backend-candle/src/lib.rs` is 366 lines.
- `scripts/check-module-size.py` enforces a 320-line limit only for:
  - `packages/rust/biors-core/src`
  - `packages/rust/biors/src`
- The Candle crate is public and release-published, but it is outside that module-size guard.

Why this matters:

The Candle backend combines config types, error type, backend execution, scoring, tensor loading, tensor validation, and Candle error mapping in one file. It is still understandable, but it is already past the repo's own module-size boundary and will become harder to review as soon as CUDA/Metal or additional model adapters are added.

Required fix:

- Split `biors-backend-candle/src/lib.rs` into focused modules, likely:
  - `config.rs`
  - `error.rs`
  - `output.rs`
  - `tensor.rs`
  - `backend.rs`
- Extend `scripts/check-module-size.py` to cover all Rust package `src` directories, not only `biors-core` and `biors`.

### A-003: Performance claims are pinned to an old benchmark baseline

Severity: release-blocking only if public copy claims current-version speedups; otherwise high-priority documentation/performance follow-up.

Status: Complete in follow-up (2026-05-31 KST). README now labels the `biors-core v0.20.0` numeric FASTA table as historical and explicitly says it is not current-version performance evidence for `0.47.4`; `cargo test --locked -p biors --test release_readiness stale_benchmark_artifact_is_labeled_historical_in_readme` guards that stale benchmark artifacts stay visibly historical.

Evidence:

- README quick proof table is still based on `biors-core v0.20.0`.
- Current workspace version is `0.47.4`.
- README says the `0.47.4` patch reduces repeated ASCII classification and adds fixed-length model-input benchmark coverage, but no new numeric benchmark artifact is committed for `0.47.4`.

Why this matters:

The docs correctly warn that numeric claims are pinned to the committed artifact, but the first-page proof table can still read like current release performance to researchers. Since the user is checking "release-ready for researchers", this should be tightened before public promotion.

Required fix:

- Either rerun and commit the benchmark artifact for `0.47.4`, or move the old numeric table lower and label it more visibly as historical.
- If keeping the current table, add a release-readiness check that fails when benchmark `environment.biors_core` is older than the promoted version and README contains numeric speedups.

### A-004: Python `build_model_inputs_checked` can silently accept non-model-ready tokenization

Severity: release blocker for the Python package.

Status: Complete in follow-up (2026-05-31 KST). Python `TokenizedProtein` now preserves `alphabet`, `valid`, `warnings`, and `errors` from core tokenization, and `build_model_inputs_checked` reconstructs core tokenized records from those fields instead of forcing `valid: true`; Python tests now cover ambiguous/invalid FASTA diagnostics and rejection of non-model-ready tokenization.

Evidence:

- `packages/rust/biors-python/src/lib.rs` exposes `PyTokenizedProtein` with only `id`, `tokens`, and `length`.
- The wrapper drops core `valid`, `warnings`, and `errors` from tokenization results.
- `build_model_inputs_checked` reconstructs core `TokenizedProtein` with:
  - `valid: true`
  - `warnings: vec![]`
  - `errors: vec![]`
- Therefore Python users can tokenize FASTA containing ambiguous or invalid residues, then call the checked builder and get model input instead of the Rust-core/CLI safety rejection.

Why this matters:

The core and CLI explicitly reject unresolved residue warnings/errors before producing model-ready input. Python is a researcher-facing surface and should preserve that safety behavior, especially for notebooks and downstream NumPy/PyTorch workflows.

Required fix:

- Add `valid`, `warnings`, and `errors` fields to `PyTokenizedProtein`, or expose a safer opaque tokenization wrapper that preserves the core record.
- Update Python tests with ambiguous/invalid FASTA proving `build_model_inputs_checked` rejects non-model-ready tokenized input.
- Update `packages/rust/biors-python/README.md` and `docs/python-api.md` if the exposed Python object shape changes.

### A-005: Python workflow API asks users to provide provenance hash manually

Severity: high-priority researcher-readiness fix for Python/notebook workflows.

Evidence:

- `prepare_workflow(input_hash, records, ...)` requires a caller-supplied `input_hash`.
- `packages/rust/biors-python/README.md` uses `input_hash="sha256:abc123"`.
- `docs/python-api.md` uses `input_hash="sha256:example"`.
- The CLI computes the input hash from actual FASTA bytes, but the Python API/docs make it easy to produce provenance that does not match the input.

Why this matters:

Reproducibility provenance is one of the project's main promises. Bio researchers using notebooks should not be nudged into fake or placeholder hashes in the first documented workflow.

Required fix:

- Add a Python API that accepts FASTA text/bytes and computes the hash internally, for example `prepare_workflow_from_fasta(fasta_text, max_length, ...)`.
- Or expose `sha256_digest` in Python and update docs to compute the hash from the exact FASTA text before calling `prepare_workflow`.
- Replace placeholder hashes in `packages/rust/biors-python/README.md`, `docs/python-api.md`, and tests.

### A-006: `sha256_digest` is not always a raw file SHA-256 despite being used in file-checksum contexts

Severity: release blocker for reproducibility/checksum contract clarity.

Evidence:

- `biors_core::hash::sha256_digest(bytes)` first tries to parse `bytes` as JSON.
- If parsing succeeds, it serializes the JSON value and hashes the canonicalized JSON bytes.
- The same function is used in file/checksum contexts such as package asset verification, package skeleton checksums, tokenizer conversion output, dataset metadata, pipeline locks, and output diffs.
- Docs describe many of these as file SHA-256 or artifact SHA-256 values.

Why this matters:

For JSON artifacts, `sha256_digest` may not match `shasum -a 256 <file>`. Researchers and package authors will reasonably expect published `sha256:` fields to be byte-for-byte file checksums unless the contract explicitly says "canonical JSON hash".

Required fix:

- Split the API into two explicit functions:
  - `sha256_bytes_digest(bytes)` for raw artifact/file checksums.
  - `sha256_canonical_json_digest(bytes_or_value)` for semantic JSON comparisons.
- Audit every caller and schema/document field name.
- Update docs to distinguish raw file hashes from canonical JSON content hashes.
- Add regression tests proving JSON file checksums match the documented contract.

### A-007: WASM workflow output uses `sha256:` input hashes while the public workflow schema expects `fnv1a64:`

Severity: release blocker for the WASM package if schema compatibility is part of the public contract.

Evidence:

- `packages/rust/biors-wasm/src/workflow.rs` computes `input_hash` with `biors_core::hash::sha256_digest(&fasta_bytes)`.
- `schemas/sequence-workflow-output.v0.json` requires `provenance.input_hash` to match `^fnv1a64:[0-9a-f]{16}$`.
- CLI workflow output uses `fnv1a64:` and has tests around that contract.
- WASM tests currently check only that `runWorkflow` returns `Ok`, not that the returned JSON matches the workflow schema.

Why this matters:

Docs describe the WASM workflow as exposing the standard preprocessing workflow. If the same workflow payload has a different hash algorithm by binding, downstream validators and reproducibility tooling will see incompatible output.

Required fix:

- Use the same stable input hasher as CLI/core FASTA-backed workflow output, or intentionally version the WASM schema separately.
- Add WASM tests that inspect `workflow.provenance.input_hash`.
- Add schema validation coverage for WASM workflow output if it is expected to match `sequence-workflow-output.v0.json`.

### A-008: WASM numeric arguments are cast without integer validation

Severity: correctness hardening before broad WASM/JS promotion.

Evidence:

- `packages/rust/biors-wasm/src/workflow.rs` reads `maxLength` with `as_f64()` and casts to `usize`.
- `padTokenId` is read with `as_f64()` and casts to `u8`.
- The checks reject negative and out-of-range values, but they do not reject fractional values.

Why this matters:

JavaScript callers can accidentally pass `maxLength: 8.7` or `padTokenId: 21.9`; the current binding truncates silently. For scientific preprocessing, parameter coercion should be explicit and deterministic.

Required fix:

- Require `Number.isInteger` semantics in the Rust-side checks, for example `f.fract() == 0.0`.
- Add WASM tests for fractional `maxLength`, fractional `padTokenId`, missing fields, and out-of-range values.

### A-009: Root-level WASM design document is stale and likely should not ship as current docs

Severity: documentation cleanup before release.

Status: Complete in follow-up (2026-05-31 KST). The unreferenced root-level `biors-wasm-design-v0.45.0.md` file was removed; current WASM public documentation remains in `docs/wasm-api.md`, `docs/wasm-readiness.md`, and `packages/rust/biors-wasm/README.md`.

Evidence:

- `biors-wasm-design-v0.45.0.md` lives at repository root, outside `docs/`.
- It describes `packages/rust/biors-wasm` as commented out of the workspace and "optional, not default".
- Current `Cargo.toml` includes `packages/rust/biors-wasm` as a workspace member.
- Current release workflow builds, tests, and publishes the WASM npm package.

Why this matters:

The file reads like a current design/source-of-truth document but contains stale implementation guidance. It can confuse contributors and release reviewers about whether WASM is implemented, optional, or published.

Required fix:

- Either delete it if it has served its purpose, or move it under a historical design/spec folder with a clear "historical design, superseded by docs/wasm-api.md and docs/wasm-readiness.md" notice.
- Keep current WASM public docs in `docs/wasm-api.md`, `docs/wasm-readiness.md`, and `packages/rust/biors-wasm/README.md`.

### A-010: WASM npm package does not include license files

Severity: packaging cleanup before npm release promotion.

Status: Complete in follow-up (2026-05-31 KST). `scripts/build-wasm-npm-package.sh` now copies both root license files into the generated npm package, suppresses the build-time missing-license warning with temporary crate-local license copies, and verifies `npm pack --dry-run --json` contains `LICENSE-APACHE` and `LICENSE-MIT`.

Evidence:

- `scripts/build-wasm-npm-package.sh` completed successfully.
- `wasm-pack` emitted: `License key is set in Cargo.toml but no LICENSE file(s) were found; Please add the LICENSE file(s) to your project directory`.
- The npm pack dry-run listed README, `.wasm`, JS, generated `.d.ts`, `index.d.ts`, and `package.json`, but no `LICENSE-MIT` or `LICENSE-APACHE`.

Why this matters:

The npm package declares `MIT OR Apache-2.0`, but consumers inspecting the npm tarball do not get the license texts. This is not a functional runtime issue, but it is avoidable release/package hygiene.

Required fix:

- Copy root `LICENSE-MIT` and `LICENSE-APACHE` into `packages/rust/biors-wasm/pkg` during `scripts/build-wasm-npm-package.sh`.
- Add those files to `packages/rust/biors-wasm/package.json` `files`.
- Add a release workflow/script check that verifies npm pack contents include the license files.

### A-011: Rust API docs overstate `no_std` posture

Severity: documentation correctness fix before public promotion.

Status: Complete in follow-up (2026-05-31 KST). `docs/rust-api.md` now describes `biors-core` as dependency-light, deterministic, and `std`-based, with WASM compatibility covered by the existing `wasm32-unknown-unknown` check rather than a `no_std` contract.

Evidence:

- `docs/rust-api.md` says `biors-core` keeps everything "`no_std`-friendly where possible".
- `biors-core` currently exposes and uses `std` surfaces such as `std::io::BufRead`, `std::io::Error`, `std::error::Error`, and reader-based FASTA APIs.
- There is no `#![no_std]` mode or feature-gated `std` policy in `biors-core`.

Why this matters:

Researchers and downstream crate authors may interpret this as embedded/WASM/no-allocator readiness. The actual WASM guarantee is narrower and already documented separately.

Required fix:

- Replace the `no_std-friendly` claim with a precise statement: dependency-light, deterministic, and WASM-checked for `biors-core`.
- If `no_std` is a future goal, move it to a roadmap/deferred section rather than current API docs.

### A-012: Current main version is already published on all configured package registries

Severity: release-process blocker if planning to tag/publish from this exact main state.

Evidence:

- `cargo publish --locked --dry-run -p biors-core` warns `crate biors-core@0.47.4 already exists on crates.io index`.
- The same warning appears for `biors-mcp-server@0.47.4`, `biors-backend-candle@0.47.4`, and `biors@0.47.4`.
- `cargo search`, `pip index versions biors`, and `npm view @bio-rs/biors-wasm version` all report `0.47.4`.
- Dry-run package verification still passes for all four crates.

Why this matters:

If a new tag is created from current main without a version bump, the release workflow will try to publish already-existing crate, PyPI, and npm versions and fail. This is normal if `main` is already the released state, but it is not a valid "next release" state.

Required fix:

- Before the next release, either confirm this is only an audit of an already-published `0.47.4`, or prepare a new version with lockstep metadata updates.
- Add a pre-tag check that compares workspace versions against crates.io/npm/PyPI and warns when the version is already published.

### A-013: `dataset inspect` still materializes full files and all records

Severity: performance improvement for large research datasets.

Evidence:

- `packages/rust/biors/src/cli/dataset.rs` uses `std::fs::read(&file.path)` to load the full file into memory.
- It then calls `parse_fasta_records_reader(Cursor::new(&bytes))`, which returns `Vec<ProteinSequence>`.
- It then builds a `Vec<DatasetSample>` for every FASTA record before output serialization.

Why this matters:

For small examples this is fine. For real lab datasets, `dataset inspect` can become memory-heavy because it holds raw file bytes, normalized records, sample metadata, and final JSON output in memory at once. This is inconsistent with the project's strongest large-file story in validation/batch paths.

Required fix:

- Add a streaming dataset inspection path that computes raw file hash and sample metadata in one pass.
- Avoid retaining full `ProteinSequence` records when only `id`, `record_index`, and sequence length are needed.
- Document that output JSON itself can still be large when sample-level mapping is requested; consider a summary-only option for very large datasets.

### A-014: Benchmark artifacts do not cover the feature surface added after the old FASTA baseline

Severity: benchmark/release-readiness gap.

Evidence:

- The committed public benchmark artifact is `benchmarks/fasta_vs_biopython.json`, generated for `biors-core v0.20.0`.
- `scripts/benchmark_fasta_vs_biopython.py` records only three workloads:
  - `pure_parse`
  - `parse_plus_validation`
  - `parse_plus_tokenization`
- `scripts/check-benchmark-artifact.py` enforces only those three workloads and four FASTA shape profiles.
- Newer public surfaces now include fixed-length model input, workflow, pipeline, dataset inspect, Python bindings, WASM bindings, MCP/service contracts, package validation/bridge, and the optional Candle backend.
- `packages/rust/biors-core/benches/fasta_workloads.rs` has a Criterion-only `model_input_fixed_length` benchmark, but that result is not committed into the public JSON/Markdown benchmark artifact.
- `packages/rust/biors-backend-candle/benches/candle_linear_probe.rs` is also Criterion-only and not part of the release-readiness benchmark artifact.

Why this matters:

The current performance proof is still useful for core FASTA parsing/tokenization, but it no longer represents the broader researcher-facing release surface. When features are added, the benchmark matrix should grow with them or explicitly state that no performance claim is made for those features.

Required fix:

- Extend the committed benchmark artifact schema to include feature-specific benchmarks, at least:
  - core fixed-length model input construction
  - CLI `workflow` end-to-end path
  - CLI `dataset inspect` on many-file and large-file inputs
  - Python binding overhead for parse/tokenize/model-input calls
  - WASM `runWorkflow`/tokenization overhead under Node
  - Candle CPU linear-probe smoke benchmark, clearly labeled as synthetic
- Update `scripts/check-benchmark-artifact.py` so those workloads are required when the corresponding feature is promoted in README/docs.
- Split "numeric public claim" benchmarks from "regression guard" benchmarks if not every result should appear on the README.
- Reword README until the committed artifact actually contains the fixed-length model-input numbers it references.

### A-015: Old hyperfine benchmark scripts should be deleted or merged into the current benchmark path

Severity: cleanup before release.

Evidence:

- `scripts/benchmark_hyperfine.py` and `scripts/biopython_bench.py` reference only each other.
- They are not listed in `scripts/check.sh` or `scripts/check-fast.sh` Python syntax checks.
- They are not the README-documented benchmark path; README points to `scripts/benchmark_fasta_vs_biopython.py`.
- `scripts/benchmark_hyperfine.py` writes to `benchmarks/fasta_vs_biopython.json` and `benchmarks/fasta_vs_biopython.md`, but its output shape is a small hyperfine-only object/table and does not match the checked `biors.benchmark.fasta_vs_biopython.v1` artifact schema.
- It requires the external `hyperfine` binary, while the maintained script uses the in-repo Python timing/report path.

Why this matters:

This is an outdated code path that can overwrite the committed benchmark artifact with an invalid schema. It also creates confusion about which benchmark is authoritative.

Required fix:

- Delete `scripts/benchmark_hyperfine.py` and `scripts/biopython_bench.py` if the current benchmark script fully replaced them.
- Or fold the useful hyperfine mode into `scripts/benchmark_fasta_vs_biopython.py` behind an explicit optional flag that still writes the current schema.
- If retained temporarily, add both scripts to syntax checks and document that they are experimental and must not overwrite public artifacts.

### A-016: Public `biors_core::fasta_scan` module exposes an internal implementation detail

Severity: pre-1.0 public API cleanup.

Evidence:

- `packages/rust/biors-core/src/lib.rs` declares `pub mod fasta_scan`.
- `docs/rust-api.md` says `fasta_scan` has no public API and all items are `pub(crate)`.
- `packages/rust/biors-core/src/fasta_scan.rs` contains only internal scanner traits/functions/types; public callers are told to use `fasta` and `tokenizer`.

Why this matters:

Even if the module has no public items, the module path itself is part of the public crate surface and appears in generated docs. Before 1.0, unnecessary public paths should be removed so researchers and downstream developers see the intended API.

Required fix:

- Change `pub mod fasta_scan` to `mod fasta_scan`.
- Keep the public documentation focused on `fasta`, `tokenizer`, and reader APIs.
- If there is a future public scanner API, expose a deliberate small type/function set instead of the internal module.

### A-017: `package::checksum` is a duplicate re-export module with an `unused_imports` allow

Severity: cleanup/refactor before public API freeze.

Evidence:

- `packages/rust/biors-core/src/package/checksum.rs` contains only:
  - `#![allow(unused_imports)]`
  - `pub use crate::hash::{is_sha256_checksum, sha256_digest};`
- `packages/rust/biors-core/src/package.rs` then re-exports those two items through `pub use checksum::{...}`.
- The main public hash API already exists at `biors_core::hash::{sha256_digest, is_sha256_checksum}`.
- No in-repo code or docs reference `biors_core::package::sha256_digest` or `biors_core::package::is_sha256_checksum`.

Why this matters:

This adds public API clutter and requires a lint suppression for code that only exists as an alias. Since checksum semantics already need cleanup in A-006, this is a good time to remove the duplicate path or document it intentionally.

Required fix:

- Prefer deleting `package/checksum.rs` and removing the `pub use checksum::{...}` line from `package.rs`.
- If the package-level checksum path is intentionally public, remove the lint suppression and document why the duplicate API exists.
- Coordinate this with the raw-vs-canonical SHA-256 split from A-006.

### A-018: Dependency minimization needs an explicit release gate

Severity: release hygiene / future maintenance risk.

Evidence:

- `biors-core` remains relatively small: normal dependencies are `serde`, `serde_json`, and `sha2`.
- Heavy or platform-specific integrations are currently isolated into separate crates:
  - `biors-backend-candle` depends on `candle-core`
  - `biors-mcp-server` depends on `rmcp` and `tokio`
  - `biors-python` depends on `pyo3`
  - `biors-wasm` depends on `wasm-bindgen`, `js-sys`, and `console_error_panic_hook`
- The repo has no automated dependency-minimization check for unused dependencies, duplicate versions, license compatibility, or unexpectedly heavy transitive dependency growth.
- `cargo tree --workspace -d` reports duplicate transitive versions such as `base64`, `darling`, `hashbrown`, `itertools`, `nom`, `serde_core`, and `thiserror`; many come from isolated integration/dev dependencies, but there is no tracked budget or review note explaining which duplication is acceptable.

Why this matters:

For researcher tooling, dependency weight affects install reliability, build time, auditability, and downstream adoption in locked lab/cluster environments. The current architecture is mostly well-isolated, but there is no guardrail preventing future feature work from pulling heavy dependencies into `biors-core` or the CLI accidentally.

Required fix:

- Add a dependency policy to release docs:
  - `biors-core` should stay dependency-light and never depend on runtime, CLI, Python, WASM, MCP, or Candle crates.
  - Heavy integrations must stay isolated in optional/package-specific crates.
  - New dependencies require a short justification in PR/release notes.
- Add automated checks where practical:
  - `cargo tree --workspace` snapshot or dependency budget review before release
  - unused dependency detection, for example `cargo machete` if adopted
  - license/advisory checks, for example `cargo deny` if adopted
- Add a release checklist item to review `Cargo.lock` for duplicate major versions and unexpected transitive growth.

### A-019: WASM package enables `console_error_panic_hook` by default

Severity: dependency-size cleanup before broad npm/browser promotion.

Evidence:

- `packages/rust/biors-wasm/Cargo.toml` defines `console_error_panic_hook` as optional.
- The same file sets `default = ["console_error_panic_hook"]`.
- `packages/rust/biors-wasm/src/lib.rs` installs the hook from a `#[wasm_bindgen(start)]` function when the feature is enabled.
- `cargo tree -p biors-wasm --target wasm32-unknown-unknown --edges normal` shows `console_error_panic_hook` in the normal dependency tree by default.

Why this matters:

The panic hook is useful for debugging, but it is not required for runtime functionality. For a production npm package, the default should be as small and deterministic as possible, with debugging hooks opt-in unless there is a clear product reason to ship them.

Required fix:

- Consider changing WASM default features to empty and documenting `--features console_error_panic_hook` for debug builds.
- Compare package `.wasm` size before/after and record the decision.
- If the hook remains default, document the tradeoff in `docs/wasm-readiness.md` and `packages/rust/biors-wasm/README.md`.

### A-020: Python wheels and binary release archives do not currently include license files

Severity: packaging cleanup before release promotion.

Evidence:

- Local `maturin build` produced `biors-0.47.4-cp39-abi3-macosx_11_0_arm64.whl`.
- Inspecting that wheel showed only `biors/biors.abi3.so` and `.dist-info/METADATA` among license-relevant files; no `LICENSE-MIT` or `LICENSE-APACHE` file was included.
- `.github/workflows/release.yml` packages release binaries by copying only `target/.../release/biors` into `dist/biors`, then creating a tarball from that single file.
- The npm license-file gap is already captured in A-010.

Why this matters:

All public packages declare `MIT OR Apache-2.0`. Researchers and institutional users often inspect package artifacts directly; each distributed artifact should include the actual license texts.

Required fix:

- Add root `LICENSE-MIT` and `LICENSE-APACHE` to the Python wheel/sdist package configuration.
- Include both license files in binary release tarballs.
- Add artifact-content checks for Python wheel, sdist, npm tarball, and binary tarballs before publish.

### A-021: `publish-crates` release job does not install a Rust toolchain

Severity: release workflow hardening.

Evidence:

- `.github/workflows/release.yml` installs Rust in `release-readiness`, `build-python-wheels`, `publish-wasm-npm`, and `build-release-binaries`.
- The `publish-crates` job runs `cargo publish` but only checks out the repo and configures the Cargo cache.
- GitHub-hosted runners often have Rust available, but the workflow is less reproducible than the other jobs and can drift with runner images.

Why this matters:

Publishing is the most side-effectful part of the release. It should not depend on implicit runner state when every other Rust job pins setup through `dtolnay/rust-toolchain@stable`.

Required fix:

- Add an explicit Rust setup step to `publish-crates`, matching the release-readiness job.
- Extend `scripts/check-release-workflow.py` so it verifies Rust setup exists in every job that runs `cargo`.

### A-022: Final release checklist is stale after this audit

Severity: documentation cleanup before release.

Evidence:

- `docs/final-release-checklist.md` says: "No known breaking cleanup is deferred for the current pre-1.0 contract set."
- This audit now has several public-surface cleanup items:
  - `fasta_scan` public module cleanup
  - package checksum alias cleanup
  - raw-vs-canonical SHA-256 contract cleanup
  - Python tokenization safety shape change
  - WASM workflow schema/hash alignment
- The checklist also does not yet mention dependency minimization, package artifact license files, registry already-published checks, Python/WASM artifact inspection, or updated benchmark coverage.

Why this matters:

The release checklist should be a living gate, not a stale confidence document. If it says no cleanup is deferred while known cleanup exists, reviewers may incorrectly approve a tag.

Required fix:

- Update `docs/final-release-checklist.md` to list this audit file as the current blocker queue until resolved.
- Add explicit checklist gates for:
  - dependency minimization / `Cargo.lock` review
  - benchmark artifact coverage matching promoted features
  - Python wheel/sdist content inspection
  - npm tarball content inspection
  - binary tarball license/readme inclusion
  - registry already-published version check
- Remove or rewrite the "No known breaking cleanup" sentence.

### A-023: MCP workflow parameters and output contract diverge from CLI/core workflow

Severity: release blocker for the MCP package if agent-callable workflows are promoted.

Evidence:

- `packages/rust/biors-mcp-server/src/server.rs` defines `WorkflowParams.kind` with allowed values `"auto"`, `"protein"`, `"dna"`, and `"rna"`.
- The MCP `workflow` tool never reads `params.kind`; it always calls the protein workflow path.
- The MCP `workflow` tool hardcodes:
  - `pad_token_id: 0`
  - `padding: NoPadding`
- The CLI `workflow` exposes padding controls and defaults to fixed-length padding.
- MCP workflow computes `input_hash` with `sha256_digest`, while the CLI workflow schema expects the `fnv1a64:` provenance format.
- `packages/rust/biors-mcp-server/tests/mcp_server.rs` tests only `doctor`, `tokenize`, and `validate`; it does not test `workflow` or `package_validate`.

Why this matters:

Agent-callable tooling is a public researcher-facing surface. A documented-but-ignored `kind` parameter and different workflow provenance/padding behavior make MCP results harder to compare with CLI/core workflow output.

Required fix:

- Either remove `kind` from MCP workflow until non-protein workflows exist, or use it and reject unsupported values clearly.
- Add `pad_token_id` and `padding` parameters to MCP workflow, matching CLI/core names and defaults.
- Align MCP workflow provenance hash with CLI workflow output or document a separate MCP contract.
- Add MCP tests for workflow success, invalid kind/profile, non-model-ready residues, and package validation.

### A-024: Python package marks itself typed but ships no usable type stubs

Severity: Python packaging/documentation cleanup.

Evidence:

- `packages/rust/biors-python/python/biors/py.typed` exists and is zero bytes, marking the package as typed under PEP 561.
- There is no `biors.pyi` or generated stub file for the native PyO3 module.
- `packages/rust/biors-python/python/biors/__init__.py` re-exports native classes/functions but provides no type annotations.

Why this matters:

Notebook users can still use the package interactively, but static type checkers and IDEs will not get the function/class signatures implied by `py.typed`. Shipping `py.typed` without stubs overstates the Python typing surface.

Required fix:

- Add a maintained `biors/__init__.pyi` or native module stub covering the public Python API.
- Or remove `py.typed` until the package actually provides typed interfaces.
- Add a packaging check that the wheel contains the chosen typing files.

### A-025: Python API tests are not part of the release workflow

Severity: release workflow coverage gap.

Evidence:

- `packages/rust/biors-python/tests/test_python_api.py` covers the Python API smoke path.
- `scripts/check.sh` compiles Python helper scripts but does not run these pytest tests.
- `.github/workflows/release.yml` builds Python wheels and sdist, then uploads/publishes them, but does not install a built wheel and run pytest.
- Manual audit verification installed the locally built wheel into a temporary venv and `pytest` passed, so this is a wiring gap rather than a currently failing test.

Why this matters:

The Python package is one of the main researcher-facing surfaces. A wheel can build successfully while import behavior, class exports, or runtime API behavior is broken. Release should verify the exact artifact that will be published.

Required fix:

- In the Python wheel build job, install each built wheel into a clean venv and run `python -m pytest packages/rust/biors-python/tests -q`.
- Add a lightweight local script for this so contributors can reproduce it before tagging.
- Include a test for the invalid-residue/model-input behavior from A-004 when that fix lands.

### A-026: WASM `WorkflowConfig` advertises `kind` and `profile`, but `runWorkflow` ignores both

Severity: release blocker for the WASM package if `runWorkflow` is promoted as a standard workflow API.

Evidence:

- `docs/wasm-api.md` documents `WorkflowConfig` with optional:
  - `kind?: "auto" | "protein" | "dna" | "rna"`
  - `profile?: "protein-20" | "protein-20-special"`
- `packages/rust/biors-wasm/src/types.rs` embeds the same TypeScript fields.
- `packages/rust/biors-wasm/src/workflow.rs` only reads:
  - `fastaBytes`
  - `maxLength`
  - `padTokenId`
  - `padding`
- Therefore JS callers can pass `kind: "dna"` or `profile: "protein-20-special"` and receive protein/default-tokenizer workflow behavior without warning.
- WASM tests only assert `runWorkflow` returns `Ok`; they do not assert `kind`, `profile`, schema shape, or provenance behavior.

Why this matters:

Silent ignored configuration is especially risky in notebook/browser workflows. Researchers may believe they selected a sequence kind or tokenizer profile when the binding actually used hardcoded protein workflow defaults.

Required fix:

- Either implement `kind` and `profile` support in `runWorkflow`, or remove those fields from the public TypeScript/docs until supported.
- Reject unsupported fields explicitly rather than ignoring them silently.
- Add WASM tests that verify profile-specific special-token behavior, invalid `kind`/`profile` handling, and workflow provenance/schema shape.

### A-027: Final local release gate does not exercise Python/WASM package artifacts

Severity: release gate gap.

Evidence:

- `scripts/check-final-release.sh` runs:
  - `scripts/check-release-workflow.py`
  - `scripts/check.sh`
  - `cargo build --locked --release -p biors`
  - `BIORS_BIN=target/release/biors sh scripts/launch-demo.sh`
  - `scripts/check-install-smoke.sh`
- It does not run:
  - `maturin build` for `biors-python`
  - install-built-wheel plus `pytest`
  - `wasm-pack test --node packages/rust/biors-wasm`
  - `scripts/build-wasm-npm-package.sh`
  - npm tarball content checks
  - `cargo publish --locked --dry-run` for the publish crates
- The manual audit ran several of those checks and found real package hygiene gaps, including the npm license warning and missing Python wheel license files.

Why this matters:

The local final gate is the command a maintainer is told to run before tagging. It currently proves the Rust CLI/core path but not the exact Python/WASM/npm/crates artifacts that tag releases publish.

Required fix:

- Extend `scripts/check-final-release.sh` or add a separate `scripts/check-package-artifacts.sh` that runs the publish-artifact checks locally.
- Fail the gate on package content problems, not only build/test failures.
- Keep slow/network-sensitive registry existence checks separate but documented, so maintainers can run them intentionally before tagging.

### A-028: Local ignored artifacts should be cleaned before release work

Severity: local cleanup / pre-release hygiene.

Evidence:

The main source audit used an isolated worktree, but the working repo at `/Users/chn_m1n/bio-rs` currently has ignored local artifacts, including:

- `.DS_Store` files under root, `.github/`, `docs/`, `examples/`, `packages/`, and `scripts/`
- Python caches:
  - `.pytest_cache/`
  - `examples/python/__pycache__/`
  - `packages/rust/biors-python/.pytest_cache/`
  - `packages/rust/biors-python/python/biors/__pycache__/`
  - `packages/rust/biors-python/tests/__pycache__/`
  - `scripts/__pycache__/`
- local Python environment: `.venv/`
- local PyO3 build artifact: `packages/rust/biors-python/python/biors/biors.abi3.so`
- generated WASM package output: `packages/rust/biors-wasm/pkg/`
- Rust build output: `target/`

Why this matters:

These files are ignored and not part of the source release, but they make local review noisier and can mask accidental generated-file assumptions. The generated Python extension and WASM `pkg/` directory should always be treated as build artifacts, not source-of-truth files.

Required fix:

- Clean ignored local artifacts before final release prep or before creating a clean audit branch/worktree.
- Do not commit these files.
- Keep `.gitignore` entries as-is, but add a release-prep note to run `git status --ignored` or a documented cleanup command before packaging.

### A-029: Security/advisory/license audit is not configured

Severity: release blocker for a public researcher-facing package set.

Evidence:

- No `deny.toml`, `cargo-deny.toml`, `audit.toml`, or equivalent advisory/license policy file exists at repo root.
- Local availability checks reported:
  - `cargo-audit missing`
  - `cargo-deny missing`
  - `cargo-machete missing`
  - `cargo-udeps missing`
- `scripts/check.sh` and `scripts/check-final-release.sh` do not run a dependency advisory or license audit.
- The workspace includes public packages with substantial transitive dependency trees through Candle, MCP, PyO3, WASM, JSON Schema test tooling, and Criterion benchmarks.

Why this matters:

For biological research tooling, users may install packages into institutional clusters, notebooks, CI pipelines, and internal regulated environments. Passing tests is not enough; releases should also document that dependency advisories and license compatibility were checked.

Required fix:

- Add an explicit dependency audit policy, preferably `cargo deny` with:
  - advisory check
  - license allowlist compatible with `MIT OR Apache-2.0`
  - duplicate dependency review policy
  - source/registry policy if needed
- Add the audit to CI or at least to `scripts/check-final-release.sh`.
- If the project intentionally avoids adding new tooling, document the manual advisory/license review process and record it in the release checklist.

### A-030: Rust toolchain/MSRV is not pinned in package metadata

Severity: release reproducibility gap.

Evidence:

- There is no `rust-toolchain.toml` or `rust-toolchain` file.
- `cargo metadata --locked --format-version 1 --no-deps` reports `rust_version = None` for all six workspace packages:
  - `biors`
  - `biors-backend-candle`
  - `biors-core`
  - `biors-mcp-server`
  - `biors-python`
  - `biors-wasm`
- `docs/install.md` describes Rust MSRV as "latest stable Rust in CI".
- CI uses `dtolnay/rust-toolchain@stable`, so the effective compiler can move between releases.

Why this matters:

Research users and downstream package maintainers need to know what Rust version is required. "Latest stable" is a moving target and makes release reproduction harder, especially when build failures appear only after a future stable compiler changes.

Required fix:

- Decide the actual MSRV for the current release line.
- Add `rust-version` to all published crate manifests or workspace package metadata if supported by the chosen Cargo version.
- Add a CI job or local check that verifies the workspace on the declared MSRV.
- Keep a separate latest-stable job if desired, but do not make it the only compatibility statement.

### A-031: GitHub Actions are version-tag pinned, not commit-SHA pinned

Severity: supply-chain hardening follow-up before high-trust release automation.

Evidence:

- `.github/workflows/ci.yml` and `.github/workflows/release.yml` use action references such as:
  - `actions/checkout@v6`
  - `dtolnay/rust-toolchain@stable`
  - `Swatinem/rust-cache@v2`
  - `actions/setup-python@v6`
  - `actions/upload-artifact@v7`
  - `actions/download-artifact@v8`
  - `pypa/gh-action-pypi-publish@release/v1`
  - `actions/setup-node@v6`
- These are major/channel tags rather than immutable commit SHAs.

Why this matters:

Release jobs publish crates, PyPI packages, npm packages, and GitHub release binaries. Major-tag pinning is common, but for a high-trust release path, immutable SHA pinning reduces supply-chain drift and makes releases easier to reproduce and audit.

Required fix:

- Consider pinning release-workflow actions to commit SHAs, especially publish and artifact actions.
- If keeping major tags, document that decision and add Dependabot/Renovate or a scheduled review for action updates.
- At minimum, treat the release workflow more strictly than ordinary PR CI.

### A-032: Schema coverage is strong for CLI, but not for binding outputs

Severity: cross-surface contract gap.

Evidence:

- Rust CLI schema coverage is broad: `packages/rust/biors/tests/schema_contract.rs` and `schema_package_contract.rs` validate CLI envelopes and many public schemas.
- JSON syntax scan found no invalid `schemas/*.json`.
- WASM tests do not validate `runWorkflow` output against `schemas/sequence-workflow-output.v0.json`.
- Python tests do not validate package/runtime JSON strings or workflow-like outputs against the same schemas.
- MCP tests do not validate tool outputs against the service or CLI schemas.

Why this matters:

The repository now promotes multiple user surfaces: CLI, Rust, Python, WASM, and MCP. A contract can stay valid in CLI while drifting in bindings, which is exactly what the WASM/MCP hash and ignored-parameter findings show.

Required fix:

- Add schema validation tests for WASM workflow output.
- Add Python tests that parse returned JSON strings and validate against package/runtime schemas.
- Add MCP tests for workflow/package outputs and either validate against shared schemas or document a distinct MCP contract.
- Add a release checklist item requiring schema parity review across every promoted binding.

### A-033: `.github/release_template.md` is not used by the release workflow

Severity: stale documentation/config cleanup.

Evidence:

- `.github/release_template.md` contains a hand-written release body template with sections for summary, changes, documentation, verification, closed issues, and next target.
- `.github/workflows/release.yml` creates releases with:
  - `gh release create "$TAG_NAME" --title "bio-rs ${TAG_NAME#v}" --generate-notes dist/*.tar.gz`
- The workflow does not pass `--notes-file .github/release_template.md`.
- No script references `.github/release_template.md`.

Why this matters:

The template looks authoritative, but release automation ignores it. That creates a documentation trap: maintainers may update a release template that never affects published release notes.

Required fix:

- Either delete `.github/release_template.md`, or wire it into release creation with an explicit notes generation step.
- If `--generate-notes` remains the source of truth, document that and remove the unused template.
- If a manual release body is required, add a pre-release check that verifies required sections are present.

### A-034: PR and issue templates lag behind the promoted package/binding surface

Severity: repository process cleanup.

Evidence:

- `.github/pull_request_template.md` asks for `scripts/check.sh`, `cargo test --workspace`, docs, and benchmarks.
- It does not mention:
  - Python wheel build/install/pytest
  - WASM `wasm-pack test` and npm pack contents
  - MCP tool tests
  - schema parity across bindings
  - dependency/advisory/license audit
  - package artifact license/checksum inspection
- `.github/ISSUE_TEMPLATE/benchmark_performance_idea.md` lists only core library and CLI end-to-end surfaces.
- The current implemented/promoted surface includes Python, WASM, MCP, package validation/bridge, service contract, and Candle backend.

Why this matters:

Repository templates shape contributor behavior. If templates still reflect the smaller core/CLI project, contributors will miss validation and benchmark updates for the newer release surfaces.

Required fix:

- Update the PR template with conditional checkboxes for Python, WASM/npm, MCP, package artifacts, schema parity, and dependency audit.
- Update the benchmark issue template to include model-input, workflow, dataset inspect, Python, WASM, MCP/service, package, and Candle benchmark surfaces.
- Keep the template concise so it remains usable, but ensure it asks about every promoted release surface.

### A-035: Release build tools are not version-pinned

Severity: release reproducibility blocker.

Evidence:

- `.github/workflows/release.yml` installs maturin with:
  - `python -m pip install --upgrade maturin`
- It installs wasm-pack with:
  - `cargo install wasm-pack --locked`
- It uses `dtolnay/rust-toolchain@stable`.
- It uses Node.js major version `24`.
- Package metadata constrains Python build-system maturin as `maturin>=1.0,<2.0`, but the release workflow still installs the latest compatible/current maturin at run time.

Why this matters:

Publishing artifacts should be reproducible. Unpinned release tool versions can change wheel tags, npm output, generated JS glue, metadata, or build behavior without any source change.

Required fix:

- Pin release tool versions explicitly:
  - Rust toolchain version or declared MSRV plus latest-stable validation
  - maturin version
  - wasm-pack version
  - Node.js patch or accepted major with documented policy
- Record tool versions in release notes or artifact metadata.
- Extend `scripts/check-release-workflow.py` to reject unpinned release tools unless explicitly allowed by policy.

### A-036: Release artifacts lack checksums, signatures, and provenance attestations

Severity: release/package trust blocker for binary downloads.

Evidence:

- `.github/workflows/release.yml` uploads binary tarballs and attaches `dist/*.tar.gz` to the GitHub release.
- The workflow does not generate `.sha256` files for release tarballs.
- The workflow does not sign artifacts or generate provenance/SLSA attestations.
- `docs/install.md` documents binary archive names but not checksum verification.
- Python/npm/crates registries provide their own metadata, but GitHub binary archives currently have no repo-provided integrity material.

Why this matters:

Researchers downloading binaries should be able to verify exactly what they installed. This matters more because the CLI emits scientific preprocessing and provenance outputs.

Required fix:

- Generate SHA-256 checksum files for each binary archive and attach them to the GitHub release.
- Document checksum verification in `docs/install.md`.
- Consider GitHub artifact attestations or another signing/provenance mechanism for release binaries.
- Add a release workflow check that verifies the checksum files match the tarballs before release creation.

### A-037: Python binding implementation should be split by responsibility

Severity: refactor before expanding Python API.

Evidence:

- `packages/rust/biors-python/src/lib.rs` is 258 lines and contains:
  - all PyO3 class definitions
  - FASTA parsing/validation functions
  - tokenization wrappers
  - model-input wrapper
  - workflow wrapper
  - package validation/runtime bridge wrappers
  - padding policy parsing
  - module registration
- The file is still small enough to scan, but every new Python-facing API will add more unrelated responsibilities to the same binding module.

Why this matters:

The Python package is a first-class researcher surface. Keeping all binding concerns in one file makes it harder to review safety-sensitive behavior like tokenized validity propagation, workflow provenance, and JSON-returning package APIs.

Required fix:

- Split Python bindings into focused modules, for example:
  - `types.rs`
  - `sequence.rs`
  - `tokenizer.rs`
  - `model_input.rs`
  - `workflow.rs`
  - `package.rs`
  - `lib.rs` only for module registration
- Add tests around each boundary while splitting, especially for A-004 and A-005.

### A-038: Package conversion CLI mixes metadata, layout inference, path validation, serialization, and error mapping

Severity: refactor before adding more package migration behavior.

Evidence:

- `packages/rust/biors/src/cli/package_convert.rs` is 297 lines.
- It handles:
  - command orchestration
  - conversion input construction
  - research metadata extraction
  - package layout inference
  - relative path cleaning/validation
  - converted manifest serialization
  - CLI error mapping
- Several helpers are generic package-layout logic rather than CLI command orchestration.

Why this matters:

Package conversion is part of the public reproducibility story. As schema migration grows, this file will become a hotspot for bugs if layout inference and CLI output writing remain coupled.

Required fix:

- Move reusable metadata/layout inference into a focused module, for example `package_convert_layout.rs`.
- Keep `package_convert.rs` as the command runner that reads inputs, calls core conversion/layout helpers, writes optional output, and prints the envelope.
- Add direct tests for layout inference edge cases if the logic is moved out of CLI-only integration tests.

### A-039: Pipeline output assembly repeats status logic and should be made table-driven

Severity: moderate refactor.

Evidence:

- `packages/rust/biors/src/cli/pipeline_output.rs` manually builds similar `PipelineStep` values in:
  - `legacy_steps`
  - `planned_steps`
  - `config_steps`
  - `export_step`
- Validation/tokenization pass/fail status logic is duplicated between legacy and config paths.
- Stage names and operations are hardcoded in multiple functions.

Why this matters:

The current file is correct enough, but adding new stages or changing status semantics will require edits in several places. A table-driven stage definition would reduce drift between dry-run plans and executed pipeline output.

Required fix:

- Introduce a small internal stage descriptor for parse/normalize/validate/tokenize/export.
- Derive planned and executed `PipelineStep` values from the same descriptor list.
- Keep `export_step` special only for model-input/output hash behavior.

### A-040: Large integration test files should be split by contract area

Severity: test maintainability refactor.

Evidence:

- Large test files include:
  - `packages/rust/biors-core/tests/package.rs`: 629 lines
  - `packages/rust/biors-core/tests/runtime.rs`: 397 lines
  - `packages/rust/biors/tests/schema_contract.rs`: 378 lines
  - `packages/rust/biors/tests/release_readiness.rs`: 314 lines
  - `packages/rust/biors/tests/cli_package.rs`: 300 lines
- These files cover many separate contracts: manifests, artifact checksums, runtime compatibility, schema validation, release readiness, package CLI behavior.

Why this matters:

Large integration files make it harder to see which behavior changed when a test fails. They also hide gaps like binding schema parity because the CLI schema suite looks comprehensive while non-CLI surfaces remain separate.

Required fix:

- Split the largest integration tests by contract area:
  - package manifest structure
  - package artifact validation
  - runtime bridge compatibility
  - schema validation for CLI
  - release readiness scripts/workflows
- Keep shared helpers in `tests/common` or small local helper modules.
- Do this as test-only refactor commits with no behavior changes.

### A-041: CLI docs use snake_case padding values, but the actual CLI accepts kebab-case

Severity: release-blocking documentation/contract mismatch.

Evidence:

- `docs/cli-contract.md` documents:
  - `--padding fixed_length|no_padding` for `model-input`
  - `--padding fixed_length|no_padding` for `workflow`
  - `--padding fixed_length|no_padding` for `pipeline`
- Actual CLI help reports:
  - `[possible values: fixed-length, no-padding]`
- Running `target/release/biors workflow --max-length 8 --padding fixed_length examples/protein.fasta` exits with code `2` and says `invalid value 'fixed_length'`.
- Running the same command with `--padding fixed-length` succeeds.
- The snake_case values are valid in JSON/API surfaces such as pipeline config, Python, WASM, and output schemas, so this is specifically a CLI documentation mismatch.

Why this matters:

Researchers following the CLI contract doc will copy a command that fails. This is exactly the kind of small contract mismatch that erodes trust before release.

Required fix:

- Update CLI docs to use `fixed-length|no-padding` for command-line flags.
- Keep JSON/API docs using `fixed_length|no_padding` where that is the actual serialized contract.
- Add a CLI contract test or doc-generation check that compares documented value enums against `biors <command> --help`.

### A-042: `CONTRIBUTING.md` still describes the older core/CLI-only workflow

Severity: contributor documentation cleanup.

Evidence:

- `CONTRIBUTING.md` prerequisites mention Rust stable and `wasm32-unknown-unknown` for the core crate check.
- The recommended workflow and PR checklist focus on `scripts/check.sh`, benchmark docs, README/docs, and performance claims.
- It does not mention when contributors must run:
  - Python wheel/test checks
  - WASM `wasm-pack` checks
  - npm pack checks
  - MCP tests
  - dependency/advisory/license audit
  - package artifact content checks
- The scope priority list also stops at CLI/core/package validation and does not reflect Python/WASM/MCP as maintained release surfaces.

Why this matters:

If contributors follow `CONTRIBUTING.md`, they can make changes to a promoted binding or package surface without running the checks needed for that surface.

Required fix:

- Add a "surface-specific checks" section:
  - Rust/CLI/core
  - Python
  - WASM/npm
  - MCP
  - package/release artifacts
  - benchmarks
  - dependency audit
- Link the final release checklist for release-affecting changes.
- Keep fast/default workflow simple but make escalation criteria explicit.

### A-043: `docs/phase7-status.md` overstates researcher-grade readiness

Severity: documentation correctness cleanup.

Evidence:

- `docs/phase7-status.md` says Phase 7 is implemented through `0.47.x`.
- It says Python bindings are "Implemented and release-workflow published".
- It says WASM bindings are "Implemented and npm-published".
- It says MCP tooling is "Implemented".
- This audit found release-blocking correctness/contract issues in all three surfaces:
  - Python model-input safety and provenance API gaps
  - WASM workflow hash/schema/ignored config gaps
  - MCP workflow ignored kind/padding/provenance gaps
- It also says the shipped Phase 7 surface is intended for real preprocessing and package integration work, while the release verdict here is not-ready.

Why this matters:

Status docs should distinguish "implemented enough to build/publish" from "researcher-grade and release-ready." The current wording makes incomplete binding contracts sound fully ready.

Required fix:

- Change the status table to include caveats or a current audit status column.
- Link this audit file until blockers are resolved.
- Use terms like "implemented, needs contract hardening" rather than "researcher-grade" for Python/WASM/MCP until A-004, A-007, A-023, and A-026 are fixed.

### A-044: `docs/public-contract-1.0-candidates.md` includes unstable/internal candidates

Severity: documentation/API stabilization cleanup.

Evidence:

- `docs/public-contract-1.0-candidates.md` lists `fasta_scan`-adjacent low-level APIs indirectly through the broad Rust API surface and includes large unstable areas such as:
  - `ExternalProcessBackend`
  - `ExternalProcessConfig`
  - optional `biors-backend-candle` types
  - runtime bridge provider expansion candidates
- This audit separately found `biors_core::fasta_scan` is publicly exposed despite being documented as having no public API.
- The same document does not list Python/WASM/MCP binding contracts in the same level of detail, even though those are now promoted surfaces.

Why this matters:

The 1.0 candidate list should be a narrowing tool. If it includes too much experimental/runtime/integration surface and omits binding-specific contracts, it will not help decide what to stabilize or refactor before 1.0.

Required fix:

- Split the document into:
  - stable-candidate core contracts
  - CLI/JSON schemas
  - binding contracts
  - experimental/runtime integration contracts
- Remove internal-only modules from candidate status.
- Mark Candle/external-process/runtime provider APIs as experimental unless they are intentionally stabilized.

### A-045: Root `.gitignore` misses common local Python artifacts

Severity: local cleanup/config hardening.

Evidence:

- Root `.gitignore` includes:
  - `/target/`
  - `packages/rust/biors-wasm/pkg/`
  - `.venv-bench/`
  - `**/*.rs.bk`
  - `.DS_Store`
  - `__pycache__/`
  - `*.py[cod]`
- `packages/rust/biors-python/.gitignore` ignores binding-local `.pytest_cache/`, `.mypy_cache/`, build outputs, and native extension files.
- The root repo does not ignore common root-level Python artifacts such as:
  - `.venv/`
  - `.pytest_cache/`
  - `.mypy_cache/`
  - `.ruff_cache/`
- In this local checkout, `.venv/` and root `.pytest_cache/` are ignored by user/global git settings, not by the repository `.gitignore`.

Why this matters:

Contributors without the same global git ignore rules may see noisy untracked Python artifacts after running local wheel/pytest checks. Since Python checks should become part of release readiness, the repo should ignore common local Python tool outputs itself.

Required fix:

- Add root-level ignores for `.venv/`, `.pytest_cache/`, `.mypy_cache/`, `.ruff_cache/`, and possibly `dist/` if root-level package artifact scripts are added.
- Keep package-specific ignores for `biors-python` and `biors-wasm`, but make the root ignore sufficient for common contributor workflows.

### A-046: Direct core/binding APIs can treat empty sequences as valid/model-ready

Severity: researcher-facing correctness hardening.

Evidence:

- FASTA parsing rejects empty records in `packages/rust/biors-core/src/fasta.rs` by returning `BioRsError::MissingSequence` when the parsed sequence buffer is empty.
- The lower-level public validation path in `packages/rust/biors-core/src/sequence/validation.rs` does not reject empty `ProteinSequence` values. The test `validate_protein_sequence_empty_is_valid` explicitly asserts that an empty sequence is valid.
- `prepare_protein_model_input_workflow` in `packages/rust/biors-core/src/workflow.rs` accepts arbitrary `ProteinSequence` slices. If a caller constructs a `ProteinSequence { sequence: vec![] }` directly, validation has zero warnings/errors and the workflow can proceed to model input generation.
- Python/WASM/MCP bindings should be audited against this same direct-construction path, because binding users are not guaranteed to come through the FASTA parser.

Why this matters:

For bio researchers, a zero-length biological sequence is usually invalid input for model preparation, even if it is syntactically possible as an in-memory struct. Treating it as model-ready risks silent empty tensors/records and makes API behavior differ depending on whether the caller used FASTA text or direct records.

Required fix:

- Make non-empty sequence length an explicit validation invariant for public workflow/model-input paths.
- Decide whether the low-level `validate_protein_sequence` API should mark empty sequences invalid or whether workflow/model-input builders should reject them separately.
- Add tests for direct Rust API, Python, WASM, and MCP workflows with empty records.

### A-047: CLI workflow provenance records non-replayable padding arguments

Severity: release blocker for reproducibility claims.

Evidence:

- `packages/rust/biors/src/cli/args.rs` uses `clap::ValueEnum` for `PaddingArg`, so the CLI accepts kebab-case values such as `fixed-length` and `no-padding`.
- `packages/rust/biors/src/cli/workflow.rs` builds provenance invocation arguments via `padding_arg_value`.
- That helper returns `fixed_length` and `no_padding`.
- Earlier command verification showed `biors workflow --padding fixed_length ...` is rejected by clap, while `--padding fixed-length` works.

Why this matters:

Workflow outputs claim to carry invocation metadata for reproducibility, but the recorded command-line argument is not directly replayable. For a pre-release researcher tool, provenance must be copy/paste runnable or explicitly typed as a normalized internal enum rather than a CLI invocation.

Required fix:

- Change CLI provenance argument serialization to use the same values clap accepts: `fixed-length` and `no-padding`.
- Add a regression test that runs the generated workflow invocation arguments back through clap or an integration CLI call.
- Keep JSON schema enum naming separate if snake_case is intentionally used for serialized policy objects.

### A-048: Auto sequence-kind detection is overconfident for ambiguous protein/nucleotide sequences

Severity: researcher-facing correctness/documentation hardening.

Evidence:

- `packages/rust/biors-core/src/sequence/detection.rs` chooses a kind by minimizing invalid-symbol counts.
- If there is no protein-only evidence and no RNA-only `U` signal, ties prefer DNA.
- `ACGT` is valid as a protein sequence and as DNA, but the current auto path classifies it as DNA.
- `packages/rust/biors/src/cli/args.rs` defaults `batch validate` and `seq validate` to `--kind auto`, while `fasta validate` defaults to explicit protein.
- The validation output records the chosen `kind`, but does not expose confidence, ambiguity, or alternate possible kinds.

Why this matters:

Auto kind detection is useful, but bio researchers often work with short peptides, primers, motifs, and degenerate alphabets where multiple biological interpretations are valid. A deterministic tie-break can be acceptable only if the output makes ambiguity visible and docs warn users to pass an explicit kind for overlapping alphabets.

Required fix:

- Add ambiguity/confidence metadata to auto-detected validation output, or at least emit a warning issue when multiple kinds tie.
- Document the tie-break behavior and recommend explicit `--kind protein|dna|rna` for short or alphabet-overlapping inputs.
- Add tests for representative ambiguous inputs such as `ACGT`, `M`, `N`, `UUUU`, and mixed invalid symbols.

### A-049: `--json` does not cover clap parse errors

Severity: CLI contract hardening.

Evidence:

- `packages/rust/biors/src/main.rs` calls `Cli::parse()` before the program can inspect `cli.json`.
- Runtime errors returned from `cli::run` are printed through `print_json_error` when `--json` is set.
- Argument parse errors raised by clap exit before that path.
- Verified command:
  - `target/release/biors --json workflow --max-length 8 --padding fixed_length examples/protein.fasta`
  - exits with code 2 and prints clap text, not a JSON error envelope.

Why this matters:

The help text says `--json` emits machine-readable JSON errors. Automation and notebooks will commonly hit parse errors for invalid enum values, missing paths, or missing required arguments. Those failures currently bypass the documented JSON error contract.

Required fix:

- Replace `Cli::parse()` with `Cli::try_parse()`/manual error handling that can detect `--json` in raw args before formatting parse errors.
- Define a stable JSON code for CLI parse failures, for example `cli.invalid_arguments`.
- Add integration tests for parse errors with and without `--json`.

### A-050: Package manifest parser silently accepts unknown fields while schemas reject them

Severity: package contract hardening.

Evidence:

- `schemas/package-manifest.v1.json` uses `additionalProperties: false` at the top level and throughout nested objects.
- `packages/rust/biors-core/src/package/manifest.rs` defines the manifest structs without `#[serde(deny_unknown_fields)]`.
- `packages/rust/biors/src/input.rs` and Python/WASM package-validation entry points deserialize manifests with `serde_json::from_str` into these structs before validation.
- Unknown fields in JSON manifests are therefore ignored by the implementation unless an external schema validator is run separately.

Why this matters:

Researchers and package authors can typo a field, include stale fields from an older template, or think a new field is enforced when it is actually ignored. This creates a split between the published JSON Schema contract and the CLI/binding behavior.

Required fix:

- Add `#[serde(deny_unknown_fields)]` to all package manifest input structs, or run JSON Schema validation before deserializing/validating manifests.
- Add regression tests for unknown top-level, artifact, metadata, fixture, runtime, and pipeline-step fields.
- Make Python/WASM/MCP package validation use the same strict behavior.

### A-051: Runtime backend enum allows `external-process` but package schemas reject it

Severity: package contract/schema alignment blocker.

Evidence:

- `packages/rust/biors-core/src/package/types.rs` includes `RuntimeBackend::ExternalProcess` serialized as `external-process`.
- `packages/rust/biors-core/src/package/runtime.rs` treats `external-process` + `local-cpu` as a compatible runtime/model pair and exposes `execution_provider: "external-process"`.
- `schemas/package-manifest.v0.json` and `schemas/package-manifest.v1.json` only allow runtime backends `onnx-webgpu` and `candle`.
- `docs/backend-architecture.md` describes `ExternalProcess` as represented in the manifest runtime backend enum.

Why this matters:

A package manifest can be accepted by Rust deserialization and runtime-bridge planning while failing the published JSON Schema. Conversely, schema-first consumers cannot produce an `external-process` manifest that the Rust planner appears to support. This is exactly the kind of contract drift that creates ecosystem incompatibility before 1.0.

Required fix:

- Decide whether `external-process` is public manifest contract or internal/experimental only.
- If public, update v0/v1 schemas, docs, examples, and schema-contract tests.
- If internal, remove it from public package manifest types or reject it in `validate_package_manifest`.

### A-052: Checksum format validation accepts uppercase hex while schemas require lowercase

Severity: schema/implementation contract cleanup.

Evidence:

- `schemas/package-manifest.v0.json` and `schemas/package-manifest.v1.json` use checksum pattern `^sha256:[0-9a-f]{64}$`, which allows only lowercase hex.
- `packages/rust/biors-core/src/hash.rs` implements `is_sha256_checksum` with `byte.is_ascii_hexdigit()`, which accepts uppercase `A-F`.
- Artifact validation in `packages/rust/biors-core/src/package/artifacts.rs` relies on that Rust helper.

Why this matters:

The same manifest checksum can pass CLI/Rust validation and fail JSON Schema validation. For package authors, this is another schema contract mismatch. For reproducibility, checksum canonicalization should be exact and documented.

Required fix:

- Choose one checksum normalization policy.
- Prefer strict lowercase `sha256:<64 lowercase hex>` to match current schemas, and reject uppercase in Rust validation.
- Add tests for uppercase checksum rejection and schema/Rust parity.

### A-053: Package validation/bridge failures collapse structured reports into debug strings

Severity: CLI UX/API contract hardening.

Evidence:

- `packages/rust/biors/src/cli/package.rs` prints `PackageValidationReport` only when validation succeeds.
- When validation fails, `run_package_validate` returns `CliError::Validation` with `message: format!("{:?}", report.issues)`.
- `run_package_bridge` similarly formats a debug vector of validation issues plus bridge blocking issues.
- `run_package_verify` formats fixture verification issues with `format!("{:?}", ...)`.
- The JSON error envelope contains only `code`, `message`, and `location`; it does not include the structured issue list/report.

Why this matters:

Package authors need actionable, stable, machine-readable diagnostics to fix manifests, checksums, layout, and fixture observations. Debug-formatted Rust structs are not a public JSON contract and are brittle for notebooks, CI, or editor integrations.

Required fix:

- Add structured diagnostic payloads for package validation, bridge, and verify failures.
- Either return the full report with `ok:false` and non-zero exit code, or extend the error schema with `details`.
- Add schema tests for invalid package outputs, not only successful reports.

### A-054: Package verification does not validate duplicate or unexpected observations

Severity: package verification correctness hardening.

Evidence:

- `packages/rust/biors-core/src/verification/fixtures.rs` matches each fixture to an observation with `.iter().find(|candidate| candidate.name == fixture.name)`.
- If the observations file contains duplicate names, the first match silently wins.
- If the observations file contains names that do not exist in the manifest, they are ignored.
- `FixtureObservation` has only `name` and `path`, and there is no pre-validation pass for duplicate names, missing names, or extra observations.

Why this matters:

Fixture verification is a release-quality gate for packaged model outputs. Duplicate or stale observation entries can make CI appear green while hiding a wrong file mapping or unused generated output.

Required fix:

- Validate observations before comparing fixtures.
- Reject duplicate observation names.
- Warn or fail on unexpected observation names, depending on the desired strictness.
- Add tests for duplicates, extras, and mixed missing/extra observation sets.

### A-055: Candle backend accepts non-binary attention masks from raw runtime payloads

Severity: runtime input validation hardening.

Evidence:

- `schemas/model-input-output.v0.json` constrains `attention_mask` items to enum `[0, 1]`.
- `packages/rust/biors-backend-candle/src/lib.rs` checks only that `input_ids.len() == attention_mask.len()`.
- During scoring, it filters with `if *mask == 0 { None } else { Some(*token_id as u32) }`, so any value other than `0` is treated as unmasked.
- The `ModelInput` builder emits only `0` and `1`, but runtime payloads can be supplied directly as JSON to the backend contract.

Why this matters:

Runtime adapters are an integration boundary. A malformed model-input payload with mask values like `2` or `255` should be rejected rather than silently changing pooling behavior. Schema and runtime validation must agree.

Required fix:

- Validate `attention_mask` values in Candle before inference.
- Consider a reusable `validate_model_input_payload` helper in `biors-core` so CLI, Python/WASM, Candle, and external-process paths share the same checks.
- Add tests for non-binary masks, length mismatch, empty unmasked tokens, and out-of-range token IDs.

### A-056: WASM workflow silently defaults invalid `padTokenId` values to zero

Severity: WASM API correctness hardening.

Evidence:

- `packages/rust/biors-wasm/src/workflow.rs` reads `padTokenId` with `get_u8_opt(&config, "padTokenId").unwrap_or(0)`.
- `get_u8_opt` returns `None` when the field is missing, non-numeric, negative, or outside `0..=255`.
- All of those cases are then treated as if the caller intentionally omitted `padTokenId`, resulting in `0`.
- Fractional numeric handling is separately covered by A-008, but this path is worse because invalid optional values are silently accepted.

Why this matters:

JavaScript users may pass values from forms, notebooks, or JSON configs. A typo like `"21"` or `999` should fail loudly, not change padding semantics. Silent defaults make model-input outputs hard to debug and undermine reproducibility.

Required fix:

- Split optional-field handling into "missing" versus "present but invalid".
- Reject non-integer, non-number, negative, and out-of-range `padTokenId` values.
- Add WASM tests for missing, string, fractional, negative, and `>255` values.

### A-057: Python public classes are not constructible despite being function input types

Severity: Python binding usability/API hardening.

Evidence:

- `packages/rust/biors-python/src/lib.rs` exposes `ProteinSequence`, `TokenizedProtein`, `ModelInput`, `ModelInputRecord`, and workflow output classes as `#[pyclass]`.
- `prepare_workflow` takes `records: Vec<PyProteinSequence>`.
- `build_model_inputs_checked` takes `tokenized: Vec<PyTokenizedProtein>`.
- None of these `#[pyclass]` types define a `#[new]` constructor.
- Verified against the built wheel:
  - `biors.ProteinSequence("seq1", "ACDE")` raises `TypeError: No constructor defined for ProteinSequence`.
  - `biors.TokenizedProtein("seq1", [0, 1], 2)` raises `TypeError: No constructor defined for TokenizedProtein`.

Why this matters:

Python researchers often want to build inputs from pandas rows, notebooks, APIs, or existing in-memory data rather than always round-tripping through FASTA text. Exposing classes as input types without constructors creates an awkward API and makes the current signatures look more flexible than they are.

Required fix:

- Add safe constructors for Python input classes, or change public functions to accept dictionaries/typed Python sequences and validate them explicitly.
- Add tests for direct Python construction and in-memory workflow/model-input preparation.
- If FASTA-only input is intentional, narrow the Python API and docs so it does not imply direct record construction is supported.

### A-058: Python tokenization output drops validity, alphabet, warning, and error diagnostics

Severity: Python binding correctness/UX hardening.

Status: Complete in follow-up (2026-05-31 KST). Python `TokenizedProtein` now exposes the core `alphabet`, `valid`, `warnings`, and `errors` fields, with `ResidueIssue` objects carrying `residue` and `position`; `packages/rust/biors-python/README.md`, `docs/python-api.md`, and Python API tests document and verify the diagnostics.

Evidence:

- Core `TokenizedProtein` includes `id`, `length`, `alphabet`, `valid`, `tokens`, `warnings`, and `errors`.
- WASM `TokenizeOutput.records` preserves the full `TokenizedProtein` records.
- Python `PyTokenizedProtein` exposes only `id`, `tokens`, and `length`.
- `tokenize_fasta_records` and `tokenize_protein` map core tokenization output into that reduced Python class.
- A separate finding, A-004, covers how this also lets `build_model_inputs_checked` reconstruct invalid tokenized records as `valid: true`.

Why this matters:

Python is a primary researcher workflow. If tokenization encounters ambiguous or invalid residues, Python users should see the same diagnostics as Rust/WASM users without needing to run a separate validation call and manually join results by record ID.

Required fix:

- Add `alphabet`, `valid`, `warnings`, and `errors` to Python `TokenizedProtein`.
- Preserve those fields when passing tokenized objects into `build_model_inputs_checked`.
- Add Python tests for ambiguous and invalid residue tokenization.

### A-059: Python `tokenize_protein` hardcodes the record ID to `user`

Severity: Python binding API polish.

Evidence:

- `packages/rust/biors-python/src/lib.rs` implements `tokenize_protein(sequence: &str)`.
- It constructs `ProteinSequence { id: "user".to_string(), sequence: ... }`.
- The resulting `PyTokenizedProtein.id` is therefore always `"user"` for this API.

Why this matters:

Notebook and pipeline users need stable sequence identifiers to join tokenized outputs back to source data. A hardcoded ID makes single-sequence tokenization convenient only for demos, not for real in-memory researcher workflows.

Required fix:

- Add an optional `id` parameter with a safe default, or require `id` explicitly.
- Document the default if it remains.
- Add a Python test that tokenized output preserves a caller-provided ID.

### A-060: MCP tools classify user input parse failures as internal errors

Severity: MCP API contract hardening.

Evidence:

- `packages/rust/biors-mcp-server/src/server.rs` maps FASTA parse/validation errors to `McpError::internal_error` in `tokenize`, `validate`, and `workflow`.
- Invalid FASTA text, missing headers, empty input, and malformed sequence records are user-supplied parameter errors, not server failures.
- The invalid profile/kind paths already use `McpError::invalid_params`, so error classification is inconsistent within the same MCP server.

Why this matters:

MCP clients and AI tool callers need to distinguish "fix your input" from "tool/server failed". Misclassifying biological input problems as internal errors makes automated recovery and user messaging worse.

Required fix:

- Map `BioRsError`/FASTA parse failures in MCP tools to `invalid_params` with structured details.
- Keep serialization and unexpected server failures as `internal_error`.
- Add MCP integration tests for invalid FASTA, empty input, invalid kind/profile, and invalid workflow config.

### A-061: Checked-in example pipeline lockfile is stale and contains non-replayable arguments

Severity: example/reproducibility cleanup.

Evidence:

- `examples/pipeline/pipeline.lock` reports `generated_by.biors_version` and `generated_by.biors_core_version` as `0.34.0`.
- Current workspace packages are `0.47.4`.
- The lockfile execution arguments include `--padding fixed_length`, which the current CLI rejects; see A-047.
- The lockfile includes `python_baseline.status: "strategy_recorded"`, not evidence that a current Python parity check was executed.
- Behavior check in `/tmp`: regenerating a lock with `biors pipeline --config examples/protein-package/pipelines/protein.toml --package examples/protein-package/manifest.json --write-lock <tmp>/pipeline.lock` produced `0.47.4` tool versions and a different `hashes.output_data_sha256` than the checked-in `examples/pipeline/pipeline.lock`.

Why this matters:

Example lockfiles are treated as reproducibility artifacts. A stale lockfile with an old tool version and non-replayable command arguments teaches users the wrong contract and can mask drift in hashes, paths, and package metadata.

Required fix:

- Regenerate `examples/pipeline/pipeline.lock` with the current release candidate after A-047 is fixed.
- Replace `strategy_recorded` with an actually verified baseline status or remove the baseline section until the check exists.
- Add a release gate that regenerates/checks example lockfiles and fails if they are stale.

### A-062: Example package citation/version text still points at `0.31.0`

Severity: documentation/example cleanup.

Evidence:

- `examples/protein-package/manifest.json` has `metadata.citation.preferred_citation: "bio-rs protein package fixture, version 0.31.0"`.
- `examples/protein-package/docs/CITATION.cff` has `version: "0.31.0"`.
- `docs/package-format.md` says "The manifest contract remains in `biors-core` for v0.31.0" in a current package-format document.
- Current release candidate/package version is `0.47.4`.

Why this matters:

Fixture package metadata is part of the public examples users copy from. Stale version strings make it unclear whether the fixture, package schema, or tool version is current.

Required fix:

- Either update fixture metadata to the current release candidate or clearly label it as an intentionally historical fixture version independent of the bio-rs release.
- Refresh `docs/package-format.md` wording so the crate split note is not anchored to an old release unless it is explicitly historical.
- Add a version-string audit to release docs for examples and metadata files.

### A-063: Release workflow checker is a brittle string-order scan, not a YAML/semantics check

Severity: release gate hardening.

Evidence:

- `scripts/check-release-workflow.py` reads `.github/workflows/release.yml` as plain text and checks for exact marker strings.
- It verifies approximate ordering of strings, but does not parse YAML.
- It does not validate job `needs`, permissions, runner matrices, action pinning, install steps, artifact paths, or publish conditions semantically.
- Existing findings such as A-021, A-027, A-031, A-035, and A-036 are not caught by this checker.

Why this matters:

The script gives a false sense of release safety. A workflow can satisfy the marker strings while still missing a toolchain install, skipping a binding test, changing permissions, or publishing artifacts without checksums.

Required fix:

- Parse the workflow YAML and validate job-level semantics explicitly.
- Add checks for required `needs`, permissions, toolchain setup, package artifact tests, checksums/attestations, and immutable action pinning.
- Keep string marker checks only as a secondary guard for expected commands.

### A-064: Benchmarks are documented and syntax-checked but not run in CI/release workflows

Severity: performance/regression gate gap.

Evidence:

- There is no `.github/workflows/benchmarks.yml`.
- `scripts/check.sh` only compiles benchmark scripts with `python3 -m py_compile` and validates the committed benchmark JSON structure.
- Criterion benches exist for `biors-core` and `biors-backend-candle`, but CI/release workflows do not run or compare them.
- The committed FASTA benchmark artifact is historical and does not cover newer feature surfaces; see A-003 and A-014.

Why this matters:

The project makes performance claims and has researcher-facing throughput expectations. Without an automated benchmark or regression comparison path, performance changes in parsing, tokenization, workflow generation, bindings, and Candle can slip into release candidates unnoticed.

Required fix:

- Add a benchmark workflow or scheduled/manual benchmark job that runs the supported benchmark suite in a controlled environment.
- Add a lightweight PR gate for benchmark harness compilation plus selected smoke workloads.
- Add release criteria for refreshing committed benchmark artifacts when feature surfaces change.

### A-065: Benchmark default dataset uses UniProt `current_release` instead of a pinned release

Severity: benchmark reproducibility gap.

Evidence:

- `scripts/benchmark_fasta_vs_biopython.py` sets `UNIPROT_HUMAN_PROTEOME_GZ_URL` under `https://ftp.uniprot.org/.../current_release/...`.
- When `--input` is omitted, the script downloads that URL and records the downloaded gzip SHA-256.
- The URL can point to different data over time even when the command line is unchanged.

Why this matters:

Benchmark evidence must be reproducible. Recording the downloaded hash helps after the fact, but a future rerun may silently benchmark a different proteome file and change shape, residue composition, runtime, and memory behavior.

Required fix:

- Pin the default benchmark dataset to a dated UniProt release or require `--input` for release-grade benchmark recording.
- Record source release/date and raw FASTA/gzip checksums in the artifact.
- Add a benchmark-artifact check that rejects `current_release` as the only source reference for published benchmark claims.

### A-066: Benchmark artifact check does not detect stale package version or commit

Severity: benchmark release gate gap.

Evidence:

- `benchmarks/fasta_vs_biopython.json` records `environment.biors_core: "0.20.0"` and git commit `a4aee2c2...`.
- Current workspace packages are `0.47.4`.
- `scripts/check-benchmark-artifact.py` validates schema shape, dataset count, workload presence, and summary fields.
- It does not compare `environment.biors_core`, `environment.git_commit`, or generation date against the current release candidate.

Why this matters:

The existing release gate can pass with a structurally valid but stale benchmark artifact. That is why old numeric performance claims can remain in the repo without a failing check.

Required fix:

- Make the benchmark check compare the artifact package version to `cargo metadata`.
- Require an explicit "historical benchmark" marker if old artifacts are intentionally retained.
- For release candidates, require either a current benchmark artifact or no numeric claims tied to the current release.

### A-067: Python API docs use a non-existent runtime bridge field

Severity: documentation/API accuracy fix.

Evidence:

- `docs/python-api.md` shows:
  - `bridge = json.loads(biors.plan_runtime_bridge(manifest_json))`
  - `print(bridge["compatible"])`
- `packages/rust/biors-core/src/package/reports.rs` defines `RuntimeBridgeReport` with `ready`, not `compatible`.
- `schemas/package-bridge-output.v0.json` also requires `ready`, not `compatible`.

Why this matters:

The Python docs give users code that will raise `KeyError` on valid output. This is a direct copy/paste break in a published API reference.

Required fix:

- Change the example to `bridge["ready"]`.
- Add a docs smoke test for Python README/API snippets or at least JSON key references in binding docs.
- Consider returning Python dicts for schema-rich functions instead of JSON strings if the intended workflow is `json.loads(...)`.

### A-068: Reliability and CLI docs overstate `--json` error coverage

Severity: documentation/contract alignment.

Evidence:

- `docs/reliability.md` says "`--json` errors write JSON to stdout and keep stderr empty."
- `docs/cli-contract.md` repeats that passing `--json` writes errors to stdout and keeps stderr empty.
- A-049 verified that clap parse errors still print human-readable clap text and exit before the JSON error path.

Why this matters:

Automation users will rely on the docs when wrapping `biors` in notebooks, pipelines, and service adapters. The current docs describe a stronger contract than the binary actually provides.

Required fix:

- Prefer fixing implementation per A-049.
- Until then, document that `--json` covers post-parse runtime errors only and does not cover clap argument parsing failures.
- Add tests that assert docs examples and CLI behavior remain aligned.

### A-069: Service interface advertises request schemas that are not checked in

Severity: service contract release blocker.

Evidence:

- `packages/rust/biors-core/src/service.rs` lists request schema names such as:
  - `service-sequence-validate-request.v0.json`
  - `service-sequence-inspect-request.v0.json`
  - `service-sequence-tokenize-request.v0.json`
  - `service-model-input-request.v0.json`
  - `service-package-request.v0.json`
  - `service-package-compatibility-request.v0.json`
- `schemas/` only contains `service-interface-output.v0.json`; none of the referenced request schemas exist.
- `docs/service-interface.md` says the route and schema metadata are stable operation contracts.

Why this matters:

The service interface is presented as a contract that hosts can adapt into OpenAPI or service wrappers. Missing request schemas make that contract incomplete and non-actionable for implementers.

Required fix:

- Add the referenced request schema files, or change the service interface to reference only existing schemas.
- Add a schema-contract test that every schema name emitted by `biors service contract` exists under `schemas/`.
- Add docs showing request/response examples for each service route.

### A-070: Package skeleton creation can overwrite existing asset/doc files without `--force`

Severity: destructive CLI behavior.

Evidence:

- `packages/rust/biors/src/cli/package_skeleton.rs` only checks whether `output_dir/manifest.json` exists before deciding that `--force` is required.
- `create_package_skeleton` then creates subdirectories and calls `copy_asset`, `write_tokenizer_config`, `write_pipeline_config`, and `write_docs`.
- `copy_asset` uses `std::fs::copy(source, target)` and the writer helpers use `std::fs::write(...)`; both overwrite existing target files.
- This means an existing `models/<name>`, `fixtures/<name>`, `tokenizers/<profile>.json`, `pipelines/protein.toml`, or docs file can be overwritten when no manifest is present.
- Behavior check in `/tmp`: an existing `out/models/model.onnx` was overwritten by `biors package init ... --model src/model.onnx` without `--force` when no manifest existed.

Why this matters:

Release-prep tooling must not destroy user package work by default. A researcher iterating on a package directory could lose model cards, fixture outputs, or tokenizer configs even though they did not pass `--force`.

Required fix:

- Treat any existing target file as requiring `--force`, not just `manifest.json`.
- Precompute the full write set before writing anything, and fail atomically if collisions exist.
- Add CLI tests covering collision detection for models, fixtures, tokenizers, pipelines, and docs.

### A-071: Python project package conversion chooses the first matching model nondeterministically

Severity: reproducibility and UX gap.

Evidence:

- `packages/rust/biors/src/cli/package_init.rs` implements `run_package_convert_project`.
- If `--model` is omitted, `find_first_file(&args.project_dir, &["onnx"])` recursively returns the first ONNX file it encounters.
- `std::fs::read_dir` iteration order is filesystem-dependent and the search does not skip common directories such as `.venv`, `__pycache__`, `target`, `.git`, build outputs, or downloaded model caches.
- If multiple ONNX files exist, the selected model is not stable or explained.

Why this matters:

Model package conversion should be deterministic and auditable. Picking an arbitrary ONNX file can package the wrong model, especially in real Python projects with exported checkpoints, test fixtures, or cached artifacts.

Required fix:

- Require `--model` when multiple ONNX candidates are found.
- Sort traversal results and return a structured candidate list in the error.
- Skip known generated/cache directories by default, with an explicit override if needed.
- Add tests with multiple ONNX files to lock the selection contract.

### A-072: Hugging Face tokenizer conversion is heuristic but emits package-ready assets

Severity: model interoperability correctness gap.

Evidence:

- `packages/rust/biors/src/cli/tokenizer_convert.rs` parses `tokenizer_config.json` as generic JSON.
- `hf_config_to_biors_config` checks only a handful of fields such as `tokenizer_class`, `do_lower_case`, `model_max_length`, and presence of special-token metadata.
- It does not read the tokenizer vocabulary, special token IDs, residue token IDs, normalizer/pre-tokenizer rules, or model-specific token ordering.
- The output still includes `package_tokenizer_asset`, `package_preprocessing_step`, `contract_version`, and `config_sha256`, which makes the result look ready to put into a package.

Why this matters:

For bio researchers using ESM, ProtBERT, or lab-specific Hugging Face exports, token ID mismatches can silently produce invalid model inputs. A heuristic conversion is acceptable as an assistive tool only if it is explicitly marked incomplete and validated against fixtures.

Required fix:

- Rename or document this as an inference/preview command unless full vocabulary conversion is implemented.
- Require fixture parity checks before suggesting package-ready assets.
- If full conversion is intended, parse vocab/tokenizer files and preserve exact special token IDs and residue mapping.
- Add benchmark and correctness fixtures for common protein tokenizer families.

### A-073: `doctor` checks only Rust/demo basics and misses release-critical bindings/tooling

Severity: release readiness diagnostics gap.

Evidence:

- `packages/rust/biors/src/cli/doctor.rs` checks `rustc`, `cargo`, `wasm32-unknown-unknown`, `examples/launch-demo.fasta`, and `examples/protein-package/manifest.json`.
- It does not check `wasm-pack`, Node/npm, `maturin`, Python version, package license files, schema availability, benchmark tooling, or security/dependency audit tooling.
- The release workflow now includes Python, WASM, MCP, package validation, and dry-run publishing surfaces, so these omissions are material.

Why this matters:

`biors doctor` is the obvious command researchers and maintainers will run before demos or releases. It currently reports a healthy environment while major supported packaging/binding surfaces may be impossible to build or validate locally.

Required fix:

- Split `doctor` into capability checks: core CLI, WASM, Python, package validation, release tooling, and benchmark tooling.
- Keep optional capabilities as warnings, but show explicit missing-tool messages and install hints.
- Add tests that assert `doctor` output schema includes the supported release surfaces.

### A-074: Package artifact path checks do not prevent symlink escape from package roots

Severity: package security and reproducibility gap.

Evidence:

- `packages/rust/biors-core/src/package/paths.rs` validates package artifact paths syntactically: non-empty, relative, and no `..`, root, or prefix components.
- `read_package_file` then calls `fs::read(base_dir.join(relative_path))`.
- There is no canonicalization check that the resolved file remains under the canonical package root after following symlinks.
- A package can place `models/model.onnx` or another artifact path as a symlink to a file outside the package root; validation and verification will read the external target.

Why this matters:

Portable biological model packages should be self-contained and inspectable. Symlink escapes can make validation depend on local machine state, leak local files into hashes/reports, or make a package pass on one machine and fail elsewhere.

Required fix:

- Canonicalize the package root and resolved artifact path before reading.
- Reject symlinks or require canonical targets to remain within the package root.
- Add tests for symlink escape, nested symlink escape, and normal in-package symlinks if those are intentionally allowed.

### A-075: Sequence validation exposes two code namespaces for the same issue kinds

Severity: API contract cleanup.

Evidence:

- `docs/error-codes.md` and `docs/cli-contract.md` list serialized sequence issue codes as `ambiguous_symbol` and `invalid_symbol`.
- CLI/JSON payloads serialize `SequenceValidationIssueCode` with `#[serde(rename_all = "snake_case")]`, so payloads use the unprefixed values.
- `packages/rust/biors-core/src/sequence/types.rs::SequenceValidationIssueCode::as_str` returns `sequence.ambiguous_symbol` and `sequence.invalid_symbol` through the `Diagnostic` implementation.
- The same conceptual issue therefore has one code in payload JSON and another code through the Rust diagnostic trait.

Why this matters:

Stable diagnostics should not require users to know which layer produced the code. This matters for wrappers that combine CLI JSON payloads, WASM output, MCP output, and Rust API diagnostics.

Required fix:

- Choose one public code spelling for sequence issues, or explicitly document the payload-code versus diagnostic-code distinction.
- Prefer serializing the same stable code string everywhere if this is still pre-1.0.
- Add tests that compare payload issue codes, WASM TypeScript declarations, schemas, and `Diagnostic::code()` values.

### A-076: Candle backend docs and error registry do not match actual Candle error codes

Severity: documentation/API accuracy fix.

Evidence:

- `docs/candle-backend.md` says payload parsing, tensor loading, shape validation, token ID checks, and Candle execution are mapped into `runtime.execution_failed`.
- `packages/rust/biors-backend-candle/src/lib.rs` emits specific codes such as `candle.load_failed`, `candle.invalid_model_input`, `candle.empty_attention_mask`, `candle.token_id_out_of_range`, `candle.tensor_failed`, `candle.inference_failed`, `candle.output_failed`, `candle.missing_tensor`, `candle.invalid_shape`, and `candle.invalid_dtype`.
- `docs/error-codes.md` lists runtime codes but does not register the `candle.*` codes.

Why this matters:

Researchers integrating a local Candle backend need stable, searchable failure codes for model-input and tensor problems. Current docs describe a single generic runtime error while the crate actually exposes a richer API surface.

Required fix:

- Update Candle docs to describe the actual `candle.*` error taxonomy.
- Add the Candle codes to the error registry or explicitly mark them as crate-local backend codes with stability guarantees.
- Add a test or script that fails when emitted error codes are absent from `docs/error-codes.md`.

### A-077: `package verify` can pass an invalid manifest because it does not run package validation first

Severity: package release gate gap.

Evidence:

- `packages/rust/biors/src/cli/package.rs::run_package_verify` reads the manifest and observations, then calls `verify_package_outputs_with_observation_base`.
- It does not call `validate_package_manifest_artifacts` before comparing fixture outputs.
- `verify_package_outputs_with_observation_base` checks fixture input/expected/observed files, hashes, and content, but it does not validate v1 metadata, declared layout, runtime compatibility, model-card/license/citation files beyond fixture-related paths, or unknown-field/schema alignment.
- The demo and quickstart present package verification as a portable package confidence step.
- Behavior check in `/tmp`: removing `metadata` from `examples/protein-package/manifest.json` still let `biors package verify manifest.invalid.json observations.json` pass with `passed=1` and `failed=0`.

Why this matters:

A manifest can have matching fixture outputs but still be invalid or incomplete as a researcher-facing package. Release gates and user docs should not let `package verify` green-light packages that `package validate` would reject.

Required fix:

- Either make `package verify` run manifest artifact validation first, or rename/docs-scope it as fixture-output verification only.
- Include validation failures in the verify report instead of dropping structured detail into a debug string.
- Add tests for manifests that have passing fixtures but missing v1 metadata or invalid layout.

### A-078: Python workflow output hides provenance and readiness diagnostics

Severity: Python API usability gap.

Evidence:

- `packages/rust/biors-python/src/lib.rs::prepare_workflow` calls `workflow::prepare_protein_model_input_workflow`.
- Core `SequenceWorkflowOutput` includes validation, tokenization, optional model input, readiness issues, and provenance.
- Python `PySequenceWorkflowOutput` exposes only `model_ready` and `records`.
- When a sequence is not model-ready, Python callers get `model_ready = False` and an empty `records` list without the underlying `readiness_issues`, warnings, errors, or provenance hashes.

Why this matters:

Researchers using Python notebooks need to understand why a sequence failed before sending data into a model. Hiding diagnostics forces them to rerun separate APIs and manually reconcile outputs.

Required fix:

- Expose the full workflow report to Python, either as typed classes or JSON/dict payloads matching `schemas/sequence-workflow-output.v0.json`.
- Include readiness issue codes, warning/error counts, validation/tokenization reports, and provenance hashes.
- Add Python tests for invalid/ambiguous sequences and provenance fields, not only the happy path.

### A-079: Python exceptions drop stable bio-rs error codes

Severity: binding contract gap.

Evidence:

- Python binding functions map most errors to `PyValueError::new_err(e.to_string())`.
- FASTA parse errors, model-input errors, padding errors, package JSON errors, and runtime bridge parse errors are surfaced as plain exception text.
- The Rust CLI has stable error codes and structured locations, but Python callers cannot catch or inspect those codes.

Why this matters:

Production notebooks, pipeline wrappers, and service adapters need machine-readable errors. Plain text messages are harder to test and localize, and they do not align with the CLI/MCP contract.

Required fix:

- Add a Python exception type carrying `code`, `message`, and optional `location`.
- Map core `Diagnostic` errors and model-input/package errors into that type.
- Add tests asserting error codes for empty FASTA, invalid FASTA, invalid padding, invalid policy, and invalid package JSON.

### A-080: Direct Rust sequence APIs expose an unenforced normalization invariant

Severity: Rust API correctness gap.

Evidence:

- `ProteinSequence.sequence` is documented as normalized, whitespace-stripped, and ASCII-uppercased.
- The type is a public struct with public fields and no constructor enforcing that invariant.
- `validate_protein_sequence_owned` checks ASCII bytes through `is_protein_20_residue_byte` without uppercasing them, so a direct `ProteinSequence { sequence: b"acde".to_vec(), ... }` is treated differently from a FASTA-parsed record.
- `tokenize_protein_with_config` does accept lowercase bytes because the tokenizer lookup table contains lowercase residues.
- The same caller-supplied lowercase `ProteinSequence` can therefore tokenize as valid while direct validation flags the same symbols as invalid.

Why this matters:

External Rust users are encouraged to migrate from JSON boundaries to native `ProteinSequence` values. For researchers, a public struct that silently relies on caller-side normalization can create inconsistent validation/tokenization outcomes.

Required fix:

- Add a constructor such as `ProteinSequence::new_normalized(id, sequence)` or `ProteinSequence::from_raw(id, sequence)` and document which one normalizes.
- Either make fields private before 1.0 or make direct validation/tokenization normalize consistently.
- Add tests for lowercase direct API input across validation, tokenization, and workflow.

### A-081: Runtime backend execution does not enforce the requested output format

Severity: runtime contract gap.

Evidence:

- `BackendCapabilities::compatibility_report` checks whether `context.requested_output_format`, when present, is included in `supported_outputs`.
- `ExternalProcessBackend::execute` parses the child `ExecutionResult` and checks only whether `result.output_format` is generally supported.
- It does not compare `result.output_format` to `context.requested_output_format`.
- A backend that supports both `A` and `B` can receive a request for `A`, return `B`, and still pass `execute_checked`.

Why this matters:

Runtime adapters are intended to be stable contracts for model execution. If a researcher asks for a specific tensor/output representation, receiving another supported format should be a structured error rather than a successful response.

Required fix:

- In `ExternalProcessBackend::execute`, when `context.requested_output_format` is `Some`, require exact equality with `result.output_format`.
- Add a runtime error code such as `runtime.output_format_mismatch`.
- Add tests with a fixture external process returning the wrong supported output format.

### A-082: Pipeline dry-run reports `ready: true` even when the input path is missing

Severity: CLI readiness semantics gap.

Evidence:

- `PipelineOutput::dry_run` hardcodes `ready: true` while all steps are only `"planned"`.
- `run_config_pipeline` returns the dry-run output immediately after config parsing, before opening or statting the configured input path.
- Behavior check in `/tmp`: a pipeline config whose `[input].path` points to `missing.fasta` returned `ok: true`, `data.ready: true`, and planned steps.

Why this matters:

`ready` is used elsewhere as an execution/readiness signal. In dry-run mode, `ready: true` can be misread as “this pipeline is executable”, even though the referenced biological input may not exist.

Required fix:

- Change dry-run semantics to `ready: false` or add a distinct `status: "planned"` / `executable: unknown` field.
- Optionally stat the input path in dry-run and report input existence separately.
- Add a test for dry-run with a missing input path.

### A-083: `package init` hardcodes ONNX/WebGPU even for non-ONNX model files

Severity: package generation correctness gap.

Evidence:

- `PackageInitArgs` accepts any `--model <PathBuf>` with no extension or format validation.
- `create_package_skeleton` always writes `model.format = onnx`, `runtime.backend = onnx-webgpu`, `runtime.target = browser-wasm-webgpu`, and `runtime.version = onnx-webgpu.v0`.
- Behavior check in `/tmp`: running `biors package init --model model.safetensors ...` produced a manifest with `"format": "onnx"` and ONNX/WebGPU runtime metadata.

Why this matters:

The repository now has a Candle backend and safetensors model format in the package schema. A package skeleton that labels a safetensors artifact as ONNX creates a non-portable, misleading package before validation ever reaches a real backend.

Required fix:

- Infer model format from extension when safe, or require `--model-format onnx|safetensors`.
- Pair default runtime with model format explicitly, or require `--runtime-backend` and `--runtime-target` for non-default formats.
- Reject unsupported format/runtime pairs during `package init`.
- Add package-init tests for `.onnx`, `.safetensors`, and unknown extensions.

### A-084: Package skeleton writes license and citation metadata as misleading files

Severity: package metadata quality gap.

Evidence:

- `write_docs` writes the license expression directly into `docs/LICENSE.txt`.
- The same function writes the `--citation` free-text string directly into `docs/CITATION.cff`.
- Behavior check in `/tmp`: `--license MIT --citation "Smith et al 2026"` produced `LICENSE.txt` containing only `MIT` and `CITATION.cff` containing only `Smith et al 2026`.
- The generated manifest then records these files as package metadata artifacts with checksums.

Why this matters:

Bio model packages need usable redistribution and citation metadata. A file named `LICENSE.txt` should not be only an SPDX expression, and a file named `CITATION.cff` should be valid Citation File Format YAML, not arbitrary free text.

Required fix:

- Keep SPDX expressions in manifest metadata, but either fetch/include actual license text only when supplied or name the generated file `LICENSE-SPDX.txt`.
- Generate valid CFF only from structured fields, or write free-form citations to `CITATION.txt`.
- Add validation for generated package metadata files and tests that parse generated CFF when the extension is `.cff`.

### A-085: Package validation does not reject duplicate fixture names

Severity: package verification ambiguity.

Evidence:

- `validate_fixture_list` checks that each fixture has non-empty `name`, `input`, and `expected_output`, but it does not check uniqueness.
- `verify_package_outputs_with_observation_base` matches observations with `.iter().find(|candidate| candidate.name == fixture.name)`.
- Behavior check in `/tmp`: duplicating the single fixture in `examples/protein-package/manifest.json` made `biors package validate` return `ok: true`, `valid: true`, and no fixture-related issue.

Why this matters:

Fixture names are the join key between manifest fixtures and observations. Duplicate fixture names make verification ambiguous and can hide missing or mismatched outputs.

Required fix:

- Reject duplicate fixture names during package validation.
- Reject duplicate observation names during package verification.
- Add tests for duplicate fixture names, duplicate observation names, and a manifest with two different expected outputs sharing one fixture name.

### A-086: WASM npm package omits required generated runtime glue

Severity: release blocker for npm package.

Evidence:

- `packages/rust/biors-wasm/pkg/biors_wasm.js` imports `./biors_wasm_bg.js` and re-exports functions from that file.
- `packages/rust/biors-wasm/package.json` lists `files` as `biors_wasm_bg.wasm`, `biors_wasm.js`, `biors_wasm.d.ts`, `index.d.ts`, and `README.md`.
- `scripts/build-wasm-npm-package.sh` copies that `files` list into the generated `pkg/package.json`.
- `npm pack packages/rust/biors-wasm/pkg --dry-run` listed only 6 files and did not include `biors_wasm_bg.js`.
- The dry-run still exits successfully, so the current release script does not catch the broken package contents.

Why this matters:

The published npm package can be installed but fail at runtime because the main JS entry imports a file that is excluded from the tarball. This directly blocks JS/WASM users from using the package.

Required fix:

- Add `biors_wasm_bg.js` to the npm package `files` list.
- Consider including `biors_wasm_bg.wasm.d.ts` if TypeScript consumers need the generated WASM module declaration.
- After `npm pack`, inspect the tarball contents in CI and run an import smoke test against the packed `.tgz`, not only the source `pkg/` directory.
- Add this to `scripts/build-wasm-npm-package.sh` or the release workflow before `npm publish`.

### A-087: WASM docs and TypeScript declarations advertise a default `init` export that the generated package does not provide

Severity: release blocker for npm package.

Evidence:

- `packages/rust/biors-wasm/README.md` documents `import init, { ... } from "@bio-rs/biors-wasm";` followed by `await init();`.
- `docs/wasm-api.md` repeats the same initialization contract.
- `packages/rust/biors-wasm/index.d.ts` re-exports `default as init` from `./biors_wasm.js`.
- The generated `packages/rust/biors-wasm/pkg/biors_wasm.js` has named exports only and no default export.
- Behavior check in `/tmp`: `node --input-type=module -e "import init, { parseFasta } from './packages/rust/biors-wasm/pkg/biors_wasm.js'; ..."` failed with `SyntaxError: The requested module ... does not provide an export named 'default'`.
- A named-only import of `parseFasta` from the same file succeeds, so the failure is specifically the documented default import contract.

Why this matters:

The first npm usage example and public `.d.ts` can send JS/TypeScript users into a runtime import failure even if the missing glue file in A-086 is fixed. This is exactly the kind of package-level breakage researchers will hit immediately in notebooks, Vite apps, or Node-based analysis tooling.

Required fix:

- Choose one public initialization contract:
  - document the current bundler-target behavior with named exports and no explicit `init`, or
  - add a wrapper entrypoint that exports a real default `init` matching the docs and TypeScript declarations.
- Make `index.d.ts`, `package.json` entrypoints, README examples, and `docs/wasm-api.md` agree.
- Add a packed npm artifact smoke test that runs the documented import form against the built tarball.

### A-088: Package validation accepts empty tokenizer/vocab and pipeline contract identifiers

Severity: package contract correctness gap.

Evidence:

- `schemas/package-manifest.v1.json` defines `asset.name`, `asset.contract_version`, `pipelineStep.name`, `pipelineStep.implementation`, `pipelineStep.contract`, and `pipelineStep.contract_version` as plain strings without `minLength`.
- `schemas/package-manifest.v0.json` has the same loose `pipelineStep` string constraints.
- `validate_package_manifest` checks top-level name, model path, v1 layout/metadata, fixture fields, and optional shapes, but it does not validate tokenizer/vocab asset names or pipeline step name/implementation/contract strings.
- Behavior check in `/tmp`: setting `preprocessing[0].name`, `preprocessing[0].implementation`, `preprocessing[0].contract`, `tokenizer.name`, `tokenizer.contract_version`, `vocab.name`, and `vocab.contract_version` to empty strings still made `biors package validate` return `ok: true`, `valid: true`, and `issues: []`.

Why this matters:

Package manifests are intended to be portable contracts. Empty tokenizer names, vocab names, pipeline implementations, or contract identifiers make a package difficult to inspect, reproduce, or map to a runtime adapter, while still looking valid to users and automation.

Required fix:

- Add `minLength: 1` to all package-manifest schema string fields that are contract identifiers rather than optional free text.
- Extend `validate_package_manifest` to reject empty tokenizer/vocab asset `name` and `contract_version` when present.
- Extend `validate_pipeline_step_configs` or a new pipeline-step validator to reject empty `name`, `implementation`, `contract`, and `contract_version` when present.
- Add CLI tests that mutate `examples/protein-package/manifest.json` with empty contract identifiers and assert `valid: false`.

### A-089: Package manifest JSON schemas are looser than Rust validation for required non-empty fields

Severity: schema contract gap.

Evidence:

- `schemas/package-manifest.v1.json` and `schemas/package-manifest.v0.json` define `shape.shape` as an array without `minItems`.
- The same schemas define `fixture.name` as a string without `minLength`.
- Rust validation rejects empty fixture names through `push_required_issue` and rejects empty shape dimensions through `validate_shape`.
- Behavior check in `/tmp`: setting `fixtures[0].name = ""`, `expected_input.shape = []`, and `expected_output.shape = []` made `biors package validate` reject the manifest with required-field and invalid-shape errors.
- `packages/rust/biors/tests/schema_contract.rs` validates positive examples against schemas but does not include negative schema tests for these manifest constraints.

Why this matters:

Researchers and package authors may validate manifests with the published JSON schemas before running the CLI. If the schema accepts values that the CLI later rejects, the schema stops being a reliable preflight contract.

Required fix:

- Add `minItems: 1` to package-manifest `shape.shape` arrays in both v0 and v1 schemas.
- Add `minLength: 1` to `fixture.name` in both schemas.
- Add negative schema tests that prove empty fixture names and empty shapes are rejected by the schema, not only by Rust-side validation.

### A-090: Full CI/release check does not verify rendered benchmark Markdown drift

Severity: benchmark/release gate gap.

Evidence:

- `scripts/check-benchmark-docs.sh` correctly runs `scripts/check-benchmark-artifact.py`, renders `scripts/render_benchmark_report.py`, and diffs the result against `benchmarks/fasta_vs_biopython.md`.
- `scripts/check-fast.sh` calls `scripts/check-benchmark-docs.sh`.
- `scripts/check.sh`, which is the CI and release-readiness gate, prints `==> benchmark docs` but only runs `python3 scripts/check-benchmark-artifact.py`.
- Therefore `scripts/check.sh` can pass when the benchmark JSON structure is valid but the committed Markdown report is stale.

Why this matters:

The README and release docs point humans at the rendered benchmark report, not only the JSON artifact. If feature work changes benchmark coverage or numbers, the main release gate should fail when the human-readable report is out of date.

Required fix:

- Replace `python3 scripts/check-benchmark-artifact.py` in `scripts/check.sh` with `scripts/check-benchmark-docs.sh`.
- Keep `check-benchmark-artifact.py` as the lower-level JSON validator, but make the full gate cover both machine artifact and rendered Markdown.
- Add a small script-level regression test or comment explaining why `check-fast.sh` and `check.sh` must not drift on benchmark documentation checks.

### A-091: Pipeline output schema does not constrain the nested workflow payload

Severity: public JSON contract gap.

Evidence:

- `PipelineOutput::from_workflow` and `PipelineOutput::from_config_workflow` serialize `workflow: Some(SequenceWorkflowOutput)`.
- `schemas/pipeline-output.v0.json` requires a `workflow` property, but defines it only as `{ "type": ["object", "null"] }`.
- The schema does not reference or inline the `sequence-workflow-output.v0.json` contract for non-null workflow payloads.
- `packages/rust/biors/tests/schema_contract.rs` validates real pipeline output against `pipeline-output.v0.json`, but that validation would still pass if the nested workflow object lost required sequence-workflow fields.

Why this matters:

Pipeline output is one of the highest-level researcher-facing contracts. Users consuming pipeline JSON need the embedded workflow payload to remain as stable as direct `biors workflow` output, especially for provenance, validation, tokenization, and model-input readiness fields.

Required fix:

- Make `pipeline-output.v0.json` constrain non-null `workflow` to the same shape as `sequence-workflow-output.v0.json`.
- If cross-file `$ref` support is inconvenient in tests, inline the sequence workflow definition or add a dedicated schema-contract test that validates `pipeline["workflow"]` against `schemas/sequence-workflow-output.v0.json` whenever it is non-null.
- Add a negative test proving an incomplete nested workflow is rejected.

### A-092: Package bridge `ready` can overstate executable model readiness

Severity: researcher-readiness contract gap.

Evidence:

- `examples/protein-package/models/protein-seed.onnx` is a 39-byte ASCII placeholder, not an executable ONNX model.
- The example model card says the artifact is a placeholder and not suitable for scientific inference.
- `biors package validate examples/protein-package/manifest.json` returns `valid: true` because validation checks layout, paths, required fields, and checksums rather than model file format.
- `biors package bridge examples/protein-package/manifest.json` returns `ready: true` and a `backend_config` for `onnx-webgpu`, because `plan_runtime_bridge` only checks the declared `(model.format, runtime.backend, runtime.target)` pair and manifest-level validation.
- The bridge report has no field that distinguishes “declared contract pair is supported” from “artifact was parsed or execution-smoke-tested”.

Why this matters:

Researchers can read `ready: true` as “this package can run with the backend”. For placeholder, corrupted, or mislabeled model artifacts, that is too strong. The tool should make clear whether readiness is only manifest/compatibility planning or actual executable model readiness.

Required fix:

- Rename or split the readiness fields, for example `contract_ready`, `artifact_checked`, and `execution_ready`.
- Add an explicit note/issue when model artifact format validation is not performed.
- For formats with cheap checks, add lightweight sniffing without pulling heavy runtime dependencies. For ONNX, even a conservative “not format-validated” marker is safer than `ready: true` alone.
- Add tests using the current placeholder ONNX fixture to prove bridge output does not imply executable readiness.

### A-093: Python validation API drops per-record diagnostics needed to fix biological inputs

Severity: Python researcher-readiness gap.

Evidence:

- `PySequenceValidationReport` exposes only `records`, `valid_records`, `warning_count`, and `error_count`.
- Core `SequenceValidationReport` includes per-sequence validation details, including residue warnings/errors and positions.
- `validate_fasta_input` maps the core report into the reduced Python class and drops `sequences`.
- Behavior check with the locally built wheel: validating `>seq1\nAC*X\n` returned `records=1`, `valid_records=0`, `warning_count=1`, `error_count=1`, but the Python object had no `sequences` attribute and only the four aggregate fields.

Why this matters:

Notebook users need to know which record and residue failed. Counts alone confirm that something is wrong but do not let a researcher repair a FASTA file or filter affected records without reimplementing validation outside the binding.

Required fix:

- Expose Python validation sequence details with record id, normalized sequence, validity, warning list, error list, residue, and position.
- Add Python tests for ambiguous and invalid residues that assert diagnostic positions are visible.
- Update `packages/rust/biors-python/README.md` and `docs/python-api.md` to show how to inspect validation diagnostics before model-input construction.

### A-094: `dataset_hash` changes when identical FASTA content is moved to another path

Severity: dataset provenance correctness gap.

Evidence:

- `DatasetHashInput` includes `resolved_files` and `samples`.
- `DatasetFile.path` and `DatasetSample.file_path` are `path.display().to_string()`, so local filesystem paths are part of the dataset hash input.
- Docs describe `dataset_hash` as being over descriptor, metadata, files, and sample mapping, but do not warn that local path strings affect the hash.
- Behavior check in `/tmp`: two copies of the same FASTA bytes with the same descriptor produced the same file SHA-256 but different `dataset_hash` values because the paths were `/tmp/.../a/input.fasta` and `/tmp/.../b/input.fasta`.

Why this matters:

Researchers often move datasets between machines, project directories, or mounted volumes. A dataset identity hash should be stable across path relocation when content, descriptor, metadata, record IDs, and ordering are unchanged, or it should be clearly labeled as a local mapping hash rather than a dataset content identity.

Required fix:

- Split the concepts into `dataset_content_hash` and `dataset_mapping_hash`, or remove local absolute/relative paths from `dataset_hash`.
- If paths must remain in a mapping hash, hash package-relative or user-specified logical paths rather than host-local paths.
- Update docs and schemas to name the semantics explicitly.
- Add a regression test that copies identical FASTA content to two directories and proves the content-level dataset hash is stable.

### A-095: `cache clean` can target broad arbitrary directories despite the safety claim

Severity: destructive-operation release blocker.

Evidence:

- `validate_clean_root` rejects only roots with fewer than two path components, `/`, and `.`.
- The clean implementation recursively collects every regular file under the chosen root and deletes those files when `--yes` is supplied.
- Docs say `cache clean` rejects broad roots.
- Behavior check in `/tmp`: `biors cache clean --root /tmp --dry-run` succeeded and reported 29,003 files as removable.
- Behavior check with a temporary non-cache directory: `biors cache clean --root <tmp> --dry-run` reported ordinary files such as `not-cache.txt` and `project/src/main.rs` as removal targets.

Why this matters:

This is a user-facing destructive command. A typo or broad `--root` can turn a cache cleanup into a recursive file deletion outside the bio-rs artifact store. The current `--dry-run` requirement helps only if users always inspect carefully; the command itself should enforce a narrow cache-root policy.

Required fix:

- Require the clean root to be the default `.biors/artifacts`, under `BIORS_ARTIFACT_STORE`, or explicitly shaped like a bio-rs artifact store with expected subdirectories.
- Reject known broad roots such as `/tmp`, home directories, repository roots, and directories that do not contain the artifact-store marker/layout.
- Add tests for `--root /tmp`, a generic project directory, and the valid `.biors/artifacts` root.
- Consider adding an artifact-store marker file before allowing `--yes` deletion.

### A-096: `tokenizer convert-hf --output` suggests manifest paths that `package validate` rejects

Severity: package authoring contract gap.

Evidence:

- `convert_huggingface_tokenizer_config` sets `package_tokenizer_asset.path` to `output_path.display().to_string()` when `--output` is supplied.
- The docs show `--output ./protein-package/tokenizers/protein-20-special.json`, but the CLI accepts any output path, including absolute paths.
- Package manifest validation requires asset paths to be relative to the package root and under the declared `tokenizers` directory.
- Behavior check in `/tmp`: `biors tokenizer convert-hf <config> --output /tmp/.../absolute-tokenizer.json` returned both `output_path` and `package_tokenizer_asset.path` as the absolute `/tmp/.../absolute-tokenizer.json`.
- Behavior check in `/tmp`: copying that suggested absolute path into `examples/protein-package/manifest.json` made `biors package validate` fail with `package.invalid_asset_path` and exit status 2.

Why this matters:

The conversion output explicitly advertises a manifest `tokenizer` asset suggestion. If a researcher follows the machine-readable suggestion from the CLI, the generated package contract can become invalid. This is especially likely in notebooks or scripts that write conversion output to a temp or build directory and then copy JSON fragments into a package manifest.

Required fix:

- Keep `output_path` as the host write destination, but emit `package_tokenizer_asset.path` as a package-relative path such as `tokenizers/<profile>.json`.
- If the provided output path is outside a known package root, add a warning that the output file must be copied into the package `tokenizers/` directory before using the manifest fragment.
- Consider adding an optional `--package-root` argument so the CLI can derive a validated package-relative tokenizer path when users intentionally write into a package directory.
- Add CLI tests for absolute `--output`, package-relative `--output`, and the generated manifest fragment passing `package validate`.

### A-097: Candle backend dependency footprint is not budgeted or feature-audited

Severity: dependency-minimization release follow-up.

Evidence:

- The core CLI dependency duplicate check is clean: `cargo tree --locked -p biors --duplicates` had nothing to print.
- The core library duplicate check is also clean: `cargo tree --locked -p biors-core --duplicates` had nothing to print.
- `biors-backend-candle` is an optional separate crate, but its normal dependency tree is much larger than the CLI tree in this audit environment: about 123 unique normal packages for `biors-backend-candle` versus about 49 for `biors`.
- `cargo tree --locked -p biors-backend-candle --duplicates` shows duplicate transitive versions such as `hashbrown`, `itertools`, and `thiserror`.
- `cargo tree --locked -p biors-backend-candle -e normal -i tokenizers` shows `tokenizers v0.22.2` pulled through `candle-core v0.10.2`.
- `cargo tree --locked -p biors-backend-candle -e normal -i zip` shows `zip v7.2.0` pulled through `candle-core v0.10.2`.
- `cargo tree --locked -p biors-backend-candle -e normal -i rayon` shows Rayon through both `candle-core` and tokenizer/GEMM paths.

Why this matters:

The repository positions heavy runtime capabilities as optional and dependency-light by default. Keeping Candle in a separate crate is the right boundary, but the published backend crate still needs an explicit dependency budget because it brings tokenizers, zip parsing, Rayon, GEMM, and duplicate transitive versions even before model-specific features are added. Without a budget, backend additions can grow supply-chain, compile-time, and binary-size risk unnoticed.

Required fix:

- Add dependency-budget checks for each published crate, not only a generic “keep dependencies light” guideline.
- Run `cargo machete`, `cargo udeps`, `cargo tree --duplicates`, and a normal dependency count in CI or release-readiness for published crates.
- Decide whether the Candle adapter should expose narrower features, document unavoidable Candle transitive dependencies, or delay publishing the backend crate until the dependency footprint is acceptable for researchers.
- Add a release note template field for dependency changes so backend dependency growth is reviewed explicitly.

### A-098: Duplicate dataset metadata keys are silently overwritten

Severity: dataset provenance correctness gap.

Evidence:

- `dataset inspect` accepts repeatable `--metadata key=value` options.
- `parse_metadata` stores values in a `BTreeMap` and calls `metadata.insert(key.to_string(), val.to_string())`.
- It validates empty keys/values, but it does not reject duplicate keys or record that an earlier value was replaced.
- Behavior check in `/tmp`: `biors dataset inspect --metadata organism=human --metadata organism=mouse <input.fasta>` succeeded and emitted only `"organism": "mouse"`.
- `dataset_hash` is computed from the final metadata map, so the output gives no indication that the user supplied conflicting metadata.

Why this matters:

Dataset descriptor metadata is part of the provenance hash. Silent overwrites make it easy to lose source, organism, split, license, or filtering metadata in scripts where options are concatenated from multiple places. For research workflows, conflicting metadata should be visible and actionable rather than last-write-wins.

Required fix:

- Reject duplicate metadata keys with a stable error such as `dataset.duplicate_metadata_key`.
- Include the duplicate key in the error location/message.
- Add CLI tests for duplicate metadata, whitespace-normalized duplicate keys, and the valid multi-key case.
- If last-write-wins is intentionally kept, document it explicitly and add a warning field to the dataset inspect output.

### A-099: Package validation does not parse referenced pipeline config artifacts

Severity: package validation correctness gap.

Evidence:

- `validate_package_manifest` checks that pipeline step config paths are non-empty, but it does not parse the referenced `biors.pipeline.v0` file.
- `validate_package_manifest_artifacts` checks that the referenced pipeline config file exists and matches its checksum, but it does not validate the config content against `load_pipeline_config` / `validate_pipeline_config`.
- `docs/package-conversion.md` says validation checks declared layout placement and checksums, but for researcher-facing packages the pipeline config is also part of the preprocessing contract.
- Behavior check in `/tmp`: replacing `examples/protein-package/pipelines/protein.toml` with a file containing `export.max_length = 0` and `padding = "invalid"`, then updating the manifest checksum to match that invalid file, still made `biors package validate` return `ok: true`, `valid: true`, and `issues: []`.
- Running `biors pipeline --config` on the same file failed with `pipeline.invalid_config: export.max_length must be greater than zero`.

Why this matters:

A package can validate as complete while its declared preprocessing config cannot execute. Researchers may treat package validation as the preflight step before sharing or running a package, so it should catch invalid pipeline configs when those configs are part of the manifest contract.

Required fix:

- Extend package artifact validation to parse pipeline config artifacts referenced by preprocessing/postprocessing steps.
- Reuse the same pipeline config validator used by `biors pipeline --config`, or move the validator into a shared non-CLI module.
- Include structured package validation issues for invalid pipeline config content, preserving the config path and the inner pipeline error.
- Add regression tests where the config file exists and checksum matches, but `export.max_length`, `export.padding`, or an unknown field is invalid.

### A-100: Package validation does not parse referenced tokenizer config artifacts

Severity: package validation correctness gap.

Evidence:

- `validate_package_manifest_artifacts` verifies tokenizer file existence and checksum, but it does not parse the tokenizer JSON into `ProteinTokenizerConfig`.
- `validate_package_manifest` does not check that the declared tokenizer asset name/profile is consistent with the referenced tokenizer file content.
- Behavior check in `/tmp`: replacing `examples/protein-package/tokenizers/protein-20-special.json` with `{"profile":"bad","add_special_tokens":"yes"}` and setting the manifest tokenizer checksum to the canonical hash accepted by bio-rs made `biors package validate` return `ok: true`, `valid: true`, and `issues: []`.
- Running `biors tokenize --config` on the same tokenizer file failed with `json.invalid: unknown variant 'bad', expected 'protein-20' or 'protein-20-special'`.

Why this matters:

Tokenizer configs define how FASTA records become token IDs. A package that passes validation with an invalid tokenizer config is not immediately usable by researchers and can fail only later during preprocessing, after the package has already been shared or accepted by CI.

Required fix:

- Parse tokenizer config artifacts during package validation when a tokenizer asset is present.
- Check that `tokenizer.name`, `tokenizer.contract_version`, and the tokenizer config profile agree.
- Report invalid tokenizer config content as structured package validation issues rather than deferring failure to `biors tokenize`.
- Add regression tests where tokenizer checksum matches but profile, JSON type, or special-token policy is invalid.

### A-101: Example package vocab artifact is accepted even though the public vocab loader rejects it

Severity: package/example contract correctness gap.

Evidence:

- `examples/protein-package/manifest.json` declares `vocab.path: "vocabs/protein-20.json"` and `vocab.contract_version: "protein-20.v0"`.
- `examples/protein-package/vocabs/protein-20.json` encodes `tokens` as strings such as `"A"`, `"C"`, and `"D"`.
- The public Rust API `load_vocab_json` deserializes into `Vocabulary`, whose `tokens` field is `Vec<VocabToken>` and each token must be an object with `residue` and `token_id`.
- Existing `loads_vocab_from_json_contract` test uses the object form: `{ "residue": "A", "token_id": 0 }`.
- Behavior check in `/tmp`: a scratch program linked against `biors_core` and calling `biors_core::tokenizer::load_vocab_json` on `examples/protein-package/vocabs/protein-20.json` failed with `invalid type: string "A", expected struct VocabToken`.
- The same package still passed `biors package validate examples/protein-package/manifest.json` with `ok: true`, `valid: true`, and `issues: []`.

Why this matters:

The example package is likely to be copied by researchers and package authors. A package should not declare a vocab artifact that the public vocab loader cannot parse, especially when `package validate` reports it as valid. This weakens confidence in example packages and in manifest validation as a release gate.

Required fix:

- Rewrite `examples/protein-package/vocabs/protein-20.json` to match the public `Vocabulary` JSON contract, or change the public vocab loader/schema to intentionally support the compact string-token form.
- Extend package validation to parse vocab assets and check `vocab.name`, `vocab.contract_version`, and token count/order against the declared contract.
- Add a regression test that validates the committed example vocab through `load_vocab_json`.
- Add JSON schema coverage for vocab artifacts if vocab files remain part of the package contract.

### A-102: Pipeline lock generation does not verify that `--config` belongs to the supplied package

Severity: reproducibility/package provenance gap.

Evidence:

- `run_pipeline` loads `--package` only when `--write-lock` is supplied, validates the package manifest, and passes `PipelineLockPackage` to `write_pipeline_lock`.
- `write_pipeline_lock` records both `pipeline_config.path` and the package manifest path/name/model/runtime fields, but it does not check that the config path is one of the pipeline config artifacts declared by the package manifest.
- Docs describe `--package` lock context as pinning the package manifest path, model checksum, runtime backend, runtime target, and backend version alongside the pipeline config hash.
- Behavior check in `/tmp`: `biors pipeline --config examples/pipeline/protein.toml --package examples/protein-package/manifest.json --write-lock <tmp>/pipeline.lock` succeeded even though `examples/pipeline/protein.toml` is outside `examples/protein-package/` and is not the manifest-declared `pipelines/protein.toml`.
- The resulting lockfile recorded `pipeline_config.path: "examples/pipeline/protein.toml"` and `package.manifest_path: "examples/protein-package/manifest.json"` together under one provenance artifact.

Why this matters:

A lockfile that combines an arbitrary pipeline config with an unrelated package manifest can look package-backed even when the package did not declare that preprocessing config. Researchers may rely on the lock to prove that a package, model, tokenizer, config, and fixtures were pinned as one reproducible unit. The current command can accidentally create a misleading mixed-context lock.

Required fix:

- When `--package` is supplied, require `--config` to resolve to one of the manifest's preprocessing or postprocessing `config.path` artifacts.
- Compare canonical paths after resolving manifest-relative package paths and config paths.
- If external configs are intentionally allowed, add an explicit `external_config: true` or `package_config_match: false` field and warn in the lock output.
- Add tests for matching package config, external config rejection/warning, and same-basename-but-different-directory configs.

### A-103: `package convert-project` scans hidden/cache environments and can package the wrong model

Severity: package authoring correctness gap.

Evidence:

- `run_package_convert_project` calls `find_first_file(&args.project_dir, &["onnx"])` when `--model` is not supplied.
- `find_file` recursively walks every directory returned by `std::fs::read_dir` and does not skip `.venv`, `.git`, `.cache`, `target`, `__pycache__`, notebook checkpoints, or other generated/cache directories.
- The first matching file is selected; there is no candidate list, deterministic sorting, or warning for ignored/hidden directories.
- Docs say `package convert-project` scans the project directory for the first `.onnx` model and advise `--model` when the project has multiple candidates, but they do not mention hidden environments or caches.
- Behavior check in `/tmp`: a project with `.venv/cache/cached.onnx` and `export/real.onnx` produced a package containing `models/cached.onnx`; the packaged file content was the cached model, not the explicit export.

Why this matters:

Real Python/Hugging Face projects commonly contain virtual environments, caches, notebooks, downloaded artifacts, and multiple exports. A packaging helper should not silently select a cached or hidden model artifact that the researcher did not intend to publish.

Required fix:

- Skip known generated/cache directories by default: `.git`, `.hg`, `.svn`, `.venv`, `venv`, `env`, `.cache`, `__pycache__`, `.ipynb_checkpoints`, `target`, `build`, and `dist`.
- Sort directory entries deterministically before scanning.
- When more than one candidate remains, fail with a candidate list and require `--model`.
- Add tests for hidden/cache directories, multiple valid export candidates, and explicit `--model` override.

### A-104: Package pipeline configs can read absolute external inputs while package validation still passes

Severity: package portability/reproducibility gap.

Evidence:

- Package manifest artifact paths are protected by `validate_package_relative_path`, which rejects absolute paths and `..` traversal for manifest-declared assets.
- `validate_package_manifest_artifacts` validates only that a declared pipeline config file exists and has the declared checksum. It does not parse the config and does not inspect `input.path`.
- `load_pipeline_config` resolves `input.path` differently: if the path is absolute, it returns the absolute path directly.
- Behavior check in `/tmp`: copied `examples/protein-package`, changed `pipelines/protein.toml` to `input.path = "/tmp/.../external.fasta"`, recomputed the pipeline config checksum in the manifest, and ran `biors package validate <tmp>/pkg/manifest.json`.
- The package validation result was `ok: true`, `valid: true`, and `issues: []`.
- Running `biors pipeline --config <tmp>/pkg/pipelines/protein.toml --package <tmp>/pkg/manifest.json --write-lock <tmp>/pipeline.lock` succeeded and the lock recorded `execution.input_path` as the absolute `/tmp/.../external.fasta`.

Why this matters:

Docs say package paths stay rooted in the package directory so packages remain portable and self-contained. A package can currently pass validation while its declared preprocessing config depends on an external local file. Researchers moving the package to another machine or archiving it for reproducibility will not have the real input dependency captured by the manifest.

Required fix:

- When package validation sees a `PipelineConfigArtifact`, parse the config and validate `input.path` against package portability rules.
- Decide whether pipeline `input.path` is relative to the config file or package root, document that explicitly, and enforce it consistently.
- Reject absolute paths and config-relative traversal that escapes the package root for package-declared pipeline configs.
- Add tests where a package pipeline config points to an absolute FASTA, to `../../outside.fasta`, and to the intended package fixture.

### A-105: `package convert-project` can silently package a hidden tokenizer config instead of the intended export

Severity: package authoring correctness gap.

Evidence:

- `run_package_convert_project` auto-detects `tokenizer_config.json` with `find_named_file(&args.project_dir, "tokenizer_config.json")` when `--tokenizer-config` is not supplied.
- `find_named_file` uses the same unsorted recursive `find_file` helper as model detection and does not skip `.venv`, `.cache`, `__pycache__`, notebook checkpoints, or downloaded model caches.
- Behavior check in `/tmp`: created a project with `.venv/cache/tokenizer_config.json` containing `{"profile":"protein-20","add_special_tokens":false}` and `export/tokenizer_config.json` containing `{"profile":"protein-20-special","add_special_tokens":true}`.
- Running `biors package convert-project <project> --model <project>/export/real.onnx ...` without `--tokenizer-config` created `tokenizers/protein-20.json`, not `tokenizers/protein-20-special.json`.
- The generated manifest declared `tokenizer.name: "protein-20"` and `preprocessing[0].contract: "protein-20"`.
- `biors package validate <generated>/manifest.json` still returned `ok: true`, `valid: true`, and `issues: []`.

Why this matters:

Tokenizer profile choice changes token IDs, special-token boundaries, and downstream model-input shape assumptions. A package helper for researchers should not silently select a cached tokenizer config from a hidden environment while the intended exported config is elsewhere in the project tree.

Required fix:

- Apply the same hidden/cache directory skip list and deterministic candidate sorting to tokenizer config detection.
- If more than one tokenizer config remains, fail with candidate paths and require `--tokenizer-config`.
- Prefer tokenizer configs near the selected model only if that policy is explicitly documented and tested.
- Add regression tests for hidden tokenizer configs, multiple visible configs, and explicit `--tokenizer-config` override.

### A-106: Full `scripts/check.sh` does not verify that the benchmark markdown matches the JSON artifact

Severity: benchmark/release gate gap.

Evidence:

- `scripts/check-benchmark-docs.sh` runs `python3 scripts/check-benchmark-artifact.py`, renders `scripts/render_benchmark_report.py`, and diffs the rendered output against `benchmarks/fasta_vs_biopython.md`.
- `scripts/check-fast.sh` calls `scripts/check-benchmark-docs.sh`.
- `scripts/check.sh` labels the step as `==> benchmark docs` but only runs `python3 scripts/check-benchmark-artifact.py`; it does not call the docs diff script.
- Behavior check in the isolated main worktree: appending a bogus line to `benchmarks/fasta_vs_biopython.md` made `python3 scripts/check-benchmark-artifact.py` exit `0`, while `scripts/check-benchmark-docs.sh` exited `1` and showed the extra line in the diff.

Why this matters:

Before release, the slower/full check should be the strongest gate. At the moment, `scripts/check.sh` can pass even when the committed human-readable benchmark report no longer matches the benchmark JSON. That is exactly the kind of drift that can turn performance documentation into an unsupported claim.

Required fix:

- Change `scripts/check.sh` to run `scripts/check-benchmark-docs.sh` instead of only `python3 scripts/check-benchmark-artifact.py`.
- Keep the artifact-only parser check inside `check-benchmark-docs.sh` so the validation remains centralized.
- Add a lightweight CI assertion or script test that the full check includes the benchmark docs diff.

### A-107: Python `validate_package_manifest` cannot validate package artifacts, paths, or checksums

Severity: Python binding/package contract gap.

Evidence:

- `packages/rust/biors-python/src/lib.rs` implements `validate_package_manifest(manifest_json)` by deserializing `PackageManifest` and calling `package::validate_package_manifest(&manifest)`.
- That Rust function intentionally validates only manifest fields that do not require filesystem access.
- The CLI `biors package validate` calls `validate_package_manifest_artifacts(&manifest, &manifest_base_dir)`, which also checks manifest-relative artifact paths, file presence, checksums, and layout.
- Behavior check in `/tmp`: a manifest whose model path and other package assets were not present returned `in_memory_valid=true` and `in_memory_issues=[]` when passed through `validate_package_manifest`, while `biors package validate` reported `package.asset_read_failed` for missing model, tokenizer, vocab, pipeline, docs, and fixture assets.
- `docs/python-api.md` presents `biors.validate_package_manifest(manifest_json)` under "Package And Runtime Planning" but does not warn that this is not equivalent to CLI package validation.

Why this matters:

Python notebooks are a natural researcher workflow for inspecting and sharing packages. A Python function named like the CLI validation command can say a package is valid even when files are missing or checksums cannot be verified. That is too easy to misread as release/package readiness.

Required fix:

- Add a Python API that accepts a manifest path or `(manifest_json, base_dir)` and calls `validate_package_manifest_artifacts`.
- Rename or document the current function as field-only validation if it remains.
- Update Python docs and tests to cover missing artifacts, checksum mismatch, and a valid example package.
- Consider returning structured Python objects or a typed dict instead of a JSON string so callers can reliably inspect artifact-validation issues.

### A-108: MCP `package_validate` has the same field-only validation gap as Python

Severity: MCP package contract gap.

Evidence:

- `packages/rust/biors-mcp-server/src/server.rs` defines `PackageValidateParams` with only `manifest_json: String`; there is no package base directory or manifest path.
- The MCP `package_validate` tool deserializes `PackageManifest` and calls `biors_core::package::validate_package_manifest(&manifest)`.
- It does not call `validate_package_manifest_artifacts`, so it cannot check artifact paths, file presence, checksum mismatches, declared layout placement, symlink escapes, or malformed referenced configs.
- `packages/rust/biors-mcp-server/README.md` lists `package_validate` as "Validate a package manifest JSON string" without warning that it is field-only.
- MCP tests currently cover `doctor`, `tokenize`, and `validate`, but not `package_validate`.

Why this matters:

MCP is explicitly agent-callable tooling. A downstream agent can tell a researcher that a package manifest is valid even when the actual package directory is missing files or has checksum drift. That is especially risky because agent workflows often summarize results instead of exposing every validation detail.

Required fix:

- Add an MCP package validation tool that accepts either a manifest path or a manifest JSON plus a package base directory and calls `validate_package_manifest_artifacts`.
- Rename the current tool to `package_validate_fields` or document the field-only scope clearly.
- Add MCP integration tests for a valid package, missing model artifact, checksum mismatch, and invalid referenced tokenizer/pipeline configs after A-099/A-100 are fixed.
- Align MCP package validation output with the CLI package validation report schema.

### A-109: `package_layout.manifest` is not checked against the actual manifest path

Severity: package layout contract gap.

Evidence:

- `docs/package-format.md` defines `package_layout.manifest` as part of the portable v1 directory contract and shows it as `"manifest.json"`.
- `validate_declared_layout` checks that `package_layout.manifest` is non-empty and package-relative, but it does not compare it to the file that was actually loaded by `biors package validate`.
- Behavior check in `/tmp`: copied `examples/protein-package`, changed `package_layout.manifest` to `"other-manifest.json"`, and ran `biors package validate <tmp>/pkg/manifest.json`.
- Validation returned `ok: true`, `valid: true`, and `issues: []`.

Why this matters:

The manifest filename is part of the package layout contract that registries, archive tooling, and artifact stores may rely on. A package should not validate as self-consistent while declaring that its manifest lives at a different path than the path being validated.

Required fix:

- Pass the manifest path or manifest filename into artifact/layout validation.
- For v1 packages, require `package_layout.manifest` to match the manifest file path relative to the package root.
- If alternate manifest names are intentionally allowed, document the rule and include the declared manifest path in inspect/validation output.
- Add tests for the normal `manifest.json`, an alternate manifest path that is actually used, and a mismatched declared manifest path.

### A-110: Default CLI pulls in YAML parsing and `libyml` for a secondary config format

Severity: dependency-minimization release follow-up.

Evidence:

- `packages/rust/biors/Cargo.toml` includes `serde_yml` and `toml` as normal dependencies of the default CLI.
- `packages/rust/biors/src/cli/pipeline_config.rs` supports `.json`, `.toml`, `.yaml`, and `.yml` pipeline config files.
- `cargo tree --locked -p biors -e normal` shows `serde_yml` and its `libyml` transitive dependency in the default CLI tree.
- The core package stays lighter; `cargo tree --locked -p biors-core -e normal` has no YAML/TOML config parser dependencies.
- The repository goal is dependency-light tooling, and pipeline configs can already be expressed as JSON or TOML.

Why this matters:

YAML support is convenient, but it adds a native parser dependency to the default CLI path for a secondary config syntax. For institutional research environments, smaller and more predictable dependency trees improve build reliability and auditability. This does not mean YAML must be removed, but it should be an explicit release decision rather than an accidental default dependency.

Required fix:

- Decide whether YAML pipeline configs are part of the default public contract.
- If yes, document the `serde_yml`/`libyml` dependency and keep a dependency-budget entry for it.
- If no, remove YAML support from the default CLI or feature-gate it behind an optional Cargo feature.
- Keep JSON/TOML examples as the dependency-light default in docs if YAML is optional.
- Add dependency tree snapshot checks so parser-format additions are reviewed before release.

### A-111: Package validation accepts empty shape-dimension strings

Severity: package contract correctness gap.

Evidence:

- `packages/rust/biors-core/src/package/validation.rs` only checks whether `DataShape.shape` is an empty vector.
- It does not check that each shape dimension string is non-empty after trimming.
- `schemas/package-manifest.v1.json` and `schemas/package-manifest.v0.json` define `shape.shape.items` as plain strings without `minLength`.
- Behavior check in the isolated main worktree: copied `examples/protein-package`, set `expected_input.shape` to `["", "256"]`, set `expected_output.shape` to `[""]`, and ran `./target/release/biors package validate <tmp>/pkg/manifest.json`.
- Validation returned `ok: true`, `valid: true`, and `issues: []`.

Why this matters:

Shape metadata is part of the model/package contract. An empty dimension is not a meaningful tensor or model-input shape, but today it can be published as a valid package and later fail or be interpreted differently by downstream loaders.

Required fix:

- Add per-dimension validation in `validate_shape`, rejecting empty or whitespace-only shape entries.
- Add `minLength: 1` to `shape.shape.items` in both package-manifest schemas.
- Consider whether dimensions should be constrained to positive integers plus a small documented symbolic set such as `batch`, `sequence`, or `features`.
- Add CLI/schema tests for `shape: []`, `shape: [""]`, `shape: [" ", "256"]`, and a valid symbolic/numeric shape.

### A-112: Core workflow API can emit schema-invalid provenance input hashes

Severity: Rust public API/schema contract gap.

Evidence:

- `prepare_protein_model_input_workflow(input_hash, records, policy)` and `prepare_protein_model_input_workflow_with_invocation(...)` accept `input_hash: String`.
- The functions pass that string directly into `SequenceWorkflowProvenance.input_hash`.
- `schemas/sequence-workflow-output.v0.json` requires `provenance.input_hash` to match `^fnv1a64:[0-9a-f]{16}$`.
- The CLI path computes that hash from FASTA reader input, but direct Rust, Python, WASM, and MCP callers can bypass the CLI computation path.
- Behavior check in `/tmp`: a scratch Rust program called `prepare_protein_model_input_workflow("not-a-fnv-hash", ...)`; the generated workflow JSON contained `"not-a-fnv-hash"` and returned successfully.

Why this matters:

The Rust API is documented as the core public contract. If it can produce workflow JSON that fails the repository's own schema, downstream tools can only rely on the CLI path. That weakens provenance for notebooks, services, and agent integrations that call the core API directly.

Required fix:

- Introduce a typed `InputHash` or validated constructor for workflow provenance.
- Prefer an API that accepts FASTA bytes/reader or parsed reader output and computes the input hash internally.
- If manual hashes remain supported, validate the expected `fnv1a64:<16 lowercase hex>` shape before constructing `SequenceWorkflowOutput`.
- Add tests that direct Rust API calls with invalid hashes fail, and schema tests that direct workflow output validates against `sequence-workflow-output.v0.json`.
- Align Python/WASM/MCP fixes with the same core-level hash constructor instead of each binding implementing its own string rules.

### A-113: Model-input JSON schema does not enforce record array invariants

Severity: model-input contract/schema gap.

Evidence:

- `schemas/model-input-output.v0.json` constrains `input_ids` items to integers in `0..=255`.
- It constrains `attention_mask` items to `0` or `1`.
- It does not express that `input_ids.length == attention_mask.length`.
- It does not express that fixed-length records should have length equal to `policy.max_length`, or that no-padding records should have length at most `policy.max_length`.
- The Rust builder always constructs matching vectors, and the Candle backend manually checks only the length equality before inference.

Why this matters:

Model-input JSON is a central interchange contract for Python, services, package fixtures, and runtime backends. A schema that accepts mismatched arrays can let malformed model-ready data pass a schema preflight and fail only later in backend-specific code.

Required fix:

- Add a shared `validate_model_input_payload` helper in `biors-core` for externally supplied JSON/runtime payloads.
- Use that helper in Candle, external-process adapters, service/package validation paths, and binding tests.
- If JSON Schema cannot express all cross-field invariants portably, document the schema as structural and pair it with the core semantic validator.
- Add negative tests for mismatched `input_ids`/`attention_mask` lengths, fixed-length outputs shorter than `max_length`, no-padding outputs longer than `max_length`, and non-binary masks.

### A-114: Standalone tokenize-output schema allows token IDs outside the core token range

Severity: tokenize contract/schema gap.

Evidence:

- `TokenizedProtein.tokens` is `Vec<u8>` in Rust, so core-produced token IDs are bounded to `0..=255`.
- `schemas/sequence-workflow-output.v0.json` defines nested tokenized records with token items constrained to `minimum: 0` and `maximum: 255`.
- `schemas/tokenize-output.v0.json` defines standalone `tokens` items with only `minimum: 0`; it does not set `maximum: 255`.
- `schemas/model-input-output.v0.json` correctly constrains `input_ids` to `0..=255`.

Why this matters:

Standalone `biors tokenize` output is a public contract that users can feed into notebooks, binding code, and model-input builders. The standalone schema should not accept token IDs that the core type and downstream model-input contract cannot represent.

Required fix:

- Add `maximum: 255` to `schemas/tokenize-output.v0.json` token items.
- Consider extracting shared schema definitions for tokenized records to avoid drift between tokenize, workflow, and model-input schemas.
- Add a negative schema test for a token ID of `256`.

### A-115: `SECURITY.md` describes an older, narrower security surface

Severity: security/release documentation gap.

Evidence:

- `SECURITY.md` says the current security surface is limited to local CLI and library processing of FASTA, JSON manifests, and fixture observations.
- Current `main` now promotes or publishes additional surfaces:
  - Python bindings
  - WASM/npm package
  - MCP server
  - offline service contract
  - optional Candle backend crate
  - external-process backend contracts
  - package conversion helpers
  - cache clean/inspect filesystem operations
- Several audit findings above are specifically about these newer surfaces, including package artifact validation gaps, symlink escape, WASM package runtime breakage, MCP error classification, and dependency/security-audit gates.

Why this matters:

Researchers and downstream integrators need to know what the project considers security-relevant. The current policy can make reports against bindings, agent tools, package conversion, runtime adapters, and local filesystem operations look out of scope even though those surfaces are part of the release.

Required fix:

- Update `SECURITY.md` scope to cover all promoted/published crates and bindings.
- Explicitly include local filesystem safety, package artifact validation, external-process adapter execution, package conversion scanning, WASM/npm distribution, MCP tool inputs, and optional Candle model artifact loading.
- Clarify that no biological/user data should be uploaded externally by default.
- Add a release checklist item that `SECURITY.md` is reviewed whenever a new public surface is added.

### A-116: Benchmark comparison script silently ignores removed datasets and workloads

Severity: benchmark regression tooling gap.

Evidence:

- `scripts/compare-benchmark-artifacts.py` builds `before_datasets` and `after_datasets`, then iterates only over `before_datasets.keys() & after_datasets.keys()`.
- It does the same intersection-only comparison for workload keys and implementation keys.
- If a benchmark artifact accidentally drops a dataset, workload, or implementation, the script prints no row and exits `0`.
- The current release concerns include benchmark expansion for newer features, so missing benchmark coverage should be visible rather than silently skipped.

Why this matters:

Benchmark comparison is useful only if it detects both slower numbers and missing coverage. A release candidate could remove the large FASTA workload or a bio-rs implementation entry and still produce a clean comparison table, making performance regressions or coverage loss easy to miss.

Required fix:

- Make `compare-benchmark-artifacts.py` report and exit non-zero on removed datasets, workloads, or implementations unless an explicit `--allow-missing` flag is passed.
- Also report added datasets/workloads so release reviewers can see when the benchmark matrix changed.
- Add a small fixture test for before/after artifacts with one missing workload.

### A-117: Sequence workflow schema is too command-specific for pipeline-reused workflow payloads

Severity: workflow/pipeline schema contract gap.

Evidence:

- `schemas/sequence-workflow-output.v0.json` requires `provenance.invocation.command` to be exactly `"biors workflow"`.
- `packages/rust/biors/src/cli/workflow.rs::workflow_output` is reused by `biors workflow`, no-config `biors pipeline`, and config-driven `biors pipeline --config`.
- `packages/rust/biors/src/cli/pipeline.rs` calls `workflow_output("biors pipeline", ...)` for no-config pipeline and `workflow_output("biors pipeline --config", ...)` for config pipeline.
- Behavior check in the isolated main worktree: `./target/release/biors pipeline --max-length 4 examples/protein.fasta` emitted nested `data.workflow.provenance.invocation.command` as `biors pipeline`.
- A-091 found that `schemas/pipeline-output.v0.json` does not validate the nested workflow object; a direct reference to the current sequence workflow schema would fail on this command const even though the payload is otherwise a `SequenceWorkflowOutput`.

Why this matters:

Pipeline output embeds the same workflow contract researchers use for provenance and model readiness. The schema should distinguish workflow payload shape from one particular CLI entrypoint, otherwise schema parity between `workflow` and `pipeline` cannot be enforced cleanly.

Required fix:

- Split the reusable workflow payload schema from the CLI-specific invocation constraint, or allow the known command values: `biors workflow`, `biors pipeline`, and `biors pipeline --config`.
- Update A-091's fix to validate pipeline nested workflow against the reusable workflow payload schema, not a schema that only accepts the direct CLI command.
- Add schema tests for direct workflow output, no-config pipeline nested workflow, and config pipeline nested workflow.

### A-118: 1.0 public-contract candidates omit already promoted service APIs and schema

Severity: release contract/documentation gap.

Evidence:

- `docs/public-contract-1.0-candidates.md` lists Rust API candidates and schema candidates for stabilization, but it does not list the `service` module APIs.
- `packages/rust/biors-core/src/lib.rs` exposes `pub mod service`.
- `packages/rust/biors-core/src/service.rs` exposes public service contract types and functions, including `SERVICE_INTERFACE_SCHEMA_VERSION`, `ServiceInterfaceDocument`, `RuntimeServiceSeparation`, `OpenApiDirection`, `ServiceRoute`, `current_service_interface_document`, `service_interface_document`, and `service_routes`.
- `docs/rust-api.md` documents `service` as part of the comprehensive `biors-core` public API.
- `README.md` promotes `biors service contract` and the service-host contract.
- `docs/cli-contract.md` says service interface payloads use `schemas/service-interface-output.v0.json`, but `docs/public-contract-1.0-candidates.md` omits that schema from its schema candidate list.

Why this matters:

Before a 1.0 release, researchers and service integrators need a clear answer on whether the service contract is stable, unstable, or intentionally excluded. Right now the API is public and documented elsewhere, but the stabilization document does not classify it, so schema tests and SemVer expectations can miss it.

Required fix:

- Decide whether the service contract is in or out of the 1.0 stable surface.
- If it is in, add the service Rust APIs and `schemas/service-interface-output.v0.json` to `docs/public-contract-1.0-candidates.md`, then add contract tests for the emitted CLI payload and `service_routes()`.
- If it is out, move it to `Not Yet Stable`, label the README/CLI docs accordingly, and avoid claiming it as a stable integration contract.
- Add a release-readiness check that cross-references schema files mentioned by `docs/cli-contract.md` against the public-contract candidate list or an explicit unstable list.

### A-119: CLI contract omits several public package options

Severity: CLI contract documentation gap.

Evidence:

- `packages/rust/biors/src/cli/package_args.rs` exposes `--doi` and `--force` for both `package init` and `package convert-project`.
- `packages/rust/biors/src/cli/package_args.rs` exposes additional `package convert` options: `--doi`, `--license-file`, `--citation-file`, `--models-dir`, `--tokenizers-dir`, `--vocabs-dir`, `--pipelines-dir`, `--fixtures-dir`, `--observed-dir`, and `--docs-dir`.
- `./target/release/biors package init --help`, `package convert-project --help`, and `package convert --help` show those options.
- `docs/cli-contract.md` lists the command forms but omits those public options.
- `docs/error-codes.md` documents `package.init_exists` as overwrite protection controlled by `--force`, so this is not an intentionally hidden internal flag.

Why this matters:

The CLI contract is the release-facing command source of truth. Package metadata and layout controls affect reproducibility, citation, redistribution, and overwrite safety for researcher-owned package directories. If they are omitted from the contract, release reviewers and downstream users cannot tell which options are supported, stable, or intentionally provisional.

Required fix:

- Update `docs/cli-contract.md` command signatures for `package init`, `package convert-project`, and `package convert` to include the missing public options.
- Mark layout-directory override options as advanced if they should remain supported but rarely used.
- Add a lightweight check that compares generated clap help for promoted commands against the command signatures documented in `docs/cli-contract.md`, or at least a release checklist that requires CLI-contract updates when package options change.

### A-120: Release workflow installs unpinned packaging tools for published artifacts

Severity: release reproducibility/dependency gate.

Evidence:

- `.github/workflows/release.yml` installs Python packaging with `python -m pip install --upgrade maturin`.
- The same workflow installs WASM packaging with `cargo install wasm-pack --locked`.
- Neither command pins a `maturin` or `wasm-pack` version.
- The workflow then uses those tools to build PyPI wheels/sdist and the npm WASM package that would be published on a tag.
- A-031 already covers moving GitHub Action refs; this is a separate moving-toolchain issue inside the release steps.

Why this matters:

Published artifacts should be reproducible from the tag. An unpinned `maturin` or `wasm-pack` release can change wheel metadata, npm glue generation, compatibility tags, included files, or build behavior without any repository diff. That is risky for researcher-facing packages and makes release failures hard to reproduce locally.

Required fix:

- Pin release packaging tools, for example `maturin==<known-good>` and `wasm-pack --version <known-good>` through a checked script or workflow env.
- Add local scripts that print the exact tool versions used for release artifacts.
- Consider checking the built wheel/sdist/npm package manifests in CI before publish, so version and file-list drift is caught before a tag tries to publish.

### A-121: 1.0 public-contract candidates are not a complete classification of the current Rust API

Severity: release contract gap before any 1.0 stabilization claim.

Evidence:

- `docs/rust-api.md` says it covers every public module, type, trait, and function exposed by `biors-core`.
- `packages/rust/biors-core/src/lib.rs` exposes public modules including `package`, `runtime`, `sequence`, `service`, `tokenizer`, `verification`, `versioning`, and `workflow`.
- `docs/public-contract-1.0-candidates.md` lists only a subset of those public APIs as stabilization candidates and has no section that classifies the remaining public APIs as intentionally unstable.
- Examples of public APIs documented in `docs/rust-api.md` but missing from the 1.0 candidate list include:
  - `validate_package_manifest`, `inspect_package_manifest`, `plan_runtime_bridge`, `convert_package_manifest`, `diff_package_manifests`, `plan_package_schema_migration`, `read_package_file`, `resolve_package_asset_path`, and `PackageArtifactError`
  - `stable_input_hash`, `StableInputHasher`, `verify_package_outputs`, `verify_package_outputs_with_observation_base`, `FixtureVerificationResult`, and `VerificationStatus`
  - `validate_model_input_policy`, `ModelInputRecord`, and `ModelInputBuildError`
  - `tokenize_protein`, `summarize_fasta_records_reader`, `summarize_tokenized_proteins`, `load_protein_20_vocab`, `protein_20_unknown_token_policy`, `SpecialTokenSet`, `TokenizedProtein`, `Vocabulary`, and `TokenizerError`
  - `package_manifest_policy`, `pipeline_config_policy`, `manifest_schema_compatibility`, `manifest_schema_migration_plan`, and the versioning policy types
- A-118 covers the service-specific version of the same issue; this finding covers the wider Rust API classification drift.

Why this matters:

Before 1.0, every public API should be classified as stable candidate, explicitly unstable, hidden behind a feature/experimental module, or made private. Leaving many public APIs unclassified makes SemVer expectations ambiguous and increases the chance that researchers build on a surface the maintainers intended to keep fluid.

Required fix:

- Generate or maintain an explicit public API inventory and require each item to be classified as `stable candidate`, `unstable`, `experimental`, or `internal but accidentally public`.
- Expand `docs/public-contract-1.0-candidates.md` to include all stable candidates and a concrete unstable list, not only a partial hand-written subset.
- For APIs intended to remain internal, reduce visibility before 1.0 or move them behind an experimental module/feature with clear documentation.
- Add a release-readiness check that fails when new `pub` APIs are added without a corresponding stability classification.

### A-122: Python `tokenize_protein` bypasses the normalization used by FASTA-backed tokenization

Severity: Python researcher-facing API correctness gap.

Evidence:

- `packages/rust/biors-python/src/lib.rs::tokenize_fasta_records` calls `biors_core::tokenizer::tokenize_fasta_records(fasta_text)`, which parses FASTA records through the core FASTA path.
- `packages/rust/biors-python/src/lib.rs::tokenize_protein(sequence)` constructs `sequence::ProteinSequence { id: "user", sequence: sequence.as_bytes().to_vec() }` directly.
- That direct path does not call `normalize_sequence` or the FASTA parser before `tokenizer::tokenize_protein(&protein)`.
- `docs/python-api.md` presents `tokenize_protein("ACDEFGHIK")` as a convenience single-sequence API, but it does not warn that callers must pre-normalize casing/whitespace themselves.
- A-079 covers the underlying direct Rust API divergence; this finding is the Python binding exposure of the same normalization hazard.

Why this matters:

Notebook users commonly paste lowercase sequences or strings with whitespace from upstream tools. FASTA-backed bio-rs commands normalize those inputs, but the Python single-sequence helper can classify the same biological sequence differently. That makes Python notebooks less predictable than the CLI and full FASTA Python path.

Required fix:

- Normalize the `sequence` argument in `tokenize_protein` before constructing `ProteinSequence`, or rename/document it as a low-level pre-normalized API.
- Add Python tests showing lowercase and whitespace-containing sequences behave consistently with `tokenize_fasta_records` when that is the intended contract.
- If strict pre-normalized input is desired, return a clear `ValueError` instead of silently producing warning/error tokenization output.

### A-123: Python interop docs mention package manifest inspection that is not exposed in Python

Severity: Python documentation/API mismatch.

Evidence:

- `docs/python-interop.md` says the PyO3 bindings cover "Package manifest inspection and runtime bridge planning".
- `docs/python-api.md` documents only `validate_package_manifest(manifest_json: str) -> str` and `plan_runtime_bridge(manifest_json: str) -> str` under "Package And Runtime Planning".
- `packages/rust/biors-python/src/lib.rs` exposes `validate_package_manifest` and `plan_runtime_bridge`, but no `inspect_package_manifest` Python function.
- `packages/rust/biors-python/python/biors/__init__.py` and `__all__` likewise export no package-inspection helper.

Why this matters:

Researchers reading the interop guide may expect the Python wheel to expose the same manifest inspection summary as the CLI/Rust API. The current wheel exposes validation and bridge planning only, so the docs should not imply an inspection function exists.

Required fix:

- Either add an `inspect_package_manifest(manifest_json: str) -> str` Python helper backed by `biors_core::package::inspect_package_manifest`, or change `docs/python-interop.md` to say package validation and runtime bridge planning.
- Add a Python test covering the exported package helper list so future docs do not drift from `__all__`.

### A-124: README schema inventory omits the service interface schema

Severity: public documentation inventory gap.

Evidence:

- `schemas/service-interface-output.v0.json` exists.
- `docs/cli-contract.md` says `biors service contract` uses `schemas/service-interface-output.v0.json`.
- `packages/rust/biors/tests/schema_contract.rs` validates `biors service contract` against `schemas/service-interface-output.v0.json`.
- The README `schemas/` workspace inventory lists the other schema files but omits `service-interface-output.v0.json`.
- A quick comparison of `schemas/*.json` against README mentions found only that file missing.

Why this matters:

The README is the first file release reviewers and new contributors scan. If the service schema is omitted from the workspace inventory while the service contract is promoted elsewhere, the public surface looks less complete and release checklist updates can miss it.

Required fix:

- Add `service-interface-output.v0.json` to the README `schemas/` inventory.
- Prefer generating or checking the README schema list from the actual `schemas/` directory so future schema additions are not missed.

### A-125: Benchmark issue template still reflects only the older FASTA/core benchmark scope

Severity: benchmark process/documentation gap.

Evidence:

- `.github/ISSUE_TEMPLATE/benchmark_performance_idea.md` asks for matched workload choices only for:
  - pure parse
  - parse plus validation
  - parse plus tokenization
- Its surface choices are only:
  - core library only
  - CLI end-to-end
- Current `main` promotes additional performance-sensitive surfaces: model-input construction, workflow/pipeline orchestration, package validation/verification, Python bindings, WASM bindings, MCP server overhead, service contract generation, and optional Candle backend execution.
- README text already says the `0.47.4` patch adds fixed-length model-input benchmark coverage, but the benchmark issue template still nudges contributors toward the older FASTA-only matrix.

Why this matters:

Benchmark requests are part of the release and contribution process. If the template omits newly promoted surfaces, performance work can keep expanding the old parse/tokenize matrix while missing regressions in the features researchers now use.

Required fix:

- Update the benchmark issue template to include model-input, workflow, pipeline, package validation/verification, Python, WASM, MCP, service contract, and Candle backend surfaces.
- Add workload fields for fixed-length model-input, no-padding model-input, package artifact validation, pipeline config execution, and binding round-trip overhead.
- Require the issue author to state whether the benchmark is a release claim, regression guard, smoke benchmark, or exploratory measurement.

### A-126: Published MCP crate metadata lacks discovery tags

Severity: package publication polish gap.

Evidence:

- `.github/workflows/release.yml` publishes `biors-mcp-server` to crates.io on tag releases.
- `packages/rust/biors-mcp-server/README.md` exists and documents usage and MCP client configuration.
- `cargo metadata --no-deps --format-version 1` reports `readme: "README.md"` for `biors-mcp-server`, so the local README is auto-detected.
- The same metadata reports empty `keywords` and empty `categories`.
- Other published Rust crates such as `biors`, `biors-core`, and `biors-backend-candle` declare `readme`, `keywords`, and `categories`.

Why this matters:

The MCP server is a public integration surface for agent-callable sequence tools. Crates.io users should be able to discover it through package metadata, especially because configuring an MCP client is not obvious from the binary name alone.

Required fix:

- Add appropriate `keywords` and `categories`, for example bioinformatics/MCP/tooling-oriented tags within crates.io limits.
- Optionally add an explicit `readme = "README.md"` for consistency with the other published crates, even though Cargo currently auto-detects it.
- Include this in package metadata checks before publishing new public crates.
