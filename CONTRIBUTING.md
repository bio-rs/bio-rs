# Contributing to bio-rs

Thanks for contributing to bio-rs.

## Prerequisites

- Rust stable toolchain (`rustup` + `cargo`)
- `wasm32-unknown-unknown` target for the core crate check:

```bash
rustup target add wasm32-unknown-unknown
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
5. Run the full check script before pushing.

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

## Scope expectations

Current contribution priority areas:

- FASTA parser correctness and edge cases
- buffered FASTA reader APIs for large inputs
- protein-20 tokenizer behavior and diagnostics
- manifest validation/reporting clarity
- typed package validation issue codes
- fixture verification UX and reporting
- CLI and JSON contract coverage
- reproducible benchmark coverage

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
- [ ] `scripts/check.sh` passes locally
- [ ] Benchmark report is regenerated when benchmark JSON changes
- [ ] README/docs updated when public behavior changed
- [ ] Benchmarks updated when making performance claims
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
