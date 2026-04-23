# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Crates.io Core](https://img.shields.io/crates/v/biors-core.svg)](https://crates.io/crates/biors-core)
[![Crates.io CLI](https://img.shields.io/crates/v/biors.svg)](https://crates.io/crates/biors)
[![npm](https://img.shields.io/npm/v/biors.svg)](https://www.npmjs.com/package/biors)
[![PyPI](https://img.shields.io/pypi/v/biors.svg)](https://pypi.org/project/biors/)
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

### CLI (Rust)

Install the CLI:
```bash
cargo install biors
```

Inspect a protein sequence:
```bash
biors inspect examples/protein.fasta
```

Tokenize for AI model input:
```bash
biors tokenize examples/protein.fasta --format json
```

### Library (Rust)

Add to your project:
```toml
[dependencies]
biors-core = "0.1"
```

## Distribution

The project is distributed across multiple ecosystems:

- **crates.io**: `biors` (CLI), `biors-core` (Library)
- **npm**: `biors` (WASM bindings - coming soon)
- **PyPI**: `biors` (Python bindings - coming soon)

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

## Workspace Structure

The project is a monorepo managed under the `packages/` directory:

```txt
packages/
  rust/
    biors/       Main CLI tool and unified entrypoint
    biors-core/  Core protein parsing and tokenization logic
  npm/           WebAssembly bindings for JavaScript/TypeScript
  python/        High-performance Python bindings via PyO3
examples/
  protein.fasta
```

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
