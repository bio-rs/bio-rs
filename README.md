# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Open source Rust/WASM tools for biological AI models.

Python is where many bio-AI models are born. bio-rs is where the tooling around
them becomes portable, inspectable, and easier to use from CLIs, browsers,
servers, and agents.

bio-rs is starting with small, production-quality building blocks for biological
AI model migration. The current release provides a protein FASTA seed module and
a portable package manifest that can describe model artifacts, preprocessing,
postprocessing, runtime targets, fixtures, and expected outputs.

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

Performance and cost improvements are benchmark targets, not current claims.

## Current Modules

### Protein FASTA

The protein FASTA seed module validates protein FASTA input and tokenizes FASTA
records into stable `protein-20` token ids.

### Package Manifest

The package manifest module describes a portable biological AI model package:

- model artifact format and path
- preprocessing and postprocessing steps
- runtime backend and target
- parity fixtures and expected outputs

`biors package inspect` emits a compact manifest summary. `biors package
validate` emits a machine-readable validation report and exits non-zero when the
manifest is incomplete.

### Runtime Bridge Plan

The runtime bridge planner checks whether a package manifest targets a supported
portable runtime. In `0.7.0`, the supported bridge is ONNX/WebGPU for browser
WASM/WebGPU execution planning.

### Verification Harness

The verification harness compares package fixtures against observed runtime
outputs. This gives each migration package a small parity report before the
project grows into full Python-baseline execution and benchmark automation.

## Current Features

- FASTA parsing for one or more protein sequences
- `protein-20` residue validation
- lowercase sequence normalization
- ambiguous residue reporting for `X`, `B`, `Z`, `J`, `U`, and `O`
- invalid residue reporting
- JSON array output from the CLI
- portable model package manifest structs in `biors-core`
- package manifest inspection and validation from the CLI
- runtime bridge planning for ONNX/WebGPU browser targets
- fixture output verification reports for package parity checks

## Release Path

- `0.6.0`: Portable package manifest inspect/validate.
- `0.7.0`: Runtime bridge planning for ONNX/WebGPU package targets.
- `0.8.0`: Verification harness for Python-baseline parity fixtures.

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

`package inspect` always emits a manifest summary object:

```json
{
  "schema_version": "biors.package.v0",
  "name": "protein-seed",
  "model_format": "onnx",
  "runtime_backend": "onnx-webgpu",
  "runtime_target": "browser-wasm-webgpu",
  "preprocessing_steps": 1,
  "postprocessing_steps": 1,
  "fixtures": 1
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
    biors-core/  FASTA parsing, tokenization, and package contracts
examples/
  multi.fasta
  protein-package/
    fixtures/
    observations.json
  protein.fasta
```

## Protein-20

```txt
A C D E F G H I K L M N P Q R S T V W Y
```

Token ids follow that order, starting at `0`.

## License

Dual licensed under MIT OR Apache-2.0.
