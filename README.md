# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Crates.io Core](https://img.shields.io/crates/v/biors-core.svg)](https://crates.io/crates/biors-core)
[![Crates.io CLI](https://img.shields.io/crates/v/biors-cli.svg)](https://crates.io/crates/biors-cli)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Rust/WASM tools for biological AI models.

`bio-rs` turns Python-born bio-AI models into portable, inspectable tools for
CLIs, browsers, servers, and agents.

Python is where many biological AI models are born. `bio-rs` is where the
model-facing tools around them become reproducible, agent-callable, and easier
to ship outside a research notebook.

`bio-rs` is open source under dual MIT OR Apache-2.0 licensing.

## Why this exists

Bio-AI does not need Rust to replace Python research workflows. It needs a
reliable tooling layer around model inputs, tokenizers, runners, browser demos,
and agent interfaces.

Rust is useful here because it is good at:

- predictable CLI and server tools
- portable WASM/browser execution
- safe input contracts for biological data
- reproducible single-binary distribution
- long-running services and agent-callable tools

## Current proof

The first target is intentionally small:

```txt
FASTA -> validated protein sequence -> token ids -> model-ready input
```

Currently implemented:

- FASTA parsing for one protein sequence
- `protein-20` residue validation
- lowercase sequence normalization
- ambiguous residue reporting for `X`, `B`, `Z`, `J`, `U`, and `O`
- invalid residue reporting
- token ids using a stable `protein-20` order
- JSON output for CLI/tool use
- `biors inspect` and `biors tokenize`

Not implemented yet:

- WASM bindings
- MCP/agent tools
- model inference runners
- external model tokenizer parity
- multi-FASTA batch processing

## Quickstart

From source:

```bash
cargo run -p biors-cli -- inspect examples/protein.fasta
```

```bash
cargo run -p biors-cli -- tokenize examples/protein.fasta --format json
```

Example output:

```json
{
  "id": "seq1",
  "length": 4,
  "alphabet": "protein-20",
  "valid": true,
  "tokens": [0, 1, 2, 3],
  "warnings": [],
  "errors": []
}
```

## Distribution

The first public distribution target is crates.io:

- `biors-core`: Rust library for protein input contracts and tokenization
- `biors-cli`: installable CLI package that provides the `biors` binary

After the first release, users should be able to install the CLI with:

```bash
cargo install biors-cli
```

Rust projects should be able to depend on the core crate with:

```toml
[dependencies]
biors-core = "0.1"
```

Before publishing, the release path should be:

```bash
scripts/check.sh
cargo publish -p biors-core --dry-run
cargo publish -p biors-cli --dry-run
cargo publish -p biors-core
cargo publish -p biors-cli
```

GitHub Releases with prebuilt binaries can come after the CLI API stabilizes.
WASM/npm distribution should wait until `biors-wasm` exists.

## Checks

This repo keeps the local pre-commit path and CI strict. Before committing,
run:

```bash
scripts/check.sh
```

The check suite runs:

- `cargo fmt --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo test --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Local git hooks are stored in `.githooks/`. Enable them with:

```bash
git config core.hooksPath .githooks
```

The hooks enforce:

- pre-commit: full `scripts/check.sh`
- pre-push: full `scripts/check.sh`
- commit-msg: Conventional Commit format

## Workspace

```txt
crates/
  biors-core/  FASTA parsing, protein validation, tokenization
  biors-cli/   CLI entrypoint for inspect/tokenize workflows
examples/
  protein.fasta
```

The repo is intended to grow as a monorepo for the bio-AI tooling stack:

1. `biors-core`: stable biological model input contracts
2. `biors-cli`: local inspection, validation, and tokenization
3. `biors-wasm`: browser-side validation and model input demos
4. `biors-runtime`: portable wrappers around Python-born model artifacts
5. `biors-mcp`: agent-callable tools
6. `biors-models`: small model integrations and reference examples

## Protein-20

The first alphabet is `protein-20`:

```txt
A C D E F G H I K L M N P Q R S T V W Y
```

Token ids follow that order, starting at `0`.

## Final goal

The long-term goal is to make useful biological AI models easier to package as
portable tools:

- CLI tools for local workflows
- WASM tools for browsers and demos
- server components for production systems
- agent-callable interfaces for automated research workflows

The first milestone is not folding or training. It is the stable input layer
that everything after it needs.
