# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Rust tools for validating protein FASTA input and tokenizing FASTA records into
stable `protein-20` token ids.

## Features

- FASTA parsing for one or more protein sequences
- `protein-20` residue validation
- lowercase sequence normalization
- ambiguous residue reporting for `X`, `B`, `Z`, `J`, `U`, and `O`
- invalid residue reporting
- JSON array output from the CLI

## Quickstart

Inspect FASTA records:

```bash
cargo run -p biors -- inspect examples/protein.fasta
```

Tokenize FASTA records:

```bash
cargo run -p biors -- tokenize examples/protein.fasta
```

Tokenize a multi-record FASTA file:

```bash
cargo run -p biors -- tokenize examples/multi.fasta
```

Use the Rust library:

```toml
[dependencies]
biors-core = "0.3.0"
```

## Checks

```bash
scripts/check.sh
```

The check suite runs `cargo fmt`, `cargo check`, `cargo test`, and `cargo clippy`
with warnings denied.

## Workspace

```txt
packages/
  rust/
    biors/       CLI
    biors-core/  FASTA parsing and tokenization library
examples/
  multi.fasta
  protein.fasta
```

## Protein-20

```txt
A C D E F G H I K L M N P Q R S T V W Y
```

Token ids follow that order, starting at `0`.

## License

Dual licensed under MIT OR Apache-2.0.
