# Contributing to bio-rs

Thanks for contributing to bio-rs.

## Prerequisites

- Rust stable toolchain (`rustup` + `cargo`)
- `wasm32-unknown-unknown` target for the core crate check:

```bash
rustup target add wasm32-unknown-unknown
```

- `cargo-deny` for dependency security audits:

```bash
cargo install --locked cargo-deny
```

Release maintainers also need pinned packaging tools such as `maturin`,
`wasm-pack`, and Node.js. Print the exact release pins with:

```bash
scripts/print-release-tool-versions.sh
```

## Local setup

```bash
git clone https://github.com/bio-rs/bio-rs.git
cd bio-rs
scripts/install-git-hooks.sh
cargo build
```

## Recommended workflow

1. Create a branch from `main`.
2. Keep changes focused (single concern per PR).
3. Add or update tests before behavior changes when possible.
4. Run `scripts/check-fast.sh` while iterating.
5. Run the surface-specific checks for the files you changed.
6. Run the full check script before pushing.

## Choosing an issue

If this is your first contribution, start with issues labeled
`good first issue`. These should be small enough to finish without knowing the
whole project.

Issues labeled `help wanted` are also open to contributors, but they may need
more discussion or domain context first.

Good first contributions include:

- README or example wording that makes current behavior clearer
- Quickstart verification on a fresh checkout
- small FASTA fixture additions
- clearer error messages for existing validation paths
- documentation fixes that separate current behavior from roadmap ideas

Documentation contributions are welcome. If a doc change updates public
behavior, include the command, input, output, or API surface it describes.

## Commands

Run the repository check suite:

```bash
scripts/check.sh
```

Run the faster local gate used by `pre-commit`:

```bash
scripts/check-fast.sh
```

Run tests only:

```bash
cargo test --workspace
```

Run formatting only:

```bash
cargo fmt --all -- --check
```

Run lint only:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

## Surface-specific checks

Start with `scripts/check-fast.sh` for normal iteration. Add the checks below
when your change touches that surface:

| Surface | Run when changed | Checks |
| --- | --- | --- |
| Rust core, CLI, schemas | `crates/biors-core`, `crates/biors`, `schemas/`, CLI docs | `cargo test -p biors-core`, `cargo test -p biors`, `scripts/check-fast.sh` |
| Python bindings | `crates/biors-python`, Python API docs, Python integration scripts | `maturin build --manifest-path crates/biors-python/Cargo.toml`, `python3 scripts/test-python-wheel.py <wheel>` |
| WASM/npm bindings | `crates/biors-wasm`, `docs/wasm-api.md`, npm package metadata | `wasm-pack test --node crates/biors-wasm`, `scripts/check-package-artifacts.sh` |
| MCP service | `crates/biors-mcp-server`, MCP JSON contracts | `cargo test -p biors-mcp-server --all-targets` |
| Package/release artifacts | `testdata/protein-package`, package schemas, release workflow, install docs | `scripts/check-package-artifacts.sh`, `python3 scripts/check-release-workflow.py` |
| Benchmarks | `benches/`, `benchmarks/`, performance docs or claims | `cargo bench -p biors-core --bench fasta_workloads`, `scripts/check-benchmark-docs.sh` |
| Dependencies/security | `Cargo.toml`, `Cargo.lock`, dependency policy, release gates | `python3 scripts/check-dependency-policy.py`, `scripts/check-security-audit.sh` |

Run `scripts/check.sh` before pushing a PR that changes more than docs. For
release-affecting changes, also review
[scripts/check-final-release.sh](scripts/check-final-release.sh); it is the
source of truth for packaging, registry, checksum, provenance, and install-flow
preflight. Local internal notes may exist under `docs/internal/`, but they are
not part of the public repository documentation set.

## Scope expectations

Current contribution priority areas:

- FASTA parser correctness and edge cases
- buffered FASTA reader APIs for large inputs
- protein-20 tokenizer behavior and diagnostics
- stable sequence workflow JSON and provenance contracts
- model-input contract fixtures and tokenizer config parity tests
- Python, WASM/npm, and MCP binding contract parity
- manifest validation/reporting clarity
- typed package validation issue codes
- fixture verification UX and reporting
- CLI, JSON schema, and package artifact contract coverage
- reproducible benchmark coverage
- dependency-light release surfaces and isolated optional integrations

## Benchmarks

For the reproducible FASTA benchmark:

```bash
python3 -m venv .venv-bench
. .venv-bench/bin/activate
pip install biopython
python scripts/benchmark_fasta_vs_biopython.py
```

The benchmark script updates both the JSON result artifact and the Markdown
report. To verify they are synchronized:

```bash
scripts/check-benchmark-docs.sh
```

The benchmark script now compares matched workloads for `biors-core` and
Biopython:

- pure parse
- parse plus validation
- parse plus tokenization

It measures core-library throughput only for bio-rs and excludes CLI startup and
pretty JSON serialization overhead.

If proposing larger roadmap work, open an issue first to align scope.

## Pull request checklist

Before opening a PR:

- [ ] Tests added/updated for changed behavior
- [ ] `scripts/check-fast.sh` passes locally while iterating
- [ ] Surface-specific checks above run for changed Python/WASM/MCP/package/release/dependency areas
- [ ] `scripts/check.sh` passes locally before pushing non-doc changes
- [ ] Benchmark report is regenerated when benchmark JSON changes
- [ ] README/docs updated when public behavior changed
- [ ] Benchmarks updated when making performance claims
- [ ] Dependency, package artifact, and release workflow checks run when those surfaces changed
- [ ] PR description explains **what changed** and **why**

## Reporting bugs

Please include:

- rust/cargo versions
- operating system
- minimal reproduction input
- expected vs actual behavior
- CLI output or stack traces

## Asking questions or sharing use cases

Use the question / use case issue template when you are not sure whether a
workflow belongs in bio-rs yet. Small, concrete examples are more useful than
broad roadmap requests.
