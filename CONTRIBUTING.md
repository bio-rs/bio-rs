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

## Asking questions or sharing use cases

Use the question / use case issue template when you are not sure whether a
workflow belongs in bio-rs yet. Small, concrete examples are more useful than
broad roadmap requests.
