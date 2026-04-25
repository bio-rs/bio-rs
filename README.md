# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Rust workspace for practical biological AI input tooling.

> Status: **v0.8.1** (workspace/package version in `Cargo.toml`)

This repository focuses on functionality that is already implemented and testable today:

- FASTA parsing (`parse_fasta_records`)
- protein-20 tokenization (`tokenize_fasta_records`)
- package manifest inspect/validate/bridge planning
- fixture verification (`package verify`)

## What exists in v0.8.1

### Core (`biors-core`)

`biors-core` is the engine crate. It contains data contracts and pure Rust logic:

- FASTA record parsing and normalization
- protein-20 tokenization and residue issue reporting
- package manifest structs + validation/inspection
- runtime bridge planning report generation
- fixture verification report generation

Use this crate when embedding bio-rs in Rust services, libraries, or tooling.

### CLI (`biors`)

`biors` is the command-line surface built on top of `biors-core`.

- Reads FASTA/JSON files (or stdin for FASTA)
- Executes core workflows
- Emits machine-readable JSON output
- Uses non-zero exit codes on invalid operations

Use this crate when you need shell-first workflows, scripting, or CI checks.

## Release history and roadmap

### Delivered

- `0.6.0`: package manifest inspect/validate
- `0.7.0`: runtime bridge planning (`package bridge`)
- `0.8.0`: fixture verification (`package verify`)
- `0.8.1`: documentation, contribution guide, and benchmark baseline hardening

### Next (post-0.8)

- `0.9.x` target: expand fixtures and verification ergonomics (larger fixture sets, clearer failure diagnostics)
- `1.0.0` target: stable contracts and runtime-facing APIs after enough real-world package validation

`0.7.0` capability notes are kept only as release history above; all "current" descriptions in this README are aligned to **0.8.1**.

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

Inspect a portable model package manifest:

```bash
cargo run -p biors -- package inspect examples/protein-package/manifest.json
```

Validate a portable model package manifest:

```bash
cargo run -p biors -- package validate examples/protein-package/manifest.json
```

Plan the portable runtime bridge for a package:

```bash
cargo run -p biors -- package bridge examples/protein-package/manifest.json
```

Verify package fixture observations:

```bash
cargo run -p biors -- package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

## Evidence and benchmarks

Performance claims should be backed by reproducible data in-repo.

- Benchmark guide and latest recorded result: `benchmarks/fasta_vs_biopython.md`
- Reproducible benchmark harness: `scripts/benchmark_fasta_vs_biopython.py`

The benchmark currently compares FASTA parse+tokenization throughput against a
Biopython baseline over generated synthetic protein FASTA inputs.

## JSON contracts

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

`package validate` always emits a validation report:

```json
{
  "valid": true,
  "issues": []
}
```

`package bridge` always emits a runtime bridge report:

```json
{
  "ready": true,
  "backend": "onnx-webgpu",
  "target": "browser-wasm-webgpu",
  "execution_provider": "webgpu",
  "blocking_issues": []
}
```

`package verify` always emits a fixture verification report:

```json
{
  "package": "protein-seed",
  "fixtures": 1,
  "passed": 1,
  "failed": 0,
  "results": [
    {
      "name": "tiny-protein",
      "input": "fixtures/tiny.fasta",
      "expected_output": "fixtures/tiny.output.json",
      "observed_output": "fixtures/tiny.output.json",
      "status": "passed",
      "issue": null
    }
  ]
}
```

## Development checks

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
    biors-core/  Core engine + contracts
examples/
  multi.fasta
  protein-package/
    fixtures/
    observations.json
  protein.fasta
```

## Protein-20 alphabet

```txt
A C D E F G H I K L M N P Q R S T V W Y
```

Token ids follow that order, starting at `0`.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for local setup, checks, and PR expectations.

## License

Dual licensed under MIT OR Apache-2.0.
