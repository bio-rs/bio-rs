# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Rust workspace for practical biological AI input tooling.

> Status: **v0.9.0** (workspace/package version in `Cargo.toml`)

This repository focuses on functionality that is already implemented and testable today:

- FASTA parsing (`parse_fasta_records`)
- protein-20 tokenization (`tokenize_fasta_records`)
- FASTA validation (`biors fasta validate`)
- model-ready input shaping (`ModelInput`)
- package manifest inspect/validate/bridge planning
- fixture verification (`package verify`)
- frozen CLI JSON success/error envelope candidates

## What exists in v0.9.0

### Core (`biors-core`)

`biors-core` is the engine crate. It contains data contracts and pure Rust logic:

- FASTA record parsing and normalization
- FASTA validation with line and record-index diagnostics
- protein-20 tokenization and residue issue reporting
- model-ready input records with attention masks and padding/truncation policy
- package manifest structs + validation/inspection
- runtime bridge planning report generation
- fixture verification report generation

Use this crate when embedding bio-rs in Rust services, libraries, or tooling.

### CLI (`biors`)

`biors` is the command-line surface built on top of `biors-core`.

- Reads FASTA/JSON files (or stdin for FASTA)
- Executes core workflows
- Emits machine-readable JSON success envelopes
- Supports JSON error mode with structured error codes
- Uses non-zero exit codes on invalid operations

Use this crate when you need shell-first workflows, scripting, or CI checks.

## Release history and roadmap

### Delivered

- `0.6.0`: package manifest inspect/validate
- `0.7.0`: runtime bridge planning (`package bridge`)
- `0.8.0`: fixture verification (`package verify`)
- `0.8.1`: documentation, contribution guide, and benchmark baseline hardening
- `0.9.0`: CLI and JSON contract freeze candidates

### Next (post-0.9)

- `1.0.0` target: stable contracts and runtime-facing APIs after enough real-world package validation

`0.7.0` capability notes are kept only as release history above; all "current" descriptions in this README are aligned to **0.9.0**.

### Not yet

These are roadmap directions, not current capabilities:

- hosted web workflows
- Python bindings
- model inference backends
- package registry or plugin ecosystem
- general-purpose chemistry or structure tooling

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

Validate FASTA records:

```bash
cargo run -p biors -- fasta validate examples/protein.fasta
```

Emit structured JSON errors:

```bash
printf 'ACDE\n' | cargo run -p biors -- --json tokenize -
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

## Proof asset

This is the smallest reproducible package verification example in the repository.

Command:

```bash
cargo run -p biors -- package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

Input:

- package manifest: `examples/protein-package/manifest.json`
- observed fixture map: `examples/protein-package/observations.json`
- expected output fixture: `examples/protein-package/fixtures/tiny.output.json`

Output shape:

```json
{
  "ok": true,
  "biors_version": "0.9.0",
  "data": {
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
}
```

This proves that a portable package manifest can point to fixture inputs and
expected JSON outputs, and that `biors` can check observed outputs against that
contract. It is a small contract test, not a model inference benchmark.

## Evidence and benchmarks

Performance claims should be backed by reproducible data in-repo.

- Benchmark guide and latest recorded result: `benchmarks/fasta_vs_biopython.md`
- Reproducible benchmark harness: `scripts/benchmark_fasta_vs_biopython.py`

The benchmark compares FASTA parse+tokenization throughput against a
Biopython baseline using the UniProt human reference proteome
(UP000005640 / taxonomy 9606).

On the latest recorded run, `biors tokenize` completed the FASTA parse +
protein-20 tokenization + full JSON output path in 0.291s, while a Biopython
parse + protein-20 token/count baseline took 0.494s.

This is a workload-specific baseline, not a broad claim that bio-rs is faster
than Biopython across all FASTA parsing workloads.

## JSON contracts

CLI success output always uses the success envelope:

```json
{
  "ok": true,
  "biors_version": "0.9.0",
  "input_hash": "fnv1a64:846a502e5067bc21",
  "data": {}
}
```

`--json` error mode always emits:

```json
{
  "ok": false,
  "error": {
    "code": "fasta.missing_header",
    "message": "FASTA input must start with a header line beginning with '>' at line 1",
    "location": {
      "line": 1,
      "record_index": null
    }
  }
}
```

`tokenize` data is an array of records:

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

`inspect` data is a summary object:

```json
{
  "records": 1,
  "total_length": 4,
  "valid_records": 1,
  "warning_count": 0,
  "error_count": 0
}
```

`package validate` data is a validation report:

```json
{
  "valid": true,
  "issues": []
}
```

`package bridge` data is a runtime bridge report:

```json
{
  "ready": true,
  "backend": "onnx-webgpu",
  "target": "browser-wasm-webgpu",
  "execution_provider": "webgpu",
  "blocking_issues": []
}
```

`package verify` data is a fixture verification report:

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
schemas/
  cli-error.v0.json
  cli-success.v0.json
  inspect-output.v0.json
  package-manifest.v0.json
  package-validation-report.v0.json
  tokenize-output.v0.json
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

## Public contracts

- CLI contract: [`docs/cli-contract.md`](docs/cli-contract.md)
- Error code registry: [`docs/error-codes.md`](docs/error-codes.md)
- 1.0 candidates: [`docs/public-contract-1.0-candidates.md`](docs/public-contract-1.0-candidates.md)
- Security policy: [`SECURITY.md`](SECURITY.md)

## License

Dual licensed under MIT OR Apache-2.0.
