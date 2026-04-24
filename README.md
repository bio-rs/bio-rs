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

Tokenize FASTA records from stdin:

```bash
cat examples/protein.fasta | cargo run -p biors -- tokenize -
```

Tokenize a multi-record FASTA file:

```bash
cargo run -p biors -- tokenize examples/multi.fasta
```

Use the Rust library:

```bash
cargo add biors-core
```

```rust
use biors_core::{summarize_tokenized_proteins, tokenize_fasta_records};

let tokenized = tokenize_fasta_records(">seq1\nACDE\n")?;
let summary = summarize_tokenized_proteins(&tokenized);

assert_eq!(summary.records, 1);
assert_eq!(tokenized[0].tokens, vec![0, 1, 2, 3]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## JSON Contracts

`tokenize` always emits an array of records:

```json
[
  {
    "id": "seq1",
    "length": 4,
    "alphabet": "protein-20",
    "valid": true,
    "tokens": [0, 1, 2, 3],
    "warnings": [],
    "errors": []
  }
]
```

`inspect` always emits a summary object:

```json
{
  "records": 1,
  "total_length": 4,
  "valid_records": 1,
  "warning_count": 0,
  "error_count": 0
}
```

## Checks

```bash
scripts/check.sh
```

The check suite runs `cargo fmt`, native Rust checks, a `biors-core`
`wasm32-unknown-unknown` build check, tests, and `cargo clippy` with warnings
denied.

Run the Rust library example:

```bash
cargo run -p biors-core --example tokenize
```

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
