# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![Benchmark](https://img.shields.io/badge/benchmark-UniProt%20FASTA-blue)](benchmarks/fasta_vs_biopython.md)
[![Contracts](https://img.shields.io/badge/contracts-JSON%20v0-blue)](docs/public-contract-1.0-candidates.md)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

bio-rs turns biological sequences into validated, model-ready inputs for bio-AI workflows.

```txt
FASTA -> validated protein sequence -> token ids -> model-ready JSON
```

> Status: **v0.9.0** — CLI and JSON contract freeze candidate phase.

## Why bio-rs?

Most bio-AI models are born in Python, but the tooling around them often needs to run somewhere else:

- local CLIs
- CI pipelines
- servers
- browsers
- agents

bio-rs focuses on the boring but important layer before inference:

- parse biological sequence input
- validate it with structured diagnostics
- tokenize it into stable IDs
- emit machine-readable JSON contracts
- keep preprocessing reproducible outside notebooks

The goal is not to replace Python research workflows.

The goal is to make the input layer around bio-AI models faster, more portable, and easier to trust.

## Quickstart

Tokenize a FASTA file:

```bash
cargo run -p biors -- tokenize examples/protein.fasta
```

Pipe FASTA through stdin:

```bash
printf '>tiny\nACDE\n' | cargo run -p biors -- tokenize -
```

Validate FASTA:

```bash
cargo run -p biors -- fasta validate examples/protein.fasta
```

Verify package fixture outputs:

```bash
cargo run -p biors -- package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

## Proof

bio-rs keeps performance claims tied to reproducible in-repo benchmarks.

Latest recorded FASTA benchmark baseline:

| Workflow | Mean time |
|---|---:|
| `biors tokenize` parse + tokenize + full JSON output | **0.291s** |
| Biopython parse + protein-20 token/count loop | **0.494s** |
| Biopython parse only | **0.055s** |

Benchmark details:

- Dataset: UniProt human reference proteome
- Proteome ID: `UP000005640`
- Taxonomy ID: `9606`
- Shape: 20,659 records, 11,456,702 residues
- Benchmark doc: [benchmarks/fasta_vs_biopython.md](benchmarks/fasta_vs_biopython.md)
- Benchmark script: [scripts/benchmark_fasta_vs_biopython.py](scripts/benchmark_fasta_vs_biopython.py)

This is a workload-specific baseline, not a broad claim that bio-rs is faster than Biopython across all FASTA workloads.

## What works today

`biors-core` provides the Rust engine and data contracts.

`biors` provides the CLI surface.

Current v0.9.0 capabilities:

- FASTA parsing and normalization
- FASTA validation with line and record-index diagnostics
- protein-20 tokenization
- residue warning/error reporting
- model-ready input records
- attention masks
- padding/truncation policy
- package manifest inspect/validate
- runtime bridge planning reports
- package fixture verification
- JSON success/error envelopes

## CLI examples

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

Validate FASTA records:

```bash
cargo run -p biors -- fasta validate examples/protein.fasta
```

Emit structured JSON errors:

```bash
printf 'ACDE\n' | cargo run -p biors -- --json tokenize -
```

Inspect a package manifest:

```bash
cargo run -p biors -- package inspect examples/protein-package/manifest.json
```

Validate a package manifest:

```bash
cargo run -p biors -- package validate examples/protein-package/manifest.json
```

Plan a runtime bridge from a package manifest:

```bash
cargo run -p biors -- package bridge examples/protein-package/manifest.json
```

Verify package fixture observations:

```bash
cargo run -p biors -- package verify \
  examples/protein-package/manifest.json \
  examples/protein-package/observations.json
```

## JSON contracts

Success output uses a stable envelope shape:

```json
{
  "ok": true,
  "biors_version": "0.9.0",
  "input_hash": "fnv1a64:846a502e5067bc21",
  "data": {}
}
```

`--json` error mode emits structured errors:

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

Tokenization output is record-oriented:

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

Public contract docs:

- [CLI contract](docs/cli-contract.md)
- [Error code registry](docs/error-codes.md)
- [1.0 contract candidates](docs/public-contract-1.0-candidates.md)
- [JSON schemas](schemas)

## Release history

Delivered:

- `0.6.0`: package manifest inspect/validate
- `0.7.0`: runtime bridge planning with `package bridge`
- `0.8.0`: fixture verification with `package verify`
- `0.8.1`: documentation, contribution guide, and benchmark baseline hardening
- `0.9.0`: CLI and JSON contract freeze candidates

Next:

- `1.0.0`: stable public contracts and runtime-facing APIs after enough real-world package validation

## Not yet

These are roadmap directions, not current capabilities:

- hosted web workflows
- Python bindings
- model inference backends
- package registry or plugin ecosystem
- general-purpose chemistry tooling
- structure tooling
- no-code or low-code workflows

## Development

Run checks:

```bash
scripts/check.sh
```

The check suite runs:

- `cargo fmt`
- Rust checks
- `biors-core` `wasm32-unknown-unknown` build check
- tests
- `cargo clippy` with warnings denied

Reproduce the FASTA benchmark:

```bash
cargo build --release -p biors
pip3 install biopython
python3 scripts/benchmark_fasta_vs_biopython.py
cat benchmarks/fasta_vs_biopython.json
```

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
  protein.fasta
  multi.fasta
  protein-package/
    manifest.json
    observations.json
    fixtures/
```

## Protein-20 alphabet

```txt
A C D E F G H I K L M N P Q R S T V W Y
```

Token IDs follow that order, starting at `0`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for local setup, checks, and PR expectations.

## License

Dual licensed under MIT OR Apache-2.0.