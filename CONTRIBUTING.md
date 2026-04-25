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
cargo build
```

## Recommended workflow

1. Create a branch from `main`.
2. Keep changes focused (single concern per PR).
3. Add/update tests with behavior changes.
4. Run the full check script before pushing.

## Commands

Run the repository check suite:

```bash
scripts/check.sh
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

## Scope expectations (v0.8.1)

Current contribution priority areas:

- FASTA parser correctness and edge cases
- protein-20 tokenizer behavior and diagnostics
- manifest validation/reporting clarity
- fixture verification UX and reporting
- reproducible benchmark coverage

If proposing larger roadmap work, open an issue first to align scope.

## Pull request checklist

Before opening a PR:

- [ ] Tests added/updated for changed behavior
- [ ] `scripts/check.sh` passes locally
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
