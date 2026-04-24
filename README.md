# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Open source Rust/WASM tools for biological AI models.

Python is where many bio-AI models are born. bio-rs is where the tooling around
them becomes portable, inspectable, and easier to use from CLIs, browsers,
servers, and agents.

The project starts with a small, working seed module for protein FASTA
validation and tokenization, then grows toward a Rust/WASM/WebGPU migration
layer for Python-born bio-AI models and their preprocessing and postprocessing
dependencies.

## 1.0.0 Goal

bio-rs reaches `1.0.0` when a Python-born biological AI model can be packaged,
inspected, verified against its Python baseline, and executed through portable
runtime surfaces:

- CLI tools
- browser-ready WASM/WebGPU
- server-side Rust usage
- agent-friendly machine-readable interfaces

The long-term goal is not only model format conversion. bio-rs should also make
the surrounding bio/chem tooling portable: the practical pieces currently often
handled by Python libraries such as BioPython and RDKit.

The performance and cost targets are benchmark goals, not current claims:
serverless and local-first deployment, lower infrastructure cost, and faster
execution where Rust/WASM/WebGPU can remove Python runtime overhead.

## Current Seed Module

The first bio-rs module validates protein FASTA input and tokenizes FASTA
records into stable `protein-20` token ids.

## Current Features

- FASTA parsing for one or more protein sequences
- `protein-20` residue validation
- lowercase sequence normalization
- ambiguous residue reporting for `X`, `B`, `Z`, `J`, `U`, and `O`
- invalid residue reporting
- JSON array output from the CLI

## Roadmap

bio-rs will grow through focused open-source releases instead of trying to ship
the whole platform at once.

- `0.6.0`: Reposition the project around bio-AI model migration while keeping
  the protein FASTA module as the working seed.
- `0.7.0`: Define a portable model package manifest for model artifacts,
  preprocessing, postprocessing, runtime backend, fixtures, and expected
  outputs.
- `0.8.0`: Build the first runtime bridge, with ONNX/WebGPU as the first backend
  candidate, and verify a small bio/protein model against its Python output.
- `0.9.0`: Add the first bio/chem foundation mappings needed by real migration
  pipelines, starting from practical BioPython/RDKit-style preprocessing gaps.
- `0.10.0+`: Start the heavyweight model track for ESMFold/AlphaFold-class
  systems by decomposing data pipelines, feature extraction, inference graphs,
  custom kernels, memory layout, and tensor movement.

AlphaFold-class migration is a lighthouse track, not an immediate claim. Some
models have separate licensing, parameter access, database, and hardware gates,
so bio-rs treats them as staged migration targets with explicit verification.

## Not Yet

bio-rs does not yet provide a full model migration engine, a browser AlphaFold
runtime, or a Rust replacement for all BioPython/RDKit functionality. Those are
the milestones this repository is moving toward.

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
